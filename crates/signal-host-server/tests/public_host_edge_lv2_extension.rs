#[path = "support/public_host_edge_plugins.rs"]
mod public_host_edge_plugins_support;

use public_host_edge_plugins_support::temp_public_server_lv2_scan_root;
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
    let scan_root = temp_public_server_lv2_scan_root();
    host.start_plugin_scan(PluginScanRequest {
        roots: vec![scan_root.root()],
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
        5
    );
    assert_eq!(
        report
            .observation
            .lv2_extension_snapshot
            .worker_required_type_count,
        3
    );
    assert_eq!(
        report
            .observation
            .lv2_extension_snapshot
            .patch_supported_type_count,
        4
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
    let sandbox = report
        .observation
        .plugin_lifecycle_snapshot
        .sandboxes
        .iter()
        .find(|sandbox| sandbox.sandbox_id == "public-server-lv2-extension")
        .expect("server lv2 sandbox should be visible");
    assert_eq!(
        sandbox
            .lv2_prepared_negotiation
            .as_ref()
            .map(|record| record.summary.as_str()),
        Some(
            "worker=WorkerRequiredAvailable urid=Negotiated patch=Supported negotiation=Negotiated"
        )
    );

    let rendered = report.render_json();
    assert!(rendered.contains("\"lv2_extension_snapshot\":{"));
    assert!(rendered.contains("\"worker_posture\":\"WorkerRequiredAvailable\""));
    assert!(rendered.contains("\"patch_exchange_posture\":\"Supported\""));
    assert!(rendered.contains("\"discovery_diagnostic_count\":2"));
    assert!(rendered.contains("\"lv2_prepared_negotiation\":{"));
}

#[test]
fn server_shared_host_edge_exports_runtime_lv2_unavailable_negotiation_truth() {
    let runtime = SignalRuntime::new(RuntimeConfig::server(48_000, 512));
    let mut host = ServerRuntimeHost::new(runtime);
    let scan_root = temp_public_server_lv2_scan_root();
    host.start_plugin_scan(PluginScanRequest {
        roots: vec![scan_root.root()],
        formats: vec![PluginFormat::Lv2],
    })
    .expect("public server lv2 extension scan should succeed");

    let error = host
        .ensure_plugin_sandbox(PluginSandboxSpec {
            sandbox_id: "public-server-lv2-worker-unavailable".into(),
            plugin_format: PluginFormat::Lv2,
            plugin_type_id: Some("plugin:lv2:worker-unavailable-public".into()),
        })
        .expect_err("worker-unavailable lv2 sandbox should fail");
    assert_eq!(error.kind, signal_runtime::RuntimeErrorKind::InvalidRequest);

    let report = host.supervisor_report();
    let record = report
        .observation
        .lv2_extension_snapshot
        .records
        .iter()
        .find(|record| record.plugin_type_id == "plugin:lv2:worker-unavailable-public")
        .expect("unavailable server lv2 extension record should be visible");
    assert_eq!(
        record.worker_posture,
        RuntimeLv2WorkerPosture::WorkerUnavailable
    );
    assert_eq!(
        record.extension_negotiation_state,
        RuntimeLv2ExtensionNegotiationState::Unavailable
    );
    let sandbox = report
        .observation
        .plugin_lifecycle_snapshot
        .sandboxes
        .iter()
        .find(|sandbox| sandbox.sandbox_id == "public-server-lv2-worker-unavailable")
        .expect("faulted lv2 sandbox should be visible");
    assert_eq!(
        sandbox
            .lv2_prepared_negotiation
            .as_ref()
            .map(|record| record.worker_posture),
        Some(RuntimeLv2WorkerPosture::WorkerUnavailable)
    );
    assert_eq!(sandbox.readiness_state.as_deref(), Some("Faulted"));
    assert!(sandbox
        .degraded_reasons
        .iter()
        .any(|reason| reason.contains("WorkerUnavailable")));

    let rendered = report.render_json();
    assert!(rendered.contains("\"plugin_type_id\":\"plugin:lv2:worker-unavailable-public\""));
    assert!(rendered.contains("\"worker_posture\":\"WorkerUnavailable\""));
    assert!(rendered.contains("\"extension_negotiation_state\":\"Unavailable\""));
    assert!(rendered.contains("\"readiness_state\":\"Faulted\""));
}
