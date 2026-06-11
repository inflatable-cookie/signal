#[path = "support/public_host_edge_plugins.rs"]
mod public_host_edge_plugins_support;
#[path = "support/public_host_edge_sandbox_broker.rs"]
mod public_host_edge_sandbox_broker_support;

use public_host_edge_plugins_support::{
    temp_public_local_au_scan_root, temp_public_local_faulty_au_scan_root,
};
use public_host_edge_sandbox_broker_support::SandboxBrokerEnvGuard;
use signal_host_local::LocalRuntimeHost;
use signal_plugin::PluginFormat;
use signal_runtime::{
    PluginSandboxLifecycleStage, PluginSandboxSpec, PluginSandboxTransportStage, PluginScanRequest,
    RuntimeConfig, RuntimeErrorKind, RuntimeSupervisorApi, SignalRuntime,
};

#[test]
fn local_shared_host_edge_exports_runtime_au_baseline_truth() {
    let runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
    let mut host = LocalRuntimeHost::new(runtime);
    let scan_root = temp_public_local_au_scan_root();
    let _broker_guard = SandboxBrokerEnvGuard::enable_for_workspace_demo_plugin(
        "au",
        &scan_root.root(),
        "plugin:au:instrument",
    );

    host.start_plugin_scan(PluginScanRequest {
        roots: vec![scan_root.root()],
        formats: vec![PluginFormat::Au],
    })
    .expect("public local au scan should succeed");
    host.ensure_plugin_sandbox(PluginSandboxSpec {
        sandbox_id: "public-host-edge-local-au".into(),
        plugin_format: PluginFormat::Au,
        plugin_type_id: Some("plugin:au:instrument".into()),
    })
    .expect("public local au sandbox ensure should succeed");

    let report = host.supervisor_report();
    assert_eq!(
        report
            .observation
            .plugin_discovery_snapshot
            .discovered_type_count,
        4
    );
    assert_eq!(
        report
            .observation
            .plugin_discovery_snapshot
            .last_scan
            .as_ref()
            .map(|scan| scan.formats.clone()),
        Some(vec![PluginFormat::Au])
    );
    assert!(report
        .observation
        .plugin_discovery_snapshot
        .discovered_types
        .iter()
        .any(|plugin| plugin.plugin_type_id == "plugin:au:instrument"
            && plugin.format == PluginFormat::Au));
    let sandbox = report
        .observation
        .plugin_lifecycle_snapshot
        .sandboxes
        .iter()
        .find(|sandbox| sandbox.sandbox_id == "public-host-edge-local-au")
        .expect("public local au sandbox should be exported");
    assert_eq!(sandbox.plugin_format, Some(PluginFormat::Au));
    assert_eq!(
        sandbox.lifecycle_stage,
        Some(PluginSandboxLifecycleStage::TransportAttached)
    );
    assert_eq!(
        sandbox.transport_stage,
        Some(PluginSandboxTransportStage::Attached)
    );
    assert_eq!(sandbox.readiness_state.as_deref(), Some("Ready"));

}

#[test]
fn local_shared_host_edge_exports_runtime_au_fault_truth_alongside_coreaudio_truth() {
    let runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
    let mut host = LocalRuntimeHost::new(runtime);
    let scan_root = temp_public_local_faulty_au_scan_root();
    let _broker_guard = SandboxBrokerEnvGuard::enable_for_workspace_demo_plugin(
        "au",
        &scan_root.root(),
        "plugin:au:render-context-fault",
    );

    let error = host
        .boot_default()
        .expect_err("public local au fault boot should fail");
    assert_eq!(error.kind, RuntimeErrorKind::InvalidRequest);

    let report = host.host_supervisor_report();
    let sandbox = report
        .observation
        .observation
        .plugin_lifecycle_snapshot
        .sandboxes
        .iter()
        .find(|sandbox| sandbox.sandbox_id == "local-default-sandbox")
        .expect("faulted local au sandbox should be exported");
    assert_eq!(sandbox.plugin_format, Some(PluginFormat::Au));
    assert_eq!(sandbox.readiness_state.as_deref(), Some("Faulted"));
    assert_eq!(
        sandbox.last_fault_detail.as_deref(),
        Some(
            "au render context activation failed for plugin:au:render-context-fault: unsupported_sample_rate"
        )
    );
    assert!(report
        .observation
        .host_io
        .hardware
        .device_id
        .starts_with("coreaudio:"));

}
