#[path = "support/public_host_edge_plugins.rs"]
mod public_host_edge_plugins_support;
#[path = "support/public_host_edge_sandbox_broker.rs"]
mod public_host_edge_sandbox_broker_support;

use public_host_edge_plugins_support::{
    temp_public_server_au_scan_root, temp_public_server_lv2_scan_root,
    temp_public_server_vst3_scan_root,
};
use public_host_edge_sandbox_broker_support::SandboxBrokerEnvGuard;
use signal_host_server::ServerRuntimeHost;
use signal_plugin::PluginFormat;
use signal_runtime::{
    PluginSandboxLifecycleStage, PluginSandboxSpec, PluginSandboxTransportStage, PluginScanRequest,
    RuntimeConfig, RuntimeSupervisorApi, SignalRuntime,
};

#[test]
fn server_public_host_edge_can_route_vst3_sandbox_through_broker_process() {
    let _guard = SandboxBrokerEnvGuard::enable_for_workspace_cargo_run();
    let runtime = SignalRuntime::new(RuntimeConfig::server(48_000, 512));
    let mut host = ServerRuntimeHost::new(runtime);
    let scan_root = temp_public_server_vst3_scan_root();

    host.start_plugin_scan(PluginScanRequest {
        roots: vec![scan_root.root()],
        formats: vec![PluginFormat::Vst3],
    })
    .expect("public server broker-backed vst3 scan should succeed");
    host.ensure_plugin_sandbox(PluginSandboxSpec {
        sandbox_id: "public-host-edge-server-vst3-broker".into(),
        plugin_format: PluginFormat::Vst3,
        plugin_type_id: Some("plugin:vst3:linux-synth".into()),
    })
    .expect("public server broker-backed vst3 sandbox ensure should succeed");

    let report = host.supervisor_report();
    let sandbox = report
        .observation
        .plugin_lifecycle_snapshot
        .sandboxes
        .iter()
        .find(|sandbox| sandbox.sandbox_id == "public-host-edge-server-vst3-broker")
        .expect("broker-backed server sandbox should be exported");
    assert_eq!(
        sandbox.lifecycle_stage,
        Some(PluginSandboxLifecycleStage::TransportAttached)
    );
    assert_eq!(
        sandbox.transport_stage,
        Some(PluginSandboxTransportStage::Attached)
    );
    assert_eq!(sandbox.readiness_state.as_deref(), Some("Ready"));
    let rendered = report.render_json();
    assert!(rendered.contains("broker:lease_attached"));
    assert!(rendered.contains("vst3:instance="));
    assert!(
        rendered.contains("component=Signal_Linux_Synth_VST3_Plugin")
            || rendered.contains("component=Signal Linux Synth VST3 Plugin")
    );
    assert!(rendered.contains("state_stored="));
    assert!(rendered.contains("state_bytes="));
    assert!(rendered.contains("execution_complete"));
    assert!(rendered.contains("processed_blocks=3"));
    assert!(rendered.contains("parameter_events=1"));
    assert!(rendered.contains("midi_events=2"));
    assert!(rendered.contains("parameter_signature="));
    assert!(rendered.contains("application_order=block0:parameters(2)->midi(1);block1:parameters(4)->midi(0);block2:parameters(1)->midi(2)"));
    assert!(rendered.contains("packet_order=block0[param_packets=2,midi_packets=1,apply=parameters_then_midi];block1[param_packets=4,midi_packets=0,apply=parameters_then_midi];block2[param_packets=1,midi_packets=2,apply=parameters_then_midi]"));
    assert!(rendered.contains("automation_delta=block0:delta[param=2,midi=1,baseline="));
    assert!(rendered.contains("automation_delta=block0:delta[param=3,midi=2,baseline="));
    assert!(rendered.contains("refresh_cycle=state_store"));
    assert!(rendered.contains("continuity_reset=refreshed"));
    assert!(rendered.contains("execution_interrupted"));
    assert!(rendered.contains("timeout=recoverable"));
    assert!(rendered.contains("resume_hint=refresh_or_stream"));
    assert!(rendered.contains("next_state_digest="));
    assert!(rendered.contains("state_transition=applied"));
    assert!(rendered.contains("execution_runs=2"));
    assert!(rendered.contains("continuity=carried_forward"));
    assert!(rendered.contains("continued_from="));
    assert!(rendered.contains("execution_runs=1"));
    assert!(rendered.contains("continuity=fresh"));
    assert!(rendered.contains("continued_from=none"));

    host.teardown_plugin_sandbox("public-host-edge-server-vst3-broker")
        .expect("broker-backed server sandbox teardown should succeed");

    let report = host.supervisor_report();
    let sandbox = report
        .observation
        .plugin_lifecycle_snapshot
        .sandboxes
        .iter()
        .find(|sandbox| sandbox.sandbox_id == "public-host-edge-server-vst3-broker")
        .expect("broker-backed server sandbox should stay exported");
    assert_eq!(
        sandbox.lifecycle_stage,
        Some(PluginSandboxLifecycleStage::InstanceDestroyed)
    );
    assert_eq!(
        sandbox.transport_stage,
        Some(PluginSandboxTransportStage::Detached)
    );
    let rendered = report.render_json();
    assert!(rendered.contains("lease_cleanup_ok"));
    assert!(rendered.contains("\"stage\":\"TransportTornDown\""));
    assert!(rendered.contains("vst3:instance="));
    assert!(rendered.contains("flushed_state_bytes="));
}

