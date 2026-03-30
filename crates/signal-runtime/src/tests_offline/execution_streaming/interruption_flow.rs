use super::*;

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
    assert!(paused.summary.contains("state=paused"));
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

#[test]
fn runtime_offline_render_execution_becomes_recoverable_and_resumes_after_interrupt() {
    let (mut runtime, imported_path) = prepare_offline_render_engine_runtime();
    let artifact_dir = temp_artifact_dir("offline-render-recoverable");

    runtime
        .begin_offline_render_execution(RuntimeOfflineRenderRequest {
            request_id: "render:recover:0001".into(),
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
        .advance_offline_render_execution("render:recover:0001")
        .expect("offline render execution should advance");

    let recoverable = runtime
        .interrupt_offline_render_execution(
            "render:recover:0001",
            "runtime restart boundary".to_string(),
        )
        .expect("offline render execution should become recoverable");
    assert_eq!(
        recoverable.state,
        RuntimeOfflineRenderExecutionState::Recoverable
    );
    assert_eq!(
        recoverable.interruption_class,
        RuntimeInterruptionClass::Resumable
    );
    assert!(recoverable.summary.contains("state=recoverable"));
    assert!(!artifact_dir.exists());

    let still_recoverable = runtime
        .advance_offline_render_execution("render:recover:0001")
        .expect("recoverable execution should not advance until resumed");
    assert_eq!(
        still_recoverable.state,
        RuntimeOfflineRenderExecutionState::Recoverable
    );
    assert_eq!(
        still_recoverable.interruption_class,
        RuntimeInterruptionClass::Resumable
    );
    assert!(still_recoverable.checkpoint.is_none());

    runtime
        .resume_offline_render_execution("render:recover:0001")
        .expect("recoverable execution should resume");
    let mut completed = None;
    for _ in 0..32 {
        let receipt = runtime
            .advance_offline_render_execution("render:recover:0001")
            .expect("resumed recoverable execution should advance");
        if let Some(result) = receipt.result {
            completed = Some(result);
            break;
        }
    }
    let completed = completed.expect("recoverable session should resume to completion");
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
