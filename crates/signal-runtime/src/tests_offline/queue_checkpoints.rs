use super::*;

#[test]
fn runtime_offline_render_queue_executes_requests_in_order_and_tracks_queue_completion_progress() {
    let (runtime, imported_path) = prepare_offline_render_engine_runtime();
    let first_artifact_dir = temp_artifact_dir("offline-render-queue-first");
    let second_artifact_dir = temp_artifact_dir("offline-render-queue-second");
    let handoff = runtime.get_plugin_recall_handoff_snapshot();
    let selection = RuntimePluginRecallHandoffSelection {
        stage_count: handoff.stage_count,
        stage_ids: handoff
            .stages
            .iter()
            .map(|stage| stage.stage_id.clone())
            .collect(),
    };

    let queue_result = runtime
        .render_offline_queue(vec![
            RuntimeOfflineRenderRequest {
                request_id: "render:queue:0001".into(),
                timeline_start_samples: 0,
                duration_samples: 64,
                export_sample_rate_hz: 48_000,
                include_main_mix: true,
                artifact_root_path: Some(first_artifact_dir.display().to_string()),
                stem_targets: vec![RuntimeOfflineRenderStemTarget {
                    stem_id: "stem:track:lead".into(),
                    target_kind: RuntimeOfflineRenderTargetKind::TrackLane,
                    target_id: Some("track:lead".into()),
                }],
                freeze_artifacts: vec![RuntimeOfflineFreezeArtifactRequest {
                    artifact_id: "freeze:track:lead".into(),
                    source_stem_id: "stem:track:lead".into(),
                    recall_selection: selection.clone(),
                }],
            },
            RuntimeOfflineRenderRequest {
                request_id: "render:queue:0002".into(),
                timeline_start_samples: 32,
                duration_samples: 64,
                export_sample_rate_hz: 24_000,
                include_main_mix: true,
                artifact_root_path: Some(second_artifact_dir.display().to_string()),
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
            },
        ])
        .expect("offline render queue should succeed");

    assert_eq!(queue_result.queue_count, 2);
    assert_eq!(queue_result.completed_job_count, 2);
    assert_eq!(
        queue_result.orchestration.decision,
        RuntimeDeferredServiceDecision::Run
    );
    assert_eq!(
        queue_result.orchestration.reason,
        RuntimeDeferredServiceReason::Ready
    );
    assert_eq!(
        queue_result.orchestration.priority_band,
        RuntimeDeferredServicePriorityBand::UserVisible
    );
    assert_eq!(queue_result.orchestration.blocking_priority_band, None);
    assert_eq!(queue_result.orchestration.backpressure_source, None);
    assert!(!queue_result.orchestration.starvation_risk);
    assert_eq!(queue_result.orchestration.starved_work_item_count, 0);
    assert_eq!(queue_result.orchestration.cancellation_cause, None);
    assert_eq!(queue_result.orchestration.cancelled_work_item_count, 0);
    assert_eq!(queue_result.orchestration.admitted_work_item_count, 2);
    assert_eq!(queue_result.orchestration.completed_work_item_count, 2);
    assert_eq!(queue_result.orchestration.deferred_work_item_count, 0);
    assert_eq!(queue_result.progress.len(), 2);
    assert_eq!(queue_result.results.len(), 2);
    assert!(queue_result.deferred_requests.is_empty());
    assert_eq!(queue_result.progress[0].request_id, "render:queue:0001");
    assert_eq!(queue_result.progress[0].queue_index, 0);
    assert_eq!(queue_result.progress[0].completed_job_count, 1);
    assert_eq!(queue_result.progress[0].progress_percent, 50);
    assert_eq!(queue_result.progress[1].request_id, "render:queue:0002");
    assert_eq!(queue_result.progress[1].queue_index, 1);
    assert_eq!(queue_result.progress[1].completed_job_count, 2);
    assert_eq!(queue_result.progress[1].progress_percent, 100);
    assert_eq!(queue_result.results[0].request_id, "render:queue:0001");
    assert_eq!(queue_result.results[1].request_id, "render:queue:0002");
    assert_eq!(
        queue_result.results[0]
            .manifest
            .artifact_root_path
            .as_deref(),
        Some(
            first_artifact_dir
                .to_str()
                .expect("first artifact dir should be valid utf-8")
        )
    );
    assert_eq!(
        queue_result.results[1]
            .manifest
            .artifact_root_path
            .as_deref(),
        Some(
            second_artifact_dir
                .to_str()
                .expect("second artifact dir should be valid utf-8")
        )
    );
    assert_eq!(queue_result.results[0].manifest.artifact_count, 3);
    assert_eq!(queue_result.results[1].manifest.artifact_count, 3);
    assert!(queue_result.results[0].manifest.report.is_some());
    assert!(queue_result.results[1].manifest.report.is_some());
    assert_eq!(
        queue_result.results[1]
            .main_mix
            .as_ref()
            .expect("second main mix should exist")
            .sample_rate()
            .0,
        24_000
    );
    assert!(queue_result.summary.contains("queue_count=2"));
    assert!(queue_result.summary.contains("completed_job_count=2"));

    let _ = fs::remove_file(imported_path);
    if let Some(path) = runtime
        .get_media_pipeline_snapshot()
        .assets
        .first()
        .and_then(|asset| asset.cache_path.as_deref())
    {
        let _ = fs::remove_file(path);
    }
    for result in &queue_result.results {
        for receipt in &result.manifest.artifacts {
            let _ = fs::remove_file(&receipt.output_path);
        }
        if let Some(report_receipt) = &result.manifest.report {
            let _ = fs::remove_file(&report_receipt.report_path);
        }
    }
    let _ = fs::remove_dir(&first_artifact_dir);
    let _ = fs::remove_dir(&second_artifact_dir);
}

