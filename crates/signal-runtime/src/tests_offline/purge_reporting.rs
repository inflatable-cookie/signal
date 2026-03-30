use super::*;

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
    assert!(purge_receipt.summary.contains("artifact_files="));
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

#[test]
fn runtime_purge_defers_in_safe_mode_and_observation_export_surfaces_last_decision() {
    let (mut runtime, imported_path) = prepare_offline_render_engine_runtime();
    let artifact_dir = temp_artifact_dir("offline-render-purge-deferred");

    let result = runtime
        .render_offline(RuntimeOfflineRenderRequest {
            request_id: "render:purge-deferred".into(),
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
        .expect("offline render should materialize deferred purge proof artifacts");
    let report_path = result
        .manifest
        .report
        .as_ref()
        .map(|receipt| receipt.report_path.clone())
        .expect("report receipt should exist");

    runtime
        .set_safe_mode(SafeModeRequest { enabled: true })
        .expect("enable safe mode");
    let deferred = runtime
        .purge_offline_render_artifacts(RuntimeOfflineRenderPurgeRequest {
            request_id: result.request_id.clone(),
            artifact_root_path: result.manifest.artifact_root_path.clone(),
            report_path: Some(report_path.clone()),
        })
        .expect("safe mode should defer purge");

    assert_eq!(
        deferred.orchestration.decision,
        RuntimeDeferredServiceDecision::Defer
    );
    assert_eq!(
        deferred.orchestration.reason,
        RuntimeDeferredServiceReason::SafeMode
    );
    assert_eq!(
        deferred.orchestration.priority_band,
        RuntimeDeferredServicePriorityBand::Maintenance
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
    assert!(!deferred.purged_report);
    assert!(!deferred.purged_artifact_root);
    assert!(PathBuf::from(&report_path).exists());
    assert!(artifact_dir.exists());

    let report = RuntimeSupervisorReport::capture(&runtime, &Default::default());
    assert_eq!(
        report
            .observation
            .last_deferred_service_receipt
            .as_ref()
            .map(|receipt| receipt.decision),
        Some(RuntimeDeferredServiceDecision::Defer)
    );
    assert!(report.render_json().contains("\"last_deferred_service\":{"));
    assert!(report
        .render_json()
        .contains("\"work_class\":\"OfflineRenderPurge\""));
    assert!(report.render_json().contains("\"decision\":\"Defer\""));

    runtime
        .set_safe_mode(SafeModeRequest { enabled: false })
        .expect("disable safe mode");
    let resumed = runtime
        .purge_offline_render_artifacts(RuntimeOfflineRenderPurgeRequest {
            request_id: result.request_id,
            artifact_root_path: result.manifest.artifact_root_path,
            report_path: Some(report_path.clone()),
        })
        .expect("cleared safe mode should allow purge");
    assert_eq!(
        resumed.orchestration.decision,
        RuntimeDeferredServiceDecision::Run
    );
    assert!(resumed.purged_report);
    assert!(resumed.purged_artifact_root);
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
