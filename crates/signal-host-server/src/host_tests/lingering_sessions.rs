use super::super::host_test_support::{
    assert_runtime_automation_continuity, assert_runtime_automation_values,
    assert_runtime_plugin_event_snapshot, assert_runtime_sequence_continuity,
    prepare_server_host_with_lifecycle, prepare_server_host_without_lifecycle,
    temp_media_fixture_path,
};
use super::super::ServerRuntimeHost;
use signal_graph::{GraphNodeExecutionClass, GraphNodeTopologyRole, GraphStageSpec};
use signal_plugin::{CompletionState, PluginFormat, WatchdogTriggerReason};
use signal_plugin_clap::ClapSandboxLifecycleHarness;
use signal_primitives::{ChannelCount, ChannelLayout};
use signal_runtime::{
    BlockDispatchStage, BrokerFailureStage, BrokerInvalidationStage, CompletionSlotStage,
    GraphContractProjection, GraphNodeBufferContractProjection, GraphNodeBusEndpointProjection,
    GraphNodeContractProjection, GraphNodeProjection, GraphNodeTopologyProjection,
    GraphProjection, HandshakeRequest, HeartbeatCycleStage, LingeringCleanupMode,
    PluginBackedNodeBinding, PluginBackedNodeBindingProjection, PluginSandboxLifecycleStage,
    PluginSandboxSpec, PluginSandboxTransportStage, PluginScanRequest, RecoveryRestartIntent,
    RuntimeConfig, RuntimeConfigRequest, RuntimeErrorKind, RuntimeExternalIoDeviceChangeState,
    RuntimeExternalIoHealthState, RuntimeExternalIoLoopbackState,
    RuntimeExternalIoMonitoringState, RuntimeExternalIoMonitoringTapPoint,
    RuntimeExternalIoPrimaryRole, RuntimeLifecycleApi, RuntimeMediaAssetRegistration,
    RuntimeMediaPreviewState, RuntimeObservationApi, RuntimePluginHostPlatform,
    RuntimePluginIsolationOutcome, RuntimePluginParityBand, RuntimeProjectionApi,
    RuntimeReadiness, RuntimeSupervisorApi, SandboxOperationFailureStage, SignalRuntime,
    StopReason, TransportAttachIntent,
};
use std::{fs, path::Path};

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

