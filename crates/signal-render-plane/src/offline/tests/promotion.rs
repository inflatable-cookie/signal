use super::support::*;
use super::*;

#[test]
fn stretch_artifact_plan_blocks_export_without_accepted_promotion() {
    let input = stretch_identity_input();
    let plan = plan_offline_stretch_artifact(
        OfflineStretchArtifactScope::Export,
        &input,
        StretchPromotionReceipt::default(),
    )
    .expect("artifact plan");

    assert_eq!(
        plan.readiness,
        OfflineStretchArtifactReadiness::AwaitingCorpusEvidence
    );
    assert!(!plan.product_facing_allowed);
}

#[test]
fn stretch_artifact_materialization_blocks_without_accepted_promotion() {
    let input = stretch_identity_input();
    let source = stretch_artifact_source(480);

    assert_eq!(
        materialize_offline_stretch_artifact_pcm(
            OfflineStretchArtifactScope::Export,
            &input,
            StretchPromotionReceipt::default(),
            &source,
        ),
        Err(OfflineStretchArtifactMaterializeError::NotReady(
            OfflineStretchArtifactReadiness::AwaitingCorpusEvidence
        ))
    );
}
