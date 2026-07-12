use super::super::super::types::{
    StretchMixedPhaseAuditPairEvidence as PairEvidence,
    StretchMixedPhaseControlEvidence as ControlEvidence,
    StretchMixedPhaseDistributionDirection as Direction,
    StretchMixedPhaseDistributionReview as Review,
};
use super::super::{hash_f64, hash_u64, HASH_OFFSET};
use super::{controls, perturbed, Kind, FRAMES};

const CUTOFFS: [f64; 5] = [0.0, 0.001, 0.003, 0.01, 0.03];
const RADII: [f64; 5] = [0.125, 0.25, 0.5, 0.75, 1.0];

mod measure;

use measure::{distribute, selection_ratio, Distribution};

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