#[test]
fn server_public_host_edge_resets_vst3_continuity_after_broker_reattach() {
    let _guard = SandboxBrokerEnvGuard::enable_for_workspace_cargo_run();
    let runtime = SignalRuntime::new(RuntimeConfig::server(48_000, 512));
    let mut host = ServerRuntimeHost::new(runtime);
    let scan_root = temp_public_server_vst3_scan_root();

    host.start_plugin_scan(PluginScanRequest {
        roots: vec![scan_root.root()],
        formats: vec![PluginFormat::Vst3],
    })
    .expect("public server broker-backed vst3 scan should succeed");

    let sandbox_id = "public-host-edge-server-vst3-broker-reset";
    host.ensure_plugin_sandbox(PluginSandboxSpec {
        sandbox_id: sandbox_id.into(),
        plugin_format: PluginFormat::Vst3,
        plugin_type_id: Some("plugin:vst3:linux-synth".into()),
    })
    .expect("first public server broker-backed vst3 sandbox ensure should succeed");
    host.teardown_plugin_sandbox(sandbox_id)
        .expect("first broker-backed server sandbox teardown should succeed");

    host.ensure_plugin_sandbox(PluginSandboxSpec {
        sandbox_id: sandbox_id.into(),
        plugin_format: PluginFormat::Vst3,
        plugin_type_id: Some("plugin:vst3:linux-synth".into()),
    })
    .expect("second public server broker-backed vst3 sandbox ensure should succeed");

    let report = host.supervisor_report();
    let sandbox = report
        .observation
        .plugin_lifecycle_snapshot
        .sandboxes
        .iter()
        .find(|sandbox| sandbox.sandbox_id == sandbox_id)
        .expect("broker-backed server reset sandbox should be exported");
    assert_eq!(
        sandbox.lifecycle_stage,
        Some(PluginSandboxLifecycleStage::TransportAttached)
    );
    assert_eq!(
        sandbox.transport_stage,
        Some(PluginSandboxTransportStage::Attached)
    );

    let rendered = report.render_json();
    assert!(rendered.contains("execution_runs=1"));
    assert!(rendered.contains("continuity=fresh"));
    assert!(rendered.contains("continued_from=none"));
    assert!(rendered.contains("execution_runs=2"));
    assert!(rendered.contains("continuity=carried_forward"));
    assert!(rendered.contains("automation_delta=block0:delta[param=2,midi=1,baseline="));
    assert!(rendered.contains("automation_delta=block0:delta[param=3,midi=2,baseline="));
    assert!(rendered.contains("refresh_cycle=state_store"));
    assert!(rendered.contains("continuity_reset=refreshed"));

    host.teardown_plugin_sandbox(sandbox_id)
        .expect("second broker-backed server sandbox teardown should succeed");
}

