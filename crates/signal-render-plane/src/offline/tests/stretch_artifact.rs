use super::support::*;
use super::*;

/// The `g10.039` listening round found the adopted artifact path emitting
/// under four seconds of audio and then silence, with the length made up by
/// zero padding. Every artifact test at the time asserted lengths, identity,
/// and receipts, and none asserted that the audio was there.
///
/// This owner covers a source long enough to cross chunk boundaries and
/// requires signal in every decile of the output.
#[test]
fn multi_chunk_artifact_carries_audio_across_the_whole_output() {
    let identity = stretch_identity_input().with_ratio_curve(vec![StretchRatioPoint::new(0, 1.25)]);
    let frames = 48_000 * 90;
    let mut pcm = Vec::with_capacity(frames * 2);
    for f in 0..frames {
        let t = f as f32 / 48_000.0;
        let chord = 0.22 * (std::f32::consts::TAU * 220.0 * t).sin()
            + 0.18 * (std::f32::consts::TAU * 277.18 * t).sin();
        pcm.push(chord);
        pcm.push(chord * 0.85);
    }
    let source = RenderSampleBuffer::stereo(48_000, Arc::from(pcm.into_boxed_slice()));
    let artifact = materialize_offline_stretch_artifact_pcm(
        OfflineStretchArtifactScope::RenderCache,
        &identity,
        accepted_product_quality_promotion_receipt("evidence-artifact-content"),
        &source,
    )
    .expect("artifact materializes");

    assert!(
        artifact.chunk_plan.chunks.len() > 1,
        "case must cross a chunk boundary"
    );
    let out = &artifact.buffer.frames;
    let total = out.len() / 2;
    let slice = total / 10;
    for part in 0..10 {
        let start = part * slice;
        let seg = &out[start * 2..(start + slice) * 2];
        let rms = (seg.iter().map(|s| (s * s) as f64).sum::<f64>() / seg.len() as f64).sqrt();
        assert!(
            rms > 1.0e-4,
            "artifact decile {part} is silent: rms {rms:.9} at {:.1}s",
            start as f32 / 48_000.0
        );
    }
}
#[test]
fn ready_stretch_artifact_materializes_cacheable_pcm_for_render_consumers() {
    let input = stretch_identity_input().with_ratio_curve(vec![StretchRatioPoint::new(0, 1.25)]);
    let source = stretch_artifact_source(480);
    let artifact = materialize_offline_stretch_artifact_pcm(
        OfflineStretchArtifactScope::Freeze,
        &input,
        accepted_product_quality_promotion_receipt("product-quality:materialize-current"),
        &source,
    )
    .expect("composite product-quality evidence should materialize");
    assert_product_quality_promotion_receipt(
        &artifact.plan.promotion_receipt,
        "product-quality:materialize-current",
    );

    assert_eq!(artifact.plan.scope, OfflineStretchArtifactScope::Freeze);
    assert_eq!(
        artifact.plan.readiness,
        OfflineStretchArtifactReadiness::Ready
    );
    assert!(artifact.plan.product_facing_allowed);
    assert_eq!(artifact.input_frame_count, 480);
    assert_eq!(artifact.output_frame_count, 600);
    assert_eq!(
        artifact.receipt.cache_identity_hash,
        artifact.plan.identity.stable_hash
    );
    assert_eq!(
        artifact.receipt.offline_path,
        OfflineHighQualityPath::Default
    );
    assert_eq!(
        artifact.receipt.promotion_evidence_id,
        "product-quality:materialize-current"
    );
    assert_eq!(
        artifact.receipt.input_frame_count,
        artifact.input_frame_count
    );
    assert_eq!(
        artifact.receipt.output_frame_count,
        artifact.output_frame_count
    );
    assert!(artifact.receipt.product_facing_allowed);
    assert_eq!(artifact.buffer.sample_rate_hz, source.sample_rate_hz);
    assert_eq!(artifact.buffer.frame_count(), artifact.output_frame_count);
    assert_eq!(
        artifact.plan.identity,
        input.identity().expect("same identity")
    );

    let spec = RenderPlanSpec {
        sample_rate_hz: 48_000,
        master_gain: 1.0,
        master_limiter: None,
        stages: vec![
            lane(
                44,
                1.0,
                vec![RenderClipSpec {
                    clip_id: 440,
                    start_frames: 0,
                    end_frames: artifact.output_frame_count as u64,
                    source: RenderSource::Samples(artifact.buffer.clone()),
                    loop_source: false,
                    fade_in_frames: 0,
                    fade_out_frames: 0,
                }],
            ),
            master(vec![44]),
        ],
    };
    let rendered = render_plan_to_pcm(
        &spec,
        &OfflineRenderOptions {
            frame_count: artifact.output_frame_count as u64,
            ..OfflineRenderOptions::default()
        },
    )
    .expect("materialized artifact should render as a sample source");

    assert_eq!(rendered.master.len(), artifact.output_frame_count * 2);
}

