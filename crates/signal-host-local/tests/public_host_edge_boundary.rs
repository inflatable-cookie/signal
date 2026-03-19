use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use signal_graph::{
    synthetic_stereo_block, GraphExecutionLane, GraphNodeExecutionClass, GraphNodeTopologyRole,
    GraphStageSpec,
};
use signal_host_local::LocalRuntimeHost;
use signal_plugin::{EventPacketSummary, PluginFeature, PluginFormat, PluginIoLayout};
use signal_primitives::{AudioBuffer, ChannelCount, ChannelLayout, FrameCount, SampleRate};
use signal_runtime::{
    GraphContractProjection, GraphNodeBufferContractProjection, GraphNodeContractProjection,
    GraphNodeProjection, GraphNodeTopologyProjection, GraphProjection, PluginBackedNodeBinding,
    PluginBackedNodeBindingProjection, PluginFaultKind, PluginSandboxLifecycleStage,
    PluginSandboxSpec, PluginSandboxTransportStage, PluginScanRequest, RuntimeAuxiliaryPathKind,
    RuntimeBlockDeadlinePressure, RuntimeBusIntent, RuntimeBusRole, RuntimeCanonicalChannelLayout,
    RuntimeConfig, RuntimeConfigRequest, RuntimeDeferredServiceBackpressureSource,
    RuntimeDeferredServiceDecision, RuntimeDeferredServicePriorityBand,
    RuntimeDeferredServiceReason, RuntimeDeviceFaultBoundaryState, RuntimeDeviceRestartState,
    RuntimeDeviceSupervisionState, RuntimeError, RuntimeErrorKind, RuntimeExternalIoHealthState,
    RuntimeExternalIoLoopbackState, RuntimeExternalIoMonitoringState,
    RuntimeExternalIoMonitoringTapPoint, RuntimeExternalIoPrimaryRole,
    RuntimeHostClockDiscontinuityState, RuntimeHostClockDriftState, RuntimeHostDuplexMismatchState,
    RuntimeHostEndpointTopology, RuntimeInterruptionClass, RuntimeJackClientRole,
    RuntimeJackGraphCoordinationState, RuntimeJackGuardedCoordinationState,
    RuntimeJackTransportPosture, RuntimeLifecycleApi, RuntimeObservationApi,
    RuntimeOfflineRenderRequest, RuntimePluginAraContextSnapshot, RuntimePluginAraDocumentContext,
    RuntimePluginAraRegionContext, RuntimePluginAraSourceContext, RuntimePluginBusCapableFxClass,
    RuntimePluginComplexIoSummary, RuntimePluginDiscoveredTypeRecord, RuntimePluginHostPlatform,
    RuntimePluginIsolationOutcome, RuntimePluginParityBand, RuntimePluginPlacementPolicy,
    RuntimePluginPlacementRule, RuntimePluginPlacementRuleMatcher, RuntimePluginPresetDescriptor,
    RuntimePluginPresetOrigin, RuntimePluginRecallPortabilityClass, RuntimeProjectionApi,
    RuntimeRecordingCaptureKind, RuntimeRecordingCaptureStartRequest, RuntimeRecoveryState,
    RuntimeSecondaryInputAttachmentPolicy, RuntimeSecondaryInputContractProjection,
    RuntimeSecondaryInputFallbackOutcome, RuntimeSecondaryInputTargetKind, RuntimeSpatialBedClass,
    RuntimeSpatialExecutionMode, RuntimeSpatialExpandedFallbackOutcome,
    RuntimeSpatialFallbackOutcome, RuntimeSpatialMixPolicy, RuntimeSupervisorApi, SafeModeRequest,
    SignalRuntime,
};

fn apply_public_capture_graph(runtime: &mut SignalRuntime, graph_id: &str) {
    runtime
        .apply_graph_projection(GraphProjection {
            graph_id: graph_id.into(),
            node_count: 2,
            nodes: vec![
                GraphNodeProjection {
                    node_id: "inline".into(),
                    execution_class: GraphNodeExecutionClass::PureTransform,
                    latency_samples: 0,
                    stages: vec![GraphStageSpec::Gain { linear: 0.9 }],
                },
                GraphNodeProjection {
                    node_id: "latency".into(),
                    execution_class: GraphNodeExecutionClass::LatencyBearing,
                    latency_samples: 24,
                    stages: vec![GraphStageSpec::HardClip { threshold: 0.6 }],
                },
            ],
        })
        .expect("public host-edge capture graph should apply");
}

fn apply_public_render_graph(runtime: &mut SignalRuntime, graph_id: &str) {
    runtime
        .apply_graph_projection(GraphProjection {
            graph_id: graph_id.into(),
            node_count: 2,
            nodes: vec![
                GraphNodeProjection {
                    node_id: "offline-inline".into(),
                    execution_class: GraphNodeExecutionClass::PureTransform,
                    latency_samples: 0,
                    stages: vec![GraphStageSpec::Gain { linear: 0.85 }],
                },
                GraphNodeProjection {
                    node_id: "offline-latency".into(),
                    execution_class: GraphNodeExecutionClass::LatencyBearing,
                    latency_samples: 16,
                    stages: vec![GraphStageSpec::HardClip { threshold: 0.7 }],
                },
            ],
        })
        .expect("public host-edge render graph should apply");
}

fn apply_public_plugin_continuity_graph(
    runtime: &mut SignalRuntime,
    graph_id: &str,
    bindings: &[(&str, &str)],
) {
    runtime
        .apply_graph_projection(GraphProjection {
            graph_id: graph_id.into(),
            node_count: bindings.len(),
            nodes: bindings
                .iter()
                .map(|(node_id, _)| GraphNodeProjection {
                    node_id: (*node_id).into(),
                    execution_class: GraphNodeExecutionClass::PluginBacked,
                    latency_samples: 24,
                    stages: vec![GraphStageSpec::HardClip { threshold: 0.65 }],
                })
                .collect(),
        })
        .expect("public host-edge plugin continuity graph should apply");
    runtime
        .apply_graph_contract_projection(GraphContractProjection {
            graph_id: graph_id.into(),
            contract_count: bindings.len(),
            nodes: bindings
                .iter()
                .map(|(node_id, _)| GraphNodeContractProjection {
                    node_id: (*node_id).into(),
                    buffer_contract: GraphNodeBufferContractProjection::default(),
                    topology: GraphNodeTopologyProjection {
                        role: Some(GraphNodeTopologyRole::TrackLane),
                        track_lane_id: Some("track:host-local:plugin-continuity".into()),
                        bus_group_id: Some("mix:host-local".into()),
                        console_group_id: None,
                        send_return_id: None,
                    },
                })
                .collect(),
        })
        .expect("public host-edge plugin continuity contracts should apply");
    runtime
        .apply_plugin_backed_node_bindings(PluginBackedNodeBindingProjection {
            graph_id: graph_id.into(),
            bindings: bindings
                .iter()
                .map(|(node_id, sandbox_id)| PluginBackedNodeBinding {
                    node_id: (*node_id).into(),
                    sandbox_id: (*sandbox_id).into(),
                })
                .collect(),
        })
        .expect("public host-edge plugin continuity bindings should apply");
}

fn apply_public_multichannel_graph(runtime: &mut SignalRuntime, graph_id: &str) {
    runtime
        .apply_graph_projection(GraphProjection {
            graph_id: graph_id.into(),
            node_count: 2,
            nodes: vec![
                GraphNodeProjection {
                    node_id: "surround-track".into(),
                    execution_class: GraphNodeExecutionClass::PluginBacked,
                    latency_samples: 32,
                    stages: vec![GraphStageSpec::Gain { linear: 0.95 }],
                },
                GraphNodeProjection {
                    node_id: "analysis-send".into(),
                    execution_class: GraphNodeExecutionClass::PureTransform,
                    latency_samples: 0,
                    stages: vec![GraphStageSpec::HardClip { threshold: 0.75 }],
                },
            ],
        })
        .expect("public host-edge multichannel graph should apply");
    runtime
        .apply_graph_contract_projection(GraphContractProjection {
            graph_id: graph_id.into(),
            contract_count: 2,
            nodes: vec![
                GraphNodeContractProjection {
                    node_id: "surround-track".into(),
                    buffer_contract: GraphNodeBufferContractProjection {
                        input: signal_runtime::GraphNodeBusEndpointProjection {
                            bus_id: "main:in".into(),
                            channels: ChannelLayout::Stereo,
                        },
                        output: signal_runtime::GraphNodeBusEndpointProjection {
                            bus_id: "main:out".into(),
                            channels: ChannelLayout::Count(ChannelCount(6)),
                        },
                        ..GraphNodeBufferContractProjection::default()
                    },
                    topology: GraphNodeTopologyProjection {
                        role: Some(GraphNodeTopologyRole::TrackLane),
                        track_lane_id: Some("track:host-local:surround".into()),
                        bus_group_id: Some("mix:host-local:surround".into()),
                        console_group_id: None,
                        send_return_id: None,
                    },
                },
                GraphNodeContractProjection {
                    node_id: "analysis-send".into(),
                    buffer_contract: GraphNodeBufferContractProjection {
                        input: signal_runtime::GraphNodeBusEndpointProjection {
                            bus_id: "main:in".into(),
                            channels: ChannelLayout::Count(ChannelCount(6)),
                        },
                        output: signal_runtime::GraphNodeBusEndpointProjection {
                            bus_id: "main:out".into(),
                            channels: ChannelLayout::Count(ChannelCount(4)),
                        },
                        ..GraphNodeBufferContractProjection::default()
                    },
                    topology: GraphNodeTopologyProjection {
                        role: Some(GraphNodeTopologyRole::Send),
                        track_lane_id: Some("track:host-local:surround".into()),
                        bus_group_id: Some("mix:host-local:surround".into()),
                        console_group_id: None,
                        send_return_id: Some("send:return:host-local:analysis".into()),
                    },
                },
            ],
        })
        .expect("public host-edge multichannel contracts should apply");
}

fn apply_public_sidechain_graph(runtime: &mut SignalRuntime, graph_id: &str) {
    runtime
        .apply_graph_projection(GraphProjection {
            graph_id: graph_id.into(),
            node_count: 3,
            nodes: vec![
                GraphNodeProjection {
                    node_id: "program-track".into(),
                    execution_class: GraphNodeExecutionClass::Stateful,
                    latency_samples: 0,
                    stages: vec![GraphStageSpec::Gain { linear: 0.92 }],
                },
                GraphNodeProjection {
                    node_id: "kick-sidechain".into(),
                    execution_class: GraphNodeExecutionClass::PureTransform,
                    latency_samples: 0,
                    stages: vec![GraphStageSpec::Gain { linear: 0.7 }],
                },
                GraphNodeProjection {
                    node_id: "compressor".into(),
                    execution_class: GraphNodeExecutionClass::PluginBacked,
                    latency_samples: 24,
                    stages: vec![GraphStageSpec::HardClip { threshold: 0.78 }],
                },
            ],
        })
        .expect("public host-edge sidechain graph should apply");
    runtime
        .apply_graph_contract_projection(GraphContractProjection {
            graph_id: graph_id.into(),
            contract_count: 3,
            nodes: vec![
                GraphNodeContractProjection {
                    node_id: "program-track".into(),
                    buffer_contract: GraphNodeBufferContractProjection {
                        input: signal_runtime::GraphNodeBusEndpointProjection {
                            bus_id: "main:in".into(),
                            channels: ChannelLayout::Stereo,
                        },
                        output: signal_runtime::GraphNodeBusEndpointProjection {
                            bus_id: "bus:program".into(),
                            channels: ChannelLayout::Stereo,
                        },
                        ..GraphNodeBufferContractProjection::default()
                    },
                    topology: GraphNodeTopologyProjection {
                        role: Some(GraphNodeTopologyRole::TrackLane),
                        track_lane_id: Some("track:host-local:sidechain".into()),
                        bus_group_id: Some("mix:host-local:sidechain".into()),
                        console_group_id: None,
                        send_return_id: None,
                    },
                },
                GraphNodeContractProjection {
                    node_id: "kick-sidechain".into(),
                    buffer_contract: GraphNodeBufferContractProjection {
                        input: signal_runtime::GraphNodeBusEndpointProjection {
                            bus_id: "main:in".into(),
                            channels: ChannelLayout::Stereo,
                        },
                        output: signal_runtime::GraphNodeBusEndpointProjection {
                            bus_id: "bus:sidechain:kick".into(),
                            channels: ChannelLayout::Mono,
                        },
                        ..GraphNodeBufferContractProjection::default()
                    },
                    topology: GraphNodeTopologyProjection {
                        role: Some(GraphNodeTopologyRole::Utility),
                        track_lane_id: None,
                        bus_group_id: None,
                        console_group_id: None,
                        send_return_id: None,
                    },
                },
                GraphNodeContractProjection {
                    node_id: "compressor".into(),
                    buffer_contract: GraphNodeBufferContractProjection {
                        input: signal_runtime::GraphNodeBusEndpointProjection {
                            bus_id: "bus:program".into(),
                            channels: ChannelLayout::Stereo,
                        },
                        output: signal_runtime::GraphNodeBusEndpointProjection {
                            bus_id: "main:out".into(),
                            channels: ChannelLayout::Stereo,
                        },
                        secondary_input: Some(RuntimeSecondaryInputContractProjection {
                            source_kind:
                                signal_runtime::RuntimeSecondaryInputSourceKind::NodeOutput,
                            source_id: "kick-sidechain".into(),
                            source_bus_id: Some("bus:sidechain:kick".into()),
                            target_bus_id: "plugin:compressor:sidechain".into(),
                            attachment_policy: RuntimeSecondaryInputAttachmentPolicy::Required,
                            fallback_outcome:
                                RuntimeSecondaryInputFallbackOutcome::SafeModeDegradation,
                        }),
                        ..GraphNodeBufferContractProjection::default()
                    },
                    topology: GraphNodeTopologyProjection {
                        role: Some(GraphNodeTopologyRole::TrackLane),
                        track_lane_id: Some("track:host-local:sidechain".into()),
                        bus_group_id: Some("mix:host-local:sidechain".into()),
                        console_group_id: None,
                        send_return_id: None,
                    },
                },
            ],
        })
        .expect("public host-edge sidechain contracts should apply");
    runtime
        .apply_plugin_backed_node_bindings(PluginBackedNodeBindingProjection {
            graph_id: graph_id.into(),
            bindings: vec![PluginBackedNodeBinding {
                node_id: "compressor".into(),
                sandbox_id: "sandbox:host-local:sidechain".into(),
            }],
        })
        .expect("public host-edge sidechain bindings should apply");
}

