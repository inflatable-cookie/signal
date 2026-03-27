#[test]
fn runtime_offline_render_contract_preview_reuses_runtime_topology_tempo_clip_and_recall_contracts()
{
    let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
    handshake_and_configure_with_disabled_forecast(&mut runtime, true);
    let scan_handle = runtime.record_plugin_scan_request(&PluginScanRequest {
        roots: vec!["~/Library/Audio/Plug-Ins/VST3".into()],
        formats: vec![PluginFormat::Vst3],
    });
    runtime.record_plugin_scan_results(
        scan_handle,
        vec![
            crate::RuntimePluginDiscoveredTypeRecord {
                plugin_type_id: "plugin:vst3:multiout-instrument".into(),
                plugin_id: "com.signal.multiout".into(),
                vendor: "Signal".into(),
                name: "Signal Multi Output Instrument".into(),
                format: PluginFormat::Vst3,
                version: Some("1.0.0".into()),
                features: vec![
                    signal_plugin::PluginFeature::Instrument,
                    signal_plugin::PluginFeature::Analyzer,
                ],
                default_io_layout: signal_plugin::PluginIoLayout {
                    audio_inputs: 0,
                    audio_outputs: 6,
                    midi_inputs: 1,
                    midi_outputs: 0,
                },
                default_multichannel_io: crate::RuntimeMultichannelIoSummary::for_plugin_io(
                    signal_plugin::PluginIoLayout {
                        audio_inputs: 0,
                        audio_outputs: 6,
                        midi_inputs: 1,
                        midi_outputs: 0,
                    },
                ),
                complex_io_summary:
                    crate::RuntimePluginComplexIoSummary::from_plugin_features_and_layout(
                        &[
                            signal_plugin::PluginFeature::Instrument,
                            signal_plugin::PluginFeature::Analyzer,
                        ],
                        signal_plugin::PluginIoLayout {
                            audio_inputs: 0,
                            audio_outputs: 6,
                            midi_inputs: 1,
                            midi_outputs: 0,
                        },
                    ),
                audio_bus_count: 1,
                parameter_count: 24,
                state_contract: signal_plugin::PluginStateContract {
                    supports_snapshot: false,
                    supports_reset: true,
                    supports_bypass: false,
                    exposes_latency: false,
                    exposes_tail: true,
                },
                processing_contract: signal_plugin::PluginProcessingContract {
                    max_block_frames: 2048,
                    sample_accurate_automation: false,
                    accepts_midi: true,
                    accepts_note_events: true,
                    supports_note_expression: true,
                    produces_midi: false,
                    silence_aware: false,
                },
                lifecycle_contract: signal_plugin::PluginLifecycleContract {
                    requires_main_thread_for_state: true,
                    supports_prepare: true,
                    supports_activate: true,
                    supports_reset_while_active: false,
                },
                lv2_extension_capabilities: None,
                summary: "plugin_type=plugin:vst3:multiout-instrument".into(),
            },
            crate::RuntimePluginDiscoveredTypeRecord {
                plugin_type_id: "plugin:vst3:bus-fx".into(),
                plugin_id: "com.signal.bus-fx".into(),
                vendor: "Signal".into(),
                name: "Signal Bus FX".into(),
                format: PluginFormat::Vst3,
                version: Some("1.0.0".into()),
                features: vec![
                    signal_plugin::PluginFeature::AudioEffect,
                    signal_plugin::PluginFeature::Utility,
                ],
                default_io_layout: signal_plugin::PluginIoLayout {
                    audio_inputs: 4,
                    audio_outputs: 4,
                    midi_inputs: 0,
                    midi_outputs: 0,
                },
                default_multichannel_io: crate::RuntimeMultichannelIoSummary::for_plugin_io(
                    signal_plugin::PluginIoLayout {
                        audio_inputs: 4,
                        audio_outputs: 4,
                        midi_inputs: 0,
                        midi_outputs: 0,
                    },
                ),
                complex_io_summary:
                    crate::RuntimePluginComplexIoSummary::from_plugin_features_and_layout(
                        &[
                            signal_plugin::PluginFeature::AudioEffect,
                            signal_plugin::PluginFeature::Utility,
                        ],
                        signal_plugin::PluginIoLayout {
                            audio_inputs: 4,
                            audio_outputs: 4,
                            midi_inputs: 0,
                            midi_outputs: 0,
                        },
                    ),
                audio_bus_count: 2,
                parameter_count: 18,
                state_contract: signal_plugin::PluginStateContract {
                    supports_snapshot: true,
                    supports_reset: true,
                    supports_bypass: true,
                    exposes_latency: true,
                    exposes_tail: true,
                },
                processing_contract: signal_plugin::PluginProcessingContract {
                    max_block_frames: 4096,
                    sample_accurate_automation: true,
                    accepts_midi: false,
                    accepts_note_events: false,
                    supports_note_expression: false,
                    produces_midi: false,
                    silence_aware: true,
                },
                lifecycle_contract: signal_plugin::PluginLifecycleContract {
                    requires_main_thread_for_state: false,
                    supports_prepare: true,
                    supports_activate: true,
                    supports_reset_while_active: true,
                },
                lv2_extension_capabilities: None,
                summary: "plugin_type=plugin:vst3:bus-fx".into(),
            },
        ],
    );
    runtime
        .apply_graph_projection(GraphProjection {
            graph_id: "graph:runtime:offline-render-preview".into(),
            node_count: 2,
            nodes: vec![
                GraphNodeProjection {
                    node_id: "plugin-a".into(),
                    execution_class: GraphNodeExecutionClass::PluginBacked,
                    latency_samples: 24,
                    stages: vec![GraphStageSpec::HardClip { threshold: 0.7 }],
                },
                GraphNodeProjection {
                    node_id: "plugin-b".into(),
                    execution_class: GraphNodeExecutionClass::PluginBacked,
                    latency_samples: 12,
                    stages: vec![GraphStageSpec::HardClip { threshold: 0.5 }],
                },
            ],
        })
        .expect("apply graph");
    runtime
        .apply_graph_contract_projection(GraphContractProjection {
            graph_id: "graph:runtime:offline-render-preview".into(),
            contract_count: 2,
            nodes: vec![
                GraphNodeContractProjection {
                    node_id: "plugin-a".into(),
                    buffer_contract: GraphNodeBufferContractProjection::default(),
                    topology: GraphNodeTopologyProjection {
                        role: Some(GraphNodeTopologyRole::TrackLane),
                        track_lane_id: Some("track:lead".into()),
                        bus_group_id: Some("mix:tracks".into()),
                        console_group_id: None,
                        send_return_id: None,
                    },
                },
                GraphNodeContractProjection {
                    node_id: "plugin-b".into(),
                    buffer_contract: GraphNodeBufferContractProjection::default(),
                    topology: GraphNodeTopologyProjection {
                        role: Some(GraphNodeTopologyRole::TrackLane),
                        track_lane_id: Some("track:lead".into()),
                        bus_group_id: Some("mix:tracks".into()),
                        console_group_id: None,
                        send_return_id: None,
                    },
                },
            ],
        })
        .expect("apply graph contracts");
    runtime
        .apply_plugin_backed_node_bindings(PluginBackedNodeBindingProjection {
            graph_id: "graph:runtime:offline-render-preview".into(),
            bindings: vec![
                PluginBackedNodeBinding {
                    node_id: "plugin-a".into(),
                    sandbox_id: "sandbox-a".into(),
                },
                PluginBackedNodeBinding {
                    node_id: "plugin-b".into(),
                    sandbox_id: "sandbox-b".into(),
                },
            ],
        })
        .expect("apply bindings");
    runtime.record_plugin_sandbox_spec(&PluginSandboxSpec {
        sandbox_id: "sandbox-a".into(),
        plugin_format: PluginFormat::Vst3,
        plugin_type_id: Some("plugin:vst3:multiout-instrument".into()),
    });
    runtime.record_plugin_sandbox_spec(&PluginSandboxSpec {
        sandbox_id: "sandbox-b".into(),
        plugin_format: PluginFormat::Vst3,
        plugin_type_id: Some("plugin:vst3:bus-fx".into()),
    });
    runtime.record_plugin_sandbox_lifecycle(
        "sandbox-a",
        PluginSandboxLifecycleStage::InstancePrepared,
        Some(1),
    );
    runtime.record_recovery_cycle(
        "sandbox-b",
        RecoveryRestartIntent::CrashRecovery,
        StopReason::DegradedModeRecovery,
        Some(2),
    );
    runtime.record_plugin_sandbox_lifecycle(
        "sandbox-b",
        PluginSandboxLifecycleStage::SandboxRestarted,
        Some(2),
    );
    runtime.record_plugin_sandbox_lifecycle(
        "sandbox-b",
        PluginSandboxLifecycleStage::InstancePrepared,
        Some(3),
    );
    runtime
        .apply_tempo_map_projection(RuntimeTempoMapProjection {
            segment_count: 1,
            segments: vec![crate::interfaces::RuntimeTempoMapSegmentProjection {
                segment_id: "tempo:offline-render".into(),
                start_samples: 0,
                end_samples: Some(48_000),
                start_tempo_bpm: 132.0,
                end_tempo_bpm: None,
                interpolation: RuntimeTempoMapInterpolation::Hold,
            }],
        })
        .expect("apply tempo map");
    runtime
        .apply_transport_projection(TransportProjection {
            playing: false,
            timeline_position_samples: 24_000,
            tempo_bpm: 90.0,
            loop_state: None,
        })
        .expect("apply transport");
    runtime
        .reconcile_clip_processing_clips(vec![RuntimeClipProcessingRegistration {
            clip_id: "clip:offline-render".into(),
            media_asset_id: None,
            warp_mode: RuntimeWarpMode::Off,
            start_samples: 0,
            duration_samples: 48_000,
            fade_in: RuntimeClipFadeEnvelope::default(),
            fade_out: RuntimeClipFadeEnvelope::default(),
            clip_gain: RuntimeClipGainEnvelope::default(),
        }])
        .expect("reconcile clip processing");

    let handoff = runtime.get_plugin_recall_handoff_snapshot();
    let selection = RuntimePluginRecallHandoffSelection {
        stage_count: 2,
        stage_ids: handoff
            .stages
            .iter()
            .map(|stage| stage.stage_id.clone())
            .collect(),
    };
    let request = RuntimeOfflineRenderRequest {
        request_id: "render:preview".into(),
        timeline_start_samples: 0,
        duration_samples: 48_000,
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
            recall_selection: selection.clone(),
        }],
    };

    let preview = RuntimeOfflineRenderContractPreview::from_runtime_state(
        &request,
        &runtime.get_execution_topology_summary(),
        &runtime.get_clip_processing_pipeline_snapshot(),
        &runtime.get_media_pipeline_snapshot(),
        &runtime.get_tempo_map_snapshot(),
        &runtime.get_marker_analysis_snapshot(),
        &handoff,
    )
    .expect("build offline render contract preview");

    assert_eq!(preview.request_id, "render:preview");
    assert_eq!(preview.timeline_end_samples, 48_000);
    assert_eq!(preview.export_sample_rate_hz, 48_000);
    assert_eq!(preview.clip_count, 1);
    assert_eq!(preview.ready_clip_count, 1);
    assert_eq!(preview.stem_count, 1);
    assert_eq!(preview.freeze_artifact_count, 1);
    assert_eq!(preview.resolved_tempo_bpm, 132.0);
    assert_eq!(
        preview.resolved_tempo_source,
        RuntimeTempoSource::TempoMapSegment
    );
    assert_eq!(preview.stem_targets[0].stem_id, "stem:track:lead");
    assert_eq!(
        preview.stem_targets[0].target_kind,
        RuntimeOfflineRenderTargetKind::TrackLane
    );
    assert_eq!(
        preview.stem_targets[0].target_id.as_deref(),
        Some("track:lead")
    );
    assert_eq!(
        preview.stem_targets[0].resolved_node_ids,
        vec!["plugin-a".to_string(), "plugin-b".to_string()]
    );
    assert_eq!(preview.freeze_artifacts[0].artifact_id, "freeze:track:lead");
    assert_eq!(preview.freeze_artifacts[0].recall_stage_count, 2);
    assert_eq!(
        preview.freeze_artifacts[0].recall_stage_ids,
        selection.stage_ids
    );
    assert_eq!(
        preview.freeze_artifacts[0].recall_states,
        vec![
            RuntimePluginRecallState::Warm,
            RuntimePluginRecallState::Recovered
        ]
    );
    assert_eq!(preview.chain_contract.chain_count, 1);
    assert_eq!(preview.chain_contract.stage_count, 2);
    assert_eq!(preview.chain_contract.pending_render_stage_count, 2);
    assert_eq!(preview.chain_contract.settling_stage_count, 0);
    assert_eq!(preview.chain_contract.compensated_stage_count, 0);
    assert_eq!(preview.chain_contract.total_planned_latency_samples, 36);
    assert_eq!(preview.chain_contract.total_realized_latency_samples, 0);
    assert_eq!(preview.chain_contract.total_tail_samples, 0);
    assert_eq!(preview.chain_contract.complex_io_stage_count, 2);
    assert_eq!(
        preview.chain_contract.multi_output_instrument_stage_count,
        1
    );
    assert_eq!(preview.chain_contract.bus_capable_fx_stage_count, 1);
    assert_eq!(preview.chain_contract.sidechain_capable_fx_stage_count, 1);
    assert_eq!(preview.chain_contract.recall_stage_count, 2);
    assert_eq!(preview.chain_contract.warm_recall_stage_count, 1);
    assert_eq!(preview.chain_contract.recovered_recall_stage_count, 1);
    assert_eq!(preview.chain_contract.cold_recall_stage_count, 0);
    assert_eq!(preview.chain_contract.unavailable_recall_stage_count, 0);
    assert_eq!(preview.chain_contract.complex_io_stages.len(), 2);
    assert_eq!(
        preview.chain_contract.complex_io_stages[0].plugin_type_id,
        Some("plugin:vst3:multiout-instrument".to_string())
    );
    assert!(
        preview.chain_contract.complex_io_stages[0]
            .topology
            .multi_output_instrument
    );
    assert_eq!(
        preview.chain_contract.complex_io_stages[0]
            .topology
            .instrument_output_group_count,
        2
    );
    assert_eq!(
        preview.chain_contract.complex_io_stages[1]
            .topology
            .bus_capable_fx_class,
        Some(RuntimePluginBusCapableFxClass::SendReturnCapableFx)
    );
    assert!(preview.chain_contract.summary.contains("pending=2"));
    assert!(preview
        .chain_contract
        .summary
        .contains("complex_io_stages=2"));
    assert!(preview.chain_contract.summary.contains("recall=2/"));
    assert!(preview.summary.contains("stems=1"));
    assert!(preview.summary.contains("freeze_artifacts=1"));
    assert!(preview.summary.contains("chain_contract=chains=1"));
}