#[test]
fn server_public_host_edge_can_route_au_sandbox_through_broker_process() {
    let _guard = SandboxBrokerEnvGuard::enable_for_workspace_cargo_run();
    let runtime = SignalRuntime::new(RuntimeConfig::server(48_000, 512));
    let mut host = ServerRuntimeHost::new(runtime);
    let scan_root = temp_public_server_au_scan_root();

    host.start_plugin_scan(PluginScanRequest {
        roots: vec![scan_root.root()],
        formats: vec![PluginFormat::Au],
    })
    .expect("public server broker-backed au scan should succeed");
    host.ensure_plugin_sandbox(PluginSandboxSpec {
        sandbox_id: "public-host-edge-server-au-broker".into(),
        plugin_format: PluginFormat::Au,
        plugin_type_id: Some("plugin:au:instrument".into()),
    })
    .expect("public server broker-backed au sandbox ensure should succeed");

    let report = host.supervisor_report();
    let sandbox = report
        .observation
        .plugin_lifecycle_snapshot
        .sandboxes
        .iter()
        .find(|sandbox| sandbox.sandbox_id == "public-host-edge-server-au-broker")
        .expect("broker-backed server au sandbox should be exported");
    assert_eq!(
        sandbox.lifecycle_stage,
        Some(PluginSandboxLifecycleStage::TransportAttached)
    );
    assert_eq!(
        sandbox.transport_stage,
        Some(PluginSandboxTransportStage::Attached)
    );
    assert_eq!(sandbox.readiness_state.as_deref(), Some("Ready"));
    let rendered = report.render_json();
    assert!(rendered.contains("broker:lease_attached"));

    host.teardown_plugin_sandbox("public-host-edge-server-au-broker")
        .expect("broker-backed server au sandbox teardown should succeed");

    let report = host.supervisor_report();
    let sandbox = report
        .observation
        .plugin_lifecycle_snapshot
        .sandboxes
        .iter()
        .find(|sandbox| sandbox.sandbox_id == "public-host-edge-server-au-broker")
        .expect("broker-backed server au sandbox should stay exported");
    assert_eq!(
        sandbox.lifecycle_stage,
        Some(PluginSandboxLifecycleStage::InstanceDestroyed)
    );
    assert_eq!(
        sandbox.transport_stage,
        Some(PluginSandboxTransportStage::Detached)
    );
    let rendered = report.render_json();
    assert!(rendered.contains("lease_cleanup_ok"));
    assert!(rendered.contains("\"stage\":\"TransportTornDown\""));
}

