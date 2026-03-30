use super::super::super::super::*;

pub(super) fn assert_timeout_prework_cache(
    summary: &LocalRuntimeHostSummary,
    supervisor: &signal_runtime::RuntimeSupervisorReport,
) {
    assert!(
        supervisor
            .observation
            .engine_block_snapshot
            .prework_cache_enabled
    );
    assert_eq!(
        supervisor
            .observation
            .engine_block_snapshot
            .prework_forecast_requested_mode,
        signal_runtime::RuntimePreworkForecastMode::RuntimeRoleDefault
    );
    assert_eq!(
        supervisor
            .observation
            .engine_block_snapshot
            .prework_forecast_mode,
        signal_runtime::RuntimePreworkForecastMode::RuntimeRoleDefault
    );
    assert_eq!(
        supervisor
            .observation
            .engine_block_snapshot
            .prework_forecast_profile,
        Some(signal_runtime::RuntimePreworkForecastProfile::Local)
    );
    assert_eq!(
        supervisor
            .observation
            .engine_block_snapshot
            .prework_forecast_profile_source,
        Some(signal_runtime::RuntimePreworkForecastProfileSource::RuntimeRoleDefault)
    );
    assert_eq!(
        supervisor
            .observation
            .engine_block_snapshot
            .prework_forecast_policy_target_window_blocks,
        Some(2)
    );
    assert_eq!(
        supervisor
            .observation
            .engine_block_snapshot
            .prework_service_pressure,
        signal_runtime::RuntimePreworkServicePressure::Elevated
    );
    assert!(
        supervisor
            .observation
            .engine_block_snapshot
            .prework_service_yield_count
            >= 1
    );
    assert!(
        supervisor
            .observation
            .engine_block_snapshot
            .prework_service_throttle_count
            >= 1
    );
    assert!(matches!(
        supervisor
            .observation
            .engine_block_snapshot
            .prework_cache_state,
        signal_runtime::RuntimePreworkCacheState::Consumed
            | signal_runtime::RuntimePreworkCacheState::Admitted
    ));
    assert_eq!(
        supervisor
            .observation
            .engine_block_snapshot
            .prework_cache_freshness_state,
        signal_runtime::RuntimePreworkFreshnessState::Fresh
    );
    assert_eq!(
        supervisor
            .observation
            .engine_block_snapshot
            .prework_cache_queue_capacity,
        3
    );
    assert!(
        supervisor
            .observation
            .engine_block_snapshot
            .prework_cache_queue_depth
            > 0
    );
    assert!(
        supervisor
            .observation
            .engine_block_snapshot
            .prework_cache_queue_depth
            <= 3
    );
    assert_eq!(
        supervisor
            .observation
            .engine_block_snapshot
            .prework_cache_peak_queue_depth,
        3
    );
    assert_eq!(
        supervisor
            .observation
            .engine_block_snapshot
            .prework_cache_window_target_count,
        3
    );
    assert_eq!(
        supervisor
            .observation
            .engine_block_snapshot
            .prework_cache_window_target_block_sequences,
        vec![
            summary.execution.last_block_sequence,
            summary.execution.last_block_sequence + 1,
            summary.execution.last_block_sequence + 2,
        ]
    );
    let engine_snapshot = &supervisor.observation.engine_block_snapshot;
    assert!(engine_snapshot.prework_cache_admissions >= engine_snapshot.prework_cache_consumptions);
    assert!(
        engine_snapshot.prework_cache_queued_admissions
            >= engine_snapshot.prework_cache_window_target_count as u64
    );
    assert!(
        engine_snapshot.prework_cache_queued_consumptions
            <= engine_snapshot.prework_cache_consumptions
    );
    assert_eq!(
        engine_snapshot.prework_cache_retirement_count,
        engine_snapshot.prework_cache_unconsumed_retirement_count
            + engine_snapshot.prework_cache_consumed_retirement_count
    );
    assert!(engine_snapshot.prework_cache_retirement_count > 0);
    assert_eq!(
        engine_snapshot.prework_cache_hits + engine_snapshot.prework_cache_misses,
        engine_snapshot.prework_cache_consumptions
    );
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
