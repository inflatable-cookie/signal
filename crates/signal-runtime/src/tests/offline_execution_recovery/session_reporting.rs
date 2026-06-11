use super::*;

#[test]
fn runtime_offline_render_session_snapshot_tracks_completed_cancellation_and_purge_receipts() {
    let (mut runtime, imported_path) = prepare_offline_render_engine_runtime();
    let completed_artifact_dir = temp_artifact_dir("offline-render-session-completed");
    let cancelled_artifact_dir = temp_artifact_dir("offline-render-session-cancelled");

    runtime
        .begin_offline_render_execution(RuntimeOfflineRenderRequest {
            request_id: "render:session:completed".into(),
            timeline_start_samples: 0,
            duration_samples: 2048,
            export_sample_rate_hz: 48_000,
            include_main_mix: true,
            artifact_root_path: Some(completed_artifact_dir.display().to_string()),
            stem_targets: Vec::new(),
            freeze_artifacts: Vec::new(),
        })
        .expect("completed session should begin");
    let mut completed_result = None;
    for _ in 0..32 {
        let receipt = runtime
            .advance_offline_render_execution("render:session:completed")
            .expect("completed session should advance");
        if let Some(result) = receipt.result {
            completed_result = Some(result);
            break;
        }
    }
    let completed_result = completed_result.expect("completed session should finish");
    let completed_snapshot = runtime.get_offline_render_session_snapshot();
    assert_eq!(completed_snapshot.active_session_count, 0);
    assert_eq!(
        completed_snapshot
            .last_session
            .as_ref()
            .map(|session| session.state),
        Some(RuntimeOfflineRenderExecutionState::Completed)
    );
    assert_eq!(
        completed_snapshot
            .last_session
            .as_ref()
            .map(|session| session.request_id.as_str()),
        Some("render:session:completed")
    );
    assert_eq!(
        completed_snapshot
            .last_session
            .as_ref()
            .map(|session| session.materialized),
        Some(true)
    );
    assert_eq!(
        completed_snapshot
            .last_session
            .as_ref()
            .map(|session| session.artifact_count),
        Some(completed_result.manifest.artifact_count)
    );
    assert_eq!(
        completed_snapshot
            .last_session
            .as_ref()
            .and_then(|session| session.report_path.as_deref()),
        completed_result
            .manifest
            .report
            .as_ref()
            .map(|report| report.report_path.as_str())
    );

    runtime
        .begin_offline_render_execution(RuntimeOfflineRenderRequest {
            request_id: "render:session:cancelled".into(),
            timeline_start_samples: 0,
            duration_samples: 2048,
            export_sample_rate_hz: 48_000,
            include_main_mix: true,
            artifact_root_path: Some(cancelled_artifact_dir.display().to_string()),
            stem_targets: Vec::new(),
            freeze_artifacts: Vec::new(),
        })
        .expect("cancelled session should begin");
    runtime
        .advance_offline_render_execution("render:session:cancelled")
        .expect("cancelled session should advance");
    runtime
        .cancel_offline_render_execution("render:session:cancelled")
        .expect("cancelled session should cancel");
    let cancelled_snapshot = runtime.get_offline_render_session_snapshot();
    assert_eq!(
        cancelled_snapshot
            .last_session
            .as_ref()
            .map(|session| session.state),
        Some(RuntimeOfflineRenderExecutionState::Cancelled)
    );
    assert_eq!(
        cancelled_snapshot
            .last_cancellation
            .as_ref()
            .map(|receipt| receipt.request_id.as_str()),
        Some("render:session:cancelled")
    );

    let completed_report_path = completed_result
        .manifest
        .report
        .as_ref()
        .map(|receipt| receipt.report_path.clone())
        .expect("completed session should materialize report");
    runtime
        .purge_offline_render_artifacts(RuntimeOfflineRenderPurgeRequest {
            request_id: completed_result.request_id.clone(),
            artifact_root_path: completed_result.manifest.artifact_root_path.clone(),
            report_path: Some(completed_report_path.clone()),
        })
        .expect("purge should succeed");
    let purged_snapshot = runtime.get_offline_render_session_snapshot();
    assert_eq!(
        purged_snapshot
            .last_purge
            .as_ref()
            .map(|receipt| receipt.request_id.as_str()),
        Some("render:session:completed")
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
    let _ = fs::remove_dir(&completed_artifact_dir);
    let _ = fs::remove_dir(&cancelled_artifact_dir);
}

#[test]
fn runtime_offline_render_session_snapshot_reports_restartable_state_across_stop_restart_and_resume(
) {
    let (mut runtime, imported_path) = prepare_offline_render_engine_runtime();
    let artifact_dir = temp_artifact_dir("offline-render-session-restartable");
    runtime.start().expect("runtime should start");

    runtime
        .begin_offline_render_execution(RuntimeOfflineRenderRequest {
            request_id: "render:session:restartable".into(),
            timeline_start_samples: 0,
            duration_samples: 2048,
            export_sample_rate_hz: 48_000,
            include_main_mix: true,
            artifact_root_path: Some(artifact_dir.display().to_string()),
            stem_targets: Vec::new(),
            freeze_artifacts: Vec::new(),
        })
        .expect("restartable session should begin");
    runtime
        .advance_offline_render_execution("render:session:restartable")
        .expect("restartable session should advance");

    runtime
        .stop(StopReason::DeviceReconfigure)
        .expect("runtime stop should succeed");
    let stopped_snapshot = runtime.get_offline_render_session_snapshot();
    assert_eq!(stopped_snapshot.active_session_count, 1);
    assert_eq!(
        stopped_snapshot.active_sessions[0].state,
        RuntimeOfflineRenderExecutionState::Recoverable
    );
    assert_eq!(
        stopped_snapshot.active_sessions[0].interruption_class,
        RuntimeInterruptionClass::Restartable
    );
    assert_eq!(
        stopped_snapshot
            .last_session
            .as_ref()
            .map(|session| session.interruption_class),
        Some(RuntimeInterruptionClass::Restartable)
    );

    runtime
        .restart(RestartRequest { reconfigure: None })
        .expect("runtime restart should succeed");
    let restarted_snapshot = runtime.get_offline_render_session_snapshot();
    assert_eq!(restarted_snapshot.active_session_count, 1);
    assert_eq!(
        restarted_snapshot.active_sessions[0].interruption_class,
        RuntimeInterruptionClass::Restartable
    );

    runtime
        .resume_offline_render_execution("render:session:restartable")
        .expect("restartable session should resume");
    let resumed_snapshot = runtime.get_offline_render_session_snapshot();
    assert_eq!(resumed_snapshot.active_session_count, 1);
    assert_eq!(
        resumed_snapshot.active_sessions[0].state,
        RuntimeOfflineRenderExecutionState::Running
    );
    assert_eq!(
        resumed_snapshot.active_sessions[0].interruption_class,
        RuntimeInterruptionClass::Steady
    );

    let mut completed = None;
    for _ in 0..32 {
        let receipt = runtime
            .advance_offline_render_execution("render:session:restartable")
            .expect("resumed restartable session should advance");
        if let Some(result) = receipt.result {
            completed = Some(result);
            break;
        }
    }
    let completed = completed.expect("restartable session should complete after resume");
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