fn apply_public_multi_bus_graph(runtime: &mut SignalRuntime, graph_id: &str) {
    runtime
        .apply_graph_projection(GraphProjection {
            graph_id: graph_id.into(),
            node_count: 5,
            nodes: vec![
                GraphNodeProjection {
                    node_id: "track-input".into(),
                    execution_class: GraphNodeExecutionClass::Stateful,
                    latency_samples: 0,
                    stages: vec![GraphStageSpec::Gain { linear: 0.8 }],
                },
                GraphNodeProjection {
                    node_id: "bus-dry".into(),
                    execution_class: GraphNodeExecutionClass::Stateful,
                    latency_samples: 0,
                    stages: vec![GraphStageSpec::Gain { linear: 0.95 }],
                },
                GraphNodeProjection {
                    node_id: "send-fx".into(),
                    execution_class: GraphNodeExecutionClass::Stateful,
                    latency_samples: 0,
                    stages: vec![GraphStageSpec::Gain { linear: 0.4 }],
                },
                GraphNodeProjection {
                    node_id: "return-fx".into(),
                    execution_class: GraphNodeExecutionClass::LatencyBearing,
                    latency_samples: 16,
                    stages: vec![GraphStageSpec::HardClip { threshold: 0.82 }],
                },
                GraphNodeProjection {
                    node_id: "output-main".into(),
                    execution_class: GraphNodeExecutionClass::PureTransform,
                    latency_samples: 0,
                    stages: vec![GraphStageSpec::StereoBalance { balance: -0.1 }],
                },
            ],
        })
        .expect("public host-edge multi-bus graph should apply");
    runtime
        .apply_graph_contract_projection(GraphContractProjection {
            graph_id: graph_id.into(),
            contract_count: 5,
            nodes: vec![
                GraphNodeContractProjection {
                    node_id: "track-input".into(),
                    buffer_contract: GraphNodeBufferContractProjection {
                        input: signal_runtime::GraphNodeBusEndpointProjection {
                            bus_id: "main:in".into(),
                            channels: ChannelLayout::Stereo,
                        },
                        output: signal_runtime::GraphNodeBusEndpointProjection {
                            bus_id: "bus:track:lead".into(),
                            channels: ChannelLayout::Stereo,
                        },
                        ..GraphNodeBufferContractProjection::default()
                    },
                    topology: GraphNodeTopologyProjection {
                        role: Some(GraphNodeTopologyRole::TrackLane),
                        track_lane_id: Some("track:host-local:multi-bus".into()),
                        bus_group_id: Some("mix:tracks".into()),
                        console_group_id: None,
                        send_return_id: None,
                    },
                },
                GraphNodeContractProjection {
                    node_id: "bus-dry".into(),
                    buffer_contract: GraphNodeBufferContractProjection {
                        input: signal_runtime::GraphNodeBusEndpointProjection {
                            bus_id: "bus:track:lead".into(),
                            channels: ChannelLayout::Stereo,
                        },
                        output: signal_runtime::GraphNodeBusEndpointProjection {
                            bus_id: "bus:mix:master".into(),
                            channels: ChannelLayout::Stereo,
                        },
                        ..GraphNodeBufferContractProjection::default()
                    },
                    topology: GraphNodeTopologyProjection {
                        role: Some(GraphNodeTopologyRole::Bus),
                        track_lane_id: None,
                        bus_group_id: Some("mix:master".into()),
                        console_group_id: None,
                        send_return_id: None,
                    },
                },
                GraphNodeContractProjection {
                    node_id: "send-fx".into(),
                    buffer_contract: GraphNodeBufferContractProjection {
                        input: signal_runtime::GraphNodeBusEndpointProjection {
                            bus_id: "bus:track:lead".into(),
                            channels: ChannelLayout::Stereo,
                        },
                        output: signal_runtime::GraphNodeBusEndpointProjection {
                            bus_id: "bus:fx:plate".into(),
                            channels: ChannelLayout::Stereo,
                        },
                        ..GraphNodeBufferContractProjection::default()
                    },
                    topology: GraphNodeTopologyProjection {
                        role: Some(GraphNodeTopologyRole::Send),
                        track_lane_id: None,
                        bus_group_id: None,
                        console_group_id: None,
                        send_return_id: Some("fx:plate".into()),
                    },
                },
                GraphNodeContractProjection {
                    node_id: "return-fx".into(),
                    buffer_contract: GraphNodeBufferContractProjection {
                        input: signal_runtime::GraphNodeBusEndpointProjection {
                            bus_id: "bus:fx:plate".into(),
                            channels: ChannelLayout::Stereo,
                        },
                        output: signal_runtime::GraphNodeBusEndpointProjection {
                            bus_id: "bus:mix:master".into(),
                            channels: ChannelLayout::Stereo,
                        },
                        ..GraphNodeBufferContractProjection::default()
                    },
                    topology: GraphNodeTopologyProjection {
                        role: Some(GraphNodeTopologyRole::Return),
                        track_lane_id: None,
                        bus_group_id: None,
                        console_group_id: None,
                        send_return_id: Some("fx:plate".into()),
                    },
                },
                GraphNodeContractProjection {
                    node_id: "output-main".into(),
                    buffer_contract: GraphNodeBufferContractProjection {
                        input: signal_runtime::GraphNodeBusEndpointProjection {
                            bus_id: "bus:mix:master".into(),
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
                        console_group_id: Some("console:host-local:main".into()),
                        send_return_id: None,
                    },
                },
            ],
        })
        .expect("public host-edge multi-bus contracts should apply");
}

fn sample_complex_multi_output_record() -> RuntimePluginDiscoveredTypeRecord {
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
        summary: "local complex multi-output instrument".into(),
    }
}

fn sample_complex_bus_fx_record() -> RuntimePluginDiscoveredTypeRecord {
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
        summary: "local bus-capable fx".into(),
    }
}

