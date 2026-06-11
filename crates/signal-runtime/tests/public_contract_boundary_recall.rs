#[path = "support/public_contract_boundary_graph_foundation.rs"]
mod public_contract_boundary_graph_foundation_support;
#[path = "support/public_contract_boundary_graph_plugin_surface.rs"]
mod public_contract_boundary_graph_plugin_surface_support;
#[path = "support/public_contract_boundary_plugin_context.rs"]
mod public_contract_boundary_plugin_context_support;
#[path = "support/public_contract_boundary_plugin_records_core.rs"]
mod public_contract_boundary_plugin_records_core_support;

use public_contract_boundary_graph_foundation_support::apply_public_plugin_continuity_graph;
use public_contract_boundary_graph_plugin_surface_support::record_public_plugin_sandbox_ready;
use public_contract_boundary_plugin_context_support::{
    sample_public_ara_context, sample_public_preset_descriptor,
};
use public_contract_boundary_plugin_records_core_support::{
    sample_backend_breadth_record, sample_discovered_type_record,
};
use signal_plugin::PluginFormat;
use signal_runtime::{
    HandshakeRequest, PluginScanRequest, RuntimeConfig, RuntimeConfigRequest, RuntimeEventRecorder,
    RuntimeLifecycleApi, RuntimeObservationApi, RuntimeObservationReport,
    RuntimeOfflineRenderContractPreview, RuntimeOfflineRenderRequest,
    RuntimePluginRecallPortabilityClass, RuntimeSupervisorReport, SignalRuntime,
};

#[test]
fn public_runtime_recall_interchange_and_ara_context_truth_is_consumable_from_reexports() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
    let recorder = RuntimeEventRecorder::default();
    runtime
        .handshake(HandshakeRequest {
            client_version: "public-runtime-recall-portability".into(),
            anticipative_preferred: true,
            max_sample_rate_hint: Some(96_000),
        })
        .expect("public runtime recall portability handshake should succeed");
    runtime
        .configure(RuntimeConfigRequest::new(48_000, 512))
        .expect("public runtime recall portability configure should succeed");
    let scan_handle = runtime.record_plugin_scan_request(&PluginScanRequest {
        roots: vec!["~/.clap".into(), "~/.vst3".into()],
        formats: vec![PluginFormat::Clap, PluginFormat::Vst3],
    });
    runtime.record_plugin_scan_results(
        scan_handle,
        vec![
            sample_discovered_type_record(),
            sample_backend_breadth_record(),
        ],
    );
    apply_public_plugin_continuity_graph(
        &mut runtime,
        "graph:public:recall-portability",
        &[("node-clap", "sandbox-clap"), ("node-vst3", "sandbox-vst3")],
    );
    record_public_plugin_sandbox_ready(
        &mut runtime,
        "sandbox-clap",
        PluginFormat::Clap,
        "plugin:clap:public-boundary",
        31,
    );
    record_public_plugin_sandbox_ready(
        &mut runtime,
        "sandbox-vst3",
        PluginFormat::Vst3,
        "plugin:vst3:public-instrument",
        32,
    );
    runtime.record_plugin_preset_descriptor("sandbox-clap", sample_public_preset_descriptor());
    runtime.record_plugin_ara_context(
        "sandbox-clap",
        sample_public_ara_context(
            RuntimePluginRecallPortabilityClass::ContextOnly,
            "doc:public-runtime",
            "source:lead-vocal",
            "region:verse-a",
            1_024,
            4_096,
        ),
    );
    runtime.record_plugin_ara_context(
        "sandbox-vst3",
        sample_public_ara_context(
            RuntimePluginRecallPortabilityClass::ContextOnly,
            "doc:public-runtime",
            "source:synth-bus",
            "region:hook-b",
            8_192,
            2_048,
        ),
    );

    let observation = RuntimeObservationReport::capture(&runtime, &recorder);
    let _supervisor = RuntimeSupervisorReport::capture(&runtime, &recorder);
    let clap_stage = observation
        .plugin_chain_snapshot
        .chains
        .iter()
        .flat_map(|chain| chain.stages.iter())
        .find(|stage| stage.node_id == "node-clap")
        .expect("public runtime recall boundary should export clap stage");
    assert_eq!(
        clap_stage.recall.payload.interchange.portability_class,
        RuntimePluginRecallPortabilityClass::Portable
    );
    assert!(
        clap_stage
            .recall
            .payload
            .interchange
            .shared_payload_available
    );
    assert!(
        !clap_stage
            .recall
            .payload
            .interchange
            .native_supplement_required
    );
    assert_eq!(
        clap_stage
            .recall
            .payload
            .interchange
            .preset_descriptor
            .as_ref()
            .and_then(|descriptor| descriptor.label.as_deref()),
        Some("Init")
    );
    assert_eq!(
        clap_stage
            .recall
            .payload
            .ara_context
            .as_ref()
            .and_then(|context| context.document_context.as_ref())
            .map(|context| context.document_id.as_str()),
        Some("doc:public-runtime")
    );
    let vst3_stage = observation
        .plugin_chain_snapshot
        .chains
        .iter()
        .flat_map(|chain| chain.stages.iter())
        .find(|stage| stage.node_id == "node-vst3")
        .expect("public runtime recall boundary should export vst3 stage");
    assert_eq!(
        vst3_stage.recall.payload.interchange.portability_class,
        RuntimePluginRecallPortabilityClass::ContextOnly
    );
    assert!(
        !vst3_stage
            .recall
            .payload
            .interchange
            .shared_payload_available
    );
    assert_eq!(
        observation
            .execution_topology_summary
            .nodes
            .iter()
            .find(|node| node.node_id == "node-clap")
            .and_then(|node| node.plugin_recall.as_ref())
            .map(|recall| recall.payload.interchange.portability_class),
        Some(RuntimePluginRecallPortabilityClass::Portable)
    );

    let handoff = runtime.get_plugin_recall_handoff_snapshot();
    let preview = RuntimeOfflineRenderContractPreview::from_runtime_state(
        &RuntimeOfflineRenderRequest {
            request_id: "render:public:recall-portability".into(),
            timeline_start_samples: 0,
            duration_samples: 4_096,
            export_sample_rate_hz: 48_000,
            include_main_mix: true,
            artifact_root_path: None,
            stem_targets: Vec::new(),
            freeze_artifacts: Vec::new(),
        },
        &runtime.get_execution_topology_summary(),
        &runtime.get_clip_processing_pipeline_snapshot(),
        &runtime.get_media_pipeline_snapshot(),
        &runtime.get_tempo_map_snapshot(),
        &runtime.get_marker_analysis_snapshot(),
        &handoff,
    )
    .expect("public runtime recall preview should build");
    assert_eq!(preview.chain_contract.recall_stage_count, 2);
}
