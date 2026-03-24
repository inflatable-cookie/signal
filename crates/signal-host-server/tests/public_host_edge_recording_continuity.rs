#[path = "support/public_host_edge_continuity.rs"]
mod public_host_edge_continuity_support;

use public_host_edge_continuity_support::apply_public_capture_graph;
use signal_graph::synthetic_stereo_block;
use signal_host_server::ServerRuntimeHost;
use signal_primitives::{FrameCount, SampleRate};
use signal_runtime::{
    RuntimeConfig, RuntimeConfigRequest, RuntimeInterruptionClass, RuntimeLifecycleApi,
    RuntimeRecordingCaptureCheckpointClass, RuntimeRecordingCaptureKind,
    RuntimeRecordingCaptureStartRequest, SignalRuntime, StopReason,
};

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
