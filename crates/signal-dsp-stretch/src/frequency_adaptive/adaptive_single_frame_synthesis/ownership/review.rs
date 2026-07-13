use super::super::super::HASH_OFFSET;
use super::{controls, hash, review_row, OwnershipDirection, OwnershipReview, CONTROLS, RATIOS};

pub(super) fn run(native: bool) -> OwnershipReview {
    let controls = controls();
    let mut rows = Vec::with_capacity(CONTROLS.len() * RATIOS.len());
    for control in CONTROLS {
        let input = &controls
            .iter()
            .find(|(candidate, _)| *candidate == control)
            .expect("ownership control")
            .1;
        for ratio in RATIOS {
            rows.push(review_row(control, input, ratio, native));
        }
    }
    let failure_counts =
        std::array::from_fn(|index| rows.iter().map(|row| row.failures[index]).sum::<usize>());
    let maximum_identity_error = std::array::from_fn(|index| {
        rows.iter()
            .map(|row| row.identity_error[index])
            .fold(0.0_f64, f64::max)
    });
    let maximum_tone_errors = std::array::from_fn(|index| {
        rows.iter()
            .map(|row| row.tone_errors[index])
            .fold(0.0_f64, f64::max)
    });
    let maximum_event_errors = std::array::from_fn(|index| {
        rows.iter()
            .map(|row| row.event_errors[index])
            .max()
            .unwrap_or(0)
    });
    let owner_counts = std::array::from_fn(|index| {
        rows.iter()
            .map(|row| row.owner_counts[index])
            .sum::<usize>()
    });
    let detected_anchors = rows.iter().map(|row| row.detected.len()).sum();
    let expected_anchors = rows.iter().map(|row| row.expected.len()).sum();
    let resolution_transitions = rows.iter().map(|row| row.resolution_transitions).sum();
    let matched_resolution_transitions = rows
        .iter()
        .map(|row| row.matched_resolution_transitions)
        .sum();
    let mut evidence_hash = HASH_OFFSET;
    for row in &rows {
        hash(&mut evidence_hash, row.hashes[8]);
    }
    if native {
        hash(&mut evidence_hash, resolution_transitions as u64);
        hash(&mut evidence_hash, matched_resolution_transitions as u64);
    }
    let pass = failure_counts == [0; 8];
    OwnershipReview {
        rows,
        failure_counts,
        maximum_identity_error,
        maximum_tone_errors,
        maximum_event_errors,
        owner_counts,
        resolution_transitions,
        matched_resolution_transitions,
        detected_anchors,
        expected_anchors,
        evidence_hash,
        direction: if pass {
            OwnershipDirection::SuccessorSyntheticQualityGate
        } else {
            OwnershipDirection::ActivePeakOrTransientAnchorRedesign
        },
    }
}
