use super::super::types::{
    StretchTransientAnchorEvidence as AnchorEvidence,
    StretchTransientControlEvidence as ControlEvidence,
    StretchTransientEvidenceDirection as Direction, StretchTransientEvidenceReview as Review,
};
use super::controls::{controls, perturbed, Kind, FRAMES};
use super::{hash_f64, hash_u64, ANCHOR_HOP, HASH_OFFSET};

mod distribution;
mod measure;

pub(crate) use distribution::mixed_phase_distribution_review;

pub(crate) fn transient_evidence_measurement_review() -> Review {
    let controls = controls();
    let mut evidence = Vec::with_capacity(controls.len());
    let mut equivalence_failures = 0;
    let mut maximum_equivalence_error = 0.0_f64;
    let mut maximum_perturbation_change = 0.0_f64;
    let mut maximum_peak_displacement = 0;
    let mut perturbation_changes = Vec::with_capacity(controls.len());
    let mut peak_displacements = Vec::with_capacity(controls.len());
    let mut unmatched_perturbation_peaks = 0;
    let mut equivalence_errors = Vec::with_capacity(controls.len());
    let mut equivalence_peak_failures = 0;
    let mut perturbation_failures = 0;
    for (index, control) in controls.iter().enumerate() {
        let mut base = measure::measure(index, &[control.samples.as_slice()]);
        base.event_offsets = events(index)
            .iter()
            .map(|event| nearest_offset(*event, &base.peaks))
            .collect();
        let mut control_equivalence_error = 0.0_f64;
        let mut perturbation_change = 0.0;
        let mut displacement = Some(0);
        if control.kind != Kind::Silence {
            for scale in [0.25, 4.0, -1.0] {
                let samples = control
                    .samples
                    .iter()
                    .map(|sample| sample * scale)
                    .collect::<Vec<_>>();
                compare_equivalent(
                    &base,
                    &measure::measure(index, &[samples.as_slice()]),
                    &mut equivalence_failures,
                    &mut maximum_equivalence_error,
                    &mut control_equivalence_error,
                    &mut equivalence_peak_failures,
                );
            }
            let changed = measure::measure(index, &[perturbed(&control.samples).as_slice()]);
            perturbation_change = occupancy_change(&base, &changed);
            displacement = peak_displacement(&base.peaks, &changed.peaks);
            maximum_perturbation_change = maximum_perturbation_change.max(perturbation_change);
            if let Some(value) = displacement {
                maximum_peak_displacement = maximum_peak_displacement.max(value);
            } else {
                unmatched_perturbation_peaks += 1;
            }
            perturbation_failures += usize::from(
                perturbation_change > 0.05 || displacement.is_none_or(|value| value > 1),
            );
        }
        let silent = vec![0.0; FRAMES];
        let hard_pan = measure::measure(index, &[control.samples.as_slice(), silent.as_slice()]);
        let swapped = measure::measure(index, &[silent.as_slice(), control.samples.as_slice()]);
        let split = control
            .samples
            .iter()
            .map(|sample| sample * std::f64::consts::FRAC_1_SQRT_2)
            .collect::<Vec<_>>();
        let centered = measure::measure(index, &[split.as_slice(), split.as_slice()]);
        for variant in [&hard_pan, &swapped, &centered] {
            compare_equivalent(
                &base,
                variant,
                &mut equivalence_failures,
                &mut maximum_equivalence_error,
                &mut control_equivalence_error,
                &mut equivalence_peak_failures,
            );
        }
        perturbation_changes.push(perturbation_change);
        peak_displacements.push(displacement);
        equivalence_errors.push(control_equivalence_error);
        evidence.push(base);
    }
    let gate_failures = gates(
        &controls,
        &evidence,
        equivalence_failures,
        perturbation_failures,
    );
    let direction = if gate_failures == [0; 7] {
        Direction::OccupancyMappingContract
    } else {
        Direction::OperatorReview
    };
    let mut review = Review {
        controls: evidence,
        gate_failures,
        maximum_perturbation_change,
        perturbation_changes,
        maximum_peak_displacement,
        peak_displacements,
        unmatched_perturbation_peaks,
        maximum_equivalence_error,
        equivalence_errors,
        equivalence_peak_failures,
        evidence_hash: 0,
        direction,
    };
    review.evidence_hash = review_hash(&review);
    review
}

