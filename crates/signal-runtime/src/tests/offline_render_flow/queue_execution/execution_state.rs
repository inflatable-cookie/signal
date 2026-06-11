use super::*;

#[test]
fn runtime_offline_render_execution_streams_checkpoints_before_delivery_completion() {
    let (mut runtime, imported_path) = prepare_offline_render_engine_runtime();
    let artifact_dir = temp_artifact_dir("offline-render-streaming");
    let handoff = runtime.get_plugin_recall_handoff_snapshot();
    let selection = RuntimePluginRecallHandoffSelection {
        stage_count: handoff.stage_count,
        stage_ids: handoff
            .stages
            .iter()
            .map(|stage| stage.stage_id.clone())
            .collect(),
    };

    let begin = runtime
        .begin_offline_render_execution(RuntimeOfflineRenderRequest {
            request_id: "render:stream:0001".into(),
            timeline_start_samples: 0,
            duration_samples: 2048,
            export_sample_rate_hz: 48_000,
            include_main_mix: true,
            artifact_root_path: Some(artifact_dir.display().to_string()),
            stem_targets: vec![RuntimeOfflineRenderStemTarget {
                stem_id: "stem:track:lead".into(),
                target_kind: RuntimeOfflineRenderTargetKind::TrackLane,
                target_id: Some("track:lead".into()),
            }],
            freeze_artifacts: vec![RuntimeOfflineFreezeArtifactRequest {
                artifact_id: "freeze:track:lead".into(),
                source_stem_id: "stem:track:lead".into(),
                recall_selection: selection,
            }],
        })
        .expect("offline render execution should begin");

    assert_eq!(begin.state, RuntimeOfflineRenderExecutionState::Running);
    assert_eq!(begin.emitted_checkpoint_count, 1);
    assert_eq!(
        begin.checkpoint.as_ref().map(|checkpoint| checkpoint.stage),
        Some(RuntimeOfflineRenderCheckpointStage::PreparingInput)
    );
    assert!(!artifact_dir.exists());

    let mut observed_stages = vec![
        begin
            .checkpoint
            .as_ref()
            .expect("begin checkpoint should exist")
            .stage,
    ];
    let mut completed_result = None;
    for _ in 0..32 {
        let receipt = runtime
            .advance_offline_render_execution("render:stream:0001")
            .expect("offline render execution step should succeed");
        if let Some(checkpoint) = receipt.checkpoint.as_ref() {
            observed_stages.push(checkpoint.stage);
            assert_eq!(receipt.state, RuntimeOfflineRenderExecutionState::Running);
            assert!(!artifact_dir.exists());
        }
        if let Some(result) = receipt.result {
            assert_eq!(receipt.state, RuntimeOfflineRenderExecutionState::Completed);
            completed_result = Some(result);
            break;
        }
    }

    let completed_result =
        completed_result.expect("offline render execution should complete within the step budget");
    assert!(observed_stages.contains(&RuntimeOfflineRenderCheckpointStage::RenderingGraph));
    assert!(observed_stages.contains(&RuntimeOfflineRenderCheckpointStage::MaterializingOutputs));
    assert!(observed_stages.contains(&RuntimeOfflineRenderCheckpointStage::FinalizingArtifacts));
    assert!(artifact_dir.exists());
    assert_eq!(completed_result.request_id, "render:stream:0001");
    assert!(completed_result.manifest.report.is_some());

    let _ = fs::remove_file(imported_path);
    if let Some(path) = runtime
        .get_media_pipeline_snapshot()
        .assets
        .first()
        .and_then(|asset| asset.cache_path.as_deref())
    {
        let _ = fs::remove_file(path);
    }
    for receipt in &completed_result.manifest.artifacts {
        let _ = fs::remove_file(&receipt.output_path);
    }
    if let Some(report_receipt) = &completed_result.manifest.report {
        let _ = fs::remove_file(&report_receipt.report_path);
    }
    let _ = fs::remove_dir(&artifact_dir);
}

