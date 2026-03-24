#[path = "support/public_contract_boundary_graph_foundation.rs"]
mod public_contract_boundary_graph_foundation_support;

use public_contract_boundary_graph_foundation_support::apply_public_capture_graph;
use signal_graph::synthetic_stereo_block;
use signal_primitives::{FrameCount, SampleRate};
use signal_runtime::{
    HandshakeRequest, RuntimeConfig, RuntimeConfigRequest, RuntimeEventRecorder,
    RuntimeInterruptionClass, RuntimeLifecycleApi, RuntimeObservationReport,
    RuntimeRecordingCaptureCheckpointClass, RuntimeRecordingCaptureKind,
    RuntimeRecordingCaptureStartRequest, SafeModeRequest, SignalRuntime,
};

#[test]
fn public_runtime_recording_continuity_boundary_reports_resumable_restartable_and_terminal_states()
{
    let recorder = RuntimeEventRecorder::default();

    let mut resumable = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
    resumable
        .handshake(HandshakeRequest {
            client_version: "public-recording-continuity".into(),
            anticipative_preferred: true,
            max_sample_rate_hint: Some(96_000),
        })
        .expect("public recording continuity handshake should succeed");
    resumable
        .configure(RuntimeConfigRequest::new(48_000, 512))
        .expect("public recording continuity configure should succeed");
    apply_public_capture_graph(&mut resumable, "graph:public:recording-resumable");
    resumable
        .start()
        .expect("public recording continuity start should succeed");
    resumable
        .start_recording_capture(RuntimeRecordingCaptureStartRequest {
            capture_kind: RuntimeRecordingCaptureKind::Audio,
            take_id: "take:public:resumable".into(),
            track_id: "track:public:resumable".into(),
            start_samples: 2_048,
            capture_path: std::env::temp_dir()
                .join("signal-public-recording-resumable.wav")
                .display()
                .to_string(),
        })
        .expect("public recording capture should start");
    resumable
        .process_engine_block(
            1,
            1,
            synthetic_stereo_block(SampleRate(48_000), FrameCount(8), 44),
        )
        .expect("public recording block should process");
    resumable
        .set_safe_mode(SafeModeRequest { enabled: true })
        .expect("public recording safe mode should enable");
    let resumable_report = RuntimeObservationReport::capture(&resumable, &recorder);
    assert_eq!(
        resumable_report
            .recording_capture_snapshot
            .active_checkpoint
            .as_ref()
            .map(|checkpoint| checkpoint.interruption_class),
        Some(RuntimeInterruptionClass::Resumable)
    );

    let mut restartable = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
    restartable
        .handshake(HandshakeRequest {
            client_version: "public-recording-continuity".into(),
            anticipative_preferred: true,
            max_sample_rate_hint: Some(96_000),
        })
        .expect("public recording continuity handshake should succeed");
    restartable
        .configure(RuntimeConfigRequest::new(48_000, 512))
        .expect("public recording continuity configure should succeed");
    apply_public_capture_graph(&mut restartable, "graph:public:recording-restartable");
    restartable
        .start()
        .expect("public recording start should succeed");
    restartable
        .start_recording_capture(RuntimeRecordingCaptureStartRequest {
            capture_kind: RuntimeRecordingCaptureKind::Audio,
            take_id: "take:public:restartable".into(),
            track_id: "track:public:restartable".into(),
            start_samples: 3_072,
            capture_path: std::env::temp_dir()
                .join("signal-public-recording-restartable.wav")
                .display()
                .to_string(),
        })
        .expect("public restartable capture should start");
    restartable
        .process_engine_block(
            1,
            1,
            synthetic_stereo_block(SampleRate(48_000), FrameCount(8), 45),
        )
        .expect("public restartable block should process");
    restartable
        .stop(signal_runtime::StopReason::DeviceReconfigure)
        .expect("public restartable stop should succeed");
    let restartable_report = RuntimeObservationReport::capture(&restartable, &recorder);
    assert_eq!(
        restartable_report
            .recording_capture_snapshot
            .last_checkpoint
            .as_ref()
            .map(|checkpoint| checkpoint.interruption_class),
        Some(RuntimeInterruptionClass::Restartable)
    );
    assert_eq!(
        restartable_report
            .recording_capture_snapshot
            .last_checkpoint
            .as_ref()
            .map(|checkpoint| checkpoint.checkpoint_class),
        Some(RuntimeRecordingCaptureCheckpointClass::Buffered)
    );

    let mut terminal = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
    terminal
        .handshake(HandshakeRequest {
            client_version: "public-recording-continuity".into(),
            anticipative_preferred: true,
            max_sample_rate_hint: Some(96_000),
        })
        .expect("public recording continuity handshake should succeed");
    terminal
        .configure(RuntimeConfigRequest::new(48_000, 512))
        .expect("public recording continuity configure should succeed");
    apply_public_capture_graph(&mut terminal, "graph:public:recording-terminal");
    terminal
        .start_recording_capture(RuntimeRecordingCaptureStartRequest {
            capture_kind: RuntimeRecordingCaptureKind::Audio,
            take_id: "take:public:terminal".into(),
            track_id: "track:public:terminal".into(),
            start_samples: 4_096,
            capture_path: "/dev/null/signal-public-recording-terminal.wav".into(),
        })
        .expect("public terminal capture should start");
    terminal
        .process_engine_block(
            1,
            1,
            synthetic_stereo_block(SampleRate(48_000), FrameCount(8), 46),
        )
        .expect("public terminal block should process");
    let terminal_error = terminal.finish_recording_capture().unwrap_err();
    assert_eq!(
        terminal_error.kind,
        signal_runtime::RuntimeErrorKind::ResourceUnavailable
    );
    let terminal_report = RuntimeObservationReport::capture(&terminal, &recorder);
    assert_eq!(
        terminal_report.recording_capture_snapshot.state,
        Some(signal_runtime::RuntimeRecordingCaptureState::Failed)
    );
    assert_eq!(
        terminal_report
            .recording_capture_snapshot
            .last_checkpoint
            .as_ref()
            .map(|checkpoint| checkpoint.checkpoint_class),
        Some(RuntimeRecordingCaptureCheckpointClass::Failed)
    );
    assert_eq!(
        terminal_report
            .recording_capture_snapshot
            .last_checkpoint
            .as_ref()
            .map(|checkpoint| checkpoint.interruption_class),
        Some(RuntimeInterruptionClass::Terminal)
    );
}
