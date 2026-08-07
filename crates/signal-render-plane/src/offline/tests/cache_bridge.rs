use super::support::*;
use super::*;

#[test]
fn render_cache_handoff_rejects_non_cache_scope_and_incomplete_receipt() {
    let input = stretch_identity_input().with_ratio_curve(vec![StretchRatioPoint::new(0, 1.25)]);
    let source = stretch_artifact_source(480);

    assert_eq!(
        build_offline_stretch_artifact_cache_handoff(artifact_build_request(
            OfflineStretchArtifactScope::Freeze,
            &input,
            rejected_promotion_receipt("evidence:cache-handoff-wrong-scope"),
            &source,
        )),
        Err(
            OfflineStretchArtifactMaterializeError::UnsupportedCacheHandoffScope {
                scope: OfflineStretchArtifactScope::Freeze
            }
        )
    );
    assert_eq!(
        build_offline_stretch_artifact_cache_handoff(artifact_build_request(
            OfflineStretchArtifactScope::RenderCache,
            &input,
            rejected_promotion_receipt("evidence:cache-handoff-incomplete"),
            &source,
        )),
        Err(OfflineStretchArtifactMaterializeError::NotReady(
            OfflineStretchArtifactReadiness::AwaitingCorpusEvidence
        ))
    );
}

#[test]
fn incomplete_receipt_cache_bridge_rejects_without_writing_handoffs() {
    let input = stretch_identity_input().with_ratio_curve(vec![StretchRatioPoint::new(0, 1.25)]);
    let changed_projection = StretchCacheIdentityInput {
        projection_epoch: "projection-44".to_string(),
        ..input.clone()
    };
    let source = stretch_artifact_source(480);
    let mut bridge = OfflineStretchArtifactRenderCacheBridge::new();

    assert!(bridge.is_empty());
    assert_eq!(
        bridge.resolve(cache_bridge_request(&input, &source)),
        Err(OfflineStretchArtifactMaterializeError::NotReady(
            OfflineStretchArtifactReadiness::AwaitingCorpusEvidence
        ))
    );
    assert_eq!(
        bridge.resolve(cache_bridge_request(&changed_projection, &source)),
        Err(OfflineStretchArtifactMaterializeError::NotReady(
            OfflineStretchArtifactReadiness::AwaitingCorpusEvidence
        ))
    );
    assert!(bridge.is_empty());
    assert!(!bridge.contains_identity_hash(
        &changed_projection
            .identity()
            .expect("changed identity should validate")
            .stable_hash
    ));
}

#[test]
fn receipt_owned_cache_bridge_writes_then_hits() {
    let input = stretch_identity_input().with_ratio_curve(vec![StretchRatioPoint::new(0, 1.25)]);
    let source = stretch_artifact_source(480);
    let mut bridge = OfflineStretchArtifactRenderCacheBridge::new();

    let first = bridge
        .resolve(artifact_build_request(
            OfflineStretchArtifactScope::RenderCache,
            &input,
            accepted_product_quality_promotion_receipt("product-quality:cache-bridge-write"),
            &source,
        ))
        .expect("accepted receipt should write cache handoff");
    assert_eq!(first.kind, OfflineStretchArtifactCacheDecisionKind::Written);
    assert_eq!(bridge.len(), 1);

    let second = bridge
        .resolve(artifact_build_request(
            OfflineStretchArtifactScope::RenderCache,
            &input,
            accepted_product_quality_promotion_receipt("product-quality:cache-bridge-hit"),
            &source,
        ))
        .expect("matching identity should reuse cache handoff");
    assert_eq!(second.kind, OfflineStretchArtifactCacheDecisionKind::Hit);
    assert_eq!(
        second.handoff.cache_identity_hash,
        first.handoff.cache_identity_hash
    );
    assert_eq!(bridge.len(), 1);
}

