use super::*;

#[test]
fn runtime_applies_delegated_execution_receipt_into_manifest_bundle() {
    let (runtime, imported_path) = prepare_offline_render_engine_runtime();
    let artifact_dir = temp_artifact_dir("offline-render-delegated-receipt");
    let mut result = runtime
        .render_offline(RuntimeOfflineRenderRequest {
            request_id: "render:delegated-receipt".into(),
            timeline_start_samples: 0,
            duration_samples: 64,
            export_sample_rate_hz: 48_000,
            include_main_mix: true,
            artifact_root_path: Some(artifact_dir.display().to_string()),
            stem_targets: Vec::new(),
            freeze_artifacts: Vec::new(),
        })
        .expect("offline render should succeed");
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
                chain_id: "track:lead".into(),
                stage_index: 0,
                node_id: "plugin-a".into(),
            },
            node_id: "plugin-a".into(),
            chain_id: "track:lead".into(),
            stage_index: 0,
            sandbox_id: Some("sandbox-a".into()),
            plugin_type_id: None,
            plugin_format: None,
            track_lane_id: Some("track:lead".into()),
            bus_group_id: Some("mix:tracks".into()),
            console_group_id: None,
            send_return_id: None,
            recall_state: RuntimePluginRecallState::Recovered,
            recall_payload: RuntimePluginRecallPayload {
                sandbox_id: Some("sandbox-a".into()),
                recovery_count: 1,
                ..RuntimePluginRecallPayload::default()
            },
            execution_owner: RuntimeOfflinePluginExecutionOwner::HostDelegated,
            host_delegate_required: true,
            override_state: RuntimeOfflinePluginOverrideState::StaleLatestBlock,
            latest_override_processing_epoch: Some(4),
            latest_override_block_sequence: Some(9),
            summary: "delegated".into(),
        }],
        summary: "boundary".into(),
    };

    let updated = runtime
        .apply_offline_plugin_delegated_execution_receipt(
            &result,
            RuntimeOfflinePluginDelegatedExecutionReceipt {
                request_id: result.request_id.clone(),
                stage_count: 1,
                completed_stage_count: 1,
                rejected_stage_count: 0,
                unavailable_stage_count: 0,
                stages: vec![RuntimeOfflinePluginDelegatedExecutionStageReceipt {
                    stage_id: RuntimePluginRecallHandoffStageId {
                        chain_id: "track:lead".into(),
                        stage_index: 0,
                        node_id: "plugin-a".into(),
                    },
                    node_id: "plugin-a".into(),
                    chain_id: "track:lead".into(),
                    stage_index: 0,
                    status: RuntimeOfflinePluginDelegatedExecutionStatus::Completed,
                    delegate_label: Some("host:offline-sandbox".into()),
                    detail: Some("rendered by delegated sandbox".into()),
                    summary: "completed".into(),
                }],
                summary: "receipt".into(),
            },
        )
        .expect("delegated execution receipt should apply");

    assert_eq!(updated.manifest.delegated_execution_request.stage_count, 1);
    assert_eq!(
        updated.manifest.delegated_execution_request.stages[0].node_id,
        "plugin-a"
    );
    assert_eq!(
        updated
            .manifest
            .delegated_execution_receipt
            .as_ref()
            .unwrap()
            .completed_stage_count,
        1
    );
    assert!(updated
        .manifest
        .summary
        .contains("delegated_request_stages=1"));
    assert!(updated.manifest.summary.contains("delegated_receipt=true"));
    let report_receipt = updated
        .manifest
        .report
        .as_ref()
        .expect("materialized report receipt should exist");
    let report_body = fs::read_to_string(&report_receipt.report_path).expect("read report");
    assert!(report_body.contains("\"delegated_receipt_stage_count\":1"));
    assert!(report_body.contains("\"delegate_label\":\"host:offline-sandbox\""));
    assert!(report_body.contains("\"status\":\"Completed\""));

    let _ = fs::remove_file(imported_path);
    if let Some(path) = runtime
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

