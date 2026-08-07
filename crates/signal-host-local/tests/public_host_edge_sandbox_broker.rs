#[path = "support/public_host_edge_plugins/mod.rs"]
mod public_host_edge_plugins_support;
#[path = "support/public_host_edge_sandbox_broker.rs"]
mod public_host_edge_sandbox_broker_support;

use public_host_edge_plugins_support::{
    temp_public_local_au_scan_root, temp_public_local_vst3_scan_root,
};
use public_host_edge_sandbox_broker_support::{run_public_sandbox_broker, SandboxBrokerEnvGuard};
use signal_host_local::LocalRuntimeHost;
use signal_plugin::PluginFormat;
use signal_runtime::{
    PluginSandboxLifecycleStage, PluginSandboxSpec, PluginSandboxTransportStage, PluginScanRequest,
    RuntimeConfig, RuntimeSupervisorApi, SignalRuntime,
};

#[test]
fn local_public_host_edge_can_drive_sandbox_broker_process_truth() {
    let output = run_public_sandbox_broker("status\nattach\nrun\nteardown\nshutdown\n");

    assert!(
        output.status.success(),
        "sandbox broker should succeed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8(output.stdout).expect("sandbox broker stdout should be utf8");
    assert!(stdout.contains("state=starting"));
    assert!(stdout.contains("state=ready"));
    assert!(stdout.contains("state=attached"));
    assert!(stdout.contains("detail=lease_attached"));
    assert!(stdout.contains("state=running"));
    assert!(stdout.contains("shm_block_roundtrip"));
    assert!(stdout.contains("execution_complete|processed_blocks=8"));
    assert!(stdout.contains("state=teardown_complete"));
    assert!(stdout.contains("detail=lease_cleanup_ok"));
    assert!(stdout.contains("state=shutdown"));
}

#[test]
fn local_public_host_edge_sees_timeout_cleanup_from_sandbox_broker_process() {
    let output = run_public_sandbox_broker("attach\nrun-timeout\nteardown\nshutdown\n");

    assert!(
        output.status.success(),
        "sandbox broker timeout run should succeed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8(output.stdout).expect("sandbox broker stdout should be utf8");
    assert!(stdout.contains("state=timed_out"));
    assert!(stdout.contains("timeout=recoverable"));
    assert!(stdout.contains("reattached_after_timeout"));
    assert!(stdout.contains("detail=lease_cleanup_ok"));
    assert!(stdout.contains("state=shutdown"));
}

#[test]
fn local_public_host_edge_can_route_vst3_sandbox_through_broker_process() {
    let _guard = SandboxBrokerEnvGuard::enable_for_workspace_cargo_run();
    let runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
    let mut host = LocalRuntimeHost::new(runtime);
    let scan_root = temp_public_local_vst3_scan_root();

    host.start_plugin_scan(PluginScanRequest {
        roots: vec![scan_root.root()],
        formats: vec![PluginFormat::Vst3],
    })
    .expect("public local broker-backed vst3 scan should succeed");
    host.ensure_plugin_sandbox(PluginSandboxSpec {
        sandbox_id: "public-host-edge-local-vst3-broker".into(),
        plugin_format: PluginFormat::Vst3,
        plugin_type_id: Some("plugin:vst3:instrument".into()),
    })
    .expect("public local broker-backed vst3 sandbox ensure should succeed");

    let report = host.supervisor_report();
    let sandbox = report
        .observation
        .plugin_lifecycle_snapshot
        .sandboxes
        .iter()
        .find(|sandbox| sandbox.sandbox_id == "public-host-edge-local-vst3-broker")
        .expect("broker-backed local sandbox should be exported");
    assert_eq!(
        sandbox.lifecycle_stage,
        Some(PluginSandboxLifecycleStage::TransportAttached)
    );
    assert_eq!(
        sandbox.transport_stage,
        Some(PluginSandboxTransportStage::Attached)
    );
    assert_eq!(sandbox.readiness_state.as_deref(), Some("Ready"));

    host.teardown_plugin_sandbox("public-host-edge-local-vst3-broker")
        .expect("broker-backed local sandbox teardown should succeed");

    let report = host.supervisor_report();
    let sandbox = report
        .observation
        .plugin_lifecycle_snapshot
        .sandboxes
        .iter()
        .find(|sandbox| sandbox.sandbox_id == "public-host-edge-local-vst3-broker")
        .expect("broker-backed local sandbox should stay exported");
    assert_eq!(
        sandbox.lifecycle_stage,
        Some(PluginSandboxLifecycleStage::InstanceDestroyed)
    );
    assert_eq!(
        sandbox.transport_stage,
        Some(PluginSandboxTransportStage::Detached)
    );
}

#[test]
fn local_public_host_edge_resets_vst3_continuity_after_broker_reattach() {
    let _guard = SandboxBrokerEnvGuard::enable_for_workspace_cargo_run();
    let runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
    let mut host = LocalRuntimeHost::new(runtime);
    let scan_root = temp_public_local_vst3_scan_root();

    host.start_plugin_scan(PluginScanRequest {
        roots: vec![scan_root.root()],
        formats: vec![PluginFormat::Vst3],
    })
    .expect("public local broker-backed vst3 scan should succeed");

    let sandbox_id = "public-host-edge-local-vst3-broker-reset";
    host.ensure_plugin_sandbox(PluginSandboxSpec {
        sandbox_id: sandbox_id.into(),
        plugin_format: PluginFormat::Vst3,
        plugin_type_id: Some("plugin:vst3:instrument".into()),
    })
    .expect("first public local broker-backed vst3 sandbox ensure should succeed");
    host.teardown_plugin_sandbox(sandbox_id)
        .expect("first broker-backed local sandbox teardown should succeed");

    host.ensure_plugin_sandbox(PluginSandboxSpec {
        sandbox_id: sandbox_id.into(),
        plugin_format: PluginFormat::Vst3,
        plugin_type_id: Some("plugin:vst3:instrument".into()),
    })
    .expect("second public local broker-backed vst3 sandbox ensure should succeed");

    let report = host.supervisor_report();
    let sandbox = report
        .observation
        .plugin_lifecycle_snapshot
        .sandboxes
        .iter()
        .find(|sandbox| sandbox.sandbox_id == sandbox_id)
        .expect("broker-backed local reset sandbox should be exported");
    assert_eq!(
        sandbox.lifecycle_stage,
        Some(PluginSandboxLifecycleStage::TransportAttached)
    );
    assert_eq!(
        sandbox.transport_stage,
        Some(PluginSandboxTransportStage::Attached)
    );

    host.teardown_plugin_sandbox(sandbox_id)
        .expect("second broker-backed local sandbox teardown should succeed");
}

#[test]
fn local_public_host_edge_can_route_au_sandbox_through_broker_process() {
    let _guard = SandboxBrokerEnvGuard::enable_for_workspace_cargo_run();
    let runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
    let mut host = LocalRuntimeHost::new(runtime);
    let scan_root = temp_public_local_au_scan_root();

    host.start_plugin_scan(PluginScanRequest {
        roots: vec![scan_root.root()],
        formats: vec![PluginFormat::Au],
    })
    .expect("public local broker-backed au scan should succeed");
    host.ensure_plugin_sandbox(PluginSandboxSpec {
        sandbox_id: "public-host-edge-local-au-broker".into(),
        plugin_format: PluginFormat::Au,
        plugin_type_id: Some("plugin:au:instrument".into()),
    })
    .expect("public local broker-backed au sandbox ensure should succeed");

    let report = host.supervisor_report();
    let sandbox = report
        .observation
        .plugin_lifecycle_snapshot
        .sandboxes
        .iter()
        .find(|sandbox| sandbox.sandbox_id == "public-host-edge-local-au-broker")
        .expect("broker-backed local au sandbox should be exported");
    assert_eq!(
        sandbox.lifecycle_stage,
        Some(PluginSandboxLifecycleStage::TransportAttached)
    );
    assert_eq!(
        sandbox.transport_stage,
        Some(PluginSandboxTransportStage::Attached)
    );
    assert_eq!(sandbox.readiness_state.as_deref(), Some("Ready"));

    host.teardown_plugin_sandbox("public-host-edge-local-au-broker")
        .expect("broker-backed local au sandbox teardown should succeed");

    let report = host.supervisor_report();
    let sandbox = report
        .observation
        .plugin_lifecycle_snapshot
        .sandboxes
        .iter()
        .find(|sandbox| sandbox.sandbox_id == "public-host-edge-local-au-broker")
        .expect("broker-backed local au sandbox should stay exported");
    assert_eq!(
        sandbox.lifecycle_stage,
        Some(PluginSandboxLifecycleStage::InstanceDestroyed)
    );
    assert_eq!(
        sandbox.transport_stage,
        Some(PluginSandboxTransportStage::Detached)
    );
}
