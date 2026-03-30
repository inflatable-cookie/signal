use super::*;

use super::*;

#[test]
fn runtime_offline_render_session_snapshot_preserves_checkpoint_through_pause_and_recoverable_states(
) {
    let (mut runtime, imported_path) = prepare_offline_render_engine_runtime();
    let artifact_dir = temp_artifact_dir("offline-render-session-snapshot");

    runtime
        .begin_offline_render_execution(RuntimeOfflineRenderRequest {
            request_id: "render:session:0001".into(),
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
        .advance_offline_render_execution("render:session:0001")
        .expect("offline render execution should advance");

    let running_snapshot = runtime.get_offline_render_session_snapshot();
    assert_eq!(running_snapshot.active_session_count, 1);
    assert_eq!(
        running_snapshot.active_sessions[0].request_id,
        "render:session:0001"
    );
    assert!(running_snapshot.active_sessions[0]
        .last_checkpoint
        .as_ref()
        .is_some());

    runtime
        .pause_offline_render_execution("render:session:0001")
        .expect("offline render execution should pause");
    let paused_snapshot = runtime.get_offline_render_session_snapshot();
    assert_eq!(paused_snapshot.active_session_count, 1);
    assert_eq!(paused_snapshot.paused_session_count, 1);
    assert_eq!(paused_snapshot.recoverable_session_count, 0);
    assert_eq!(
        paused_snapshot.active_sessions[0].state,
        RuntimeOfflineRenderExecutionState::Paused
    );
    assert_eq!(
        paused_snapshot.active_sessions[0].interruption_class,
        RuntimeInterruptionClass::Resumable
    );
    assert!(paused_snapshot.active_sessions[0]
        .active_checkpoint
        .is_some());
    assert!(paused_snapshot.active_sessions[0].last_checkpoint.is_some());
    assert_eq!(
        paused_snapshot
            .last_session
            .as_ref()
            .map(|session| session.state),
        Some(RuntimeOfflineRenderExecutionState::Paused)
    );

    runtime
        .resume_offline_render_execution("render:session:0001")
        .expect("paused execution should resume");
    runtime
        .interrupt_offline_render_execution(
            "render:session:0001",
            "recoverable interruption".into(),
        )
        .expect("running execution should become recoverable");
    let recoverable_snapshot = runtime.get_offline_render_session_snapshot();
    assert_eq!(recoverable_snapshot.active_session_count, 1);
    assert_eq!(recoverable_snapshot.paused_session_count, 0);
    assert_eq!(recoverable_snapshot.recoverable_session_count, 1);
    assert_eq!(
        recoverable_snapshot.active_sessions[0].state,
        RuntimeOfflineRenderExecutionState::Recoverable
    );
    assert_eq!(
        recoverable_snapshot.active_sessions[0].interruption_class,
        RuntimeInterruptionClass::Resumable
    );
    assert_eq!(
        recoverable_snapshot.active_sessions[0].interruption_count,
        1
    );
    assert!(recoverable_snapshot.active_sessions[0]
        .active_checkpoint
        .is_some());
    assert!(recoverable_snapshot.active_sessions[0]
        .last_checkpoint
        .is_some());
    assert_eq!(
        recoverable_snapshot
            .last_session
            .as_ref()
            .map(|session| session.state),
        Some(RuntimeOfflineRenderExecutionState::Recoverable)
    );

    runtime
        .cancel_offline_render_execution("render:session:0001")
        .expect("recoverable execution should cancel for cleanup");

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

