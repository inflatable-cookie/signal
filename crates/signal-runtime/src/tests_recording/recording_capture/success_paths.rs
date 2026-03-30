use super::*;

#[test]
fn runtime_recording_capture_buffers_output_and_commits_wav() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
    handshake_and_configure_with_anticipative(&mut runtime, true);
    apply_latency_runtime_graph(&mut runtime, "graph:runtime:recording-capture");

    let capture_path = temp_capture_path("recording-capture");
    runtime
        .apply_transport_projection(TransportProjection {
            playing: true,
            timeline_position_samples: 2_048,
            tempo_bpm: 120.0,
            loop_state: None,
        })
        .unwrap();
    runtime
        .start_recording_capture(RuntimeRecordingCaptureStartRequest {
            capture_kind: RuntimeRecordingCaptureKind::Audio,
            take_id: "take:test:0001".to_string(),
            track_id: "track:test:0001".to_string(),
            start_samples: 2_048,
            capture_path: capture_path.display().to_string(),
        })
        .unwrap();

    runtime
        .process_engine_block(
            1,
            1,
            synthetic_stereo_block(SampleRate(48_000), FrameCount(16), 77),
        )
        .unwrap();

    let recording = runtime.get_recording_capture_snapshot();
    assert!(recording.capture_ready);
    assert_eq!(
        recording.state,
        Some(RuntimeRecordingCaptureState::Capturing)
    );
    assert_eq!(
        recording.capture_kind,
        Some(RuntimeRecordingCaptureKind::Audio)
    );
    assert_eq!(recording.active_take_id.as_deref(), Some("take:test:0001"));
    assert_eq!(recording.buffered_block_count, 1);
    assert_eq!(recording.buffered_frame_count, 16);
    assert_eq!(recording.buffered_event_count, 0);
    assert_eq!(recording.captured_channel_count, 2);
    assert_eq!(
        recording
            .active_checkpoint
            .as_ref()
            .map(|checkpoint| checkpoint.checkpoint_class),
        Some(RuntimeRecordingCaptureCheckpointClass::Streaming)
    );

    let receipt = runtime.finish_recording_capture().unwrap();
    assert_eq!(receipt.capture_kind, RuntimeRecordingCaptureKind::Audio);
    assert_eq!(receipt.take_id, "take:test:0001");
    assert_eq!(receipt.duration_samples, 16);
    assert_eq!(receipt.channel_count, 2);
    assert_eq!(
        receipt.committed_checkpoint.checkpoint_class,
        RuntimeRecordingCaptureCheckpointClass::Committed
    );
    assert!(capture_path.exists());

    let committed = runtime.get_recording_capture_snapshot();
    assert_eq!(committed.state, Some(RuntimeRecordingCaptureState::Idle));
    assert_eq!(
        committed.last_committed_path.as_deref(),
        Some(capture_path.to_string_lossy().as_ref())
    );
    assert_eq!(committed.last_committed_duration_samples, Some(16));
    assert_eq!(
        committed
            .last_checkpoint
            .as_ref()
            .map(|checkpoint| checkpoint.checkpoint_class),
        Some(RuntimeRecordingCaptureCheckpointClass::Committed)
    );

    let observation = RuntimeObservationReport::capture(&runtime, &RuntimeEventRecorder::default());
    assert_eq!(
        observation.recording_capture_snapshot.capture_kind,
        Some(RuntimeRecordingCaptureKind::Audio)
    );
    let observation_json = observation.render_json();
    assert!(observation_json.contains("\"recording_capture_snapshot\":{"));
    assert!(observation_json.contains("\"checkpoint_class\":\"Committed\""));

    let _ = fs::remove_file(capture_path);
}

#[test]
fn runtime_recording_capture_resumes_same_identity_after_safe_mode_clears() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
    handshake_and_configure_with_anticipative(&mut runtime, true);
    apply_latency_runtime_graph(&mut runtime, "graph:runtime:recording-resumable");
    runtime.start().unwrap();

    let capture_path = temp_capture_path("recording-resumable");
    runtime
        .start_recording_capture(RuntimeRecordingCaptureStartRequest {
            capture_kind: RuntimeRecordingCaptureKind::Audio,
            take_id: "take:test:resumable".to_string(),
            track_id: "track:test:resumable".to_string(),
            start_samples: 4_096,
            capture_path: capture_path.display().to_string(),
        })
        .unwrap();
    runtime
        .process_engine_block(
            1,
            1,
            synthetic_stereo_block(SampleRate(48_000), FrameCount(10), 55),
        )
        .unwrap();

    runtime
        .set_safe_mode(SafeModeRequest { enabled: true })
        .unwrap();
    let resumable = RuntimeObservationReport::capture(&runtime, &RuntimeEventRecorder::default());
    assert_eq!(
        resumable
            .recording_capture_snapshot
            .active_checkpoint
            .as_ref()
            .map(|checkpoint| checkpoint.interruption_class),
        Some(RuntimeInterruptionClass::Resumable)
    );
    assert_eq!(
        resumable
            .recording_capture_snapshot
            .active_take_id
            .as_deref(),
        Some("take:test:resumable")
    );

    runtime
        .set_safe_mode(SafeModeRequest { enabled: false })
        .unwrap();
    runtime
        .process_engine_block(
            2,
            2,
            synthetic_stereo_block(SampleRate(48_000), FrameCount(6), 56),
        )
        .unwrap();
    let receipt = runtime.finish_recording_capture().unwrap();
    assert_eq!(receipt.take_id, "take:test:resumable");
    assert_eq!(receipt.duration_samples, 16);

    let recording = runtime.get_recording_capture_snapshot();
    assert_eq!(
        recording
            .last_checkpoint
            .as_ref()
            .map(|checkpoint| checkpoint.checkpoint_class),
        Some(RuntimeRecordingCaptureCheckpointClass::Committed)
    );
    assert_eq!(
        recording.last_committed_take_id.as_deref(),
        Some("take:test:resumable")
    );

    let _ = fs::remove_file(capture_path);
}
