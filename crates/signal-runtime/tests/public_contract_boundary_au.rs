use signal_plugin::PluginFormat;
use signal_runtime::{
    PluginSandboxInstanceStateRecord, PluginSandboxLifecycleStage, PluginSandboxSpec,
    PluginSandboxTransportStage, PluginScanRequest, RuntimeConfig, RuntimeEventRecorder,
    RuntimeObservationReport, RuntimeSupervisorReport, SignalRuntime,
};

#[path = "support/public_contract_boundary_plugin_records_core.rs"]
mod public_contract_boundary_plugin_records_core_support;

use public_contract_boundary_plugin_records_core_support::sample_au_breadth_record;

#[test]
fn public_runtime_au_boundary_reports_runtime_owned_discovery_and_lifecycle_truth() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
    let recorder = RuntimeEventRecorder::default();
    let scan_handle = runtime.record_plugin_scan_request(&PluginScanRequest {
        roots: vec!["~/Library/Audio/Plug-Ins/Components".into()],
        formats: vec![PluginFormat::Au],
    });
    runtime.record_plugin_scan_results(scan_handle, vec![sample_au_breadth_record()]);
    runtime.record_plugin_sandbox_spec(&PluginSandboxSpec {
        sandbox_id: "public-au-sandbox".into(),
        plugin_format: PluginFormat::Au,
        plugin_type_id: Some("plugin:au:public-instrument".into()),
    });
    runtime.record_plugin_sandbox_lifecycle(
        "public-au-sandbox",
        PluginSandboxLifecycleStage::PluginTypeLoaded,
        Some(1),
    );
    runtime.record_plugin_sandbox_lifecycle(
        "public-au-sandbox",
        PluginSandboxLifecycleStage::InstancePrepared,
        Some(1),
    );
    runtime.record_plugin_sandbox_instance_state(PluginSandboxInstanceStateRecord {
        sandbox_id: "public-au-sandbox".into(),
        plugin_type_id: "plugin:au:public-instrument".into(),
        instance_id: "instance:public:au".into(),
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
        "public-au-sandbox",
        "lease-public-au",
        "region-public-au",
        PluginSandboxTransportStage::Attached,
        Some(1),
        Some("public au transport attached".into()),
    );

    let observation = RuntimeObservationReport::capture(&runtime, &recorder);
    let _supervisor = RuntimeSupervisorReport::capture(&runtime, &recorder);

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
        Some(vec![PluginFormat::Au])
    );
    assert_eq!(
        observation.plugin_discovery_snapshot.discovered_types[0].plugin_type_id,
        "plugin:au:public-instrument"
    );
    assert_eq!(
        observation.plugin_discovery_snapshot.discovered_types[0].format,
        PluginFormat::Au
    );
    let sandbox = observation
        .plugin_lifecycle_snapshot
        .sandboxes
        .iter()
        .find(|sandbox| sandbox.sandbox_id == "public-au-sandbox")
        .expect("public au sandbox should be visible");
    assert_eq!(sandbox.plugin_format, Some(PluginFormat::Au));
    assert_eq!(
        sandbox.plugin_type_id.as_deref(),
        Some("plugin:au:public-instrument")
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

}
