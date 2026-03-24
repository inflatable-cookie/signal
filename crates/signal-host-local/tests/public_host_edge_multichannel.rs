#[path = "support/public_host_edge_multichannel_graph.rs"]
mod public_host_edge_multichannel_graph_support;

use public_host_edge_multichannel_graph_support::apply_public_multichannel_graph;
use signal_host_local::LocalRuntimeHost;
use signal_plugin::{PluginFeature, PluginFormat, PluginIoLayout};
use signal_runtime::{
    RuntimeBusIntent, RuntimeCanonicalChannelLayout, RuntimeConfig, RuntimeConfigRequest,
    RuntimeLifecycleApi, SignalRuntime,
};

#[test]
fn local_shared_host_edge_exports_runtime_multichannel_truth() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
    runtime
        .handshake(signal_runtime::HandshakeRequest {
            client_version: "public-host-local-multichannel".into(),
            anticipative_preferred: true,
            max_sample_rate_hint: Some(96_000),
        })
        .expect("local host-edge multichannel handshake should succeed");
    runtime
        .configure(RuntimeConfigRequest::new(48_000, 256))
        .expect("local host-edge multichannel configure should succeed");
    apply_public_multichannel_graph(&mut runtime, "graph:host-local:multichannel");
    let scan_handle = runtime.record_plugin_scan_request(&signal_runtime::PluginScanRequest {
        roots: vec!["~/.clap".into()],
        formats: vec![PluginFormat::Clap],
    });
    runtime.record_plugin_scan_results(
        scan_handle,
        vec![signal_runtime::RuntimePluginDiscoveredTypeRecord {
            plugin_type_id: "plugin:clap:host-local-multichannel".into(),
            plugin_id: "com.signal.host-local-multichannel".into(),
            vendor: "Signal".into(),
            name: "Signal Host Local Multichannel".into(),
            format: PluginFormat::Clap,
            version: Some("1.0.0".into()),
            features: vec![PluginFeature::AudioEffect],
            default_io_layout: PluginIoLayout {
                audio_inputs: 2,
                audio_outputs: 6,
                midi_inputs: 1,
                midi_outputs: 0,
            },
            default_multichannel_io: signal_runtime::RuntimeMultichannelIoSummary::for_plugin_io(
                PluginIoLayout {
                    audio_inputs: 2,
                    audio_outputs: 6,
                    midi_inputs: 1,
                    midi_outputs: 0,
                },
            ),
            complex_io_summary:
                signal_runtime::RuntimePluginComplexIoSummary::from_plugin_features_and_layout(
                    &[PluginFeature::AudioEffect],
                    PluginIoLayout {
                        audio_inputs: 2,
                        audio_outputs: 6,
                        midi_inputs: 1,
                        midi_outputs: 0,
                    },
                ),
            audio_bus_count: 2,
            parameter_count: 6,
            state_contract: signal_plugin::PluginStateContract {
                supports_snapshot: true,
                supports_reset: true,
                supports_bypass: true,
                exposes_latency: true,
                exposes_tail: true,
            },
            processing_contract: signal_plugin::PluginProcessingContract {
                max_block_frames: 1024,
                sample_accurate_automation: true,
                accepts_midi: true,
                accepts_note_events: true,
                supports_note_expression: true,
                produces_midi: false,
                silence_aware: true,
            },
            lifecycle_contract: signal_plugin::PluginLifecycleContract {
                requires_main_thread_for_state: false,
                supports_prepare: true,
                supports_activate: true,
                supports_reset_while_active: true,
            },
            lv2_extension_capabilities: None,
            summary: "local multichannel boundary plugin".into(),
        }],
    );

    let host = LocalRuntimeHost::new(runtime);
    let report = host.supervisor_report();
    let track_node = report
        .observation
        .execution_topology_summary
        .nodes
        .iter()
        .find(|node| node.node_id == "surround-track")
        .expect("surround-track node should be present");
    assert_eq!(
        track_node.output_layout.canonical_layout,
        Some(RuntimeCanonicalChannelLayout::Surround5_1)
    );
    assert_eq!(track_node.output_bus_intent, RuntimeBusIntent::MainProgram);
    let send_node = report
        .observation
        .execution_topology_summary
        .nodes
        .iter()
        .find(|node| node.node_id == "analysis-send")
        .expect("analysis-send node should be present");
    assert_eq!(
        send_node.output_layout.canonical_layout,
        Some(RuntimeCanonicalChannelLayout::Quad)
    );
    assert_eq!(send_node.output_bus_intent, RuntimeBusIntent::AuxSend);
    assert_eq!(
        report
            .observation
            .plugin_discovery_snapshot
            .discovered_types[0]
            .default_multichannel_io
            .output_layout
            .canonical_layout,
        Some(RuntimeCanonicalChannelLayout::Surround5_1)
    );

    let rendered = report.render_json();
    assert!(rendered.contains("\"canonical_layout\":\"Surround5_1\""));
    assert!(rendered.contains("\"output_bus_intent\":\"MainProgram\""));
    assert!(rendered.contains("\"default_multichannel_io\":{"));
}
