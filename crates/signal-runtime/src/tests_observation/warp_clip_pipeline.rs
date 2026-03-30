use super::*;

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

#[test]
fn runtime_reconciles_clip_processing_against_media_and_warp_readiness() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
    handshake_and_configure_with_anticipative(&mut runtime, true);

    let imported_path = temp_capture_path("clip-processing-ready");
    write_test_wav(&imported_path);
    runtime
        .reconcile_media_assets(vec![RuntimeMediaAssetRegistration {
            asset_id: "asset:sha256:clip-processing-ready".to_string(),
            content_hash: "clip-processing-ready".to_string(),
            source_path: imported_path.display().to_string(),
            file_name: "clip-processing-ready.wav".to_string(),
            byte_size: fs::metadata(&imported_path).unwrap().len(),
            sample_rate_hz: 48_000,
            channel_count: 1,
            duration_samples: 128,
            waveform_bin_count: 8,
        }])
        .unwrap();
    runtime
        .reconcile_warp_clips(vec![RuntimeWarpClipRegistration {
            clip_id: "clip:processing-ready".to_string(),
            media_asset_id: Some("asset:sha256:clip-processing-ready".to_string()),
            mode: RuntimeWarpMode::ElastiqueDraft,
            source_tempo_bpm: Some(120.0),
            anchor_timeline_samples: 0,
            start_samples: 0,
            duration_samples: 48_000,
        }])
        .unwrap();
    runtime
        .reconcile_clip_processing_clips(vec![RuntimeClipProcessingRegistration {
            clip_id: "clip:processing-ready".to_string(),
            media_asset_id: Some("asset:sha256:clip-processing-ready".to_string()),
            warp_mode: RuntimeWarpMode::ElastiqueDraft,
            start_samples: 0,
            duration_samples: 48_000,
            fade_in: RuntimeClipFadeEnvelope {
                duration_samples: 2_048,
                shape: RuntimeClipFadeShape::SmoothStep,
            },
            fade_out: RuntimeClipFadeEnvelope {
                duration_samples: 4_096,
                shape: RuntimeClipFadeShape::EqualPower,
            },
            clip_gain: RuntimeClipGainEnvelope {
                start_linear: 0.82,
                end_linear: 0.64,
                shape: RuntimeClipGainShape::Linear,
            },
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

    let ready = runtime.get_clip_processing_pipeline_snapshot();
    assert_eq!(ready.clip_count, 1);
    assert_eq!(ready.ready_clip_count, 1);
    assert_eq!(ready.pending_media_clip_count, 0);
    assert_eq!(ready.pending_warp_clip_count, 0);
    assert_eq!(ready.invalid_clip_count, 0);
    assert_eq!(ready.faded_clip_count, 1);
    assert_eq!(ready.gain_shaped_clip_count, 1);
    assert_eq!(ready.warped_clip_count, 1);
    assert_eq!(ready.treatment_stage_count, 4);
    assert_eq!(
        ready.clips[0].readiness,
        RuntimeClipProcessingReadiness::Ready
    );
    assert_eq!(ready.clips[0].fade_in_end_samples, 2_048);
    assert_eq!(ready.clips[0].fade_out_start_samples, 43_904);
    assert_eq!(
        ready.clips[0].treatment_stages,
        vec![
            RuntimeClipProcessingStage::Warp,
            RuntimeClipProcessingStage::FadeIn,
            RuntimeClipProcessingStage::GainShape,
            RuntimeClipProcessingStage::FadeOut,
        ]
    );
    assert_eq!(
        ready.clips[0].fade_in.shape,
        RuntimeClipFadeShape::SmoothStep
    );
    assert_eq!(
        ready.clips[0].fade_out.shape,
        RuntimeClipFadeShape::EqualPower
    );
    assert_eq!(ready.clips[0].clip_gain.shape, RuntimeClipGainShape::Linear);
    assert!((ready.clips[0].clip_gain.start_linear - 0.82).abs() < f32::EPSILON);
    assert!((ready.clips[0].clip_gain.end_linear - 0.64).abs() < f32::EPSILON);
    assert_eq!(
        ready.clips[0].project_tempo_source,
        Some(RuntimeTempoSource::TransportProjection)
    );
    assert!((ready.clips[0].realized_warp_ratio.unwrap_or_default() - 1.5).abs() < 0.000_1);

    runtime
        .apply_transport_projection(TransportProjection {
            playing: false,
            timeline_position_samples: 0,
            tempo_bpm: 300.0,
            loop_state: None,
        })
        .unwrap();

    let invalid = runtime.get_clip_processing_pipeline_snapshot();
    assert_eq!(invalid.clip_count, 1);
    assert_eq!(invalid.ready_clip_count, 0);
    assert_eq!(invalid.invalid_clip_count, 1);
    assert_eq!(
        invalid.clips[0].readiness,
        RuntimeClipProcessingReadiness::Invalid
    );
    assert!(invalid.clips[0]
        .last_error
        .as_deref()
        .unwrap_or_default()
        .contains("outside baseline support"));

    runtime
        .reconcile_clip_processing_clips(vec![RuntimeClipProcessingRegistration {
            clip_id: "clip:processing-ready".to_string(),
            media_asset_id: Some("asset:sha256:clip-processing-ready".to_string()),
            warp_mode: RuntimeWarpMode::ElastiqueDraft,
            start_samples: 0,
            duration_samples: 48_000,
            fade_in: RuntimeClipFadeEnvelope {
                duration_samples: 2_048,
                shape: RuntimeClipFadeShape::Linear,
            },
            fade_out: RuntimeClipFadeEnvelope {
                duration_samples: 4_096,
                shape: RuntimeClipFadeShape::Linear,
            },
            clip_gain: RuntimeClipGainEnvelope {
                start_linear: 0.82,
                end_linear: 0.64,
                shape: RuntimeClipGainShape::Hold,
            },
        }])
        .unwrap();
    let invalid_gain_shape = runtime.get_clip_processing_pipeline_snapshot();
    assert_eq!(invalid_gain_shape.invalid_clip_count, 1);
    assert_eq!(
        invalid_gain_shape.clips[0].readiness,
        RuntimeClipProcessingReadiness::Invalid
    );
    assert!(invalid_gain_shape.clips[0]
        .last_error
        .as_deref()
        .unwrap_or_default()
        .contains("hold clip gain shape requires identical start and end gain"));

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
