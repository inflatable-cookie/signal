use super::super::super::*;

#[test]
fn local_host_recovers_after_lingering_deferred_teardown_cleanup() {
    let runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
    let mut host = LocalRuntimeHost::new(runtime);
    let summary = host
        .boot_with_recovery_deferred_teardown_then_cleanup()
        .expect("lingering cleanup recovery should succeed");
    let supervisor = host.supervisor_report();

    assert_eq!(summary.execution.processing_epoch, 2);
    assert_eq!(summary.execution.restart_count, 1);
    assert_eq!(
        summary.execution.last_recovery_intent,
        Some(RecoveryRestartIntent::WatchdogRecovery)
    );
    assert_eq!(
        summary.execution.last_stop_reason,
        Some(StopReason::DegradedModeRecovery)
    );
    assert_eq!(supervisor.observation.control_snapshot.start_count, 2);
    assert_eq!(supervisor.observation.control_snapshot.stop_count, 1);
    assert!(supervisor.observation.control_snapshot.running);
    assert_eq!(supervisor.observation.readiness, RuntimeReadiness::Ready);
    assert_eq!(
        supervisor
            .observation
            .diagnostics_snapshot
            .active_plugin_sandboxes,
        1
    );
    assert_eq!(
        supervisor
            .observation
            .transport_concurrency_snapshot
            .current_attached_sessions,
        1
    );
    assert_eq!(
        supervisor
            .observation
            .transport_concurrency_snapshot
            .current_lingering_sessions,
        0
    );
    assert_eq!(
        supervisor
            .observation
            .transport_concurrency_snapshot
            .peak_lingering_sessions,
        2
    );
    assert_eq!(
        supervisor
            .observation
            .transport_concurrency_snapshot
            .current_detach_faulted_sessions,
        0
    );
    assert_eq!(
        supervisor
            .observation
            .transport_concurrency_snapshot
            .active_sessions
            .len(),
        1
    );
    assert_eq!(
        supervisor
            .observation
            .transport_concurrency_snapshot
            .active_sessions[0]
            .state,
        signal_runtime::TransportSessionState::AttachActive
    );
    assert_runtime_automation_continuity(&supervisor, 1, 2, &[1, 2], 1);
    assert_runtime_sequence_continuity(&supervisor, &[1, 2], 0, 9, 0, 1);
}

#[test]
fn local_host_recovers_after_lingering_cleanup_fails_once_more() {
    let runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
    let mut host = LocalRuntimeHost::new(runtime);
    let summary = host
        .boot_with_recovery_deferred_teardown_cleanup_retry()
        .expect("cleanup retry recovery should succeed");
    let supervisor = host.supervisor_report();

    assert_eq!(summary.execution.processing_epoch, 2);
    assert_eq!(summary.execution.restart_count, 1);
    assert_eq!(supervisor.observation.control_snapshot.start_count, 2);
    assert_eq!(supervisor.observation.control_snapshot.stop_count, 1);
    assert!(supervisor.observation.control_snapshot.running);
    assert_eq!(supervisor.observation.readiness, RuntimeReadiness::Ready);
    assert_eq!(
        supervisor
            .observation
            .transport_concurrency_snapshot
            .current_attached_sessions,
        1
    );
    assert_eq!(
        supervisor
            .observation
            .transport_concurrency_snapshot
            .current_lingering_sessions,
        0
    );
    assert_eq!(
        supervisor
            .observation
            .transport_concurrency_snapshot
            .peak_lingering_sessions,
        2
    );
    assert!(supervisor
        .observation
        .observation
        .broker_failure_events
        .iter()
        .any(|failure| {
            failure.stage == BrokerFailureStage::TransportTeardown
                && failure
                    .detail
                    .contains("injected lingering cleanup retry failure")
        }));
    assert_eq!(
        supervisor
            .observation
            .transport_concurrency_snapshot
            .active_sessions[0]
            .state,
        signal_runtime::TransportSessionState::AttachActive
    );
}
