use signal_host_server::ServerRuntimeHost;
use signal_plugin::PluginFormat;
use signal_runtime::{
    PluginSandboxSpec, PluginScanRequest, RuntimeConfig, RuntimeLv2ExtensionNegotiationState,
    RuntimeLv2PatchExchangePosture, RuntimeLv2UridNegotiationPosture, RuntimeLv2WorkerPosture,
    RuntimeSupervisorApi, SignalRuntime,
};

#[test]
fn server_shared_host_edge_exports_runtime_lv2_extension_truth() {
    let runtime = SignalRuntime::new(RuntimeConfig::server(48_000, 512));
    let mut host = ServerRuntimeHost::new(runtime);
    host.start_plugin_scan(PluginScanRequest {
        roots: vec!["~/.lv2".into(), "/usr/lib/lv2".into()],
        formats: vec![PluginFormat::Lv2],
    })
    .expect("public server lv2 extension scan should succeed");
    host.ensure_plugin_sandbox(PluginSandboxSpec {
        sandbox_id: "public-server-lv2-extension".into(),
        plugin_format: PluginFormat::Lv2,
        plugin_type_id: Some("plugin:lv2:linux-synth".into()),
    })
    .expect("public server lv2 extension sandbox should succeed");

    let report = host.supervisor_report();
    assert_eq!(
        report.observation.lv2_extension_snapshot.plugin_type_count,
        4
    );
    assert_eq!(
        report
            .observation
            .lv2_extension_snapshot
            .worker_required_type_count,
        2
    );
    assert_eq!(
        report
            .observation
            .lv2_extension_snapshot
            .patch_supported_type_count,
        3
    );
    let record = report
        .observation
        .lv2_extension_snapshot
        .records
        .iter()
        .find(|record| record.plugin_type_id == "plugin:lv2:linux-synth")
        .expect("server lv2 extension record should be visible");
    assert_eq!(
        record.worker_posture,
        RuntimeLv2WorkerPosture::WorkerRequiredAvailable
    );
    assert_eq!(
        record.urid_negotiation_posture,
        RuntimeLv2UridNegotiationPosture::Negotiated
    );
    assert_eq!(
        record.patch_exchange_posture,
        RuntimeLv2PatchExchangePosture::Supported
    );
    assert_eq!(
        record.extension_negotiation_state,
        RuntimeLv2ExtensionNegotiationState::Negotiated
    );

    let rendered = report.render_json();
    assert!(rendered.contains("\"lv2_extension_snapshot\":{"));
    assert!(rendered.contains("\"worker_posture\":\"WorkerRequiredAvailable\""));
    assert!(rendered.contains("\"patch_exchange_posture\":\"Supported\""));
}
