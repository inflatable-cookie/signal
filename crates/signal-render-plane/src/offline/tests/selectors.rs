use super::support::*;
use super::*;

#[test]
fn compression_short_window_selector_materializes_static_stereo_and_changes_identity() {
    let default_input =
        stretch_identity_input().with_ratio_curve(vec![StretchRatioPoint::new(0, 0.75)]);
    let selector_input = default_input
        .clone()
        .with_offline_path(OfflineHighQualityPath::CompressionShortWindowSelector);
    let source = stretch_artifact_source(2_048);

    let default_artifact = materialize_offline_stretch_artifact_pcm(
        OfflineStretchArtifactScope::RenderCache,
        &default_input,
        accepted_product_quality_promotion_receipt("product-quality:default-path-static"),
        &source,
    )
    .expect("default path should materialize");
    let selector_artifact = materialize_offline_stretch_artifact_pcm(
        OfflineStretchArtifactScope::RenderCache,
        &selector_input,
        accepted_selector_promotion_receipt("fma-rubberband:selector-path-static"),
        &source,
    )
    .expect("selector path should materialize static stereo");

    assert_eq!(
        selector_artifact.plan.offline_path,
        OfflineHighQualityPath::CompressionShortWindowSelector
    );
    assert_eq!(
        selector_artifact.receipt.offline_path,
        OfflineHighQualityPath::CompressionShortWindowSelector
    );
    assert_eq!(selector_artifact.output_frame_count, 1_536);
    assert_ne!(
        default_artifact.receipt.cache_identity_hash,
        selector_artifact.receipt.cache_identity_hash
    );
    assert!(selector_artifact
        .receipt
        .cache_identity_key
        .contains("offline_path=compression-short-window-selector"));
}

#[test]
fn expansion_short_window_selector_materializes_static_stereo_and_changes_identity() {
    let default_input =
        stretch_identity_input().with_ratio_curve(vec![StretchRatioPoint::new(0, 1.25)]);
    let selector_input = default_input
        .clone()
        .with_offline_path(OfflineHighQualityPath::ExpansionShortWindowSelector);
    let source = stretch_artifact_source(2_048);

    let default_artifact = materialize_offline_stretch_artifact_pcm(
        OfflineStretchArtifactScope::RenderCache,
        &default_input,
        accepted_product_quality_promotion_receipt("product-quality:default-expansion-static"),
        &source,
    )
    .expect("default path should materialize");
    let selector_artifact = materialize_offline_stretch_artifact_pcm(
        OfflineStretchArtifactScope::RenderCache,
        &selector_input,
        accepted_expansion_selector_promotion_receipt(
            "fma-rubberband:expansion-selector-path-static",
        ),
        &source,
    )
    .expect("expansion selector path should materialize static stereo");

    assert_eq!(
        selector_artifact.plan.offline_path,
        OfflineHighQualityPath::ExpansionShortWindowSelector
    );
    assert_eq!(
        selector_artifact.receipt.offline_path,
        OfflineHighQualityPath::ExpansionShortWindowSelector
    );
    assert_eq!(selector_artifact.output_frame_count, 2_560);
    assert_ne!(
        default_artifact.receipt.cache_identity_hash,
        selector_artifact.receipt.cache_identity_hash
    );
    assert!(selector_artifact
        .receipt
        .cache_identity_key
        .contains("offline_path=expansion-short-window-selector"));
}

#[test]
fn compression_short_window_selector_rejects_default_promotion_receipt() {
    let selector_input = stretch_identity_input()
        .with_ratio_curve(vec![StretchRatioPoint::new(0, 0.75)])
        .with_offline_path(OfflineHighQualityPath::CompressionShortWindowSelector);
    let source = stretch_artifact_source(2_048);
    let default_receipt =
        accepted_product_quality_promotion_receipt("product-quality:default-path-not-selector");

    let plan = plan_offline_stretch_artifact(
        OfflineStretchArtifactScope::RenderCache,
        &selector_input,
        default_receipt.clone(),
    )
    .expect("selector plan should still validate identity");
    assert_eq!(
        plan.readiness,
        OfflineStretchArtifactReadiness::AwaitingCorpusEvidence
    );
    assert!(!plan.product_facing_allowed);
    assert_eq!(
        materialize_offline_stretch_artifact_pcm(
            OfflineStretchArtifactScope::RenderCache,
            &selector_input,
            default_receipt,
            &source,
        ),
        Err(OfflineStretchArtifactMaterializeError::NotReady(
            OfflineStretchArtifactReadiness::AwaitingCorpusEvidence
        ))
    );
    assert_eq!(
        build_offline_stretch_artifact_pcm(incomplete_receipt_build_request(
            OfflineStretchArtifactScope::RenderCache,
            &selector_input,
            "synthetic:selector-default-policy",
            &source,
        )),
        Err(OfflineStretchArtifactMaterializeError::NotReady(
            OfflineStretchArtifactReadiness::AwaitingCorpusEvidence
        ))
    );
}