#[test]
fn direct_receipt_materialization_keeps_lower_level_plan_gate() {
    let input = stretch_identity_input().with_ratio_curve(vec![StretchRatioPoint::new(0, 1.25)]);
    let source = stretch_artifact_source(480);
    let artifact = materialize_offline_stretch_artifact_pcm(
        OfflineStretchArtifactScope::RenderCache,
        &input,
        accepted_product_quality_promotion_receipt("product-quality:direct-materialize-current"),
        &source,
    )
    .expect("accepted direct receipt should materialize through the lower-level gate");

    assert_eq!(
        artifact.plan.scope,
        OfflineStretchArtifactScope::RenderCache
    );
    assert_eq!(
        artifact.plan.readiness,
        OfflineStretchArtifactReadiness::Ready
    );
    assert!(artifact.plan.product_facing_allowed);
    assert_eq!(artifact.input_frame_count, 480);
    assert_eq!(artifact.output_frame_count, 600);
    assert_eq!(
        artifact.receipt.cache_identity_hash,
        artifact.plan.identity.stable_hash
    );
    assert_eq!(
        artifact.receipt.promotion_evidence_id,
        "product-quality:direct-materialize-current"
    );
    assert_eq!(
        artifact.receipt.input_frame_count,
        artifact.input_frame_count
    );
    assert_eq!(
        artifact.receipt.output_frame_count,
        artifact.output_frame_count
    );
    assert!(artifact.receipt.product_facing_allowed);
    assert_eq!(artifact.buffer.sample_rate_hz, source.sample_rate_hz);
    assert_eq!(artifact.buffer.frame_count(), artifact.output_frame_count);
    assert_eq!(
        artifact.plan.identity,
        input.identity().expect("same identity")
    );

    assert_product_quality_promotion_receipt(
        &artifact.plan.promotion_receipt,
        "product-quality:direct-materialize-current",
    );
}

#[test]
fn chunked_artifact_materialization_records_bounded_chunk_plan() {
    let input = stretch_identity_input()
        .with_ratio_curve(vec![StretchRatioPoint::new(0, 1.25)])
        .with_pitch_curve(vec![StretchPitchPoint::new(0, 0.0)]);
    let source = stretch_artifact_source(1_024);
    let artifact = materialize_offline_stretch_artifact_pcm_with_chunk_config(
        OfflineStretchArtifactScope::RenderCache,
        &input,
        accepted_product_quality_promotion_receipt("product-quality:chunked-materialize-current"),
        &source,
        StretchOfflineChunkConfig::new(256, 64),
    )
    .expect("accepted direct receipt should materialize with bounded chunks");

    assert_eq!(artifact.chunk_plan.chunks.len(), 4);
    assert_eq!(
        artifact.chunk_plan.total_source_frames,
        source.frame_count()
    );
    assert_eq!(artifact.chunk_plan.total_output_frames, 1_280);
    assert_eq!(artifact.output_frame_count, 1_280);
    assert_eq!(artifact.buffer.frame_count(), artifact.output_frame_count);
    assert_eq!(artifact.receipt.chunk_count, 4);
    assert_eq!(artifact.receipt.max_chunk_source_frames, 256);
    assert_eq!(artifact.receipt.chunk_overlap_frames, 64);
    assert!(artifact.receipt.max_chunk_render_source_frames <= 256 + 64 * 2);
    assert_eq!(
        artifact.receipt.max_chunk_render_source_frames,
        artifact.chunk_plan.max_render_source_frames()
    );
    assert!(artifact.receipt.product_facing_allowed);
}

