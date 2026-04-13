#[path = "support/public_host_edge_runtime_recall.rs"]
mod public_host_edge_runtime_recall;
#[path = "support/public_host_edge_continuity.rs"]
mod public_host_edge_continuity_support;

use public_host_edge_runtime_recall::{sample_host_ara_context, sample_host_preset_descriptor};
use public_host_edge_continuity_support::{
    apply_public_plugin_continuity_graph, record_public_plugin_sandbox_ready,
};
use signal_host_local::LocalRuntimeHost;
use signal_plugin::{PluginFeature, PluginFormat};
use signal_runtime::{
    PluginScanRequest, RuntimeConfig, RuntimeConfigRequest, RuntimeLifecycleApi,
    RuntimeMultichannelIoSummary, RuntimePluginComplexIoSummary, RuntimePluginDiscoveredTypeRecord,
    RuntimePluginRecallPortabilityClass, SignalRuntime,
};

#[test]
fn local_shared_host_edge_exports_runtime_recall_portability_truth() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
    runtime
        .handshake(signal_runtime::HandshakeRequest {
            client_version: "public-host-local-recall-portability".into(),
            anticipative_preferred: true,
            max_sample_rate_hint: Some(96_000),
        })
        .expect("local host-edge recall portability handshake should succeed");
    runtime
        .configure(RuntimeConfigRequest::new(48_000, 512))
        .expect("local host-edge recall portability configure should succeed");
    let scan_handle = runtime.record_plugin_scan_request(&PluginScanRequest {
        roots: vec!["~/.clap".into()],
        formats: vec![PluginFormat::Clap],
    });
    runtime.record_plugin_scan_results(
        scan_handle,
        vec![RuntimePluginDiscoveredTypeRecord {
            plugin_type_id: "plugin:clap:default".into(),
            plugin_id: "com.signal.local-default".into(),
            vendor: "Signal".into(),
            name: "Signal Local Default".into(),
            format: PluginFormat::Clap,
            version: Some("1.0.0".into()),
            features: vec![PluginFeature::AudioEffect],
            default_io_layout: signal_plugin::PluginIoLayout {
                audio_inputs: 2,
                audio_outputs: 2,
                midi_inputs: 1,
                midi_outputs: 0,
            },
            default_multichannel_io: RuntimeMultichannelIoSummary::for_plugin_io(
                signal_plugin::PluginIoLayout {
                    audio_inputs: 2,
                    audio_outputs: 2,
                    midi_inputs: 1,
                    midi_outputs: 0,
                },
            ),
            complex_io_summary: RuntimePluginComplexIoSummary::from_plugin_features_and_layout(
                &[PluginFeature::AudioEffect],
                signal_plugin::PluginIoLayout {
                    audio_inputs: 2,
                    audio_outputs: 2,
                    midi_inputs: 1,
                    midi_outputs: 0,
                },
            ),
            audio_bus_count: 2,
            parameter_count: 4,
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
            summary: "local host recall portability type".into(),
        }],
    );
    apply_public_plugin_continuity_graph(
        &mut runtime,
        "graph:host-local:recall-portability",
        &[("node-local-clap", "sandbox-local-clap")],
    );
    record_public_plugin_sandbox_ready(
        &mut runtime,
        "sandbox-local-clap",
        PluginFormat::Clap,
        "plugin:clap:default",
        41,
    );
    runtime.record_plugin_preset_descriptor("sandbox-local-clap", sample_host_preset_descriptor());
    runtime.record_plugin_ara_context("sandbox-local-clap", sample_host_ara_context());

    let host = LocalRuntimeHost::new(runtime);
    let report = host.supervisor_report();
    let recall = report
        .observation
        .execution_topology_summary
        .nodes
        .iter()
        .find(|node| node.node_id == "node-local-clap")
        .and_then(|node| node.plugin_recall.as_ref())
        .expect("local host-edge recall portability should be exported");
    assert_eq!(
        recall.payload.interchange.portability_class,
        RuntimePluginRecallPortabilityClass::Portable
    );
    assert_eq!(
        recall
            .payload
            .interchange
            .preset_descriptor
            .as_ref()
            .and_then(|descriptor| descriptor.label.as_deref()),
        Some("Local Lead")
    );
    assert_eq!(
        recall
            .payload
            .ara_context
            .as_ref()
            .and_then(|context| context.region_context.as_ref())
            .map(|region| region.region_id.as_str()),
        Some("region:chorus")
    );

    let rendered = report.render_json();
    assert!(rendered.contains("\"interchange\":{"));
    assert!(rendered.contains("\"portability_class\":\"Portable\""));
    assert!(rendered.contains("\"preset_id\":\"preset:user:local-lead\""));
    assert!(rendered.contains("\"document_id\":\"doc:host-local\""));
}