#[test]
fn server_public_host_edge_can_route_lv2_sandbox_through_broker_process() {
    let _guard = SandboxBrokerEnvGuard::enable_for_workspace_cargo_run();
    let runtime = SignalRuntime::new(RuntimeConfig::server(48_000, 512));
    let mut host = ServerRuntimeHost::new(runtime);
    let scan_root = temp_public_server_lv2_scan_root();

    host.start_plugin_scan(PluginScanRequest {
        roots: vec![scan_root.root()],
        formats: vec![PluginFormat::Lv2],
    })
    .expect("public server broker-backed lv2 scan should succeed");
    host.ensure_plugin_sandbox(PluginSandboxSpec {
        sandbox_id: "public-host-edge-server-lv2-broker".into(),
        plugin_format: PluginFormat::Lv2,
        plugin_type_id: Some("plugin:lv2:linux-synth".into()),
    })
    .expect("public server broker-backed lv2 sandbox ensure should succeed");

    let report = host.supervisor_report();
    let sandbox = report
        .observation
        .plugin_lifecycle_snapshot
        .sandboxes
        .iter()
        .find(|sandbox| sandbox.sandbox_id == "public-host-edge-server-lv2-broker")
        .expect("broker-backed server lv2 sandbox should be exported");
    assert_eq!(
        sandbox.lifecycle_stage,
        Some(PluginSandboxLifecycleStage::TransportAttached)
    );
    assert_eq!(
        sandbox.transport_stage,
        Some(PluginSandboxTransportStage::Attached)
    );
    assert_eq!(sandbox.readiness_state.as_deref(), Some("Ready"));
    assert_eq!(
        sandbox
            .lv2_prepared_negotiation
            .as_ref()
            .map(|record| record.worker_posture),
        Some(signal_runtime::RuntimeLv2WorkerPosture::WorkerRequiredAvailable)
    );
    assert_eq!(
        sandbox
            .lv2_prepared_negotiation
            .as_ref()
            .map(|record| record.extension_negotiation_state),
        Some(signal_runtime::RuntimeLv2ExtensionNegotiationState::Negotiated)
    );
    let rendered = report.render_json();
    assert!(rendered.contains("broker:lease_attached|lv2:plugin_type=plugin:lv2:linux-synth"));
    assert!(rendered.contains("\"lv2_prepared_negotiation\":{"));
    assert!(rendered.contains("\"worker_posture\":\"WorkerRequiredAvailable\""));
    assert!(rendered.contains("\"extension_negotiation_state\":\"Negotiated\""));
    assert!(rendered.contains("execution_complete"));
    assert!(rendered.contains("processed_blocks=3"));
    assert!(rendered.contains("last_block_sequence=2"));
    assert!(rendered.contains("block_frames=256"));
    assert!(rendered.contains("stream_order=block0[frames=128,patch=1,midi=0,completion=Applied];block1[frames=192,patch=2,midi=1,completion=Applied];block2[frames=256,patch=1,midi=2,completion=Applied]"));
    assert!(rendered.contains("patch_messages=1"));
    assert!(rendered.contains("midi_events=2"));
    assert!(rendered.contains("completion=Applied"));

    host.teardown_plugin_sandbox("public-host-edge-server-lv2-broker")
        .expect("broker-backed server lv2 sandbox teardown should succeed");

    let report = host.supervisor_report();
    let sandbox = report
        .observation
        .plugin_lifecycle_snapshot
        .sandboxes
        .iter()
        .find(|sandbox| sandbox.sandbox_id == "public-host-edge-server-lv2-broker")
        .expect("broker-backed server lv2 sandbox should stay exported");
    assert_eq!(
        sandbox.lifecycle_stage,
        Some(PluginSandboxLifecycleStage::InstanceDestroyed)
    );
    assert_eq!(
        sandbox.transport_stage,
        Some(PluginSandboxTransportStage::Detached)
    );
    let rendered = report.render_json();
    assert!(rendered.contains(
        "lease_cleanup_ok|lv2:instance=instance:server:lv2:public-host-edge-server-lv2-broker"
    ));
    assert!(rendered.contains("\"stage\":\"TransportTornDown\""));
}

#[test]
fn server_public_host_edge_can_drive_broker_backed_lv2_crash_recovery() {
    let scan_root = temp_public_server_lv2_scan_root();
    let _guard = SandboxBrokerEnvGuard::enable_for_workspace_demo_plugin(
        "lv2",
        &scan_root.root(),
        "plugin:lv2:linux-synth",
    );
    let runtime = SignalRuntime::new(RuntimeConfig::server(48_000, 512));
    let mut host = ServerRuntimeHost::new(runtime);

    let summary = host
        .boot_with_crash_recovery()
        .expect("public server broker-backed lv2 crash recovery should succeed");

    assert_eq!(
        summary.execution.last_recovery_intent,
        Some(signal_host_server::RecoveryRestartIntent::CrashRecovery)
    );
    assert_eq!(
        summary.execution.last_stop_reason,
        Some(signal_runtime::StopReason::DegradedModeRecovery)
    );
    assert!(summary.execution.restart_count >= 1);
    assert!(summary.execution.processing_epoch >= 2);
    assert_eq!(summary.scan_roots, vec![scan_root.root()]);
    assert!(!summary.transport.shared_memory_lease_id.is_empty());
    assert!(!summary.transport.shared_memory_region_id.is_empty());

    let report = host.supervisor_report();
    let rendered = report.render_json();
    assert!(rendered.contains("broker:lease_attached"));
    assert!(rendered.contains("execution_complete"));
    assert!(rendered.contains("processed_blocks=3"));
    assert!(rendered.contains("stream_order=block0[frames=128,patch=1,midi=0,completion=Applied];block1[frames=192,patch=2,midi=1,completion=Applied];block2[frames=256,patch=1,midi=2,completion=Applied]"));
    assert!(rendered.contains("\"stop_reason\":\"DegradedModeRecovery\""));
}

