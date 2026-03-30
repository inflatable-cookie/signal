use super::*;

#[test]
fn runtime_preview_transform_snapshot_derives_from_stretch_and_artifact_baselines() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
    handshake_and_configure_with_anticipative(&mut runtime, true);

    let imported_path = temp_capture_path("preview-transform-ready");
    write_transient_test_wav(&imported_path);
    runtime
        .reconcile_media_assets(vec![RuntimeMediaAssetRegistration {
            asset_id: "asset:sha256:preview-transform-ready".to_string(),
            content_hash: "preview-transform-ready".to_string(),
            source_path: imported_path.display().to_string(),
            file_name: "preview-transform-ready.wav".to_string(),
            byte_size: fs::metadata(&imported_path).unwrap().len(),
            sample_rate_hz: 48_000,
            channel_count: 1,
            duration_samples: 48_000,
            waveform_bin_count: 32,
        }])
        .unwrap();
    runtime
        .reconcile_warp_clips(vec![RuntimeWarpClipRegistration {
            clip_id: "clip:preview-transform-ready".to_string(),
            media_asset_id: Some("asset:sha256:preview-transform-ready".to_string()),
            mode: RuntimeWarpMode::ElastiqueDraft,
            source_tempo_bpm: Some(120.0),
            anchor_timeline_samples: 0,
            start_samples: 0,
            duration_samples: 48_000,
        }])
        .unwrap();
    runtime
        .reconcile_clip_processing_clips(vec![RuntimeClipProcessingRegistration {
            clip_id: "clip:preview-transform-ready".to_string(),
            media_asset_id: Some("asset:sha256:preview-transform-ready".to_string()),
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
    runtime
        .start_media_preview("asset:sha256:preview-transform-ready")
        .expect("preview transform media preview should start");

    let preview_transform = runtime.get_preview_transform_snapshot();
    assert_eq!(preview_transform.clip_count, 1);
    assert_eq!(preview_transform.active_audition_clip_count, 1);
    assert_eq!(preview_transform.scrub_supported_clip_count, 1);
    assert_eq!(preview_transform.ready_clip_count, 1);
    assert_eq!(preview_transform.pending_clip_count, 0);
    assert_eq!(preview_transform.degraded_clip_count, 0);
    assert_eq!(preview_transform.invalidated_clip_count, 0);
    assert_eq!(preview_transform.unsupported_clip_count, 0);
    assert_eq!(preview_transform.stretch_aligned_clip_count, 0);
    assert_eq!(preview_transform.artifact_backed_clip_count, 1);
    assert_eq!(preview_transform.fallback_clip_count, 0);
    assert_eq!(
        preview_transform.preview_device_policy.routing_posture,
        RuntimePreviewOutputRoutingPosture::GuardedPreviewOutputRouting
    );
    assert_eq!(
        preview_transform.preview_device_policy.audition_sink_class,
        RuntimeAuditionSinkClass::GuardedPreviewSink
    );
    assert_eq!(
        preview_transform
            .preview_device_policy
            .audition_sink_authority,
        RuntimeAuditionSinkAuthority::RuntimeDefault
    );
    assert_eq!(
        preview_transform
            .preview_device_policy
            .low_latency_device_policy_class,
        RuntimeLowLatencyDevicePolicyClass::GuardedLowLatencyDevicePolicy
    );
    assert_eq!(
        preview_transform
            .preview_device_policy
            .low_latency_device_policy_outcome,
        RuntimeLowLatencyDevicePolicyOutcome::ObserveOnlyPreview
    );
    assert_eq!(
        preview_transform.preview_workflow.queue_posture,
        RuntimePreviewBrowserQueuePosture::SingleActivePreviewQueue
    );
    assert_eq!(
        preview_transform.preview_workflow.queue_class,
        RuntimePreviewBrowserQueueClass::SingleAssetAuditionQueue
    );
    assert_eq!(
        preview_transform.preview_workflow.queue_outcome,
        RuntimePreviewBrowserQueueOutcome::PreserveActivePreviewRequest
    );
    assert_eq!(
        preview_transform.preview_workflow.audition_posture,
        RuntimeMediaAuditionOrchestrationPosture::DirectRuntimeAuditionOrchestration
    );
    assert_eq!(
        preview_transform.preview_workflow.audition_authority,
        RuntimeMediaAuditionOrchestrationAuthority::RuntimeDefault
    );
    assert_eq!(
        preview_transform
            .preview_workflow
            .audition_continuity_outcome,
        RuntimeMediaAuditionContinuityOutcome::PreserveActiveAudition
    );
    assert_eq!(
        preview_transform
            .preview_workflow
            .transform_scheduling_posture,
        RuntimePreviewTransformSchedulingPosture::DirectRuntimeTransformScheduling
    );
    assert_eq!(
        preview_transform
            .preview_workflow
            .transform_scheduling_authority,
        RuntimePreviewTransformSchedulingAuthority::PreviewDemandDerived
    );
    assert_eq!(
        preview_transform
            .preview_workflow
            .transform_scheduling_outcome,
        RuntimePreviewTransformSchedulingOutcome::PreferArtifactBackedPreview
    );
    assert_eq!(
        preview_transform
            .preview_workflow
            .queued_preview_request_count,
        1
    );
    assert_eq!(
        preview_transform.preview_workflow.previewable_asset_count,
        1
    );
    assert_eq!(
        preview_transform
            .preview_workflow
            .active_audition_clip_count,
        1
    );
    assert_eq!(
        preview_transform
            .preview_workflow
            .pending_transform_clip_count,
        0
    );
    assert_eq!(
        preview_transform
            .preview_workflow
            .ready_transform_clip_count,
        1
    );
    assert_eq!(
        preview_transform
            .preview_workflow
            .fallback_transform_clip_count,
        0
    );
    assert_eq!(
        preview_transform.clips[0].service_class,
        RuntimePreviewTransformServiceClass::ArtifactBacked
    );
    assert_eq!(
        preview_transform.clips[0].readiness,
        RuntimePreviewTransformReadiness::Ready
    );
    assert_eq!(
        preview_transform.clips[0].fallback_kind,
        RuntimePreviewTransformFallbackKind::None
    );
    assert!(preview_transform.clips[0].audition_active);
    assert!(preview_transform.clips[0].scrub_supported);

    let observation = RuntimeObservationReport::capture(&runtime, &RuntimeEventRecorder::default());
    assert_eq!(observation.preview_transform_snapshot.clip_count, 1);
    assert_eq!(observation.preview_transform_snapshot.ready_clip_count, 1);
    assert_eq!(
        observation
            .preview_transform_snapshot
            .active_audition_clip_count,
        1
    );
    assert_eq!(
        observation
            .preview_transform_snapshot
            .preview_device_policy
            .routing_posture,
        RuntimePreviewOutputRoutingPosture::GuardedPreviewOutputRouting
    );
    assert_eq!(
        observation
            .preview_transform_snapshot
            .preview_workflow
            .queue_posture,
        RuntimePreviewBrowserQueuePosture::SingleActivePreviewQueue
    );
    assert!(observation
        .render_json()
        .contains("\"preview_transform_snapshot\":{\"clip_count\":1"));
    assert!(observation.render_json().contains(
        "\"preview_device_policy\":{\"routing_posture\":\"GuardedPreviewOutputRouting\""
    ));
    assert!(observation
        .render_json()
        .contains("\"preview_workflow\":{\"queue_posture\":\"SingleActivePreviewQueue\""));

    let rendered = runtime
        .render_clip_processing_buffer(RuntimeClipRenderRequest {
            clip_id: "clip:preview-transform-ready".to_string(),
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
        rendered.preview_transform_snapshot.service_class,
        RuntimePreviewTransformServiceClass::ArtifactBacked
    );
    assert_eq!(
        rendered.preview_transform_snapshot.readiness,
        RuntimePreviewTransformReadiness::Ready
    );
    assert!(rendered.preview_transform_snapshot.audition_active);
    assert!(rendered
        .summary
        .contains("preview=ArtifactBacked/Ready/None/None"));

    let preview = RuntimeOfflineRenderContractPreview::from_runtime_state(
        &RuntimeOfflineRenderRequest {
            request_id: "render:preview-transform-preview".into(),
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
    .expect("build preview transform offline render preview");
    assert_eq!(preview.preview_transform_snapshot.clip_count, 1);
    assert_eq!(preview.preview_transform_snapshot.ready_clip_count, 1);
    assert_eq!(
        preview
            .preview_transform_snapshot
            .artifact_backed_clip_count,
        1
    );
    assert_eq!(
        preview
            .preview_transform_snapshot
            .active_audition_clip_count,
        0
    );
    assert_eq!(
        preview
            .preview_transform_snapshot
            .preview_device_policy
            .routing_posture,
        RuntimePreviewOutputRoutingPosture::NoPreviewOutputRouting
    );
    assert_eq!(
        preview
            .preview_transform_snapshot
            .preview_workflow
            .queue_posture,
        RuntimePreviewBrowserQueuePosture::GuardedPreviewQueue
    );
    assert_eq!(
        preview
            .preview_transform_snapshot
            .preview_workflow
            .queue_class,
        RuntimePreviewBrowserQueueClass::PreviewAssetSelectionQueue
    );
    assert_eq!(
        preview
            .preview_transform_snapshot
            .preview_workflow
            .queue_outcome,
        RuntimePreviewBrowserQueueOutcome::CollapseToSingleActivePreview
    );
    assert_eq!(
        preview
            .preview_transform_snapshot
            .preview_workflow
            .audition_continuity_outcome,
        RuntimeMediaAuditionContinuityOutcome::ResumePreviewAudition
    );
    assert_eq!(
        preview
            .preview_transform_snapshot
            .preview_workflow
            .transform_scheduling_outcome,
        RuntimePreviewTransformSchedulingOutcome::PreferArtifactBackedPreview
    );
    assert!(preview
        .summary
        .contains("preview_transform=1/artifact_backed=1/fallback=0"));

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