fn apply_public_complex_io_graph(runtime: &mut SignalRuntime, graph_id: &str) {
    runtime
        .apply_graph_projection(GraphProjection {
            graph_id: graph_id.into(),
            node_count: 2,
            nodes: vec![
                GraphNodeProjection {
                    node_id: "plugin-multiout".into(),
                    execution_class: GraphNodeExecutionClass::PluginBacked,
                    latency_samples: 24,
                    stages: vec![GraphStageSpec::HardClip { threshold: 0.7 }],
                },
                GraphNodeProjection {
                    node_id: "plugin-bus-fx".into(),
                    execution_class: GraphNodeExecutionClass::PluginBacked,
                    latency_samples: 12,
                    stages: vec![GraphStageSpec::HardClip { threshold: 0.5 }],
                },
            ],
        })
        .expect("public local complex io graph should apply");
    runtime
        .apply_graph_contract_projection(GraphContractProjection {
            graph_id: graph_id.into(),
            contract_count: 2,
            nodes: vec![
                GraphNodeContractProjection {
                    node_id: "plugin-multiout".into(),
                    buffer_contract: GraphNodeBufferContractProjection::default(),
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
                    buffer_contract: GraphNodeBufferContractProjection::default(),
                    topology: GraphNodeTopologyProjection {
                        role: Some(GraphNodeTopologyRole::TrackLane),
                        track_lane_id: Some("track:host-local:complex-io".into()),
                        bus_group_id: Some("mix:host-local:complex-io".into()),
                        console_group_id: None,
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
    runtime.record_plugin_sandbox_spec(&PluginSandboxSpec {
        sandbox_id: "sandbox:host-local:multiout".into(),
        plugin_format: PluginFormat::Vst3,
        plugin_type_id: Some("plugin:vst3:host-local-multiout".into()),
    });
    runtime.record_plugin_sandbox_spec(&PluginSandboxSpec {
        sandbox_id: "sandbox:host-local:bus-fx".into(),
        plugin_format: PluginFormat::Vst3,
        plugin_type_id: Some("plugin:vst3:host-local-bus-fx".into()),
    });
    runtime.record_plugin_sandbox_lifecycle(
        "sandbox:host-local:multiout",
        PluginSandboxLifecycleStage::InstancePrepared,
        Some(1),
    );
    runtime.record_plugin_sandbox_transport(
        "sandbox:host-local:multiout",
        "lease-host-local-multiout",
        "region-host-local-multiout",
        PluginSandboxTransportStage::Attached,
        Some(1),
        Some("host local complex io multiout attached".into()),
    );
    runtime.record_plugin_sandbox_lifecycle(
        "sandbox:host-local:bus-fx",
        PluginSandboxLifecycleStage::SandboxRestarted,
        Some(2),
    );
    runtime.record_plugin_sandbox_lifecycle(
        "sandbox:host-local:bus-fx",
        PluginSandboxLifecycleStage::InstancePrepared,
        Some(2),
    );
    runtime.record_plugin_sandbox_transport(
        "sandbox:host-local:bus-fx",
        "lease-host-local-bus-fx",
        "region-host-local-bus-fx",
        PluginSandboxTransportStage::Attached,
        Some(2),
        Some("host local complex io bus fx attached".into()),
    );
}

fn apply_public_spatial_graph(runtime: &mut SignalRuntime, graph_id: &str) {
    runtime
        .apply_graph_projection(GraphProjection {
            graph_id: graph_id.into(),
            node_count: 2,
            nodes: vec![
                GraphNodeProjection {
                    node_id: "spatial-stereo".into(),
                    execution_class: GraphNodeExecutionClass::PluginBacked,
                    latency_samples: 12,
                    stages: vec![GraphStageSpec::StereoBalance { balance: -0.2 }],
                },
                GraphNodeProjection {
                    node_id: "spatial-surround".into(),
                    execution_class: GraphNodeExecutionClass::PluginBacked,
                    latency_samples: 20,
                    stages: vec![GraphStageSpec::StereoBalance { balance: 0.35 }],
                },
            ],
        })
        .expect("public local spatial graph should apply");
    runtime
        .apply_graph_contract_projection(GraphContractProjection {
            graph_id: graph_id.into(),
            contract_count: 2,
            nodes: vec![
                GraphNodeContractProjection {
                    node_id: "spatial-stereo".into(),
                    buffer_contract: GraphNodeBufferContractProjection {
                        input: signal_runtime::GraphNodeBusEndpointProjection {
                            bus_id: "main:in".into(),
                            channels: ChannelLayout::Stereo,
                        },
                        output: signal_runtime::GraphNodeBusEndpointProjection {
                            bus_id: "bus:spatial:stereo".into(),
                            channels: ChannelLayout::Stereo,
                        },
                        ..GraphNodeBufferContractProjection::default()
                    },
                    topology: GraphNodeTopologyProjection {
                        role: Some(GraphNodeTopologyRole::TrackLane),
                        track_lane_id: Some("track:host-local:spatial-stereo".into()),
                        bus_group_id: Some("bus:host-local:spatial-stereo".into()),
                        console_group_id: None,
                        send_return_id: None,
                    },
                },
                GraphNodeContractProjection {
                    node_id: "spatial-surround".into(),
                    buffer_contract: GraphNodeBufferContractProjection {
                        input: signal_runtime::GraphNodeBusEndpointProjection {
                            bus_id: "main:surround-in".into(),
                            channels: ChannelLayout::Count(ChannelCount(6)),
                        },
                        output: signal_runtime::GraphNodeBusEndpointProjection {
                            bus_id: "bus:spatial:surround".into(),
                            channels: ChannelLayout::Count(ChannelCount(6)),
                        },
                        ..GraphNodeBufferContractProjection::default()
                    },
                    topology: GraphNodeTopologyProjection {
                        role: Some(GraphNodeTopologyRole::TrackLane),
                        track_lane_id: Some("track:host-local:spatial-surround".into()),
                        bus_group_id: Some("bus:host-local:spatial-surround".into()),
                        console_group_id: None,
                        send_return_id: None,
                    },
                },
            ],
        })
        .expect("public local spatial contracts should apply");
    runtime
        .apply_plugin_backed_node_bindings(PluginBackedNodeBindingProjection {
            graph_id: graph_id.into(),
            bindings: vec![
                PluginBackedNodeBinding {
                    node_id: "spatial-stereo".into(),
                    sandbox_id: "sandbox:host-local:spatial-stereo".into(),
                },
                PluginBackedNodeBinding {
                    node_id: "spatial-surround".into(),
                    sandbox_id: "sandbox:host-local:spatial-surround".into(),
                },
            ],
        })
        .expect("public local spatial bindings should apply");
    runtime.record_plugin_sandbox_lifecycle(
        "sandbox:host-local:spatial-stereo",
        PluginSandboxLifecycleStage::InstancePrepared,
        Some(1),
    );
    runtime.record_plugin_sandbox_lifecycle(
        "sandbox:host-local:spatial-surround",
        PluginSandboxLifecycleStage::InstancePrepared,
        Some(1),
    );
}

fn public_local_media_fixture_path(label: &str) -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock should be monotonic enough for test files")
        .as_nanos();
    std::env::temp_dir().join(format!(
        "signal-host-local-public-media-{label}-{}-{unique}.wav",
        std::process::id()
    ))
}

fn write_public_test_wav(path: &Path) {
    let channels = 1u16;
    let sample_rate = 48_000u32;
    let bits_per_sample = 16u16;
    let frame_count = 128u32;
    let block_align = channels * (bits_per_sample / 8);
    let byte_rate = sample_rate * block_align as u32;
    let data_size = frame_count * block_align as u32;
    let riff_size = 36 + data_size;
    let mut bytes = Vec::with_capacity((44 + data_size) as usize);
    bytes.extend_from_slice(b"RIFF");
    bytes.extend_from_slice(&riff_size.to_le_bytes());
    bytes.extend_from_slice(b"WAVE");
    bytes.extend_from_slice(b"fmt ");
    bytes.extend_from_slice(&16u32.to_le_bytes());
    bytes.extend_from_slice(&1u16.to_le_bytes());
    bytes.extend_from_slice(&channels.to_le_bytes());
    bytes.extend_from_slice(&sample_rate.to_le_bytes());
    bytes.extend_from_slice(&byte_rate.to_le_bytes());
    bytes.extend_from_slice(&block_align.to_le_bytes());
    bytes.extend_from_slice(&bits_per_sample.to_le_bytes());
    bytes.extend_from_slice(b"data");
    bytes.extend_from_slice(&data_size.to_le_bytes());
    for index in 0..frame_count {
        let sample =
            (((index as f32 / (frame_count - 1) as f32) * 2.0) - 1.0) * i16::MAX as f32 * 0.5;
        bytes.extend_from_slice(&(sample as i16).to_le_bytes());
    }
    fs::write(path, bytes).expect("public local media fixture should be written");
}

fn write_public_transient_test_wav(path: &Path) {
    let channels = 1u16;
    let sample_rate = 48_000u32;
    let bits_per_sample = 16u16;
    let frame_count = 48_000u32;
    let block_align = channels * (bits_per_sample / 8);
    let byte_rate = sample_rate * block_align as u32;
    let data_size = frame_count * block_align as u32;
    let riff_size = 36 + data_size;
    let mut bytes = Vec::with_capacity((44 + data_size) as usize);
    bytes.extend_from_slice(b"RIFF");
    bytes.extend_from_slice(&riff_size.to_le_bytes());
    bytes.extend_from_slice(b"WAVE");
    bytes.extend_from_slice(b"fmt ");
    bytes.extend_from_slice(&16u32.to_le_bytes());
    bytes.extend_from_slice(&1u16.to_le_bytes());
    bytes.extend_from_slice(&channels.to_le_bytes());
    bytes.extend_from_slice(&sample_rate.to_le_bytes());
    bytes.extend_from_slice(&byte_rate.to_le_bytes());
    bytes.extend_from_slice(&block_align.to_le_bytes());
    bytes.extend_from_slice(&bits_per_sample.to_le_bytes());
    bytes.extend_from_slice(b"data");
    bytes.extend_from_slice(&data_size.to_le_bytes());
    for index in 0..frame_count {
        let sample = if index % 6_000 == 0 { i16::MAX } else { 0 };
        bytes.extend_from_slice(&sample.to_le_bytes());
    }
    fs::write(path, bytes).expect("public local transient media fixture should be written");
}

fn record_public_plugin_sandbox_ready(
    runtime: &mut SignalRuntime,
    sandbox_id: &str,
    plugin_format: PluginFormat,
    plugin_type_id: &str,
    epoch: u64,
) {
    runtime.record_plugin_sandbox_spec(&PluginSandboxSpec {
        sandbox_id: sandbox_id.into(),
        plugin_format,
        plugin_type_id: Some(plugin_type_id.into()),
    });
    runtime.record_plugin_sandbox_lifecycle(
        sandbox_id,
        PluginSandboxLifecycleStage::InstancePrepared,
        Some(epoch),
    );
    runtime.record_plugin_sandbox_transport(
        sandbox_id,
        &format!("lease-{sandbox_id}"),
        &format!("region-{sandbox_id}"),
        PluginSandboxTransportStage::Attached,
        Some(epoch),
        None,
    );
}

fn sample_host_preset_descriptor() -> RuntimePluginPresetDescriptor {
    RuntimePluginPresetDescriptor {
        preset_id: Some("preset:user:local-lead".into()),
        label: Some("Local Lead".into()),
        origin: RuntimePluginPresetOrigin::User,
        summary: "local host preset descriptor".into(),
    }
}

fn sample_host_ara_context() -> RuntimePluginAraContextSnapshot {
    RuntimePluginAraContextSnapshot {
        portability_class: RuntimePluginRecallPortabilityClass::ContextOnly,
        document_context: Some(RuntimePluginAraDocumentContext {
            document_id: "doc:host-local".into(),
            display_label: Some("Song".into()),
            summary: "local host ara document".into(),
        }),
        source_context: Some(RuntimePluginAraSourceContext {
            source_id: "source:take-01".into(),
            display_label: Some("Take 01".into()),
            summary: "local host ara source".into(),
        }),
        region_context: Some(RuntimePluginAraRegionContext {
            region_id: "region:chorus".into(),
            display_label: Some("Chorus".into()),
            timeline_start_samples: Some(2_048),
            duration_samples: Some(8_192),
            summary: "local host ara region".into(),
        }),
        summary: "local host ara context".into(),
    }
}

#[test]
fn local_shared_host_edge_is_consumable_without_private_helpers() {
    let runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
    let mut host = LocalRuntimeHost::new(runtime);

    host.start_plugin_scan(PluginScanRequest {
        roots: vec!["~/Library/Audio/Plug-Ins/CLAP".into()],
        formats: vec![PluginFormat::Clap],
    })
    .expect("public host-edge scan should succeed");
    host.ensure_plugin_sandbox(PluginSandboxSpec {
        sandbox_id: "public-host-edge-local".into(),
        plugin_format: PluginFormat::Clap,
        plugin_type_id: None,
    })
    .expect("public host-edge sandbox ensure should succeed");

    let report = host.supervisor_report();
    assert_eq!(report.observation.plugin_discovery_snapshot.scan_count, 1);
    assert_eq!(
        report
            .observation
            .plugin_discovery_snapshot
            .discovered_type_count,
        2
    );
    assert_eq!(
        report.observation.plugin_lifecycle_snapshot.sandboxes.len(),
        1
    );
    assert_eq!(
        report.observation.plugin_lifecycle_snapshot.sandboxes[0].plugin_format,
        Some(PluginFormat::Clap)
    );
    assert_eq!(
        report.observation.fault_status.recovery_state,
        RuntimeRecoveryState::Steady
    );
    assert_eq!(
        report.observation.interruption_summary.class,
        RuntimeInterruptionClass::Steady
    );
    assert_eq!(
        report.observation.fault_diagnostic_receipt.primary_family,
        None
    );

    let rendered = report.render_json();
    assert!(rendered.contains("\"fault_status\":{"));
    assert!(rendered.contains("\"fault_diagnostic_receipt\":{"));
    assert!(rendered.contains("\"interruption_summary\":{"));
    assert!(rendered.contains("\"recording_capture_snapshot\":{"));
    assert!(rendered.contains("\"plugin_discovery_snapshot\":{"));
    assert!(rendered.contains("\"plugin_type_id\":\"plugin:clap:default\""));
    assert!(rendered.contains("\"event_stream\":"));
}

#[test]
fn local_shared_host_edge_exports_resumable_recording_checkpoint_truth() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
    runtime
        .handshake(signal_runtime::HandshakeRequest {
            client_version: "public-host-edge-recording".into(),
            anticipative_preferred: true,
            max_sample_rate_hint: Some(96_000),
        })
        .unwrap();
    runtime
        .configure(RuntimeConfigRequest::new(48_000, 512))
        .unwrap();
    apply_public_capture_graph(&mut runtime, "graph:host-local:recording");
    runtime.start().unwrap();
    runtime
        .start_recording_capture(RuntimeRecordingCaptureStartRequest {
            capture_kind: RuntimeRecordingCaptureKind::Audio,
            take_id: "take:local:resumable".into(),
            track_id: "track:local:resumable".into(),
            start_samples: 2_048,
            capture_path: std::env::temp_dir()
                .join("signal-local-host-recording-resumable.wav")
                .display()
                .to_string(),
        })
        .unwrap();
    runtime
        .process_engine_block(
            1,
            1,
            synthetic_stereo_block(SampleRate(48_000), FrameCount(8), 61),
        )
        .unwrap();
    runtime
        .set_safe_mode(SafeModeRequest { enabled: true })
        .unwrap();

    let host = LocalRuntimeHost::new(runtime);
    let report = host.supervisor_report();
    assert_eq!(
        report
            .observation
            .recording_capture_snapshot
            .active_checkpoint
            .as_ref()
            .map(|checkpoint| checkpoint.interruption_class),
        Some(RuntimeInterruptionClass::Resumable)
    );

    let rendered = report.render_json();
    assert!(rendered.contains("\"recording_capture_snapshot\":{"));
    assert!(rendered.contains("\"interruption_class\":\"Resumable\""));
}

#[test]
fn local_shared_host_edge_exports_plugin_placement_and_shared_boundary_continuity_truth() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
    runtime
        .handshake(signal_runtime::HandshakeRequest {
            client_version: "public-host-edge-plugin-continuity".into(),
            anticipative_preferred: true,
            max_sample_rate_hint: Some(96_000),
        })
        .unwrap();
    runtime
        .configure(RuntimeConfigRequest::new(48_000, 512))
        .unwrap();
    runtime
        .apply_plugin_placement_policy(RuntimePluginPlacementPolicy {
            default_outcome: RuntimePluginIsolationOutcome::IsolatedSandbox,
            rules: vec![RuntimePluginPlacementRule {
                rule_id: "share-verified-clap".into(),
                matcher: RuntimePluginPlacementRuleMatcher::PluginTypeId(
                    "plugin://host-local-shared".into(),
                ),
                outcome: RuntimePluginIsolationOutcome::SharedSandbox,
                sandbox_group_key: Some("shared:host-local".into()),
            }],
        })
        .unwrap();
    apply_public_plugin_continuity_graph(
        &mut runtime,
        "graph:host-local:plugin-continuity",
        &[
            ("plugin-a", "sandbox-shared"),
            ("plugin-b", "sandbox-shared"),
            ("plugin-c", "sandbox-isolated"),
        ],
    );
    record_public_plugin_sandbox_ready(
        &mut runtime,
        "sandbox-shared",
        PluginFormat::Clap,
        "plugin://host-local-shared",
        1,
    );
    record_public_plugin_sandbox_ready(
        &mut runtime,
        "sandbox-isolated",
        PluginFormat::Clap,
        "plugin://host-local-isolated",
        1,
    );
    runtime.record_plugin_sandbox_fault(
        "sandbox-shared",
        PluginFaultKind::Crash,
        "local shared crash",
        Some(2),
    );
    runtime.record_plugin_sandbox_fault(
        "sandbox-shared",
        PluginFaultKind::Timeout,
        "local shared timeout",
        Some(3),
    );

    let host = LocalRuntimeHost::new(runtime);
    let report = host.supervisor_report();
    let shared = report
        .observation
        .plugin_lifecycle_snapshot
        .sandboxes
        .iter()
        .find(|sandbox| sandbox.sandbox_id == "sandbox-shared")
        .expect("shared host-local boundary should be visible");
    assert_eq!(
        shared.placement_outcome,
        RuntimePluginIsolationOutcome::SharedSandbox
    );
    assert_eq!(
        shared.placement_rule_id.as_deref(),
        Some("share-verified-clap")
    );
    assert_eq!(shared.sandbox_group_key, "shared:host-local");
    assert_eq!(shared.shared_boundary_member_count, 2);
    assert_eq!(shared.continuity_class, RuntimeInterruptionClass::Terminal);
    assert!(!shared.rebindable);
    let isolated = report
        .observation
        .plugin_lifecycle_snapshot
        .sandboxes
        .iter()
        .find(|sandbox| sandbox.sandbox_id == "sandbox-isolated")
        .expect("isolated host-local boundary should remain visible");
    assert_eq!(isolated.continuity_class, RuntimeInterruptionClass::Steady);

    let rendered = report.render_json();
    assert!(rendered.contains("\"plugin_lifecycle_snapshot\":{"));
    assert!(rendered.contains("\"placement_outcome\":\"SharedSandbox\""));
    assert!(rendered.contains("\"sandbox_group_key\":\"shared:host-local\""));
    assert!(rendered.contains("\"shared_boundary_member_count\":2"));
    assert!(rendered.contains("\"continuity_class\":\"Terminal\""));
}

#[test]
fn local_shared_host_edge_exports_runtime_vst3_baseline_truth() {
    let runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
    let mut host = LocalRuntimeHost::new(runtime);

    host.start_plugin_scan(PluginScanRequest {
        roots: vec!["~/Library/Audio/Plug-Ins/VST3".into()],
        formats: vec![PluginFormat::Vst3],
    })
    .expect("public local vst3 scan should succeed");
    host.ensure_plugin_sandbox(PluginSandboxSpec {
        sandbox_id: "public-host-edge-local-vst3".into(),
        plugin_format: PluginFormat::Vst3,
        plugin_type_id: Some("plugin:vst3:instrument".into()),
    })
    .expect("public local vst3 sandbox ensure should succeed");

    let report = host.supervisor_report();
    assert_eq!(
        report
            .observation
            .plugin_discovery_snapshot
            .discovered_type_count,
        2
    );
    assert_eq!(
        report
            .observation
            .plugin_discovery_snapshot
            .last_scan
            .as_ref()
            .map(|scan| scan.formats.clone()),
        Some(vec![PluginFormat::Vst3])
    );
    assert!(report
        .observation
        .plugin_discovery_snapshot
        .discovered_types
        .iter()
        .any(|plugin| plugin.plugin_type_id == "plugin:vst3:instrument"
            && plugin.format == PluginFormat::Vst3));
    let sandbox = report
        .observation
        .plugin_lifecycle_snapshot
        .sandboxes
        .iter()
        .find(|sandbox| sandbox.sandbox_id == "public-host-edge-local-vst3")
        .expect("public local vst3 sandbox should be exported");
    assert_eq!(sandbox.plugin_format, Some(PluginFormat::Vst3));
    assert_eq!(
        sandbox.lifecycle_stage,
        Some(PluginSandboxLifecycleStage::TransportAttached)
    );
    assert_eq!(
        sandbox.transport_stage,
        Some(PluginSandboxTransportStage::Attached)
    );
    assert_eq!(sandbox.readiness_state.as_deref(), Some("Ready"));

    let rendered = report.render_json();
    assert!(rendered.contains("\"plugin_type_id\":\"plugin:vst3:instrument\""));
    assert!(rendered.contains("\"formats\":[\"Vst3\"]"));
}

#[test]
fn local_shared_host_edge_exports_runtime_au_baseline_truth() {
    let runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
    let mut host = LocalRuntimeHost::new(runtime);

    host.start_plugin_scan(PluginScanRequest {
        roots: vec!["~/Library/Audio/Plug-Ins/Components".into()],
        formats: vec![PluginFormat::Au],
    })
    .expect("public local au scan should succeed");
    host.ensure_plugin_sandbox(PluginSandboxSpec {
        sandbox_id: "public-host-edge-local-au".into(),
        plugin_format: PluginFormat::Au,
        plugin_type_id: Some("plugin:au:instrument".into()),
    })
    .expect("public local au sandbox ensure should succeed");

    let report = host.supervisor_report();
    assert_eq!(
        report
            .observation
            .plugin_discovery_snapshot
            .discovered_type_count,
        2
    );
    assert_eq!(
        report
            .observation
            .plugin_discovery_snapshot
            .last_scan
            .as_ref()
            .map(|scan| scan.formats.clone()),
        Some(vec![PluginFormat::Au])
    );
    assert!(report
        .observation
        .plugin_discovery_snapshot
        .discovered_types
        .iter()
        .any(|plugin| plugin.plugin_type_id == "plugin:au:instrument"
            && plugin.format == PluginFormat::Au));
    let sandbox = report
        .observation
        .plugin_lifecycle_snapshot
        .sandboxes
        .iter()
        .find(|sandbox| sandbox.sandbox_id == "public-host-edge-local-au")
        .expect("public local au sandbox should be exported");
    assert_eq!(sandbox.plugin_format, Some(PluginFormat::Au));
    assert_eq!(
        sandbox.lifecycle_stage,
        Some(PluginSandboxLifecycleStage::TransportAttached)
    );
    assert_eq!(
        sandbox.transport_stage,
        Some(PluginSandboxTransportStage::Attached)
    );
    assert_eq!(sandbox.readiness_state.as_deref(), Some("Ready"));

    let rendered = report.render_json();
    assert!(rendered.contains("\"plugin_type_id\":\"plugin:au:instrument\""));
    assert!(rendered.contains("\"formats\":[\"Au\"]"));
}

#[test]
fn local_shared_host_edge_exports_runtime_cross_adapter_parity_truth() {
    let runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
    let mut host = LocalRuntimeHost::new(runtime);

    host.start_plugin_scan(PluginScanRequest {
        roots: vec![
            "~/.clap".into(),
            "~/.vst3".into(),
            "~/Library/Audio/Plug-Ins/Components".into(),
        ],
        formats: vec![PluginFormat::Clap, PluginFormat::Vst3, PluginFormat::Au],
    })
    .expect("public local parity scan should succeed");
    host.ensure_plugin_sandbox(PluginSandboxSpec {
        sandbox_id: "public-host-edge-local-parity-vst3".into(),
        plugin_format: PluginFormat::Vst3,
        plugin_type_id: Some("plugin:vst3:instrument".into()),
    })
    .expect("public local parity vst3 sandbox ensure should succeed");
    host.ensure_plugin_sandbox(PluginSandboxSpec {
        sandbox_id: "public-host-edge-local-parity-au".into(),
        plugin_format: PluginFormat::Au,
        plugin_type_id: Some("plugin:au:instrument".into()),
    })
    .expect("public local parity au sandbox ensure should succeed");

    let report = host.supervisor_report();
    let discovery = &report.observation.plugin_discovery_snapshot;
    assert_eq!(discovery.parity_coverage.len(), 3);
    let clap_parity = discovery
        .parity_coverage
        .iter()
        .find(|record| record.format == PluginFormat::Clap)
        .expect("public local parity report should include clap parity");
    assert_eq!(clap_parity.parity_band, RuntimePluginParityBand::Portable);
    assert_eq!(
        clap_parity.supported_platforms,
        vec![
            RuntimePluginHostPlatform::MacOs,
            RuntimePluginHostPlatform::Linux,
            RuntimePluginHostPlatform::Windows,
        ]
    );
    let au_parity = discovery
        .parity_coverage
        .iter()
        .find(|record| record.format == PluginFormat::Au)
        .expect("public local parity report should include au parity");
    assert_eq!(au_parity.parity_band, RuntimePluginParityBand::Guarded);
    assert_eq!(
        au_parity.supported_platforms,
        vec![RuntimePluginHostPlatform::MacOs]
    );
    assert_eq!(
        au_parity.unsupported_platforms,
        vec![
            RuntimePluginHostPlatform::Linux,
            RuntimePluginHostPlatform::Windows,
        ]
    );
    let lifecycle_au = report
        .observation
        .plugin_lifecycle_snapshot
        .parity_coverage
        .iter()
        .find(|record| record.format == PluginFormat::Au)
        .expect("public local parity lifecycle should include au parity");
    assert_eq!(lifecycle_au.sandbox_count, 1);
    assert_eq!(lifecycle_au.ready_sandbox_count, 1);
    assert_eq!(lifecycle_au.active_transport_count, 1);
    let lifecycle_vst3 = report
        .observation
        .plugin_lifecycle_snapshot
        .parity_coverage
        .iter()
        .find(|record| record.format == PluginFormat::Vst3)
        .expect("public local parity lifecycle should include vst3 parity");
    assert_eq!(
        lifecycle_vst3.parity_band,
        RuntimePluginParityBand::Portable
    );
    assert_eq!(lifecycle_vst3.sandbox_count, 1);
    assert_eq!(lifecycle_vst3.ready_sandbox_count, 1);
    assert_eq!(lifecycle_vst3.active_transport_count, 1);

    let rendered = report.render_json();
    assert!(rendered.contains("\"parity_coverage\":["));
    assert!(rendered.contains("\"parity_band\":\"Portable\""));
    assert!(rendered.contains("\"parity_band\":\"Guarded\""));
    assert!(rendered.contains("\"unsupported_platforms\":[\"Linux\",\"Windows\"]"));
}

#[test]
fn local_shared_host_edge_exports_runtime_generic_event_truth() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
    runtime.record_plugin_event_summary(
        9,
        "lease:public-local-events",
        14,
        196,
        EventPacketSummary {
            total_events: 8,
            parameter_value_events: 1,
            parameter_modulation_events: 1,
            parameter_gesture_events: 1,
            note_events: 1,
            note_expression_events: 3,
            note_expression_pressure_events: 1,
            note_expression_timbre_events: 1,
            note_expression_tuning_events: 1,
            midi_events: 1,
        },
    );
    let mut host = LocalRuntimeHost::new(runtime);

    host.start_plugin_scan(PluginScanRequest {
        roots: vec!["~/.clap".into(), "~/.vst3".into()],
        formats: vec![PluginFormat::Clap, PluginFormat::Vst3],
    })
    .expect("public local generic event scan should succeed");

    let report = host.supervisor_report();
    let snapshot = &report.observation.plugin_event_snapshot;
    assert_eq!(snapshot.last_processing_epoch, Some(9));
    assert_eq!(snapshot.last_block_sequence, Some(14));
    assert_eq!(snapshot.last_generated_event_bytes, 196);
    assert_eq!(snapshot.total_events, 8);
    assert_eq!(snapshot.note_expression_events, 3);
    assert_eq!(snapshot.midi_events, 1);
    assert_eq!(snapshot.segment_epochs, vec![9]);
    assert_eq!(
        report
            .observation
            .plugin_discovery_snapshot
            .capability_coverage
            .supports_note_expression_count,
        2
    );

    let rendered = report.render_json();
    assert!(rendered.contains("\"plugin_events\":{"));
    assert!(rendered.contains("\"note_expression_events\":3"));
    assert!(rendered.contains("\"supports_note_expression_count\":2"));
}

#[test]
fn local_shared_host_edge_exports_runtime_controller_expression_truth() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
    runtime.record_plugin_event_summary(
        15,
        "lease:public-local-controller-expression",
        24,
        224,
        EventPacketSummary {
            total_events: 9,
            parameter_value_events: 1,
            parameter_modulation_events: 1,
            parameter_gesture_events: 1,
            note_events: 1,
            note_expression_events: 4,
            note_expression_pressure_events: 1,
            note_expression_timbre_events: 1,
            note_expression_tuning_events: 2,
            midi_events: 1,
        },
    );
    let mut host = LocalRuntimeHost::new(runtime);

    host.start_plugin_scan(PluginScanRequest {
        roots: vec!["~/.clap".into(), "~/.vst3".into()],
        formats: vec![PluginFormat::Clap, PluginFormat::Vst3],
    })
    .expect("public local controller-expression scan should succeed");

    let report = host.supervisor_report();
    let snapshot = &report.observation.plugin_event_snapshot;
    assert_eq!(snapshot.note_expression_pressure_events, 1);
    assert_eq!(snapshot.note_expression_timbre_events, 1);
    assert_eq!(snapshot.note_expression_tuning_events, 2);
    assert_eq!(
        snapshot.mpe_posture,
        signal_runtime::RuntimeControllerExpressionMpePosture::Guarded
    );
    assert_eq!(
        snapshot.midi2_posture,
        signal_runtime::RuntimeControllerExpressionMidi2Posture::Guarded
    );
    assert_eq!(
        report.observation.external_midi_snapshot.graph_state,
        signal_runtime::RuntimeExternalMidiGraphState::Empty
    );

    let rendered = report.render_json();
    assert!(rendered.contains("\"note_expression_pressure_events\":1"));
    assert!(rendered.contains("\"note_expression_timbre_events\":1"));
    assert!(rendered.contains("\"note_expression_tuning_events\":2"));
    assert!(rendered.contains("\"mpe_posture\":\"Guarded\""));
    assert!(rendered.contains("\"midi2_posture\":\"Guarded\""));
    assert!(rendered.contains("\"external_midi_snapshot\":{"));
    assert!(rendered.contains("\"graph_state\":\"Empty\""));
}

