use signal_graph::{GraphNodeExecutionClass, GraphNodeTopologyRole, GraphStageSpec};
use signal_plugin::{PluginFeature, PluginFormat, PluginIoLayout};
use signal_primitives::{ChannelCount, ChannelLayout};
use signal_runtime::{
    GraphContractProjection, GraphNodeBufferContractProjection, GraphNodeContractProjection,
    GraphNodeProjection, GraphNodeTopologyProjection, GraphProjection, PluginBackedNodeBinding,
    PluginBackedNodeBindingProjection, RuntimePluginComplexIoSummary,
    RuntimePluginDiscoveredTypeRecord, RuntimeProjectionApi, SignalRuntime,
};

pub fn sample_complex_multi_output_record() -> RuntimePluginDiscoveredTypeRecord {
    RuntimePluginDiscoveredTypeRecord {
        plugin_type_id: "plugin:vst3:host-local-multiout".into(),
        plugin_id: "com.signal.host-local-multiout".into(),
        vendor: "Signal".into(),
        name: "Signal Host Local Multi Output".into(),
        format: PluginFormat::Vst3,
        version: Some("1.0.0".into()),
        features: vec![PluginFeature::Instrument, PluginFeature::Analyzer],
        default_io_layout: PluginIoLayout {
            audio_inputs: 0,
            audio_outputs: 6,
            midi_inputs: 1,
            midi_outputs: 0,
        },
        default_multichannel_io: signal_runtime::RuntimeMultichannelIoSummary::for_plugin_io(
            PluginIoLayout {
                audio_inputs: 0,
                audio_outputs: 6,
                midi_inputs: 1,
                midi_outputs: 0,
            },
        ),
        complex_io_summary: RuntimePluginComplexIoSummary::from_plugin_features_and_layout(
            &[PluginFeature::Instrument, PluginFeature::Analyzer],
            PluginIoLayout {
                audio_inputs: 0,
                audio_outputs: 6,
                midi_inputs: 1,
                midi_outputs: 0,
            },
        ),
        audio_bus_count: 1,
        parameter_count: 24,
        state_contract: signal_plugin::PluginStateContract {
            supports_snapshot: true,
            supports_reset: true,
            supports_bypass: true,
            exposes_latency: true,
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
    }
}

pub fn sample_complex_bus_fx_record() -> RuntimePluginDiscoveredTypeRecord {
    RuntimePluginDiscoveredTypeRecord {
        plugin_type_id: "plugin:vst3:host-local-bus-fx".into(),
        plugin_id: "com.signal.host-local-bus-fx".into(),
        vendor: "Signal".into(),
        name: "Signal Host Local Bus FX".into(),
        format: PluginFormat::Vst3,
        version: Some("1.0.0".into()),
        features: vec![PluginFeature::AudioEffect, PluginFeature::Utility],
        default_io_layout: PluginIoLayout {
            audio_inputs: 4,
            audio_outputs: 4,
            midi_inputs: 0,
            midi_outputs: 0,
        },
        default_multichannel_io: signal_runtime::RuntimeMultichannelIoSummary::for_plugin_io(
            PluginIoLayout {
                audio_inputs: 4,
                audio_outputs: 4,
                midi_inputs: 0,
                midi_outputs: 0,
            },
        ),
        complex_io_summary: RuntimePluginComplexIoSummary::from_plugin_features_and_layout(
            &[PluginFeature::AudioEffect, PluginFeature::Utility],
            PluginIoLayout {
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
    }
}

pub fn apply_public_complex_io_graph(runtime: &mut SignalRuntime, graph_id: &str) {
    runtime
        .apply_graph_projection(GraphProjection {
            graph_id: graph_id.into(),
            node_count: 3,
            nodes: vec![
                GraphNodeProjection {
                    node_id: "plugin-multiout".into(),
                    execution_class: GraphNodeExecutionClass::PluginBacked,
                    latency_samples: 32,
                    stages: vec![GraphStageSpec::Gain { linear: 0.95 }],
                },
                GraphNodeProjection {
                    node_id: "plugin-bus-fx".into(),
                    execution_class: GraphNodeExecutionClass::PluginBacked,
                    latency_samples: 16,
                    stages: vec![GraphStageSpec::HardClip { threshold: 0.8 }],
                },
                GraphNodeProjection {
                    node_id: "output-main".into(),
                    execution_class: GraphNodeExecutionClass::PureTransform,
                    latency_samples: 0,
                    stages: vec![GraphStageSpec::Gain { linear: 0.9 }],
                },
            ],
        })
        .expect("public local complex io graph should apply");
    runtime
        .apply_graph_contract_projection(GraphContractProjection {
            graph_id: graph_id.into(),
            contract_count: 3,
            nodes: vec![
                GraphNodeContractProjection {
                    node_id: "plugin-multiout".into(),
                    buffer_contract: GraphNodeBufferContractProjection {
                        input: signal_runtime::GraphNodeBusEndpointProjection {
                            bus_id: "main:in".into(),
                            channels: ChannelLayout::Stereo,
                        },
                        output: signal_runtime::GraphNodeBusEndpointProjection {
                            bus_id: "bus:surround".into(),
                            channels: ChannelLayout::Count(ChannelCount(6)),
                        },
                        ..GraphNodeBufferContractProjection::default()
                    },
                    topology: GraphNodeTopologyProjection {
                        role: Some(GraphNodeTopologyRole::TrackLane),
                        track_lane_id: Some("track:host-local:complex-io".into()),
                        bus_group_id: Some("mix:host-local:complex-io".into()),
                        console_group_id: None,
                        send_return_id: None,
                    },
                },
                GraphNodeContractProjection {
                    node_id: "plugin-bus-fx".into(),
                    buffer_contract: GraphNodeBufferContractProjection {
                        input: signal_runtime::GraphNodeBusEndpointProjection {
                            bus_id: "bus:surround".into(),
                            channels: ChannelLayout::Stereo,
                        },
                        output: signal_runtime::GraphNodeBusEndpointProjection {
                            bus_id: "bus:fx".into(),
                            channels: ChannelLayout::Stereo,
                        },
                        ..GraphNodeBufferContractProjection::default()
                    },
                    topology: GraphNodeTopologyProjection {
                        role: Some(GraphNodeTopologyRole::Bus),
                        track_lane_id: None,
                        bus_group_id: Some("mix:host-local:complex-io".into()),
                        console_group_id: None,
                        send_return_id: Some("fx:host-local:complex-io".into()),
                    },
                },
                GraphNodeContractProjection {
                    node_id: "output-main".into(),
                    buffer_contract: GraphNodeBufferContractProjection {
                        input: signal_runtime::GraphNodeBusEndpointProjection {
                            bus_id: "bus:fx".into(),
                            channels: ChannelLayout::Stereo,
                        },
                        output: signal_runtime::GraphNodeBusEndpointProjection {
                            bus_id: "main:out".into(),
                            channels: ChannelLayout::Stereo,
                        },
                        ..GraphNodeBufferContractProjection::default()
                    },
                    topology: GraphNodeTopologyProjection {
                        role: Some(GraphNodeTopologyRole::ConsoleNode),
                        track_lane_id: None,
                        bus_group_id: None,
                        console_group_id: Some("console:host-local:complex-io".into()),
                        send_return_id: None,
                    },
                },
            ],
        })
        .expect("public local complex io contracts should apply");
    runtime
        .apply_plugin_backed_node_bindings(PluginBackedNodeBindingProjection {
            graph_id: graph_id.into(),
            bindings: vec![
                PluginBackedNodeBinding {
                    node_id: "plugin-multiout".into(),
                    sandbox_id: "sandbox:host-local:multiout".into(),
                },
                PluginBackedNodeBinding {
                    node_id: "plugin-bus-fx".into(),
                    sandbox_id: "sandbox:host-local:bus-fx".into(),
                },
            ],
        })
        .expect("public local complex io bindings should apply");
}
