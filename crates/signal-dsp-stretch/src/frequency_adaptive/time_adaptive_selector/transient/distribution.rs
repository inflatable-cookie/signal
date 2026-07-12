use super::super::super::types::{
    StretchMixedPhaseAuditPairEvidence as PairEvidence,
    StretchMixedPhaseBandEvidence as BandEvidence,
    StretchMixedPhaseControlEvidence as ControlEvidence,
    StretchMixedPhaseDistributionDirection as Direction,
    StretchMixedPhaseDistributionReview as Review,
};
use super::super::{hash_f64, hash_u64, input_hash, ANCHOR_HOP, HASH_OFFSET};
use super::measure::{spectra, MIXED_SCALE};
use super::{controls, perturbed, Kind, FRAMES};

const BINS: std::ops::RangeInclusive<usize> = 1..=2046;
const BAND_EDGES: [f64; 4] = [0.001, 0.003, 0.01, 0.03];
const CUTOFFS: [f64; 5] = [0.0, 0.001, 0.003, 0.01, 0.03];
const RADII: [f64; 5] = [0.125, 0.25, 0.5, 0.75, 1.0];
const QUANTILES: [f64; 9] = [0.0, 0.01, 0.05, 0.25, 0.5, 0.75, 0.95, 0.99, 1.0];

#[derive(Clone)]
struct Cell {
    normalized_magnitude: f64,
    mixed_phase: f64,
    magnitude: f64,
    event: bool,
}

struct Distribution {
    evidence: ControlEvidence,
    cells: Vec<Cell>,
    signature: Vec<f64>,
}

pub(crate) fn mixed_phase_distribution_review() -> Review {
    let source = controls();
    let mut distributions = Vec::with_capacity(source.len() * 2);
    let mut failures = [0; 4];
    let mut maximum_equivalence_error = 0.0_f64;
    let mut equivalence_errors = Vec::with_capacity(source.len());
    for (index, control) in source.iter().enumerate() {
        let base = distribute(index, false, &[control.samples.as_slice()]);
        accumulate_failures(&base.evidence, &mut failures);
        let mut control_equivalence_error = 0.0_f64;
        if control.kind != Kind::Silence {
            for scale in [0.25, 4.0, -1.0] {
                let variant = control
                    .samples
                    .iter()
                    .map(|sample| sample * scale)
                    .collect::<Vec<_>>();
                compare_equivalent(
                    &base,
                    &distribute(index, false, &[variant.as_slice()]),
                    &mut failures,
                    &mut maximum_equivalence_error,
                    &mut control_equivalence_error,
                );
            }
        }
        let silence = vec![0.0; FRAMES];
        let split = control
            .samples
            .iter()
            .map(|sample| sample * std::f64::consts::FRAC_1_SQRT_2)
            .collect::<Vec<_>>();
        for variant in [
            distribute(
                index,
                false,
                &[control.samples.as_slice(), silence.as_slice()],
            ),
            distribute(
                index,
                false,
                &[silence.as_slice(), control.samples.as_slice()],
            ),
            distribute(index, false, &[split.as_slice(), split.as_slice()]),
        ] {
            compare_equivalent(
                &base,
                &variant,
                &mut failures,
                &mut maximum_equivalence_error,
                &mut control_equivalence_error,
            );
        }
        let changed_samples = perturbed(&control.samples);
        let changed = distribute(index, true, &[changed_samples.as_slice()]);
        accumulate_failures(&changed.evidence, &mut failures);
        distributions.push(base);
        distributions.push(changed);
        equivalence_errors.push(control_equivalence_error);
    }
    let audit_pairs = audit(&source, &distributions);
    let direction = if failures != [0; 4] {
        Direction::StructuralFailure
    } else if audit_pairs.iter().any(|pair| pair.separates) {
        Direction::Calibratable
    } else {
        Direction::Overlapping
    };
    let mut review = Review {
        controls: distributions
            .iter()
            .map(|distribution| distribution.evidence.clone())
            .collect(),
        audit_pairs,
        structural_failures: failures,
        maximum_equivalence_error,
        equivalence_errors,
        evidence_hash: 0,
        direction,
    };
    review.evidence_hash = review_hash(&review);
    review
}