#[test]
fn server_host_recovers_after_lingering_deferred_teardown_cleanup() {
    let runtime = SignalRuntime::new(RuntimeConfig::server(48_000, 512));
    let mut host = ServerRuntimeHost::new(runtime);
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
fn server_host_recovers_after_lingering_cleanup_fails_once_more() {
    let runtime = SignalRuntime::new(RuntimeConfig::server(48_000, 512));
    let mut host = ServerRuntimeHost::new(runtime);
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

#[test]
fn server_host_sweeps_orphan_lingering_sessions_before_overlap_recovery() {
    let (mut host, protocol, mut lifecycle, run) = prepare_server_host_with_lifecycle();
    let orphan_region = host
        .broker
        .create_region("server-orphan-lingering", 256)
        .expect("orphan region");
    let orphan_transport = orphan_region.metadata().clone();
    host.runtime
        .begin_transport_session_with_metadata(
            "server-default-sandbox",
            "lease-orphan",
            orphan_transport.region_id.as_str(),
            TransportAttachIntent::RecoveryOverlap,
            Some(orphan_transport.backing_path.clone()),
            Some(orphan_transport.total_bytes),
        )
        .expect("orphan transport session");
    host.runtime.record_plugin_sandbox_transport(
        "server-default-sandbox",
        "lease-orphan",
        orphan_transport.region_id.as_str(),
        PluginSandboxTransportStage::DetachFault,
        Some(1),
        Some("replacement rollback linger".into()),
    );

    let recovered = host
        .recover_sandbox(
            &protocol,
            "server-default-sandbox",
            &mut lifecycle,
            &run,
            RecoveryRestartIntent::WatchdogRecovery,
            None,
        )
        .expect("orphan lingering sweep recovery");
    let supervisor = host.supervisor_report();

    assert_eq!(recovered.processing_epoch, 2);
    assert_eq!(supervisor.observation.readiness, RuntimeReadiness::Ready);
    assert!(supervisor.observation.control_snapshot.running);
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
            .peak_attached_sessions,
        2
    );
    assert!(supervisor
        .observation
        .transport_concurrency_snapshot
        .active_sessions
        .iter()
        .all(|session| session.lease_id != "lease-orphan"));
    assert!(!Path::new(&orphan_transport.backing_path).exists());
}

#[test]
fn server_host_aborts_when_orphan_lingering_cleanup_fails_before_overlap_recovery() {
    let (mut host, protocol, mut lifecycle, run) = prepare_server_host_with_lifecycle();
    host.runtime
        .begin_transport_session_with_metadata(
            "server-default-sandbox",
            "lease-orphan",
            "region-orphan-failure",
            TransportAttachIntent::RecoveryOverlap,
            None,
            None,
        )
        .expect("orphan transport session");
    host.runtime.record_plugin_sandbox_transport(
        "server-default-sandbox",
        "lease-orphan",
        "region-orphan-failure",
        PluginSandboxTransportStage::DetachFault,
        Some(1),
        Some("replacement rollback linger".into()),
    );

    let error = host
        .recover_sandbox(
            &protocol,
            "server-default-sandbox",
            &mut lifecycle,
            &run,
            RecoveryRestartIntent::WatchdogRecovery,
            None,
        )
        .expect_err("orphan lingering cleanup failure should abort recovery");
    let supervisor = host.supervisor_report();

    assert_eq!(error.kind, RuntimeErrorKind::ResourceUnavailable);
    assert!(error.message.contains("missing backing_path metadata"));
    assert!(!supervisor.observation.control_snapshot.running);
    assert_eq!(supervisor.observation.readiness, RuntimeReadiness::Stopped);
    assert_eq!(
        supervisor
            .observation
            .transport_concurrency_snapshot
            .current_attached_sessions,
        2
    );
    assert_eq!(
        supervisor
            .observation
            .transport_concurrency_snapshot
            .current_lingering_sessions,
        1
    );
    assert!(supervisor
        .observation
        .transport_concurrency_snapshot
        .active_sessions
        .iter()
        .any(|session| session.lease_id == "lease-orphan"));
}

#[test]
fn server_host_cleans_multiple_orphan_lingering_sessions_for_same_sandbox() {
    let runtime = SignalRuntime::new(RuntimeConfig::server(48_000, 512));
    let mut host = ServerRuntimeHost::new(runtime);
    let orphan_region_a = host
        .broker
        .create_region("server-orphan-a", 256)
        .expect("orphan region a");
    let orphan_transport_a = orphan_region_a.metadata().clone();
    let orphan_region_b = host
        .broker
        .create_region("server-orphan-b", 256)
        .expect("orphan region b");
    let orphan_transport_b = orphan_region_b.metadata().clone();

    host.runtime
        .begin_transport_session_with_metadata(
            "server-default-sandbox",
            "lease-orphan-a",
            orphan_transport_a.region_id.as_str(),
            TransportAttachIntent::SteadyState,
            Some(orphan_transport_a.backing_path.clone()),
            Some(orphan_transport_a.total_bytes),
        )
        .expect("orphan session a");
    host.runtime.record_plugin_sandbox_transport(
        "server-default-sandbox",
        "lease-orphan-a",
        orphan_transport_a.region_id.as_str(),
        PluginSandboxTransportStage::DetachFault,
        Some(1),
        Some("orphan a lingering".into()),
    );
    host.runtime
        .begin_transport_session_with_metadata(
            "server-default-sandbox",
            "lease-orphan-b",
            orphan_transport_b.region_id.as_str(),
            TransportAttachIntent::RecoveryOverlap,
            Some(orphan_transport_b.backing_path.clone()),
            Some(orphan_transport_b.total_bytes),
        )
        .expect("orphan session b");
    host.runtime.record_plugin_sandbox_transport(
        "server-default-sandbox",
        "lease-orphan-b",
        orphan_transport_b.region_id.as_str(),
        PluginSandboxTransportStage::DetachFault,
        Some(1),
        Some("orphan b lingering".into()),
    );

    host.cleanup_orphan_lingering_sessions_for_sandbox(
        "server-default-sandbox",
        1,
        None,
        None,
        LingeringCleanupMode::StrictPreAttach,
    )
    .expect("multiple orphan cleanup");

    let supervisor = host.supervisor_report();
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
            .current_lingering_sessions,
        0
    );
    assert!(supervisor
        .observation
        .transport_concurrency_snapshot
        .active_sessions
        .is_empty());
    assert!(!Path::new(&orphan_transport_a.backing_path).exists());
    assert!(!Path::new(&orphan_transport_b.backing_path).exists());
}

