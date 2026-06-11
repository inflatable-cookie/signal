use super::*;

#[test]
fn runtime_offline_render_session_snapshot_reports_failed_terminal_state_on_delivery_error() {
    let (mut runtime, imported_path) = prepare_offline_render_engine_runtime();

    runtime
        .begin_offline_render_execution(RuntimeOfflineRenderRequest {
            request_id: "render:session:terminal".into(),
            timeline_start_samples: 0,
            duration_samples: 256,
            export_sample_rate_hz: 48_000,
            include_main_mix: true,
            artifact_root_path: Some("/dev/null/signal-runtime-offline-render-terminal".into()),
            stem_targets: Vec::new(),
            freeze_artifacts: Vec::new(),
        })
        .expect("terminal session should begin");

    let mut failure = None;
    for _ in 0..16 {
        match runtime.advance_offline_render_execution("render:session:terminal") {
            Ok(_) => continue,
            Err(error) => {
                failure = Some(error);
                break;
            }
        }
    }
    let failure = failure.expect("terminal session should fail during delivery");
    assert!(matches!(
        failure.kind,
        RuntimeErrorKind::ResourceUnavailable | RuntimeErrorKind::Fatal
    ));

    let snapshot = runtime.get_offline_render_session_snapshot();
    assert_eq!(snapshot.active_session_count, 0);
    assert_eq!(
        snapshot.last_session.as_ref().map(|session| session.state),
        Some(RuntimeOfflineRenderExecutionState::Failed)
    );
    assert_eq!(
        snapshot
            .last_session
            .as_ref()
            .map(|session| session.interruption_class),
        Some(RuntimeInterruptionClass::Terminal)
    );
    assert_eq!(
        snapshot
            .last_session
            .as_ref()
            .and_then(|session| session.last_checkpoint.as_ref())
            .map(|checkpoint| checkpoint.stage),
        Some(RuntimeOfflineRenderCheckpointStage::FinalizingArtifacts)
    );

    let _report = RuntimeSupervisorReport::capture(&runtime, &Default::default());

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
fn runtime_offline_render_queue_throttles_when_runtime_is_running() {
    let (mut runtime, imported_path) = prepare_offline_render_engine_runtime();
    runtime.start().expect("start runtime");

    let first_artifact_dir = temp_artifact_dir("offline-render-queue-throttle-first");
    let second_artifact_dir = temp_artifact_dir("offline-render-queue-throttle-second");
    let queue_result = runtime
        .render_offline_queue(vec![
            RuntimeOfflineRenderRequest {
                request_id: "render:queue:throttle:0001".into(),
                timeline_start_samples: 0,
                duration_samples: 64,
                export_sample_rate_hz: 48_000,
                include_main_mix: true,
                artifact_root_path: Some(first_artifact_dir.display().to_string()),
                stem_targets: Vec::new(),
                freeze_artifacts: Vec::new(),
            },
            RuntimeOfflineRenderRequest {
                request_id: "render:queue:throttle:0002".into(),
                timeline_start_samples: 32,
                duration_samples: 64,
                export_sample_rate_hz: 48_000,
                include_main_mix: true,
                artifact_root_path: Some(second_artifact_dir.display().to_string()),
                stem_targets: Vec::new(),
                freeze_artifacts: Vec::new(),
            },
        ])
        .expect("running runtime should throttle offline render queue");

    assert_eq!(
        queue_result.orchestration.decision,
        RuntimeDeferredServiceDecision::Throttle
    );
    assert_eq!(
        queue_result.orchestration.interruption_class,
        RuntimeInterruptionClass::Resumable
    );
    assert_eq!(
        queue_result.orchestration.reason,
        RuntimeDeferredServiceReason::RealtimeActive
    );
    assert_eq!(
        queue_result.orchestration.priority_band,
        RuntimeDeferredServicePriorityBand::UserVisible
    );
    assert_eq!(
        queue_result.orchestration.blocking_priority_band,
        Some(RuntimeDeferredServicePriorityBand::RealtimeCritical)
    );
    assert_eq!(
        queue_result.orchestration.backpressure_source,
        Some(RuntimeDeferredServiceBackpressureSource::RealtimeAudio)
    );
    assert!(queue_result.orchestration.starvation_risk);
    assert_eq!(queue_result.orchestration.starved_work_item_count, 1);
    assert_eq!(queue_result.orchestration.cancellation_cause, None);
    assert_eq!(queue_result.orchestration.cancelled_work_item_count, 0);
    assert_eq!(queue_result.orchestration.admitted_work_item_count, 1);
    assert_eq!(queue_result.orchestration.completed_work_item_count, 1);
    assert_eq!(queue_result.orchestration.deferred_work_item_count, 1);
    assert_eq!(queue_result.completed_job_count, 1);
    assert_eq!(queue_result.progress.len(), 1);
    assert_eq!(queue_result.results.len(), 1);
    assert_eq!(queue_result.deferred_requests.len(), 1);
    assert_eq!(
        queue_result.results[0].request_id,
        "render:queue:throttle:0001"
    );
    assert_eq!(
        queue_result.deferred_requests[0].request_id,
        "render:queue:throttle:0002"
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
    for receipt in &queue_result.results[0].manifest.artifacts {
        let _ = fs::remove_file(&receipt.output_path);
    }
    if let Some(report_receipt) = &queue_result.results[0].manifest.report {
        let _ = fs::remove_file(&report_receipt.report_path);
    }
    let _ = fs::remove_dir(&first_artifact_dir);
    let _ = fs::remove_dir(&second_artifact_dir);
}

#[test]
fn runtime_offline_render_queue_defers_and_resumes_after_safe_mode_clears() {
    let (mut runtime, imported_path) = prepare_offline_render_engine_runtime();
    runtime
        .set_safe_mode(SafeModeRequest { enabled: true })
        .expect("enable safe mode");

    let deferred = runtime
        .render_offline_queue(vec![RuntimeOfflineRenderRequest {
            request_id: "render:queue:safe-mode:0001".into(),
            timeline_start_samples: 0,
            duration_samples: 64,
            export_sample_rate_hz: 48_000,
            include_main_mix: true,
            artifact_root_path: None,
            stem_targets: Vec::new(),
            freeze_artifacts: Vec::new(),
        }])
        .expect("safe mode should defer offline render queue");

    assert_eq!(
        deferred.orchestration.decision,
        RuntimeDeferredServiceDecision::Defer
    );
    assert_eq!(
        deferred.orchestration.interruption_class,
        RuntimeInterruptionClass::Resumable
    );
    assert_eq!(
        deferred.orchestration.reason,
        RuntimeDeferredServiceReason::SafeMode
    );
    assert_eq!(
        deferred.orchestration.priority_band,
        RuntimeDeferredServicePriorityBand::UserVisible
    );
    assert_eq!(
        deferred.orchestration.blocking_priority_band,
        Some(RuntimeDeferredServicePriorityBand::RecoveryCritical)
    );
    assert_eq!(
        deferred.orchestration.backpressure_source,
        Some(RuntimeDeferredServiceBackpressureSource::SafeMode)
    );
    assert!(deferred.orchestration.starvation_risk);
    assert_eq!(deferred.orchestration.starved_work_item_count, 1);
    assert_eq!(deferred.orchestration.cancellation_cause, None);
    assert_eq!(deferred.orchestration.cancelled_work_item_count, 0);
    assert_eq!(deferred.completed_job_count, 0);
    assert!(deferred.progress.is_empty());
    assert!(deferred.results.is_empty());
    assert_eq!(deferred.deferred_requests.len(), 1);

    runtime
        .set_safe_mode(SafeModeRequest { enabled: false })
        .expect("disable safe mode");
    let resumed = runtime
        .render_offline_queue(deferred.deferred_requests)
        .expect("cleared safe mode should resume deferred queue");

    assert_eq!(
        resumed.orchestration.decision,
        RuntimeDeferredServiceDecision::Run
    );
    assert_eq!(
        resumed.orchestration.interruption_class,
        RuntimeInterruptionClass::Steady
    );
    assert_eq!(resumed.completed_job_count, 1);
    assert_eq!(resumed.results.len(), 1);
    assert!(resumed.deferred_requests.is_empty());
    assert_eq!(resumed.results[0].request_id, "render:queue:safe-mode:0001");

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
fn runtime_offline_render_purge_removes_report_and_artifact_root() {
    let (runtime, imported_path) = prepare_offline_render_engine_runtime();
    let artifact_dir = temp_artifact_dir("offline-render-purge");

    let result = runtime
        .render_offline(RuntimeOfflineRenderRequest {
            request_id: "render:purge-proof".into(),
            timeline_start_samples: 0,
            duration_samples: 64,
            export_sample_rate_hz: 48_000,
            include_main_mix: true,
            artifact_root_path: Some(artifact_dir.display().to_string()),
            stem_targets: vec![RuntimeOfflineRenderStemTarget {
                stem_id: "stem:track:lead".into(),
                target_kind: RuntimeOfflineRenderTargetKind::TrackLane,
                target_id: Some("track:lead".into()),
            }],
            freeze_artifacts: Vec::new(),
        })
        .expect("offline render should materialize purge proof artifacts");
    let report_path = result
        .manifest
        .report
        .as_ref()
        .map(|receipt| receipt.report_path.clone())
        .expect("report receipt should exist");
    assert!(PathBuf::from(&report_path).exists());
    assert!(artifact_dir.exists());

    let purge_receipt = runtime
        .purge_offline_render_artifacts(RuntimeOfflineRenderPurgeRequest {
            request_id: result.request_id.clone(),
            artifact_root_path: result.manifest.artifact_root_path.clone(),
            report_path: Some(report_path.clone()),
        })
        .expect("offline render purge should succeed");

    assert_eq!(purge_receipt.request_id, "render:purge-proof");
    assert_eq!(
        purge_receipt.orchestration.decision,
        RuntimeDeferredServiceDecision::Run
    );
    assert_eq!(
        purge_receipt.orchestration.reason,
        RuntimeDeferredServiceReason::Ready
    );
    assert_eq!(
        purge_receipt.orchestration.priority_band,
        RuntimeDeferredServicePriorityBand::Maintenance
    );
    assert_eq!(purge_receipt.orchestration.blocking_priority_band, None);
    assert_eq!(purge_receipt.orchestration.backpressure_source, None);
    assert!(!purge_receipt.orchestration.starvation_risk);
    assert_eq!(purge_receipt.orchestration.starved_work_item_count, 0);
    assert_eq!(purge_receipt.orchestration.cancellation_cause, None);
    assert_eq!(purge_receipt.orchestration.cancelled_work_item_count, 0);
    assert!(purge_receipt.purged_report);
    assert!(purge_receipt.purged_artifact_root);
    assert!(purge_receipt.purged_report_byte_count > 0);
    assert!(purge_receipt.purged_artifact_file_count > 0);
    assert!(purge_receipt.purged_artifact_byte_count > 0);
    assert!(!PathBuf::from(&report_path).exists());
    assert!(!artifact_dir.exists());

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
