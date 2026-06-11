use super::super::*;
use std::fs;

#[test]
fn local_host_round_trips_delegated_offline_execution_through_runtime_finalization() {
    let (host, imported_path) = prepare_local_host_for_offline_render();
    let artifact_dir = temp_artifact_dir("offline-render-local-host-delegated");
    let handoff = host.runtime.get_plugin_recall_handoff_snapshot();
    let mut result = host
        .runtime
        .render_offline(RuntimeOfflineRenderRequest {
            request_id: "render:local-host-delegated".into(),
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
        .expect("offline render should succeed");
    let first_handoff_stage = handoff
        .stages
        .first()
        .expect("offline render fixture should expose a recall handoff stage");
    let sample_probe = |buffer: &AudioBuffer| {
        buffer
            .samples()
            .iter()
            .copied()
            .find(|sample| sample.abs() > 1.0e-6)
            .expect("offline render output should include a non-zero sample")
    };
    let original_main_mix = sample_probe(
        result
            .main_mix
            .as_ref()
            .expect("offline render should include a main mix"),
    );
    let original_main_peak = result
        .main_mix_peak_level
        .expect("offline render should include a main mix peak");
    let original_stem = sample_probe(&result.stems[0].output);
    let original_stem_peak = result.stems[0].peak_level;
    let original_freeze = sample_probe(&result.freeze_artifacts[0].output);
    let original_freeze_peak = result.freeze_artifacts[0].peak_level;
    result.plugin_execution_boundary = RuntimeOfflinePluginExecutionBoundary {
        request_id: result.request_id.clone(),
        timeline_start_samples: 0,
        duration_samples: 64,
        runtime_sample_rate_hz: 48_000,
        export_sample_rate_hz: 48_000,
        block_size: 32,
        block_count: 2,
        stage_count: 1,
        signal_stage_model_stage_count: 0,
        host_delegate_stage_count: 1,
        fresh_override_stage_count: 0,
        stale_override_stage_count: 1,
        stages: vec![RuntimeOfflinePluginExecutionStageBoundary {
            stage_id: RuntimePluginRecallHandoffStageId {
                chain_id: first_handoff_stage.stage_id.chain_id.clone(),
                stage_index: first_handoff_stage.stage_id.stage_index,
                node_id: first_handoff_stage.stage_id.node_id.clone(),
            },
            node_id: first_handoff_stage.node_id.clone(),
            chain_id: first_handoff_stage.chain_id.clone(),
            stage_index: first_handoff_stage.stage_index,
            sandbox_id: first_handoff_stage.recall_payload.sandbox_id.clone(),
            plugin_type_id: first_handoff_stage.recall_payload.plugin_type_id.clone(),
            plugin_format: first_handoff_stage.recall_payload.plugin_format,
            track_lane_id: first_handoff_stage.track_lane_id.clone(),
            bus_group_id: first_handoff_stage.bus_group_id.clone(),
            console_group_id: first_handoff_stage.console_group_id.clone(),
            send_return_id: first_handoff_stage.send_return_id.clone(),
            recall_state: first_handoff_stage.recall_state,
            recall_payload: first_handoff_stage.recall_payload.clone(),
            execution_owner: RuntimeOfflinePluginExecutionOwner::HostDelegated,
            host_delegate_required: true,
            override_state: RuntimeOfflinePluginOverrideState::StaleLatestBlock,
            latest_override_processing_epoch: Some(1),
            latest_override_block_sequence: Some(1),
        }],
    };

    let updated = host
        .finalize_offline_render_with_local_delegated_executor(result)
        .expect("local delegated finalization should succeed");

    let attenuation = 0.5_f32;
    assert_eq!(updated.manifest.delegated_execution_request.stage_count, 1);
    assert_eq!(
        updated
            .manifest
            .delegated_execution_request
            .stages
            .first()
            .map(|stage| stage.node_id.as_str()),
        Some("plugin")
    );
    let receipt = updated
        .manifest
        .delegated_execution_receipt
        .as_ref()
        .expect("delegated receipt should be materialized");
    assert_eq!(receipt.completed_stage_count, 1);
    assert_eq!(receipt.unavailable_stage_count, 0);
    assert_eq!(
        receipt.stages[0].delegate_label.as_deref(),
        Some("local-host-delegated-executor")
    );
    assert_eq!(
        receipt.stages[0].status,
        signal_runtime::RuntimeOfflinePluginDelegatedExecutionStatus::Completed
    );
    assert!(
        (sample_probe(updated.main_mix.as_ref().unwrap()) - (original_main_mix * attenuation))
            .abs()
            < 1.0e-6
    );
    assert!(
        (updated.main_mix_peak_level.unwrap() - (original_main_peak * attenuation)).abs() < 1.0e-6
    );
    assert!((sample_probe(&updated.stems[0].output) - (original_stem * attenuation)).abs() < 1.0e-6);
    assert!((updated.stems[0].peak_level - (original_stem_peak * attenuation)).abs() < 1.0e-6);
    assert!(
        (sample_probe(&updated.freeze_artifacts[0].output) - (original_freeze * attenuation))
            .abs()
            < 1.0e-6
    );
    assert!(
        (updated.freeze_artifacts[0].peak_level - (original_freeze_peak * attenuation)).abs()
            < 1.0e-6
    );
    let report_receipt = updated
        .manifest
        .report
        .as_ref()
        .expect("materialized report receipt should exist");
    let report_body = fs::read_to_string(&report_receipt.report_path).expect("read report");
    assert!(report_body.contains("\"delegate_label\":\"local-host-delegated-executor\""));
    assert!(report_body.contains("\"delegated_receipt_stage_count\":1"));

    let main_mix_receipt = updated
        .manifest
        .artifacts
        .iter()
        .find(|receipt| receipt.artifact_kind == RuntimeOfflineRenderArtifactKind::MainMix)
        .expect("main mix receipt should exist");
    let mut main_mix_reader =
        hound::WavReader::open(&main_mix_receipt.output_path).expect("main mix wav readable");
    let first_non_zero_sample = main_mix_reader
        .samples::<f32>()
        .find_map(|sample| {
            let sample = sample.expect("main mix wav sample should decode");
            (sample.abs() > 1.0e-6).then_some(sample)
        })
        .expect("main mix wav should contain a non-zero sample");
    assert!((first_non_zero_sample - (original_main_mix * attenuation)).abs() < 1.0e-5);

    let _ = fs::remove_file(imported_path);
    if let Some(path) = host
        .runtime
        .get_media_pipeline_snapshot()
        .assets
        .first()
        .and_then(|asset| asset.cache_path.as_deref())
    {
        let _ = fs::remove_file(path);
    }
    for receipt in &updated.manifest.artifacts {
        let _ = fs::remove_file(&receipt.output_path);
    }
    if let Some(report_receipt) = &updated.manifest.report {
        let _ = fs::remove_file(&report_receipt.report_path);
    }
    let _ = fs::remove_dir(&artifact_dir);
}
