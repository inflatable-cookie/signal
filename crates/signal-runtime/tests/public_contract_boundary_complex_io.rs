#[path = "support/public_contract_boundary_graph_plugin_surface.rs"]
mod public_contract_boundary_graph_plugin_surface_support;
#[path = "support/public_contract_boundary_plugin_records_complex.rs"]
mod public_contract_boundary_plugin_records_complex_support;

use public_contract_boundary_graph_plugin_surface_support::apply_public_complex_io_graph;
use public_contract_boundary_plugin_records_complex_support::{
    sample_complex_bus_fx_record, sample_complex_multi_output_record,
};
use signal_graph::synthetic_stereo_block;
use signal_plugin::PluginFormat;
use signal_primitives::{AudioBuffer, ChannelLayout, FrameCount, SampleRate};
use signal_runtime::{
    HandshakeRequest, PluginScanRequest, RuntimeConfig, RuntimeConfigRequest,
    RuntimeDynamicBusNegotiationPosture, RuntimeEventRecorder, RuntimeLifecycleApi,
    RuntimeObservationReport, RuntimePluginBusCapableFxClass,
    RuntimePluginNegotiationFallbackOutcome, RuntimePluginPinGroupIdentity,
    RuntimePluginPinMatrixPosture, SignalRuntime,
};

