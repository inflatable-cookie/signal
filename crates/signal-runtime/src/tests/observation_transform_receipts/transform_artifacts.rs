use super::super::*;

#[test]
fn runtime_transform_artifact_snapshot_derives_from_stretch_and_marker_analysis_baselines() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
    handshake_and_configure_with_anticipative(&mut runtime, true);

    let imported_path = temp_capture_path("transform-artifact-ready");
    write_transient_test_wav(&imported_path);
    runtime
        .reconcile_media_assets(vec![RuntimeMediaAssetRegistration {
            asset_id: "asset:sha256:transform-artifact-ready".to_string(),
            content_hash: "transform-artifact-ready".to_string(),
            source_path: imported_path.display().to_string(),
            file_name: "transform-artifact-ready.wav".to_string(),
            byte_size: fs::metadata(&imported_path).unwrap().len(),
            sample_rate_hz: 48_000,
            channel_count: 1,
            duration_samples: 48_000,
            waveform_bin_count: 32,
        }])
        .unwrap();
    runtime
        .reconcile_warp_clips(vec![RuntimeWarpClipRegistration {
            clip_id: "clip:transform-artifact-ready".to_string(),
            media_asset_id: Some("asset:sha256:transform-artifact-ready".to_string()),
            mode: RuntimeWarpMode::ElastiqueDraft,
            source_tempo_bpm: Some(120.0),
            anchor_timeline_samples: 0,
            start_samples: 0,
            duration_samples: 48_000,
        }])
        .unwrap();
    runtime
        .reconcile_clip_processing_clips(vec![RuntimeClipProcessingRegistration {
            clip_id: "clip:transform-artifact-ready".to_string(),
            media_asset_id: Some("asset:sha256:transform-artifact-ready".to_string()),
            warp_mode: RuntimeWarpMode::ElastiqueDraft,
            start_samples: 0,
            duration_samples: 48_000,
            fade_in: RuntimeClipFadeEnvelope::default(),
            fade_out: RuntimeClipFadeEnvelope::default(),
            clip_gain: RuntimeClipGainEnvelope::default(),
        }])
        .unwrap();
    runtime
        .apply_transport_projection(TransportProjection {
            playing: false,
            timeline_position_samples: 0,
            tempo_bpm: 180.0,
            loop_state: None,
        })
        .unwrap();

    let transform_artifact = runtime.get_transform_artifact_snapshot();
    assert_eq!(transform_artifact.clip_count, 1);
    assert_eq!(transform_artifact.ready_clip_count, 1);
    assert_eq!(transform_artifact.pending_media_clip_count, 0);
    assert_eq!(transform_artifact.degraded_clip_count, 0);
    assert_eq!(transform_artifact.invalidated_clip_count, 0);
    assert_eq!(transform_artifact.unsupported_clip_count, 0);
    assert_eq!(transform_artifact.cached_media_ready_clip_count, 1);
    assert_eq!(transform_artifact.reusable_clip_count, 1);
    assert_eq!(transform_artifact.requires_render_clip_count, 0);
    assert_eq!(transform_artifact.guarded_reuse_clip_count, 0);
    assert_eq!(
        transform_artifact.transform_persistence.persistence_posture,
        RuntimeTransformPersistencePosture::AssetScopedTransformPersistence
    );
    assert_eq!(
        transform_artifact
            .transform_persistence
            .retention_policy_class,
        RuntimeTransformRetentionPolicyClass::AssetLifetimeRetentionPolicy
    );
    assert_eq!(
        transform_artifact.transform_persistence.retention_authority,
        RuntimeTransformRetentionAuthority::RuntimeDefault
    );
    assert_eq!(
        transform_artifact.transform_persistence.retention_outcome,
        RuntimeTransformRetentionOutcome::PreserveAssetScopedTransforms
    );
    assert_eq!(
        transform_artifact
            .transform_persistence
            .cache_placement_posture,
        RuntimeTransformCachePlacementPosture::RuntimeCacheRootPlacement
    );
    assert_eq!(
        transform_artifact
            .transform_persistence
            .cache_placement_authority,
        RuntimeTransformCachePlacementAuthority::RuntimeDefault
    );
    assert_eq!(
        transform_artifact
            .transform_persistence
            .cache_placement_outcome,
        RuntimeTransformCachePlacementOutcome::PreserveRuntimeCacheRoot
    );
    assert_eq!(
        transform_artifact
            .transform_persistence
            .persistent_clip_count,
        1
    );
    assert_eq!(
        transform_artifact
            .transform_persistence
            .guarded_persistence_clip_count,
        0
    );
    assert_eq!(
        transform_artifact
            .transform_persistence
            .invalidated_persistence_clip_count,
        0
    );
    assert!(!transform_artifact
        .transform_persistence
        .cache_root_path
        .is_empty());
    assert_eq!(
        transform_artifact.clips[0].readiness,
        RuntimeTransformArtifactReadiness::Ready
    );
    assert_eq!(
        transform_artifact.clips[0].reuse_state,
        RuntimeTransformArtifactReuseState::Reusable
    );
    assert!(transform_artifact.clips[0].cached_media_ready);

    let observation = RuntimeObservationReport::capture(&runtime, &RuntimeEventRecorder::default());
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
        RuntimeTransformPersistencePosture::AssetScopedTransformPersistence
    );

    let rendered = runtime
        .render_clip_processing_buffer(RuntimeClipRenderRequest {
            clip_id: "clip:transform-artifact-ready".to_string(),
            timeline_start_samples: 0,
            input_stage: RuntimeClipRenderInputStage::PostWarp,
            buffer: AudioBuffer::from_interleaved(
                SampleRate(48_000),
                ChannelLayout::Mono,
                vec![0.5; 8],
            ),
        })
        .unwrap();
    assert_eq!(
        rendered.transform_artifact_snapshot.readiness,
        RuntimeTransformArtifactReadiness::Ready
    );
    assert_eq!(
        rendered.transform_artifact_snapshot.reuse_state,
        RuntimeTransformArtifactReuseState::Reusable
    );
    assert!(rendered.transform_artifact_snapshot.cached_media_ready);

    let preview = RuntimeOfflineRenderContractPreview::from_runtime_state(
        &RuntimeOfflineRenderRequest {
            request_id: "render:transform-artifact-preview".into(),
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
    .expect("build transform artifact offline render preview");
    assert_eq!(preview.transform_artifact_snapshot.clip_count, 1);
    assert_eq!(preview.transform_artifact_snapshot.ready_clip_count, 1);
    assert_eq!(preview.transform_artifact_snapshot.reusable_clip_count, 1);
    assert_eq!(
        preview
            .transform_artifact_snapshot
            .transform_persistence
            .retention_outcome,
        RuntimeTransformRetentionOutcome::PreserveAssetScopedTransforms
    );

    let _ = fs::remove_file(imported_path);
    if let Some(path) = runtime
        .get_media_pipeline_snapshot()
        .assets
        .first()
        .and_then(|asset| asset.cache_path.as_deref())
    {
        let _ = fs::remove_file(path);
    }
}
