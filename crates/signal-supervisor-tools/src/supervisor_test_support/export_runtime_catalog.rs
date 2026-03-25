use crate::{render_supervisor_export_json, HostProfile, Scenario};
use signal_plugin::PluginFormat;
use signal_runtime::{
    PluginSandboxLifecycleStage, PluginSandboxSpec, PluginScanRequest, RuntimeConfig,
    RuntimeSupervisorReport, SignalRuntime,
};

use super::{sample_backend_breadth_record, sample_discovered_type_record};

pub(crate) fn verify_export_json_carries_runtime_owned_plugin_discovery_catalog() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
    let scan_handle = runtime.record_plugin_scan_request(&PluginScanRequest {
        roots: vec!["~/Library/Audio/Plug-Ins/CLAP".into()],
        formats: vec![PluginFormat::Clap],
    });
    runtime.record_plugin_scan_results(
        scan_handle,
        vec![
            sample_discovered_type_record(),
            sample_backend_breadth_record(),
        ],
    );
    runtime.record_plugin_sandbox_spec(&PluginSandboxSpec {
        sandbox_id: "export-consumer-sandbox".into(),
        plugin_format: PluginFormat::Clap,
        plugin_type_id: None,
    });
    runtime.record_plugin_sandbox_lifecycle(
        "export-consumer-sandbox",
        PluginSandboxLifecycleStage::SandboxEnsured,
        None,
    );

    let report = RuntimeSupervisorReport::capture(&runtime, &Default::default());
    let export = render_supervisor_export_json(
        HostProfile::Local,
        Scenario::Default,
        "{}".into(),
        &report.profiling_receipt(),
        &report.soak_receipt(),
        &report,
    );

    assert!(export.contains("\"host_summary\":{}"));
    assert!(export.contains("\"supervisor_report\":{"));
    assert!(export.contains("\"plugin_discovery_snapshot\":{"));
    assert!(export.contains("\"discovered_type_count\":2"));
    assert!(export.contains("\"discovered_format_count\":2"));
    assert!(export.contains("\"plugin_type_id\":\"plugin:clap:export-consumer\""));
    assert!(export.contains("\"plugin_type_id\":\"plugin:vst3:export-instrument\""));
    assert!(export.contains("\"format\":\"Clap\""));
    assert!(export.contains("\"multi_format_catalog\":true"));
    assert!(export.contains("\"requires_main_thread_for_state_count\":1"));
    assert!(export.contains("\"format_coverage\":["));
    assert!(export.contains("\"supports_snapshot\":true"));
    assert!(export.contains("\"supports_activate\":true"));
}

pub(crate) fn verify_export_json_carries_runtime_owned_plugin_discovery_capability_coverage() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
    let scan_handle = runtime.record_plugin_scan_request(&PluginScanRequest {
        roots: vec!["~/Library/Audio/Plug-Ins".into()],
        formats: vec![PluginFormat::Clap, PluginFormat::Vst3],
    });
    runtime.record_plugin_scan_results(
        scan_handle,
        vec![
            sample_discovered_type_record(),
            sample_backend_breadth_record(),
        ],
    );

    let report = RuntimeSupervisorReport::capture(&runtime, &Default::default());
    let export = render_supervisor_export_json(
        HostProfile::Local,
        Scenario::Default,
        "{}".into(),
        &report.profiling_receipt(),
        &report.soak_receipt(),
        &report,
    );

    assert!(export.contains("\"discovered_format_count\":2"));
    assert!(export.contains("\"multi_format_catalog\":true"));
    assert!(export.contains("\"requires_main_thread_for_state_count\":1"));
    assert!(export.contains("\"max_parameter_count\":24"));
    assert!(export.contains("\"format\":\"Vst3\""));
    assert!(export.contains("\"instrument_count\":1"));
}