#[test]
fn runtime_offline_render_contract_preview_rejects_misaligned_chain_and_recall_contracts() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
    handshake_and_configure_with_disabled_forecast(&mut runtime, true);
    runtime
        .apply_graph_projection(GraphProjection {
            graph_id: "graph:runtime:offline-render-misaligned-contract".into(),
            node_count: 1,
            nodes: vec![GraphNodeProjection {
                node_id: "plugin-a".into(),
                execution_class: GraphNodeExecutionClass::PluginBacked,
                latency_samples: 24,
                stages: vec![GraphStageSpec::HardClip { threshold: 0.7 }],
            }],
        })
        .expect("apply graph");
    runtime
        .apply_graph_contract_projection(GraphContractProjection {
            graph_id: "graph:runtime:offline-render-misaligned-contract".into(),
            contract_count: 1,
            nodes: vec![GraphNodeContractProjection {
                node_id: "plugin-a".into(),
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
        .expect("apply graph contracts");
    runtime
        .apply_plugin_backed_node_bindings(PluginBackedNodeBindingProjection {
            graph_id: "graph:runtime:offline-render-misaligned-contract".into(),
            bindings: vec![PluginBackedNodeBinding {
                node_id: "plugin-a".into(),
                sandbox_id: "sandbox-a".into(),
            }],
        })
        .expect("apply bindings");
    runtime.record_plugin_sandbox_lifecycle(
        "sandbox-a",
        PluginSandboxLifecycleStage::InstancePrepared,
        Some(1),
    );

    let mut handoff = runtime.get_plugin_recall_handoff_snapshot();
    handoff.stage_count = 0;
    handoff.stages.clear();
    handoff.summary = "stages=0".into();
    let request = RuntimeOfflineRenderRequest {
        request_id: "render:misaligned".into(),
        timeline_start_samples: 0,
        duration_samples: 48_000,
        export_sample_rate_hz: 48_000,
        include_main_mix: true,
        artifact_root_path: None,
        stem_targets: Vec::new(),
        freeze_artifacts: Vec::new(),
    };

    let error = RuntimeOfflineRenderContractPreview::from_runtime_state(
        &request,
        &runtime.get_execution_topology_summary(),
        &runtime.get_clip_processing_pipeline_snapshot(),
        &runtime.get_media_pipeline_snapshot(),
        &runtime.get_tempo_map_snapshot(),
        &runtime.get_marker_analysis_snapshot(),
        &handoff,
    )
    .expect_err("misaligned chain and recall contracts should fail");
    assert_eq!(error.kind, RuntimeErrorKind::InvalidState);
    assert!(error
        .message
        .contains("aligned plugin chain and recall handoff"));
}

#[test]
fn runtime_offline_render_contract_preview_carries_sidechain_dependency_receipts() {
    let runtime = prepare_sidechain_runtime();
    let handoff = runtime.get_plugin_recall_handoff_snapshot();
    let request = RuntimeOfflineRenderRequest {
        request_id: "render:sidechain-preview".into(),
        timeline_start_samples: 0,
        duration_samples: 24_000,
        export_sample_rate_hz: 48_000,
        include_main_mix: true,
        artifact_root_path: None,
        stem_targets: Vec::new(),
        freeze_artifacts: Vec::new(),
    };

    let preview = RuntimeOfflineRenderContractPreview::from_runtime_state(
        &request,
        &runtime.get_execution_topology_summary(),
        &runtime.get_clip_processing_pipeline_snapshot(),
        &runtime.get_media_pipeline_snapshot(),
        &runtime.get_tempo_map_snapshot(),
        &runtime.get_marker_analysis_snapshot(),
        &handoff,
    )
    .expect("build offline render sidechain preview");

    assert_eq!(preview.chain_contract.secondary_input_count, 1);
    assert_eq!(preview.chain_contract.required_secondary_input_count, 1);
    assert_eq!(preview.chain_contract.optional_secondary_input_count, 0);
    assert_eq!(preview.chain_contract.disabled_secondary_input_count, 0);
    assert_eq!(
        preview
            .chain_contract
            .terminal_fallback_secondary_input_count,
        0
    );
    assert_eq!(preview.chain_contract.bus_connection_count, 2);
    assert_eq!(preview.chain_contract.auxiliary_path_count, 1);
    let route = &preview.chain_contract.secondary_inputs[0];
    assert_eq!(route.source_id, "sidechain-feed");
    assert_eq!(
        route.target_kind,
        RuntimeSecondaryInputTargetKind::RenderInput
    );
    assert_eq!(route.target_id, "offline-render");
    assert_eq!(route.target_bus_id, "plugin:compressor:sidechain");
    assert_eq!(
        route.fallback_outcome,
        crate::RuntimeSecondaryInputFallbackOutcome::SafeModeDegradation
    );
    assert!(preview
        .chain_contract
        .bus_connections
        .iter()
        .any(|connection| {
            connection.connection_id
                == "track-input:bus:track:lead->plugin-compressor:bus:track:lead"
                && connection.source_bus_role == crate::RuntimeBusRole::ProgramMain
                && connection.target_bus_role == crate::RuntimeBusRole::ProgramMain
        }));
    assert!(preview.chain_contract.auxiliary_paths.iter().any(|path| {
        path.auxiliary_path_id == "bus_group:mix:tracks"
            && path.path_kind == crate::RuntimeAuxiliaryPathKind::Submix
    }));
    assert!(preview
        .chain_contract
        .summary
        .contains("secondary_inputs=1"));
    assert!(preview
        .chain_contract
        .summary
        .contains("bus_connections=2 auxiliary_paths=1"));
    assert!(preview.summary.contains("chain_contract=chains=1"));
}

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
    assert!(result.summary.contains("stems=1"));
    assert!(result.summary.contains("freeze_artifacts=1"));

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

#[test]
fn runtime_offline_render_queue_executes_requests_in_order_and_tracks_queue_completion_progress() {
    let (runtime, imported_path) = prepare_offline_render_engine_runtime();
    let first_artifact_dir = temp_artifact_dir("offline-render-queue-first");
    let second_artifact_dir = temp_artifact_dir("offline-render-queue-second");
    let handoff = runtime.get_plugin_recall_handoff_snapshot();
    let selection = RuntimePluginRecallHandoffSelection {
        stage_count: handoff.stage_count,
        stage_ids: handoff
            .stages
            .iter()
            .map(|stage| stage.stage_id.clone())
            .collect(),
    };

    let queue_result = runtime
        .render_offline_queue(vec![
            RuntimeOfflineRenderRequest {
                request_id: "render:queue:0001".into(),
                timeline_start_samples: 0,
                duration_samples: 64,
                export_sample_rate_hz: 48_000,
                include_main_mix: true,
                artifact_root_path: Some(first_artifact_dir.display().to_string()),
                stem_targets: vec![RuntimeOfflineRenderStemTarget {
                    stem_id: "stem:track:lead".into(),
                    target_kind: RuntimeOfflineRenderTargetKind::TrackLane,
                    target_id: Some("track:lead".into()),
                }],
                freeze_artifacts: vec![RuntimeOfflineFreezeArtifactRequest {
                    artifact_id: "freeze:track:lead".into(),
                    source_stem_id: "stem:track:lead".into(),
                    recall_selection: selection.clone(),
                }],
            },
            RuntimeOfflineRenderRequest {
                request_id: "render:queue:0002".into(),
                timeline_start_samples: 32,
                duration_samples: 64,
                export_sample_rate_hz: 24_000,
                include_main_mix: true,
                artifact_root_path: Some(second_artifact_dir.display().to_string()),
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
            },
        ])
        .expect("offline render queue should succeed");

    assert_eq!(queue_result.queue_count, 2);
    assert_eq!(queue_result.completed_job_count, 2);
    assert_eq!(
        queue_result.orchestration.decision,
        RuntimeDeferredServiceDecision::Run
    );
    assert_eq!(
        queue_result.orchestration.reason,
        RuntimeDeferredServiceReason::Ready
    );
    assert_eq!(
        queue_result.orchestration.priority_band,
        RuntimeDeferredServicePriorityBand::UserVisible
    );
    assert_eq!(queue_result.orchestration.blocking_priority_band, None);
    assert_eq!(queue_result.orchestration.backpressure_source, None);
    assert!(!queue_result.orchestration.starvation_risk);
    assert_eq!(queue_result.orchestration.starved_work_item_count, 0);
    assert_eq!(queue_result.orchestration.cancellation_cause, None);
    assert_eq!(queue_result.orchestration.cancelled_work_item_count, 0);
    assert_eq!(queue_result.orchestration.admitted_work_item_count, 2);
    assert_eq!(queue_result.orchestration.completed_work_item_count, 2);
    assert_eq!(queue_result.orchestration.deferred_work_item_count, 0);
    assert_eq!(queue_result.progress.len(), 2);
    assert_eq!(queue_result.results.len(), 2);
    assert!(queue_result.deferred_requests.is_empty());
    assert_eq!(queue_result.progress[0].request_id, "render:queue:0001");
    assert_eq!(queue_result.progress[0].queue_index, 0);
    assert_eq!(queue_result.progress[0].completed_job_count, 1);
    assert_eq!(queue_result.progress[0].progress_percent, 50);
    assert_eq!(queue_result.progress[1].request_id, "render:queue:0002");
    assert_eq!(queue_result.progress[1].queue_index, 1);
    assert_eq!(queue_result.progress[1].completed_job_count, 2);
    assert_eq!(queue_result.progress[1].progress_percent, 100);
    assert_eq!(queue_result.results[0].request_id, "render:queue:0001");
    assert_eq!(queue_result.results[1].request_id, "render:queue:0002");
    assert_eq!(
        queue_result.results[0]
            .manifest
            .artifact_root_path
            .as_deref(),
        Some(
            first_artifact_dir
                .to_str()
                .expect("first artifact dir should be valid utf-8")
        )
    );
    assert_eq!(
        queue_result.results[1]
            .manifest
            .artifact_root_path
            .as_deref(),
        Some(
            second_artifact_dir
                .to_str()
                .expect("second artifact dir should be valid utf-8")
        )
    );
    assert_eq!(queue_result.results[0].manifest.artifact_count, 3);
    assert_eq!(queue_result.results[1].manifest.artifact_count, 3);
    assert!(queue_result.results[0].manifest.report.is_some());
    assert!(queue_result.results[1].manifest.report.is_some());
    assert_eq!(
        queue_result.results[1]
            .main_mix
            .as_ref()
            .expect("second main mix should exist")
            .sample_rate()
            .0,
        24_000
    );
    assert!(queue_result.summary.contains("queue_count=2"));
    assert!(queue_result.summary.contains("completed_job_count=2"));

    let _ = fs::remove_file(imported_path);
    if let Some(path) = runtime
        .get_media_pipeline_snapshot()
        .assets
        .first()
        .and_then(|asset| asset.cache_path.as_deref())
    {
        let _ = fs::remove_file(path);
    }
    for result in &queue_result.results {
        for receipt in &result.manifest.artifacts {
            let _ = fs::remove_file(&receipt.output_path);
        }
        if let Some(report_receipt) = &result.manifest.report {
            let _ = fs::remove_file(&report_receipt.report_path);
        }
    }
    let _ = fs::remove_dir(&first_artifact_dir);
    let _ = fs::remove_dir(&second_artifact_dir);
}

#[test]
fn runtime_offline_render_with_checkpoints_reports_runtime_owned_progress_stages() {
    let (runtime, imported_path) = prepare_offline_render_engine_runtime();
    let artifact_dir = temp_artifact_dir("offline-render-checkpoints");
    let handoff = runtime.get_plugin_recall_handoff_snapshot();
    let selection = RuntimePluginRecallHandoffSelection {
        stage_count: handoff.stage_count,
        stage_ids: handoff
            .stages
            .iter()
            .map(|stage| stage.stage_id.clone())
            .collect(),
    };

    let execution = runtime
        .render_offline_with_checkpoints(RuntimeOfflineRenderRequest {
            request_id: "render:checkpoint:0001".into(),
            timeline_start_samples: 0,
            duration_samples: 2048,
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
                recall_selection: selection,
            }],
        })
        .expect("offline render with checkpoints should succeed");

    assert_eq!(execution.request_id, "render:checkpoint:0001");
    assert_eq!(execution.result.request_id, "render:checkpoint:0001");
    assert_eq!(execution.checkpoint_count, execution.checkpoints.len());
    assert!(execution.checkpoint_count >= 4);
    assert_eq!(
        execution
            .checkpoints
            .first()
            .map(|checkpoint| checkpoint.stage),
        Some(RuntimeOfflineRenderCheckpointStage::PreparingInput)
    );
    assert!(execution.checkpoints.iter().any(|checkpoint| {
        checkpoint.stage == RuntimeOfflineRenderCheckpointStage::RenderingGraph
            && checkpoint.progress_percent >= 10
            && checkpoint.progress_percent <= 90
    }));
    assert_eq!(
        execution
            .checkpoints
            .last()
            .map(|checkpoint| checkpoint.stage),
        Some(RuntimeOfflineRenderCheckpointStage::FinalizingArtifacts)
    );
    assert_eq!(
        execution
            .checkpoints
            .last()
            .map(|checkpoint| checkpoint.progress_percent),
        Some(99)
    );
    assert!(execution
        .checkpoints
        .windows(2)
        .all(|window| window[0].checkpoint_index < window[1].checkpoint_index));
    assert_eq!(
        execution
            .checkpoints
            .last()
            .map(|checkpoint| checkpoint.checkpoint_count),
        Some(execution.checkpoint_count)
    );
    assert!(execution.summary.contains("checkpoints="));

    let _ = fs::remove_file(imported_path);
    if let Some(path) = runtime
        .get_media_pipeline_snapshot()
        .assets
        .first()
        .and_then(|asset| asset.cache_path.as_deref())
    {
        let _ = fs::remove_file(path);
    }
    for receipt in &execution.result.manifest.artifacts {
        let _ = fs::remove_file(&receipt.output_path);
    }
    if let Some(report_receipt) = &execution.result.manifest.report {
        let _ = fs::remove_file(&report_receipt.report_path);
    }
    let _ = fs::remove_dir(&artifact_dir);
}

#[test]
fn runtime_offline_render_execution_streams_checkpoints_before_delivery_completion() {
    let (mut runtime, imported_path) = prepare_offline_render_engine_runtime();
    let artifact_dir = temp_artifact_dir("offline-render-streaming");
    let handoff = runtime.get_plugin_recall_handoff_snapshot();
    let selection = RuntimePluginRecallHandoffSelection {
        stage_count: handoff.stage_count,
        stage_ids: handoff
            .stages
            .iter()
            .map(|stage| stage.stage_id.clone())
            .collect(),
    };

    let begin = runtime
        .begin_offline_render_execution(RuntimeOfflineRenderRequest {
            request_id: "render:stream:0001".into(),
            timeline_start_samples: 0,
            duration_samples: 2048,
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
                recall_selection: selection,
            }],
        })
        .expect("offline render execution should begin");

    assert_eq!(begin.state, RuntimeOfflineRenderExecutionState::Running);
    assert_eq!(begin.emitted_checkpoint_count, 1);
    assert_eq!(
        begin.checkpoint.as_ref().map(|checkpoint| checkpoint.stage),
        Some(RuntimeOfflineRenderCheckpointStage::PreparingInput)
    );
    assert!(!artifact_dir.exists());

    let mut observed_stages = vec![
        begin
            .checkpoint
            .as_ref()
            .expect("begin checkpoint should exist")
            .stage,
    ];
    let mut completed_result = None;
    for _ in 0..32 {
        let receipt = runtime
            .advance_offline_render_execution("render:stream:0001")
            .expect("offline render execution step should succeed");
        if let Some(checkpoint) = receipt.checkpoint.as_ref() {
            observed_stages.push(checkpoint.stage);
            assert_eq!(receipt.state, RuntimeOfflineRenderExecutionState::Running);
            assert!(!artifact_dir.exists());
        }
        if let Some(result) = receipt.result {
            assert_eq!(receipt.state, RuntimeOfflineRenderExecutionState::Completed);
            completed_result = Some(result);
            break;
        }
    }

    let completed_result =
        completed_result.expect("offline render execution should complete within the step budget");
    assert!(observed_stages.contains(&RuntimeOfflineRenderCheckpointStage::RenderingGraph));
    assert!(observed_stages.contains(&RuntimeOfflineRenderCheckpointStage::MaterializingOutputs));
    assert!(observed_stages.contains(&RuntimeOfflineRenderCheckpointStage::FinalizingArtifacts));
    assert!(artifact_dir.exists());
    assert_eq!(completed_result.request_id, "render:stream:0001");
    assert!(completed_result.manifest.report.is_some());

    let _ = fs::remove_file(imported_path);
    if let Some(path) = runtime
        .get_media_pipeline_snapshot()
        .assets
        .first()
        .and_then(|asset| asset.cache_path.as_deref())
    {
        let _ = fs::remove_file(path);
    }
    for receipt in &completed_result.manifest.artifacts {
        let _ = fs::remove_file(&receipt.output_path);
    }
    if let Some(report_receipt) = &completed_result.manifest.report {
        let _ = fs::remove_file(&report_receipt.report_path);
    }
    let _ = fs::remove_dir(&artifact_dir);
}

