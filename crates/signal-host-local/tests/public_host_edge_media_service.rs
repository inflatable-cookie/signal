#[path = "support/public_host_edge_media.rs"]
mod public_host_edge_media;

use std::fs;

use public_host_edge_media::{public_local_media_fixture_path, write_public_test_wav};
use signal_host_local::LocalRuntimeHost;
use signal_runtime::{
    RuntimeConfig, RuntimeConfigRequest, RuntimeLifecycleApi, RuntimeMediaPreviewState,
    RuntimeObservationApi, RuntimeOfflineStretchArtifactPlanRegistration,
    RuntimeOfflineStretchArtifactReadiness, RuntimeOfflineStretchArtifactScope,
    RuntimeSupervisorApi, SignalRuntime, StretchBackendTier, StretchCacheIdentityInput,
    StretchChannelLayout, StretchPitchPoint, StretchPromotionReceipt, StretchPromotionStatus,
    StretchRatioPoint, StretchWarpMarker,
};

#[test]
fn local_shared_host_edge_exports_runtime_media_service_truth() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
    runtime
        .handshake(signal_runtime::HandshakeRequest {
            client_version: "public-host-local-media-service".into(),
            anticipative_preferred: true,
            max_sample_rate_hint: Some(96_000),
        })
        .expect("local host-edge media-service handshake should succeed");
    runtime
        .configure(RuntimeConfigRequest::new(48_000, 256))
        .expect("local host-edge media-service configure should succeed");

    let ready_path = public_local_media_fixture_path("ready");
    let missing_path = public_local_media_fixture_path("missing");
    write_public_test_wav(&ready_path);

    runtime
        .reconcile_media_assets(vec![
            signal_runtime::RuntimeMediaAssetRegistration {
                asset_id: "asset:sha256:host-local-media-ready".into(),
                content_hash: "host-local-media-ready".into(),
                source_path: ready_path.display().to_string(),
                file_name: "host-local-media-ready.wav".into(),
                byte_size: fs::metadata(&ready_path)
                    .expect("public local media fixture should exist")
                    .len(),
                sample_rate_hz: 48_000,
                channel_count: 1,
                duration_samples: 128,
                waveform_bin_count: 16,
            },
            signal_runtime::RuntimeMediaAssetRegistration {
                asset_id: "asset:sha256:host-local-media-missing".into(),
                content_hash: "host-local-media-missing".into(),
                source_path: missing_path.display().to_string(),
                file_name: "host-local-media-missing.wav".into(),
                byte_size: 0,
                sample_rate_hz: 48_000,
                channel_count: 1,
                duration_samples: 128,
                waveform_bin_count: 16,
            },
        ])
        .expect("local host-edge media assets should reconcile");
    runtime
        .start_media_preview("asset:sha256:host-local-media-ready")
        .expect("local host-edge media preview should start");

    let host = LocalRuntimeHost::new(runtime);
    let report = host.supervisor_report();

    assert_eq!(report.observation.media_pipeline_snapshot.asset_count, 2);
    assert_eq!(
        report.observation.media_pipeline_snapshot.ready_asset_count,
        1
    );
    assert_eq!(
        report
            .observation
            .media_pipeline_snapshot
            .invalid_asset_count,
        1
    );
    assert_eq!(
        report
            .observation
            .media_service_snapshot
            .indexed_asset_count,
        2
    );
    assert_eq!(
        report
            .observation
            .media_service_snapshot
            .waveform_ready_asset_count,
        1
    );
    assert_eq!(
        report.observation.media_service_snapshot.preview_state,
        RuntimeMediaPreviewState::Previewing
    );
    assert_eq!(
        report
            .observation
            .media_service_snapshot
            .previewing_asset_id
            .as_deref(),
        Some("asset:sha256:host-local-media-ready")
    );
    assert_eq!(
        report
            .observation
            .media_service_snapshot
            .last_invalidated_asset_id
            .as_deref(),
        Some("asset:sha256:host-local-media-missing")
    );
    assert!(
        report
            .observation
            .media_service_snapshot
            .invalidation_active
    );

    let _ = fs::remove_file(&ready_path);
    if let Some(path) = host
        .runtime()
        .get_media_pipeline_snapshot()
        .assets
        .iter()
        .find(|asset| asset.asset_id == "asset:sha256:host-local-media-ready")
        .and_then(|asset| asset.cache_path.as_deref())
    {
        let _ = fs::remove_file(path);
    }
}

#[test]
fn local_shared_host_edge_exports_offline_stretch_artifact_receipts() {
    let runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
    let identity_input = StretchCacheIdentityInput::signal_native(
        StretchBackendTier::OfflineHighQuality,
        "sha256:host-local-stretch-source",
        StretchChannelLayout::new(2, 48_000),
        "projection-host-local-3",
    )
    .with_ratio_curve(vec![
        StretchRatioPoint::new(0, 1.0),
        StretchRatioPoint::new(96_000, 0.75),
    ])
    .with_pitch_curve(vec![StretchPitchPoint::new(0, 2.0)])
    .with_warp_markers(vec![StretchWarpMarker::new(0, 0)]);
    let expected_hash = identity_input
        .identity()
        .expect("identity should validate")
        .stable_hash;

    let mut host = LocalRuntimeHost::new(runtime);
    host.reconcile_offline_stretch_artifact_plans(vec![
        RuntimeOfflineStretchArtifactPlanRegistration {
            plan_id: "host-stretch-plan:offline-hq".into(),
            clip_id: Some("clip:host-offline-hq".into()),
            media_asset_id: Some("asset:host-offline-hq".into()),
            scope: RuntimeOfflineStretchArtifactScope::Freeze,
            identity_input,
            promotion_receipt: StretchPromotionReceipt::accepted_offline_high_quality(
                "stretch-corpus:host-local",
                8,
                8,
            ),
        },
    ])
    .expect("host should forward offline stretch artifact plans");

    let report = host.supervisor_report();
    let snapshot = &report.observation.offline_stretch_artifact_plan_snapshot;
    assert_eq!(snapshot.plan_count, 1);
    assert_eq!(snapshot.awaiting_corpus_evidence_count, 0);
    assert_eq!(snapshot.ready_plan_count, 1);
    let plan = &snapshot.plans[0];
    assert_eq!(plan.plan_id, "host-stretch-plan:offline-hq");
    assert_eq!(
        plan.readiness,
        RuntimeOfflineStretchArtifactReadiness::Ready
    );
    assert_eq!(plan.promotion_status, StretchPromotionStatus::Accepted);
    assert_eq!(
        plan.promotion_evidence_id.as_deref(),
        Some("stretch-corpus:host-local")
    );
    assert_eq!(plan.promotion_passed_case_count, 8);
    assert_eq!(plan.promotion_required_case_count, 8);
    assert!(plan.promotion_compared_to_draft_baseline);
    assert!(plan.product_facing_allowed);
    assert_eq!(
        plan.cache_identity_hash.as_deref(),
        Some(expected_hash.as_str())
    );
}
