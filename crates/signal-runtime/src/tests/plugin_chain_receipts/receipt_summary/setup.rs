use super::super::super::*;

pub(super) fn build_plugin_chain_receipt_runtime() -> SignalRuntime {
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
            graph_id: "graph:runtime:plugin-chain".into(),
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
            graph_id: "graph:runtime:plugin-chain".into(),
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
            graph_id: "graph:runtime:plugin-chain".into(),
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
        PluginSandboxLifecycleStage::SandboxEnsured,
        None,
    );
    runtime.record_plugin_sandbox_lifecycle(
        "sandbox-a",
        PluginSandboxLifecycleStage::InstancePrepared,
        Some(1),
    );
    runtime.record_plugin_sandbox_transport(
        "sandbox-a",
        "lease-a",
        "region-a",
        PluginSandboxTransportStage::Attached,
        Some(1),
        None,
    );

    runtime.record_recovery_cycle(
        "sandbox-b",
        RecoveryRestartIntent::CrashRecovery,
        StopReason::DegradedModeRecovery,
        Some(1),
    );
    runtime.record_plugin_sandbox_lifecycle(
        "sandbox-b",
        PluginSandboxLifecycleStage::SandboxRestarted,
        Some(1),
    );
    runtime.record_plugin_sandbox_lifecycle(
        "sandbox-b",
        PluginSandboxLifecycleStage::InstancePrepared,
        Some(2),
    );
    runtime.record_plugin_sandbox_transport(
        "sandbox-b",
        "lease-b",
        "region-b",
        PluginSandboxTransportStage::Attached,
        Some(2),
        None,
    );

    runtime
        .apply_plugin_node_render_batch(PluginNodeRenderBatch {
            graph_id: "graph:runtime:plugin-chain".into(),
            processing_epoch: 1,
            block_sequence: 1,
            renders: vec![
                PluginNodeRender {
                    node_id: "plugin-a".into(),
                    sandbox_id: "sandbox-a".into(),
                    output: AudioBuffer::new(
                        SampleRate(48_000),
                        ChannelLayout::Stereo,
                        FrameCount(4),
                    ),
                    latency_samples: 32,
                    tail_samples: 48,
                    bypassed: false,
                },
                PluginNodeRender {
                    node_id: "plugin-b".into(),
                    sandbox_id: "sandbox-b".into(),
                    output: AudioBuffer::new(
                        SampleRate(48_000),
                        ChannelLayout::Stereo,
                        FrameCount(4),
                    ),
                    latency_samples: 16,
                    tail_samples: 24,
                    bypassed: true,
                },
            ],
        })
        .expect("apply render batch");
    runtime
        .process_engine_block(
            1,
            1,
            AudioBuffer::new(SampleRate(48_000), ChannelLayout::Stereo, FrameCount(4)),
        )
        .expect("process block");
    runtime
}