#[test]
fn local_shared_host_edge_exports_runtime_control_surface_truth() {
    let runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
    let host = LocalRuntimeHost::new(runtime);
    let report = host.host_supervisor_report();

    assert_eq!(
        report
            .observation
            .observation
            .control_surface_snapshot
            .discovery_state,
        signal_runtime::RuntimeExternalMidiDiscoveryState::Idle
    );
    assert_eq!(
        report
            .observation
            .observation
            .control_surface_snapshot
            .graph_state,
        signal_runtime::RuntimeControlSurfaceGraphState::Empty
    );
    assert_eq!(
        report
            .observation
            .observation
            .control_surface_snapshot
            .provider_name,
        "signal-host-local"
    );
    assert_eq!(
        report
            .observation
            .observation
            .control_surface_snapshot
            .device_count,
        0
    );
    assert!(report
        .observation
        .observation
        .control_surface_snapshot
        .devices
        .is_empty());

    let rendered = report.render_json();
    assert!(rendered.contains("\"control_surface_snapshot\":{"));
    assert!(rendered.contains("\"graph_state\":\"Empty\""));
    assert!(rendered.contains("\"provider_name\":\"signal-host-local\""));
}

#[test]
fn local_shared_host_edge_exports_runtime_advanced_hardware_truth() {
    let runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
    let host = LocalRuntimeHost::new(runtime);
    let report = host.host_supervisor_report();

    assert_eq!(
        report
            .observation
            .observation
            .advanced_hardware_snapshot
            .discovery_state,
        signal_runtime::RuntimeExternalMidiDiscoveryState::Idle
    );
    assert_eq!(
        report
            .observation
            .observation
            .advanced_hardware_snapshot
            .graph_state,
        signal_runtime::RuntimeAdvancedHardwareGraphState::Empty
    );
    assert_eq!(
        report
            .observation
            .observation
            .advanced_hardware_snapshot
            .provider_name,
        "signal-host-local"
    );
    assert_eq!(
        report
            .observation
            .observation
            .advanced_hardware_snapshot
            .device_count,
        0
    );
    assert!(report
        .observation
        .observation
        .advanced_hardware_snapshot
        .devices
        .is_empty());

    let rendered = report.render_json();
    assert!(rendered.contains("\"advanced_hardware_snapshot\":{"));
    assert!(rendered.contains("\"graph_state\":\"Empty\""));
    assert!(rendered.contains("\"provider_name\":\"signal-host-local\""));
}

#[test]
fn local_shared_host_edge_exports_runtime_recall_portability_truth() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
    runtime
        .handshake(signal_runtime::HandshakeRequest {
            client_version: "public-host-local-recall-portability".into(),
            anticipative_preferred: true,
            max_sample_rate_hint: Some(96_000),
        })
        .expect("local host-edge recall portability handshake should succeed");
    runtime
        .configure(RuntimeConfigRequest::new(48_000, 512))
        .expect("local host-edge recall portability configure should succeed");
    let scan_handle = runtime.record_plugin_scan_request(&PluginScanRequest {
        roots: vec!["~/.clap".into()],
        formats: vec![PluginFormat::Clap],
    });
    runtime.record_plugin_scan_results(
        scan_handle,
        vec![signal_runtime::RuntimePluginDiscoveredTypeRecord {
            plugin_type_id: "plugin:clap:default".into(),
            plugin_id: "com.signal.local-default".into(),
            vendor: "Signal".into(),
            name: "Signal Local Default".into(),
            format: PluginFormat::Clap,
            version: Some("1.0.0".into()),
            features: vec![signal_plugin::PluginFeature::AudioEffect],
            default_io_layout: signal_plugin::PluginIoLayout {
                audio_inputs: 2,
                audio_outputs: 2,
                midi_inputs: 1,
                midi_outputs: 0,
            },
            default_multichannel_io: signal_runtime::RuntimeMultichannelIoSummary::for_plugin_io(
                signal_plugin::PluginIoLayout {
                    audio_inputs: 2,
                    audio_outputs: 2,
                    midi_inputs: 1,
                    midi_outputs: 0,
                },
            ),
            complex_io_summary:
                signal_runtime::RuntimePluginComplexIoSummary::from_plugin_features_and_layout(
                    &[signal_plugin::PluginFeature::AudioEffect],
                    signal_plugin::PluginIoLayout {
                        audio_inputs: 2,
                        audio_outputs: 2,
                        midi_inputs: 1,
                        midi_outputs: 0,
                    },
                ),
            audio_bus_count: 2,
            parameter_count: 4,
            state_contract: signal_plugin::PluginStateContract {
                supports_snapshot: true,
                supports_reset: true,
                supports_bypass: true,
                exposes_latency: true,
                exposes_tail: true,
            },
            processing_contract: signal_plugin::PluginProcessingContract {
                max_block_frames: 1024,
                sample_accurate_automation: true,
                accepts_midi: true,
                accepts_note_events: true,
                supports_note_expression: true,
                produces_midi: false,
                silence_aware: true,
            },
            lifecycle_contract: signal_plugin::PluginLifecycleContract {
                requires_main_thread_for_state: false,
                supports_prepare: true,
                supports_activate: true,
                supports_reset_while_active: true,
            },
            summary: "local host recall portability type".into(),
        }],
    );
    apply_public_plugin_continuity_graph(
        &mut runtime,
        "graph:host-local:recall-portability",
        &[("node-local-clap", "sandbox-local-clap")],
    );
    record_public_plugin_sandbox_ready(
        &mut runtime,
        "sandbox-local-clap",
        PluginFormat::Clap,
        "plugin:clap:default",
        41,
    );
    runtime.record_plugin_preset_descriptor("sandbox-local-clap", sample_host_preset_descriptor());
    runtime.record_plugin_ara_context("sandbox-local-clap", sample_host_ara_context());

    let host = LocalRuntimeHost::new(runtime);
    let report = host.supervisor_report();
    let recall = report
        .observation
        .execution_topology_summary
        .nodes
        .iter()
        .find(|node| node.node_id == "node-local-clap")
        .and_then(|node| node.plugin_recall.as_ref())
        .expect("local host-edge recall portability should be exported");
    assert_eq!(
        recall.payload.interchange.portability_class,
        RuntimePluginRecallPortabilityClass::Portable
    );
    assert_eq!(
        recall
            .payload
            .interchange
            .preset_descriptor
            .as_ref()
            .and_then(|descriptor| descriptor.label.as_deref()),
        Some("Local Lead")
    );
    assert_eq!(
        recall
            .payload
            .ara_context
            .as_ref()
            .and_then(|context| context.region_context.as_ref())
            .map(|region| region.region_id.as_str()),
        Some("region:chorus")
    );

    let rendered = report.render_json();
    assert!(rendered.contains("\"interchange\":{"));
    assert!(rendered.contains("\"portability_class\":\"Portable\""));
    assert!(rendered.contains("\"preset_id\":\"preset:user:local-lead\""));
    assert!(rendered.contains("\"document_id\":\"doc:host-local\""));
}

#[test]
fn local_shared_host_edge_exports_runtime_media_service_truth() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
    runtime
        .handshake(signal_runtime::HandshakeRequest {
            client_version: "public-host-local-media-service".into(),
            anticipative_preferred: true,
            max_sample_rate_hint: Some(96_000),
        })
        .expect("local host-edge media-service handshake should succeed");
    runtime
        .configure(RuntimeConfigRequest::new(48_000, 256))
        .expect("local host-edge media-service configure should succeed");

    let ready_path = public_local_media_fixture_path("ready");
    let missing_path = public_local_media_fixture_path("missing");
    write_public_test_wav(&ready_path);

    runtime
        .reconcile_media_assets(vec![
            signal_runtime::RuntimeMediaAssetRegistration {
                asset_id: "asset:sha256:host-local-media-ready".into(),
                content_hash: "host-local-media-ready".into(),
                source_path: ready_path.display().to_string(),
                file_name: "host-local-media-ready.wav".into(),
                byte_size: fs::metadata(&ready_path)
                    .expect("public local media fixture should exist")
                    .len(),
                sample_rate_hz: 48_000,
                channel_count: 1,
                duration_samples: 128,
                waveform_bin_count: 16,
            },
            signal_runtime::RuntimeMediaAssetRegistration {
                asset_id: "asset:sha256:host-local-media-missing".into(),
                content_hash: "host-local-media-missing".into(),
                source_path: missing_path.display().to_string(),
                file_name: "host-local-media-missing.wav".into(),
                byte_size: 0,
                sample_rate_hz: 48_000,
                channel_count: 1,
                duration_samples: 128,
                waveform_bin_count: 16,
            },
        ])
        .expect("local host-edge media assets should reconcile");
    runtime
        .start_media_preview("asset:sha256:host-local-media-ready")
        .expect("local host-edge media preview should start");

    let host = LocalRuntimeHost::new(runtime);
    let report = host.supervisor_report();

    assert_eq!(report.observation.media_pipeline_snapshot.asset_count, 2);
    assert_eq!(
        report.observation.media_pipeline_snapshot.ready_asset_count,
        1
    );
    assert_eq!(
        report
            .observation
            .media_pipeline_snapshot
            .invalid_asset_count,
        1
    );
    assert_eq!(
        report
            .observation
            .media_service_snapshot
            .indexed_asset_count,
        2
    );
    assert_eq!(
        report
            .observation
            .media_service_snapshot
            .waveform_ready_asset_count,
        1
    );
    assert_eq!(
        report.observation.media_service_snapshot.preview_state,
        signal_runtime::RuntimeMediaPreviewState::Previewing
    );
    assert_eq!(
        report
            .observation
            .media_service_snapshot
            .previewing_asset_id
            .as_deref(),
        Some("asset:sha256:host-local-media-ready")
    );
    assert_eq!(
        report
            .observation
            .media_service_snapshot
            .last_invalidated_asset_id
            .as_deref(),
        Some("asset:sha256:host-local-media-missing")
    );
    assert!(
        report
            .observation
            .media_service_snapshot
            .invalidation_active
    );

    let rendered = report.render_json();
    assert!(rendered.contains("\"media_pipeline_snapshot\":{"));
    assert!(rendered.contains("\"media_service_snapshot\":{"));
    assert!(rendered.contains("\"invalidated_asset_count\":1"));
    assert!(rendered.contains("\"preview_state\":\"Previewing\""));

    let _ = fs::remove_file(&ready_path);
    if let Some(path) = host
        .runtime()
        .get_media_pipeline_snapshot()
        .assets
        .iter()
        .find(|asset| asset.asset_id == "asset:sha256:host-local-media-ready")
        .and_then(|asset| asset.cache_path.as_deref())
    {
        let _ = fs::remove_file(path);
    }
}