#[test]
fn expansion_short_window_selector_rejects_default_promotion_receipt() {
    let selector_input = stretch_identity_input()
        .with_ratio_curve(vec![StretchRatioPoint::new(0, 1.25)])
        .with_offline_path(OfflineHighQualityPath::ExpansionShortWindowSelector);
    let source = stretch_artifact_source(2_048);
    let default_receipt = accepted_product_quality_promotion_receipt(
        "product-quality:default-path-not-expansion-selector",
    );

    let plan = plan_offline_stretch_artifact(
        OfflineStretchArtifactScope::RenderCache,
        &selector_input,
        default_receipt.clone(),
    )
    .expect("expansion selector plan should still validate identity");
    assert_eq!(
        plan.readiness,
        OfflineStretchArtifactReadiness::AwaitingCorpusEvidence
    );
    assert!(!plan.product_facing_allowed);
    assert_eq!(
        materialize_offline_stretch_artifact_pcm(
            OfflineStretchArtifactScope::RenderCache,
            &selector_input,
            default_receipt,
            &source,
        ),
        Err(OfflineStretchArtifactMaterializeError::NotReady(
            OfflineStretchArtifactReadiness::AwaitingCorpusEvidence
        ))
    );
}

#[test]
fn compression_short_window_selector_rejects_unproven_artifact_combinations() {
    let dynamic_input = stretch_identity_input()
        .with_ratio_curve(vec![
            StretchRatioPoint::new(0, 0.75),
            StretchRatioPoint::new(240, 1.25),
        ])
        .with_offline_path(OfflineHighQualityPath::CompressionShortWindowSelector);
    let pitch_input = stretch_identity_input()
        .with_ratio_curve(vec![StretchRatioPoint::new(0, 0.75)])
        .with_pitch_curve(vec![StretchPitchPoint::new(0, 2.0)])
        .with_offline_path(OfflineHighQualityPath::CompressionShortWindowSelector);
    let source = stretch_artifact_source(480);

    assert_eq!(
        materialize_offline_stretch_artifact_pcm(
            OfflineStretchArtifactScope::RenderCache,
            &dynamic_input,
            accepted_selector_promotion_receipt("fma-rubberband:selector-dynamic"),
            &source,
        ),
        Err(
            OfflineStretchArtifactMaterializeError::UnsupportedOfflinePathDynamicRatio {
                path: OfflineHighQualityPath::CompressionShortWindowSelector
            }
        )
    );
    assert_eq!(
        materialize_offline_stretch_artifact_pcm(
            OfflineStretchArtifactScope::RenderCache,
            &pitch_input,
            accepted_selector_promotion_receipt("fma-rubberband:selector-pitch"),
            &source,
        ),
        Err(
            OfflineStretchArtifactMaterializeError::UnsupportedOfflinePathPitchShift {
                path: OfflineHighQualityPath::CompressionShortWindowSelector
            }
        )
    );
}

#[test]
fn expansion_short_window_selector_rejects_unproven_artifact_combinations() {
    let dynamic_input = stretch_identity_input()
        .with_ratio_curve(vec![
            StretchRatioPoint::new(0, 1.25),
            StretchRatioPoint::new(240, 1.5),
        ])
        .with_offline_path(OfflineHighQualityPath::ExpansionShortWindowSelector);
    let pitch_input = stretch_identity_input()
        .with_ratio_curve(vec![StretchRatioPoint::new(0, 1.25)])
        .with_pitch_curve(vec![StretchPitchPoint::new(0, 2.0)])
        .with_offline_path(OfflineHighQualityPath::ExpansionShortWindowSelector);
    let source = stretch_artifact_source(480);

    assert_eq!(
        materialize_offline_stretch_artifact_pcm(
            OfflineStretchArtifactScope::RenderCache,
            &dynamic_input,
            accepted_expansion_selector_promotion_receipt(
                "fma-rubberband:expansion-selector-dynamic"
            ),
            &source,
        ),
        Err(
            OfflineStretchArtifactMaterializeError::UnsupportedOfflinePathDynamicRatio {
                path: OfflineHighQualityPath::ExpansionShortWindowSelector
            }
        )
    );
    assert_eq!(
        materialize_offline_stretch_artifact_pcm(
            OfflineStretchArtifactScope::RenderCache,
            &pitch_input,
            accepted_expansion_selector_promotion_receipt(
                "fma-rubberband:expansion-selector-pitch"
            ),
            &source,
        ),
        Err(
            OfflineStretchArtifactMaterializeError::UnsupportedOfflinePathPitchShift {
                path: OfflineHighQualityPath::ExpansionShortWindowSelector
            }
        )
    );
}
