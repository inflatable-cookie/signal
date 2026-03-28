use super::*;

pub(super) fn assert_lease_rollover_summary(
    summary: &ServerRuntimeHostSummary,
    supervisor: &signal_runtime::RuntimeSupervisorReport,
) {
    assert_eq!(summary.execution.processing_epoch, 4);
    assert_eq!(summary.execution.restart_count, 3);
    assert_eq!(summary.execution.teardown_count, 3);
    assert_eq!(
        summary.execution.last_recovery_intent,
        Some(RecoveryRestartIntent::WatchdogRecovery)
    );
    assert_eq!(
        summary.execution.last_stop_reason,
        Some(StopReason::DegradedModeRecovery)
    );
    assert_eq!(summary.execution.processed_blocks, 12);
    assert_eq!(summary.execution.last_block_sequence, 17);
    assert_eq!(summary.faults.heartbeat_misses, 6);
    assert_eq!(
        supervisor
            .observation
            .supervision_snapshot
            .watchdog_restart_count,
        3
    );
    assert!(
        supervisor
            .observation
            .supervision_snapshot
            .safe_mode_enabled
    );
    assert!(summary.transport.shared_memory_lease_id.contains("epoch-4"));
    assert_eq!(summary.last_payload.first_output_sample, Some(17.0));
    assert!(matches!(
        supervisor.observation.readiness,
        signal_runtime::RuntimeReadiness::Degraded { .. }
    ));
    assert_eq!(supervisor.observation.control_snapshot.start_count, 4);
    assert_eq!(supervisor.observation.control_snapshot.stop_count, 3);
    assert_eq!(
        supervisor.observation.control_snapshot.last_stop_reason,
        Some(StopReason::DegradedModeRecovery)
    );
    assert_eq!(supervisor.recovery_event_count(), 3);
}