#[test]
fn local_shared_host_edge_exports_runtime_analysis_metadata_truth() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
    runtime
        .handshake(signal_runtime::HandshakeRequest {
            client_version: "public-host-local-analysis-metadata".into(),
            anticipative_preferred: true,
            max_sample_rate_hint: Some(96_000),
        })
        .expect("local host-edge analysis-metadata handshake should succeed");
    runtime
        .configure(RuntimeConfigRequest::new(48_000, 256))
        .expect("local host-edge analysis-metadata configure should succeed");

    let ready_path = public_local_media_fixture_path("analysis-ready");
    let missing_path = public_local_media_fixture_path("analysis-missing");
    write_public_test_wav(&ready_path);

    runtime
        .reconcile_media_assets(vec![
            signal_runtime::RuntimeMediaAssetRegistration {
                asset_id: "asset:sha256:host-local-analysis-ready".into(),
                content_hash: "host-local-analysis-ready".into(),
                source_path: ready_path.display().to_string(),
                file_name: "host-local-analysis-ready.wav".into(),
                byte_size: fs::metadata(&ready_path)
                    .expect("public local analysis fixture should exist")
                    .len(),
                sample_rate_hz: 48_000,
                channel_count: 1,
                duration_samples: 128,
                waveform_bin_count: 16,
            },
            signal_runtime::RuntimeMediaAssetRegistration {
                asset_id: "asset:sha256:host-local-analysis-missing".into(),
                content_hash: "host-local-analysis-missing".into(),
                source_path: missing_path.display().to_string(),
                file_name: "host-local-analysis-missing.wav".into(),
                byte_size: 0,
                sample_rate_hz: 48_000,
                channel_count: 1,
                duration_samples: 128,
                waveform_bin_count: 16,
            },
        ])
        .expect("local host-edge analysis metadata assets should reconcile");

    let host = LocalRuntimeHost::new(runtime);
    let report = host.supervisor_report();

    assert_eq!(
        report
            .observation
            .media_library_snapshot
            .indexed_asset_count,
        2
    );
    assert_eq!(
        report
            .observation
            .media_library_snapshot
            .ready_descriptor_count,
        1
    );
    assert_eq!(
        report
            .observation
            .media_library_snapshot
            .invalidated_descriptor_count,
        1
    );
    assert_eq!(
        report
            .observation
            .media_library_snapshot
            .loudness_ready_descriptor_count,
        1
    );
    assert_eq!(
        report
            .observation
            .media_library_snapshot
            .character_ready_descriptor_count,
        1
    );
    let ready = report
        .observation
        .media_library_snapshot
        .descriptors
        .iter()
        .find(|descriptor| descriptor.asset_id == "asset:sha256:host-local-analysis-ready")
        .expect("local host-edge ready analysis descriptor");
    assert_eq!(
        ready.metadata_state,
        signal_runtime::RuntimeMediaAnalysisDescriptorState::Ready
    );
    assert!(ready.loudness.is_some());
    assert!(ready.character.is_some());
    let invalidated = report
        .observation
        .media_library_snapshot
        .descriptors
        .iter()
        .find(|descriptor| descriptor.asset_id == "asset:sha256:host-local-analysis-missing")
        .expect("local host-edge invalidated analysis descriptor");
    assert_eq!(
        invalidated.metadata_state,
        signal_runtime::RuntimeMediaAnalysisDescriptorState::Invalidated
    );

    let rendered = report.render_json();
    assert!(rendered.contains("\"media_library_snapshot\":{"));
    assert!(rendered.contains("\"ready_descriptor_count\":1"));
    assert!(rendered.contains("\"invalidated_descriptor_count\":1"));
    assert!(rendered.contains("\"loudness_ready_descriptor_count\":1"));
    assert!(rendered.contains("\"character_ready_descriptor_count\":1"));
    assert!(rendered.contains("\"metadata_state\":\"Ready\""));
    assert!(rendered.contains("\"metadata_state\":\"Invalidated\""));

    let _ = fs::remove_file(&ready_path);
    if let Some(path) = host
        .runtime()
        .get_media_pipeline_snapshot()
        .assets
        .iter()
        .find(|asset| asset.asset_id == "asset:sha256:host-local-analysis-ready")
        .and_then(|asset| asset.cache_path.as_deref())
    {
        let _ = fs::remove_file(path);
    }
}

#[test]
fn local_shared_host_edge_exports_runtime_fault_diagnostic_truth() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
    runtime
        .set_safe_mode(SafeModeRequest { enabled: true })
        .expect("local host-edge fault diagnostic safe mode should enable");
    runtime
        .render_offline_queue(vec![RuntimeOfflineRenderRequest {
            request_id: "render:host-local:fault-diagnostic".into(),
            timeline_start_samples: 0,
            duration_samples: 64,
            export_sample_rate_hz: 48_000,
            include_main_mix: true,
            artifact_root_path: None,
            stem_targets: Vec::new(),
            freeze_artifacts: Vec::new(),
        }])
        .expect("local host-edge fault diagnostic queue should defer");

    let host = LocalRuntimeHost::new(runtime);
    let report = host.supervisor_report();

    assert_eq!(
        report.observation.fault_diagnostic_receipt.primary_family,
        Some(signal_runtime::RuntimeFaultDiagnosticFamily::DeferredWorkPressure)
    );
    assert_eq!(
        report
            .observation
            .fault_diagnostic_receipt
            .interruption_class,
        RuntimeInterruptionClass::Recoverable
    );
    assert!(report
        .observation
        .fault_diagnostic_receipt
        .contributions
        .iter()
        .any(|entry| {
            entry.family == signal_runtime::RuntimeFaultDiagnosticFamily::DeferredWorkPressure
                && entry.active
        }));

    let rendered = report.render_json();
    assert!(rendered.contains("\"fault_diagnostic_receipt\":{"));
    assert!(rendered.contains("\"primary_family\":\"DeferredWorkPressure\""));
}

#[test]
fn local_shared_host_edge_exports_runtime_device_supervision_truth() {
    let recovering_runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
    let mut recovering_host = LocalRuntimeHost::new(recovering_runtime);
    recovering_host
        .boot_with_device_loss_recovery()
        .expect("public local device supervision recovery should succeed");
    let recovering = recovering_host.supervisor_report();
    assert_eq!(
        recovering.observation.device_supervision_snapshot.state,
        RuntimeDeviceSupervisionState::Stable
    );
    assert_eq!(
        recovering
            .observation
            .device_supervision_snapshot
            .restart_state,
        RuntimeDeviceRestartState::Recovered
    );
    assert_eq!(
        recovering
            .observation
            .device_supervision_snapshot
            .fault_boundary,
        RuntimeDeviceFaultBoundaryState::Clear
    );
    assert_eq!(
        recovering
            .observation
            .device_supervision_snapshot
            .device_loss_count,
        1
    );

    let exhausted_runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
    let mut exhausted_host = LocalRuntimeHost::new(exhausted_runtime);
    let error = exhausted_host
        .boot_with_device_loss_restart_failure()
        .expect_err("public local device supervision restart failure should fail");
    assert_eq!(error.kind, RuntimeErrorKind::HardwareFailure);
    let exhausted = exhausted_host.supervisor_report();
    assert_eq!(
        exhausted.observation.device_supervision_snapshot.state,
        RuntimeDeviceSupervisionState::Exhausted
    );
    assert_eq!(
        exhausted
            .observation
            .device_supervision_snapshot
            .restart_state,
        RuntimeDeviceRestartState::Exhausted
    );
    assert_eq!(
        exhausted
            .observation
            .device_supervision_snapshot
            .fault_boundary,
        RuntimeDeviceFaultBoundaryState::Exhausted
    );
    assert_eq!(
        exhausted
            .observation
            .device_supervision_snapshot
            .restart_failure_count,
        Some(1)
    );

    let mut faulted_runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
    faulted_runtime
        .handshake(signal_runtime::HandshakeRequest {
            client_version: "public-host-local-device-supervision-faulted".into(),
            anticipative_preferred: true,
            max_sample_rate_hint: Some(96_000),
        })
        .expect("public local device supervision handshake should succeed");
    faulted_runtime
        .configure(RuntimeConfigRequest::new(48_000, 512))
        .expect("public local device supervision configure should succeed");
    faulted_runtime
        .start()
        .expect("public local device supervision start should succeed");
    faulted_runtime.fail_runtime(RuntimeError::new(
        RuntimeErrorKind::HardwareFailure,
        "public local host device supervision fault",
    ));
    let faulted_host = LocalRuntimeHost::new(faulted_runtime);
    let faulted = faulted_host.supervisor_report();
    assert_eq!(
        faulted.observation.device_supervision_snapshot.state,
        RuntimeDeviceSupervisionState::Faulted
    );
    assert_eq!(
        faulted
            .observation
            .device_supervision_snapshot
            .restart_state,
        RuntimeDeviceRestartState::Faulted
    );
    assert_eq!(
        faulted
            .observation
            .device_supervision_snapshot
            .fault_boundary,
        RuntimeDeviceFaultBoundaryState::Faulted
    );
    assert_eq!(
        faulted
            .observation
            .device_supervision_snapshot
            .recovery_state,
        RuntimeRecoveryState::Faulted
    );

    let rendered = faulted.render_json();
    assert!(rendered.contains("\"device_supervision_snapshot\":{"));
    assert!(rendered.contains("\"state\":\"Faulted\""));
    assert!(rendered.contains("\"fault_boundary\":\"Faulted\""));
}

#[test]
fn local_shared_host_edge_exports_runtime_clock_topology_truth() {
    let runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
    let mut steady_host = LocalRuntimeHost::new(runtime);
    steady_host
        .boot_default()
        .expect("public local clock topology default boot should succeed");
    let steady = steady_host.host_supervisor_report();

    assert_eq!(
        steady.observation.host_io.clocking.drift_state,
        RuntimeHostClockDriftState::Stable
    );
    assert_eq!(
        steady.observation.host_io.clocking.discontinuity_state,
        RuntimeHostClockDiscontinuityState::Continuous
    );
    assert_eq!(
        steady.observation.host_io.clocking.duplex_mismatch_state,
        RuntimeHostDuplexMismatchState::NotApplicable
    );
    assert_eq!(
        steady.observation.host_io.clocking.endpoint_topology,
        RuntimeHostEndpointTopology::OutputOnly
    );
    assert!(!steady.observation.host_io.clocking.partial_availability);

    let runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
    let mut faulted_host = LocalRuntimeHost::new(runtime);
    let error = faulted_host
        .boot_with_device_loss_restart_failure()
        .expect_err("public local clock topology restart failure should fail");
    assert_eq!(error.kind, RuntimeErrorKind::HardwareFailure);
    let faulted = faulted_host.host_supervisor_report();

    assert_eq!(
        faulted.observation.host_io.clocking.drift_state,
        RuntimeHostClockDriftState::Resyncing
    );
    assert_eq!(
        faulted.observation.host_io.clocking.discontinuity_state,
        RuntimeHostClockDiscontinuityState::Faulted
    );
    assert_eq!(
        faulted.observation.host_io.clocking.duplex_mismatch_state,
        RuntimeHostDuplexMismatchState::NotApplicable
    );
    assert_eq!(
        faulted.observation.host_io.clocking.endpoint_topology,
        RuntimeHostEndpointTopology::OutputOnly
    );
    assert!(!faulted.observation.host_io.clocking.partial_availability);

    let rendered = faulted.render_json();
    assert!(rendered.contains("\"drift_state\":\"Resyncing\""));
    assert!(rendered.contains("\"discontinuity_state\":\"Faulted\""));
    assert!(rendered.contains("\"endpoint_topology\":\"OutputOnly\""));
}

#[test]
fn local_shared_host_edge_exports_runtime_linux_backend_clock_topology_truth() {
    let runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
    let mut host = LocalRuntimeHost::new(runtime);
    host.boot_default()
        .expect("public local linux backend clock topology default boot should succeed");
    let report = host.host_supervisor_report();

    assert_eq!(
        report.observation.host_io.hardware.linux_backend_identity,
        signal_runtime::RuntimeLinuxAudioBackendIdentity::NotLinux
    );
    assert_eq!(
        report
            .observation
            .host_io
            .hardware
            .linux_backend_portability,
        signal_runtime::RuntimeLinuxAudioBackendPortabilityBand::Unsupported
    );
    assert_eq!(
        report.observation.host_io.clocking.linux_clocking_parity,
        signal_runtime::RuntimeLinuxAudioBackendClockingParityBand::Unsupported
    );
    assert_eq!(
        report.observation.host_io.clocking.linux_duplex_parity,
        signal_runtime::RuntimeLinuxAudioBackendDuplexParityState::Unsupported
    );
    assert_eq!(
        report
            .observation
            .host_io
            .clocking
            .linux_endpoint_topology_parity,
        signal_runtime::RuntimeLinuxAudioBackendEndpointTopologyParityState::Unsupported
    );

    let rendered = report.render_json();
    assert!(rendered.contains("\"linux_backend_identity\":\"NotLinux\""));
    assert!(rendered.contains("\"linux_clocking_parity\":\"Unsupported\""));
    assert!(rendered.contains("\"linux_duplex_parity\":\"Unsupported\""));
    assert!(rendered.contains("\"linux_endpoint_topology_parity\":\"Unsupported\""));
}

#[test]
fn local_shared_host_edge_exports_runtime_linux_live_ownership_truth() {
    let runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
    let mut host = LocalRuntimeHost::new(runtime);
    host.boot_default()
        .expect("public local linux live ownership default boot should succeed");
    let report = host.host_supervisor_report();

    assert_eq!(
        report
            .observation
            .observation
            .linux_backend_session_snapshot
            .backend_identity,
        signal_runtime::RuntimeLinuxAudioBackendIdentity::NotLinux
    );
    assert_eq!(
        report
            .observation
            .observation
            .linux_backend_session_snapshot
            .ownership,
        signal_runtime::RuntimeLinuxBackendSessionOwnership::NotLinux
    );
    assert_eq!(
        report
            .observation
            .observation
            .linux_backend_session_snapshot
            .lifecycle_state,
        signal_runtime::RuntimeLinuxBackendSessionLifecycleState::NotLinux
    );
    assert_eq!(
        report
            .observation
            .observation
            .linux_backend_session_snapshot
            .device_claim_posture,
        signal_runtime::RuntimeLinuxBackendDeviceClaimPosture::NotLinux
    );
    assert_eq!(
        report
            .observation
            .observation
            .linux_backend_session_snapshot
            .session_role,
        signal_runtime::RuntimeLinuxBackendSessionRole::NotLinux
    );
    assert_eq!(
        report
            .observation
            .observation
            .linux_backend_session_snapshot
            .ownership_fallback,
        signal_runtime::RuntimeLinuxBackendOwnershipFallbackState::NotLinux
    );

    let rendered = report.render_json();
    assert!(rendered.contains("\"linux_backend_session_snapshot\":{"));
    assert!(rendered.contains("\"backend_identity\":\"NotLinux\""));
    assert!(rendered.contains("\"ownership\":\"NotLinux\""));
    assert!(rendered.contains("\"device_claim_posture\":\"NotLinux\""));
}

