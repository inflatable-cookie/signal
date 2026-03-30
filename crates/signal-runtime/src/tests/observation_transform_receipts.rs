use super::*;

#[test]
fn runtime_observation_clip_render_and_offline_render_preview_surface_stretch_engine_receipts() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
    handshake_and_configure_with_anticipative(&mut runtime, true);

    let imported_path = temp_capture_path("stretch-engine-ready");
    write_test_wav(&imported_path);
    runtime
        .reconcile_media_assets(vec![RuntimeMediaAssetRegistration {
            asset_id: "asset:sha256:stretch-engine-ready".to_string(),
            content_hash: "stretch-engine-ready".to_string(),
            source_path: imported_path.display().to_string(),
            file_name: "stretch-engine-ready.wav".to_string(),
            byte_size: fs::metadata(&imported_path).unwrap().len(),
            sample_rate_hz: 48_000,
            channel_count: 1,
            duration_samples: 128,
            waveform_bin_count: 8,
        }])
        .unwrap();
    runtime
        .reconcile_warp_clips(vec![RuntimeWarpClipRegistration {
            clip_id: "clip:stretch-engine-ready".to_string(),
            media_asset_id: Some("asset:sha256:stretch-engine-ready".to_string()),
            mode: RuntimeWarpMode::ElastiqueDraft,
            source_tempo_bpm: Some(120.0),
            anchor_timeline_samples: 0,
            start_samples: 0,
            duration_samples: 48_000,
        }])
        .unwrap();
    runtime
        .reconcile_clip_processing_clips(vec![RuntimeClipProcessingRegistration {
            clip_id: "clip:stretch-engine-ready".to_string(),
            media_asset_id: Some("asset:sha256:stretch-engine-ready".to_string()),
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

    let observation = RuntimeObservationReport::capture(&runtime, &RuntimeEventRecorder::default());
    assert_eq!(observation.stretch_engine_snapshot.clip_count, 1);
    assert_eq!(observation.stretch_engine_snapshot.ready_clip_count, 1);
    assert_eq!(
        observation.stretch_engine_snapshot.sample_domain_clip_count,
        1
    );
    assert_eq!(observation.stretch_engine_snapshot.fallback_clip_count, 0);
    assert_eq!(
        observation.stretch_engine_snapshot.clips[0].engine_class,
        RuntimeStretchEngineClass::SampleDomain
    );
    assert_eq!(
        observation.stretch_engine_snapshot.clips[0].readiness,
        RuntimeStretchReadiness::Ready
    );
    assert_eq!(
        observation.stretch_engine_snapshot.clips[0].fallback_kind,
        RuntimeStretchFallbackKind::None
    );
    assert!(observation
        .render_compact()
        .contains("stretch_clips=1/1/1/0/0/0/0/0"));
    assert!(observation
        .render_json()
        .contains("\"stretch_engine_snapshot\":{\"clip_count\":1"));

    let rendered = runtime
        .render_clip_processing_buffer(RuntimeClipRenderRequest {
            clip_id: "clip:stretch-engine-ready".to_string(),
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
        rendered.stretch_engine_snapshot.engine_class,
        RuntimeStretchEngineClass::SampleDomain
    );
    assert_eq!(
        rendered.stretch_engine_snapshot.readiness,
        RuntimeStretchReadiness::Ready
    );
    assert_eq!(
        rendered.stretch_engine_snapshot.fallback_kind,
        RuntimeStretchFallbackKind::None
    );
    assert!(rendered.summary.contains("stretch=SampleDomain/Ready/None"));

    let preview = RuntimeOfflineRenderContractPreview::from_runtime_state(
        &RuntimeOfflineRenderRequest {
            request_id: "render:stretch-engine-preview".into(),
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
    .expect("build stretch engine offline render preview");
    assert_eq!(preview.stretch_engine_snapshot.clip_count, 1);
    assert_eq!(preview.stretch_engine_snapshot.ready_clip_count, 1);
    assert_eq!(preview.stretch_engine_snapshot.sample_domain_clip_count, 1);
    assert_eq!(preview.stretch_engine_snapshot.fallback_clip_count, 0);
    assert_eq!(
        preview.stretch_engine_snapshot.clips[0].engine_class,
        RuntimeStretchEngineClass::SampleDomain
    );
    assert_eq!(
        preview.stretch_engine_snapshot.clips[0].readiness,
        RuntimeStretchReadiness::Ready
    );
    assert!(preview.summary.contains("stretch=1/fallback=0"));

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

#[test]
fn runtime_marker_analysis_snapshot_derives_from_stretch_and_media_baselines() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
    handshake_and_configure_with_anticipative(&mut runtime, true);

    let imported_path = temp_capture_path("marker-analysis-ready");
    write_transient_test_wav(&imported_path);
    runtime
        .reconcile_media_assets(vec![RuntimeMediaAssetRegistration {
            asset_id: "asset:sha256:marker-analysis-ready".to_string(),
            content_hash: "marker-analysis-ready".to_string(),
            source_path: imported_path.display().to_string(),
            file_name: "marker-analysis-ready.wav".to_string(),
            byte_size: fs::metadata(&imported_path).unwrap().len(),
            sample_rate_hz: 48_000,
            channel_count: 1,
            duration_samples: 48_000,
            waveform_bin_count: 32,
        }])
        .unwrap();
    runtime
        .reconcile_warp_clips(vec![RuntimeWarpClipRegistration {
            clip_id: "clip:marker-analysis-ready".to_string(),
            media_asset_id: Some("asset:sha256:marker-analysis-ready".to_string()),
            mode: RuntimeWarpMode::ElastiqueDraft,
            source_tempo_bpm: Some(120.0),
            anchor_timeline_samples: 0,
            start_samples: 0,
            duration_samples: 48_000,
        }])
        .unwrap();
    runtime
        .reconcile_clip_processing_clips(vec![RuntimeClipProcessingRegistration {
            clip_id: "clip:marker-analysis-ready".to_string(),
            media_asset_id: Some("asset:sha256:marker-analysis-ready".to_string()),
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

    let marker_analysis = runtime.get_marker_analysis_snapshot();
    assert_eq!(marker_analysis.clip_count, 1);
    assert_eq!(marker_analysis.ready_clip_count, 1);
    assert_eq!(marker_analysis.pending_media_clip_count, 0);
    assert_eq!(marker_analysis.degraded_clip_count, 0);
    assert_eq!(marker_analysis.invalidated_clip_count, 0);
    assert_eq!(marker_analysis.unsupported_clip_count, 0);
    assert_eq!(marker_analysis.tempo_assist_ready_clip_count, 1);
    assert!(marker_analysis.warp_marker_count > 0);
    assert!(marker_analysis.transient_anchor_count > 0);
    assert_eq!(
        marker_analysis.clips[0].readiness,
        RuntimeMarkerAnalysisReadiness::Ready
    );
    assert_eq!(
        marker_analysis.clips[0].tempo_assist_posture,
        RuntimeTempoAssistPosture::Ready
    );
    assert_eq!(
        marker_analysis.clips[0].tempo_assist_hint_source,
        RuntimeTempoAssistHintSource::SourceTempo
    );
    assert_eq!(marker_analysis.clips[0].tempo_assist_hint_bpm, Some(120.0));

    let observation = RuntimeObservationReport::capture(&runtime, &RuntimeEventRecorder::default());
    assert_eq!(observation.marker_analysis_snapshot.clip_count, 1);
    assert_eq!(observation.marker_analysis_snapshot.ready_clip_count, 1);
    assert_eq!(
        observation
            .marker_analysis_snapshot
            .tempo_assist_ready_clip_count,
        1
    );
    assert!(observation
        .render_compact()
        .contains("marker_analysis_clips=1/1/0/0/0"));
    assert!(observation
        .render_json()
        .contains("\"marker_analysis_snapshot\":{\"clip_count\":1"));

    let supervisor = RuntimeSupervisorReport::capture(&runtime, &RuntimeEventRecorder::default());
    let multiline = supervisor.render_multiline();
    assert!(multiline.contains("marker_analysis_clip_count=1"));
    assert!(multiline.contains("marker_analysis_tempo_assist_ready_clip_count=1"));
    assert!(supervisor
        .render_json()
        .contains("\"marker_analysis_snapshot\":{\"clip_count\":1"));

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
    assert!(observation
        .render_compact()
        .contains("transform_artifacts=1/1/0/0/0"));
    assert!(observation
        .render_json()
        .contains("\"transform_artifact_snapshot\":{\"clip_count\":1"));
    assert!(observation.render_json().contains(
        "\"transform_persistence\":{\"persistence_posture\":\"AssetScopedTransformPersistence\""
    ));

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
    assert!(rendered
        .summary
        .contains("transform=Ready/Reusable/cached_media=true"));

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
    assert!(preview.summary.contains("transform_artifacts=1/reusable=1"));

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
