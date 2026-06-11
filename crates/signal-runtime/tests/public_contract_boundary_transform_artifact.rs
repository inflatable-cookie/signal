use std::fs;

#[path = "support/public_contract_boundary_media.rs"]
mod public_contract_boundary_media_support;

use public_contract_boundary_media_support::{
    public_media_fixture_path, write_public_transient_test_wav,
};
use signal_primitives::{AudioBuffer, ChannelLayout, SampleRate};
use signal_runtime::{
    HandshakeRequest, RuntimeConfig, RuntimeConfigRequest, RuntimeEventRecorder,
    RuntimeLifecycleApi, RuntimeObservationApi, RuntimeObservationReport,
    RuntimeOfflineRenderContractPreview, RuntimeOfflineRenderRequest, RuntimeProjectionApi,
    RuntimeSupervisorReport, SignalRuntime,
};

#[test]
fn public_runtime_transform_artifact_boundary_reports_runtime_owned_artifact_truth() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
    runtime
        .handshake(HandshakeRequest {
            client_version: "public-runtime-transform-artifact-boundary".into(),
            anticipative_preferred: true,
            max_sample_rate_hint: Some(96_000),
        })
        .expect("public runtime transform-artifact handshake should succeed");
    runtime
        .configure(RuntimeConfigRequest::new(48_000, 256))
        .expect("public runtime transform-artifact configure should succeed");

    let ready_path = public_media_fixture_path("transform-artifact-ready");
    write_public_transient_test_wav(&ready_path);
    runtime
        .reconcile_media_assets(vec![signal_runtime::RuntimeMediaAssetRegistration {
            asset_id: "asset:sha256:public-transform-artifact-ready".into(),
            content_hash: "public-transform-artifact-ready".into(),
            source_path: ready_path.display().to_string(),
            file_name: "public-transform-artifact-ready.wav".into(),
            byte_size: fs::metadata(&ready_path).unwrap().len(),
            sample_rate_hz: 48_000,
            channel_count: 1,
            duration_samples: 48_000,
            waveform_bin_count: 8,
        }])
        .expect("public transform-artifact media asset should reconcile");
    runtime
        .reconcile_warp_clips(vec![signal_runtime::RuntimeWarpClipRegistration {
            clip_id: "clip:public-transform-artifact".into(),
            media_asset_id: Some("asset:sha256:public-transform-artifact-ready".into()),
            mode: signal_runtime::RuntimeWarpMode::ElastiqueDraft,
            source_tempo_bpm: Some(120.0),
            anchor_timeline_samples: 0,
            start_samples: 0,
            duration_samples: 48_000,
        }])
        .expect("public transform-artifact warp clip should reconcile");
    runtime
        .reconcile_clip_processing_clips(vec![signal_runtime::RuntimeClipProcessingRegistration {
            clip_id: "clip:public-transform-artifact".into(),
            media_asset_id: Some("asset:sha256:public-transform-artifact-ready".into()),
            warp_mode: signal_runtime::RuntimeWarpMode::ElastiqueDraft,
            start_samples: 0,
            duration_samples: 48_000,
            fade_in: signal_runtime::RuntimeClipFadeEnvelope::default(),
            fade_out: signal_runtime::RuntimeClipFadeEnvelope::default(),
            clip_gain: signal_runtime::RuntimeClipGainEnvelope::default(),
        }])
        .expect("public transform-artifact clip-processing clip should reconcile");
    runtime
        .apply_transport_projection(signal_runtime::TransportProjection {
            playing: false,
            timeline_position_samples: 0,
            tempo_bpm: 180.0,
            loop_state: None,
        })
        .expect("public transform-artifact transport projection should apply");

    let recorder = RuntimeEventRecorder::default();
    let observation = RuntimeObservationReport::capture(&runtime, &recorder);
    assert_eq!(observation.transform_artifact_snapshot.clip_count, 1);
    assert_eq!(observation.transform_artifact_snapshot.ready_clip_count, 1);
    assert_eq!(
        observation.transform_artifact_snapshot.reusable_clip_count,
        1
    );
    assert_eq!(
        observation
            .transform_artifact_snapshot
            .transform_persistence
            .persistence_posture,
        signal_runtime::RuntimeTransformPersistencePosture::AssetScopedTransformPersistence
    );
    assert_eq!(
        observation
            .transform_artifact_snapshot
            .transform_persistence
            .retention_outcome,
        signal_runtime::RuntimeTransformRetentionOutcome::PreserveAssetScopedTransforms
    );
    assert_eq!(
        observation
            .transform_artifact_snapshot
            .transform_persistence
            .cache_placement_outcome,
        signal_runtime::RuntimeTransformCachePlacementOutcome::PreserveRuntimeCacheRoot
    );
    assert_eq!(
        observation.transform_artifact_snapshot.clips[0].readiness,
        signal_runtime::RuntimeTransformArtifactReadiness::Ready
    );
    assert_eq!(
        observation.transform_artifact_snapshot.clips[0].reuse_state,
        signal_runtime::RuntimeTransformArtifactReuseState::Reusable
    );
    assert!(observation.transform_artifact_snapshot.clips[0].cached_media_ready);

    let rendered = runtime
        .render_clip_processing_buffer(signal_runtime::RuntimeClipRenderRequest {
            clip_id: "clip:public-transform-artifact".into(),
            timeline_start_samples: 0,
            input_stage: signal_runtime::RuntimeClipRenderInputStage::PostWarp,
            buffer: AudioBuffer::from_interleaved(
                SampleRate(48_000),
                ChannelLayout::Mono,
                vec![0.25; 8],
            ),
        })
        .expect("public transform-artifact clip render should succeed");
    assert_eq!(
        rendered.transform_artifact_snapshot.readiness,
        signal_runtime::RuntimeTransformArtifactReadiness::Ready
    );
    assert_eq!(
        rendered.transform_artifact_snapshot.reuse_state,
        signal_runtime::RuntimeTransformArtifactReuseState::Reusable
    );
    assert!(rendered.transform_artifact_snapshot.cached_media_ready);

    let preview = RuntimeOfflineRenderContractPreview::from_runtime_state(
        &RuntimeOfflineRenderRequest {
            request_id: "render:public-transform-artifact-preview".into(),
            timeline_start_samples: 0,
            duration_samples: 24_000,
            export_sample_rate_hz: 48_000,
            include_main_mix: true,
            artifact_root_path: None,
            stem_targets: Vec::new(),
            freeze_artifacts: Vec::new(),
        },
        &runtime.get_execution_topology_summary(),
        &runtime.get_clip_processing_pipeline_snapshot(),
        &runtime.get_media_pipeline_snapshot(),
        &runtime.get_tempo_map_snapshot(),
        &runtime.get_marker_analysis_snapshot(),
        &runtime.get_plugin_recall_handoff_snapshot(),
    )
    .expect("public transform-artifact preview should build");
    assert_eq!(preview.transform_artifact_snapshot.clip_count, 1);
    assert_eq!(preview.transform_artifact_snapshot.ready_clip_count, 1);
    assert_eq!(preview.transform_artifact_snapshot.reusable_clip_count, 1);
    assert_eq!(
        preview
            .transform_artifact_snapshot
            .transform_persistence
            .retention_outcome,
        signal_runtime::RuntimeTransformRetentionOutcome::PreserveAssetScopedTransforms
    );
    assert_eq!(
        preview.transform_artifact_snapshot.clips[0].reuse_state,
        signal_runtime::RuntimeTransformArtifactReuseState::Reusable
    );

    let _supervisor = RuntimeSupervisorReport::capture(&runtime, &recorder);

    let _ = fs::remove_file(&ready_path);
    if let Some(path) = runtime
        .get_media_pipeline_snapshot()
        .assets
        .first()
        .and_then(|asset| asset.cache_path.as_deref())
    {
        let _ = fs::remove_file(path);
    }
}
