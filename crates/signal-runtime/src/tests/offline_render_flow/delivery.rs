use super::*;

#[test]
fn runtime_offline_render_renders_main_mix_stem_and_freeze_from_runtime_owned_state() {
    let (runtime, imported_path) = prepare_offline_render_engine_runtime();

    let processed_before = runtime.get_engine_block_snapshot().processed_blocks;
    let handoff = runtime.get_plugin_recall_handoff_snapshot();
    let selection = RuntimePluginRecallHandoffSelection {
        stage_count: handoff.stage_count,
        stage_ids: handoff
            .stages
            .iter()
            .map(|stage| stage.stage_id.clone())
            .collect(),
    };

    let result = runtime
        .render_offline(RuntimeOfflineRenderRequest {
            request_id: "render:engine-proof".into(),
            timeline_start_samples: 0,
            duration_samples: 64,
            export_sample_rate_hz: 48_000,
            include_main_mix: true,
            artifact_root_path: None,
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
        .expect("offline render should succeed");

    assert_eq!(
        runtime.get_engine_block_snapshot().processed_blocks,
        processed_before
    );
    assert_eq!(result.rendered_frame_count, 64);
    assert_eq!(result.block_count, 1);
    assert_eq!(result.stems.len(), 1);
    assert_eq!(result.freeze_artifacts.len(), 1);
    assert_eq!(result.manifest.artifact_count, 0);
    assert!(result.manifest.artifacts.is_empty());
    assert!(result.manifest.report.is_none());
    assert!(!result.manifest.materialized);
    assert_eq!(result.manifest.delegated_execution_request.stage_count, 0);
    assert!(result.manifest.delegated_execution_receipt.is_none());
    assert_eq!(result.plugin_execution_boundary.stage_count, 1);
    assert_eq!(
        result
            .plugin_execution_boundary
            .signal_stage_model_stage_count,
        1
    );
    assert_eq!(result.main_mix.as_ref().unwrap().frames().0, 64);
    assert_eq!(result.stems[0].output.frames().0, 64);
    assert_eq!(
        result.freeze_artifacts[0].recall_states,
        vec![RuntimePluginRecallState::Recovered]
    );
    assert_eq!(
        result.freeze_artifacts[0].output.samples(),
        result.stems[0].output.samples()
    );
    assert_eq!(
        result.main_mix.as_ref().unwrap().samples(),
        result.stems[0].output.samples()
    );
    assert!((result.main_mix_peak_level.unwrap() - 0.5).abs() < 1.0e-6);
    assert!(result.main_mix_rms_level.unwrap() > 0.15);
    assert!(result.main_mix_rms_level.unwrap() < 0.5);
    let rendered = result.main_mix.as_ref().unwrap().samples();
    assert!((rendered[0] + 0.5).abs() < 1.0e-6);
    assert!((rendered[1] + 0.5).abs() < 1.0e-6);
    assert!((rendered[2] + 0.492_187_5).abs() < 1.0e-6);

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
fn runtime_offline_render_writes_artifact_receipts_and_resamples_export_rate() {
    let (runtime, imported_path) = prepare_offline_render_engine_runtime();
    let artifact_dir = temp_artifact_dir("offline-render-artifacts");
    let handoff = runtime.get_plugin_recall_handoff_snapshot();

    let result = runtime
        .render_offline(RuntimeOfflineRenderRequest {
            request_id: "render:artifact-proof".into(),
            timeline_start_samples: 0,
            duration_samples: 64,
            export_sample_rate_hz: 24_000,
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
                recall_selection: RuntimePluginRecallHandoffSelection {
                    stage_count: handoff.stage_count,
                    stage_ids: handoff
                        .stages
                        .iter()
                        .map(|stage| stage.stage_id.clone())
                        .collect(),
                },
            }],
        })
        .expect("offline render with artifacts should succeed");

    assert_eq!(result.runtime_frame_count, 64);
    assert_eq!(result.rendered_frame_count, 32);
    assert_eq!(result.main_mix.as_ref().unwrap().sample_rate().0, 24_000);
    assert_eq!(result.main_mix.as_ref().unwrap().frames().0, 32);
    assert_eq!(result.stems[0].output.sample_rate().0, 24_000);
    assert_eq!(result.freeze_artifacts[0].output.sample_rate().0, 24_000);
    assert_eq!(result.manifest.artifact_count, 3);
    assert!(result.manifest.materialized);
    assert_eq!(result.manifest.delegated_execution_request.stage_count, 0);
    assert!(result.manifest.delegated_execution_receipt.is_none());
    assert_eq!(
        result.manifest.artifact_root_path.as_deref(),
        Some(
            artifact_dir
                .to_str()
                .expect("artifact dir should be valid utf-8")
        )
    );
    assert_eq!(
        result
            .manifest
            .report
            .as_ref()
            .map(|receipt| receipt.artifact_count),
        Some(3)
    );
    assert!(result
        .manifest
        .artifacts
        .iter()
        .all(|receipt| receipt.sample_rate_hz == 24_000));

    let main_mix_receipt = result
        .manifest
        .artifacts
        .iter()
        .find(|receipt| receipt.artifact_kind == RuntimeOfflineRenderArtifactKind::MainMix)
        .expect("main mix receipt should exist");
    let main_mix_reader =
        hound::WavReader::open(&main_mix_receipt.output_path).expect("main mix wav readable");
    assert_eq!(main_mix_reader.spec().sample_rate, 24_000);

    let report_receipt = result
        .manifest
        .report
        .as_ref()
        .expect("report receipt should exist");
    let report_body = fs::read_to_string(&report_receipt.report_path).expect("read report");
    assert!(report_body.contains("\"artifact_count\":3"));
    assert!(report_body.contains("\"delegated_stage_count\":0"));
    assert!(report_body.contains("\"rendered_frame_count\":32"));

    let _ = fs::remove_file(imported_path);
    if let Some(path) = runtime
        .get_media_pipeline_snapshot()
        .assets
        .first()
        .and_then(|asset| asset.cache_path.as_deref())
    {
        let _ = fs::remove_file(path);
    }
    for receipt in &result.manifest.artifacts {
        let _ = fs::remove_file(&receipt.output_path);
    }
    if let Some(report_receipt) = &result.manifest.report {
        let _ = fs::remove_file(&report_receipt.report_path);
    }
    let _ = fs::remove_dir(&artifact_dir);
}