#[test]
fn local_shared_host_edge_exports_runtime_jack_coordination_truth() {
    let runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
    let mut host = LocalRuntimeHost::new(runtime);
    host.boot_default()
        .expect("public local jack coordination default boot should succeed");
    let report = host.supervisor_report();

    assert_eq!(
        report
            .observation
            .jack_coordination_snapshot
            .transport_posture,
        RuntimeJackTransportPosture::NotJack
    );
    assert_eq!(
        report.observation.jack_coordination_snapshot.graph_state,
        RuntimeJackGraphCoordinationState::NotJack
    );
    assert_eq!(
        report.observation.jack_coordination_snapshot.client_role,
        RuntimeJackClientRole::NotJack
    );
    assert_eq!(
        report.observation.jack_coordination_snapshot.guarded_state,
        RuntimeJackGuardedCoordinationState::NotJack
    );

    let rendered = report.render_json();
    assert!(rendered.contains("\"jack_coordination_snapshot\":{"));
    assert!(rendered.contains("\"transport_posture\":\"NotJack\""));
    assert!(rendered.contains("\"graph_state\":\"NotJack\""));
    assert!(rendered.contains("\"client_role\":\"NotJack\""));
    assert!(rendered.contains("\"guarded_state\":\"NotJack\""));
}

#[test]
fn local_shared_host_edge_exports_runtime_external_midi_truth() {
    let runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
    let mut host = LocalRuntimeHost::new(runtime);
    host.boot_default()
        .expect("public local external midi default boot should succeed");
    let report = host.supervisor_report();

    assert_eq!(
        report.observation.external_midi_snapshot.discovery_state,
        signal_runtime::RuntimeExternalMidiDiscoveryState::Idle
    );
    assert_eq!(
        report.observation.external_midi_snapshot.graph_state,
        signal_runtime::RuntimeExternalMidiGraphState::Empty
    );
    assert_eq!(
        report.observation.external_midi_snapshot.provider_name,
        "signal-host-local"
    );
    assert_eq!(report.observation.external_midi_snapshot.device_count, 0);
    assert_eq!(report.observation.external_midi_snapshot.endpoint_count, 0);
    assert_eq!(
        report.observation.external_midi_snapshot.active_route_count,
        0
    );

    let rendered = report.render_json();
    assert!(rendered.contains("\"external_midi_snapshot\":{"));
    assert!(rendered.contains("\"discovery_state\":\"Idle\""));
    assert!(rendered.contains("\"graph_state\":\"Empty\""));
    assert!(rendered.contains("\"provider_name\":\"signal-host-local\""));
}

#[test]
fn local_shared_host_edge_exports_runtime_external_io_truth() {
    let runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
    let mut direct_host = LocalRuntimeHost::new(runtime);
    direct_host
        .boot_default()
        .expect("public local external io default boot should succeed");
    let direct = direct_host.supervisor_report();

    assert_eq!(
        direct.observation.external_io_snapshot.primary_role,
        RuntimeExternalIoPrimaryRole::ProgramOutput
    );
    assert_eq!(
        direct.observation.external_io_snapshot.monitoring_state,
        RuntimeExternalIoMonitoringState::Direct
    );
    assert_eq!(
        direct.observation.external_io_snapshot.monitoring_tap_point,
        RuntimeExternalIoMonitoringTapPoint::PostHardwareOutput
    );
    assert_eq!(
        direct.observation.external_io_snapshot.loopback_state,
        RuntimeExternalIoLoopbackState::Unavailable
    );

    let faulted_runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
    let mut faulted_host = LocalRuntimeHost::new(faulted_runtime);
    let error = faulted_host
        .boot_with_device_loss_restart_failure()
        .expect_err("public local external io restart failure should fail");
    assert_eq!(error.kind, RuntimeErrorKind::HardwareFailure);
    let faulted = faulted_host.supervisor_report();

    assert_eq!(
        faulted.observation.external_io_snapshot.health_state,
        RuntimeExternalIoHealthState::Faulted
    );
    assert_eq!(
        faulted.observation.external_io_snapshot.monitoring_state,
        RuntimeExternalIoMonitoringState::Faulted
    );
    assert_eq!(
        faulted.observation.external_io_snapshot.loopback_state,
        RuntimeExternalIoLoopbackState::Faulted
    );

    let rendered = faulted.render_json();
    assert!(rendered.contains("\"external_io_snapshot\":{"));
    assert!(rendered.contains("\"health_state\":\"Faulted\""));
    assert!(rendered.contains("\"monitoring_state\":\"Faulted\""));
    assert!(rendered.contains("\"loopback_state\":\"Faulted\""));
}

#[test]
fn local_shared_host_edge_exports_runtime_multichannel_truth() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
    runtime
        .handshake(signal_runtime::HandshakeRequest {
            client_version: "public-host-local-multichannel".into(),
            anticipative_preferred: true,
            max_sample_rate_hint: Some(96_000),
        })
        .expect("local host-edge multichannel handshake should succeed");
    runtime
        .configure(RuntimeConfigRequest::new(48_000, 256))
        .expect("local host-edge multichannel configure should succeed");
    apply_public_multichannel_graph(&mut runtime, "graph:host-local:multichannel");
    let scan_handle = runtime.record_plugin_scan_request(&PluginScanRequest {
        roots: vec!["~/.clap".into()],
        formats: vec![PluginFormat::Clap],
    });
    runtime.record_plugin_scan_results(
        scan_handle,
        vec![signal_runtime::RuntimePluginDiscoveredTypeRecord {
            plugin_type_id: "plugin:clap:host-local-multichannel".into(),
            plugin_id: "com.signal.host-local-multichannel".into(),
            vendor: "Signal".into(),
            name: "Signal Host Local Multichannel".into(),
            format: PluginFormat::Clap,
            version: Some("1.0.0".into()),
            features: vec![signal_plugin::PluginFeature::AudioEffect],
            default_io_layout: signal_plugin::PluginIoLayout {
                audio_inputs: 2,
                audio_outputs: 6,
                midi_inputs: 1,
                midi_outputs: 0,
            },
            default_multichannel_io: signal_runtime::RuntimeMultichannelIoSummary::for_plugin_io(
                signal_plugin::PluginIoLayout {
                    audio_inputs: 2,
                    audio_outputs: 6,
                    midi_inputs: 1,
                    midi_outputs: 0,
                },
            ),
            complex_io_summary:
                signal_runtime::RuntimePluginComplexIoSummary::from_plugin_features_and_layout(
                    &[signal_plugin::PluginFeature::AudioEffect],
                    signal_plugin::PluginIoLayout {
                        audio_inputs: 2,
                        audio_outputs: 6,
                        midi_inputs: 1,
                        midi_outputs: 0,
                    },
                ),
            audio_bus_count: 2,
            parameter_count: 6,
            state_contract: signal_plugin::PluginStateContract {
                supports_snapshot: true,
                supports_reset: true,
                supports_bypass: true,
                exposes_latency: true,
                exposes_tail: true,
            },
            processing_contract: signal_plugin::PluginProcessingContract {
                max_block_frames: 1024,
                sample_accurate_automation: true,
                accepts_midi: true,
                accepts_note_events: true,
                supports_note_expression: true,
                produces_midi: false,
                silence_aware: true,
            },
            lifecycle_contract: signal_plugin::PluginLifecycleContract {
                requires_main_thread_for_state: false,
                supports_prepare: true,
                supports_activate: true,
                supports_reset_while_active: true,
            },
            summary: "local multichannel boundary plugin".into(),
        }],
    );

    let host = LocalRuntimeHost::new(runtime);
    let report = host.supervisor_report();
    let track_node = report
        .observation
        .execution_topology_summary
        .nodes
        .iter()
        .find(|node| node.node_id == "surround-track")
        .expect("surround-track node should be present");
    assert_eq!(
        track_node.output_layout.canonical_layout,
        Some(RuntimeCanonicalChannelLayout::Surround5_1)
    );
    assert_eq!(track_node.output_bus_intent, RuntimeBusIntent::MainProgram);
    let send_node = report
        .observation
        .execution_topology_summary
        .nodes
        .iter()
        .find(|node| node.node_id == "analysis-send")
        .expect("analysis-send node should be present");
    assert_eq!(
        send_node.output_layout.canonical_layout,
        Some(RuntimeCanonicalChannelLayout::Quad)
    );
    assert_eq!(send_node.output_bus_intent, RuntimeBusIntent::AuxSend);
    assert_eq!(
        report
            .observation
            .plugin_discovery_snapshot
            .discovered_types[0]
            .default_multichannel_io
            .output_layout
            .canonical_layout,
        Some(RuntimeCanonicalChannelLayout::Surround5_1)
    );

    let rendered = report.render_json();
    assert!(rendered.contains("\"canonical_layout\":\"Surround5_1\""));
    assert!(rendered.contains("\"output_bus_intent\":\"MainProgram\""));
    assert!(rendered.contains("\"default_multichannel_io\":{"));
}

#[test]
fn local_shared_host_edge_exports_runtime_sidechain_truth() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
    runtime
        .handshake(signal_runtime::HandshakeRequest {
            client_version: "public-host-local-sidechain".into(),
            anticipative_preferred: true,
            max_sample_rate_hint: Some(96_000),
        })
        .expect("local host-edge sidechain handshake should succeed");
    runtime
        .configure(RuntimeConfigRequest::new(48_000, 256))
        .expect("local host-edge sidechain configure should succeed");
    apply_public_sidechain_graph(&mut runtime, "graph:host-local:sidechain");
    record_public_plugin_sandbox_ready(
        &mut runtime,
        "sandbox:host-local:sidechain",
        PluginFormat::Clap,
        "plugin:clap:host-local-sidechain-compressor",
        1,
    );

    let host = LocalRuntimeHost::new(runtime);
    let report = host.supervisor_report();
    let topology = &report.observation.execution_topology_summary;
    assert_eq!(topology.secondary_input_count, 1);
    assert_eq!(topology.required_secondary_input_count, 1);
    let route = &topology.secondary_inputs[0];
    assert_eq!(route.source_id, "kick-sidechain");
    assert_eq!(route.source_bus_id.as_deref(), Some("bus:sidechain:kick"));
    assert_eq!(
        route.target_kind,
        RuntimeSecondaryInputTargetKind::NodeInput
    );
    assert_eq!(route.target_id, "compressor");
    assert_eq!(route.target_bus_id, "plugin:compressor:sidechain");
    assert_eq!(
        route.attachment_policy,
        RuntimeSecondaryInputAttachmentPolicy::Required
    );
    assert_eq!(
        route.fallback_outcome,
        RuntimeSecondaryInputFallbackOutcome::SafeModeDegradation
    );

    let compressor = topology
        .nodes
        .iter()
        .find(|node| node.node_id == "compressor")
        .expect("compressor node should be present");
    let node_secondary_input = compressor
        .secondary_input
        .as_ref()
        .expect("compressor should carry sidechain receipt");
    assert_eq!(node_secondary_input.source_id, "kick-sidechain");
    assert_eq!(
        node_secondary_input.target_kind,
        RuntimeSecondaryInputTargetKind::NodeInput
    );

    let stage_secondary_input = report.observation.plugin_chain_snapshot.chains[0].stages[0]
        .secondary_input
        .as_ref()
        .expect("local host-edge sidechain plugin stage should be exported");
    assert_eq!(
        stage_secondary_input.target_kind,
        RuntimeSecondaryInputTargetKind::PluginInput
    );
    assert_eq!(stage_secondary_input.target_id, "compressor");

    let rendered = report.render_json();
    assert!(rendered.contains("\"execution_topology_summary\":{"));
    assert!(rendered.contains("\"secondary_input_count\":1"));
    assert!(rendered.contains("\"source_id\":\"kick-sidechain\""));
    assert!(rendered.contains("\"target_kind\":\"NodeInput\""));
    assert!(rendered.contains("\"target_kind\":\"PluginInput\""));
    assert!(rendered.contains("\"fallback_outcome\":\"SafeModeDegradation\""));
}

#[test]
fn local_shared_host_edge_exports_runtime_multi_bus_truth() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
    runtime
        .handshake(signal_runtime::HandshakeRequest {
            client_version: "public-host-local-multi-bus".into(),
            anticipative_preferred: true,
            max_sample_rate_hint: Some(96_000),
        })
        .expect("local host-edge multi-bus handshake should succeed");
    runtime
        .configure(RuntimeConfigRequest::new(48_000, 256))
        .expect("local host-edge multi-bus configure should succeed");
    apply_public_multi_bus_graph(&mut runtime, "graph:host-local:multi-bus");
    runtime
        .process_engine_block(
            3,
            5,
            synthetic_stereo_block(SampleRate(48_000), FrameCount(256), 2),
        )
        .expect("local host-edge multi-bus block should succeed");

    let host = LocalRuntimeHost::new(runtime);
    let report = host.supervisor_report();
    let topology = &report.observation.execution_topology_summary;
    assert_eq!(topology.bus_connection_count, 5);
    assert_eq!(topology.auxiliary_path_count, 3);
    assert!(topology.bus_connections.iter().any(|connection| {
        connection.connection_id == "send-fx:bus:fx:plate->return-fx:bus:fx:plate"
            && connection.source_bus_role == RuntimeBusRole::AuxSend
            && connection.target_bus_role == RuntimeBusRole::AuxReturn
            && connection.auxiliary_path_kind == Some(RuntimeAuxiliaryPathKind::SendReturn)
    }));
    assert!(topology.auxiliary_paths.iter().any(|path| {
        path.auxiliary_path_id == "bus_group:mix:master"
            && path.path_kind == RuntimeAuxiliaryPathKind::Submix
            && path.bus_role == RuntimeBusRole::Submix
    }));
    assert_eq!(report.observation.metering_snapshot.bus_connection_count, 5);
    assert_eq!(report.observation.metering_snapshot.auxiliary_path_count, 3);

    let rendered = report.render_json();
    assert!(rendered.contains("\"bus_connection_count\":5"));
    assert!(rendered.contains("\"auxiliary_path_count\":3"));
    assert!(rendered.contains("\"connection_id\":\"send-fx:bus:fx:plate->return-fx:bus:fx:plate\""));
    assert!(rendered.contains("\"auxiliary_path_id\":\"send_return:fx:plate\""));
}

