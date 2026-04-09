#[path = "support/public_host_edge_plugins.rs"]
mod public_host_edge_plugins_support;

use public_host_edge_plugins_support::temp_public_server_lv2_scan_root;
use signal_host_server::ServerRuntimeHost;
use signal_plugin::PluginFormat;
use signal_runtime::{
    PluginSandboxLifecycleStage, PluginSandboxSpec, PluginSandboxTransportStage, PluginScanRequest,
    RuntimeConfig, RuntimePluginHostPlatform, RuntimeSupervisorApi, SignalRuntime,
};

#[test]
fn server_shared_host_edge_exports_runtime_lv2_baseline_truth() {
    let runtime = SignalRuntime::new(RuntimeConfig::server(48_000, 512));
    let mut host = ServerRuntimeHost::new(runtime);
    let scan_root = temp_public_server_lv2_scan_root();

    host.start_plugin_scan(PluginScanRequest {
        roots: vec![scan_root.root()],
        formats: vec![PluginFormat::Lv2],
    })
    .expect("public server lv2 scan should succeed");
    host.ensure_plugin_sandbox(PluginSandboxSpec {
        sandbox_id: "public-host-edge-server-lv2".into(),
        plugin_format: PluginFormat::Lv2,
        plugin_type_id: Some("plugin:lv2:linux-synth".into()),
    })
    .expect("public server lv2 sandbox ensure should succeed");

    let report = host.supervisor_report();
    assert_eq!(
        report
            .observation
            .plugin_discovery_snapshot
            .discovered_type_count,
        5
    );
    assert_eq!(
        report
            .observation
            .plugin_discovery_snapshot
            .last_scan
            .as_ref()
            .map(|scan| scan.formats.clone()),
        Some(vec![PluginFormat::Lv2])
    );
    assert_eq!(
        report
            .observation
            .plugin_discovery_snapshot
            .last_scan
            .as_ref()
            .map(|scan| scan.discovery_diagnostic_count),
        Some(2)
    );
    assert!(report
        .observation
        .plugin_discovery_snapshot
        .last_scan
        .as_ref()
        .is_some_and(|scan| scan.discovery_diagnostics.iter().any(|diagnostic| {
            diagnostic.kind == signal_runtime::RuntimePluginScanDiagnosticKind::MalformedManifest
                && diagnostic.bundle_root.ends_with("Broken Manifest.lv2")
        })));
    assert!(report
        .observation
        .plugin_discovery_snapshot
        .last_scan
        .as_ref()
        .is_some_and(|scan| scan.discovery_diagnostics.iter().any(|diagnostic| {
            diagnostic.kind
                == signal_runtime::RuntimePluginScanDiagnosticKind::UnsupportedRequiredFeature
                && diagnostic.plugin_type_id.as_deref() == Some("plugin:lv2:unsupported-public")
        })));
    assert!(report
        .observation
        .plugin_discovery_snapshot
        .discovered_types
        .iter()
        .any(|plugin| plugin.plugin_type_id == "plugin:lv2:linux-synth"
            && plugin.format == PluginFormat::Lv2));
    let parity = report
        .observation
        .plugin_discovery_snapshot
        .parity_coverage
        .iter()
        .find(|record| record.format == PluginFormat::Lv2)
        .expect("public server lv2 parity should be exported");
    assert_eq!(
        parity.supported_platforms,
        vec![RuntimePluginHostPlatform::Linux]
    );
    assert_eq!(
        parity.unsupported_platforms,
        vec![
            RuntimePluginHostPlatform::MacOs,
            RuntimePluginHostPlatform::Windows,
        ]
    );
    let sandbox = report
        .observation
        .plugin_lifecycle_snapshot
        .sandboxes
        .iter()
        .find(|sandbox| sandbox.sandbox_id == "public-host-edge-server-lv2")
        .expect("public server lv2 sandbox should be exported");
    assert_eq!(sandbox.plugin_format, Some(PluginFormat::Lv2));
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
            .map(|record| record.urid_negotiation_posture),
        Some(signal_runtime::RuntimeLv2UridNegotiationPosture::Negotiated)
    );

    let rendered = report.render_json();
    assert!(rendered.contains("\"plugin_type_id\":\"plugin:lv2:linux-synth\""));
    assert!(rendered.contains("\"formats\":[\"Lv2\"]"));
    assert!(rendered.contains("\"discovery_diagnostic_count\":2"));
    assert!(rendered.contains("\"kind\":\"MalformedManifest\""));
    assert!(rendered.contains("\"kind\":\"UnsupportedRequiredFeature\""));
    assert!(rendered.contains("\"lv2_prepared_negotiation\":{"));
    assert!(rendered.contains("\"supported_platforms\":[\"Linux\"]"));
}