#[test]
fn runtime_offline_render_execution_cancels_without_persisted_artifacts() {
    let (mut runtime, imported_path) = prepare_offline_render_engine_runtime();
    let artifact_dir = temp_artifact_dir("offline-render-cancel");

    runtime
        .begin_offline_render_execution(RuntimeOfflineRenderRequest {
            request_id: "render:cancel:0001".into(),
            timeline_start_samples: 0,
            duration_samples: 2048,
            export_sample_rate_hz: 48_000,
            include_main_mix: true,
            artifact_root_path: Some(artifact_dir.display().to_string()),
            stem_targets: Vec::new(),
            freeze_artifacts: Vec::new(),
        })
        .expect("offline render execution should begin");
    runtime
        .advance_offline_render_execution("render:cancel:0001")
        .expect("offline render execution should advance");

    let cancelled = runtime
        .cancel_offline_render_execution("render:cancel:0001")
        .expect("offline render execution should cancel");

    assert_eq!(cancelled.request_id, "render:cancel:0001");
    assert!(cancelled.cancelled_after_checkpoint_count >= 1);
    assert!(cancelled.rendered_frame_count > 0);
    assert!(!artifact_dir.exists());
    assert!(runtime
        .advance_offline_render_execution("render:cancel:0001")
        .is_err());

    let _ = fs::remove_file(imported_path);
    if let Some(path) = runtime
        .get_media_pipeline_snapshot()
        .assets
        .first()
        .and_then(|asset| asset.cache_path.as_deref())
    {
        let _ = fs::remove_file(path);
    }
    let _ = fs::remove_dir(&artifact_dir);
}

#[test]
fn runtime_offline_render_execution_pauses_and_resumes_without_early_delivery() {
    let (mut runtime, imported_path) = prepare_offline_render_engine_runtime();
    let artifact_dir = temp_artifact_dir("offline-render-pause-resume");

    runtime
        .begin_offline_render_execution(RuntimeOfflineRenderRequest {
            request_id: "render:pause:0001".into(),
            timeline_start_samples: 0,
            duration_samples: 2048,
            export_sample_rate_hz: 48_000,
            include_main_mix: true,
            artifact_root_path: Some(artifact_dir.display().to_string()),
            stem_targets: Vec::new(),
            freeze_artifacts: Vec::new(),
        })
        .expect("offline render execution should begin");
    runtime
        .advance_offline_render_execution("render:pause:0001")
        .expect("offline render execution should advance");

    let paused = runtime
        .pause_offline_render_execution("render:pause:0001")
        .expect("offline render execution should pause");
    assert_eq!(paused.state, RuntimeOfflineRenderExecutionState::Paused);
    assert_eq!(
        paused.interruption_class,
        RuntimeInterruptionClass::Resumable
    );
    assert!(!paused.interruption_rebindable);
    assert!(!artifact_dir.exists());

    let still_paused = runtime
        .advance_offline_render_execution("render:pause:0001")
        .expect("paused offline render execution should not advance");
    assert_eq!(
        still_paused.state,
        RuntimeOfflineRenderExecutionState::Paused
    );
    assert_eq!(
        still_paused.interruption_class,
        RuntimeInterruptionClass::Resumable
    );
    assert!(still_paused.checkpoint.is_none());
    assert!(!artifact_dir.exists());

    let resumed = runtime
        .resume_offline_render_execution("render:pause:0001")
        .expect("offline render execution should resume");
    assert_eq!(resumed.state, RuntimeOfflineRenderExecutionState::Running);
    assert_eq!(resumed.interruption_class, RuntimeInterruptionClass::Steady);

    let mut completed = None;
    for _ in 0..32 {
        let receipt = runtime
            .advance_offline_render_execution("render:pause:0001")
            .expect("resumed offline render execution should advance");
        if let Some(result) = receipt.result {
            completed = Some(result);
            break;
        }
    }
    let completed = completed.expect("paused session should resume to completion");
    assert!(artifact_dir.exists());
    assert!(completed.manifest.report.is_some());

    let _ = fs::remove_file(imported_path);
    if let Some(path) = runtime
        .get_media_pipeline_snapshot()
        .assets
        .first()
        .and_then(|asset| asset.cache_path.as_deref())
    {
        let _ = fs::remove_file(path);
    }
    for receipt in &completed.manifest.artifacts {
        let _ = fs::remove_file(&receipt.output_path);
    }
    if let Some(report_receipt) = &completed.manifest.report {
        let _ = fs::remove_file(&report_receipt.report_path);
    }
    let _ = fs::remove_dir(&artifact_dir);
}
