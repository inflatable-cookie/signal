#[path = "support/public_host_edge_media.rs"]
mod public_host_edge_media;
#[path = "support/public_host_edge_stretch.rs"]
mod public_host_edge_stretch;

use std::fs;

use public_host_edge_media::{public_local_media_fixture_path, write_public_test_wav};
use public_host_edge_stretch::{
    accepted_stretch_policy_request, cache_consumption_options, cache_consumption_spec,
    host_stretch_identity_input, host_stretch_source, rejected_stretch_policy_request,
    stretch_build_request, CACHE_CONSUMPTION_STAGE_ID,
};
use signal_host_local::LocalRuntimeHost;
use signal_render_plane::{
    build_offline_stretch_artifact_cache_handoff_with_synthetic_policy,
    build_offline_stretch_artifact_render_source_with_synthetic_policy,
    plan_offline_stretch_artifact_with_synthetic_policy, render_plan_to_pcm,
    OfflineStretchArtifactMaterializeError, OfflineStretchArtifactReadiness,
    OfflineStretchArtifactRenderCacheBridge,
    OfflineStretchArtifactScope as RenderOfflineStretchArtifactScope, RenderSource,
};
use signal_runtime::{
    RuntimeConfig, RuntimeConfigRequest, RuntimeLifecycleApi, RuntimeMediaPreviewState,
    RuntimeObservationApi, RuntimeOfflineStretchArtifactCacheDecisionKind,
    RuntimeOfflineStretchArtifactPlanRegistration, RuntimeOfflineStretchArtifactReadiness,
    RuntimeOfflineStretchArtifactScope, RuntimeSupervisorApi, SignalRuntime,
    StretchPromotionStatus,
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
    let identity_input = host_stretch_identity_input(
        "sha256:host-local-stretch-source",
        "projection-host-local-3",
    );
    let expected_identity = identity_input.identity().expect("identity should validate");
    let source = host_stretch_source(0.25, 480);
    let artifact_source =
        build_offline_stretch_artifact_render_source_with_synthetic_policy(stretch_build_request(
            accepted_stretch_policy_request(
                RenderOfflineStretchArtifactScope::Freeze,
                &identity_input,
                "stretch-corpus:host-local",
            ),
            &source,
        ))
        .expect("host-local freeze artifact should produce a policy-gated render source");
    let RenderSource::Samples(buffer) = &artifact_source.source else {
        panic!("host-local artifact source should be RenderSource::Samples");
    };
    assert_eq!(
        buffer.frame_count(),
        artifact_source.artifact.output_frame_count
    );
    let artifact = &artifact_source.artifact;
    let expected_passed_case_count = artifact.plan.promotion_receipt.passed_case_count;
    let expected_required_case_count = artifact.plan.promotion_receipt.required_case_count;
    let mut cache_bridge = OfflineStretchArtifactRenderCacheBridge::new();
    let cache_decision = cache_bridge
        .resolve_with_synthetic_policy(stretch_build_request(
            accepted_stretch_policy_request(
                RenderOfflineStretchArtifactScope::RenderCache,
                &identity_input,
                "stretch-corpus:host-local-cache",
            ),
            &source,
        ))
        .expect("host-local render-cache decision should resolve");
    let cache_render_source = cache_decision.handoff.source.clone();
    let cache_output_frames = cache_decision.handoff.receipt.output_frame_count as u64;
    let rendered = render_plan_to_pcm(
        &cache_consumption_spec(cache_render_source, cache_output_frames),
        &cache_consumption_options(cache_output_frames),
    )
    .expect("host-local cache decision source should render through export/freeze path");
    assert_eq!(rendered.master.len(), cache_output_frames as usize * 2);
    assert_eq!(rendered.stems.len(), 1);
    assert_eq!(rendered.stems[0].0, CACHE_CONSUMPTION_STAGE_ID);
    assert_eq!(rendered.stems[0].1.len(), cache_output_frames as usize * 2);

    let mut host = LocalRuntimeHost::new(runtime);
    host.reconcile_offline_stretch_artifact_plans(vec![
        RuntimeOfflineStretchArtifactPlanRegistration {
            plan_id: "host-stretch-plan:offline-hq".into(),
            clip_id: Some("clip:host-offline-hq".into()),
            media_asset_id: Some("asset:host-offline-hq".into()),
            scope: RuntimeOfflineStretchArtifactScope::Freeze,
            identity_input: identity_input.clone(),
            promotion_receipt: artifact.plan.promotion_receipt.clone(),
        },
    ])
    .expect("host should forward offline stretch artifact plans");
    host.record_offline_stretch_artifact_materialization_receipt(
        "host-stretch-artifact:offline-hq",
        "host-stretch-plan:offline-hq",
        Some("clip:host-offline-hq".into()),
        Some("asset:host-offline-hq".into()),
        artifact.receipt.clone(),
    )
    .expect("host should record render-plane stretch artifact materialization");
    host.record_offline_stretch_artifact_cache_decision_receipt(
        "host-stretch-cache-decision:write",
        "host-stretch-plan:offline-hq",
        Some("clip:host-offline-hq".into()),
        Some("asset:host-offline-hq".into()),
        cache_decision,
    )
    .expect("host should record render-plane stretch artifact cache decision");

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
    assert_eq!(plan.promotion_passed_case_count, expected_passed_case_count);
    assert_eq!(
        plan.promotion_required_case_count,
        expected_required_case_count
    );
    assert!(plan.promotion_compared_to_draft_baseline);
    assert!(plan.product_facing_allowed);
    assert_eq!(
        plan.cache_identity_hash.as_deref(),
        Some(expected_identity.stable_hash.as_str())
    );
    assert_eq!(snapshot.materialized_artifact_count, 1);
    assert_eq!(snapshot.product_facing_materialized_artifact_count, 1);
    let artifact = &snapshot.materialized_artifacts[0];
    assert_eq!(artifact.artifact_id, "host-stretch-artifact:offline-hq");
    assert_eq!(artifact.plan_id, "host-stretch-plan:offline-hq");
    assert_eq!(artifact.cache_identity_hash, expected_identity.stable_hash);
    assert_eq!(artifact.cache_identity_key, expected_identity.canonical_key);
    assert_eq!(artifact.promotion_evidence_id, "stretch-corpus:host-local");
    assert_eq!(artifact.input_frame_count, 480);
    assert_eq!(artifact.output_frame_count, 600);
    assert_eq!(
        artifact.chunk_count,
        artifact_source.artifact.receipt.chunk_count
    );
    assert_eq!(
        artifact.max_chunk_source_frames,
        artifact_source.artifact.receipt.max_chunk_source_frames
    );
    assert_eq!(
        artifact.chunk_overlap_frames,
        artifact_source.artifact.receipt.chunk_overlap_frames
    );
    assert_eq!(
        artifact.max_chunk_render_source_frames,
        artifact_source
            .artifact
            .receipt
            .max_chunk_render_source_frames
    );
    assert!(artifact.product_facing_allowed);
    assert_eq!(snapshot.cache_decision_count, 1);
    assert_eq!(snapshot.cache_write_count, 1);
    let cache_decision = &snapshot.cache_decisions[0];
    assert_eq!(
        cache_decision.kind,
        RuntimeOfflineStretchArtifactCacheDecisionKind::Written
    );
    assert_eq!(
        cache_decision.decision_id,
        "host-stretch-cache-decision:write"
    );
    assert_eq!(
        cache_decision.cache_identity_hash,
        expected_identity.stable_hash
    );
    assert_eq!(cache_decision.output_frame_count, 600);
    assert_eq!(cache_decision.chunk_count, 1);
    assert_eq!(
        cache_decision.max_chunk_source_frames,
        artifact_source.artifact.receipt.max_chunk_source_frames
    );
    assert_eq!(
        cache_decision.chunk_overlap_frames,
        artifact_source.artifact.receipt.chunk_overlap_frames
    );
    assert_eq!(cache_decision.max_chunk_render_source_frames, 480);
    assert!(cache_decision.product_facing_allowed);
}

