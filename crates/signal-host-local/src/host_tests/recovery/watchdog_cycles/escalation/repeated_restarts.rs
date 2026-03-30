use super::super::super::*;

#[test]
fn local_host_enters_safe_mode_after_repeated_watchdog_restarts() {
    let runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
    let mut host = LocalRuntimeHost::new(runtime);
    let summary = host
        .boot_with_escalating_heartbeat_failures()
        .expect("escalating heartbeat recovery boot");
    let supervisor = host.supervisor_report();

    assert_eq!(summary.execution.processing_epoch, 3);
    assert_eq!(summary.execution.restart_count, 2);
    assert_eq!(summary.execution.teardown_count, 2);
    assert_eq!(
        summary.execution.last_recovery_intent,
        Some(RecoveryRestartIntent::WatchdogRecovery)
    );
    assert_eq!(
        summary.execution.last_stop_reason,
        Some(StopReason::DegradedModeRecovery)
    );
    assert_eq!(summary.execution.processed_blocks, 10);
    assert_eq!(
        summary.execution.last_block_sequence, 11,
        "unexpected escalating heartbeat summary: {summary:?}"
    );
    assert_eq!(summary.faults.heartbeat_misses, 4);
    assert!(summary.faults.watchdog_triggered);
    assert_eq!(
        supervisor
            .observation
            .supervision_snapshot
            .watchdog_restart_count,
        2
    );
    assert!(
        supervisor
            .observation
            .supervision_snapshot
            .safe_mode_enabled
    );
    assert!(matches!(
        supervisor.observation.readiness,
        signal_runtime::RuntimeReadiness::Degraded { .. }
    ));
    assert_eq!(supervisor.observation.control_snapshot.start_count, 3);
    assert_eq!(supervisor.observation.control_snapshot.stop_count, 2);
    assert_eq!(
        supervisor.observation.control_snapshot.last_stop_reason,
        Some(StopReason::DegradedModeRecovery)
    );
    assert_runtime_automation_values(
        &supervisor,
        RuntimeAutomationExpectations {
            value_events: 10,
            modulation_events: 10,
            gesture_begin_events: 2,
            gesture_end_events: 8,
            first_value: 2.0 / 7.0,
            last_value: 3.0 / 7.0,
            last_modulation: 0.14,
        },
    );
    assert_runtime_automation_continuity(&supervisor, 2, 3, &[2, 3], 1);
    assert_runtime_sequence_continuity(&supervisor, &[2, 3], 2, 11, 0, 1);
    assert_plugin_dispatch_summary(&summary, &supervisor, 0);
}
