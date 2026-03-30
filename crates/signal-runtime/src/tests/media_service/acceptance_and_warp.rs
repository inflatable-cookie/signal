use super::super::*;

#[test]
fn runtime_acceptance_receipt_scopes_integrated_runtime_lanes_and_targets() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
    handshake_and_configure_with_anticipative(&mut runtime, true);
    apply_latency_runtime_graph(&mut runtime, "graph:runtime:acceptance-scope");

    let ready_path = temp_capture_path("acceptance-media-ready");
    write_test_wav(&ready_path);
    runtime
        .reconcile_media_assets(vec![RuntimeMediaAssetRegistration {
            asset_id: "asset:sha256:acceptance".to_string(),
            content_hash: "acceptance".to_string(),
            source_path: ready_path.display().to_string(),
            file_name: "acceptance.wav".to_string(),
            byte_size: fs::metadata(&ready_path).unwrap().len(),
            sample_rate_hz: 48_000,
            channel_count: 1,
            duration_samples: 128,
            waveform_bin_count: 8,
        }])
        .expect("ready media should reconcile");
    runtime
        .start_recording_capture(RuntimeRecordingCaptureStartRequest {
            capture_kind: RuntimeRecordingCaptureKind::Audio,
            take_id: "take:acceptance".to_string(),
            track_id: "track:acceptance".to_string(),
            start_samples: 0,
            capture_path: temp_capture_path("acceptance-take").display().to_string(),
        })
        .expect("recording capture should start");

    let receipt = runtime.get_acceptance_receipt();
    assert_eq!(receipt.runtime_lane_count, 6);
    assert!(receipt.playback_ready);
    assert!(receipt.recording_ready);
    assert!(receipt.media_ready);
    assert!(!receipt.clip_processing_ready);
    assert!(!receipt.plugin_ready);
    assert!(receipt.recovery_ready);
    assert_eq!(receipt.minimum_trace_observation_count, 128);
    assert_eq!(receipt.minimum_soak_event_count, 64);
    assert_eq!(receipt.runtime_ready_lane_count, 4);

    let _ = fs::remove_file(ready_path);
}

#[test]
fn runtime_reconciles_warp_clips_against_media_readiness_and_project_tempo() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
    handshake_and_configure_with_anticipative(&mut runtime, true);

    let imported_path = temp_capture_path("warp-ready");
    write_test_wav(&imported_path);
    runtime
        .reconcile_media_assets(vec![RuntimeMediaAssetRegistration {
            asset_id: "asset:sha256:warp-ready".to_string(),
            content_hash: "warp-ready".to_string(),
            source_path: imported_path.display().to_string(),
            file_name: "warp-ready.wav".to_string(),
            byte_size: fs::metadata(&imported_path).unwrap().len(),
            sample_rate_hz: 48_000,
            channel_count: 1,
            duration_samples: 128,
            waveform_bin_count: 8,
        }])
        .unwrap();
    runtime
        .reconcile_warp_clips(vec![RuntimeWarpClipRegistration {
            clip_id: "clip:warp-ready".to_string(),
            media_asset_id: Some("asset:sha256:warp-ready".to_string()),
            mode: RuntimeWarpMode::ElastiqueDraft,
            source_tempo_bpm: Some(120.0),
            anchor_timeline_samples: 0,
            start_samples: 0,
            duration_samples: 48_000,
        }])
        .unwrap();
    runtime
        .apply_transport_projection(TransportProjection {
            playing: false,
            timeline_position_samples: 0,
            tempo_bpm: 180.0,
            loop_state: None,
        })
        .unwrap();

    let ready = runtime.get_warp_pipeline_snapshot();
    assert_eq!(ready.clip_count, 1);
    assert_eq!(ready.ready_clip_count, 1);
    assert_eq!(ready.degraded_clip_count, 0);
    assert_eq!(
        ready.resolved_project_tempo_source,
        RuntimeTempoSource::TransportProjection
    );
    assert_eq!(ready.clips[0].readiness, RuntimeWarpReadiness::Ready);
    assert_eq!(
        ready.clips[0].project_tempo_source,
        RuntimeTempoSource::TransportProjection
    );
    assert!((ready.clips[0].realized_ratio - 1.5).abs() < 0.000_1);

    runtime
        .apply_transport_projection(TransportProjection {
            playing: false,
            timeline_position_samples: 0,
            tempo_bpm: 300.0,
            loop_state: None,
        })
        .unwrap();
    let degraded = runtime.get_warp_pipeline_snapshot();
    assert_eq!(degraded.ready_clip_count, 0);
    assert_eq!(degraded.degraded_clip_count, 1);
    assert_eq!(
        degraded.resolved_project_tempo_source,
        RuntimeTempoSource::TransportProjection
    );
    assert_eq!(degraded.clips[0].readiness, RuntimeWarpReadiness::Degraded);
    assert!(degraded.clips[0]
        .last_error
        .as_deref()
        .unwrap_or_default()
        .contains("outside baseline support"));

    let _ = fs::remove_file(imported_path);
    if let Some(path) = runtime
        .get_media_pipeline_snapshot()
        .assets
        .first()
        .and_then(|asset| asset.cache_path.as_deref())
    {
        let _ = fs::remove_file(path);
    }
}
