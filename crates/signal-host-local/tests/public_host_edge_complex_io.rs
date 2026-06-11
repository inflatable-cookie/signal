#[path = "support/public_host_edge_complex_io.rs"]
mod public_host_edge_complex_io_support;

use public_host_edge_complex_io_support::{
    apply_public_complex_io_graph, sample_complex_bus_fx_record, sample_complex_multi_output_record,
};
use signal_graph::synthetic_stereo_block;
use signal_host_local::LocalRuntimeHost;
use signal_primitives::{AudioBuffer, ChannelLayout, FrameCount, SampleRate};
use signal_runtime::{
    RuntimeConfig, RuntimeConfigRequest, RuntimeLifecycleApi, RuntimePluginBusCapableFxClass,
    SignalRuntime,
};

#[test]
fn local_shared_host_edge_exports_runtime_complex_io_truth() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
    runtime
        .handshake(signal_runtime::HandshakeRequest {
            client_version: "public-host-local-complex-io".into(),
            anticipative_preferred: true,
            max_sample_rate_hint: Some(96_000),
        })
        .expect("public local complex io handshake should succeed");
    runtime
        .configure(RuntimeConfigRequest::new(48_000, 256))
        .expect("public local complex io configure should succeed");
    apply_public_complex_io_graph(&mut runtime, "graph:host-local:complex-io");
    let scan_handle = runtime.record_plugin_scan_request(&signal_runtime::PluginScanRequest {
        roots: vec!["~/Library/Audio/Plug-Ins/VST3".into()],
        formats: vec![signal_plugin::PluginFormat::Vst3],
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
            graph_id: "graph:host-local:complex-io".into(),
            processing_epoch: 1,
            block_sequence: 1,
            renders: vec![
                signal_runtime::PluginNodeRender {
                    node_id: "plugin-multiout".into(),
                    sandbox_id: "sandbox:host-local:multiout".into(),
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
                    sandbox_id: "sandbox:host-local:bus-fx".into(),
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
        .expect("public local complex io render batch should apply");
    runtime
        .process_engine_block(
            5,
            7,
            synthetic_stereo_block(SampleRate(48_000), FrameCount(256), 5),
        )
        .expect("public local complex io block should process");

    let host = LocalRuntimeHost::new(runtime);
    let report = host.supervisor_report();
    let discovery = &report.observation.plugin_discovery_snapshot;
    assert_eq!(discovery.discovered_type_count, 2);
    assert_eq!(discovery.capability_coverage.complex_io_type_count, 2);
    assert_eq!(
        discovery.capability_coverage.multi_output_instrument_count,
        1
    );
    assert_eq!(discovery.capability_coverage.bus_capable_fx_count, 1);
    assert!(discovery.discovered_types.iter().any(|record| {
        record.plugin_type_id == "plugin:vst3:host-local-multiout"
            && record.complex_io_summary.multi_output_instrument
    }));
    assert!(discovery.discovered_types.iter().any(|record| {
        record.plugin_type_id == "plugin:vst3:host-local-bus-fx"
            && record.complex_io_summary.bus_capable_fx_class
                == Some(RuntimePluginBusCapableFxClass::SendReturnCapableFx)
    }));

    let plugin_chain = &report.observation.plugin_chain_snapshot;
    assert!(plugin_chain.chain_count >= 1);
    assert_eq!(plugin_chain.stage_count, 2);
    assert!(plugin_chain
        .chains
        .iter()
        .flat_map(|chain| chain.stages.iter())
        .any(|stage| stage.node_id == "plugin-multiout"));
    assert!(plugin_chain
        .chains
        .iter()
        .flat_map(|chain| chain.stages.iter())
        .any(|stage| stage.node_id == "plugin-bus-fx"));
    let pin_matrix = &report.observation.plugin_pin_matrix_snapshot;
    assert_eq!(pin_matrix.plugin_type_count, 2);
    assert!(pin_matrix
        .records
        .iter()
        .any(|record| record.plugin_type_id == "plugin:vst3:host-local-multiout"));
    assert!(pin_matrix
        .records
        .iter()
        .any(|record| record.plugin_type_id == "plugin:vst3:host-local-bus-fx"));

}