#[test]
fn materialization_receipts_audit_cache_identity_inputs() {
    let base = stretch_identity_input()
        .with_ratio_curve(vec![StretchRatioPoint::new(0, 1.25)])
        .with_pitch_curve(vec![StretchPitchPoint::new(0, 0.0)])
        .with_warp_markers(vec![StretchWarpMarker::new(0, 0)]);
    let changed_engine = StretchCacheIdentityInput {
        engine_version: "signal-native-stretch-v3-test".to_string(),
        ..base.clone()
    };
    let changed_media = StretchCacheIdentityInput {
        source_content_hash: "sha256:render-source-b".to_string(),
        ..base.clone()
    };
    let changed_projection = StretchCacheIdentityInput {
        projection_epoch: "projection-43".to_string(),
        ..base.clone()
    };
    let changed_ratio = base
        .clone()
        .with_ratio_curve(vec![StretchRatioPoint::new(0, 1.5)]);
    let changed_pitch = base
        .clone()
        .with_pitch_curve(vec![StretchPitchPoint::new(0, 1.0)]);
    let changed_marker = base
        .clone()
        .with_warp_markers(vec![StretchWarpMarker::new(96, 128)]);
    let source = stretch_artifact_source(96);
    let receipt = accepted_product_quality_promotion_receipt("product-quality:identity-audit");

    let mut observed = Vec::new();
    for input in [
        &base,
        &changed_engine,
        &changed_media,
        &changed_projection,
        &changed_ratio,
        &changed_pitch,
        &changed_marker,
    ] {
        let artifact = materialize_offline_stretch_artifact_pcm(
            OfflineStretchArtifactScope::RenderCache,
            input,
            receipt.clone(),
            &source,
        )
        .expect("identity variant should materialize");
        observed.push((
            artifact.receipt.cache_identity_hash,
            artifact.receipt.cache_identity_key,
        ));
    }

    for (index, (hash, _)) in observed.iter().enumerate() {
        assert!(
            observed
                .iter()
                .enumerate()
                .all(|(other_index, (other_hash, _))| index == other_index || hash != other_hash),
            "identity hash {hash} should be unique"
        );
    }
    assert!(observed[1]
        .1
        .contains("engine=signal-native-stretch-v3-test"));
    assert!(observed[2]
        .1
        .contains("source_content_hash=sha256:render-source-b"));
    assert!(observed[3].1.contains("projection_epoch=projection-43"));
    assert!(observed[4].1.contains("ratio_curve="));
    assert!(observed[5].1.contains("pitch_curve="));
    assert!(observed[6].1.contains("warp_markers=96:128"));
}

#[test]
fn materialization_receipts_make_chunk_policy_auditable() {
    let input = stretch_identity_input().with_ratio_curve(vec![StretchRatioPoint::new(0, 1.25)]);
    let source = stretch_artifact_source(1_024);
    let receipt = accepted_product_quality_promotion_receipt("product-quality:chunk-policy-audit");

    let fine = materialize_offline_stretch_artifact_pcm_with_chunk_config(
        OfflineStretchArtifactScope::RenderCache,
        &input,
        receipt.clone(),
        &source,
        StretchOfflineChunkConfig::new(256, 64),
    )
    .expect("fine chunk policy should materialize");
    let coarse = materialize_offline_stretch_artifact_pcm_with_chunk_config(
        OfflineStretchArtifactScope::RenderCache,
        &input,
        receipt,
        &source,
        StretchOfflineChunkConfig::new(512, 128),
    )
    .expect("coarse chunk policy should materialize");

    assert_eq!(fine.output_frame_count, coarse.output_frame_count);
    assert_eq!(fine.receipt.chunk_count, 4);
    assert_eq!(coarse.receipt.chunk_count, 2);
    assert_eq!(fine.receipt.max_chunk_source_frames, 256);
    assert_eq!(coarse.receipt.max_chunk_source_frames, 512);
    assert_eq!(fine.receipt.chunk_overlap_frames, 64);
    assert_eq!(coarse.receipt.chunk_overlap_frames, 128);
    assert_ne!(
        fine.receipt.max_chunk_render_source_frames,
        coarse.receipt.max_chunk_render_source_frames
    );
}