fn distribute(control: usize, perturbed: bool, channels: &[&[f64]]) -> Distribution {
    let (spectra, reflected_reads) = spectra(channels);
    let mut cells = Vec::new();
    let mut nonzero_cells = 0;
    let mut non_finite = 0;
    for anchor in (0..FRAMES).step_by(ANCHOR_HOP) {
        let frame = anchor / ANCHOR_HOP + 2;
        for channel in 0..channels.len() {
            let current = &spectra[channel][frame];
            let before = &spectra[channel][frame - 1];
            let after = &spectra[channel][frame + 1];
            let energy = BINS.clone().map(|bin| current[bin].norm_sqr()).sum::<f64>();
            if energy == 0.0 {
                continue;
            }
            let norm = energy.sqrt();
            for bin in BINS.clone() {
                let magnitude = current[bin].norm();
                if magnitude == 0.0 {
                    continue;
                }
                nonzero_cells += 1;
                let cross =
                    after[bin + 1] * before[bin + 1].conj() * after[bin].conj() * before[bin];
                let cell = Cell {
                    normalized_magnitude: magnitude / norm,
                    mixed_phase: cross.arg() / MIXED_SCALE,
                    magnitude,
                    event: event_anchor(control, anchor),
                };
                non_finite += usize::from(
                    !cell.normalized_magnitude.is_finite()
                        || !cell.mixed_phase.is_finite()
                        || !cell.magnitude.is_finite(),
                );
                if cell.normalized_magnitude.is_finite()
                    && cell.mixed_phase.is_finite()
                    && cell.magnitude.is_finite()
                {
                    cells.push(cell);
                }
            }
        }
    }
    let bands = summarize(&cells);
    let signature = signature(&cells, &bands);
    let mut evidence = ControlEvidence {
        control,
        perturbed,
        bands,
        structural_counts: [nonzero_cells, cells.len(), reflected_reads, non_finite],
        hashes: [input_hash(channels), 0],
    };
    evidence.hashes[1] = control_hash(&evidence);
    Distribution {
        evidence,
        cells,
        signature,
    }
}

fn summarize(cells: &[Cell]) -> Vec<BandEvidence> {
    let mut evidence = Vec::with_capacity(10);
    for event in [false, true] {
        for band in 0..5 {
            let selected = cells
                .iter()
                .filter(|cell| cell.event == event && band_index(cell.normalized_magnitude) == band)
                .collect::<Vec<_>>();
            let mut phases = selected
                .iter()
                .map(|cell| cell.mixed_phase)
                .collect::<Vec<_>>();
            phases.sort_by(f64::total_cmp);
            evidence.push(BandEvidence {
                band,
                event,
                cell_count: selected.len(),
                magnitude_sum: selected.iter().map(|cell| cell.magnitude).sum(),
                quantiles: quantiles(&phases),
            });
        }
    }
    evidence
}

fn signature(cells: &[Cell], bands: &[BandEvidence]) -> Vec<f64> {
    let count = cells.len() as f64;
    let magnitude = cells.iter().map(|cell| cell.magnitude).sum::<f64>();
    let mut values = Vec::with_capacity(45);
    for band in bands {
        values.push(if count == 0.0 {
            0.0
        } else {
            band.cell_count as f64 / count
        });
        values.push(if magnitude == 0.0 {
            0.0
        } else {
            band.magnitude_sum / magnitude
        });
    }
    for cutoff in CUTOFFS {
        for radius in RADII {
            values.push(selection_ratio(cells, cutoff, radius, |_| true));
        }
    }
    values
}

fn audit(
    source: &[super::super::controls::Control],
    distributions: &[Distribution],
) -> Vec<PairEvidence> {
    let mut pairs = Vec::with_capacity(25);
    for cutoff in CUTOFFS {
        for radius in RADII {
            let event_recall = [
                Kind::Impulse,
                Kind::DenseImpulses,
                Kind::BoundaryImpulses,
                Kind::Mixed,
            ]
            .map(|kind| family_extreme(source, distributions, kind, cutoff, radius, true));
            let negative_leakage = [Kind::Steady, Kind::Chirp, Kind::Noise]
                .map(|kind| family_extreme(source, distributions, kind, cutoff, radius, false));
            let separates = event_recall.iter().all(|value| *value >= 0.5)
                && negative_leakage.iter().all(|value| *value <= 0.01);
            pairs.push(PairEvidence {
                magnitude_cutoff: cutoff,
                mixed_phase_radius: radius,
                event_recall,
                negative_leakage,
                separates,
            });
        }
    }
    pairs
}