#[test]
fn chunked_artifact_materialization_is_deterministic_for_dynamic_ratio() {
    let input = stretch_identity_input()
        .with_ratio_curve(vec![
            StretchRatioPoint::new(0, 1.0),
            StretchRatioPoint::new(512, 1.25),
        ])
        .with_pitch_curve(vec![StretchPitchPoint::new(0, 0.0)]);
    let source = stretch_artifact_source(2_048);
    let receipt =
        accepted_product_quality_promotion_receipt("product-quality:chunked-dynamic-ratio");

    let first = materialize_offline_stretch_artifact_pcm_with_chunk_config(
        OfflineStretchArtifactScope::RenderCache,
        &input,
        receipt.clone(),
        &source,
        StretchOfflineChunkConfig::new(512, 128),
    )
    .expect("first chunked materialization should succeed");
    let repeated = materialize_offline_stretch_artifact_pcm_with_chunk_config(
        OfflineStretchArtifactScope::RenderCache,
        &input,
        receipt,
        &source,
        StretchOfflineChunkConfig::new(512, 128),
    )
    .expect("repeated chunked materialization should succeed");

    assert_eq!(first.output_frame_count, 2_432);
    assert_eq!(
        first.chunk_plan.total_output_frames,
        first.output_frame_count
    );
    assert_eq!(first.receipt.chunk_count, 4);
    assert_eq!(first.buffer.sample_rate_hz, repeated.buffer.sample_rate_hz);
    assert_eq!(first.buffer.frame_count(), repeated.buffer.frame_count());
    assert!(
        max_abs_delta(&first.buffer.frames, &repeated.buffer.frames) < 1.0e-6,
        "chunked materialization should be sample-stable"
    );
    assert_eq!(first.chunk_plan, repeated.chunk_plan);
    assert_eq!(
        first.receipt.cache_identity_hash,
        repeated.receipt.cache_identity_hash
    );
}

