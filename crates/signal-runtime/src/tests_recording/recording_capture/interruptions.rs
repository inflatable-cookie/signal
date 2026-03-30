use super::*;

#[test]
fn runtime_recording_capture_cancels_without_committing_file() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
    handshake_and_configure_with_anticipative(&mut runtime, true);
    apply_latency_runtime_graph(&mut runtime, "graph:runtime:recording-cancel");

    let capture_path = temp_capture_path("recording-cancel");
    runtime
        .start_recording_capture(RuntimeRecordingCaptureStartRequest {
            capture_kind: RuntimeRecordingCaptureKind::Audio,
            take_id: "take:test:cancel".to_string(),
            track_id: "track:test:cancel".to_string(),
            start_samples: 512,
            capture_path: capture_path.display().to_string(),
        })
        .unwrap();
    runtime
        .process_engine_block(
            1,
            1,
            synthetic_stereo_block(SampleRate(48_000), FrameCount(8), 33),
        )
        .unwrap();
    runtime.cancel_recording_capture().unwrap();

    let recording = runtime.get_recording_capture_snapshot();
    assert_eq!(recording.state, Some(RuntimeRecordingCaptureState::Idle));
    assert_eq!(recording.active_take_id, None);
    assert_eq!(recording.last_committed_path, None);
    assert_eq!(
        recording
            .last_checkpoint
            .as_ref()
            .map(|checkpoint| checkpoint.interruption_class),
        Some(RuntimeInterruptionClass::Restartable)
    );
    assert!(!capture_path.exists());
}

#[test]
fn runtime_recording_capture_preserves_restartable_checkpoint_across_stop_and_reconfigure() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
    handshake_and_configure_with_anticipative(&mut runtime, true);
    apply_latency_runtime_graph(&mut runtime, "graph:runtime:recording-restartable");
    runtime.start().unwrap();

    let capture_path = temp_capture_path("recording-restartable");
    runtime
        .start_recording_capture(RuntimeRecordingCaptureStartRequest {
            capture_kind: RuntimeRecordingCaptureKind::Audio,
            take_id: "take:test:restartable".to_string(),
            track_id: "track:test:restartable".to_string(),
            start_samples: 1_024,
            capture_path: capture_path.display().to_string(),
        })
        .unwrap();
    runtime
        .process_engine_block(
            1,
            1,
            synthetic_stereo_block(SampleRate(48_000), FrameCount(12), 91),
        )
        .unwrap();

    runtime.stop(StopReason::DeviceReconfigure).unwrap();
    runtime
        .configure(RuntimeConfigRequest {
            sample_rate: SampleRate(48_000),
            block_size: 256,
            anticipative_enabled: true,
            realtime_safe_mode: false,
            max_graph_latency_ms: None,
            max_background_load_percent: None,
        })
        .unwrap();

    let recording = runtime.get_recording_capture_snapshot();
    assert_eq!(recording.state, Some(RuntimeRecordingCaptureState::Idle));
    assert_eq!(
        recording.capture_kind,
        Some(RuntimeRecordingCaptureKind::Audio)
    );
    assert_eq!(
        recording
            .last_checkpoint
            .as_ref()
            .map(|checkpoint| checkpoint.checkpoint_class),
        Some(RuntimeRecordingCaptureCheckpointClass::Buffered)
    );
    assert_eq!(
        recording
            .last_checkpoint
            .as_ref()
            .map(|checkpoint| checkpoint.interruption_class),
        Some(RuntimeInterruptionClass::Restartable)
    );
    assert_eq!(
        recording
            .last_checkpoint
            .as_ref()
            .map(|checkpoint| checkpoint.buffered_frame_count),
        Some(12)
    );
    assert_eq!(recording.last_committed_path, None);
}

#[test]
fn runtime_recording_capture_reports_terminal_checkpoint_on_commit_failure() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
    handshake_and_configure_with_anticipative(&mut runtime, true);
    apply_latency_runtime_graph(&mut runtime, "graph:runtime:recording-terminal");

    runtime
        .start_recording_capture(RuntimeRecordingCaptureStartRequest {
            capture_kind: RuntimeRecordingCaptureKind::Audio,
            take_id: "take:test:terminal".to_string(),
            track_id: "track:test:terminal".to_string(),
            start_samples: 2_560,
            capture_path: "/dev/null/signal-runtime-recording-terminal.wav".to_string(),
        })
        .unwrap();
    runtime
        .process_engine_block(
            1,
            1,
            synthetic_stereo_block(SampleRate(48_000), FrameCount(8), 71),
        )
        .unwrap();

    let error = runtime.finish_recording_capture().unwrap_err();
    assert_eq!(error.kind, RuntimeErrorKind::ResourceUnavailable);

    let failed = runtime.get_recording_capture_snapshot();
    assert_eq!(failed.state, Some(RuntimeRecordingCaptureState::Failed));
    assert_eq!(failed.active_take_id, None);
    assert_eq!(
        failed
            .last_checkpoint
            .as_ref()
            .map(|checkpoint| checkpoint.checkpoint_class),
        Some(RuntimeRecordingCaptureCheckpointClass::Failed)
    );
    assert_eq!(
        failed
            .last_checkpoint
            .as_ref()
            .map(|checkpoint| checkpoint.interruption_class),
        Some(RuntimeInterruptionClass::Terminal)
    );
}