fn family_extreme(
    source: &[super::super::controls::Control],
    distributions: &[Distribution],
    kind: Kind,
    cutoff: f64,
    radius: f64,
    event: bool,
) -> f64 {
    let values = distributions
        .iter()
        .filter(|distribution| source[distribution.evidence.control].kind == kind)
        .map(|distribution| {
            selection_ratio(&distribution.cells, cutoff, radius, |cell| {
                !event || cell.event
            })
        });
    if event {
        values.fold(1.0, f64::min)
    } else {
        values.fold(0.0, f64::max)
    }
}

fn selection_ratio(
    cells: &[Cell],
    cutoff: f64,
    radius: f64,
    region: impl Fn(&Cell) -> bool,
) -> f64 {
    let denominator = cells
        .iter()
        .filter(|cell| region(cell))
        .map(|cell| cell.magnitude)
        .sum::<f64>();
    if denominator == 0.0 {
        return 0.0;
    }
    cells
        .iter()
        .filter(|cell| {
            region(cell)
                && cell.normalized_magnitude >= cutoff
                && (cell.mixed_phase - 1.0).abs() <= radius
        })
        .map(|cell| cell.magnitude)
        .sum::<f64>()
        / denominator
}

fn compare_equivalent(
    base: &Distribution,
    variant: &Distribution,
    failures: &mut [usize; 4],
    maximum_error: &mut f64,
    control_error: &mut f64,
) {
    let error = base
        .signature
        .iter()
        .zip(&variant.signature)
        .map(|(left, right)| (left - right).abs())
        .fold(0.0, f64::max);
    *maximum_error = maximum_error.max(error);
    *control_error = control_error.max(error);
    failures[3] += usize::from(error > 1.0e-12);
    accumulate_failures(&variant.evidence, failures);
}

fn accumulate_failures(evidence: &ControlEvidence, failures: &mut [usize; 4]) {
    failures[0] += usize::from(evidence.structural_counts[0] != evidence.structural_counts[1]);
    failures[1] += evidence
        .bands
        .iter()
        .filter(|band| band.quantiles.windows(2).any(|pair| pair[0] > pair[1]))
        .count();
    failures[2] += evidence.structural_counts[3];
}

fn event_anchor(control: usize, anchor: usize) -> bool {
    let events: &[usize] = match control {
        5 | 11 => &[FRAMES / 2],
        6 => &[FRAMES / 2 - 128, FRAMES / 2 + 128],
        7 => &[0, FRAMES - 1],
        _ => &[],
    };
    events.iter().any(|event| anchor.abs_diff(*event) <= 256)
}

fn band_index(value: f64) -> usize {
    BAND_EDGES.partition_point(|edge| value >= *edge)
}

fn quantiles(values: &[f64]) -> [f64; 9] {
    if values.is_empty() {
        return [0.0; 9];
    }
    QUANTILES.map(|probability| {
        let index = (probability * (values.len() - 1) as f64).floor() as usize;
        values[index]
    })
}

fn control_hash(evidence: &ControlEvidence) -> u64 {
    let mut hash = HASH_OFFSET;
    hash_u64(&mut hash, evidence.control as u64);
    hash_u64(&mut hash, evidence.perturbed as u64);
    for band in &evidence.bands {
        hash_u64(&mut hash, band.band as u64);
        hash_u64(&mut hash, band.event as u64);
        hash_u64(&mut hash, band.cell_count as u64);
        hash_f64(&mut hash, band.magnitude_sum);
        for value in band.quantiles {
            hash_f64(&mut hash, value);
        }
    }
    for value in evidence.structural_counts {
        hash_u64(&mut hash, value as u64);
    }
    hash
}

fn review_hash(review: &Review) -> u64 {
    let mut hash = HASH_OFFSET;
    for control in &review.controls {
        hash_u64(&mut hash, control.hashes[1]);
    }
    for pair in &review.audit_pairs {
        hash_f64(&mut hash, pair.magnitude_cutoff);
        hash_f64(&mut hash, pair.mixed_phase_radius);
        for value in pair.event_recall {
            hash_f64(&mut hash, value);
        }
        for value in pair.negative_leakage {
            hash_f64(&mut hash, value);
        }
        hash_u64(&mut hash, pair.separates as u64);
    }
    for failure in review.structural_failures {
        hash_u64(&mut hash, failure as u64);
    }
    hash_f64(&mut hash, review.maximum_equivalence_error);
    for error in &review.equivalence_errors {
        hash_f64(&mut hash, *error);
    }
    hash
}