#[test]
fn composite_quality_artifacts_preserve_cache_identity_and_change_on_inputs() {
    let input = stretch_identity_input().with_ratio_curve(vec![StretchRatioPoint::new(0, 1.25)]);
    let changed_projection = StretchCacheIdentityInput {
        projection_epoch: "projection-43".to_string(),
        ..input.clone()
    };
    let changed_curve = stretch_identity_input()
        .with_ratio_curve(vec![StretchRatioPoint::new(0, 1.5)])
        .with_pitch_curve(vec![StretchPitchPoint::new(0, 0.0)]);
    let changed_path = stretch_identity_input()
        .with_ratio_curve(vec![StretchRatioPoint::new(0, 0.75)])
        .with_offline_path(OfflineHighQualityPath::CompressionShortWindowSelector);
    let source = stretch_artifact_source(480);
    let build = |identity_input: &StretchCacheIdentityInput| {
        materialize_offline_stretch_artifact_pcm(
            OfflineStretchArtifactScope::RenderCache,
            identity_input,
            accepted_product_quality_promotion_receipt("product-quality:builder-cache-identity"),
            &source,
        )
        .expect("composite evidence should produce cacheable PCM")
    };

    let base = build(&input);
    let repeated = build(&input);
    let projection_changed = build(&changed_projection);
    let curve_changed = build(&changed_curve);
    let path_changed = materialize_offline_stretch_artifact_pcm(
        OfflineStretchArtifactScope::RenderCache,
        &changed_path,
        accepted_selector_promotion_receipt("fma-rubberband:builder-cache-identity"),
        &source,
    )
    .expect("selector-specific evidence should produce selector cache identity");

    assert_eq!(
        base.receipt.cache_identity_hash,
        repeated.receipt.cache_identity_hash
    );
    assert_eq!(
        base.receipt.cache_identity_key,
        repeated.receipt.cache_identity_key
    );
    assert_ne!(
        base.receipt.cache_identity_hash,
        projection_changed.receipt.cache_identity_hash
    );
    assert_ne!(
        base.receipt.cache_identity_hash,
        curve_changed.receipt.cache_identity_hash
    );
    assert_ne!(
        base.receipt.cache_identity_hash,
        path_changed.receipt.cache_identity_hash
    );
    assert_eq!(
        path_changed.receipt.offline_path,
        OfflineHighQualityPath::CompressionShortWindowSelector
    );
    assert_ne!(base.buffer, projection_changed.buffer);
    assert_ne!(base.buffer, curve_changed.buffer);
    assert_ne!(base.output_frame_count, curve_changed.output_frame_count);
}
#[test]
fn chunked_artifact_renders_realistic_duration_through_export_fixture() {
    let input = stretch_identity_input()
        .with_ratio_curve(vec![
            StretchRatioPoint::new(0, 1.0),
            StretchRatioPoint::new(24_000, 1.25),
        ])
        .with_pitch_curve(vec![StretchPitchPoint::new(0, 0.0)]);
    let source = stretch_artifact_source(48_000);
    let artifact = materialize_offline_stretch_artifact_pcm_with_chunk_config(
        OfflineStretchArtifactScope::Export,
        &input,
        accepted_product_quality_promotion_receipt("product-quality:realistic-duration-chunked"),
        &source,
        StretchOfflineChunkConfig::new(8_000, 512),
    )
    .expect("realistic-duration chunked artifact should materialize");

    assert_eq!(artifact.input_frame_count, 48_000);
    assert_eq!(artifact.output_frame_count, 54_000);
    assert_eq!(artifact.receipt.chunk_count, 6);
    assert!(artifact.receipt.max_chunk_render_source_frames <= 9_024);

    let spec = RenderPlanSpec {
        sample_rate_hz: 48_000,
        master_gain: 1.0,
        master_limiter: None,
        stages: vec![
            lane(
                48,
                1.0,
                vec![RenderClipSpec {
                    clip_id: 480,
                    start_frames: 0,
                    end_frames: artifact.output_frame_count as u64,
                    source: RenderSource::Samples(artifact.buffer.clone()),
                    loop_source: false,
                    fade_in_frames: 0,
                    fade_out_frames: 0,
                }],
            ),
            master(vec![48]),
        ],
    };
    let rendered = render_plan_to_pcm(
        &spec,
        &OfflineRenderOptions {
            frame_count: artifact.output_frame_count as u64,
            block_frames: 512,
            ..OfflineRenderOptions::default()
        },
    )
    .expect("chunked artifact should render through export fixture");

    assert_eq!(rendered.master.len(), artifact.output_frame_count * 2);
    assert_eq!(rendered.sample_rate_hz, 48_000);
}
#[test]
fn stretch_artifact_plan_marks_unsupported_channel_layout_as_capability_blocker() {
    let input = StretchCacheIdentityInput::signal_native(
        StretchBackendTier::OfflineHighQuality,
        "sha256:mono-render-source",
        StretchChannelLayout::new(1, 48_000),
        "projection-mono",
    )
    .with_ratio_curve(vec![StretchRatioPoint::new(0, 1.25)])
    .with_pitch_curve(vec![StretchPitchPoint::new(0, 0.0)]);
    let source =
        RenderSampleBuffer::stereo(48_000, Arc::from(vec![0.0f32; 480].into_boxed_slice()));
    let receipt = accepted_product_quality_promotion_receipt("product-quality:mono-capability");
    let plan = plan_offline_stretch_artifact(
        OfflineStretchArtifactScope::RenderCache,
        &input,
        receipt.clone(),
    )
    .expect("mono identity should still produce an observable plan");

    assert_eq!(
        plan.readiness,
        OfflineStretchArtifactReadiness::UnsupportedCapability
    );
    assert_eq!(
        plan.capability_status,
        OfflineStretchArtifactCapabilityStatus::UnsupportedChannelLayout { channels: 1 }
    );
    assert!(!plan.product_facing_allowed);
    assert_eq!(
        materialize_offline_stretch_artifact_pcm(
            OfflineStretchArtifactScope::RenderCache,
            &input,
            receipt,
            &source,
        ),
        Err(OfflineStretchArtifactMaterializeError::UnsupportedChannelLayout { channels: 1 })
    );
}

