use std::fs;

#[path = "support/public_contract_boundary_media.rs"]
mod public_contract_boundary_media_support;

use public_contract_boundary_media_support::{
    public_media_fixture_path, public_offline_stretch_artifact_materialization,
    write_public_test_wav,
};
use signal_runtime::{
    current_synthetic_offline_high_quality_promotion_receipt, HandshakeRequest, RuntimeConfig,
    RuntimeConfigRequest, RuntimeEventRecorder, RuntimeLifecycleApi, RuntimeMediaPreviewState,
    RuntimeObservationApi, RuntimeObservationReport,
    RuntimeOfflineStretchArtifactCacheDecisionKind,
    RuntimeOfflineStretchArtifactCacheDecisionRegistration,
    RuntimeOfflineStretchArtifactPlanRegistration, RuntimeOfflineStretchArtifactReadiness,
    RuntimeOfflineStretchArtifactScope, RuntimeSupervisorReport, SignalRuntime, StretchBackendTier,
    StretchCacheIdentityInput, StretchChannelLayout, StretchPitchPoint, StretchPromotionStatus,
    StretchRatioPoint, StretchWarpMarker,
};

#[test]
fn public_runtime_media_service_boundary_reports_runtime_owned_readiness_and_invalidation_truth() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
    runtime
        .handshake(HandshakeRequest {
            client_version: "public-runtime-media-service".into(),
            anticipative_preferred: true,
            max_sample_rate_hint: Some(96_000),
        })
        .expect("public runtime media-service handshake should succeed");
    runtime
        .configure(RuntimeConfigRequest::new(48_000, 256))
        .expect("public runtime media-service configure should succeed");
    let recorder = RuntimeEventRecorder::default();

    let ready_path = public_media_fixture_path("ready");
    let missing_path = public_media_fixture_path("missing");
    write_public_test_wav(&ready_path);

    runtime
        .reconcile_media_assets(vec![
            signal_runtime::RuntimeMediaAssetRegistration {
                asset_id: "asset:sha256:public-media-ready".into(),
                content_hash: "public-media-ready".into(),
                source_path: ready_path.display().to_string(),
                file_name: "public-media-ready.wav".into(),
                byte_size: fs::metadata(&ready_path)
                    .expect("public media fixture should exist")
                    .len(),
                sample_rate_hz: 48_000,
                channel_count: 1,
                duration_samples: 128,
                waveform_bin_count: 16,
            },
            signal_runtime::RuntimeMediaAssetRegistration {
                asset_id: "asset:sha256:public-media-missing".into(),
                content_hash: "public-media-missing".into(),
                source_path: missing_path.display().to_string(),
                file_name: "public-media-missing.wav".into(),
                byte_size: 0,
                sample_rate_hz: 48_000,
                channel_count: 1,
                duration_samples: 128,
                waveform_bin_count: 16,
            },
        ])
        .expect("public runtime media assets should reconcile");
    runtime
        .start_media_preview("asset:sha256:public-media-ready")
        .expect("public runtime media preview should start");

    let observation = RuntimeObservationReport::capture(&runtime, &recorder);
    let supervisor = RuntimeSupervisorReport::capture(&runtime, &recorder);

    assert_eq!(observation.media_pipeline_snapshot.asset_count, 2);
    assert_eq!(observation.media_pipeline_snapshot.ready_asset_count, 1);
    assert_eq!(observation.media_pipeline_snapshot.invalid_asset_count, 1);
    assert_eq!(observation.media_service_snapshot.indexed_asset_count, 2);
    assert_eq!(
        observation
            .media_service_snapshot
            .analysis_ready_asset_count,
        1
    );
    assert_eq!(
        observation
            .media_service_snapshot
            .waveform_ready_asset_count,
        1
    );
    assert_eq!(
        observation.media_service_snapshot.previewable_asset_count,
        1
    );
    assert_eq!(
        observation.media_service_snapshot.invalidated_asset_count,
        1
    );
    assert!(observation.media_service_snapshot.invalidation_active);
    assert_eq!(
        observation.media_service_snapshot.preview_state,
        RuntimeMediaPreviewState::Previewing
    );
    assert_eq!(
        observation
            .media_service_snapshot
            .previewing_asset_id
            .as_deref(),
        Some("asset:sha256:public-media-ready")
    );
    assert_eq!(
        observation
            .media_service_snapshot
            .last_invalidated_asset_id
            .as_deref(),
        Some("asset:sha256:public-media-missing")
    );
    assert!(observation
        .media_service_snapshot
        .last_invalidation_error
        .is_some());
    assert_eq!(
        supervisor.observation.media_pipeline_snapshot.asset_count,
        observation.media_pipeline_snapshot.asset_count
    );
    assert_eq!(
        supervisor.observation.media_service_snapshot.preview_state,
        RuntimeMediaPreviewState::Previewing
    );

    let _ = fs::remove_file(&ready_path);
    if let Some(path) = runtime
        .get_media_pipeline_snapshot()
        .assets
        .iter()
        .find(|asset| asset.asset_id == "asset:sha256:public-media-ready")
        .and_then(|asset| asset.cache_path.as_deref())
    {
        let _ = fs::remove_file(path);
    }
}

