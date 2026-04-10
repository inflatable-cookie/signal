use crate::host::host_test_support::prepare_server_host_without_lifecycle;
use crate::ServerRuntimeHost;
use signal_runtime::{
    BrokerFailureStage, RuntimeConfig, RuntimeErrorKind, RuntimeReadiness, SignalRuntime,
    StopReason,
};

#[test]
fn server_host_rolls_back_replacement_transport_when_recovery_start_fails() {
    let runtime = SignalRuntime::new(RuntimeConfig::server(48_000, 512));
    let mut host = ServerRuntimeHost::new(runtime);
    let error = host
        .boot_with_recovery_restart_failure()
        .expect_err("recovery start failure should abort");
    let supervisor = host.supervisor_report();

    assert_eq!(error.kind, RuntimeErrorKind::ResourceUnavailable);
    assert!(
        error.message.contains("injected replacement start failure"),
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
fn server_host_rolls_back_partial_overlap_when_competing_recovery_attach_is_rejected() {
    let runtime = SignalRuntime::new(RuntimeConfig::server(48_000, 512));
    let mut host = ServerRuntimeHost::new(runtime);
    let error = host
        .boot_with_recovery_overlap_contention()
        .expect_err("overlap contention should abort recovery");
    let supervisor = host.supervisor_report();

    assert_eq!(error.kind, RuntimeErrorKind::ResourceUnavailable);
    assert!(
        error.message.contains("recovery overlap session limit 1"),
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
    assert_eq!(
        supervisor
            .observation
            .transport_concurrency_snapshot
            .last_rejected_sandbox_id
            .as_deref(),
        Some("server-default-sandbox")
    );
    assert!(supervisor
        .observation
        .transport_concurrency_snapshot
        .last_rejection_reason
        .as_deref()
        .is_some_and(|reason| reason.contains("recovery overlap session limit 1")));
    assert!(supervisor
        .observation
        .transport_session_summary
        .active_sessions
        .is_empty());
}

#[test]
fn server_host_handles_interleaved_recovery_failures_across_retries() {
    let runtime = SignalRuntime::new(RuntimeConfig::server(48_000, 512));
    let mut host = ServerRuntimeHost::new(runtime);
    let error = host
        .boot_with_recovery_interleaved_failures()
        .expect_err("interleaved failures should abort recovery");
    let supervisor = host.supervisor_report();

    assert_eq!(error.kind, RuntimeErrorKind::ResourceUnavailable);
    assert!(
        error.message.contains("recovery overlap session limit 1"),
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
    assert_eq!(
        supervisor
            .observation
            .transport_concurrency_snapshot
            .last_rejected_sandbox_id
            .as_deref(),
        Some("server-default-sandbox")
    );
    assert!(supervisor
        .observation
        .transport_concurrency_snapshot
        .last_rejection_reason
        .as_deref()
        .is_some_and(|reason| reason.contains("recovery overlap session limit 1")));
    assert!(supervisor
        .observation
        .observation
        .broker_failure_events
        .iter()
        .any(|failure| {
            failure.stage == BrokerFailureStage::TransportTeardown
                && failure.detail.contains("deferred old transport teardown")
        }));
    assert!(supervisor
        .observation
        .transport_session_summary
        .active_sessions
        .is_empty());
}
