use signal_plugin::PluginFormat;
use signal_runtime::{
    PluginSandboxInstanceStateRecord, PluginSandboxLifecycleStage, PluginSandboxSpec,
    PluginSandboxTransportStage, PluginScanRequest, RuntimeConfig, RuntimeEventRecorder,
    RuntimeObservationReport, RuntimeSupervisorReport, SignalRuntime,
};

#[path = "support/public_contract_boundary_plugin_records_core.rs"]
mod public_contract_boundary_plugin_records_core_support;

use public_contract_boundary_plugin_records_core_support::sample_backend_breadth_record;

#[test]
fn public_runtime_vst3_boundary_reports_runtime_owned_discovery_and_lifecycle_truth() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
    let recorder = RuntimeEventRecorder::default();
    let scan_handle = runtime.record_plugin_scan_request(&PluginScanRequest {
        roots: vec!["~/.vst3".into(), "/usr/lib/vst3".into()],
        formats: vec![PluginFormat::Vst3],
    });
    runtime.record_plugin_scan_results(scan_handle, vec![sample_backend_breadth_record()]);
    runtime.record_plugin_sandbox_spec(&PluginSandboxSpec {
        sandbox_id: "public-vst3-sandbox".into(),
        plugin_format: PluginFormat::Vst3,
        plugin_type_id: Some("plugin:vst3:public-instrument".into()),
    });
    runtime.record_plugin_sandbox_lifecycle(
        "public-vst3-sandbox",
        PluginSandboxLifecycleStage::PluginTypeLoaded,
        Some(1),
    );
    runtime.record_plugin_sandbox_lifecycle(
        "public-vst3-sandbox",
        PluginSandboxLifecycleStage::InstancePrepared,
        Some(1),
    );
    runtime.record_plugin_sandbox_instance_state(PluginSandboxInstanceStateRecord {
        sandbox_id: "public-vst3-sandbox".into(),
        plugin_type_id: "plugin:vst3:public-instrument".into(),
        instance_id: "instance:public:vst3".into(),
        lifecycle_state: "Prepared".into(),
        readiness_state: "Ready".into(),
        degraded_reasons: Vec::new(),
        active: true,
        processing_epoch: Some(1),
        processing_sample_rate_hz: Some(48_000),
        processing_max_block_frames: Some(512),
        audio_inputs: Some(0),
        audio_outputs: Some(2),
        midi_inputs: Some(1),
        midi_outputs: Some(0),
        last_fault: None,
    });
    runtime.record_plugin_sandbox_transport(
        "public-vst3-sandbox",
        "lease-public-vst3",
        "region-public-vst3",
        PluginSandboxTransportStage::Attached,
        Some(1),
        Some("public vst3 transport attached".into()),
    );

    let observation = RuntimeObservationReport::capture(&runtime, &recorder);
    let supervisor = RuntimeSupervisorReport::capture(&runtime, &recorder);

    assert_eq!(
        observation.plugin_discovery_snapshot.discovered_type_count,
        1
    );
    assert_eq!(
        observation
            .plugin_discovery_snapshot
            .last_scan
            .as_ref()
            .map(|scan| scan.formats.clone()),
        Some(vec![PluginFormat::Vst3])
    );
    assert_eq!(
        observation.plugin_discovery_snapshot.discovered_types[0].plugin_type_id,
        "plugin:vst3:public-instrument"
    );
    assert_eq!(
        observation.plugin_discovery_snapshot.discovered_types[0].format,
        PluginFormat::Vst3
    );
    let sandbox = observation
        .plugin_lifecycle_snapshot
        .sandboxes
        .iter()
        .find(|sandbox| sandbox.sandbox_id == "public-vst3-sandbox")
        .expect("public vst3 sandbox should be visible");
    assert_eq!(sandbox.plugin_format, Some(PluginFormat::Vst3));
    assert_eq!(
        sandbox.plugin_type_id.as_deref(),
        Some("plugin:vst3:public-instrument")
    );
    assert_eq!(
        sandbox.lifecycle_stage,
        Some(PluginSandboxLifecycleStage::InstancePrepared)
    );
    assert_eq!(
        sandbox.transport_stage,
        Some(PluginSandboxTransportStage::Attached)
    );
    assert_eq!(sandbox.readiness_state.as_deref(), Some("Ready"));
    assert!(sandbox.active_transport);

    let observation_json = observation.render_json();
    assert!(observation_json.contains("\"formats\":[\"Vst3\"]"));
    assert!(observation_json.contains("\"plugin_type_id\":\"plugin:vst3:public-instrument\""));
    assert!(observation_json.contains("\"transport_stage\":\"Attached\""));

    let supervisor_json = supervisor.render_json();
    assert!(supervisor_json.contains("\"plugin_discovery_snapshot\":{"));
    assert!(supervisor_json.contains("\"plugin_lifecycle_snapshot\":{"));
    assert!(supervisor_json.contains("\"plugin_type_id\":\"plugin:vst3:public-instrument\""));
}
