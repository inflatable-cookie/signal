use super::super::super::super::*;

#[test]
fn local_host_vst3_scan_and_sandbox_surface_runtime_owned_receipts() {
    let runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
    let mut host = LocalRuntimeHost::new(runtime);

    host.start_plugin_scan(PluginScanRequest {
        roots: vec!["~/Library/Audio/Plug-Ins/VST3".into()],
        formats: vec![PluginFormat::Vst3],
    })
    .expect("vst3 plugin scan");
    host.ensure_plugin_sandbox(PluginSandboxSpec {
        sandbox_id: "local-vst3-sandbox".into(),
        plugin_format: PluginFormat::Vst3,
        plugin_type_id: Some("plugin:vst3:instrument".into()),
    })
    .expect("vst3 sandbox ensure");

    let report = host.host_supervisor_report();
    assert_eq!(
        report.observation.observation.plugin_discovery_snapshot.discovered_type_count,
        4
    );
    assert_eq!(
        report
            .observation
            .observation
            .plugin_discovery_snapshot
            .last_scan
            .as_ref()
            .map(|scan| scan.formats.clone()),
        Some(vec![PluginFormat::Vst3])
    );
    assert!(report
        .observation
        .observation
        .plugin_discovery_snapshot
        .discovered_types
        .iter()
        .any(|plugin| plugin.plugin_type_id == "plugin:vst3:instrument"
            && plugin.format == PluginFormat::Vst3
            && plugin.processing_contract.accepts_note_events));
    assert!(report
        .observation
        .observation
        .plugin_discovery_snapshot
        .discovered_types
        .iter()
        .any(
            |plugin| plugin.plugin_type_id == "plugin:vst3:multiout-instrument"
                && plugin.complex_io_summary.multi_output_instrument
                && plugin.complex_io_summary.instrument_output_group_count >= 2
        ));
    assert!(report
        .observation
        .observation
        .plugin_discovery_snapshot
        .discovered_types
        .iter()
        .any(|plugin| plugin.plugin_type_id == "plugin:vst3:bus-fx"
            && plugin.complex_io_summary.bus_capable_fx_class.is_some()));
    let sandbox = report
        .observation
        .observation
        .plugin_lifecycle_snapshot
        .sandboxes
        .iter()
        .find(|sandbox| sandbox.sandbox_id == "local-vst3-sandbox")
        .expect("local vst3 sandbox should be exported");
    assert_eq!(sandbox.plugin_format, Some(PluginFormat::Vst3));
    assert_eq!(
        sandbox.plugin_type_id.as_deref(),
        Some("plugin:vst3:instrument")
    );
    assert_eq!(
        sandbox.lifecycle_stage,
        Some(PluginSandboxLifecycleStage::TransportAttached)
    );
    assert_eq!(
        sandbox.transport_stage,
        Some(PluginSandboxTransportStage::Attached)
    );
    assert_eq!(sandbox.readiness_state.as_deref(), Some("Ready"));
    assert!(sandbox.active);
    assert!(sandbox.active_transport);
    let au_parity = report
        .observation
        .observation
        .plugin_discovery_snapshot
        .parity_coverage
        .iter()
        .find(|record| record.format == PluginFormat::Au)
        .expect("local au parity should be present");
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
    assert_eq!(au_parity.discovered_type_count, 0);
    assert_eq!(au_parity.sandbox_count, 0);
}

#[test]
fn local_host_au_scan_and_sandbox_surface_runtime_owned_receipts() {
    let runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
    let mut host = LocalRuntimeHost::new(runtime);

    host.start_plugin_scan(PluginScanRequest {
        roots: vec!["~/Library/Audio/Plug-Ins/Components".into()],
        formats: vec![PluginFormat::Au],
    })
    .expect("au plugin scan");
    host.ensure_plugin_sandbox(PluginSandboxSpec {
        sandbox_id: "local-au-sandbox".into(),
        plugin_format: PluginFormat::Au,
        plugin_type_id: Some("plugin:au:instrument".into()),
    })
    .expect("au sandbox ensure");

    let report = host.host_supervisor_report();
    assert_eq!(
        report.observation.observation.plugin_discovery_snapshot.discovered_type_count,
        4
    );
    assert_eq!(
        report
            .observation
            .observation
            .plugin_discovery_snapshot
            .last_scan
            .as_ref()
            .map(|scan| scan.formats.clone()),
        Some(vec![PluginFormat::Au])
    );
    assert!(report
        .observation
        .observation
        .plugin_discovery_snapshot
        .discovered_types
        .iter()
        .any(|plugin| plugin.plugin_type_id == "plugin:au:instrument"
            && plugin.format == PluginFormat::Au
            && plugin.processing_contract.accepts_note_events));
    assert!(report
        .observation
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
        .observation
        .plugin_discovery_snapshot
        .discovered_types
        .iter()
        .any(|plugin| plugin.plugin_type_id == "plugin:au:bus-fx"
            && plugin.complex_io_summary.bus_capable_fx_class.is_some()));
    let sandbox = report
        .observation
        .observation
        .plugin_lifecycle_snapshot
        .sandboxes
        .iter()
        .find(|sandbox| sandbox.sandbox_id == "local-au-sandbox")
        .expect("local au sandbox should be exported");
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
    assert_eq!(sandbox.readiness_state.as_deref(), Some("Ready"));
    assert!(sandbox.active);
    assert!(sandbox.active_transport);
}