#[test]
fn server_host_reconciles_late_lingering_completion_without_disturbing_active_replacement() {
    let (mut host, protocol) = prepare_server_host_without_lifecycle();
    let late_region = host
        .broker
        .create_region("server-late-lingering", 256)
        .expect("late lingering region");
    let late_transport = late_region.metadata().clone();
    host.runtime
        .begin_transport_session_with_metadata(
            "server-default-sandbox",
            "lease-late-origin",
            late_transport.region_id.as_str(),
            TransportAttachIntent::SteadyState,
            Some(late_transport.backing_path.clone()),
            Some(late_transport.total_bytes),
        )
        .expect("late lingering session");
    host.runtime.record_plugin_sandbox_transport(
        "server-default-sandbox",
        "lease-late-origin",
        late_transport.region_id.as_str(),
        PluginSandboxTransportStage::DetachFault,
        Some(1),
        Some("late origin teardown completion".into()),
    );
    let mut lifecycle = ClapSandboxLifecycleHarness::default();
    let recovered = host
        .run_lifecycle(&protocol, "server-default-sandbox", 2, &mut lifecycle)
        .expect("replacement lifecycle");

    host.reconcile_late_lingering_sessions_after_start("server-default-sandbox", &recovered);

    let supervisor = host.supervisor_report();
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
            .active_sessions
            .len(),
        1
    );
    assert_eq!(
        supervisor
            .observation
            .transport_concurrency_snapshot
            .active_sessions[0]
            .lease_id,
        recovered.shared_memory_lease_id
    );
    assert!(!Path::new(&late_transport.backing_path).exists());
}

#[test]
fn server_host_keeps_active_replacement_running_when_late_lingering_cleanup_fails() {
    let (mut host, protocol) = prepare_server_host_without_lifecycle();
    host.runtime
        .begin_transport_session_with_metadata(
            "server-default-sandbox",
            "lease-late-origin",
            "region-late-origin-failure",
            TransportAttachIntent::SteadyState,
            None,
            None,
        )
        .expect("late lingering session");
    host.runtime.record_plugin_sandbox_transport(
        "server-default-sandbox",
        "lease-late-origin",
        "region-late-origin-failure",
        PluginSandboxTransportStage::DetachFault,
        Some(1),
        Some("late origin teardown completion".into()),
    );
    let mut lifecycle = ClapSandboxLifecycleHarness::default();
    let recovered = host
        .run_lifecycle(&protocol, "server-default-sandbox", 2, &mut lifecycle)
        .expect("replacement lifecycle");

    host.reconcile_late_lingering_sessions_after_start("server-default-sandbox", &recovered);

    let supervisor = host.supervisor_report();
    assert!(supervisor.observation.control_snapshot.running);
    assert_eq!(supervisor.observation.readiness, RuntimeReadiness::Ready);
    assert_eq!(
        supervisor
            .observation
            .transport_concurrency_snapshot
            .current_attached_sessions,
        2
    );
    assert_eq!(
        supervisor
            .observation
            .transport_concurrency_snapshot
            .current_lingering_sessions,
        1
    );
    assert!(supervisor
        .observation
        .transport_concurrency_snapshot
        .active_sessions
        .iter()
        .any(|session| session.lease_id == recovered.shared_memory_lease_id));
    assert!(supervisor
        .observation
        .transport_concurrency_snapshot
        .active_sessions
        .iter()
        .any(|session| session.lease_id == "lease-late-origin"));
    assert!(supervisor
        .observation
        .observation
        .broker_failure_events
        .iter()
        .any(|failure| {
            failure.stage == BrokerFailureStage::TransportTeardown
                && failure.detail.contains("missing backing_path metadata")
        }));
}

