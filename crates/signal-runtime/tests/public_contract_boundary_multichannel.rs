#[path = "support/public_contract_boundary_graph_multichannel.rs"]
mod public_contract_boundary_graph_multichannel_support;
#[path = "support/public_contract_boundary_plugin_records_core.rs"]
mod public_contract_boundary_plugin_records_core_support;

use public_contract_boundary_graph_multichannel_support::apply_public_multichannel_graph;
use public_contract_boundary_plugin_records_core_support::sample_discovered_type_record;
use signal_plugin::PluginFormat;
use signal_runtime::{
    HandshakeRequest, PluginScanRequest, RuntimeBusIntent, RuntimeCanonicalChannelLayout,
    RuntimeConfig, RuntimeConfigRequest, RuntimeEventRecorder, RuntimeLifecycleApi,
    RuntimeObservationReport, SignalRuntime,
};

#[test]
fn public_runtime_multichannel_boundary_reports_runtime_owned_layout_and_role_truth() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
    runtime
        .handshake(HandshakeRequest {
            client_version: "public-runtime-multichannel-boundary".into(),
            anticipative_preferred: true,
            max_sample_rate_hint: Some(96_000),
        })
        .expect("public runtime multichannel handshake should succeed");
    runtime
        .configure(RuntimeConfigRequest::new(48_000, 256))
        .expect("public runtime multichannel configure should succeed");
    apply_public_multichannel_graph(&mut runtime, "graph:public:multichannel");
    let scan_handle = runtime.record_plugin_scan_request(&PluginScanRequest {
        roots: vec!["~/Library/Audio/Plug-Ins".into()],
        formats: vec![PluginFormat::Clap],
    });
    runtime.record_plugin_scan_results(scan_handle, vec![sample_discovered_type_record()]);
    let recorder = RuntimeEventRecorder::default();

    let observation = RuntimeObservationReport::capture(&runtime, &recorder);
    let topology = &observation.execution_topology_summary;
    let track_node = topology
        .nodes
        .iter()
        .find(|node| node.node_id == "surround-track")
        .expect("surround-track node should be present");
    assert_eq!(
        track_node.input_layout.canonical_layout,
        Some(RuntimeCanonicalChannelLayout::Stereo)
    );
    assert_eq!(
        track_node.output_layout.canonical_layout,
        Some(RuntimeCanonicalChannelLayout::Surround5_1)
    );
    assert_eq!(track_node.input_bus_intent, RuntimeBusIntent::MainProgram);
    assert_eq!(track_node.output_bus_intent, RuntimeBusIntent::MainProgram);

    let send_node = topology
        .nodes
        .iter()
        .find(|node| node.node_id == "analysis-send")
        .expect("analysis-send node should be present");
    assert_eq!(
        send_node.input_layout.canonical_layout,
        Some(RuntimeCanonicalChannelLayout::Surround5_1)
    );
    assert_eq!(
        send_node.output_layout.canonical_layout,
        Some(RuntimeCanonicalChannelLayout::Quad)
    );
    assert_eq!(send_node.output_bus_intent, RuntimeBusIntent::AuxSend);

    let discovery = &observation.plugin_discovery_snapshot;
    assert_eq!(
        discovery.discovered_types[0]
            .default_multichannel_io
            .input_layout
            .canonical_layout,
        Some(RuntimeCanonicalChannelLayout::Stereo)
    );
    assert_eq!(
        discovery.discovered_types[0]
            .default_multichannel_io
            .output_layout
            .canonical_layout,
        Some(RuntimeCanonicalChannelLayout::Stereo)
    );
}
