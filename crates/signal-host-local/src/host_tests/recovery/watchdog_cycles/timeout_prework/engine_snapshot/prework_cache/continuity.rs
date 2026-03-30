use super::super::super::super::super::*;

pub(super) fn assert_timeout_prework_cache_continuity(
    summary: &LocalRuntimeHostSummary,
    supervisor: &signal_runtime::RuntimeSupervisorReport,
) {
    assert!(supervisor
        .observation
        .engine_block_snapshot
        .last_prework_output_peak
        .is_some());
    assert_eq!(
        supervisor
            .observation
            .engine_block_snapshot
            .last_prework_output_peak,
        supervisor
            .observation
            .engine_block_snapshot
            .last_realtime_input_peak
    );
    assert_eq!(
        supervisor
            .observation
            .engine_block_snapshot
            .last_prework_admission_processing_epoch,
        Some(2)
    );
    assert!(supervisor
        .observation
        .engine_block_snapshot
        .last_prework_admission_block_sequence
        .is_some_and(|sequence| sequence >= summary.execution.last_block_sequence));
    assert!(supervisor
        .observation
        .engine_block_snapshot
        .last_prework_admitted_from_block_sequence
        .is_some_and(|sequence| sequence <= summary.execution.last_block_sequence));
    assert_eq!(
        supervisor
            .observation
            .engine_block_snapshot
            .last_prework_consumption_processing_epoch,
        Some(2)
    );
    assert_eq!(
        supervisor
            .observation
            .engine_block_snapshot
            .last_prework_consumption_block_sequence,
        Some(summary.execution.last_block_sequence)
    );
    assert!(supervisor
        .observation
        .engine_block_snapshot
        .last_prework_consumed_from_block_sequence
        .is_some_and(|sequence| sequence <= summary.execution.last_block_sequence));
    assert!(
        matches!(
            supervisor
                .observation
                .engine_block_snapshot
                .last_prework_retirement_reason,
            Some(signal_runtime::RuntimePreworkRetirementReason::PlanningWindowRevised)
                | Some(signal_runtime::RuntimePreworkRetirementReason::TransportStarted)
                | Some(signal_runtime::RuntimePreworkRetirementReason::TransportStopped)
                | Some(signal_runtime::RuntimePreworkRetirementReason::TransportSeeked)
                | Some(signal_runtime::RuntimePreworkRetirementReason::TransportTempoChanged)
                | Some(signal_runtime::RuntimePreworkRetirementReason::TransportLoopStateChanged)
                | Some(signal_runtime::RuntimePreworkRetirementReason::TransportLoopWrapped)
                | Some(signal_runtime::RuntimePreworkRetirementReason::ParameterBatchApplied)
                | Some(signal_runtime::RuntimePreworkRetirementReason::InputSignatureChanged)
                | Some(signal_runtime::RuntimePreworkRetirementReason::ProcessingEpochExpired)
                | Some(signal_runtime::RuntimePreworkRetirementReason::BlockSequenceExpired)
                | Some(signal_runtime::RuntimePreworkRetirementReason::SupersededByAdmission)
                | Some(signal_runtime::RuntimePreworkRetirementReason::QueueCapacityExceeded)
        ),
        "unexpected prework retirement reason: {:?}",
        supervisor
            .observation
            .engine_block_snapshot
            .last_prework_retirement_reason
    );
    assert!(supervisor
        .observation
        .engine_block_snapshot
        .last_prework_retired_unconsumed
        .is_some());
    assert_eq!(
        supervisor
            .observation
            .engine_block_snapshot
            .prework_cache_valid_until_processing_epoch,
        Some(3)
    );
    assert!(supervisor
        .observation
        .engine_block_snapshot
        .prework_cache_valid_until_block_sequence
        .is_some_and(|sequence| sequence >= summary.execution.last_block_sequence));
    assert!(supervisor
        .observation
        .engine_block_snapshot
        .prework_cache_remaining_valid_blocks
        .is_some_and(|remaining| remaining > 0));
    assert_eq!(
        supervisor
            .observation
            .engine_block_snapshot
            .total_latency_samples,
        24
    );
}
