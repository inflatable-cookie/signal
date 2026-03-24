#[path = "support/public_host_edge_multichannel_graph.rs"]
mod public_host_edge_multichannel_graph_support;

use public_host_edge_multichannel_graph_support::apply_public_multichannel_graph;
use signal_host_server::ServerRuntimeHost;
use signal_plugin::{PluginFeature, PluginFormat, PluginIoLayout};
use signal_runtime::{
    RuntimeBusIntent, RuntimeCanonicalChannelLayout, RuntimeConfig, RuntimeConfigRequest,
    RuntimeLifecycleApi, SignalRuntime,
};

#[test]
fn server_shared_host_edge_exports_runtime_multichannel_truth() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::server(48_000, 512));
    runtime
        .handshake(signal_runtime::HandshakeRequest {
            client_version: "public-host-server-multichannel".into(),
            anticipative_preferred: true,
            max_sample_rate_hint: Some(96_000),
        })
        .expect("server host-edge multichannel handshake should succeed");
    runtime
        .configure(RuntimeConfigRequest::new(48_000, 256))
        .expect("server host-edge multichannel configure should succeed");
    apply_public_multichannel_graph(&mut runtime, "graph:host-server:multichannel");
    let scan_handle = runtime.record_plugin_scan_request(&signal_runtime::PluginScanRequest {
        roots: vec!["/usr/lib/vst3".into()],
        formats: vec![PluginFormat::Vst3],
    });
    runtime.record_plugin_scan_results(
        scan_handle,
        vec![signal_runtime::RuntimePluginDiscoveredTypeRecord {
            plugin_type_id: "plugin:vst3:host-server-multichannel".into(),
            plugin_id: "com.signal.host-server-multichannel".into(),
            vendor: "Signal".into(),
            name: "Signal Host Server Multichannel".into(),
            format: PluginFormat::Vst3,
            version: Some("1.0.0".into()),
            features: vec![PluginFeature::Instrument],
            default_io_layout: PluginIoLayout {
                audio_inputs: 0,
                audio_outputs: 6,
                midi_inputs: 1,
                midi_outputs: 0,
            },
            default_multichannel_io: signal_runtime::RuntimeMultichannelIoSummary::for_plugin_io(
                PluginIoLayout {
                    audio_inputs: 0,
                    audio_outputs: 6,
                    midi_inputs: 1,
                    midi_outputs: 0,
                },
            ),
            complex_io_summary:
                signal_runtime::RuntimePluginComplexIoSummary::from_plugin_features_and_layout(
                    &[PluginFeature::Instrument],
                    PluginIoLayout {
                        audio_inputs: 0,
                        audio_outputs: 6,
                        midi_inputs: 1,
                        midi_outputs: 0,
                    },
                ),
            audio_bus_count: 1,
            parameter_count: 6,
            state_contract: signal_plugin::PluginStateContract {
                supports_snapshot: false,
                supports_reset: true,
                supports_bypass: false,
                exposes_latency: false,
                exposes_tail: true,
            },
            processing_contract: signal_plugin::PluginProcessingContract {
                max_block_frames: 1024,
                sample_accurate_automation: false,
                accepts_midi: true,
                accepts_note_events: true,
                supports_note_expression: true,
                produces_midi: false,
                silence_aware: false,
            },
            lifecycle_contract: signal_plugin::PluginLifecycleContract {
                requires_main_thread_for_state: true,
                supports_prepare: true,
                supports_activate: true,
                supports_reset_while_active: false,
            },
            lv2_extension_capabilities: None,
            summary: "server multichannel boundary plugin".into(),
        }],
    );

    let host = ServerRuntimeHost::new(runtime);
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
    assert_eq!(
        report
            .observation
            .external_io_snapshot
            .io_layout
            .output_layout
            .channel_count,
        0
    );
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
