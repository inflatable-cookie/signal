use super::*;

#[test]
fn runtime_tempo_map_projection_drives_warp_ratio_and_export_reports() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
    handshake_and_configure_with_anticipative(&mut runtime, true);

    let imported_path = temp_capture_path("warp-tempo-map");
    write_test_wav(&imported_path);
    runtime
        .reconcile_media_assets(vec![RuntimeMediaAssetRegistration {
            asset_id: "asset:sha256:warp-tempo-map".to_string(),
            content_hash: "warp-tempo-map".to_string(),
            source_path: imported_path.display().to_string(),
            file_name: "warp-tempo-map.wav".to_string(),
            byte_size: fs::metadata(&imported_path).unwrap().len(),
            sample_rate_hz: 48_000,
            channel_count: 1,
            duration_samples: 128,
            waveform_bin_count: 8,
        }])
        .unwrap();
    runtime
        .apply_tempo_map_projection(RuntimeTempoMapProjection {
            segment_count: 2,
            segments: vec![
                crate::interfaces::RuntimeTempoMapSegmentProjection {
                    segment_id: "tempo:intro".to_string(),
                    start_samples: 0,
                    end_samples: Some(48_000),
                    start_tempo_bpm: 120.0,
                    end_tempo_bpm: None,
                    interpolation: RuntimeTempoMapInterpolation::Hold,
                },
                crate::interfaces::RuntimeTempoMapSegmentProjection {
                    segment_id: "tempo:lift".to_string(),
                    start_samples: 48_000,
                    end_samples: Some(96_000),
                    start_tempo_bpm: 120.0,
                    end_tempo_bpm: Some(180.0),
                    interpolation: RuntimeTempoMapInterpolation::Linear,
                },
            ],
        })
        .unwrap();
    runtime
        .reconcile_warp_clips(vec![RuntimeWarpClipRegistration {
            clip_id: "clip:warp-tempo-map".to_string(),
            media_asset_id: Some("asset:sha256:warp-tempo-map".to_string()),
            mode: RuntimeWarpMode::Repitch,
            source_tempo_bpm: Some(120.0),
            anchor_timeline_samples: 0,
            start_samples: 0,
            duration_samples: 48_000,
        }])
        .unwrap();
    runtime
        .apply_transport_projection(TransportProjection {
            playing: false,
            timeline_position_samples: 72_000,
            tempo_bpm: 90.0,
            loop_state: None,
        })
        .unwrap();

    let tempo_map = runtime.get_tempo_map_snapshot();
    assert_eq!(tempo_map.segment_count, 2);
    assert_eq!(tempo_map.active_segment_id.as_deref(), Some("tempo:lift"));
    assert_eq!(tempo_map.active_segment_index, Some(1));
    assert_eq!(tempo_map.tempo_source, RuntimeTempoSource::TempoMapSegment);
    assert!((tempo_map.resolved_tempo_bpm - 150.0).abs() < 0.000_1);

    let warp = runtime.get_warp_pipeline_snapshot();
    assert_eq!(warp.clip_count, 1);
    assert_eq!(warp.ready_clip_count, 1);
    assert_eq!(warp.degraded_clip_count, 0);
    assert_eq!(
        warp.resolved_project_tempo_source,
        RuntimeTempoSource::TempoMapSegment
    );
    assert_eq!(
        warp.resolved_project_tempo_segment_id.as_deref(),
        Some("tempo:lift")
    );
    assert!((warp.resolved_project_tempo_bpm - 150.0).abs() < 0.000_1);
    assert_eq!(
        warp.clips[0].project_tempo_source,
        RuntimeTempoSource::TempoMapSegment
    );
    assert_eq!(
        warp.clips[0].project_tempo_segment_id.as_deref(),
        Some("tempo:lift")
    );
    assert!((warp.clips[0].project_tempo_bpm - 150.0).abs() < 0.000_1);
    assert!((warp.clips[0].realized_ratio - 1.25).abs() < 0.000_1);

    let report = RuntimeObservationReport::capture(&runtime, &RuntimeEventRecorder::default());
    assert_eq!(
        report.tempo_map_snapshot.tempo_source,
        RuntimeTempoSource::TempoMapSegment
    );
    assert_eq!(
        report.warp_pipeline_snapshot.resolved_project_tempo_source,
        RuntimeTempoSource::TempoMapSegment
    );

    let _supervisor = RuntimeSupervisorReport::capture(&runtime, &RuntimeEventRecorder::default());

    runtime
        .apply_transport_projection(TransportProjection {
            playing: false,
            timeline_position_samples: 120_000,
            tempo_bpm: 90.0,
            loop_state: None,
        })
        .unwrap();
    let fallback_tempo_map = runtime.get_tempo_map_snapshot();
    assert_eq!(fallback_tempo_map.active_segment_id, None);
    assert_eq!(
        fallback_tempo_map.tempo_source,
        RuntimeTempoSource::TransportProjection
    );
    assert!((fallback_tempo_map.resolved_tempo_bpm - 90.0).abs() < 0.000_1);
    let fallback_warp = runtime.get_warp_pipeline_snapshot();
    assert_eq!(
        fallback_warp.resolved_project_tempo_source,
        RuntimeTempoSource::TransportProjection
    );
    assert_eq!(fallback_warp.resolved_project_tempo_segment_id, None);
    assert!((fallback_warp.clips[0].realized_ratio - 0.75).abs() < 0.000_1);

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
fn runtime_clip_render_path_applies_fade_gain_and_clip_bounds() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
    handshake_and_configure_with_anticipative(&mut runtime, true);

    runtime
        .reconcile_clip_processing_clips(vec![RuntimeClipProcessingRegistration {
            clip_id: "clip:render-envelope".to_string(),
            media_asset_id: None,
            warp_mode: RuntimeWarpMode::Off,
            start_samples: 10,
            duration_samples: 5,
            fade_in: RuntimeClipFadeEnvelope {
                duration_samples: 2,
                shape: RuntimeClipFadeShape::Linear,
            },
            fade_out: RuntimeClipFadeEnvelope {
                duration_samples: 2,
                shape: RuntimeClipFadeShape::Linear,
            },
            clip_gain: RuntimeClipGainEnvelope {
                start_linear: 1.0,
                end_linear: 0.5,
                shape: RuntimeClipGainShape::Linear,
            },
        }])
        .unwrap();

    let result = runtime
        .render_clip_processing_buffer(RuntimeClipRenderRequest {
            clip_id: "clip:render-envelope".to_string(),
            timeline_start_samples: 8,
            input_stage: RuntimeClipRenderInputStage::PostWarp,
            buffer: AudioBuffer::from_interleaved(
                SampleRate(48_000),
                ChannelLayout::Mono,
                vec![1.0; 7],
            ),
        })
        .unwrap();

    assert_eq!(
        result.clip_processing_snapshot.treatment_stages,
        vec![
            RuntimeClipProcessingStage::FadeIn,
            RuntimeClipProcessingStage::GainShape,
            RuntimeClipProcessingStage::FadeOut,
        ]
    );
    assert_eq!(result.timeline_start_samples, 8);
    assert_eq!(result.timeline_end_samples, 15);
    assert_eq!(result.first_frame_gain, Some(0.0));
    assert_eq!(result.last_frame_gain, Some(0.0));
    assert!((result.peak_applied_gain.unwrap_or_default() - 0.875).abs() < 1.0e-6);
    let expected = [0.0_f32, 0.0, 0.0, 0.875, 0.75, 0.625, 0.0];
    for (actual, expected) in result.output.samples().iter().zip(expected.iter()) {
        assert!((actual - expected).abs() < 1.0e-6);
    }
}

