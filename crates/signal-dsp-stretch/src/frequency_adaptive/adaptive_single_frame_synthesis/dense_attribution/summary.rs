use super::super::super::HASH_OFFSET;
use super::evidence::{DenseMode, RowEvidence, Stage};
use super::{DenseAttributionDirection, DenseAttributionReview, RATIOS};

pub(super) fn summarize(rows: Vec<RowEvidence>) -> DenseAttributionReview {
    let failing_rows = rows.iter().filter(|row| row.hard_failure).count();
    let mut stage_counts = [0; 5];
    for row in rows.iter().filter(|row| row.hard_failure) {
        let index = match row.stage {
            Stage::AnchorPlacement => 0,
            Stage::EventReset => 1,
            Stage::ActiveOwnerTransport => 2,
            Stage::OverlapSynthesis => 3,
            Stage::MetricAssociation => 4,
            Stage::PassingControl => continue,
        };
        stage_counts[index] += 1;
    }
    let maximum_errors = [DenseMode::Ordinary, DenseMode::Successor].map(|mode| {
        rows.iter()
            .filter(|row| row.mode == mode)
            .flat_map(|row| row.errors)
            .max()
            .unwrap_or(0)
    });
    let row_errors = [DenseMode::Ordinary, DenseMode::Successor].map(|mode| {
        std::array::from_fn(|index| {
            rows.iter()
                .find(|row| row.mode == mode && row.ratio == RATIOS[index])
                .expect("dense ratio row")
                .errors
        })
    });
    let successor_events = rows
        .iter()
        .filter(|row| row.mode == DenseMode::Successor)
        .flat_map(|row| row.events.iter())
        .collect::<Vec<_>>();
    let anchor_failures = successor_events
        .iter()
        .filter(|event| !event.attached)
        .count();
    let reset_failures = successor_events
        .iter()
        .filter(|event| !event.event_assignment)
        .count();
    let owner_failures = successor_events
        .iter()
        .filter(|event| event.active_state_hash == 0 || event.owner_counts[3] == 0)
        .count();
    let maximum_closure_error = [0, 1].map(|index| {
        rows.iter()
            .flat_map(|row| row.events.iter())
            .map(|event| event.closure_error[index])
            .fold(0.0_f64, f64::max)
    });
    let maximum_cancellation_ratio = rows
        .iter()
        .flat_map(|row| row.events.iter())
        .map(|event| event.cancellation_ratio)
        .fold(0.0_f64, f64::max);
    let traced_contributions = rows
        .iter()
        .flat_map(|row| row.events.iter())
        .map(|event| event.contributions.len())
        .sum();
    let failure = rows
        .iter()
        .find(|row| row.hard_failure)
        .expect("frozen dense failure");
    let replica_contributions = failure.peak_contributions[1].clone();
    let target_contributions = failure
        .events
        .each_ref()
        .map(|event| event.contributions.clone());
    let mut evidence_hash = HASH_OFFSET;
    for row in &rows {
        hash(&mut evidence_hash, row.hashes[7]);
    }
    DenseAttributionReview {
        row_count: rows.len(),
        failing_rows,
        stage_counts,
        maximum_errors,
        row_errors,
        anchor_failures,
        reset_failures,
        owner_failures,
        maximum_closure_error,
        maximum_cancellation_ratio,
        traced_contributions,
        failure_targets: failure.events.each_ref().map(|event| event.target),
        failure_peaks: failure.events.each_ref().map(|event| event.actual_peak),
        failure_target_values: failure.events.each_ref().map(|event| event.target_value),
        failure_peak_values: failure.events.each_ref().map(|event| event.peak_value),
        failure_local_peaks: failure.events.each_ref().map(|event| event.local_peaks),
        target_contributions,
        replica_contributions,
        evidence_hash,
        direction: direction(failure.stage),
        rows,
    }
}

fn direction(stage: Stage) -> DenseAttributionDirection {
    match stage {
        Stage::PassingControl => DenseAttributionDirection::SuccessorSyntheticQualityGate,
        Stage::AnchorPlacement => DenseAttributionDirection::AnchorPlacementRedesign,
        Stage::EventReset => DenseAttributionDirection::EventResetRedesign,
        Stage::ActiveOwnerTransport => DenseAttributionDirection::ActiveOwnerTransportRedesign,
        Stage::OverlapSynthesis => DenseAttributionDirection::OverlapSynthesisRedesign,
        Stage::MetricAssociation => DenseAttributionDirection::MetricAssociationReview,
    }
}

fn hash(state: &mut u64, value: u64) {
    *state = (*state ^ value).wrapping_mul(0x100_0000_01b3);
}
