use signal_plugin::{
    PluginFeature, PluginFormat, PluginIoLayout, PluginLifecycleContract, PluginProcessingContract,
    PluginStateContract,
};
use signal_runtime::{
    PluginSandboxLifecycleStage, PluginSandboxSpec, PluginScanRequest, RuntimeConfig,
    RuntimeEventRecorder, RuntimeObservationReport, RuntimePluginDiscoveredTypeRecord,
    RuntimeSupervisorReport, SignalRuntime,
};

fn sample_discovered_type_record() -> RuntimePluginDiscoveredTypeRecord {
    RuntimePluginDiscoveredTypeRecord {
        plugin_type_id: "plugin:clap:public-boundary".into(),
        plugin_id: "com.signal.public-boundary".into(),
        vendor: "Signal".into(),
        name: "Signal Public Boundary".into(),
        format: PluginFormat::Clap,
        version: Some("1.0.0".into()),
        features: vec![PluginFeature::AudioEffect, PluginFeature::Utility],
        default_io_layout: PluginIoLayout {
            audio_inputs: 2,
            audio_outputs: 2,
            midi_inputs: 1,
            midi_outputs: 1,
        },
        audio_bus_count: 2,
        parameter_count: 8,
        state_contract: PluginStateContract {
            supports_snapshot: true,
            supports_reset: true,
            supports_bypass: true,
            exposes_latency: true,
            exposes_tail: true,
        },
        processing_contract: PluginProcessingContract {
            max_block_frames: 4096,
            sample_accurate_automation: true,
            accepts_midi: true,
            accepts_note_events: true,
            produces_midi: true,
            silence_aware: true,
        },
        lifecycle_contract: PluginLifecycleContract {
            requires_main_thread_for_state: false,
            supports_prepare: true,
            supports_activate: true,
            supports_reset_while_active: true,
        },
        summary: "public boundary discovered plugin".into(),
    }
}

#[test]
fn public_runtime_contract_boundary_is_consumable_from_reexports() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
    let recorder = RuntimeEventRecorder::default();
    let scan_handle = runtime.record_plugin_scan_request(&PluginScanRequest {
        roots: vec!["~/Library/Audio/Plug-Ins/CLAP".into()],
        formats: vec![PluginFormat::Clap],
    });
    runtime.record_plugin_scan_results(scan_handle, vec![sample_discovered_type_record()]);
    runtime.record_plugin_sandbox_spec(&PluginSandboxSpec {
        sandbox_id: "public-boundary-sandbox".into(),
        plugin_format: PluginFormat::Clap,
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
    assert_eq!(observation.plugin_discovery_snapshot.scan_count, 1);
    assert_eq!(
        observation.plugin_discovery_snapshot.discovered_type_count,
        1
    );
    assert_eq!(
        observation.plugin_discovery_snapshot.discovered_types[0].plugin_type_id,
        "plugin:clap:public-boundary"
    );
    assert_eq!(
        observation.plugin_discovery_snapshot.discovered_types[0].features,
        vec![PluginFeature::AudioEffect, PluginFeature::Utility]
    );
    assert_eq!(
        observation.plugin_lifecycle_snapshot.sandboxes[0].plugin_format,
        Some(PluginFormat::Clap)
    );

    let observation_json = observation.render_json();
    assert!(observation_json.contains("\"sample_rate\":48000"));
    assert!(observation_json.contains("\"block_size\":512"));
    assert!(observation_json.contains("\"engine_block_snapshot\":{"));
    assert!(observation_json.contains("\"execution_topology_summary\":{"));
    assert!(observation_json.contains("\"plugin_discovery_snapshot\":{"));
    assert!(observation_json.contains("\"plugin_type_id\":\"plugin:clap:public-boundary\""));
    assert!(observation_json.contains("\"supports_snapshot\":true"));

    let supervisor_json = supervisor.render_json();
    assert!(supervisor_json.contains("\"sample_rate\":48000"));
    assert!(supervisor_json.contains("\"block_size\":512"));
    assert!(supervisor_json.contains("\"event_stream\":0"));
    assert!(supervisor_json.contains("\"plugin_discovery_snapshot\":{"));
    assert!(supervisor_json.contains("\"discovered_type_count\":1"));

    let profiling_json = profiling.render_json();
    assert!(profiling_json.contains("\"sample_rate_hz\":48000"));
    assert!(profiling_json.contains("\"block_size\":512"));
    assert!(profiling_json.contains("\"summary\":"));

    let soak_json = soak.render_json();
    assert!(soak_json.contains("\"event_stream_count\":0"));
    assert!(soak_json.contains("\"summary\":"));

    assert!(profiling
        .render_multiline()
        .contains("sample_rate_hz=48000"));
    assert!(soak.render_multiline().contains("event_stream_count=0"));
    assert!(supervisor.render_multiline().contains("event_stream=0"));
}