#[test]
fn local_shared_host_edge_blocks_rejected_offline_stretch_cache_artifacts() {
    let runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
    let identity_input = host_stretch_identity_input(
        "sha256:host-local-rejected-stretch-source",
        "projection-host-local-rejected",
    );
    let source = host_stretch_source(0.125, 480);
    let policy_request = rejected_stretch_policy_request(
        RenderOfflineStretchArtifactScope::RenderCache,
        &identity_input,
        "stretch-corpus:host-local-rejected-cache",
    );
    let rejected_plan = plan_offline_stretch_artifact_with_synthetic_policy(policy_request)
        .expect("rejected policy should still produce a non-ready plan");
    assert_eq!(
        rejected_plan.readiness,
        OfflineStretchArtifactReadiness::AwaitingCorpusEvidence
    );
    let build_request = stretch_build_request(policy_request, &source);

    assert_eq!(
        build_offline_stretch_artifact_cache_handoff_with_synthetic_policy(build_request),
        Err(OfflineStretchArtifactMaterializeError::NotReady(
            OfflineStretchArtifactReadiness::AwaitingCorpusEvidence
        ))
    );
    assert_eq!(
        build_offline_stretch_artifact_render_source_with_synthetic_policy(build_request),
        Err(OfflineStretchArtifactMaterializeError::NotReady(
            OfflineStretchArtifactReadiness::AwaitingCorpusEvidence
        ))
    );
    let mut cache_bridge = OfflineStretchArtifactRenderCacheBridge::new();
    assert_eq!(
        cache_bridge.resolve_with_synthetic_policy(build_request),
        Err(OfflineStretchArtifactMaterializeError::NotReady(
            OfflineStretchArtifactReadiness::AwaitingCorpusEvidence
        ))
    );
    assert!(cache_bridge.is_empty());

    let mut host = LocalRuntimeHost::new(runtime);
    host.reconcile_offline_stretch_artifact_plans(vec![
        RuntimeOfflineStretchArtifactPlanRegistration {
            plan_id: "host-stretch-plan:rejected-cache".into(),
            clip_id: Some("clip:host-rejected-cache".into()),
            media_asset_id: Some("asset:host-rejected-cache".into()),
            scope: RuntimeOfflineStretchArtifactScope::RenderCache,
            identity_input,
            promotion_receipt: rejected_plan.promotion_receipt.clone(),
        },
    ])
    .expect("host should forward rejected offline stretch artifact plan");

    let report = host.supervisor_report();
    let snapshot = &report.observation.offline_stretch_artifact_plan_snapshot;
    assert_eq!(snapshot.plan_count, 1);
    assert_eq!(snapshot.ready_plan_count, 0);
    assert_eq!(snapshot.awaiting_corpus_evidence_count, 1);
    assert_eq!(snapshot.materialized_artifact_count, 0);
    assert_eq!(snapshot.cache_decision_count, 0);
    let plan = &snapshot.plans[0];
    assert_eq!(plan.plan_id, "host-stretch-plan:rejected-cache");
    assert_eq!(
        plan.readiness,
        RuntimeOfflineStretchArtifactReadiness::AwaitingCorpusEvidence
    );
    assert!(!plan.product_facing_allowed);
}
