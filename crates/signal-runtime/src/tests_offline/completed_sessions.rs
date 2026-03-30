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

    let report = RuntimeSupervisorReport::capture(&runtime, &Default::default());
    assert!(report
        .render_json()
        .contains("\"offline_render_session_snapshot\":{"));
    assert!(report
        .render_json()
        .contains("\"request_id\":\"render:session:cancelled\""));

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

    let report = RuntimeSupervisorReport::capture(&runtime, &Default::default());
    assert!(report
        .render_json()
        .contains("\"offline_render_session_snapshot\":{"));
    assert!(report.render_json().contains("\"state\":\"Failed\""));
    assert!(report
        .render_json()
        .contains("\"interruption_class\":\"Terminal\""));

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

