use super::*;

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
    assert!(!receipt.starvation_risk);
    assert_eq!(receipt.starved_work_item_count, 0);
    assert_eq!(
        receipt.cancellation_cause,
        Some(RuntimeDeferredServiceCancellationCause::InvalidRequest)
    );
    assert_eq!(receipt.cancelled_work_item_count, 0);
    assert!(report
        .render_json()
        .contains("\"cancellation_cause\":\"InvalidRequest\""));
}

#[test]
fn runtime_prepare_offline_plugin_execution_boundary_surfaces_runtime_owned_stage_contracts() {
    let (runtime, imported_path) = prepare_offline_render_engine_runtime();
    let boundary = runtime
        .prepare_offline_plugin_execution_boundary(&RuntimeOfflineRenderRequest {
            request_id: "render:boundary".into(),
            timeline_start_samples: 0,
            duration_samples: 64,
            export_sample_rate_hz: 48_000,
            include_main_mix: true,
            artifact_root_path: None,
            stem_targets: Vec::new(),
            freeze_artifacts: Vec::new(),
        })
        .expect("offline plugin boundary should build");

    assert_eq!(boundary.stage_count, 1);
    assert_eq!(boundary.block_count, 1);
    assert_eq!(boundary.signal_stage_model_stage_count, 1);
    assert_eq!(boundary.host_delegate_stage_count, 0);
    assert_eq!(boundary.fresh_override_stage_count, 1);
    assert_eq!(boundary.stale_override_stage_count, 0);
    assert_eq!(
        boundary.stages[0].execution_owner,
        RuntimeOfflinePluginExecutionOwner::SignalStageModel
    );
    assert!(!boundary.stages[0].host_delegate_required);
    assert_eq!(
        boundary.stages[0].override_state,
        RuntimeOfflinePluginOverrideState::FreshLatestBlock
    );
    assert_eq!(boundary.stages[0].sandbox_id.as_deref(), Some("sandbox-a"));
    assert_eq!(
        boundary.stages[0].recall_state,
        RuntimePluginRecallState::Recovered
    );
    assert_eq!(boundary.stages[0].plugin_type_id.as_deref(), None);
    assert_eq!(boundary.stages[0].plugin_format, Some(PluginFormat::Clap));
    assert_eq!(
        boundary.stages[0].recall_payload.plugin_type_id.as_deref(),
        None
    );
    assert_eq!(
        boundary.stages[0].recall_payload.plugin_format,
        Some(PluginFormat::Clap)
    );
    let delegated_request = runtime
        .prepare_offline_plugin_delegated_execution_request(&RuntimeOfflineRenderRequest {
            request_id: "render:boundary".into(),
            timeline_start_samples: 0,
            duration_samples: 64,
            export_sample_rate_hz: 48_000,
            include_main_mix: true,
            artifact_root_path: None,
            stem_targets: Vec::new(),
            freeze_artifacts: Vec::new(),
        })
        .expect("delegated execution request should build");
    assert_eq!(delegated_request.stage_count, 0);
    assert!(delegated_request.stages.is_empty());

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
fn runtime_offline_plugin_delegated_execution_request_filters_host_stages() {
    let boundary = RuntimeOfflinePluginExecutionBoundary {
        request_id: "render:delegated-boundary".into(),
        timeline_start_samples: 0,
        duration_samples: 128,
        runtime_sample_rate_hz: 48_000,
        export_sample_rate_hz: 48_000,
        block_size: 32,
        block_count: 4,
        stage_count: 2,
        signal_stage_model_stage_count: 1,
        host_delegate_stage_count: 1,
        fresh_override_stage_count: 0,
        stale_override_stage_count: 1,
        stages: vec![
            RuntimeOfflinePluginExecutionStageBoundary {
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
                latest_override_processing_epoch: Some(7),
                latest_override_block_sequence: Some(12),
                summary: "delegated".into(),
            },
            RuntimeOfflinePluginExecutionStageBoundary {
                stage_id: RuntimePluginRecallHandoffStageId {
                    chain_id: "track:lead".into(),
                    stage_index: 1,
                    node_id: "plugin-b".into(),
                },
                node_id: "plugin-b".into(),
                chain_id: "track:lead".into(),
                stage_index: 1,
                sandbox_id: Some("sandbox-b".into()),
                plugin_type_id: None,
                plugin_format: None,
                track_lane_id: Some("track:lead".into()),
                bus_group_id: Some("mix:tracks".into()),
                console_group_id: None,
                send_return_id: None,
                recall_state: RuntimePluginRecallState::Warm,
                recall_payload: RuntimePluginRecallPayload {
                    sandbox_id: Some("sandbox-b".into()),
                    ..RuntimePluginRecallPayload::default()
                },
                execution_owner: RuntimeOfflinePluginExecutionOwner::SignalStageModel,
                host_delegate_required: false,
                override_state: RuntimeOfflinePluginOverrideState::NotAvailable,
                latest_override_processing_epoch: None,
                latest_override_block_sequence: None,
                summary: "signal".into(),
            },
        ],
        summary: "boundary".into(),
    };

    let delegated_request = boundary.delegated_execution_request();

    assert_eq!(delegated_request.request_id, "render:delegated-boundary");
    assert_eq!(delegated_request.stage_count, 1);
    assert_eq!(delegated_request.stages[0].node_id, "plugin-a");
    assert_eq!(delegated_request.stages[0].plugin_format, None);
    assert_eq!(
        delegated_request.stages[0].override_state,
        RuntimeOfflinePluginOverrideState::StaleLatestBlock
    );
    assert_eq!(
        delegated_request.stages[0]
            .latest_override_processing_epoch
            .unwrap(),
        7
    );
}
