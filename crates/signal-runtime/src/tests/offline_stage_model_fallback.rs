use super::*;

#[test]
fn runtime_applies_delegated_execution_outcome_into_runtime_owned_finalization() {
    let (runtime, imported_path) = prepare_offline_render_engine_runtime();
    let artifact_dir = temp_artifact_dir("offline-render-delegated-outcome");
    let handoff = runtime.get_plugin_recall_handoff_snapshot();
    let mut result = runtime
        .render_offline(RuntimeOfflineRenderRequest {
            request_id: "render:delegated-outcome".into(),
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
        .apply_offline_plugin_delegated_execution_outcome(
            &result,
            RuntimeOfflinePluginDelegatedExecutionOutcome {
                receipt: RuntimeOfflinePluginDelegatedExecutionReceipt {
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
                merge: RuntimeOfflinePluginDelegatedExecutionMerge {
                    request_id: result.request_id.clone(),
                    main_mix: Some(filled_stereo_buffer(48_000, 64, 0.2)),
                    stems: vec![RuntimeOfflinePluginDelegatedStemOutput {
                        stem_id: "stem:track:lead".into(),
                        output: filled_stereo_buffer(48_000, 64, 0.1),
                        summary: "stem override".into(),
                    }],
                    freeze_artifacts: vec![RuntimeOfflinePluginDelegatedFreezeArtifactOutput {
                        artifact_id: "freeze:track:lead".into(),
                        output: filled_stereo_buffer(48_000, 64, 0.05),
                        summary: "freeze override".into(),
                    }],
                    summary: "merge".into(),
                },
                summary: "outcome".into(),
            },
        )
        .expect("delegated execution outcome should apply");

    assert!((updated.main_mix_peak_level.unwrap() - 0.2).abs() < 1.0e-6);
    assert!((updated.stems[0].peak_level - 0.1).abs() < 1.0e-6);
    assert!((updated.freeze_artifacts[0].peak_level - 0.05).abs() < 1.0e-6);
    assert_eq!(updated.main_mix.as_ref().unwrap().samples()[0], 0.2);
    assert_eq!(updated.stems[0].output.samples()[0], 0.1);
    assert_eq!(updated.freeze_artifacts[0].output.samples()[0], 0.05);
    let report_receipt = updated
        .manifest
        .report
        .as_ref()
        .expect("materialized report receipt should exist");
    let report_body = fs::read_to_string(&report_receipt.report_path).expect("read report");
    assert!(report_body.contains("\"delegate_label\":\"host:offline-sandbox\""));
    assert!(report_body.contains("\"peak_level\":0.200000"));
    assert!(report_body.contains("\"peak_level\":0.100000"));
    assert!(report_body.contains("\"peak_level\":0.050000"));

    let main_mix_receipt = updated
        .manifest
        .artifacts
        .iter()
        .find(|receipt| receipt.artifact_kind == RuntimeOfflineRenderArtifactKind::MainMix)
        .expect("main mix receipt should exist");
    let mut main_mix_reader =
        hound::WavReader::open(&main_mix_receipt.output_path).expect("main mix wav readable");
    let first_sample = main_mix_reader
        .samples::<f32>()
        .next()
        .expect("main mix wav should contain samples")
        .expect("main mix wav sample should decode");
    assert!((first_sample - 0.2).abs() < 1.0e-6);

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
fn runtime_offline_render_decodes_non_wav_cached_media_assets() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 32));
    handshake_and_configure_with_disabled_forecast(&mut runtime, true);

    let imported_path = temp_media_path("offline-render-aiff", "aiff");
    let content_hash = imported_path
        .file_stem()
        .and_then(|stem| stem.to_str())
        .expect("offline render AIFF helper path should have a file stem")
        .to_string();
    let asset_id = format!("asset:sha256:{content_hash}");
    write_test_aiff(&imported_path);
    runtime
        .reconcile_media_assets(vec![RuntimeMediaAssetRegistration {
            asset_id: asset_id.clone(),
            content_hash: content_hash.clone(),
            source_path: imported_path.display().to_string(),
            file_name: "offline-render-aiff.aiff".to_string(),
            byte_size: fs::metadata(&imported_path).unwrap().len(),
            sample_rate_hz: 48_000,
            channel_count: 1,
            duration_samples: 128,
            waveform_bin_count: 8,
        }])
        .unwrap();
    runtime
        .reconcile_clip_processing_clips(vec![RuntimeClipProcessingRegistration {
            clip_id: "clip:offline-render-aiff".into(),
            media_asset_id: Some(asset_id),
            warp_mode: RuntimeWarpMode::Off,
            start_samples: 0,
            duration_samples: 64,
            fade_in: RuntimeClipFadeEnvelope::default(),
            fade_out: RuntimeClipFadeEnvelope::default(),
            clip_gain: RuntimeClipGainEnvelope::default(),
        }])
        .unwrap();
    runtime
        .apply_graph_projection(GraphProjection {
            graph_id: "graph:runtime:offline-render-aiff".into(),
            node_count: 1,
            nodes: vec![GraphNodeProjection {
                node_id: "track".into(),
                execution_class: GraphNodeExecutionClass::PureTransform,
                latency_samples: 0,
                stages: vec![GraphStageSpec::Gain { linear: 1.0 }],
            }],
        })
        .unwrap();
    runtime
        .apply_graph_contract_projection(GraphContractProjection {
            graph_id: "graph:runtime:offline-render-aiff".into(),
            contract_count: 1,
            nodes: vec![GraphNodeContractProjection {
                node_id: "track".into(),
                buffer_contract: GraphNodeBufferContractProjection::default(),
                topology: GraphNodeTopologyProjection {
                    role: Some(GraphNodeTopologyRole::TrackLane),
                    track_lane_id: Some("track:lead".into()),
                    bus_group_id: Some("mix:tracks".into()),
                    console_group_id: None,
                    send_return_id: None,
                },
            }],
        })
        .unwrap();

    let result = runtime
        .render_offline(RuntimeOfflineRenderRequest {
            request_id: "render:aiff".into(),
            timeline_start_samples: 0,
            duration_samples: 64,
            export_sample_rate_hz: 48_000,
            include_main_mix: true,
            artifact_root_path: None,
            stem_targets: Vec::new(),
            freeze_artifacts: Vec::new(),
        })
        .expect("offline render should decode AIFF media");

    assert_eq!(result.main_mix.as_ref().unwrap().sample_rate().0, 48_000);
    assert_eq!(result.main_mix.as_ref().unwrap().frames().0, 64);
    assert!(result.main_mix_peak_level.unwrap() > 0.45);
    assert!(result.main_mix_rms_level.unwrap() > 0.15);

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
fn runtime_offline_render_falls_back_to_plugin_stage_model_without_cached_render() {
    let (runtime, imported_path) =
        prepare_offline_render_engine_runtime_without_cached_plugin_render();

    let result = runtime
        .render_offline(RuntimeOfflineRenderRequest {
            request_id: "render:stage-model".into(),
            timeline_start_samples: 0,
            duration_samples: 64,
            export_sample_rate_hz: 48_000,
            include_main_mix: true,
            artifact_root_path: None,
            stem_targets: Vec::new(),
            freeze_artifacts: Vec::new(),
        })
        .expect("offline render should fall back to the plugin stage model");

    assert_eq!(result.rendered_frame_count, 64);
    assert!(result.main_mix_peak_level.unwrap() <= 0.5 + 1.0e-6);
    assert!(result.main_mix_peak_level.unwrap() >= 0.49);
    let first_samples = &result.main_mix.as_ref().unwrap().samples()[..4];
    assert!((first_samples[0] + 0.5).abs() < 1.0e-6);
    assert!((first_samples[1] + 0.5).abs() < 1.0e-6);
    assert!((first_samples[2] + 0.5).abs() < 1.0e-6);
    assert!((first_samples[3] + 0.5).abs() < 1.0e-6);

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
