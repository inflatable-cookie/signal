#[path = "support/public_host_edge_continuity.rs"]
mod public_host_edge_continuity_support;

use public_host_edge_continuity_support::{
    apply_public_plugin_continuity_graph, record_public_plugin_sandbox_ready,
};
use signal_host_local::LocalRuntimeHost;
use signal_plugin::PluginFormat;
use signal_runtime::{
    PluginFaultKind, RuntimeConfig, RuntimeConfigRequest, RuntimeInterruptionClass,
    RuntimeLifecycleApi, RuntimePluginIsolationOutcome, RuntimePluginPlacementPolicy,
    RuntimePluginPlacementRule, RuntimePluginPlacementRuleMatcher, RuntimeProjectionApi,
    SignalRuntime,
};

#[test]
fn local_shared_host_edge_exports_plugin_placement_and_shared_boundary_continuity_truth() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
    runtime
        .handshake(signal_runtime::HandshakeRequest {
            client_version: "public-host-edge-plugin-continuity".into(),
            anticipative_preferred: true,
            max_sample_rate_hint: Some(96_000),
        })
        .unwrap();
    runtime
        .configure(RuntimeConfigRequest::new(48_000, 512))
        .unwrap();
    runtime
        .apply_plugin_placement_policy(RuntimePluginPlacementPolicy {
            default_outcome: RuntimePluginIsolationOutcome::IsolatedSandbox,
            rules: vec![RuntimePluginPlacementRule {
                rule_id: "share-verified-clap".into(),
                matcher: RuntimePluginPlacementRuleMatcher::PluginTypeId(
                    "plugin://host-local-shared".into(),
                ),
                outcome: RuntimePluginIsolationOutcome::SharedSandbox,
                sandbox_group_key: Some("shared:host-local".into()),
            }],
        })
        .unwrap();
    apply_public_plugin_continuity_graph(
        &mut runtime,
        "graph:host-local:plugin-continuity",
        &[
            ("plugin-a", "sandbox-shared"),
            ("plugin-b", "sandbox-shared"),
            ("plugin-c", "sandbox-isolated"),
        ],
    );
    record_public_plugin_sandbox_ready(
        &mut runtime,
        "sandbox-shared",
        PluginFormat::Clap,
        "plugin://host-local-shared",
        1,
    );
    record_public_plugin_sandbox_ready(
        &mut runtime,
        "sandbox-isolated",
        PluginFormat::Clap,
        "plugin://host-local-isolated",
        1,
    );
    runtime.record_plugin_sandbox_fault(
        "sandbox-shared",
        PluginFaultKind::Crash,
        "local shared crash",
        Some(2),
    );
    runtime.record_plugin_sandbox_fault(
        "sandbox-shared",
        PluginFaultKind::Timeout,
        "local shared timeout",
        Some(3),
    );

    let host = LocalRuntimeHost::new(runtime);
    let report = host.supervisor_report();
    let shared = report
        .observation
        .plugin_lifecycle_snapshot
        .sandboxes
        .iter()
        .find(|sandbox| sandbox.sandbox_id == "sandbox-shared")
        .expect("shared host-local boundary should be visible");
    assert_eq!(
        shared.placement_outcome,
        RuntimePluginIsolationOutcome::SharedSandbox
    );
    assert_eq!(
        shared.placement_rule_id.as_deref(),
        Some("share-verified-clap")
    );
    assert_eq!(shared.sandbox_group_key, "shared:host-local");
    assert_eq!(shared.shared_boundary_member_count, 2);
    assert_eq!(shared.continuity_class, RuntimeInterruptionClass::Terminal);
    assert!(!shared.rebindable);
    let isolated = report
        .observation
        .plugin_lifecycle_snapshot
        .sandboxes
        .iter()
        .find(|sandbox| sandbox.sandbox_id == "sandbox-isolated")
        .expect("isolated host-local boundary should remain visible");
    assert_eq!(isolated.continuity_class, RuntimeInterruptionClass::Steady);

}
