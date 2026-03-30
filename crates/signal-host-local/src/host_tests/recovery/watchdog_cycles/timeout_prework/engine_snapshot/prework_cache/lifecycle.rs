use super::super::super::super::super::*;

pub(super) fn assert_timeout_prework_cache_lifecycle(
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
}
