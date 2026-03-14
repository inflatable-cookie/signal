use signal_graph::{synthetic_stereo_block, GraphNodeExecutionClass, GraphStageSpec};
use signal_host_local::LocalRuntimeHost;
use signal_plugin::PluginFormat;
use signal_primitives::{FrameCount, SampleRate};
use signal_runtime::{
    GraphNodeProjection, GraphProjection, PluginSandboxSpec, PluginScanRequest, RuntimeConfig,
    RuntimeConfigRequest, RuntimeInterruptionClass, RuntimeLifecycleApi, RuntimeProjectionApi,
    RuntimeRecordingCaptureKind, RuntimeRecordingCaptureStartRequest, RuntimeRecoveryState,
    RuntimeSupervisorApi, SafeModeRequest, SignalRuntime,
};

fn apply_public_capture_graph(runtime: &mut SignalRuntime, graph_id: &str) {
    runtime
        .apply_graph_projection(GraphProjection {
            graph_id: graph_id.into(),
            node_count: 2,
            nodes: vec![
                GraphNodeProjection {
                    node_id: "inline".into(),
                    execution_class: GraphNodeExecutionClass::PureTransform,
                    latency_samples: 0,
                    stages: vec![GraphStageSpec::Gain { linear: 0.9 }],
                },
                GraphNodeProjection {
                    node_id: "latency".into(),
                    execution_class: GraphNodeExecutionClass::LatencyBearing,
                    latency_samples: 24,
                    stages: vec![GraphStageSpec::HardClip { threshold: 0.6 }],
                },
            ],
        })
        .expect("public host-edge capture graph should apply");
}

#[test]
fn local_shared_host_edge_is_consumable_without_private_helpers() {
    let runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
    let mut host = LocalRuntimeHost::new(runtime);

    host.start_plugin_scan(PluginScanRequest {
        roots: vec!["~/Library/Audio/Plug-Ins/CLAP".into()],
        formats: vec![PluginFormat::Clap],
    })
    .expect("public host-edge scan should succeed");
    host.ensure_plugin_sandbox(PluginSandboxSpec {
        sandbox_id: "public-host-edge-local".into(),
        plugin_format: PluginFormat::Clap,
    })
    .expect("public host-edge sandbox ensure should succeed");

    let report = host.supervisor_report();
    assert_eq!(report.observation.plugin_discovery_snapshot.scan_count, 1);
    assert_eq!(
        report
            .observation
            .plugin_discovery_snapshot
            .discovered_type_count,
        2
    );
    assert_eq!(
        report.observation.plugin_lifecycle_snapshot.sandboxes.len(),
        1
    );
    assert_eq!(
        report.observation.plugin_lifecycle_snapshot.sandboxes[0].plugin_format,
        Some(PluginFormat::Clap)
    );
    assert_eq!(
        report.observation.fault_status.recovery_state,
        RuntimeRecoveryState::Steady
    );
    assert_eq!(
        report.observation.interruption_summary.class,
        RuntimeInterruptionClass::Steady
    );

    let rendered = report.render_json();
    assert!(rendered.contains("\"fault_status\":{"));
    assert!(rendered.contains("\"interruption_summary\":{"));
    assert!(rendered.contains("\"recording_capture_snapshot\":{"));
    assert!(rendered.contains("\"plugin_discovery_snapshot\":{"));
    assert!(rendered.contains("\"plugin_type_id\":\"plugin:clap:default\""));
    assert!(rendered.contains("\"event_stream\":"));
}

#[test]
fn local_shared_host_edge_exports_resumable_recording_checkpoint_truth() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
    runtime
        .handshake(signal_runtime::HandshakeRequest {
            client_version: "public-host-edge-recording".into(),
            anticipative_preferred: true,
            max_sample_rate_hint: Some(96_000),
        })
        .unwrap();
    runtime
        .configure(RuntimeConfigRequest::new(48_000, 512))
        .unwrap();
    apply_public_capture_graph(&mut runtime, "graph:host-local:recording");
    runtime.start().unwrap();
    runtime
        .start_recording_capture(RuntimeRecordingCaptureStartRequest {
            capture_kind: RuntimeRecordingCaptureKind::Audio,
            take_id: "take:local:resumable".into(),
            track_id: "track:local:resumable".into(),
            start_samples: 2_048,
            capture_path: std::env::temp_dir()
                .join("signal-local-host-recording-resumable.wav")
                .display()
                .to_string(),
        })
        .unwrap();
    runtime
        .process_engine_block(
            1,
            1,
            synthetic_stereo_block(SampleRate(48_000), FrameCount(8), 61),
        )
        .unwrap();
    runtime
        .set_safe_mode(SafeModeRequest { enabled: true })
        .unwrap();

    let host = LocalRuntimeHost::new(runtime);
    let report = host.supervisor_report();
    assert_eq!(
        report
            .observation
            .recording_capture_snapshot
            .active_checkpoint
            .as_ref()
            .map(|checkpoint| checkpoint.interruption_class),
        Some(RuntimeInterruptionClass::Resumable)
    );

    let rendered = report.render_json();
    assert!(rendered.contains("\"recording_capture_snapshot\":{"));
    assert!(rendered.contains("\"interruption_class\":\"Resumable\""));
}