#[test]
fn stretch_artifact_plan_marks_pitch_automation_as_capability_blocker() {
    let input = stretch_identity_input().with_pitch_curve(vec![
        StretchPitchPoint::new(0, 0.0),
        StretchPitchPoint::new(240, 2.0),
    ]);
    let source = stretch_artifact_source(480);
    let receipt =
        accepted_product_quality_promotion_receipt("product-quality:pitch-automation-plan");
    let plan = plan_offline_stretch_artifact(
        OfflineStretchArtifactScope::RenderCache,
        &input,
        receipt.clone(),
    )
    .expect("pitch automation identity should still produce an observable plan");

    assert_eq!(
        plan.readiness,
        OfflineStretchArtifactReadiness::UnsupportedCapability
    );
    assert_eq!(
        plan.capability_status,
        OfflineStretchArtifactCapabilityStatus::UnsupportedPitchAutomation
    );
    assert!(!plan.product_facing_allowed);
    assert_eq!(
        materialize_offline_stretch_artifact_pcm(
            OfflineStretchArtifactScope::RenderCache,
            &input,
            receipt,
            &source,
        ),
        Err(OfflineStretchArtifactMaterializeError::UnsupportedPitchAutomation)
    );
}

#[test]
fn incomplete_receipt_blocks_static_pitch_with_dynamic_ratio_curve() {
    let input = stretch_identity_input()
        .with_ratio_curve(vec![
            StretchRatioPoint::new(0, 1.0),
            StretchRatioPoint::new(240, 1.25),
        ])
        .with_pitch_curve(vec![StretchPitchPoint::new(0, 2.0)]);
    let source = stretch_artifact_source(480);

    let result = build_offline_stretch_artifact_pcm(incomplete_receipt_build_request(
        OfflineStretchArtifactScope::RenderCache,
        &input,
        "synthetic:static-pitch-dynamic-ratio",
        &source,
    ));

    assert_eq!(
        result,
        Err(OfflineStretchArtifactMaterializeError::NotReady(
            OfflineStretchArtifactReadiness::AwaitingCorpusEvidence
        ))
    );
}

#[test]
fn direct_receipt_materializes_static_pitch_with_dynamic_ratio_curve() {
    let input = stretch_identity_input()
        .with_ratio_curve(vec![
            StretchRatioPoint::new(0, 1.0),
            StretchRatioPoint::new(240, 1.25),
        ])
        .with_pitch_curve(vec![StretchPitchPoint::new(0, 2.0)]);
    let source = stretch_artifact_source(480);

    let artifact = materialize_offline_stretch_artifact_pcm(
        OfflineStretchArtifactScope::RenderCache,
        &input,
        accepted_product_quality_promotion_receipt(
            "product-quality:direct-static-pitch-dynamic-ratio",
        ),
        &source,
    )
    .expect("accepted direct receipt should materialize static pitch plus dynamic ratio");

    assert_eq!(artifact.output_frame_count, 540);
    assert_eq!(artifact.buffer.frame_count(), artifact.output_frame_count);
    assert_eq!(
        artifact.receipt.promotion_evidence_id,
        "product-quality:direct-static-pitch-dynamic-ratio"
    );
}

