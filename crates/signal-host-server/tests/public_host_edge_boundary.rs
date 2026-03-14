use signal_graph::{synthetic_stereo_block, GraphNodeExecutionClass, GraphStageSpec};
use signal_host_server::ServerRuntimeHost;
use signal_plugin::PluginFormat;
use signal_primitives::{FrameCount, SampleRate};
use signal_runtime::{
    GraphNodeProjection, GraphProjection, PluginSandboxSpec, PluginScanRequest, RuntimeConfig,
    RuntimeConfigRequest, RuntimeInterruptionClass, RuntimeLifecycleApi, RuntimeProjectionApi,
    RuntimeRecordingCaptureCheckpointClass, RuntimeRecordingCaptureKind,
    RuntimeRecordingCaptureStartRequest, RuntimeRecoveryState, RuntimeSupervisorApi, SignalRuntime,
    StopReason,
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
fn server_shared_host_edge_is_consumable_without_private_helpers() {
    let runtime = SignalRuntime::new(RuntimeConfig::server(48_000, 512));
    let mut host = ServerRuntimeHost::new(runtime);

    host.start_plugin_scan(PluginScanRequest {
        roots: vec!["/srv/plugins/clap".into()],
        formats: vec![PluginFormat::Clap],
    })
    .expect("public host-edge scan should succeed");
    host.ensure_plugin_sandbox(PluginSandboxSpec {
        sandbox_id: "public-host-edge-server".into(),
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
    assert!(rendered.contains("\"plugin_type_id\":\"plugin:clap:server\""));
    assert!(rendered.contains("\"event_stream\":"));
}

#[test]
fn server_shared_host_edge_exports_restartable_and_terminal_recording_checkpoint_truth() {
    let mut restartable_runtime = SignalRuntime::new(RuntimeConfig::server(48_000, 512));
    restartable_runtime
        .handshake(signal_runtime::HandshakeRequest {
            client_version: "public-host-edge-recording".into(),
            anticipative_preferred: true,
            max_sample_rate_hint: Some(96_000),
        })
        .unwrap();
    restartable_runtime
        .configure(RuntimeConfigRequest::new(48_000, 512))
        .unwrap();
    apply_public_capture_graph(
        &mut restartable_runtime,
        "graph:host-server:recording-restartable",
    );
    restartable_runtime.start().unwrap();
    restartable_runtime
        .start_recording_capture(RuntimeRecordingCaptureStartRequest {
            capture_kind: RuntimeRecordingCaptureKind::Audio,
            take_id: "take:server:restartable".into(),
            track_id: "track:server:restartable".into(),
            start_samples: 3_072,
            capture_path: std::env::temp_dir()
                .join("signal-server-host-recording-restartable.wav")
                .display()
                .to_string(),
        })
        .unwrap();
    restartable_runtime
        .process_engine_block(
            1,
            1,
            synthetic_stereo_block(SampleRate(48_000), FrameCount(8), 62),
        )
        .unwrap();
    restartable_runtime
        .stop(StopReason::DeviceReconfigure)
        .unwrap();

    let restartable_host = ServerRuntimeHost::new(restartable_runtime);
    let restartable_report = restartable_host.supervisor_report();
    assert_eq!(
        restartable_report
            .observation
            .recording_capture_snapshot
            .last_checkpoint
            .as_ref()
            .map(|checkpoint| checkpoint.interruption_class),
        Some(RuntimeInterruptionClass::Restartable)
    );
    assert_eq!(
        restartable_report
            .observation
            .recording_capture_snapshot
            .last_checkpoint
            .as_ref()
            .map(|checkpoint| checkpoint.checkpoint_class),
        Some(RuntimeRecordingCaptureCheckpointClass::Buffered)
    );

    let mut terminal_runtime = SignalRuntime::new(RuntimeConfig::server(48_000, 512));
    terminal_runtime
        .handshake(signal_runtime::HandshakeRequest {
            client_version: "public-host-edge-recording".into(),
            anticipative_preferred: true,
            max_sample_rate_hint: Some(96_000),
        })
        .unwrap();
    terminal_runtime
        .configure(RuntimeConfigRequest::new(48_000, 512))
        .unwrap();
    apply_public_capture_graph(
        &mut terminal_runtime,
        "graph:host-server:recording-terminal",
    );
    terminal_runtime
        .start_recording_capture(RuntimeRecordingCaptureStartRequest {
            capture_kind: RuntimeRecordingCaptureKind::Audio,
            take_id: "take:server:terminal".into(),
            track_id: "track:server:terminal".into(),
            start_samples: 4_096,
            capture_path: "/dev/null/signal-server-host-recording-terminal.wav".into(),
        })
        .unwrap();
    terminal_runtime
        .process_engine_block(
            1,
            1,
            synthetic_stereo_block(SampleRate(48_000), FrameCount(8), 63),
        )
        .unwrap();
    let _ = terminal_runtime.finish_recording_capture().unwrap_err();

    let terminal_host = ServerRuntimeHost::new(terminal_runtime);
    let terminal_report = terminal_host.supervisor_report();
    assert_eq!(
        terminal_report.observation.recording_capture_snapshot.state,
        Some(signal_runtime::RuntimeRecordingCaptureState::Failed)
    );
    assert_eq!(
        terminal_report
            .observation
            .recording_capture_snapshot
            .last_checkpoint
            .as_ref()
            .map(|checkpoint| checkpoint.interruption_class),
        Some(RuntimeInterruptionClass::Terminal)
    );

    let rendered = terminal_report.render_json();
    assert!(rendered.contains("\"recording_capture_snapshot\":{"));
    assert!(rendered.contains("\"interruption_class\":\"Terminal\""));
}
