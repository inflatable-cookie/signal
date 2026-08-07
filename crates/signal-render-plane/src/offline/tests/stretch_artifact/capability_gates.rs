use super::super::support::*;
use super::super::*;

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