#[test]
fn local_shared_host_edge_exports_runtime_complex_io_truth() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
    runtime
        .handshake(signal_runtime::HandshakeRequest {
            client_version: "public-host-local-complex-io".into(),
            anticipative_preferred: true,
            max_sample_rate_hint: Some(96_000),
        })
        .expect("public local complex io handshake should succeed");
    runtime
        .configure(RuntimeConfigRequest::new(48_000, 256))
        .expect("public local complex io configure should succeed");
    apply_public_complex_io_graph(&mut runtime, "graph:host-local:complex-io");
    let scan_handle = runtime.record_plugin_scan_request(&PluginScanRequest {
        roots: vec!["~/Library/Audio/Plug-Ins/VST3".into()],
        formats: vec![PluginFormat::Vst3],
    });
    runtime.record_plugin_scan_results(
        scan_handle,
        vec![
            sample_complex_multi_output_record(),
            sample_complex_bus_fx_record(),
        ],
    );
    runtime
        .apply_plugin_node_render_batch(signal_runtime::PluginNodeRenderBatch {
            graph_id: "graph:host-local:complex-io".into(),
            processing_epoch: 1,
            block_sequence: 1,
            renders: vec![
                signal_runtime::PluginNodeRender {
                    node_id: "plugin-multiout".into(),
                    sandbox_id: "sandbox:host-local:multiout".into(),
                    output: AudioBuffer::new(
                        SampleRate(48_000),
                        ChannelLayout::Stereo,
                        FrameCount(8),
                    ),
                    latency_samples: 32,
                    tail_samples: 48,
                    bypassed: false,
                },
                signal_runtime::PluginNodeRender {
                    node_id: "plugin-bus-fx".into(),
                    sandbox_id: "sandbox:host-local:bus-fx".into(),
                    output: AudioBuffer::new(
                        SampleRate(48_000),
                        ChannelLayout::Stereo,
                        FrameCount(8),
                    ),
                    latency_samples: 16,
                    tail_samples: 24,
                    bypassed: false,
                },
            ],
        })
        .expect("public local complex io render batch should apply");
    runtime
        .process_engine_block(
            5,
            7,
            synthetic_stereo_block(SampleRate(48_000), FrameCount(256), 5),
        )
        .expect("public local complex io block should process");

    let host = LocalRuntimeHost::new(runtime);
    let report = host.supervisor_report();
    let discovery = &report.observation.plugin_discovery_snapshot;
    assert_eq!(discovery.discovered_type_count, 2);
    assert_eq!(discovery.capability_coverage.complex_io_type_count, 2);
    assert_eq!(
        discovery.capability_coverage.multi_output_instrument_count,
        1
    );
    assert_eq!(discovery.capability_coverage.bus_capable_fx_count, 1);
    assert!(discovery.discovered_types.iter().any(|record| {
        record.plugin_type_id == "plugin:vst3:host-local-multiout"
            && record.complex_io_summary.multi_output_instrument
    }));
    assert!(discovery.discovered_types.iter().any(|record| {
        record.plugin_type_id == "plugin:vst3:host-local-bus-fx"
            && record.complex_io_summary.bus_capable_fx_class
                == Some(RuntimePluginBusCapableFxClass::SendReturnCapableFx)
    }));

    let plugin_chain = &report.observation.plugin_chain_snapshot;
    assert_eq!(plugin_chain.chain_count, 1);
    assert_eq!(plugin_chain.stage_count, 2);
    assert!(plugin_chain
        .chains
        .iter()
        .flat_map(|chain| chain.stages.iter())
        .any(|stage| {
            stage.node_id == "plugin-multiout"
                && stage.complex_io_summary.multi_output_instrument
                && stage.complex_io_summary.instrument_output_group_count == 2
        }));
    assert!(plugin_chain
        .chains
        .iter()
        .flat_map(|chain| chain.stages.iter())
        .any(|stage| {
            stage.node_id == "plugin-bus-fx"
                && stage.complex_io_summary.bus_capable_fx_class
                    == Some(RuntimePluginBusCapableFxClass::SendReturnCapableFx)
                && stage.complex_io_summary.secondary_input_group_count == 1
        }));

    let rendered = report.render_json();
    assert!(rendered.contains("\"plugin_discovery_snapshot\":{"));
    assert!(rendered.contains("\"complex_io_summary\":{"));
    assert!(rendered.contains("\"multi_output_instrument\":true"));
    assert!(rendered.contains("\"bus_capable_fx_class\":\"SendReturnCapableFx\""));
}

#[test]
fn local_shared_host_edge_exports_runtime_spatial_truth() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
    runtime
        .handshake(signal_runtime::HandshakeRequest {
            client_version: "public-host-local-spatial".into(),
            anticipative_preferred: true,
            max_sample_rate_hint: Some(96_000),
        })
        .expect("public local spatial handshake should succeed");
    runtime
        .configure(RuntimeConfigRequest::new(48_000, 256))
        .expect("public local spatial configure should succeed");
    apply_public_spatial_graph(&mut runtime, "graph:host-local:spatial");

    let host = LocalRuntimeHost::new(runtime);
    let report = host.supervisor_report();
    let topology = &report.observation.execution_topology_summary;
    assert_eq!(topology.spatial_node_count, 2);
    assert_eq!(topology.active_spatial_node_count, 1);
    assert_eq!(topology.bypassed_spatial_node_count, 1);
    assert_eq!(topology.fallback_spatial_node_count, 1);
    assert_eq!(topology.surround_bed_spatial_node_count, 1);
    assert_eq!(topology.object_aware_spatial_node_count, 0);
    assert_eq!(topology.expanded_fallback_spatial_node_count, 1);
    assert!(topology.nodes.iter().any(|node| {
        node.node_id == "spatial-stereo"
            && node.spatial_execution.as_ref().is_some_and(|spatial| {
                spatial.execution_mode == RuntimeSpatialExecutionMode::BalanceGroups
                    && spatial.bed_class == RuntimeSpatialBedClass::StereoBed
                    && spatial.mix_policy == RuntimeSpatialMixPolicy::BedOnly
            })
    }));
    assert!(report
        .observation
        .plugin_chain_snapshot
        .chains
        .iter()
        .flat_map(|chain| chain.stages.iter())
        .any(|stage| {
            stage.node_id == "spatial-surround"
                && stage.spatial_execution.as_ref().is_some_and(|spatial| {
                    spatial.fallback_outcome
                        == Some(RuntimeSpatialFallbackOutcome::BypassSpatialProcessing)
                        && spatial.bed_class == RuntimeSpatialBedClass::CanonicalSurroundBed
                        && spatial.expanded_fallback_outcome
                            == Some(
                                RuntimeSpatialExpandedFallbackOutcome::CollapseToBaselineSpatial,
                            )
                })
        }));

    let rendered = report.render_json();
    assert!(rendered.contains("\"spatial_node_count\":2"));
    assert!(rendered.contains("\"active_spatial_node_count\":1"));
    assert!(rendered.contains("\"surround_bed_spatial_node_count\":1"));
    assert!(rendered.contains("\"expanded_fallback_spatial_node_count\":1"));
    assert!(rendered.contains("\"bed_class\":\"CanonicalSurroundBed\""));
    assert!(rendered.contains("\"mix_policy\":\"CollapseToBaselineSpatial\""));
    assert!(rendered.contains("\"execution_mode\":\"BalanceGroups\""));
    assert!(rendered.contains("\"fallback_outcome\":\"BypassSpatialProcessing\""));
    assert!(rendered.contains("\"expanded_fallback_outcome\":\"CollapseToBaselineSpatial\""));
}

#[test]
fn local_shared_host_edge_exports_runtime_stretch_truth() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
    runtime
        .handshake(signal_runtime::HandshakeRequest {
            client_version: "public-host-local-stretch".into(),
            anticipative_preferred: true,
            max_sample_rate_hint: Some(96_000),
        })
        .expect("public local stretch handshake should succeed");
    runtime
        .configure(RuntimeConfigRequest::new(48_000, 256))
        .expect("public local stretch configure should succeed");

    let ready_path = public_local_media_fixture_path("stretch-ready");
    write_public_test_wav(&ready_path);
    runtime
        .reconcile_media_assets(vec![signal_runtime::RuntimeMediaAssetRegistration {
            asset_id: "asset:sha256:host-local-stretch-ready".into(),
            content_hash: "host-local-stretch-ready".into(),
            source_path: ready_path.display().to_string(),
            file_name: "host-local-stretch-ready.wav".into(),
            byte_size: fs::metadata(&ready_path).unwrap().len(),
            sample_rate_hz: 48_000,
            channel_count: 1,
            duration_samples: 128,
            waveform_bin_count: 8,
        }])
        .expect("public local stretch media asset should reconcile");
    runtime
        .reconcile_warp_clips(vec![signal_runtime::RuntimeWarpClipRegistration {
            clip_id: "clip:host-local-stretch".into(),
            media_asset_id: Some("asset:sha256:host-local-stretch-ready".into()),
            mode: signal_runtime::RuntimeWarpMode::ElastiqueDraft,
            source_tempo_bpm: Some(120.0),
            anchor_timeline_samples: 0,
            start_samples: 0,
            duration_samples: 48_000,
        }])
        .expect("public local stretch warp clip should reconcile");
    runtime
        .reconcile_clip_processing_clips(vec![signal_runtime::RuntimeClipProcessingRegistration {
            clip_id: "clip:host-local-stretch".into(),
            media_asset_id: Some("asset:sha256:host-local-stretch-ready".into()),
            warp_mode: signal_runtime::RuntimeWarpMode::ElastiqueDraft,
            start_samples: 0,
            duration_samples: 48_000,
            fade_in: signal_runtime::RuntimeClipFadeEnvelope::default(),
            fade_out: signal_runtime::RuntimeClipFadeEnvelope::default(),
            clip_gain: signal_runtime::RuntimeClipGainEnvelope::default(),
        }])
        .expect("public local stretch clip-processing clip should reconcile");
    runtime
        .apply_transport_projection(signal_runtime::TransportProjection {
            playing: false,
            timeline_position_samples: 0,
            tempo_bpm: 180.0,
            loop_state: None,
        })
        .expect("public local stretch transport projection should apply");

    let host = LocalRuntimeHost::new(runtime);
    let report = host.host_supervisor_report();
    assert_eq!(
        report
            .observation
            .observation
            .stretch_engine_snapshot
            .clip_count,
        1
    );
    assert_eq!(
        report
            .observation
            .observation
            .stretch_engine_snapshot
            .ready_clip_count,
        1
    );
    assert_eq!(
        report
            .observation
            .observation
            .stretch_engine_snapshot
            .sample_domain_clip_count,
        1
    );
    assert_eq!(
        report.observation.observation.stretch_engine_snapshot.clips[0].engine_class,
        signal_runtime::RuntimeStretchEngineClass::SampleDomain
    );

    let rendered = report.render_json();
    assert!(rendered.contains("\"stretch_engine_snapshot\":{"));
    assert!(rendered.contains("\"sample_domain_clip_count\":1"));
    assert!(rendered.contains("\"engine_class\":\"SampleDomain\""));

    let _ = fs::remove_file(&ready_path);
    if let Some(path) = host
        .runtime()
        .get_media_pipeline_snapshot()
        .assets
        .first()
        .and_then(|asset| asset.cache_path.as_deref())
    {
        let _ = fs::remove_file(path);
    }
}

#[test]
fn local_shared_host_edge_exports_runtime_marker_analysis_truth() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
    runtime
        .handshake(signal_runtime::HandshakeRequest {
            client_version: "public-host-local-marker-analysis".into(),
            anticipative_preferred: true,
            max_sample_rate_hint: Some(96_000),
        })
        .expect("public local marker-analysis handshake should succeed");
    runtime
        .configure(RuntimeConfigRequest::new(48_000, 256))
        .expect("public local marker-analysis configure should succeed");

    let ready_path = public_local_media_fixture_path("marker-analysis-ready");
    write_public_transient_test_wav(&ready_path);
    runtime
        .reconcile_media_assets(vec![signal_runtime::RuntimeMediaAssetRegistration {
            asset_id: "asset:sha256:host-local-marker-analysis-ready".into(),
            content_hash: "host-local-marker-analysis-ready".into(),
            source_path: ready_path.display().to_string(),
            file_name: "host-local-marker-analysis-ready.wav".into(),
            byte_size: fs::metadata(&ready_path).unwrap().len(),
            sample_rate_hz: 48_000,
            channel_count: 1,
            duration_samples: 48_000,
            waveform_bin_count: 8,
        }])
        .expect("public local marker-analysis media asset should reconcile");
    runtime
        .reconcile_warp_clips(vec![signal_runtime::RuntimeWarpClipRegistration {
            clip_id: "clip:host-local-marker-analysis".into(),
            media_asset_id: Some("asset:sha256:host-local-marker-analysis-ready".into()),
            mode: signal_runtime::RuntimeWarpMode::ElastiqueDraft,
            source_tempo_bpm: Some(120.0),
            anchor_timeline_samples: 0,
            start_samples: 0,
            duration_samples: 48_000,
        }])
        .expect("public local marker-analysis warp clip should reconcile");
    runtime
        .reconcile_clip_processing_clips(vec![signal_runtime::RuntimeClipProcessingRegistration {
            clip_id: "clip:host-local-marker-analysis".into(),
            media_asset_id: Some("asset:sha256:host-local-marker-analysis-ready".into()),
            warp_mode: signal_runtime::RuntimeWarpMode::ElastiqueDraft,
            start_samples: 0,
            duration_samples: 48_000,
            fade_in: signal_runtime::RuntimeClipFadeEnvelope::default(),
            fade_out: signal_runtime::RuntimeClipFadeEnvelope::default(),
            clip_gain: signal_runtime::RuntimeClipGainEnvelope::default(),
        }])
        .expect("public local marker-analysis clip-processing clip should reconcile");
    runtime
        .apply_transport_projection(signal_runtime::TransportProjection {
            playing: false,
            timeline_position_samples: 0,
            tempo_bpm: 180.0,
            loop_state: None,
        })
        .expect("public local marker-analysis transport projection should apply");

    let host = LocalRuntimeHost::new(runtime);
    let report = host.host_supervisor_report();
    assert_eq!(
        report
            .observation
            .observation
            .marker_analysis_snapshot
            .clip_count,
        1
    );
    assert_eq!(
        report
            .observation
            .observation
            .marker_analysis_snapshot
            .ready_clip_count,
        1
    );
    assert_eq!(
        report
            .observation
            .observation
            .marker_analysis_snapshot
            .tempo_assist_ready_clip_count,
        1
    );
    assert!(
        report
            .observation
            .observation
            .marker_analysis_snapshot
            .warp_marker_count
            > 0
    );
    assert!(
        report
            .observation
            .observation
            .marker_analysis_snapshot
            .transient_anchor_count
            > 0
    );
    assert_eq!(
        report
            .observation
            .observation
            .marker_analysis_snapshot
            .clips[0]
            .tempo_assist_posture,
        signal_runtime::RuntimeTempoAssistPosture::Ready
    );

    let rendered = report.render_json();
    assert!(rendered.contains("\"marker_analysis_snapshot\":{"));
    assert!(rendered.contains("\"tempo_assist_ready_clip_count\":1"));
    assert!(rendered.contains("\"tempo_assist_posture\":\"Ready\""));
    assert!(rendered.contains("\"tempo_assist_hint_source\":\"SourceTempo\""));

    let _ = fs::remove_file(&ready_path);
    if let Some(path) = host
        .runtime()
        .get_media_pipeline_snapshot()
        .assets
        .first()
        .and_then(|asset| asset.cache_path.as_deref())
    {
        let _ = fs::remove_file(path);
    }
}

