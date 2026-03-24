#[path = "support/public_contract_boundary_graph_multichannel.rs"]
mod public_contract_boundary_graph_multichannel_support;

use public_contract_boundary_graph_multichannel_support::apply_public_sidechain_graph;
use signal_graph::synthetic_stereo_block;
use signal_plugin::PluginFormat;
use signal_primitives::{FrameCount, SampleRate};
use signal_runtime::{
    HandshakeRequest, PluginSandboxLifecycleStage, PluginSandboxSpec, PluginSandboxTransportStage,
    RuntimeConfig, RuntimeConfigRequest, RuntimeEventRecorder, RuntimeInterruptionClass,
    RuntimeLifecycleApi, RuntimeObservationReport, RuntimeSecondaryInputAttachmentPolicy,
    RuntimeSecondaryInputFallbackOutcome, RuntimeSecondaryInputTargetKind, SignalRuntime,
};

#[test]
fn public_runtime_sidechain_boundary_reports_runtime_owned_secondary_input_truth() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
    runtime
        .handshake(HandshakeRequest {
            client_version: "public-runtime-sidechain-boundary".into(),
            anticipative_preferred: true,
            max_sample_rate_hint: Some(96_000),
        })
        .expect("public runtime sidechain handshake should succeed");
    runtime
        .configure(RuntimeConfigRequest::new(48_000, 256))
        .expect("public runtime sidechain configure should succeed");
    apply_public_sidechain_graph(&mut runtime, "graph:public:sidechain");
    runtime.record_plugin_sandbox_spec(&PluginSandboxSpec {
        sandbox_id: "sandbox:public:sidechain".into(),
        plugin_format: PluginFormat::Clap,
        plugin_type_id: Some("plugin:clap:public-boundary".into()),
    });
    runtime.record_plugin_sandbox_lifecycle(
        "sandbox:public:sidechain",
        PluginSandboxLifecycleStage::InstancePrepared,
        Some(1),
    );
    runtime.record_plugin_sandbox_transport(
        "sandbox:public:sidechain",
        "lease-public-sidechain",
        "region-public-sidechain",
        PluginSandboxTransportStage::Attached,
        Some(1),
        Some("public sidechain transport attached".into()),
    );
    runtime
        .process_engine_block(
            2,
            3,
            synthetic_stereo_block(SampleRate(48_000), FrameCount(256), 2),
        )
        .expect("public runtime sidechain block should succeed");

    let recorder = RuntimeEventRecorder::default();
    let observation = RuntimeObservationReport::capture(&runtime, &recorder);
    let topology = &observation.execution_topology_summary;
    assert_eq!(topology.secondary_input_count, 1);
    assert_eq!(topology.required_secondary_input_count, 1);
    let route = &topology.secondary_inputs[0];
    assert_eq!(route.source_id, "kick-sidechain");
    assert_eq!(route.target_id, "compressor");
    assert_eq!(
        route.target_kind,
        RuntimeSecondaryInputTargetKind::NodeInput
    );
    assert_eq!(route.target_bus_id, "plugin:compressor:sidechain");
    assert_eq!(
        route.attachment_policy,
        RuntimeSecondaryInputAttachmentPolicy::Required
    );
    assert_eq!(
        route.fallback_outcome,
        RuntimeSecondaryInputFallbackOutcome::SafeModeDegradation
    );

    let compressor = topology
        .nodes
        .iter()
        .find(|node| node.node_id == "compressor")
        .expect("compressor node should be present");
    let node_secondary_input = compressor
        .secondary_input
        .as_ref()
        .expect("compressor should carry sidechain receipt");
    assert_eq!(node_secondary_input.source_id, "kick-sidechain");
    assert_eq!(
        node_secondary_input.target_kind,
        RuntimeSecondaryInputTargetKind::NodeInput
    );

    let stage = observation
        .plugin_chain_snapshot
        .chains
        .iter()
        .find(|chain| chain.stage_count == 1)
        .and_then(|chain| chain.stages.first())
        .expect("plugin chain stage should be present");
    let stage_secondary_input = stage
        .secondary_input
        .as_ref()
        .expect("plugin chain stage should carry sidechain receipt");
    assert_eq!(
        stage_secondary_input.target_kind,
        RuntimeSecondaryInputTargetKind::PluginInput
    );
    assert_eq!(stage_secondary_input.target_id, "compressor");

    let rendered = observation.render_json();
    assert!(rendered.contains("\"secondary_input_count\":1"));
    assert!(rendered.contains("\"target_kind\":\"NodeInput\""));
    assert!(rendered.contains("\"target_kind\":\"PluginInput\""));
    assert!(rendered.contains("\"fallback_outcome\":\"SafeModeDegradation\""));
}