#[test]
fn public_runtime_reports_offline_stretch_artifact_plan_receipts_with_promotion_gate() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
    let recorder = RuntimeEventRecorder::default();
    let accepted_plan = StretchCacheIdentityInput::signal_native(
        StretchBackendTier::OfflineHighQuality,
        "sha256:public-runtime-stretch-source",
        StretchChannelLayout::new(2, 48_000),
        "projection-17",
    )
    .with_ratio_curve(vec![
        StretchRatioPoint::new(0, 1.0),
        StretchRatioPoint::new(48_000, 1.5),
    ])
    .with_pitch_curve(vec![StretchPitchPoint::new(0, 0.0)])
    .with_warp_markers(vec![StretchWarpMarker::new(0, 0)]);
    let invalid_tier = StretchCacheIdentityInput::signal_native(
        StretchBackendTier::RealtimePreview,
        "sha256:public-runtime-stretch-source",
        StretchChannelLayout::new(2, 48_000),
        "projection-17",
    );
    let promotion =
        current_synthetic_offline_high_quality_promotion_receipt("stretch-corpus:public-runtime");

    runtime
        .reconcile_offline_stretch_artifact_plans(vec![
            RuntimeOfflineStretchArtifactPlanRegistration {
                plan_id: "stretch-plan:offline-hq".into(),
                clip_id: Some("clip:offline-hq".into()),
                media_asset_id: Some("asset:offline-hq".into()),
                scope: RuntimeOfflineStretchArtifactScope::Export,
                identity_input: accepted_plan.clone(),
                promotion_receipt: promotion.clone(),
            },
            RuntimeOfflineStretchArtifactPlanRegistration {
                plan_id: "stretch-plan:preview".into(),
                clip_id: Some("clip:preview".into()),
                media_asset_id: Some("asset:preview".into()),
                scope: RuntimeOfflineStretchArtifactScope::RenderCache,
                identity_input: invalid_tier,
                promotion_receipt: promotion.clone(),
            },
        ])
        .expect("offline stretch artifact plans should reconcile");
    let materialization =
        public_offline_stretch_artifact_materialization(&accepted_plan, &promotion);
    let expected_hash = materialization.cache_identity_hash.clone();
    runtime
        .reconcile_offline_stretch_artifact_materializations(vec![materialization])
        .expect("offline stretch artifact materializations should reconcile");
    let expected_key = accepted_plan
        .identity()
        .expect("accepted identity should validate")
        .canonical_key;
    runtime
        .reconcile_offline_stretch_artifact_cache_decisions(vec![
            RuntimeOfflineStretchArtifactCacheDecisionRegistration {
                decision_id: "stretch-cache-decision:write".into(),
                plan_id: "stretch-plan:offline-hq".into(),
                clip_id: Some("clip:offline-hq".into()),
                media_asset_id: Some("asset:offline-hq".into()),
                scope: RuntimeOfflineStretchArtifactScope::RenderCache,
                kind: RuntimeOfflineStretchArtifactCacheDecisionKind::Written,
                tier: StretchBackendTier::OfflineHighQuality,
                offline_path: accepted_plan.offline_path,
                cache_identity_hash: expected_hash.clone(),
                cache_identity_key: expected_key.clone(),
                promotion_evidence_id: "stretch-corpus:public-runtime".into(),
                output_frame_count: 72_000,
                product_facing_allowed: true,
            },
            RuntimeOfflineStretchArtifactCacheDecisionRegistration {
                decision_id: "stretch-cache-decision:hit".into(),
                plan_id: "stretch-plan:offline-hq".into(),
                clip_id: Some("clip:offline-hq".into()),
                media_asset_id: Some("asset:offline-hq".into()),
                scope: RuntimeOfflineStretchArtifactScope::RenderCache,
                kind: RuntimeOfflineStretchArtifactCacheDecisionKind::Hit,
                tier: StretchBackendTier::OfflineHighQuality,
                offline_path: accepted_plan.offline_path,
                cache_identity_hash: expected_hash.clone(),
                cache_identity_key: expected_key.clone(),
                promotion_evidence_id: "stretch-corpus:public-runtime".into(),
                output_frame_count: 72_000,
                product_facing_allowed: true,
            },
            RuntimeOfflineStretchArtifactCacheDecisionRegistration {
                decision_id: "stretch-cache-decision:invalidate".into(),
                plan_id: "stretch-plan:offline-hq".into(),
                clip_id: Some("clip:offline-hq".into()),
                media_asset_id: Some("asset:offline-hq".into()),
                scope: RuntimeOfflineStretchArtifactScope::RenderCache,
                kind: RuntimeOfflineStretchArtifactCacheDecisionKind::Invalidated,
                tier: StretchBackendTier::OfflineHighQuality,
                offline_path: accepted_plan.offline_path,
                cache_identity_hash: expected_hash.clone(),
                cache_identity_key: expected_key,
                promotion_evidence_id: "stretch-corpus:public-runtime".into(),
                output_frame_count: 72_000,
                product_facing_allowed: true,
            },
        ])
        .expect("offline stretch artifact cache decisions should reconcile");

    let observation = RuntimeObservationReport::capture(&runtime, &recorder);
    let supervisor = RuntimeSupervisorReport::capture(&runtime, &recorder);
    let snapshot = &observation.offline_stretch_artifact_plan_snapshot;

    assert_eq!(snapshot.plan_count, 2);
    assert_eq!(snapshot.awaiting_corpus_evidence_count, 0);
    assert_eq!(snapshot.invalid_plan_count, 1);
    assert_eq!(snapshot.ready_plan_count, 1);
    let offline_plan = snapshot
        .plans
        .iter()
        .find(|plan| plan.plan_id == "stretch-plan:offline-hq")
        .expect("offline high-quality plan should be present");
    assert_eq!(
        offline_plan.readiness,
        RuntimeOfflineStretchArtifactReadiness::Ready
    );
    assert_eq!(offline_plan.offline_path, accepted_plan.offline_path);
    assert!(offline_plan.product_facing_allowed);
    assert_eq!(
        offline_plan.promotion_status,
        StretchPromotionStatus::Accepted
    );
    assert_eq!(
        offline_plan.promotion_evidence_id.as_deref(),
        Some("stretch-corpus:public-runtime")
    );
    assert_eq!(
        offline_plan.promotion_passed_case_count,
        promotion.passed_case_count
    );
    assert_eq!(
        offline_plan.promotion_required_case_count,
        promotion.required_case_count
    );
    assert!(offline_plan.promotion_compared_to_draft_baseline);
    assert_eq!(
        offline_plan.cache_identity_hash.as_deref(),
        Some(expected_hash.as_str())
    );
    assert_eq!(snapshot.materialized_artifact_count, 1);
    assert_eq!(snapshot.product_facing_materialized_artifact_count, 1);
    assert_eq!(snapshot.cache_decision_count, 3);
    assert_eq!(snapshot.cache_write_count, 1);
    assert_eq!(snapshot.cache_hit_count, 1);
    assert_eq!(snapshot.cache_invalidation_count, 1);
    let artifact = snapshot
        .materialized_artifacts
        .iter()
        .find(|artifact| artifact.artifact_id == "stretch-artifact:offline-hq")
        .expect("materialized offline high-quality artifact should be present");
    assert_eq!(artifact.cache_identity_hash, expected_hash);
    assert_eq!(artifact.offline_path, accepted_plan.offline_path);
    assert_eq!(artifact.output_frame_count, 72_000);
    assert!(artifact.product_facing_allowed);
    let cache_hit = snapshot
        .cache_decisions
        .iter()
        .find(|decision| decision.decision_id == "stretch-cache-decision:hit")
        .expect("cache hit decision should be present");
    assert_eq!(
        cache_hit.kind,
        RuntimeOfflineStretchArtifactCacheDecisionKind::Hit
    );
    assert_eq!(cache_hit.cache_identity_hash, expected_hash);
    assert_eq!(cache_hit.offline_path, accepted_plan.offline_path);
    assert_eq!(cache_hit.output_frame_count, artifact.output_frame_count);
    assert!(cache_hit.product_facing_allowed);
    let invalid_plan = snapshot
        .plans
        .iter()
        .find(|plan| plan.plan_id == "stretch-plan:preview")
        .expect("invalid preview plan should be present");
    assert_eq!(
        invalid_plan.readiness,
        RuntimeOfflineStretchArtifactReadiness::Invalid
    );
    assert!(invalid_plan
        .last_error
        .as_deref()
        .is_some_and(|error| error.contains("OfflineHighQuality")));
    assert_eq!(
        supervisor
            .observation
            .offline_stretch_artifact_plan_snapshot
            .materialized_artifact_count,
        snapshot.materialized_artifact_count
    );
    assert_eq!(
        supervisor
            .observation
            .offline_stretch_artifact_plan_snapshot
            .cache_decision_count,
        snapshot.cache_decision_count
    );
}
