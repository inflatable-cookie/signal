use super::*;

pub(crate) fn build_runtime_offline_render_contract_preview() -> (
    RuntimeOfflineRenderContractPreview,
    RuntimePluginRecallHandoffSelection,
) {
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

    (preview, selection)
}