#[test]
fn server_public_host_edge_reports_broker_backed_lv2_overlap_contention() {
    let scan_root = temp_public_server_lv2_scan_root();
    let _guard = SandboxBrokerEnvGuard::enable_for_workspace_demo_plugin(
        "lv2",
        &scan_root.root(),
        "plugin:lv2:linux-synth",
    );
    let runtime = SignalRuntime::new(RuntimeConfig::server(48_000, 512));
    let mut host = ServerRuntimeHost::new(runtime);

    let error = host
        .boot_with_recovery_overlap_contention()
        .expect_err("public server broker-backed lv2 overlap contention should abort recovery");

    assert_eq!(
        error.kind,
        signal_runtime::RuntimeErrorKind::ResourceUnavailable
    );
    assert!(error.message.contains("recovery overlap session limit 1"));

    let report = host.supervisor_report();
    assert_eq!(report.observation.control_snapshot.start_count, 1);
    assert_eq!(report.observation.control_snapshot.stop_count, 1);
    assert_eq!(
        report.observation.control_snapshot.last_stop_reason,
        Some(signal_runtime::StopReason::DegradedModeRecovery)
    );
    assert!(!report.observation.control_snapshot.running);
    assert_eq!(
        report
            .observation
            .transport_concurrency_snapshot
            .peak_attached_sessions,
        2
    );
    assert_eq!(
        report
            .observation
            .transport_concurrency_snapshot
            .current_attached_sessions,
        0
    );
    assert_eq!(
        report
            .observation
            .transport_concurrency_snapshot
            .last_rejected_sandbox_id
            .as_deref(),
        Some("server-default-sandbox")
    );
    assert!(report
        .observation
        .transport_concurrency_snapshot
        .last_rejection_reason
        .as_deref()
        .is_some_and(|reason| reason.contains("recovery overlap session limit 1")));
    assert!(report
        .observation
        .transport_session_summary
        .active_sessions
        .is_empty());

    let rendered = report.render_json();
    assert!(rendered.contains("broker:lease_attached"));
    assert!(rendered.contains("\"stop_reason\":\"DegradedModeRecovery\""));
    assert!(rendered.contains("recovery overlap session limit 1"));
}

#[test]
fn server_public_host_edge_reports_broker_backed_lv2_deferred_teardown_fault() {
    let scan_root = temp_public_server_lv2_scan_root();
    let _guard = SandboxBrokerEnvGuard::enable_for_workspace_demo_plugin(
        "lv2",
        &scan_root.root(),
        "plugin:lv2:linux-synth",
    );
    let runtime = SignalRuntime::new(RuntimeConfig::server(48_000, 512));
    let mut host = ServerRuntimeHost::new(runtime);

    let error = host
        .boot_with_recovery_deferred_teardown_failure()
        .expect_err("public server broker-backed lv2 deferred teardown fault should abort");

    assert_eq!(
        error.kind,
        signal_runtime::RuntimeErrorKind::ResourceUnavailable
    );
    assert!(error
        .message
        .contains("deferred old transport teardown during recovery retry"));

    let report = host.supervisor_report();
    assert_eq!(report.observation.control_snapshot.start_count, 1);
    assert_eq!(report.observation.control_snapshot.stop_count, 1);
    assert_eq!(
        report.observation.control_snapshot.last_stop_reason,
        Some(signal_runtime::StopReason::DegradedModeRecovery)
    );
    assert!(!report.observation.control_snapshot.running);
    assert_eq!(
        report
            .observation
            .diagnostics_snapshot
            .active_plugin_sandboxes,
        1
    );
    assert_eq!(
        report
            .observation
            .transport_concurrency_snapshot
            .current_attached_sessions,
        1
    );
    assert_eq!(
        report
            .observation
            .transport_concurrency_snapshot
            .current_lingering_sessions,
        1
    );
    assert_eq!(
        report
            .observation
            .transport_concurrency_snapshot
            .current_detach_faulted_sessions,
        1
    );
    assert_eq!(
        report
            .observation
            .transport_concurrency_snapshot
            .peak_attached_sessions,
        2
    );
    assert_eq!(
        report
            .observation
            .transport_concurrency_snapshot
            .active_sessions
            .len(),
        1
    );
    assert_eq!(
        report
            .observation
            .transport_concurrency_snapshot
            .active_sessions[0]
            .state,
        signal_runtime::TransportSessionState::DetachFaulted
    );

    let rendered = report.render_json();
    assert!(rendered.contains("broker:lease_attached"));
    assert!(rendered.contains("execution_complete"));
    assert!(rendered.contains("processed_blocks=3"));
    assert!(rendered.contains("stream_order=block0[frames=128,patch=1,midi=0,completion=Applied];block1[frames=192,patch=2,midi=1,completion=Applied];block2[frames=256,patch=1,midi=2,completion=Applied]"));
    assert!(rendered.contains("deferred old transport teardown during recovery retry"));
    assert!(rendered.contains("\"state\":\"DetachFaulted\""));
}