#[test]
fn runtime_clip_render_path_requires_post_warp_input_for_warp_enabled_clips() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
    handshake_and_configure_with_anticipative(&mut runtime, true);

    let imported_path = temp_capture_path("clip-render-post-warp");
    write_test_wav(&imported_path);
    runtime
        .reconcile_media_assets(vec![RuntimeMediaAssetRegistration {
            asset_id: "asset:sha256:clip-render-post-warp".to_string(),
            content_hash: "clip-render-post-warp".to_string(),
            source_path: imported_path.display().to_string(),
            file_name: "clip-render-post-warp.wav".to_string(),
            byte_size: fs::metadata(&imported_path).unwrap().len(),
            sample_rate_hz: 48_000,
            channel_count: 1,
            duration_samples: 128,
            waveform_bin_count: 8,
        }])
        .unwrap();
    runtime
        .reconcile_warp_clips(vec![RuntimeWarpClipRegistration {
            clip_id: "clip:render-post-warp".to_string(),
            media_asset_id: Some("asset:sha256:clip-render-post-warp".to_string()),
            mode: RuntimeWarpMode::Repitch,
            source_tempo_bpm: Some(120.0),
            anchor_timeline_samples: 0,
            start_samples: 0,
            duration_samples: 8,
        }])
        .unwrap();
    runtime
        .reconcile_clip_processing_clips(vec![RuntimeClipProcessingRegistration {
            clip_id: "clip:render-post-warp".to_string(),
            media_asset_id: Some("asset:sha256:clip-render-post-warp".to_string()),
            warp_mode: RuntimeWarpMode::Repitch,
            start_samples: 0,
            duration_samples: 8,
            fade_in: RuntimeClipFadeEnvelope {
                duration_samples: 0,
                shape: RuntimeClipFadeShape::Linear,
            },
            fade_out: RuntimeClipFadeEnvelope {
                duration_samples: 0,
                shape: RuntimeClipFadeShape::Linear,
            },
            clip_gain: RuntimeClipGainEnvelope {
                start_linear: 1.0,
                end_linear: 1.0,
                shape: RuntimeClipGainShape::Hold,
            },
        }])
        .unwrap();

    let raw_input_error = runtime
        .render_clip_processing_buffer(RuntimeClipRenderRequest {
            clip_id: "clip:render-post-warp".to_string(),
            timeline_start_samples: 0,
            input_stage: RuntimeClipRenderInputStage::RawClip,
            buffer: AudioBuffer::from_interleaved(
                SampleRate(48_000),
                ChannelLayout::Mono,
                vec![1.0; 8],
            ),
        })
        .expect_err("warp-enabled clip render should require post-warp input");
    assert_eq!(
        raw_input_error.kind,
        RuntimeErrorKind::UnsupportedCapability
    );
    assert!(raw_input_error.message.contains("require post-warp input"));

    let rendered = runtime
        .render_clip_processing_buffer(RuntimeClipRenderRequest {
            clip_id: "clip:render-post-warp".to_string(),
            timeline_start_samples: 0,
            input_stage: RuntimeClipRenderInputStage::PostWarp,
            buffer: AudioBuffer::from_interleaved(
                SampleRate(48_000),
                ChannelLayout::Mono,
                vec![0.25; 8],
            ),
        })
        .unwrap();
    assert_eq!(
        rendered.clip_processing_snapshot.treatment_stages,
        vec![RuntimeClipProcessingStage::Warp]
    );
    assert_eq!(
        rendered.clip_processing_snapshot.project_tempo_source,
        Some(RuntimeTempoSource::DefaultFallback)
    );
    assert_eq!(rendered.output.samples(), &[0.25; 8]);

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