#[test]
fn public_runtime_complex_io_boundary_reports_runtime_owned_topology_truth() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
    runtime
        .handshake(HandshakeRequest {
            client_version: "public-runtime-complex-io-boundary".into(),
            anticipative_preferred: true,
            max_sample_rate_hint: Some(96_000),
        })
        .expect("public runtime complex io handshake should succeed");
    runtime
        .configure(RuntimeConfigRequest::new(48_000, 256))
        .expect("public runtime complex io configure should succeed");
    apply_public_complex_io_graph(&mut runtime, "graph:public:complex-io");
    let scan_handle = runtime.record_plugin_scan_request(&PluginScanRequest {
        roots: vec!["~/Library/Audio/Plug-Ins/VST3".into()],
        formats: vec![PluginFormat::Vst3],
    });
    runtime.record_plugin_scan_results(
        scan_handle,
        vec![
            sample_complex_multi_output_record(),
            sample_complex_bus_fx_record(),
        ],
    );
    runtime
        .apply_plugin_node_render_batch(signal_runtime::PluginNodeRenderBatch {
            graph_id: "graph:public:complex-io".into(),
            processing_epoch: 1,
            block_sequence: 1,
            renders: vec![
                signal_runtime::PluginNodeRender {
                    node_id: "plugin-multiout".into(),
                    sandbox_id: "sandbox:public:multiout".into(),
                    output: AudioBuffer::new(
                        SampleRate(48_000),
                        ChannelLayout::Stereo,
                        FrameCount(8),
                    ),
                    latency_samples: 32,
                    tail_samples: 48,
                    bypassed: false,
                },
                signal_runtime::PluginNodeRender {
                    node_id: "plugin-bus-fx".into(),
                    sandbox_id: "sandbox:public:bus-fx".into(),
                    output: AudioBuffer::new(
                        SampleRate(48_000),
                        ChannelLayout::Stereo,
                        FrameCount(8),
                    ),
                    latency_samples: 16,
                    tail_samples: 24,
                    bypassed: false,
                },
            ],
        })
        .expect("public complex io render batch should apply");
    runtime
        .process_engine_block(
            4,
            6,
            synthetic_stereo_block(SampleRate(48_000), FrameCount(256), 3),
        )
        .expect("public runtime complex io block should succeed");

    let recorder = RuntimeEventRecorder::default();
    let observation = RuntimeObservationReport::capture(&runtime, &recorder);
    let discovery = &observation.plugin_discovery_snapshot;
    assert_eq!(discovery.discovered_type_count, 2);
    assert_eq!(discovery.capability_coverage.complex_io_type_count, 2);
    assert_eq!(
        discovery.capability_coverage.multi_output_instrument_count,
        1
    );
    assert_eq!(discovery.capability_coverage.bus_capable_fx_count, 1);
    assert_eq!(
        discovery
            .capability_coverage
            .max_complex_io_port_group_count,
        4
    );
    assert!(discovery.discovered_types.iter().any(|record| {
        record.plugin_type_id == "plugin:vst3:public-multiout"
            && record.complex_io_summary.multi_output_instrument
            && record.complex_io_summary.instrument_output_group_count == 2
    }));
    assert!(discovery.discovered_types.iter().any(|record| {
        record.plugin_type_id == "plugin:vst3:public-bus-fx"
            && record.complex_io_summary.bus_capable_fx_class
                == Some(RuntimePluginBusCapableFxClass::SendReturnCapableFx)
            && record.complex_io_summary.secondary_input_group_count == 1
    }));
    let pin_matrix = &observation.plugin_pin_matrix_snapshot;
    assert_eq!(pin_matrix.plugin_type_count, 2);
    assert_eq!(pin_matrix.negotiated_type_count, 2);
    assert_eq!(pin_matrix.dynamic_negotiated_type_count, 2);
    let multiout_pin_matrix = pin_matrix
        .records
        .iter()
        .find(|record| record.plugin_type_id == "plugin:vst3:public-multiout")
        .expect("public multi-output pin matrix record should be visible");
    assert_eq!(
        multiout_pin_matrix.pin_matrix_posture,
        RuntimePluginPinMatrixPosture::Negotiated
    );
    assert_eq!(
        multiout_pin_matrix.dynamic_bus_negotiation_posture,
        RuntimeDynamicBusNegotiationPosture::Negotiated
    );
    assert_eq!(
        multiout_pin_matrix.fallback_outcome,
        RuntimePluginNegotiationFallbackOutcome::RoutePrimaryOnly
    );
    assert!(multiout_pin_matrix
        .pin_group_identities
        .contains(&RuntimePluginPinGroupIdentity::PrimaryProgramPath));
    assert!(multiout_pin_matrix
        .pin_group_identities
        .contains(&RuntimePluginPinGroupIdentity::SecondaryProgramPath));
    let bus_fx_pin_matrix = pin_matrix
        .records
        .iter()
        .find(|record| record.plugin_type_id == "plugin:vst3:public-bus-fx")
        .expect("public bus-fx pin matrix record should be visible");
    assert_eq!(
        bus_fx_pin_matrix.pin_matrix_posture,
        RuntimePluginPinMatrixPosture::Negotiated
    );
    assert_eq!(
        bus_fx_pin_matrix.dynamic_bus_negotiation_posture,
        RuntimeDynamicBusNegotiationPosture::Negotiated
    );
    assert_eq!(
        bus_fx_pin_matrix.fallback_outcome,
        RuntimePluginNegotiationFallbackOutcome::GuardedDegradation
    );
    assert!(bus_fx_pin_matrix
        .pin_group_identities
        .contains(&RuntimePluginPinGroupIdentity::SidechainPath));
    assert!(bus_fx_pin_matrix
        .pin_group_identities
        .contains(&RuntimePluginPinGroupIdentity::AuxReturnPath));

    let plugin_chain = &observation.plugin_chain_snapshot;
    assert_eq!(plugin_chain.chain_count, 1);
    assert_eq!(plugin_chain.stage_count, 2);
    let multiout_stage = plugin_chain
        .chains
        .iter()
        .flat_map(|chain| chain.stages.iter())
        .find(|stage| stage.node_id == "plugin-multiout")
        .expect("multi-output stage should be present");
    assert!(multiout_stage.complex_io_summary.has_complex_topology);
    assert!(multiout_stage.complex_io_summary.multi_output_instrument);
    assert_eq!(
        multiout_stage
            .complex_io_summary
            .instrument_output_group_count,
        2
    );
    let bus_fx_stage = plugin_chain
        .chains
        .iter()
        .flat_map(|chain| chain.stages.iter())
        .find(|stage| stage.node_id == "plugin-bus-fx")
        .expect("bus fx stage should be present");
    assert_eq!(
        bus_fx_stage.complex_io_summary.bus_capable_fx_class,
        Some(RuntimePluginBusCapableFxClass::SendReturnCapableFx)
    );
    assert_eq!(
        bus_fx_stage.complex_io_summary.secondary_input_group_count,
        1
    );

    let _supervisor = signal_runtime::RuntimeSupervisorReport::capture(&runtime, &recorder);
}
