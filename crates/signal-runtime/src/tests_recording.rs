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

#[test]
fn runtime_reconciles_media_assets_into_shared_ready_cache_state() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
    handshake_and_configure_with_anticipative(&mut runtime, true);

    let imported_path = temp_capture_path("media-imported");
    let recorded_path = temp_capture_path("media-recorded");
    write_test_wav(&imported_path);
    write_test_wav(&recorded_path);

    runtime
        .reconcile_media_assets(vec![
            RuntimeMediaAssetRegistration {
                asset_id: "asset:sha256:imported".to_string(),
                content_hash: "imported".to_string(),
                source_path: imported_path.display().to_string(),
                file_name: "imported.wav".to_string(),
                byte_size: fs::metadata(&imported_path).unwrap().len(),
                sample_rate_hz: 48_000,
                channel_count: 1,
                duration_samples: 128,
                waveform_bin_count: 8,
            },
            RuntimeMediaAssetRegistration {
                asset_id: "asset:sha256:recorded".to_string(),
                content_hash: "recorded".to_string(),
                source_path: recorded_path.display().to_string(),
                file_name: "recorded.wav".to_string(),
                byte_size: fs::metadata(&recorded_path).unwrap().len(),
                sample_rate_hz: 48_000,
                channel_count: 1,
                duration_samples: 128,
                waveform_bin_count: 8,
            },
        ])
        .unwrap();

    let snapshot = runtime.get_media_pipeline_snapshot();
    assert_eq!(snapshot.asset_count, 2);
    assert_eq!(snapshot.ready_asset_count, 2);
    assert_eq!(snapshot.invalid_asset_count, 0);
    assert!(snapshot.assets.iter().all(|asset| {
        asset.state == Some(RuntimeMediaAssetState::Ready) && asset.cache_path.as_deref().is_some()
    }));

    let cached_path = PathBuf::from(
        snapshot.assets[0]
            .cache_path
            .as_deref()
            .expect("cached media should exist"),
    );
    fs::remove_file(&cached_path).unwrap();

    runtime
        .reconcile_media_assets(vec![RuntimeMediaAssetRegistration {
            asset_id: "asset:sha256:imported".to_string(),
            content_hash: "imported".to_string(),
            source_path: imported_path.display().to_string(),
            file_name: "imported.wav".to_string(),
            byte_size: fs::metadata(&imported_path).unwrap().len(),
            sample_rate_hz: 48_000,
            channel_count: 1,
            duration_samples: 128,
            waveform_bin_count: 8,
        }])
        .unwrap();

    let rebuilt = runtime.get_media_pipeline_snapshot();
    assert_eq!(rebuilt.asset_count, 1);
    assert_eq!(rebuilt.ready_asset_count, 1);
    assert_eq!(rebuilt.assets[0].state, Some(RuntimeMediaAssetState::Ready));
    assert!(rebuilt.assets[0].rebuild_count >= 1);

    let _ = fs::remove_file(imported_path);
    let _ = fs::remove_file(recorded_path);
    if let Some(path) = rebuilt.assets[0].cache_path.as_deref() {
        let _ = fs::remove_file(path);
    }
}
