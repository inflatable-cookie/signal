#[path = "support/public_host_edge_plugins.rs"]
mod public_host_edge_plugins_support;

use public_host_edge_plugins_support::{
    temp_public_local_au_scan_root, temp_public_local_clap_scan_root,
    temp_public_local_vst3_scan_root,
};
use signal_host_local::LocalRuntimeHost;
use signal_plugin::PluginFormat;
use signal_runtime::{
    PluginSandboxLifecycleStage, PluginSandboxSpec, PluginScanRequest, RuntimeConfig,
    RuntimePluginHostPlatform, RuntimePluginParityBand, RuntimeSupervisorApi, SignalRuntime,
};

#[test]
fn local_shared_host_edge_exports_runtime_cross_adapter_parity_truth() {
    let runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
    let mut host = LocalRuntimeHost::new(runtime);
    let clap_root = temp_public_local_clap_scan_root();
    let vst3_root = temp_public_local_vst3_scan_root();
    let au_root = temp_public_local_au_scan_root();

    host.start_plugin_scan(PluginScanRequest {
        roots: vec![clap_root.root(), vst3_root.root(), au_root.root()],
        formats: vec![PluginFormat::Clap, PluginFormat::Vst3, PluginFormat::Au],
    })
    .expect("public local parity scan should succeed");
    host.ensure_plugin_sandbox(PluginSandboxSpec {
        sandbox_id: "public-host-edge-local-parity-vst3".into(),
        plugin_format: PluginFormat::Vst3,
        plugin_type_id: Some("plugin:vst3:instrument".into()),
    })
    .expect("public local parity vst3 sandbox ensure should succeed");
    host.ensure_plugin_sandbox(PluginSandboxSpec {
        sandbox_id: "public-host-edge-local-parity-au".into(),
        plugin_format: PluginFormat::Au,
        plugin_type_id: Some("plugin:au:instrument".into()),
    })
    .expect("public local parity au sandbox ensure should succeed");

    let report = host.supervisor_report();
    let discovery = &report.observation.plugin_discovery_snapshot;
    assert_eq!(discovery.parity_coverage.len(), 3);
    let clap_parity = discovery
        .parity_coverage
        .iter()
        .find(|record| record.format == PluginFormat::Clap)
        .expect("public local parity report should include clap parity");
    assert_eq!(clap_parity.parity_band, RuntimePluginParityBand::Portable);
    assert_eq!(
        clap_parity.supported_platforms,
        vec![
            RuntimePluginHostPlatform::MacOs,
            RuntimePluginHostPlatform::Linux,
            RuntimePluginHostPlatform::Windows,
        ]
    );
    let au_parity = discovery
        .parity_coverage
        .iter()
        .find(|record| record.format == PluginFormat::Au)
        .expect("public local parity report should include au parity");
    assert_eq!(au_parity.parity_band, RuntimePluginParityBand::Guarded);
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
    let lifecycle_au = report
        .observation
        .plugin_lifecycle_snapshot
        .parity_coverage
        .iter()
        .find(|record| record.format == PluginFormat::Au)
        .expect("public local parity lifecycle should include au parity");
    assert_eq!(lifecycle_au.sandbox_count, 1);
    assert_eq!(lifecycle_au.ready_sandbox_count, 1);
    assert_eq!(lifecycle_au.active_transport_count, 1);
    let lifecycle_vst3 = report
        .observation
        .plugin_lifecycle_snapshot
        .parity_coverage
        .iter()
        .find(|record| record.format == PluginFormat::Vst3)
        .expect("public local parity lifecycle should include vst3 parity");
    assert_eq!(
        lifecycle_vst3.parity_band,
        RuntimePluginParityBand::Portable
    );
    assert_eq!(lifecycle_vst3.sandbox_count, 1);
    assert_eq!(lifecycle_vst3.ready_sandbox_count, 1);
    assert_eq!(lifecycle_vst3.active_transport_count, 1);
}

#[test]
fn local_shared_host_edge_exports_bounded_clap_sandbox_lifecycle_truth() {
    let runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
    let mut host = LocalRuntimeHost::new(runtime);
    let clap_root = temp_public_local_clap_scan_root();

    host.start_plugin_scan(PluginScanRequest {
        roots: vec![clap_root.root()],
        formats: vec![PluginFormat::Clap],
    })
    .expect("public local clap scan should succeed");
    // Report what the scan actually found when the ensure fails. Discovery
    // returns an empty list rather than an error when a fixture will not load,
    // so the bare ensure failure ("plugin type was not discovered in the last
    // local CLAP scan") says nothing about whether the scan saw no files, saw
    // the files but could not load them, or loaded something unexpected. This
    // failed once on CI and never in twenty local runs, so the next occurrence
    // needs to carry its own evidence.
    if let Err(error) = host.ensure_plugin_sandbox(PluginSandboxSpec {
        sandbox_id: "public-host-edge-local-clap-gap".into(),
        plugin_format: PluginFormat::Clap,
        plugin_type_id: Some("plugin:clap:default".into()),
    }) {
        let report = host.supervisor_report();
        let discovered: Vec<_> = report
            .observation
            .plugin_discovery_snapshot
            .discovered_types
            .iter()
            .map(|plugin| format!("{} ({:?})", plugin.plugin_type_id, plugin.format))
            .collect();
        let root_entries: Vec<String> = std::fs::read_dir(clap_root.root())
            .map(|entries| {
                entries
                    .filter_map(|entry| entry.ok())
                    .map(|entry| {
                        let length = entry.metadata().map(|meta| meta.len()).unwrap_or(0);
                        format!("{} ({length} bytes)", entry.file_name().to_string_lossy())
                    })
                    .collect()
            })
            .unwrap_or_else(|error| vec![format!("<unreadable: {error}>")]);
        panic!(
            "public local clap sandbox ensure should succeed: {error:?}\n\
             discovered types: {discovered:?}\n\
             scan root {}: {root_entries:?}",
            clap_root.root(),
        );
    }

    let report = host.supervisor_report();
    assert_eq!(
        report
            .observation
            .plugin_discovery_snapshot
            .last_scan
            .as_ref()
            .map(|scan| scan.formats.clone()),
        Some(vec![PluginFormat::Clap])
    );
    assert!(report
        .observation
        .plugin_discovery_snapshot
        .discovered_types
        .iter()
        .any(|plugin| plugin.plugin_type_id == "plugin:clap:default"
            && plugin.format == PluginFormat::Clap));

    let sandbox = report
        .observation
        .plugin_lifecycle_snapshot
        .sandboxes
        .iter()
        .find(|sandbox| sandbox.sandbox_id == "public-host-edge-local-clap-gap")
        .expect("public local clap sandbox should be exported");
    assert_eq!(sandbox.plugin_format, Some(PluginFormat::Clap));
    assert_eq!(
        sandbox.lifecycle_stage,
        Some(PluginSandboxLifecycleStage::TransportAttached)
    );
    assert_eq!(
        sandbox.state,
        signal_runtime::RuntimePluginLifecycleState::Ready
    );
    assert!(sandbox.active_transport);
    assert!(sandbox.active);
    assert!(sandbox.last_fault_kind.is_none());
    assert!(sandbox.last_fault_detail.is_none());
    assert!(sandbox.active_lease_id.is_some());
    assert!(sandbox.active_region_id.is_some());
}
