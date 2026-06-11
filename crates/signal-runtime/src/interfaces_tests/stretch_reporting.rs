use super::*;

#[test]
fn runtime_stretch_engine_snapshot_derives_from_clip_processing_baselines() {
    let pipeline = RuntimeClipProcessingPipelineSnapshot {
        clip_count: 4,
        ready_clip_count: 3,
        pending_media_clip_count: 0,
        pending_warp_clip_count: 0,
        invalid_clip_count: 1,
        faded_clip_count: 0,
        gain_shaped_clip_count: 0,
        warped_clip_count: 3,
        treatment_stage_count: 3,
        clips: vec![
            RuntimeClipProcessingSnapshot {
                clip_id: "clip:stretch-disabled".into(),
                media_asset_id: None,
                warp_mode: RuntimeWarpMode::Off,
                start_samples: 0,
                duration_samples: 64,
                fade_in: RuntimeClipFadeEnvelope::default(),
                fade_out: RuntimeClipFadeEnvelope::default(),
                fade_in_end_samples: 0,
                fade_out_start_samples: 64,
                clip_gain: RuntimeClipGainEnvelope::default(),
                treatment_stages: Vec::new(),
                realized_warp_ratio: None,
                project_tempo_source: None,
                project_tempo_segment_id: None,
                readiness: RuntimeClipProcessingReadiness::Ready,
                last_error: None,
            },
            RuntimeClipProcessingSnapshot {
                clip_id: "clip:stretch-ratio".into(),
                media_asset_id: Some("asset:ratio".into()),
                warp_mode: RuntimeWarpMode::Repitch,
                start_samples: 0,
                duration_samples: 64,
                fade_in: RuntimeClipFadeEnvelope::default(),
                fade_out: RuntimeClipFadeEnvelope::default(),
                fade_in_end_samples: 0,
                fade_out_start_samples: 64,
                clip_gain: RuntimeClipGainEnvelope::default(),
                treatment_stages: vec![RuntimeClipProcessingStage::Warp],
                realized_warp_ratio: Some(0.75),
                project_tempo_source: Some(RuntimeTempoSource::TransportProjection),
                project_tempo_segment_id: None,
                readiness: RuntimeClipProcessingReadiness::Ready,
                last_error: None,
            },
            RuntimeClipProcessingSnapshot {
                clip_id: "clip:stretch-sample-domain".into(),
                media_asset_id: Some("asset:sample-domain".into()),
                warp_mode: RuntimeWarpMode::ElastiqueDraft,
                start_samples: 0,
                duration_samples: 64,
                fade_in: RuntimeClipFadeEnvelope::default(),
                fade_out: RuntimeClipFadeEnvelope::default(),
                fade_in_end_samples: 0,
                fade_out_start_samples: 64,
                clip_gain: RuntimeClipGainEnvelope::default(),
                treatment_stages: vec![RuntimeClipProcessingStage::Warp],
                realized_warp_ratio: Some(1.5),
                project_tempo_source: Some(RuntimeTempoSource::TransportProjection),
                project_tempo_segment_id: None,
                readiness: RuntimeClipProcessingReadiness::Ready,
                last_error: None,
            },
            RuntimeClipProcessingSnapshot {
                clip_id: "clip:stretch-fallback".into(),
                media_asset_id: Some("asset:fallback".into()),
                warp_mode: RuntimeWarpMode::ElastiqueDraft,
                start_samples: 0,
                duration_samples: 64,
                fade_in: RuntimeClipFadeEnvelope::default(),
                fade_out: RuntimeClipFadeEnvelope::default(),
                fade_in_end_samples: 0,
                fade_out_start_samples: 64,
                clip_gain: RuntimeClipGainEnvelope::default(),
                treatment_stages: vec![RuntimeClipProcessingStage::Warp],
                realized_warp_ratio: Some(0.6),
                project_tempo_source: Some(RuntimeTempoSource::TransportProjection),
                project_tempo_segment_id: None,
                readiness: RuntimeClipProcessingReadiness::Invalid,
                last_error: Some("outside baseline support".into()),
            },
        ],
    };

    let stretch = RuntimeStretchEngineSnapshot::from_clip_processing_pipeline(&pipeline);

    assert_eq!(stretch.clip_count, 4);
    assert_eq!(stretch.disabled_clip_count, 1);
    assert_eq!(stretch.ready_clip_count, 2);
    assert_eq!(stretch.pending_media_clip_count, 0);
    assert_eq!(stretch.pending_warp_clip_count, 0);
    assert_eq!(stretch.degraded_clip_count, 1);
    assert_eq!(stretch.sample_domain_clip_count, 1);
    assert_eq!(stretch.ratio_only_clip_count, 1);
    assert_eq!(stretch.fallback_clip_count, 1);
    assert_eq!(
        stretch.clips[0].engine_class,
        RuntimeStretchEngineClass::Disabled
    );
    assert_eq!(
        stretch.clips[1].engine_class,
        RuntimeStretchEngineClass::RatioOnly
    );
    assert_eq!(stretch.clips[1].readiness, RuntimeStretchReadiness::Ready);
    assert_eq!(
        stretch.clips[2].engine_class,
        RuntimeStretchEngineClass::SampleDomain
    );
    assert_eq!(stretch.clips[2].readiness, RuntimeStretchReadiness::Ready);
    assert_eq!(
        stretch.clips[3].engine_class,
        RuntimeStretchEngineClass::Fallback
    );
    assert_eq!(
        stretch.clips[3].readiness,
        RuntimeStretchReadiness::Degraded
    );
    assert_eq!(
        stretch.clips[3].fallback_kind,
        RuntimeStretchFallbackKind::RatioOnly
    );
}
