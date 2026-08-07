use super::super::support::*;
use super::super::*;

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
