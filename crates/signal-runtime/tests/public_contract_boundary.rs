#[path = "support/public_contract_boundary_graph_bus.rs"]
mod public_contract_boundary_graph_bus_support;
#[path = "support/public_contract_boundary_graph_foundation.rs"]
mod public_contract_boundary_graph_foundation_support;
#[path = "support/public_contract_boundary_graph_multichannel.rs"]
mod public_contract_boundary_graph_multichannel_support;
#[path = "support/public_contract_boundary_graph_plugin_surface.rs"]
mod public_contract_boundary_graph_plugin_surface_support;
#[path = "support/public_contract_boundary_plugin_records_complex.rs"]
mod public_contract_boundary_plugin_records_complex_support;
#[path = "support/public_contract_boundary_plugin_records_core.rs"]
mod public_contract_boundary_plugin_records_core_support;

use public_contract_boundary_plugin_records_core_support::{
    sample_backend_breadth_record, sample_discovered_type_record,
};
use signal_plugin::{PluginFeature, PluginFormat};
use signal_runtime::{
    PluginSandboxLifecycleStage, PluginSandboxSpec, PluginScanRequest, RuntimeConfig,
    RuntimeEventRecorder, RuntimeInterruptionClass, RuntimeObservationReport, RuntimeRecoveryState,
    RuntimeSupervisorReport, SignalRuntime,
};

#[test]
fn public_runtime_contract_boundary_is_consumable_from_reexports() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
    let recorder = RuntimeEventRecorder::default();
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
        sandbox_id: "public-boundary-sandbox".into(),
        plugin_format: PluginFormat::Clap,
        plugin_type_id: None,
    });
    runtime.record_plugin_sandbox_lifecycle(
        "public-boundary-sandbox",
        PluginSandboxLifecycleStage::SandboxEnsured,
        None,
    );

    let observation = RuntimeObservationReport::capture(&runtime, &recorder);
    let supervisor = RuntimeSupervisorReport::capture(&runtime, &recorder);
    let profiling = supervisor.profiling_receipt();
    let soak = supervisor.soak_receipt();

    assert_eq!(profiling.sample_rate_hz, 48_000);
    assert_eq!(profiling.block_size, 512);
    assert_eq!(soak.event_stream_count, 0);
    assert_eq!(
        observation.fault_status.recovery_state,
        RuntimeRecoveryState::Steady
    );
    assert_eq!(
        observation.interruption_summary.class,
        RuntimeInterruptionClass::Steady
    );
    assert!(!observation.interruption_summary.active);
    assert_eq!(observation.fault_diagnostic_receipt.primary_family, None);
    assert_eq!(observation.fault_diagnostic_receipt.contributions.len(), 4);
    assert_eq!(
        observation.recording_capture_snapshot.state,
        Some(signal_runtime::RuntimeRecordingCaptureState::Idle)
    );
    assert_eq!(observation.plugin_discovery_snapshot.scan_count, 1);
    assert_eq!(
        observation.plugin_discovery_snapshot.discovered_type_count,
        2
    );
    assert_eq!(
        observation
            .plugin_discovery_snapshot
            .discovered_format_count,
        2
    );
    assert_eq!(
        observation.plugin_discovery_snapshot.discovered_types[0].plugin_type_id,
        "plugin:clap:public-boundary"
    );
    assert_eq!(
        observation.plugin_discovery_snapshot.discovered_types[0].features,
        vec![PluginFeature::AudioEffect, PluginFeature::Utility]
    );
    assert!(
        observation
            .plugin_discovery_snapshot
            .capability_coverage
            .multi_format_catalog
    );
    assert_eq!(
        observation
            .plugin_discovery_snapshot
            .capability_coverage
            .requires_main_thread_for_state_count,
        1
    );
    assert_eq!(
        observation.plugin_discovery_snapshot.format_coverage[1].format,
        PluginFormat::Vst3
    );
    assert_eq!(
        observation.plugin_lifecycle_snapshot.sandboxes[0].plugin_format,
        Some(PluginFormat::Clap)
    );

}

#[test]
fn public_runtime_plugin_discovery_coverage_is_consumable_from_reexports() {
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

    let observation = RuntimeObservationReport::capture(&runtime, &RuntimeEventRecorder::default());
    let coverage = &observation.plugin_discovery_snapshot.capability_coverage;
    let format_coverage = &observation.plugin_discovery_snapshot.format_coverage;

    assert_eq!(
        observation
            .plugin_discovery_snapshot
            .discovered_format_count,
        2
    );
    assert!(coverage.multi_format_catalog);
    assert_eq!(coverage.audio_effect_count, 1);
    assert_eq!(coverage.instrument_count, 1);
    assert_eq!(coverage.requires_main_thread_for_state_count, 1);
    assert_eq!(coverage.max_parameter_count, 12);
    assert_eq!(format_coverage.len(), 2);
    assert_eq!(format_coverage[0].format, PluginFormat::Clap);
    assert_eq!(format_coverage[0].supports_activate_count, 1);
    assert_eq!(format_coverage[1].format, PluginFormat::Vst3);
    assert_eq!(format_coverage[1].instrument_count, 1);

}
