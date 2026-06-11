use super::*;

#[test]
fn local_shared_host_edge_is_consumable_without_private_helpers() {
    let _demo_override = signal_host_local::ensure_default_demo_plugin_override();
    let demo_root =
        std::env::var("SIGNAL_HOST_DEMO_PLUGIN_ROOT").expect("demo override should set root");
    let plugin_type_id = std::env::var("SIGNAL_HOST_DEMO_PLUGIN_TYPE_ID")
        .expect("demo override should set plugin type");
    let runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
    let mut host = LocalRuntimeHost::new(runtime);

    host.start_plugin_scan(PluginScanRequest {
        roots: vec![demo_root],
        formats: vec![PluginFormat::Vst3],
    })
    .expect("public host-edge scan should succeed");
    host.ensure_plugin_sandbox(PluginSandboxSpec {
        sandbox_id: "public-host-edge-local".into(),
        plugin_format: PluginFormat::Vst3,
        plugin_type_id: Some(plugin_type_id),
    })
    .expect("public host-edge sandbox ensure should succeed");

    let report = host.supervisor_report();
    assert_eq!(report.observation.plugin_discovery_snapshot.scan_count, 1);
    assert_eq!(
        report
            .observation
            .plugin_discovery_snapshot
            .discovered_type_count,
        1
    );
    assert_eq!(
        report.observation.plugin_lifecycle_snapshot.sandboxes.len(),
        1
    );
    assert_eq!(
        report.observation.plugin_lifecycle_snapshot.sandboxes[0].plugin_format,
        Some(PluginFormat::Vst3)
    );
    assert_eq!(
        report.observation.fault_status.recovery_state,
        RuntimeRecoveryState::Steady
    );
    assert_eq!(
        report.observation.interruption_summary.class,
        RuntimeInterruptionClass::Steady
    );
    assert_eq!(
        report.observation.fault_diagnostic_receipt.primary_family,
        None
    );

}
