use super::super::super::host_test_support::assert_runtime_automation_continuity;
use super::super::super::host_test_support::assert_runtime_sequence_continuity;
use super::super::super::ServerRuntimeHost;
use signal_runtime::{
    RuntimeConfig, RuntimeErrorKind, RuntimeReadiness, SignalRuntime, StopReason,
};

#[test]
fn server_host_rolls_back_replacement_transport_when_recovery_teardown_fails() {
    let runtime = SignalRuntime::new(RuntimeConfig::server(48_000, 512));
    let mut host = ServerRuntimeHost::new(runtime);
    let error = host
        .boot_with_recovery_teardown_failure()
        .expect_err("recovery teardown failure should abort");
    let supervisor = host.supervisor_report();

    assert_eq!(error.kind, RuntimeErrorKind::ResourceUnavailable);
    assert!(
        error
            .message
            .contains("injected old transport teardown failure"),
        "unexpected error: {}",
        error.message
    );
    assert_eq!(supervisor.observation.control_snapshot.start_count, 1);
    assert_eq!(supervisor.observation.control_snapshot.stop_count, 1);
    assert_eq!(
        supervisor.observation.control_snapshot.last_stop_reason,
        Some(StopReason::DegradedModeRecovery)
    );
    assert!(!supervisor.observation.control_snapshot.running);
    assert_eq!(supervisor.observation.readiness, RuntimeReadiness::Stopped);
    assert_eq!(
        supervisor
            .observation
            .diagnostics_snapshot
            .active_plugin_sandboxes,
        0
    );
    assert_eq!(
        supervisor
            .observation
            .transport_concurrency_snapshot
            .current_attached_sessions,
        0
    );
    assert_eq!(
        supervisor
            .observation
            .transport_concurrency_snapshot
            .peak_attached_sessions,
        2
    );
    assert!(supervisor
        .observation
        .transport_session_summary
        .active_sessions
        .is_empty());
    assert_eq!(
        supervisor
            .observation
            .transport_session_summary
            .current_attached_session_count,
        0
    );
    assert_eq!(supervisor.observation.control_snapshot.restart_count, 0);
}

#[test]
fn server_host_exposes_lingering_detach_fault_state_after_deferred_recovery_teardown_failure() {
    let runtime = SignalRuntime::new(RuntimeConfig::server(48_000, 512));
    let mut host = ServerRuntimeHost::new(runtime);
    let error = host
        .boot_with_recovery_deferred_teardown_failure()
        .expect_err("deferred teardown failure should abort");
    let supervisor = host.supervisor_report();

    assert_eq!(error.kind, RuntimeErrorKind::ResourceUnavailable);
    assert!(
        error
            .message
            .contains("deferred old transport teardown during recovery retry"),
        "unexpected error: {}",
        error.message
    );
    assert_eq!(supervisor.observation.control_snapshot.start_count, 1);
    assert_eq!(supervisor.observation.control_snapshot.stop_count, 1);
    assert_eq!(
        supervisor.observation.control_snapshot.last_stop_reason,
        Some(StopReason::DegradedModeRecovery)
    );
    assert!(!supervisor.observation.control_snapshot.running);
    assert_eq!(supervisor.observation.readiness, RuntimeReadiness::Stopped);
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
        1
    );
    assert_eq!(
        supervisor
            .observation
            .transport_concurrency_snapshot
            .current_detach_faulted_sessions,
        1
    );
    assert_eq!(
        supervisor
            .observation
            .transport_concurrency_snapshot
            .peak_attached_sessions,
        2
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
        signal_runtime::TransportSessionState::DetachFaulted
    );
}
