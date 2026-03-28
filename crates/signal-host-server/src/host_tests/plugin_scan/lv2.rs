use super::super::super::ServerRuntimeHost;
use signal_plugin::PluginFormat;
use signal_runtime::{
    PluginSandboxLifecycleStage, PluginSandboxSpec, PluginSandboxTransportStage,
    PluginScanRequest, RuntimeConfig, RuntimePluginHostPlatform,
    RuntimePluginIsolationOutcome, RuntimePluginParityBand, RuntimeSupervisorApi,
    SignalRuntime,
};

#[test]
fn server_host_lv2_scan_and_sandbox_surface_linux_runtime_owned_receipts() {
    let runtime = SignalRuntime::new(RuntimeConfig::server(48_000, 512));
    let mut host = ServerRuntimeHost::new(runtime);

    host.start_plugin_scan(PluginScanRequest {
        roots: vec!["~/.lv2".into(), "/usr/lib/lv2".into()],
        formats: vec![PluginFormat::Lv2],
    })
    .expect("server lv2 plugin scan");
    host.ensure_plugin_sandbox(PluginSandboxSpec {
        sandbox_id: "server-lv2-sandbox".into(),
        plugin_format: PluginFormat::Lv2,
        plugin_type_id: Some("plugin:lv2:linux-synth".into()),
    })
    .expect("server lv2 sandbox ensure");

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
        Some(vec![PluginFormat::Lv2])
    );
    assert!(report
        .observation
        .plugin_discovery_snapshot
        .discovered_types
        .iter()
        .any(|plugin| plugin.plugin_type_id == "plugin:lv2:linux-synth"
            && plugin.format == PluginFormat::Lv2
            && plugin.default_io_layout.midi_inputs == 1));
    assert!(report
        .observation
        .plugin_discovery_snapshot
        .discovered_types
        .iter()
        .any(
            |plugin| plugin.plugin_type_id == "plugin:lv2:multiout-instrument"
                && plugin.complex_io_summary.multi_output_instrument
                && plugin.complex_io_summary.instrument_output_group_count >= 2
        ));
    assert!(report
        .observation
        .plugin_discovery_snapshot
        .discovered_types
        .iter()
        .any(|plugin| plugin.plugin_type_id == "plugin:lv2:bus-fx"
            && plugin.complex_io_summary.bus_capable_fx_class.is_some()));
    let sandbox = report
        .observation
        .plugin_lifecycle_snapshot
        .sandboxes
        .iter()
        .find(|sandbox| sandbox.sandbox_id == "server-lv2-sandbox")
        .expect("server lv2 sandbox should be exported");
    assert_eq!(sandbox.plugin_format, Some(PluginFormat::Lv2));
    assert_eq!(
        sandbox.plugin_type_id.as_deref(),
        Some("plugin:lv2:linux-synth")
    );
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
    let lv2_parity = report
        .observation
        .plugin_discovery_snapshot
        .parity_coverage
        .iter()
        .find(|record| record.format == PluginFormat::Lv2)
        .expect("server lv2 parity should be present");
    assert_eq!(
        lv2_parity.supported_platforms,
        vec![RuntimePluginHostPlatform::Linux]
    );
    assert_eq!(
        lv2_parity.unsupported_platforms,
        vec![
            RuntimePluginHostPlatform::MacOs,
            RuntimePluginHostPlatform::Windows,
        ]
    );
    assert_eq!(lv2_parity.discovered_type_count, 4);
    assert_eq!(lv2_parity.sandbox_count, 1);
    assert_eq!(
        lv2_parity.linux_parity_band,
        RuntimePluginParityBand::Portable
    );
    assert!(lv2_parity.linux_supported);
    assert_eq!(
        lv2_parity.linux_preferred_sandbox_outcome,
        Some(RuntimePluginIsolationOutcome::IsolatedSandbox)
    );
    assert!(lv2_parity.linux_strict_sandbox_default);
    assert!(lv2_parity.prepare_capable_type_count >= 1);
    assert!(lv2_parity.activate_capable_type_count >= 1);

    let rendered = report.render_json();
    assert!(rendered.contains("\"plugin_format\":\"Lv2\""));
    assert!(rendered.contains("\"formats\":[\"Lv2\"]"));
    assert!(rendered.contains("\"transport_stage\":\"Attached\""));
    assert!(rendered.contains("\"linux_parity_band\":\"Portable\""));
    assert!(rendered.contains("\"linux_preferred_sandbox_outcome\":\"IsolatedSandbox\""));
}