#[test]
fn stretch_artifact_materialization_rejects_pitch_automation() {
    let input = stretch_identity_input().with_pitch_curve(vec![
        StretchPitchPoint::new(0, 0.0),
        StretchPitchPoint::new(240, 2.0),
    ]);
    let source = stretch_artifact_source(480);

    assert_eq!(
        build_offline_stretch_artifact_pcm(incomplete_receipt_build_request(
            OfflineStretchArtifactScope::RenderCache,
            &input,
            "synthetic:pitch-automation",
            &source,
        ),),
        Err(OfflineStretchArtifactMaterializeError::UnsupportedPitchAutomation)
    );
}

#[test]
fn direct_receipt_materialization_rejects_pitch_automation() {
    let input = stretch_identity_input().with_pitch_curve(vec![
        StretchPitchPoint::new(0, 0.0),
        StretchPitchPoint::new(240, 2.0),
    ]);
    let source = stretch_artifact_source(480);

    assert_eq!(
        materialize_offline_stretch_artifact_pcm(
            OfflineStretchArtifactScope::RenderCache,
            &input,
            accepted_product_quality_promotion_receipt("product-quality:direct-pitch-automation",),
            &source,
        ),
        Err(OfflineStretchArtifactMaterializeError::UnsupportedPitchAutomation)
    );
}
#[test]
fn stretch_artifact_plan_changes_identity_when_projection_changes() {
    let input = stretch_identity_input();
    let changed = StretchCacheIdentityInput {
        projection_epoch: "projection-43".to_string(),
        ..stretch_identity_input()
    };
    let base_plan = plan_offline_stretch_artifact(
        OfflineStretchArtifactScope::RenderCache,
        &input,
        StretchPromotionReceipt::default(),
    )
    .expect("artifact plan");
    let changed_plan = plan_offline_stretch_artifact(
        OfflineStretchArtifactScope::RenderCache,
        &changed,
        StretchPromotionReceipt::default(),
    )
    .expect("artifact plan");

    assert_ne!(
        base_plan.identity.stable_hash,
        changed_plan.identity.stable_hash
    );
    assert!(!base_plan.product_facing_allowed);
    assert!(!changed_plan.product_facing_allowed);
}

#[test]
fn stretch_artifact_plan_rejects_preview_or_repitch_tiers() {
    let preview = StretchCacheIdentityInput::signal_native(
        StretchBackendTier::RealtimePreview,
        "sha256:render-source",
        StretchChannelLayout::new(2, 48_000),
        "projection-42",
    );
    let repitch = StretchCacheIdentityInput::signal_native(
        StretchBackendTier::Repitch,
        "sha256:render-source",
        StretchChannelLayout::new(2, 48_000),
        "projection-42",
    );

    assert_eq!(
        plan_offline_stretch_artifact(
            OfflineStretchArtifactScope::Export,
            &preview,
            accepted_product_quality_promotion_receipt("product-quality:ok"),
        ),
        Err(OfflineStretchArtifactPlanError::UnsupportedTier(
            StretchBackendTier::RealtimePreview
        ))
    );
    assert_eq!(
        plan_offline_stretch_artifact(
            OfflineStretchArtifactScope::Freeze,
            &repitch,
            accepted_product_quality_promotion_receipt("product-quality:ok"),
        ),
        Err(OfflineStretchArtifactPlanError::UnsupportedTier(
            StretchBackendTier::Repitch
        ))
    );
}
