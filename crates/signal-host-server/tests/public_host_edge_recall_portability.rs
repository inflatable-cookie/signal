#[path = "support/public_host_edge_continuity.rs"]
mod public_host_edge_continuity_support;
#[path = "support/public_host_edge_runtime_recall.rs"]
mod public_host_edge_runtime_recall;

use public_host_edge_continuity_support::{
    apply_public_plugin_continuity_graph, record_public_plugin_sandbox_ready,
};
use public_host_edge_runtime_recall::sample_server_ara_context;
use signal_host_server::ServerRuntimeHost;
use signal_plugin::{PluginFeature, PluginFormat};
use signal_runtime::{
    PluginScanRequest, RuntimeConfig, RuntimeConfigRequest, RuntimeLifecycleApi,
    RuntimeMultichannelIoSummary, RuntimePluginComplexIoSummary, RuntimePluginDiscoveredTypeRecord,
    RuntimePluginRecallPortabilityClass, SignalRuntime,
};

#[test]
fn server_shared_host_edge_exports_runtime_recall_portability_truth() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::server(48_000, 512));
    runtime
        .handshake(signal_runtime::HandshakeRequest {
            client_version: "public-host-server-recall-portability".into(),
            anticipative_preferred: true,
            max_sample_rate_hint: Some(96_000),
        })
        .expect("server host-edge recall portability handshake should succeed");
    runtime
        .configure(RuntimeConfigRequest::new(48_000, 512))
        .expect("server host-edge recall portability configure should succeed");
    let scan_handle = runtime.record_plugin_scan_request(&PluginScanRequest {
        roots: vec!["/usr/lib/vst3".into()],
        formats: vec![PluginFormat::Vst3],
    });
    runtime.record_plugin_scan_results(
        scan_handle,
        vec![RuntimePluginDiscoveredTypeRecord {
            plugin_type_id: "plugin:vst3:server-recall".into(),
            plugin_id: "com.signal.server-recall".into(),
            vendor: "Signal".into(),
            name: "Signal Server Recall".into(),
            format: PluginFormat::Vst3,
            version: Some("1.0.0".into()),
            features: vec![PluginFeature::Instrument],
            default_io_layout: signal_plugin::PluginIoLayout {
                audio_inputs: 0,
                audio_outputs: 2,
                midi_inputs: 1,
                midi_outputs: 0,
            },
            default_multichannel_io: RuntimeMultichannelIoSummary::for_plugin_io(
                signal_plugin::PluginIoLayout {
                    audio_inputs: 0,
                    audio_outputs: 2,
                    midi_inputs: 1,
                    midi_outputs: 0,
                },
            ),
            complex_io_summary: RuntimePluginComplexIoSummary::from_plugin_features_and_layout(
                &[PluginFeature::Instrument],
                signal_plugin::PluginIoLayout {
                    audio_inputs: 0,
                    audio_outputs: 2,
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
            summary: "server host recall portability type".into(),
        }],
    );
    apply_public_plugin_continuity_graph(
        &mut runtime,
        "graph:host-server:recall-portability",
        &[("node-server-vst3", "sandbox-server-vst3")],
    );
    record_public_plugin_sandbox_ready(
        &mut runtime,
        "sandbox-server-vst3",
        PluginFormat::Vst3,
        "plugin:vst3:server-recall",
        52,
    );
    runtime.record_plugin_ara_context("sandbox-server-vst3", sample_server_ara_context());

    let host = ServerRuntimeHost::new(runtime);
    let report = host.supervisor_report();
    let recall = report
        .observation
        .execution_topology_summary
        .nodes
        .iter()
        .find(|node| node.node_id == "node-server-vst3")
        .and_then(|node| node.plugin_recall.as_ref())
        .expect("server host-edge recall portability should be exported");
    assert_eq!(
        recall.payload.interchange.portability_class,
        RuntimePluginRecallPortabilityClass::ContextOnly
    );
    assert!(!recall.payload.interchange.shared_payload_available);
    assert_eq!(
        recall
            .payload
            .ara_context
            .as_ref()
            .and_then(|context| context.document_context.as_ref())
            .map(|document| document.document_id.as_str()),
        Some("doc:host-server")
    );

    let rendered = report.render_json();
    assert!(rendered.contains("\"interchange\":{"));
    assert!(rendered.contains("\"portability_class\":\"ContextOnly\""));
    assert!(rendered.contains("\"source_id\":\"source:stem-bus\""));
    assert!(rendered.contains("\"region_id\":\"region:bridge\""));
}
