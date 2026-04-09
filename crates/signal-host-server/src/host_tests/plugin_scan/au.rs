use super::super::super::ServerRuntimeHost;
use crate::host::host_test_support::temp_server_au_scan_root;
use signal_plugin::PluginFormat;
use signal_runtime::{
    PluginSandboxLifecycleStage, PluginSandboxSpec, PluginSandboxTransportStage,
    PluginScanRequest, RuntimeConfig, RuntimePluginHostPlatform, RuntimeSupervisorApi,
    SignalRuntime,
};

#[test]
fn server_host_au_scan_and_sandbox_surface_runtime_owned_receipts() {
    let runtime = SignalRuntime::new(RuntimeConfig::server(48_000, 512));
    let mut host = ServerRuntimeHost::new(runtime);
    let scan_root = temp_server_au_scan_root();

    host.start_plugin_scan(PluginScanRequest {
        roots: vec![scan_root.root()],
        formats: vec![PluginFormat::Au],
    })
    .expect("server au plugin scan");
    host.ensure_plugin_sandbox(PluginSandboxSpec {
        sandbox_id: "server-au-sandbox".into(),
        plugin_format: PluginFormat::Au,
        plugin_type_id: Some("plugin:au:instrument".into()),
    })
    .expect("server au sandbox ensure");

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
            && plugin.format == PluginFormat::Au
            && plugin.default_io_layout.midi_inputs == 1));
    assert!(report
        .observation
        .plugin_discovery_snapshot
        .discovered_types
        .iter()
        .any(
            |plugin| plugin.plugin_type_id == "plugin:au:multiout-instrument"
                && plugin.complex_io_summary.multi_output_instrument
                && plugin.complex_io_summary.instrument_output_group_count >= 2
        ));
    assert!(report
        .observation
        .plugin_discovery_snapshot
        .discovered_types
        .iter()
        .any(|plugin| plugin.plugin_type_id == "plugin:au:bus-fx"
            && plugin.complex_io_summary.bus_capable_fx_class.is_some()));
    let sandbox = report
        .observation
        .plugin_lifecycle_snapshot
        .sandboxes
        .iter()
        .find(|sandbox| sandbox.sandbox_id == "server-au-sandbox")
        .expect("server au sandbox should be exported");
    assert_eq!(sandbox.plugin_format, Some(PluginFormat::Au));
    assert_eq!(sandbox.plugin_type_id.as_deref(), Some("plugin:au:instrument"));
    assert_eq!(
        sandbox.lifecycle_stage,
        Some(PluginSandboxLifecycleStage::TransportAttached)
    );
    assert_eq!(
        sandbox.transport_stage,
        Some(PluginSandboxTransportStage::Attached)
    );
    assert!(sandbox.active);
    assert!(sandbox.active_transport);
    let au_parity = report
        .observation
        .plugin_discovery_snapshot
        .parity_coverage
        .iter()
        .find(|record| record.format == PluginFormat::Au)
        .expect("server au parity should be present");
    assert_eq!(
        au_parity.supported_platforms,
        vec![RuntimePluginHostPlatform::MacOs]
    );
    assert_eq!(
        au_parity.unsupported_platforms,
        vec![
            RuntimePluginHostPlatform::Linux,
            RuntimePluginHostPlatform::Windows,
        ]
    );
    assert_eq!(au_parity.discovered_type_count, 4);
    assert_eq!(au_parity.sandbox_count, 1);

    let rendered = report.render_json();
    assert!(rendered.contains("\"plugin_format\":\"Au\""));
    assert!(rendered.contains("\"formats\":[\"Au\"]"));
    assert!(rendered.contains("\"transport_stage\":\"Attached\""));
    assert!(rendered.contains("\"parity_coverage\":["));
    assert!(rendered.contains("\"parity_band\":\"Guarded\""));
    assert!(rendered.contains("\"supported_platforms\":[\"MacOs\"]"));
    assert!(rendered.contains("\"unsupported_platforms\":[\"Linux\",\"Windows\"]"));
}