#[test]
fn runtime_offline_render_execution_cancels_without_persisted_artifacts() {
    let (mut runtime, imported_path) = prepare_offline_render_engine_runtime();
    let artifact_dir = temp_artifact_dir("offline-render-cancel");

    runtime
        .begin_offline_render_execution(RuntimeOfflineRenderRequest {
            request_id: "render:cancel:0001".into(),
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
        .advance_offline_render_execution("render:cancel:0001")
        .expect("offline render execution should advance");

    let cancelled = runtime
        .cancel_offline_render_execution("render:cancel:0001")
        .expect("offline render execution should cancel");

    assert_eq!(cancelled.request_id, "render:cancel:0001");
    assert!(cancelled.cancelled_after_checkpoint_count >= 1);
    assert!(cancelled.rendered_frame_count > 0);
    assert!(!artifact_dir.exists());
    assert!(runtime
        .advance_offline_render_execution("render:cancel:0001")
        .is_err());

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

#[test]
fn runtime_offline_render_queue_throttles_when_runtime_is_running() {
    let (mut runtime, imported_path) = prepare_offline_render_engine_runtime();
    runtime.start().expect("start runtime");

    let first_artifact_dir = temp_artifact_dir("offline-render-queue-throttle-first");
    let second_artifact_dir = temp_artifact_dir("offline-render-queue-throttle-second");
    let queue_result = runtime
        .render_offline_queue(vec![
            RuntimeOfflineRenderRequest {
                request_id: "render:queue:throttle:0001".into(),
                timeline_start_samples: 0,
                duration_samples: 64,
                export_sample_rate_hz: 48_000,
                include_main_mix: true,
                artifact_root_path: Some(first_artifact_dir.display().to_string()),
                stem_targets: Vec::new(),
                freeze_artifacts: Vec::new(),
            },
            RuntimeOfflineRenderRequest {
                request_id: "render:queue:throttle:0002".into(),
                timeline_start_samples: 32,
                duration_samples: 64,
                export_sample_rate_hz: 48_000,
                include_main_mix: true,
                artifact_root_path: Some(second_artifact_dir.display().to_string()),
                stem_targets: Vec::new(),
                freeze_artifacts: Vec::new(),
            },
        ])
        .expect("running runtime should throttle offline render queue");

    assert_eq!(
        queue_result.orchestration.decision,
        RuntimeDeferredServiceDecision::Throttle
    );
    assert_eq!(
        queue_result.orchestration.interruption_class,
        RuntimeInterruptionClass::Resumable
    );
    assert_eq!(
        queue_result.orchestration.reason,
        RuntimeDeferredServiceReason::RealtimeActive
    );
    assert_eq!(
        queue_result.orchestration.priority_band,
        RuntimeDeferredServicePriorityBand::UserVisible
    );
    assert_eq!(
        queue_result.orchestration.blocking_priority_band,
        Some(RuntimeDeferredServicePriorityBand::RealtimeCritical)
    );
    assert_eq!(
        queue_result.orchestration.backpressure_source,
        Some(RuntimeDeferredServiceBackpressureSource::RealtimeAudio)
    );
    assert!(queue_result.orchestration.starvation_risk);
    assert_eq!(queue_result.orchestration.starved_work_item_count, 1);
    assert_eq!(queue_result.orchestration.cancellation_cause, None);
    assert_eq!(queue_result.orchestration.cancelled_work_item_count, 0);
    assert_eq!(queue_result.orchestration.admitted_work_item_count, 1);
    assert_eq!(queue_result.orchestration.completed_work_item_count, 1);
    assert_eq!(queue_result.orchestration.deferred_work_item_count, 1);
    assert_eq!(queue_result.completed_job_count, 1);
    assert_eq!(queue_result.progress.len(), 1);
    assert_eq!(queue_result.results.len(), 1);
    assert_eq!(queue_result.deferred_requests.len(), 1);
    assert_eq!(
        queue_result.results[0].request_id,
        "render:queue:throttle:0001"
    );
    assert_eq!(
        queue_result.deferred_requests[0].request_id,
        "render:queue:throttle:0002"
    );
    assert!(queue_result.summary.contains("deferred_job_count=1"));

    let _ = fs::remove_file(imported_path);
    if let Some(path) = runtime
        .get_media_pipeline_snapshot()
        .assets
        .first()
        .and_then(|asset| asset.cache_path.as_deref())
    {
        let _ = fs::remove_file(path);
    }
    for receipt in &queue_result.results[0].manifest.artifacts {
        let _ = fs::remove_file(&receipt.output_path);
    }
    if let Some(report_receipt) = &queue_result.results[0].manifest.report {
        let _ = fs::remove_file(&report_receipt.report_path);
    }
    let _ = fs::remove_dir(&first_artifact_dir);
    let _ = fs::remove_dir(&second_artifact_dir);
}

#[test]
fn runtime_offline_render_queue_defers_and_resumes_after_safe_mode_clears() {
    let (mut runtime, imported_path) = prepare_offline_render_engine_runtime();
    runtime
        .set_safe_mode(SafeModeRequest { enabled: true })
        .expect("enable safe mode");

    let deferred = runtime
        .render_offline_queue(vec![RuntimeOfflineRenderRequest {
            request_id: "render:queue:safe-mode:0001".into(),
            timeline_start_samples: 0,
            duration_samples: 64,
            export_sample_rate_hz: 48_000,
            include_main_mix: true,
            artifact_root_path: None,
            stem_targets: Vec::new(),
            freeze_artifacts: Vec::new(),
        }])
        .expect("safe mode should defer offline render queue");

    assert_eq!(
        deferred.orchestration.decision,
        RuntimeDeferredServiceDecision::Defer
    );
    assert_eq!(
        deferred.orchestration.interruption_class,
        RuntimeInterruptionClass::Resumable
    );
    assert_eq!(
        deferred.orchestration.reason,
        RuntimeDeferredServiceReason::SafeMode
    );
    assert_eq!(
        deferred.orchestration.priority_band,
        RuntimeDeferredServicePriorityBand::UserVisible
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
    assert_eq!(deferred.completed_job_count, 0);
    assert!(deferred.progress.is_empty());
    assert!(deferred.results.is_empty());
    assert_eq!(deferred.deferred_requests.len(), 1);

    runtime
        .set_safe_mode(SafeModeRequest { enabled: false })
        .expect("disable safe mode");
    let resumed = runtime
        .render_offline_queue(deferred.deferred_requests)
        .expect("cleared safe mode should resume deferred queue");

    assert_eq!(
        resumed.orchestration.decision,
        RuntimeDeferredServiceDecision::Run
    );
    assert_eq!(
        resumed.orchestration.interruption_class,
        RuntimeInterruptionClass::Steady
    );
    assert_eq!(resumed.completed_job_count, 1);
    assert_eq!(resumed.results.len(), 1);
    assert!(resumed.deferred_requests.is_empty());
    assert_eq!(resumed.results[0].request_id, "render:queue:safe-mode:0001");

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

#[test]
fn runtime_offline_render_ignores_stale_plugin_override_and_uses_stage_model() {
    let (mut runtime, imported_path) =
        prepare_offline_render_engine_runtime_without_cached_plugin_render();
    runtime
        .apply_plugin_node_render_batch(PluginNodeRenderBatch {
            graph_id: "graph:runtime:offline-render-stage-model".into(),
            processing_epoch: 1,
            block_sequence: 1,
            renders: vec![PluginNodeRender {
                node_id: "plugin".into(),
                sandbox_id: "sandbox-a".into(),
                output: AudioBuffer::new(SampleRate(48_000), ChannelLayout::Stereo, FrameCount(32)),
                latency_samples: 0,
                tail_samples: 0,
                bypassed: false,
            }],
        })
        .expect("seed a zero-valued live plugin render override");
    runtime
        .process_engine_block(
            1,
            1,
            AudioBuffer::new(SampleRate(48_000), ChannelLayout::Stereo, FrameCount(32)),
        )
        .expect("consume the seeded live plugin render override");
    runtime
        .process_engine_block(
            1,
            2,
            AudioBuffer::new(SampleRate(48_000), ChannelLayout::Stereo, FrameCount(32)),
        )
        .expect("advance the live engine beyond the last plugin render override");

    let result = runtime
        .render_offline(RuntimeOfflineRenderRequest {
            request_id: "render:stale-plugin-override".into(),
            timeline_start_samples: 0,
            duration_samples: 64,
            export_sample_rate_hz: 48_000,
            include_main_mix: true,
            artifact_root_path: None,
            stem_targets: Vec::new(),
            freeze_artifacts: Vec::new(),
        })
        .expect("offline render should fall back after the live override becomes stale");

    assert_eq!(result.rendered_frame_count, 64);
    assert!((result.main_mix_peak_level.unwrap() - 0.5).abs() < 1.0e-6);
    assert_eq!(result.plugin_execution_boundary.stage_count, 1);
    assert_eq!(
        result.plugin_execution_boundary.fresh_override_stage_count,
        0
    );
    assert_eq!(
        result.plugin_execution_boundary.stale_override_stage_count,
        1
    );
    assert_eq!(
        result.plugin_execution_boundary.stages[0].override_state,
        RuntimeOfflinePluginOverrideState::StaleLatestBlock
    );
    let first_samples = &result.main_mix.as_ref().unwrap().samples()[..6];
    assert!((first_samples[0] + 0.5).abs() < 1.0e-6);
    assert!((first_samples[1] + 0.5).abs() < 1.0e-6);
    assert!((first_samples[2] + 0.5).abs() < 1.0e-6);
    assert!((first_samples[3] + 0.5).abs() < 1.0e-6);
    assert!((first_samples[4] + 0.5).abs() < 1.0e-6);
    assert!((first_samples[5] + 0.5).abs() < 1.0e-6);

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
