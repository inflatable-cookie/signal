use super::*;

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
    assert!(queue_result.summary.contains("deferred_job_count=1"));

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
fn runtime_offline_render_invalid_request_abort_surfaces_typed_cancellation_policy() {
    let runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 64));

    let error = runtime
        .render_offline_queue(Vec::new())
        .expect_err("empty offline render queue should be rejected");

    assert_eq!(error.kind, RuntimeErrorKind::InvalidRequest);
    let report = RuntimeSupervisorReport::capture(&runtime, &RuntimeEventRecorder::default());
    let receipt = report
        .observation
        .last_deferred_service_receipt
        .as_ref()
        .expect("invalid request should record a deferred-service receipt");
    assert_eq!(
        receipt.work_class,
        RuntimeDeferredServiceClass::OfflineRenderQueue
    );
    assert_eq!(receipt.decision, RuntimeDeferredServiceDecision::Abort);
    assert_eq!(receipt.reason, RuntimeDeferredServiceReason::InvalidRequest);
    assert_eq!(
        receipt.priority_band,
        RuntimeDeferredServicePriorityBand::UserVisible
    );
    assert_eq!(receipt.blocking_priority_band, None);
    assert_eq!(receipt.backpressure_source, None);
}