#[test]
fn runtime_offline_render_receipts_pin_delegated_unavailable_boundary() {
    let (runtime, imported_path) = prepare_offline_render_engine_runtime();
    let artifact_dir = temp_artifact_dir("offline-render-receipt-unavailable");
    let mut result = runtime
        .render_offline(RuntimeOfflineRenderRequest {
            request_id: "render:delegated-unavailable".into(),
            timeline_start_samples: 0,
            duration_samples: 64,
            export_sample_rate_hz: 48_000,
            include_main_mix: true,
            artifact_root_path: Some(artifact_dir.display().to_string()),
            stem_targets: Vec::new(),
            freeze_artifacts: Vec::new(),
        })
        .expect("offline render should succeed");
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
                chain_id: "track:lead".into(),
                stage_index: 0,
                node_id: "plugin-a".into(),
            },
            node_id: "plugin-a".into(),
            chain_id: "track:lead".into(),
            stage_index: 0,
            sandbox_id: Some("sandbox-a".into()),
            plugin_type_id: None,
            plugin_format: None,
            track_lane_id: Some("track:lead".into()),
            bus_group_id: Some("mix:tracks".into()),
            console_group_id: None,
            send_return_id: None,
            recall_state: RuntimePluginRecallState::Recovered,
            recall_payload: RuntimePluginRecallPayload {
                sandbox_id: Some("sandbox-a".into()),
                recovery_count: 1,
                ..RuntimePluginRecallPayload::default()
            },
            execution_owner: RuntimeOfflinePluginExecutionOwner::HostDelegated,
            host_delegate_required: true,
            override_state: RuntimeOfflinePluginOverrideState::StaleLatestBlock,
            latest_override_processing_epoch: Some(4),
            latest_override_block_sequence: Some(9),
            summary: "delegated".into(),
        }],
        summary: "boundary".into(),
    };

    let updated = runtime
        .apply_offline_plugin_delegated_execution_receipt(
            &result,
            RuntimeOfflinePluginDelegatedExecutionReceipt {
                request_id: result.request_id.clone(),
                stage_count: 1,
                completed_stage_count: 0,
                rejected_stage_count: 0,
                unavailable_stage_count: 1,
                stages: vec![RuntimeOfflinePluginDelegatedExecutionStageReceipt {
                    stage_id: RuntimePluginRecallHandoffStageId {
                        chain_id: "track:lead".into(),
                        stage_index: 0,
                        node_id: "plugin-a".into(),
                    },
                    node_id: "plugin-a".into(),
                    chain_id: "track:lead".into(),
                    stage_index: 0,
                    status: RuntimeOfflinePluginDelegatedExecutionStatus::Unavailable,
                    delegate_label: Some("host:offline-sandbox".into()),
                    detail: Some("delegate not available during degraded recovery".into()),
                    summary: "unavailable".into(),
                }],
                summary: "receipt".into(),
            },
        )
        .expect("delegated unavailable receipt should apply");

    let profiling = updated.profiling_receipt();
    let soak = updated.soak_receipt();
    assert_eq!(profiling.delegated_stage_count, 1);
    assert_eq!(profiling.stale_override_stage_count, 1);
    assert_eq!(profiling.artifact_count, 1);
    assert!(profiling.report_materialized);
    assert!(profiling
        .render_json()
        .contains("\"delegated_stage_count\":1"));
    assert_eq!(soak.delegated_stage_count, 1);
    assert_eq!(soak.delegated_completed_stage_count, 0);
    assert_eq!(soak.delegated_rejected_stage_count, 0);
    assert_eq!(soak.delegated_unavailable_stage_count, 1);
    assert!(soak
        .render_json()
        .contains("\"delegated_unavailable_stage_count\":1"));

    let _ = fs::remove_file(imported_path);
    if let Some(path) = runtime
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
