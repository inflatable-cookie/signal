// Unit tests for the local runtime host (included into `host::tests`).
use super::*;
use signal_plugin::PluginIsolationTier;
use signal_runtime::{RuntimeConfig, RuntimeErrorKind};

fn booted_host() -> (LocalRuntimeHost, LocalRuntimeHostSummary) {
    // Held across the boot. `SIGNAL_HOST_DEMO_PLUGIN_*` is process-global, and
    // one test in this module installs an override while the rest boot without
    // one — so an unguarded boot can otherwise scan a fixture root that is
    // being torn down and fail with "plugin type was not discovered".
    let _env_lock = demo_plugin_env_lock();
    let runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
    let mut host = LocalRuntimeHost::new(runtime);
    let summary = host.boot_default().expect("local host should boot");
    (host, summary)
}

#[test]
fn boot_default_reports_running_stream_and_topology() {
    let (host, summary) = booted_host();
    assert_eq!(summary.audio_pump.stream_state, LocalAudioStreamState::Running);
    assert!(summary.topology.node_count > 0);
    assert!(!summary.hardware.device_id.is_empty());
    let report = host.host_supervisor_report();
    assert!(!report.events.is_empty());
    assert_eq!(
        report
            .observation
            .observation
            .execution_topology_summary
            .node_count,
        summary.topology.node_count
    );
}

#[test]
fn boot_default_never_scans_system_plugin_directories() {
    // Note: a parallel test may have installed a fixture override via
    // `ensure_default_demo_plugin_override` (process-global env), so assert
    // the safety property directly: no real plugin directory is ever scanned.
    let (_host, summary) = booted_host();
    for root in &summary.scan_roots {
        assert!(
            !root.contains("Library/Audio/Plug-Ins") && !root.starts_with('~'),
            "default boot must not scan system plugin directories, got {root}"
        );
    }
}

#[test]
fn boot_with_demo_override_discovers_fixture_plugin_and_records_sandbox() {
    let _guard = ensure_default_demo_plugin_override();
    let runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
    let mut host = LocalRuntimeHost::new(runtime);
    let summary = host.boot_default().expect("local host should boot");
    assert_eq!(summary.scan_roots.len(), 1);
    assert!(!host.discovered_vst3_types.is_empty());
    let report = host.host_supervisor_report();
    assert!(report
        .observation
        .observation
        .plugin_discovery_snapshot
        .discovered_type_count
        > 0);
}

#[test]
fn ensure_plugin_sandbox_rejects_undiscovered_plugin_types() {
    let (mut host, _summary) = booted_host();
    let error = host
        .ensure_plugin_sandbox(signal_runtime::PluginSandboxSpec {
            sandbox_id: "missing-sandbox".into(),
            plugin_format: PluginFormat::Clap,
            plugin_type_id: Some("plugin:clap:not-discovered".into()),
        })
        .expect_err("undiscovered plugin type should be rejected");
    assert!(error.message.contains("not discovered"));
}

#[test]
fn prepare_plugin_processor_rejects_shared_sandbox() {
    let (mut host, _summary) = booted_host();
    let error = host
        .prepare_plugin_processor("plugin:clap:any", PluginIsolationTier::SharedSandbox)
        .expect_err("SharedSandbox should be a typed rejection");
    assert_eq!(error.kind, RuntimeErrorKind::UnsupportedCapability);
    assert!(error.message.contains("shared_sandbox_unimplemented"));
}

#[test]
fn prepare_plugin_processor_rejects_unscanned_plugin_types() {
    let (mut host, _summary) = booted_host();
    let error = host
        .prepare_plugin_processor("plugin:clap:not-discovered", PluginIsolationTier::InProcess)
        .expect_err("unscanned plugin type should be rejected");
    assert_eq!(error.kind, RuntimeErrorKind::ResourceUnavailable);
    assert!(error.message.contains("not discovered"));
}

#[test]
fn prepare_dedicated_sandbox_rejects_missing_broker_lease() {
    let _guard = ensure_default_demo_plugin_override();
    let runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
    let mut host = LocalRuntimeHost::new(runtime);
    host.boot_default().expect("local host should boot");
    let plugin_type_id = host
        .discovered_vst3_types
        .keys()
        .next()
        .expect("demo override should discover a VST3 type")
        .clone();
    let error = host
        .prepare_plugin_processor(&plugin_type_id, PluginIsolationTier::DedicatedSandbox)
        .expect_err("DedicatedSandbox without a broker lease should fail");
    assert_eq!(error.kind, RuntimeErrorKind::ResourceUnavailable);
    assert!(error.message.contains("broker lease"));
}