#[test]
fn runtime_offline_render_with_checkpoints_reports_runtime_owned_progress_stages() {
    let (runtime, imported_path) = prepare_offline_render_engine_runtime();
    let artifact_dir = temp_artifact_dir("offline-render-checkpoints");
    let handoff = runtime.get_plugin_recall_handoff_snapshot();
    let selection = RuntimePluginRecallHandoffSelection {
        stage_count: handoff.stage_count,
        stage_ids: handoff
            .stages
            .iter()
            .map(|stage| stage.stage_id.clone())
            .collect(),
    };

    let execution = runtime
        .render_offline_with_checkpoints(RuntimeOfflineRenderRequest {
            request_id: "render:checkpoint:0001".into(),
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
        .expect("offline render with checkpoints should succeed");

    assert_eq!(execution.request_id, "render:checkpoint:0001");
    assert_eq!(execution.result.request_id, "render:checkpoint:0001");
    assert_eq!(execution.checkpoint_count, execution.checkpoints.len());
    assert!(execution.checkpoint_count >= 4);
    assert_eq!(
        execution
            .checkpoints
            .first()
            .map(|checkpoint| checkpoint.stage),
        Some(RuntimeOfflineRenderCheckpointStage::PreparingInput)
    );
    assert!(execution.checkpoints.iter().any(|checkpoint| {
        checkpoint.stage == RuntimeOfflineRenderCheckpointStage::RenderingGraph
            && checkpoint.progress_percent >= 10
            && checkpoint.progress_percent <= 90
    }));
    assert_eq!(
        execution
            .checkpoints
            .last()
            .map(|checkpoint| checkpoint.stage),
        Some(RuntimeOfflineRenderCheckpointStage::FinalizingArtifacts)
    );
    assert_eq!(
        execution
            .checkpoints
            .last()
            .map(|checkpoint| checkpoint.progress_percent),
        Some(99)
    );
    assert!(execution
        .checkpoints
        .windows(2)
        .all(|window| window[0].checkpoint_index < window[1].checkpoint_index));
    assert_eq!(
        execution
            .checkpoints
            .last()
            .map(|checkpoint| checkpoint.checkpoint_count),
        Some(execution.checkpoint_count)
    );
    assert!(execution.summary.contains("checkpoints="));

    let _ = fs::remove_file(imported_path);
    if let Some(path) = runtime
        .get_media_pipeline_snapshot()
        .assets
        .first()
        .and_then(|asset| asset.cache_path.as_deref())
    {
        let _ = fs::remove_file(path);
    }
    for receipt in &execution.result.manifest.artifacts {
        let _ = fs::remove_file(&receipt.output_path);
    }
    if let Some(report_receipt) = &execution.result.manifest.report {
        let _ = fs::remove_file(&report_receipt.report_path);
    }
    let _ = fs::remove_dir(&artifact_dir);
}