fn compare_equivalent(
    base: &ControlEvidence,
    variant: &ControlEvidence,
    failures: &mut usize,
    maximum_error: &mut f64,
    control_error: &mut f64,
    peak_failures: &mut usize,
) {
    let error = occupancy_change(base, variant);
    *maximum_error = maximum_error.max(error);
    *control_error = control_error.max(error);
    *peak_failures += usize::from(base.peaks != variant.peaks);
    *failures += usize::from(base.peaks != variant.peaks || error > 1.0e-12);
}

fn gates(
    controls: &[super::controls::Control],
    evidence: &[ControlEvidence],
    equivalence: usize,
    perturbation: usize,
) -> [usize; 7] {
    let mut failures = [0; 7];
    for (control, report) in controls.iter().zip(evidence) {
        match control.kind {
            Kind::Silence => {
                failures[0] += usize::from(
                    !report.peaks.is_empty()
                        || report
                            .anchors
                            .iter()
                            .any(|anchor| anchor.cell_counts != [0, 0] || anchor.occupancy != 0.0),
                );
            }
            Kind::Steady | Kind::Chirp | Kind::Noise => {
                failures[0] += usize::from(!report.peaks.is_empty());
            }
            Kind::Impulse | Kind::BoundaryImpulses => {
                failures[1] += report
                    .event_offsets
                    .iter()
                    .filter(|offset| offset.is_none_or(|value| value > 256))
                    .count();
            }
            Kind::DenseImpulses => {
                failures[2] += usize::from(!dense_gate(&report.peaks));
            }
            Kind::Mixed => {
                failures[3] += usize::from(
                    report.event_offsets[0].is_none_or(|value| value > 256)
                        || report
                            .peaks
                            .iter()
                            .any(|peak| *peak < 2_048 || *peak >= 6_144),
                );
            }
        }
        failures[6] += usize::from(
            report.structural_counts[1] != 0
                || report.hashes.iter().any(|hash| *hash == 0)
                || report.anchors.len() != FRAMES / ANCHOR_HOP,
        );
    }
    failures[4] = equivalence;
    failures[5] = perturbation;
    failures
}

fn events(control: usize) -> Vec<usize> {
    match control {
        5 | 11 => vec![FRAMES / 2],
        6 => vec![FRAMES / 2 - 128, FRAMES / 2 + 128],
        7 => vec![0, FRAMES - 1],
        _ => Vec::new(),
    }
}

fn dense_gate(peaks: &[usize]) -> bool {
    let events = [FRAMES / 2 - 128, FRAMES / 2 + 128];
    peaks.iter().enumerate().any(|(left_index, left)| {
        left.abs_diff(events[0]) <= 128
            && peaks.iter().enumerate().any(|(right_index, right)| {
                right_index != left_index && right.abs_diff(events[1]) <= 128
            })
    })
}

fn nearest_offset(event: usize, peaks: &[usize]) -> Option<usize> {
    peaks.iter().map(|peak| peak.abs_diff(event)).min()
}

fn occupancy_change(left: &ControlEvidence, right: &ControlEvidence) -> f64 {
    left.anchors
        .iter()
        .zip(&right.anchors)
        .map(|(left, right)| (left.occupancy - right.occupancy).abs())
        .fold(0.0, f64::max)
}

fn peak_displacement(left: &[usize], right: &[usize]) -> Option<usize> {
    if left.len() != right.len() {
        return None;
    }
    Some(
        left.iter()
            .zip(right)
            .map(|(left, right)| left.abs_diff(*right) / ANCHOR_HOP)
            .max()
            .unwrap_or(0),
    )
}

fn review_hash(review: &Review) -> u64 {
    let mut hash = HASH_OFFSET;
    for control in &review.controls {
        hash_u64(&mut hash, control.hashes[4]);
    }
    for failure in review.gate_failures {
        hash_u64(&mut hash, failure as u64);
    }
    hash_f64(&mut hash, review.maximum_perturbation_change);
    for value in &review.perturbation_changes {
        hash_f64(&mut hash, *value);
    }
    hash_u64(&mut hash, review.maximum_peak_displacement as u64);
    for value in &review.peak_displacements {
        hash_u64(&mut hash, value.unwrap_or(usize::MAX) as u64);
    }
    hash_u64(&mut hash, review.unmatched_perturbation_peaks as u64);
    hash_f64(&mut hash, review.maximum_equivalence_error);
    for value in &review.equivalence_errors {
        hash_f64(&mut hash, *value);
    }
    hash_u64(&mut hash, review.equivalence_peak_failures as u64);
    hash
}