#[test]
fn server_host_sweeps_prior_late_lingering_before_next_overlap_recovery() {
    let (mut host, protocol) = prepare_server_host_without_lifecycle();
    let late_region = host
        .broker
        .create_region("server-adjacent-lingering", 256)
        .expect("late lingering region");
    let late_transport = late_region.metadata().clone();
    host.runtime
        .begin_transport_session_with_metadata(
            "server-default-sandbox",
            "lease-prior-lingering",
            late_transport.region_id.as_str(),
            TransportAttachIntent::SteadyState,
            Some(late_transport.backing_path.clone()),
            Some(late_transport.total_bytes),
        )
        .expect("prior late lingering session");
    host.runtime.record_plugin_sandbox_transport(
        "server-default-sandbox",
        "lease-prior-lingering",
        late_transport.region_id.as_str(),
        PluginSandboxTransportStage::DetachFault,
        Some(1),
        Some("prior late completion".into()),
    );
    let mut lifecycle = ClapSandboxLifecycleHarness::default();
    let recovered_epoch2 = host
        .run_lifecycle(&protocol, "server-default-sandbox", 2, &mut lifecycle)
        .expect("replacement lifecycle");
    let recovered_transport = recovered_epoch2
        .transport
        .as_ref()
        .expect("recovered transport");
    host.runtime.record_plugin_sandbox_transport(
        "server-default-sandbox",
        recovered_epoch2.shared_memory_lease_id.as_str(),
        recovered_transport.region_id.as_str(),
        PluginSandboxTransportStage::DetachFault,
        Some(recovered_epoch2.processing_epoch),
        Some("current replacement became lingering before adjacent recovery".into()),
    );

    let recovered_epoch3 = host
        .recover_sandbox(
            &protocol,
            "server-default-sandbox",
            &mut lifecycle,
            &recovered_epoch2,
            RecoveryRestartIntent::WatchdogRecovery,
            None,
        )
        .expect("adjacent recovery should sweep prior lingering session");
    let supervisor = host.supervisor_report();

    assert_eq!(recovered_epoch3.processing_epoch, 3);
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
            .peak_attached_sessions,
        2
    );
    assert!(supervisor
        .observation
        .transport_concurrency_snapshot
        .active_sessions
        .iter()
        .all(|session| session.lease_id != "lease-prior-lingering"));
    assert!(!Path::new(&late_transport.backing_path).exists());
}

#[test]
fn server_host_aborts_adjacent_overlap_recovery_when_prior_late_lingering_lacks_metadata() {
    let (mut host, protocol) = prepare_server_host_without_lifecycle();
    host.runtime
        .begin_transport_session_with_metadata(
            "server-default-sandbox",
            "lease-prior-lingering",
            "region-prior-lingering-failure",
            TransportAttachIntent::SteadyState,
            None,
            None,
        )
        .expect("prior late lingering session");
    host.runtime.record_plugin_sandbox_transport(
        "server-default-sandbox",
        "lease-prior-lingering",
        "region-prior-lingering-failure",
        PluginSandboxTransportStage::DetachFault,
        Some(1),
        Some("prior late completion".into()),
    );
    let mut lifecycle = ClapSandboxLifecycleHarness::default();
    let recovered_epoch2 = host
        .run_lifecycle(&protocol, "server-default-sandbox", 2, &mut lifecycle)
        .expect("replacement lifecycle");

    let error = host
        .recover_sandbox(
            &protocol,
            "server-default-sandbox",
            &mut lifecycle,
            &recovered_epoch2,
            RecoveryRestartIntent::WatchdogRecovery,
            None,
        )
        .expect_err("adjacent recovery should abort on stale lingering metadata");
    let supervisor = host.supervisor_report();

    assert_eq!(error.kind, RuntimeErrorKind::ResourceUnavailable);
    assert!(error.message.contains("missing backing_path metadata"));
    assert!(!supervisor.observation.control_snapshot.running);
    assert_eq!(supervisor.observation.readiness, RuntimeReadiness::Stopped);
    assert_eq!(
        supervisor
            .observation
            .transport_concurrency_snapshot
            .current_attached_sessions,
        2
    );
    assert_eq!(
        supervisor
            .observation
            .transport_concurrency_snapshot
            .current_lingering_sessions,
        1
    );
    assert!(supervisor
        .observation
        .transport_concurrency_snapshot
        .active_sessions
        .iter()
        .any(|session| session.lease_id == "lease-prior-lingering"));
    assert!(supervisor
        .observation
        .transport_concurrency_snapshot
        .active_sessions
        .iter()
        .any(|session| session.lease_id == recovered_epoch2.shared_memory_lease_id));
}

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