#[test]
fn server_public_host_edge_recovers_after_broker_backed_lv2_cleanup_retry() {
    let scan_root = temp_public_server_lv2_scan_root();
    let _guard = SandboxBrokerEnvGuard::enable_for_workspace_demo_plugin(
        "lv2",
        &scan_root.root(),
        "plugin:lv2:linux-synth",
    );
    let runtime = SignalRuntime::new(RuntimeConfig::server(48_000, 512));
    let mut host = ServerRuntimeHost::new(runtime);

    let summary = host
        .boot_with_recovery_deferred_teardown_cleanup_retry()
        .expect("public server broker-backed lv2 cleanup retry recovery should succeed");

    assert_eq!(summary.execution.processing_epoch, 2);
    assert_eq!(summary.execution.restart_count, 1);
    assert_eq!(
        summary.execution.last_stop_reason,
        Some(signal_runtime::StopReason::DegradedModeRecovery)
    );
    assert_eq!(summary.scan_roots, vec![scan_root.root()]);
    assert!(!summary.transport.shared_memory_lease_id.is_empty());
    assert!(!summary.transport.shared_memory_region_id.is_empty());

    let report = host.supervisor_report();
    assert_eq!(report.observation.control_snapshot.start_count, 2);
    assert_eq!(report.observation.control_snapshot.stop_count, 1);
    assert!(report.observation.control_snapshot.running);
    assert_eq!(
        report
            .observation
            .transport_concurrency_snapshot
            .current_attached_sessions,
        1
    );
    assert_eq!(
        report
            .observation
            .transport_concurrency_snapshot
            .current_lingering_sessions,
        0
    );
    assert_eq!(
        report
            .observation
            .transport_concurrency_snapshot
            .current_detach_faulted_sessions,
        0
    );
    assert_eq!(
        report
            .observation
            .transport_concurrency_snapshot
            .peak_lingering_sessions,
        2
    );
    assert_eq!(
        report
            .observation
            .transport_concurrency_snapshot
            .active_sessions[0]
            .state,
        signal_runtime::TransportSessionState::AttachActive
    );
    assert!(report
        .observation
        .observation
        .broker_failure_events
        .iter()
        .any(|failure| {
            failure.stage == signal_runtime::BrokerFailureStage::TransportTeardown
                && failure
                    .detail
                    .contains("injected lingering cleanup retry failure")
        }));

    let rendered = report.render_json();
    assert!(rendered.contains("broker:lease_attached"));
    assert!(rendered.contains("execution_complete"));
    assert!(rendered.contains("processed_blocks=3"));
    assert!(rendered.contains("stream_order=block0[frames=128,patch=1,midi=0,completion=Applied];block1[frames=192,patch=2,midi=1,completion=Applied];block2[frames=256,patch=1,midi=2,completion=Applied]"));
    assert!(rendered.contains("injected lingering cleanup retry failure"));
    assert!(rendered.contains("\"state\":\"AttachActive\""));
}