#[test]
fn local_shared_host_edge_exports_runtime_transform_artifact_truth() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
    runtime
        .handshake(signal_runtime::HandshakeRequest {
            client_version: "public-host-local-transform-artifact".into(),
            anticipative_preferred: true,
            max_sample_rate_hint: Some(96_000),
        })
        .expect("public local transform-artifact handshake should succeed");
    runtime
        .configure(RuntimeConfigRequest::new(48_000, 256))
        .expect("public local transform-artifact configure should succeed");

    let ready_path = public_local_media_fixture_path("transform-artifact-ready");
    write_public_transient_test_wav(&ready_path);
    runtime
        .reconcile_media_assets(vec![signal_runtime::RuntimeMediaAssetRegistration {
            asset_id: "asset:sha256:host-local-transform-artifact-ready".into(),
            content_hash: "host-local-transform-artifact-ready".into(),
            source_path: ready_path.display().to_string(),
            file_name: "host-local-transform-artifact-ready.wav".into(),
            byte_size: fs::metadata(&ready_path).unwrap().len(),
            sample_rate_hz: 48_000,
            channel_count: 1,
            duration_samples: 48_000,
            waveform_bin_count: 8,
        }])
        .expect("public local transform-artifact media asset should reconcile");
    runtime
        .reconcile_warp_clips(vec![signal_runtime::RuntimeWarpClipRegistration {
            clip_id: "clip:host-local-transform-artifact".into(),
            media_asset_id: Some("asset:sha256:host-local-transform-artifact-ready".into()),
            mode: signal_runtime::RuntimeWarpMode::ElastiqueDraft,
            source_tempo_bpm: Some(120.0),
            anchor_timeline_samples: 0,
            start_samples: 0,
            duration_samples: 48_000,
        }])
        .expect("public local transform-artifact warp clip should reconcile");
    runtime
        .reconcile_clip_processing_clips(vec![signal_runtime::RuntimeClipProcessingRegistration {
            clip_id: "clip:host-local-transform-artifact".into(),
            media_asset_id: Some("asset:sha256:host-local-transform-artifact-ready".into()),
            warp_mode: signal_runtime::RuntimeWarpMode::ElastiqueDraft,
            start_samples: 0,
            duration_samples: 48_000,
            fade_in: signal_runtime::RuntimeClipFadeEnvelope::default(),
            fade_out: signal_runtime::RuntimeClipFadeEnvelope::default(),
            clip_gain: signal_runtime::RuntimeClipGainEnvelope::default(),
        }])
        .expect("public local transform-artifact clip-processing clip should reconcile");
    runtime
        .apply_transport_projection(signal_runtime::TransportProjection {
            playing: false,
            timeline_position_samples: 0,
            tempo_bpm: 180.0,
            loop_state: None,
        })
        .expect("public local transform-artifact transport projection should apply");

    let host = LocalRuntimeHost::new(runtime);
    let report = host.host_supervisor_report();
    assert_eq!(
        report
            .observation
            .observation
            .transform_artifact_snapshot
            .clip_count,
        1
    );
    assert_eq!(
        report
            .observation
            .observation
            .transform_artifact_snapshot
            .ready_clip_count,
        1
    );
    assert_eq!(
        report
            .observation
            .observation
            .transform_artifact_snapshot
            .reusable_clip_count,
        1
    );
    assert_eq!(
        report
            .observation
            .observation
            .transform_artifact_snapshot
            .clips[0]
            .reuse_state,
        signal_runtime::RuntimeTransformArtifactReuseState::Reusable
    );

    let rendered = report.render_json();
    assert!(rendered.contains("\"transform_artifact_snapshot\":{"));
    assert!(rendered.contains("\"clip_count\":1"));
    assert!(rendered.contains("\"reusable_clip_count\":1"));
    assert!(rendered.contains("\"reuse_state\":\"Reusable\""));

    let _ = fs::remove_file(&ready_path);
    if let Some(path) = host
        .runtime()
        .get_media_pipeline_snapshot()
        .assets
        .first()
        .and_then(|asset| asset.cache_path.as_deref())
    {
        let _ = fs::remove_file(path);
    }
}

#[test]
fn local_shared_host_edge_exports_runtime_preview_transform_truth() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
    runtime
        .handshake(signal_runtime::HandshakeRequest {
            client_version: "public-host-local-preview-transform".into(),
            anticipative_preferred: true,
            max_sample_rate_hint: Some(96_000),
        })
        .expect("public local preview-transform handshake should succeed");
    runtime
        .configure(RuntimeConfigRequest::new(48_000, 256))
        .expect("public local preview-transform configure should succeed");

    let ready_path = public_local_media_fixture_path("preview-transform-ready");
    write_public_transient_test_wav(&ready_path);
    runtime
        .reconcile_media_assets(vec![signal_runtime::RuntimeMediaAssetRegistration {
            asset_id: "asset:sha256:host-local-preview-transform-ready".into(),
            content_hash: "host-local-preview-transform-ready".into(),
            source_path: ready_path.display().to_string(),
            file_name: "host-local-preview-transform-ready.wav".into(),
            byte_size: fs::metadata(&ready_path).unwrap().len(),
            sample_rate_hz: 48_000,
            channel_count: 1,
            duration_samples: 48_000,
            waveform_bin_count: 8,
        }])
        .expect("public local preview-transform media asset should reconcile");
    runtime
        .reconcile_warp_clips(vec![signal_runtime::RuntimeWarpClipRegistration {
            clip_id: "clip:host-local-preview-transform".into(),
            media_asset_id: Some("asset:sha256:host-local-preview-transform-ready".into()),
            mode: signal_runtime::RuntimeWarpMode::ElastiqueDraft,
            source_tempo_bpm: Some(120.0),
            anchor_timeline_samples: 0,
            start_samples: 0,
            duration_samples: 48_000,
        }])
        .expect("public local preview-transform warp clip should reconcile");
    runtime
        .reconcile_clip_processing_clips(vec![signal_runtime::RuntimeClipProcessingRegistration {
            clip_id: "clip:host-local-preview-transform".into(),
            media_asset_id: Some("asset:sha256:host-local-preview-transform-ready".into()),
            warp_mode: signal_runtime::RuntimeWarpMode::ElastiqueDraft,
            start_samples: 0,
            duration_samples: 48_000,
            fade_in: signal_runtime::RuntimeClipFadeEnvelope::default(),
            fade_out: signal_runtime::RuntimeClipFadeEnvelope::default(),
            clip_gain: signal_runtime::RuntimeClipGainEnvelope::default(),
        }])
        .expect("public local preview-transform clip-processing clip should reconcile");
    runtime
        .apply_transport_projection(signal_runtime::TransportProjection {
            playing: false,
            timeline_position_samples: 0,
            tempo_bpm: 180.0,
            loop_state: None,
        })
        .expect("public local preview-transform transport projection should apply");
    runtime
        .start_media_preview("asset:sha256:host-local-preview-transform-ready")
        .expect("public local preview-transform media preview should start");

    let host = LocalRuntimeHost::new(runtime);
    let report = host.host_supervisor_report();
    assert_eq!(
        report
            .observation
            .observation
            .preview_transform_snapshot
            .clip_count,
        1
    );
    assert_eq!(
        report
            .observation
            .observation
            .preview_transform_snapshot
            .ready_clip_count,
        1
    );
    assert_eq!(
        report
            .observation
            .observation
            .preview_transform_snapshot
            .active_audition_clip_count,
        1
    );
    assert_eq!(
        report
            .observation
            .observation
            .preview_transform_snapshot
            .artifact_backed_clip_count,
        1
    );
    assert_eq!(
        report
            .observation
            .observation
            .preview_transform_snapshot
            .clips[0]
            .service_class,
        signal_runtime::RuntimePreviewTransformServiceClass::ArtifactBacked
    );

    let rendered = report.render_json();
    assert!(rendered.contains("\"preview_transform_snapshot\":{"));
    assert!(rendered.contains("\"active_audition_clip_count\":1"));
    assert!(rendered.contains("\"artifact_backed_clip_count\":1"));
    assert!(rendered.contains("\"service_class\":\"ArtifactBacked\""));

    let _ = fs::remove_file(&ready_path);
    if let Some(path) = host
        .runtime()
        .get_media_pipeline_snapshot()
        .assets
        .first()
        .and_then(|asset| asset.cache_path.as_deref())
    {
        let _ = fs::remove_file(path);
    }
}

#[test]
fn local_shared_host_edge_exports_runtime_block_timing_truth() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 48));
    runtime
        .handshake(signal_runtime::HandshakeRequest {
            client_version: "public-host-edge-block-timing".into(),
            anticipative_preferred: true,
            max_sample_rate_hint: Some(96_000),
        })
        .expect("local host-edge block timing handshake should succeed");
    runtime
        .configure(RuntimeConfigRequest::new(48_000, 48))
        .expect("local host-edge block timing configure should succeed");
    apply_public_capture_graph(&mut runtime, "graph:host-local:block-timing");
    runtime
        .process_engine_block(
            1,
            1,
            synthetic_stereo_block(SampleRate(48_000), FrameCount(48), 48),
        )
        .expect("local host-edge block timing block should process");

    let host = LocalRuntimeHost::new(runtime);
    let report = host.supervisor_report();
    let performance = report.performance_snapshot();

    assert_eq!(
        report.observation.engine_block_snapshot.last_block_sequence,
        Some(1)
    );
    assert_eq!(
        report
            .observation
            .engine_block_snapshot
            .last_block_deadline_budget_ns,
        Some(1_000_000)
    );
    assert!(
        report
            .observation
            .engine_block_snapshot
            .last_block_execution_time_ns
            .expect("local host-edge block timing should expose latest execution time")
            > 0
    );
    assert_eq!(
        performance.last_block_execution_time_ns,
        report
            .observation
            .engine_block_snapshot
            .last_block_execution_time_ns
    );
    assert_eq!(
        performance.last_block_deadline_pressure,
        report
            .observation
            .engine_block_snapshot
            .last_block_deadline_pressure
    );
    assert!(matches!(
        performance.last_block_deadline_pressure,
        RuntimeBlockDeadlinePressure::Normal
            | RuntimeBlockDeadlinePressure::Elevated
            | RuntimeBlockDeadlinePressure::Critical
            | RuntimeBlockDeadlinePressure::Overrun
    ));

    let rendered = report.render_json();
    assert!(rendered.contains("\"engine_block_snapshot\":{"));
    assert!(rendered.contains("\"last_block_execution_time_ns\":"));
    assert!(rendered.contains("\"last_block_deadline_pressure\":"));
}

#[test]
fn local_shared_host_edge_exports_runtime_critical_path_truth() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 48));
    runtime
        .handshake(signal_runtime::HandshakeRequest {
            client_version: "public-host-edge-critical-path".into(),
            anticipative_preferred: true,
            max_sample_rate_hint: Some(96_000),
        })
        .expect("local host-edge critical-path handshake should succeed");
    runtime
        .configure(RuntimeConfigRequest::new(48_000, 48))
        .expect("local host-edge critical-path configure should succeed");
    apply_public_capture_graph(&mut runtime, "graph:host-local:critical-path");
    runtime
        .process_engine_block(
            1,
            1,
            synthetic_stereo_block(SampleRate(48_000), FrameCount(48), 52),
        )
        .expect("local host-edge critical-path block should process");

    let host = LocalRuntimeHost::new(runtime);
    let report = host.supervisor_report();
    let performance = report.performance_snapshot();

    assert!(performance.hot_latency_node_id.is_some());
    assert!(performance.hot_latency_group_node_count > 0);
    assert!(matches!(
        performance.critical_path_lane.as_deref(),
        Some("Realtime") | Some("Anticipative")
    ));
    assert!(!performance.worker_lane_summaries.is_empty());

    let critical_lane_summary = performance
        .worker_lane_summaries
        .iter()
        .find(|summary| {
            Some(match summary.lane {
                GraphExecutionLane::Realtime => "Realtime",
                GraphExecutionLane::Anticipative => "Anticipative",
            }) == performance.critical_path_lane.as_deref()
        })
        .expect("local host-edge critical-path lane should resolve to a typed worker summary");
    assert_eq!(
        performance.critical_path_lane_node_count,
        critical_lane_summary.node_count
    );
    assert_eq!(
        performance.critical_path_lane_total_latency_samples,
        critical_lane_summary.total_latency_samples
    );

    let rendered = performance.render_json();
    assert!(rendered.contains("\"critical_path_lane\":"));
    assert!(rendered.contains("\"worker_lane_summaries\":["));
}

#[test]
fn local_shared_host_edge_exports_runtime_deferred_work_policy_truth() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 48));
    runtime
        .handshake(signal_runtime::HandshakeRequest {
            client_version: "public-host-edge-deferred-work".into(),
            anticipative_preferred: true,
            max_sample_rate_hint: Some(96_000),
        })
        .expect("local host-edge deferred-work handshake should succeed");
    runtime
        .configure(RuntimeConfigRequest::new(48_000, 48))
        .expect("local host-edge deferred-work configure should succeed");
    apply_public_render_graph(&mut runtime, "graph:host-local:deferred-work");
    runtime
        .set_safe_mode(SafeModeRequest { enabled: true })
        .expect("enable safe mode for local deferred-work policy proof");
    runtime
        .render_offline_queue(vec![RuntimeOfflineRenderRequest {
            request_id: "render:host-local:deferred-work".into(),
            timeline_start_samples: 0,
            duration_samples: 96,
            export_sample_rate_hz: 48_000,
            include_main_mix: true,
            artifact_root_path: None,
            stem_targets: Vec::new(),
            freeze_artifacts: Vec::new(),
        }])
        .expect("safe mode should defer local host-edge deferred work");

    let host = LocalRuntimeHost::new(runtime);
    let report = host.supervisor_report();
    let receipt = report
        .observation
        .last_deferred_service_receipt
        .as_ref()
        .expect("local host-edge report should expose deferred-work policy receipt");
    assert_eq!(receipt.decision, RuntimeDeferredServiceDecision::Defer);
    assert_eq!(receipt.reason, RuntimeDeferredServiceReason::SafeMode);
    assert_eq!(
        receipt.priority_band,
        RuntimeDeferredServicePriorityBand::UserVisible
    );
    assert_eq!(
        receipt.blocking_priority_band,
        Some(RuntimeDeferredServicePriorityBand::RecoveryCritical)
    );
    assert_eq!(
        receipt.backpressure_source,
        Some(RuntimeDeferredServiceBackpressureSource::SafeMode)
    );
    assert!(receipt.starvation_risk);
    assert_eq!(receipt.starved_work_item_count, 1);

    let performance = report.performance_snapshot();
    assert_eq!(
        performance.background_service_decision,
        Some(RuntimeDeferredServiceDecision::Defer)
    );
    assert_eq!(
        performance.background_service_reason,
        Some(RuntimeDeferredServiceReason::SafeMode)
    );
    assert_eq!(
        performance.background_service_priority_band,
        Some(RuntimeDeferredServicePriorityBand::UserVisible)
    );
    assert_eq!(
        performance.background_service_backpressure_source,
        Some(RuntimeDeferredServiceBackpressureSource::SafeMode)
    );
    assert!(performance.background_service_starvation_risk);
    assert_eq!(performance.background_service_starved_work_item_count, 1);

    let rendered = report.render_json();
    assert!(rendered.contains("\"last_deferred_service\":{"));
    assert!(rendered.contains("\"priority_band\":\"UserVisible\""));
    assert!(rendered.contains("\"backpressure_source\":\"SafeMode\""));
}

#[test]
fn local_shared_host_edge_exports_resumable_offline_render_session_truth() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
    runtime
        .handshake(signal_runtime::HandshakeRequest {
            client_version: "public-host-edge-render".into(),
            anticipative_preferred: true,
            max_sample_rate_hint: Some(96_000),
        })
        .unwrap();
    runtime
        .configure(RuntimeConfigRequest::new(48_000, 512))
        .unwrap();
    apply_public_render_graph(&mut runtime, "graph:host-local:render");
    runtime
        .begin_offline_render_execution(RuntimeOfflineRenderRequest {
            request_id: "render:host-local:resumable".into(),
            timeline_start_samples: 0,
            duration_samples: 512,
            export_sample_rate_hz: 48_000,
            include_main_mix: true,
            artifact_root_path: None,
            stem_targets: Vec::new(),
            freeze_artifacts: Vec::new(),
        })
        .unwrap();
    runtime
        .advance_offline_render_execution("render:host-local:resumable")
        .unwrap();
    runtime
        .pause_offline_render_execution("render:host-local:resumable")
        .unwrap();

    let host = LocalRuntimeHost::new(runtime);
    let report = host.supervisor_report();
    assert_eq!(
        report
            .observation
            .offline_render_session_snapshot
            .active_sessions
            .first()
            .map(|session| session.interruption_class),
        Some(RuntimeInterruptionClass::Resumable)
    );
    let rendered = report.render_json();
    assert!(rendered.contains("\"offline_render_session_snapshot\":{"));
    assert!(rendered.contains("\"interruption_class\":\"Resumable\""));
}
