use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use signal_graph::{
    synthetic_stereo_block, GraphExecutionLane, GraphNodeExecutionClass, GraphNodeTopologyRole,
    GraphStageSpec,
};
use signal_host_server::ServerRuntimeHost;
use signal_plugin::{EventPacketSummary, PluginFeature, PluginFormat, PluginIoLayout};
use signal_primitives::{AudioBuffer, ChannelCount, ChannelLayout, FrameCount, SampleRate};
use signal_runtime::{
    GraphContractProjection, GraphNodeBufferContractProjection, GraphNodeContractProjection,
    GraphNodeProjection, GraphNodeTopologyProjection, GraphProjection, PluginBackedNodeBinding,
    PluginBackedNodeBindingProjection, PluginFaultKind, PluginSandboxLifecycleStage,
    PluginSandboxSpec, PluginSandboxTransportStage, PluginScanRequest, RestartRequest,
    RuntimeAuxiliaryPathKind, RuntimeBlockDeadlinePressure, RuntimeBusIntent, RuntimeBusRole,
    RuntimeCanonicalChannelLayout, RuntimeConfig, RuntimeConfigRequest,
    RuntimeDeferredServiceCancellationCause, RuntimeDeferredServiceDecision,
    RuntimeDeferredServicePriorityBand, RuntimeDeferredServiceReason, RuntimeDeploymentClass,
    RuntimeDeviceFaultBoundaryState, RuntimeDeviceRestartState, RuntimeDeviceSupervisionState,
    RuntimeError, RuntimeErrorKind, RuntimeExternalIoHealthState, RuntimeExternalIoLoopbackState,
    RuntimeExternalIoMonitoringState, RuntimeExternalIoMonitoringTapPoint,
    RuntimeExternalIoPrimaryRole, RuntimeFoldDownPolicy, RuntimeImmersiveExportAuthority,
    RuntimeImmersiveExportClass, RuntimeImmersiveExportOutcome,
    RuntimeImmersiveObjectRenderingPosture, RuntimeImmersiveRoomOutcome, RuntimeInterruptionClass,
    RuntimeJackClientRole, RuntimeJackGraphCoordinationState, RuntimeJackGuardedCoordinationState,
    RuntimeJackTransportPosture, RuntimeLifecycleApi, RuntimeMonitoringOutcome,
    RuntimeMonitoringSceneAuthority, RuntimeMonitoringSceneClass, RuntimeObservationApi,
    RuntimeOfflineRenderExecutionState, RuntimeOfflineRenderPurgeRequest,
    RuntimeOfflineRenderRequest, RuntimePluginAraContextSnapshot, RuntimePluginAraDocumentContext,
    RuntimePluginAraRegionContext, RuntimePluginAraSourceContext, RuntimePluginBusCapableFxClass,
    RuntimePluginComplexIoSummary, RuntimePluginDiscoveredTypeRecord, RuntimePluginHostPlatform,
    RuntimePluginIsolationOutcome, RuntimePluginParityBand, RuntimePluginPlacementPolicy,
    RuntimePluginPlacementRule, RuntimePluginPlacementRuleMatcher,
    RuntimePluginRecallPortabilityClass, RuntimeProjectionApi,
    RuntimeRecordingCaptureCheckpointClass, RuntimeRecordingCaptureKind,
    RuntimeRecordingCaptureStartRequest, RuntimeRecoveryState, RuntimeRendererCapabilityAuthority,
    RuntimeRendererCapabilityNegotiationPosture, RuntimeRoomPolicyAuthority,
    RuntimeRoomPolicyClass, RuntimeSecondaryInputAttachmentPolicy,
    RuntimeSecondaryInputContractProjection, RuntimeSecondaryInputFallbackOutcome,
    RuntimeSecondaryInputTargetKind, RuntimeSpatialBedClass, RuntimeSpatialExecutionMode,
    RuntimeSpatialExpandedFallbackOutcome, RuntimeSpatialFallbackOutcome, RuntimeSpatialMixPolicy,
    RuntimeSupervisorApi, RuntimeWatchdogTrigger, SignalRuntime, StopReason, WatchdogRestartRecord,
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
                        track_lane_id: Some("track:host-server:plugin-continuity".into()),
                        bus_group_id: Some("mix:host-server".into()),
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
        .expect("public server multichannel graph should apply");
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
                        track_lane_id: Some("track:host-server:surround".into()),
                        bus_group_id: Some("mix:host-server:surround".into()),
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
                        track_lane_id: Some("track:host-server:surround".into()),
                        bus_group_id: Some("mix:host-server:surround".into()),
                        console_group_id: None,
                        send_return_id: Some("send:return:host-server:analysis".into()),
                    },
                },
            ],
        })
        .expect("public server multichannel contracts should apply");
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
        .expect("public server sidechain graph should apply");
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
                        track_lane_id: Some("track:host-server:sidechain".into()),
                        bus_group_id: Some("mix:host-server:sidechain".into()),
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
                        track_lane_id: Some("track:host-server:sidechain".into()),
                        bus_group_id: Some("mix:host-server:sidechain".into()),
                        console_group_id: None,
                        send_return_id: None,
                    },
                },
            ],
        })
        .expect("public server sidechain contracts should apply");
    runtime
        .apply_plugin_backed_node_bindings(PluginBackedNodeBindingProjection {
            graph_id: graph_id.into(),
            bindings: vec![PluginBackedNodeBinding {
                node_id: "compressor".into(),
                sandbox_id: "sandbox:host-server:sidechain".into(),
            }],
        })
        .expect("public server sidechain bindings should apply");
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
        .expect("public server multi-bus graph should apply");
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
                        track_lane_id: Some("track:host-server:multi-bus".into()),
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
                        console_group_id: Some("console:host-server:main".into()),
                        send_return_id: None,
                    },
                },
            ],
        })
        .expect("public server multi-bus contracts should apply");
}

fn sample_complex_multi_output_record() -> RuntimePluginDiscoveredTypeRecord {
    RuntimePluginDiscoveredTypeRecord {
        plugin_type_id: "plugin:vst3:host-server-multiout".into(),
        plugin_id: "com.signal.host-server-multiout".into(),
        vendor: "Signal".into(),
        name: "Signal Host Server Multi Output".into(),
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
        lv2_extension_capabilities: None,
        summary: "server complex multi-output instrument".into(),
    }
}

fn sample_complex_bus_fx_record() -> RuntimePluginDiscoveredTypeRecord {
    RuntimePluginDiscoveredTypeRecord {
        plugin_type_id: "plugin:vst3:host-server-bus-fx".into(),
        plugin_id: "com.signal.host-server-bus-fx".into(),
        vendor: "Signal".into(),
        name: "Signal Host Server Bus FX".into(),
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
        summary: "server bus-capable fx".into(),
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
        .expect("public server complex io graph should apply");
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
                        track_lane_id: Some("track:host-server:complex-io".into()),
                        bus_group_id: Some("mix:host-server:complex-io".into()),
                        console_group_id: None,
                        send_return_id: None,
                    },
                },
                GraphNodeContractProjection {
                    node_id: "plugin-bus-fx".into(),
                    buffer_contract: GraphNodeBufferContractProjection::default(),
                    topology: GraphNodeTopologyProjection {
                        role: Some(GraphNodeTopologyRole::TrackLane),
                        track_lane_id: Some("track:host-server:complex-io".into()),
                        bus_group_id: Some("mix:host-server:complex-io".into()),
                        console_group_id: None,
                        send_return_id: None,
                    },
                },
            ],
        })
        .expect("public server complex io contracts should apply");
    runtime
        .apply_plugin_backed_node_bindings(PluginBackedNodeBindingProjection {
            graph_id: graph_id.into(),
            bindings: vec![
                PluginBackedNodeBinding {
                    node_id: "plugin-multiout".into(),
                    sandbox_id: "sandbox:host-server:multiout".into(),
                },
                PluginBackedNodeBinding {
                    node_id: "plugin-bus-fx".into(),
                    sandbox_id: "sandbox:host-server:bus-fx".into(),
                },
            ],
        })
        .expect("public server complex io bindings should apply");
    runtime.record_plugin_sandbox_spec(&PluginSandboxSpec {
        sandbox_id: "sandbox:host-server:multiout".into(),
        plugin_format: PluginFormat::Vst3,
        plugin_type_id: Some("plugin:vst3:host-server-multiout".into()),
    });
    runtime.record_plugin_sandbox_spec(&PluginSandboxSpec {
        sandbox_id: "sandbox:host-server:bus-fx".into(),
        plugin_format: PluginFormat::Vst3,
        plugin_type_id: Some("plugin:vst3:host-server-bus-fx".into()),
    });
    runtime.record_plugin_sandbox_lifecycle(
        "sandbox:host-server:multiout",
        PluginSandboxLifecycleStage::InstancePrepared,
        Some(1),
    );
    runtime.record_plugin_sandbox_transport(
        "sandbox:host-server:multiout",
        "lease-host-server-multiout",
        "region-host-server-multiout",
        PluginSandboxTransportStage::Attached,
        Some(1),
        Some("host server complex io multiout attached".into()),
    );
    runtime.record_plugin_sandbox_lifecycle(
        "sandbox:host-server:bus-fx",
        PluginSandboxLifecycleStage::SandboxRestarted,
        Some(2),
    );
    runtime.record_plugin_sandbox_lifecycle(
        "sandbox:host-server:bus-fx",
        PluginSandboxLifecycleStage::InstancePrepared,
        Some(2),
    );
    runtime.record_plugin_sandbox_transport(
        "sandbox:host-server:bus-fx",
        "lease-host-server-bus-fx",
        "region-host-server-bus-fx",
        PluginSandboxTransportStage::Attached,
        Some(2),
        Some("host server complex io bus fx attached".into()),
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
        .expect("public server spatial graph should apply");
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
                        track_lane_id: Some("track:host-server:spatial-stereo".into()),
                        bus_group_id: Some("bus:host-server:spatial-stereo".into()),
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
                        track_lane_id: Some("track:host-server:spatial-surround".into()),
                        bus_group_id: Some("bus:host-server:spatial-surround".into()),
                        console_group_id: None,
                        send_return_id: None,
                    },
                },
            ],
        })
        .expect("public server spatial contracts should apply");
    runtime
        .apply_plugin_backed_node_bindings(PluginBackedNodeBindingProjection {
            graph_id: graph_id.into(),
            bindings: vec![
                PluginBackedNodeBinding {
                    node_id: "spatial-stereo".into(),
                    sandbox_id: "sandbox:host-server:spatial-stereo".into(),
                },
                PluginBackedNodeBinding {
                    node_id: "spatial-surround".into(),
                    sandbox_id: "sandbox:host-server:spatial-surround".into(),
                },
            ],
        })
        .expect("public server spatial bindings should apply");
    runtime.record_plugin_sandbox_lifecycle(
        "sandbox:host-server:spatial-stereo",
        PluginSandboxLifecycleStage::InstancePrepared,
        Some(1),
    );
    runtime.record_plugin_sandbox_lifecycle(
        "sandbox:host-server:spatial-surround",
        PluginSandboxLifecycleStage::InstancePrepared,
        Some(1),
    );
}

fn public_server_media_fixture_path(label: &str) -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock should be monotonic enough for test files")
        .as_nanos();
    std::env::temp_dir().join(format!(
        "signal-host-server-public-media-{label}-{}-{unique}.wav",
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
    fs::write(path, bytes).expect("public server media fixture should be written");
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
    fs::write(path, bytes).expect("public server transient media fixture should be written");
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

fn sample_server_ara_context() -> RuntimePluginAraContextSnapshot {
    RuntimePluginAraContextSnapshot {
        portability_class: RuntimePluginRecallPortabilityClass::ContextOnly,
        document_context: Some(RuntimePluginAraDocumentContext {
            document_id: "doc:host-server".into(),
            display_label: Some("Server Session".into()),
            summary: "server host ara document".into(),
        }),
        source_context: Some(RuntimePluginAraSourceContext {
            source_id: "source:stem-bus".into(),
            display_label: Some("Stem Bus".into()),
            summary: "server host ara source".into(),
        }),
        region_context: Some(RuntimePluginAraRegionContext {
            region_id: "region:bridge".into(),
            display_label: Some("Bridge".into()),
            timeline_start_samples: Some(16_384),
            duration_samples: Some(4_096),
            summary: "server host ara region".into(),
        }),
        summary: "server host ara context".into(),
    }
}

#[test]
fn server_shared_host_edge_is_consumable_without_private_helpers() {
    let runtime = SignalRuntime::new(RuntimeConfig::server(48_000, 512));
    let mut host = ServerRuntimeHost::new(runtime);

    host.start_plugin_scan(PluginScanRequest {
        roots: vec!["/srv/plugins/clap".into()],
        formats: vec![PluginFormat::Clap],
    })
    .expect("public host-edge scan should succeed");
    host.ensure_plugin_sandbox(PluginSandboxSpec {
        sandbox_id: "public-host-edge-server".into(),
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
        4
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
    assert!(rendered.contains("\"plugin_type_id\":\"plugin:clap:server\""));
    assert!(rendered.contains("\"event_stream\":"));
}

#[test]
fn server_shared_host_edge_exports_restartable_and_terminal_recording_checkpoint_truth() {
    let mut restartable_runtime = SignalRuntime::new(RuntimeConfig::server(48_000, 512));
    restartable_runtime
        .handshake(signal_runtime::HandshakeRequest {
            client_version: "public-host-edge-recording".into(),
            anticipative_preferred: true,
            max_sample_rate_hint: Some(96_000),
        })
        .unwrap();
    restartable_runtime
        .configure(RuntimeConfigRequest::new(48_000, 512))
        .unwrap();
    apply_public_capture_graph(
        &mut restartable_runtime,
        "graph:host-server:recording-restartable",
    );
    restartable_runtime.start().unwrap();
    restartable_runtime
        .start_recording_capture(RuntimeRecordingCaptureStartRequest {
            capture_kind: RuntimeRecordingCaptureKind::Audio,
            take_id: "take:server:restartable".into(),
            track_id: "track:server:restartable".into(),
            start_samples: 3_072,
            capture_path: std::env::temp_dir()
                .join("signal-server-host-recording-restartable.wav")
                .display()
                .to_string(),
        })
        .unwrap();
    restartable_runtime
        .process_engine_block(
            1,
            1,
            synthetic_stereo_block(SampleRate(48_000), FrameCount(8), 62),
        )
        .unwrap();
    restartable_runtime
        .stop(StopReason::DeviceReconfigure)
        .unwrap();

    let restartable_host = ServerRuntimeHost::new(restartable_runtime);
    let restartable_report = restartable_host.supervisor_report();
    assert_eq!(
        restartable_report
            .observation
            .recording_capture_snapshot
            .last_checkpoint
            .as_ref()
            .map(|checkpoint| checkpoint.interruption_class),
        Some(RuntimeInterruptionClass::Restartable)
    );
    assert_eq!(
        restartable_report
            .observation
            .recording_capture_snapshot
            .last_checkpoint
            .as_ref()
            .map(|checkpoint| checkpoint.checkpoint_class),
        Some(RuntimeRecordingCaptureCheckpointClass::Buffered)
    );

    let mut terminal_runtime = SignalRuntime::new(RuntimeConfig::server(48_000, 512));
    terminal_runtime
        .handshake(signal_runtime::HandshakeRequest {
            client_version: "public-host-edge-recording".into(),
            anticipative_preferred: true,
            max_sample_rate_hint: Some(96_000),
        })
        .unwrap();
    terminal_runtime
        .configure(RuntimeConfigRequest::new(48_000, 512))
        .unwrap();
    apply_public_capture_graph(
        &mut terminal_runtime,
        "graph:host-server:recording-terminal",
    );
    terminal_runtime
        .start_recording_capture(RuntimeRecordingCaptureStartRequest {
            capture_kind: RuntimeRecordingCaptureKind::Audio,
            take_id: "take:server:terminal".into(),
            track_id: "track:server:terminal".into(),
            start_samples: 4_096,
            capture_path: "/dev/null/signal-server-host-recording-terminal.wav".into(),
        })
        .unwrap();
    terminal_runtime
        .process_engine_block(
            1,
            1,
            synthetic_stereo_block(SampleRate(48_000), FrameCount(8), 63),
        )
        .unwrap();
    let _ = terminal_runtime.finish_recording_capture().unwrap_err();

    let terminal_host = ServerRuntimeHost::new(terminal_runtime);
    let terminal_report = terminal_host.supervisor_report();
    assert_eq!(
        terminal_report.observation.recording_capture_snapshot.state,
        Some(signal_runtime::RuntimeRecordingCaptureState::Failed)
    );
    assert_eq!(
        terminal_report
            .observation
            .recording_capture_snapshot
            .last_checkpoint
            .as_ref()
            .map(|checkpoint| checkpoint.interruption_class),
        Some(RuntimeInterruptionClass::Terminal)
    );

    let rendered = terminal_report.render_json();
    assert!(rendered.contains("\"recording_capture_snapshot\":{"));
    assert!(rendered.contains("\"interruption_class\":\"Terminal\""));
}

#[test]
fn server_shared_host_edge_exports_plugin_placement_and_shared_boundary_continuity_truth() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::server(48_000, 512));
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
                    "plugin://host-server-shared".into(),
                ),
                outcome: RuntimePluginIsolationOutcome::SharedSandbox,
                sandbox_group_key: Some("shared:host-server".into()),
            }],
        })
        .unwrap();
    apply_public_plugin_continuity_graph(
        &mut runtime,
        "graph:host-server:plugin-continuity",
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
        "plugin://host-server-shared",
        1,
    );
    record_public_plugin_sandbox_ready(
        &mut runtime,
        "sandbox-isolated",
        PluginFormat::Clap,
        "plugin://host-server-isolated",
        1,
    );
    runtime.record_plugin_sandbox_fault(
        "sandbox-shared",
        PluginFaultKind::Crash,
        "server shared crash",
        Some(2),
    );
    runtime.record_plugin_sandbox_fault(
        "sandbox-shared",
        PluginFaultKind::Timeout,
        "server shared timeout",
        Some(3),
    );

    let host = ServerRuntimeHost::new(runtime);
    let report = host.supervisor_report();
    let shared = report
        .observation
        .plugin_lifecycle_snapshot
        .sandboxes
        .iter()
        .find(|sandbox| sandbox.sandbox_id == "sandbox-shared")
        .expect("shared host-server boundary should be visible");
    assert_eq!(
        shared.placement_outcome,
        RuntimePluginIsolationOutcome::SharedSandbox
    );
    assert_eq!(
        shared.placement_rule_id.as_deref(),
        Some("share-verified-clap")
    );
    assert_eq!(shared.sandbox_group_key, "shared:host-server");
    assert_eq!(shared.shared_boundary_member_count, 2);
    assert_eq!(shared.continuity_class, RuntimeInterruptionClass::Terminal);
    assert!(!shared.rebindable);
    let isolated = report
        .observation
        .plugin_lifecycle_snapshot
        .sandboxes
        .iter()
        .find(|sandbox| sandbox.sandbox_id == "sandbox-isolated")
        .expect("isolated host-server boundary should remain visible");
    assert_eq!(isolated.continuity_class, RuntimeInterruptionClass::Steady);

    let rendered = report.render_json();
    assert!(rendered.contains("\"plugin_lifecycle_snapshot\":{"));
    assert!(rendered.contains("\"placement_outcome\":\"SharedSandbox\""));
    assert!(rendered.contains("\"sandbox_group_key\":\"shared:host-server\""));
    assert!(rendered.contains("\"shared_boundary_member_count\":2"));
    assert!(rendered.contains("\"continuity_class\":\"Terminal\""));
}

#[test]
fn server_shared_host_edge_exports_runtime_vst3_baseline_truth() {
    let runtime = SignalRuntime::new(RuntimeConfig::server(48_000, 512));
    let mut host = ServerRuntimeHost::new(runtime);

    host.start_plugin_scan(PluginScanRequest {
        roots: vec!["~/.vst3".into(), "/usr/lib/vst3".into()],
        formats: vec![PluginFormat::Vst3],
    })
    .expect("public server vst3 scan should succeed");
    host.ensure_plugin_sandbox(PluginSandboxSpec {
        sandbox_id: "public-host-edge-server-vst3".into(),
        plugin_format: PluginFormat::Vst3,
        plugin_type_id: Some("plugin:vst3:linux-synth".into()),
    })
    .expect("public server vst3 sandbox ensure should succeed");

    let report = host.supervisor_report();
    assert_eq!(
        report
            .observation
            .plugin_discovery_snapshot
            .discovered_type_count,
        4
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
        .any(|plugin| plugin.plugin_type_id == "plugin:vst3:linux-synth"
            && plugin.format == PluginFormat::Vst3));
    let sandbox = report
        .observation
        .plugin_lifecycle_snapshot
        .sandboxes
        .iter()
        .find(|sandbox| sandbox.sandbox_id == "public-host-edge-server-vst3")
        .expect("public server vst3 sandbox should be exported");
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
    assert!(rendered.contains("\"plugin_type_id\":\"plugin:vst3:linux-synth\""));
    assert!(rendered.contains("\"formats\":[\"Vst3\"]"));
}

#[test]
fn server_shared_host_edge_exports_runtime_au_baseline_truth() {
    let runtime = SignalRuntime::new(RuntimeConfig::server(48_000, 512));
    let mut host = ServerRuntimeHost::new(runtime);

    host.start_plugin_scan(PluginScanRequest {
        roots: vec!["~/Library/Audio/Plug-Ins/Components".into()],
        formats: vec![PluginFormat::Au],
    })
    .expect("public server au scan should succeed");
    host.ensure_plugin_sandbox(PluginSandboxSpec {
        sandbox_id: "public-host-edge-server-au".into(),
        plugin_format: PluginFormat::Au,
        plugin_type_id: Some("plugin:au:instrument".into()),
    })
    .expect("public server au sandbox ensure should succeed");

    let report = host.supervisor_report();
    assert_eq!(
        report
            .observation
            .plugin_discovery_snapshot
            .discovered_type_count,
        4
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
        .find(|sandbox| sandbox.sandbox_id == "public-host-edge-server-au")
        .expect("public server au sandbox should be exported");
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
fn server_shared_host_edge_exports_runtime_lv2_baseline_truth() {
    let runtime = SignalRuntime::new(RuntimeConfig::server(48_000, 512));
    let mut host = ServerRuntimeHost::new(runtime);

    host.start_plugin_scan(PluginScanRequest {
        roots: vec!["~/.lv2".into(), "/usr/lib/lv2".into()],
        formats: vec![PluginFormat::Lv2],
    })
    .expect("public server lv2 scan should succeed");
    host.ensure_plugin_sandbox(PluginSandboxSpec {
        sandbox_id: "public-host-edge-server-lv2".into(),
        plugin_format: PluginFormat::Lv2,
        plugin_type_id: Some("plugin:lv2:linux-synth".into()),
    })
    .expect("public server lv2 sandbox ensure should succeed");

    let report = host.supervisor_report();
    assert_eq!(
        report
            .observation
            .plugin_discovery_snapshot
            .discovered_type_count,
        4
    );
    assert_eq!(
        report
            .observation
            .plugin_discovery_snapshot
            .last_scan
            .as_ref()
            .map(|scan| scan.formats.clone()),
        Some(vec![PluginFormat::Lv2])
    );
    assert!(report
        .observation
        .plugin_discovery_snapshot
        .discovered_types
        .iter()
        .any(|plugin| plugin.plugin_type_id == "plugin:lv2:linux-synth"
            && plugin.format == PluginFormat::Lv2));
    let parity = report
        .observation
        .plugin_discovery_snapshot
        .parity_coverage
        .iter()
        .find(|record| record.format == PluginFormat::Lv2)
        .expect("public server lv2 parity should be exported");
    assert_eq!(
        parity.supported_platforms,
        vec![RuntimePluginHostPlatform::Linux]
    );
    assert_eq!(
        parity.unsupported_platforms,
        vec![
            RuntimePluginHostPlatform::MacOs,
            RuntimePluginHostPlatform::Windows,
        ]
    );
    let sandbox = report
        .observation
        .plugin_lifecycle_snapshot
        .sandboxes
        .iter()
        .find(|sandbox| sandbox.sandbox_id == "public-host-edge-server-lv2")
        .expect("public server lv2 sandbox should be exported");
    assert_eq!(sandbox.plugin_format, Some(PluginFormat::Lv2));
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
    assert!(rendered.contains("\"plugin_type_id\":\"plugin:lv2:linux-synth\""));
    assert!(rendered.contains("\"formats\":[\"Lv2\"]"));
    assert!(rendered.contains("\"supported_platforms\":[\"Linux\"]"));
}

#[test]
fn server_shared_host_edge_exports_runtime_cross_adapter_parity_truth() {
    let runtime = SignalRuntime::new(RuntimeConfig::server(48_000, 512));
    let mut host = ServerRuntimeHost::new(runtime);

    host.start_plugin_scan(PluginScanRequest {
        roots: vec![
            "~/.clap".into(),
            "/usr/lib/vst3".into(),
            "~/Library/Audio/Plug-Ins/Components".into(),
        ],
        formats: vec![PluginFormat::Clap, PluginFormat::Vst3, PluginFormat::Au],
    })
    .expect("public server parity scan should succeed");
    host.ensure_plugin_sandbox(PluginSandboxSpec {
        sandbox_id: "public-host-edge-server-parity-vst3".into(),
        plugin_format: PluginFormat::Vst3,
        plugin_type_id: Some("plugin:vst3:linux-synth".into()),
    })
    .expect("public server parity vst3 sandbox ensure should succeed");
    host.ensure_plugin_sandbox(PluginSandboxSpec {
        sandbox_id: "public-host-edge-server-parity-au".into(),
        plugin_format: PluginFormat::Au,
        plugin_type_id: Some("plugin:au:instrument".into()),
    })
    .expect("public server parity au sandbox ensure should succeed");

    let report = host.supervisor_report();
    let discovery = &report.observation.plugin_discovery_snapshot;
    assert_eq!(discovery.parity_coverage.len(), 3);
    let clap_parity = discovery
        .parity_coverage
        .iter()
        .find(|record| record.format == PluginFormat::Clap)
        .expect("public server parity report should include clap parity");
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
        .expect("public server parity report should include au parity");
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
        .expect("public server parity lifecycle should include au parity");
    assert_eq!(lifecycle_au.sandbox_count, 1);
    assert_eq!(lifecycle_au.ready_sandbox_count, 1);
    assert_eq!(lifecycle_au.active_transport_count, 1);
    let lifecycle_vst3 = report
        .observation
        .plugin_lifecycle_snapshot
        .parity_coverage
        .iter()
        .find(|record| record.format == PluginFormat::Vst3)
        .expect("public server parity lifecycle should include vst3 parity");
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
fn server_shared_host_edge_exports_runtime_linux_plugin_parity_truth() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::server(48_000, 512));
    runtime
        .handshake(signal_runtime::HandshakeRequest {
            client_version: "public-server-linux-plugin-parity".into(),
            anticipative_preferred: true,
            max_sample_rate_hint: Some(96_000),
        })
        .expect("public server linux parity handshake should succeed");
    runtime
        .configure(RuntimeConfigRequest::new(48_000, 512))
        .expect("public server linux parity configure should succeed");
    runtime.record_plugin_format_platform_coverage(vec![
        signal_runtime::RuntimePluginFormatPlatformCoverageRecord {
            format: PluginFormat::Clap,
            supported_platforms: vec![
                RuntimePluginHostPlatform::MacOs,
                RuntimePluginHostPlatform::Linux,
                RuntimePluginHostPlatform::Windows,
            ],
            unsupported_platforms: Vec::new(),
            linux_parity_band: RuntimePluginParityBand::Portable,
            linux_preferred_sandbox_outcome: Some(RuntimePluginIsolationOutcome::IsolatedSandbox),
            linux_strict_sandbox_default: true,
            summary:
                "platforms=MacOs/Linux/Windows linux=Portable linux_policy=IsolatedSandbox unsupported=none"
                    .into(),
        },
        signal_runtime::RuntimePluginFormatPlatformCoverageRecord {
            format: PluginFormat::Vst3,
            supported_platforms: vec![
                RuntimePluginHostPlatform::MacOs,
                RuntimePluginHostPlatform::Linux,
                RuntimePluginHostPlatform::Windows,
            ],
            unsupported_platforms: Vec::new(),
            linux_parity_band: RuntimePluginParityBand::Portable,
            linux_preferred_sandbox_outcome: Some(RuntimePluginIsolationOutcome::IsolatedSandbox),
            linux_strict_sandbox_default: true,
            summary:
                "platforms=MacOs/Linux/Windows linux=Portable linux_policy=IsolatedSandbox unsupported=none"
                    .into(),
        },
        signal_runtime::RuntimePluginFormatPlatformCoverageRecord {
            format: PluginFormat::Lv2,
            supported_platforms: vec![RuntimePluginHostPlatform::Linux],
            unsupported_platforms: vec![
                RuntimePluginHostPlatform::MacOs,
                RuntimePluginHostPlatform::Windows,
            ],
            linux_parity_band: RuntimePluginParityBand::Portable,
            linux_preferred_sandbox_outcome: Some(RuntimePluginIsolationOutcome::IsolatedSandbox),
            linux_strict_sandbox_default: true,
            summary:
                "platforms=Linux linux=Portable linux_policy=IsolatedSandbox unsupported=MacOs/Windows"
                    .into(),
        },
    ]);
    runtime
        .apply_plugin_placement_policy(RuntimePluginPlacementPolicy {
            default_outcome: RuntimePluginIsolationOutcome::IsolatedSandbox,
            rules: vec![
                RuntimePluginPlacementRule {
                    rule_id: "server-linux-share-clap".into(),
                    matcher: RuntimePluginPlacementRuleMatcher::PluginFormat(PluginFormat::Clap),
                    outcome: RuntimePluginIsolationOutcome::SharedSandbox,
                    sandbox_group_key: Some("linux:clap".into()),
                },
                RuntimePluginPlacementRule {
                    rule_id: "server-linux-inline-vst3".into(),
                    matcher: RuntimePluginPlacementRuleMatcher::PluginFormat(PluginFormat::Vst3),
                    outcome: RuntimePluginIsolationOutcome::InProcess,
                    sandbox_group_key: None,
                },
            ],
        })
        .expect("public server linux parity placement policy should apply");

    let scan_handle = runtime.record_plugin_scan_request(&PluginScanRequest {
        roots: vec![
            "~/.clap".into(),
            "/usr/lib/vst3".into(),
            "/usr/lib/lv2".into(),
        ],
        formats: vec![PluginFormat::Clap, PluginFormat::Vst3, PluginFormat::Lv2],
    });
    runtime.record_plugin_scan_results(
        scan_handle,
        vec![
            signal_runtime::RuntimePluginDiscoveredTypeRecord {
                plugin_type_id: "plugin:clap:server-linux-parity".into(),
                plugin_id: "com.signal.server-linux-parity-clap".into(),
                vendor: "Signal".into(),
                name: "Server Linux Parity CLAP".into(),
                format: PluginFormat::Clap,
                version: Some("1.0.0".into()),
                features: vec![PluginFeature::AudioEffect],
                default_io_layout: PluginIoLayout {
                    audio_inputs: 2,
                    audio_outputs: 2,
                    midi_inputs: 0,
                    midi_outputs: 0,
                },
                default_multichannel_io:
                    signal_runtime::RuntimeMultichannelIoSummary::for_plugin_io(PluginIoLayout {
                        audio_inputs: 2,
                        audio_outputs: 2,
                        midi_inputs: 0,
                        midi_outputs: 0,
                    }),
                complex_io_summary: RuntimePluginComplexIoSummary::from_plugin_features_and_layout(
                    &[PluginFeature::AudioEffect],
                    PluginIoLayout {
                        audio_inputs: 2,
                        audio_outputs: 2,
                        midi_inputs: 0,
                        midi_outputs: 0,
                    },
                ),
                audio_bus_count: 1,
                parameter_count: 8,
                state_contract: signal_plugin::PluginStateContract {
                    supports_snapshot: true,
                    supports_reset: true,
                    supports_bypass: true,
                    exposes_latency: false,
                    exposes_tail: false,
                },
                processing_contract: signal_plugin::PluginProcessingContract {
                    max_block_frames: 2048,
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
                summary: "server linux parity clap".into(),
            },
            RuntimePluginDiscoveredTypeRecord {
                plugin_type_id: "plugin:vst3:server-linux-parity".into(),
                plugin_id: "com.signal.server-linux-parity-vst3".into(),
                vendor: "Signal".into(),
                name: "Server Linux Parity VST3".into(),
                format: PluginFormat::Vst3,
                version: Some("1.0.0".into()),
                features: vec![PluginFeature::Instrument],
                default_io_layout: PluginIoLayout {
                    audio_inputs: 0,
                    audio_outputs: 2,
                    midi_inputs: 1,
                    midi_outputs: 0,
                },
                default_multichannel_io:
                    signal_runtime::RuntimeMultichannelIoSummary::for_plugin_io(PluginIoLayout {
                        audio_inputs: 0,
                        audio_outputs: 2,
                        midi_inputs: 1,
                        midi_outputs: 0,
                    }),
                complex_io_summary: RuntimePluginComplexIoSummary::from_plugin_features_and_layout(
                    &[PluginFeature::Instrument],
                    PluginIoLayout {
                        audio_inputs: 0,
                        audio_outputs: 2,
                        midi_inputs: 1,
                        midi_outputs: 0,
                    },
                ),
                audio_bus_count: 1,
                parameter_count: 12,
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
                    requires_main_thread_for_state: false,
                    supports_prepare: true,
                    supports_activate: true,
                    supports_reset_while_active: true,
                },
                lv2_extension_capabilities: None,
                summary: "server linux parity vst3".into(),
            },
            RuntimePluginDiscoveredTypeRecord {
                plugin_type_id: "plugin:lv2:server-linux-parity".into(),
                plugin_id: "com.signal.server-linux-parity-lv2".into(),
                vendor: "Signal".into(),
                name: "Server Linux Parity LV2".into(),
                format: PluginFormat::Lv2,
                version: Some("1.0.0".into()),
                features: vec![PluginFeature::Utility],
                default_io_layout: PluginIoLayout {
                    audio_inputs: 2,
                    audio_outputs: 2,
                    midi_inputs: 0,
                    midi_outputs: 0,
                },
                default_multichannel_io:
                    signal_runtime::RuntimeMultichannelIoSummary::for_plugin_io(PluginIoLayout {
                        audio_inputs: 2,
                        audio_outputs: 2,
                        midi_inputs: 0,
                        midi_outputs: 0,
                    }),
                complex_io_summary: RuntimePluginComplexIoSummary::from_plugin_features_and_layout(
                    &[PluginFeature::Utility],
                    PluginIoLayout {
                        audio_inputs: 2,
                        audio_outputs: 2,
                        midi_inputs: 0,
                        midi_outputs: 0,
                    },
                ),
                audio_bus_count: 1,
                parameter_count: 6,
                state_contract: signal_plugin::PluginStateContract {
                    supports_snapshot: true,
                    supports_reset: true,
                    supports_bypass: true,
                    exposes_latency: false,
                    exposes_tail: false,
                },
                processing_contract: signal_plugin::PluginProcessingContract {
                    max_block_frames: 2048,
                    sample_accurate_automation: false,
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
                summary: "server linux parity lv2".into(),
            },
        ],
    );

    runtime.record_plugin_sandbox_spec(&PluginSandboxSpec {
        sandbox_id: "server-linux-clap-sandbox".into(),
        plugin_format: PluginFormat::Clap,
        plugin_type_id: Some("plugin:clap:server-linux-parity".into()),
    });
    runtime.record_plugin_sandbox_lifecycle(
        "server-linux-clap-sandbox",
        PluginSandboxLifecycleStage::InstancePrepared,
        Some(1),
    );
    runtime.record_plugin_sandbox_transport(
        "server-linux-clap-sandbox",
        "lease-server-linux-clap",
        "region-server-linux-clap",
        PluginSandboxTransportStage::Attached,
        Some(1),
        None,
    );

    runtime.record_plugin_sandbox_spec(&PluginSandboxSpec {
        sandbox_id: "server-linux-vst3-sandbox".into(),
        plugin_format: PluginFormat::Vst3,
        plugin_type_id: Some("plugin:vst3:server-linux-parity".into()),
    });
    runtime.record_recovery_cycle(
        "server-linux-vst3-sandbox",
        signal_runtime::RecoveryRestartIntent::CrashRecovery,
        StopReason::DegradedModeRecovery,
        Some(2),
    );
    runtime.record_plugin_sandbox_lifecycle(
        "server-linux-vst3-sandbox",
        PluginSandboxLifecycleStage::SandboxRestarted,
        Some(2),
    );

    runtime.record_plugin_sandbox_spec(&PluginSandboxSpec {
        sandbox_id: "server-linux-lv2-sandbox".into(),
        plugin_format: PluginFormat::Lv2,
        plugin_type_id: Some("plugin:lv2:server-linux-parity".into()),
    });
    runtime.record_plugin_sandbox_fault(
        "server-linux-lv2-sandbox",
        PluginFaultKind::Crash,
        "server linux lv2 parity fault",
        Some(3),
    );

    let host = ServerRuntimeHost::new(runtime);
    let report = host.supervisor_report();
    let discovery = &report.observation.plugin_discovery_snapshot;

    let clap = discovery
        .parity_coverage
        .iter()
        .find(|record| record.format == PluginFormat::Clap)
        .expect("server linux clap parity should be exported");
    assert_eq!(clap.linux_parity_band, RuntimePluginParityBand::Portable);
    assert!(clap.linux_supported);
    assert_eq!(
        clap.linux_preferred_sandbox_outcome,
        Some(RuntimePluginIsolationOutcome::IsolatedSandbox)
    );
    assert!(clap.linux_strict_sandbox_default);

    let vst3 = report
        .observation
        .plugin_lifecycle_snapshot
        .parity_coverage
        .iter()
        .find(|record| record.format == PluginFormat::Vst3)
        .expect("server linux vst3 parity should be exported");
    assert_eq!(vst3.linux_parity_band, RuntimePluginParityBand::Portable);
    assert!(vst3.linux_supported);
    assert_eq!(vst3.in_process_sandbox_count, 1);
    assert_eq!(vst3.restarting_sandbox_count, 1);
    assert_eq!(vst3.rebindable_sandbox_count, 1);

    let lv2 = report
        .observation
        .plugin_lifecycle_snapshot
        .parity_coverage
        .iter()
        .find(|record| record.format == PluginFormat::Lv2)
        .expect("server linux lv2 parity should be exported");
    assert_eq!(lv2.linux_parity_band, RuntimePluginParityBand::Portable);
    assert!(lv2.linux_supported);
    assert_eq!(lv2.faulted_sandbox_count, 1);
    assert_eq!(
        lv2.unsupported_platforms,
        vec![
            RuntimePluginHostPlatform::MacOs,
            RuntimePluginHostPlatform::Windows,
        ]
    );

    let rendered = report.render_json();
    assert!(rendered.contains("\"linux_parity_band\":\"Portable\""));
    assert!(rendered.contains("\"linux_supported\":true"));
    assert!(rendered.contains("\"linux_preferred_sandbox_outcome\":\"IsolatedSandbox\""));
    assert!(rendered.contains("\"restarting_sandbox_count\":1"));
    assert!(rendered.contains("\"faulted_sandbox_count\":1"));
}

#[test]
fn server_shared_host_edge_exports_runtime_generic_event_truth() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::server(48_000, 512));
    runtime.record_plugin_event_summary(
        11,
        "lease:public-server-events",
        18,
        212,
        EventPacketSummary {
            total_events: 9,
            parameter_value_events: 1,
            parameter_modulation_events: 1,
            parameter_gesture_events: 1,
            note_events: 2,
            note_expression_events: 3,
            note_expression_pressure_events: 1,
            note_expression_timbre_events: 1,
            note_expression_tuning_events: 1,
            midi_events: 1,
        },
    );
    let mut host = ServerRuntimeHost::new(runtime);

    host.start_plugin_scan(PluginScanRequest {
        roots: vec!["~/.clap".into(), "/usr/lib/vst3".into()],
        formats: vec![PluginFormat::Clap, PluginFormat::Vst3],
    })
    .expect("public server generic event scan should succeed");

    let report = host.supervisor_report();
    let snapshot = &report.observation.plugin_event_snapshot;
    assert_eq!(snapshot.last_processing_epoch, Some(11));
    assert_eq!(snapshot.last_block_sequence, Some(18));
    assert_eq!(snapshot.last_generated_event_bytes, 212);
    assert_eq!(snapshot.total_events, 9);
    assert_eq!(snapshot.note_expression_events, 3);
    assert_eq!(snapshot.midi_events, 1);
    assert_eq!(snapshot.segment_epochs, vec![11]);
    assert!(
        report
            .observation
            .plugin_discovery_snapshot
            .capability_coverage
            .supports_note_expression_count
            >= 2
    );

    let rendered = report.render_json();
    assert!(rendered.contains("\"plugin_events\":{"));
    assert!(rendered.contains("\"note_expression_events\":3"));
    assert!(rendered.contains("\"supports_note_expression_count\":"));
}

#[test]
fn server_shared_host_edge_exports_runtime_controller_expression_truth() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::server(48_000, 512));
    runtime.record_plugin_event_summary(
        17,
        "lease:public-server-controller-expression",
        27,
        240,
        EventPacketSummary {
            total_events: 10,
            parameter_value_events: 1,
            parameter_modulation_events: 1,
            parameter_gesture_events: 1,
            note_events: 2,
            note_expression_events: 4,
            note_expression_pressure_events: 1,
            note_expression_timbre_events: 1,
            note_expression_tuning_events: 2,
            midi_events: 1,
        },
    );
    let mut host = ServerRuntimeHost::new(runtime);

    host.start_plugin_scan(PluginScanRequest {
        roots: vec!["~/.clap".into(), "/usr/lib/vst3".into()],
        formats: vec![PluginFormat::Clap, PluginFormat::Vst3],
    })
    .expect("public server controller-expression scan should succeed");

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
fn server_shared_host_edge_exports_runtime_control_surface_truth() {
    let runtime = SignalRuntime::new(RuntimeConfig::server(48_000, 512));
    let host = ServerRuntimeHost::new(runtime);
    let report = host.supervisor_report();

    assert_eq!(
        report.observation.control_surface_snapshot.discovery_state,
        signal_runtime::RuntimeExternalMidiDiscoveryState::Idle
    );
    assert_eq!(
        report.observation.control_surface_snapshot.graph_state,
        signal_runtime::RuntimeControlSurfaceGraphState::Empty
    );
    assert_eq!(
        report.observation.control_surface_snapshot.provider_name,
        "signal-host-server"
    );
    assert_eq!(report.observation.control_surface_snapshot.device_count, 0);
    assert!(report
        .observation
        .control_surface_snapshot
        .devices
        .is_empty());

    let rendered = report.render_json();
    assert!(rendered.contains("\"control_surface_snapshot\":{"));
    assert!(rendered.contains("\"graph_state\":\"Empty\""));
    assert!(rendered.contains("\"provider_name\":\"signal-host-server\""));
}

#[test]
fn server_shared_host_edge_exports_runtime_advanced_hardware_truth() {
    let runtime = SignalRuntime::new(RuntimeConfig::server(48_000, 512));
    let host = ServerRuntimeHost::new(runtime);
    let report = host.supervisor_report();

    assert_eq!(
        report
            .observation
            .advanced_hardware_snapshot
            .discovery_state,
        signal_runtime::RuntimeExternalMidiDiscoveryState::Idle
    );
    assert_eq!(
        report.observation.advanced_hardware_snapshot.graph_state,
        signal_runtime::RuntimeAdvancedHardwareGraphState::Empty
    );
    assert_eq!(
        report.observation.advanced_hardware_snapshot.provider_name,
        "signal-host-server"
    );
    assert_eq!(
        report.observation.advanced_hardware_snapshot.device_count,
        0
    );
    assert_eq!(
        report
            .observation
            .advanced_hardware_snapshot
            .display_transport_device_count,
        0
    );
    assert_eq!(
        report
            .observation
            .advanced_hardware_snapshot
            .motor_transport_device_count,
        0
    );
    assert_eq!(
        report
            .observation
            .advanced_hardware_snapshot
            .haptic_transport_device_count,
        0
    );
    assert_eq!(
        report
            .observation
            .advanced_hardware_snapshot
            .scene_mapping_device_count,
        0
    );
    assert_eq!(
        report
            .observation
            .advanced_hardware_snapshot
            .feedback_page_device_count,
        0
    );
    assert_eq!(
        report
            .observation
            .advanced_hardware_snapshot
            .safe_action_graph_device_count,
        0
    );
    assert!(report
        .observation
        .advanced_hardware_snapshot
        .devices
        .is_empty());

    let rendered = report.render_json();
    assert!(rendered.contains("\"advanced_hardware_snapshot\":{"));
    assert!(rendered.contains("\"graph_state\":\"Empty\""));
    assert!(rendered.contains("\"provider_name\":\"signal-host-server\""));
    assert!(rendered.contains("\"display_transport_device_count\":0"));
    assert!(rendered.contains("\"motor_transport_device_count\":0"));
    assert!(rendered.contains("\"haptic_transport_device_count\":0"));
    assert!(rendered.contains("\"scene_mapping_device_count\":0"));
    assert!(rendered.contains("\"feedback_page_device_count\":0"));
    assert!(rendered.contains("\"safe_action_graph_device_count\":0"));
}

#[test]
fn server_shared_host_edge_exports_runtime_recall_portability_truth() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::server(48_000, 512));
    runtime
        .handshake(signal_runtime::HandshakeRequest {
            client_version: "public-host-server-recall-portability".into(),
            anticipative_preferred: true,
            max_sample_rate_hint: Some(96_000),
        })
        .expect("server host-edge recall portability handshake should succeed");
    runtime
        .configure(RuntimeConfigRequest::new(48_000, 512))
        .expect("server host-edge recall portability configure should succeed");
    let scan_handle = runtime.record_plugin_scan_request(&PluginScanRequest {
        roots: vec!["/usr/lib/vst3".into()],
        formats: vec![PluginFormat::Vst3],
    });
    runtime.record_plugin_scan_results(
        scan_handle,
        vec![signal_runtime::RuntimePluginDiscoveredTypeRecord {
            plugin_type_id: "plugin:vst3:server-recall".into(),
            plugin_id: "com.signal.server-recall".into(),
            vendor: "Signal".into(),
            name: "Signal Server Recall".into(),
            format: PluginFormat::Vst3,
            version: Some("1.0.0".into()),
            features: vec![signal_plugin::PluginFeature::Instrument],
            default_io_layout: signal_plugin::PluginIoLayout {
                audio_inputs: 0,
                audio_outputs: 2,
                midi_inputs: 1,
                midi_outputs: 0,
            },
            default_multichannel_io: signal_runtime::RuntimeMultichannelIoSummary::for_plugin_io(
                signal_plugin::PluginIoLayout {
                    audio_inputs: 0,
                    audio_outputs: 2,
                    midi_inputs: 1,
                    midi_outputs: 0,
                },
            ),
            complex_io_summary:
                signal_runtime::RuntimePluginComplexIoSummary::from_plugin_features_and_layout(
                    &[signal_plugin::PluginFeature::Instrument],
                    signal_plugin::PluginIoLayout {
                        audio_inputs: 0,
                        audio_outputs: 2,
                        midi_inputs: 1,
                        midi_outputs: 0,
                    },
                ),
            audio_bus_count: 1,
            parameter_count: 6,
            state_contract: signal_plugin::PluginStateContract {
                supports_snapshot: false,
                supports_reset: true,
                supports_bypass: false,
                exposes_latency: false,
                exposes_tail: true,
            },
            processing_contract: signal_plugin::PluginProcessingContract {
                max_block_frames: 1024,
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
            summary: "server host recall portability type".into(),
        }],
    );
    apply_public_plugin_continuity_graph(
        &mut runtime,
        "graph:host-server:recall-portability",
        &[("node-server-vst3", "sandbox-server-vst3")],
    );
    record_public_plugin_sandbox_ready(
        &mut runtime,
        "sandbox-server-vst3",
        PluginFormat::Vst3,
        "plugin:vst3:server-recall",
        52,
    );
    runtime.record_plugin_ara_context("sandbox-server-vst3", sample_server_ara_context());

    let host = ServerRuntimeHost::new(runtime);
    let report = host.supervisor_report();
    let recall = report
        .observation
        .execution_topology_summary
        .nodes
        .iter()
        .find(|node| node.node_id == "node-server-vst3")
        .and_then(|node| node.plugin_recall.as_ref())
        .expect("server host-edge recall portability should be exported");
    assert_eq!(
        recall.payload.interchange.portability_class,
        RuntimePluginRecallPortabilityClass::ContextOnly
    );
    assert!(!recall.payload.interchange.shared_payload_available);
    assert_eq!(
        recall
            .payload
            .ara_context
            .as_ref()
            .and_then(|context| context.document_context.as_ref())
            .map(|document| document.document_id.as_str()),
        Some("doc:host-server")
    );

    let rendered = report.render_json();
    assert!(rendered.contains("\"interchange\":{"));
    assert!(rendered.contains("\"portability_class\":\"ContextOnly\""));
    assert!(rendered.contains("\"source_id\":\"source:stem-bus\""));
    assert!(rendered.contains("\"region_id\":\"region:bridge\""));
}

#[test]
fn server_shared_host_edge_exports_runtime_media_service_truth() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::server(48_000, 512));
    runtime
        .handshake(signal_runtime::HandshakeRequest {
            client_version: "public-host-server-media-service".into(),
            anticipative_preferred: true,
            max_sample_rate_hint: Some(96_000),
        })
        .expect("server host-edge media-service handshake should succeed");
    runtime
        .configure(RuntimeConfigRequest::new(48_000, 256))
        .expect("server host-edge media-service configure should succeed");

    let ready_path = public_server_media_fixture_path("ready");
    let missing_path = public_server_media_fixture_path("missing");
    write_public_test_wav(&ready_path);

    runtime
        .reconcile_media_assets(vec![
            signal_runtime::RuntimeMediaAssetRegistration {
                asset_id: "asset:sha256:host-server-media-ready".into(),
                content_hash: "host-server-media-ready".into(),
                source_path: ready_path.display().to_string(),
                file_name: "host-server-media-ready.wav".into(),
                byte_size: fs::metadata(&ready_path)
                    .expect("public server media fixture should exist")
                    .len(),
                sample_rate_hz: 48_000,
                channel_count: 1,
                duration_samples: 128,
                waveform_bin_count: 16,
            },
            signal_runtime::RuntimeMediaAssetRegistration {
                asset_id: "asset:sha256:host-server-media-missing".into(),
                content_hash: "host-server-media-missing".into(),
                source_path: missing_path.display().to_string(),
                file_name: "host-server-media-missing.wav".into(),
                byte_size: 0,
                sample_rate_hz: 48_000,
                channel_count: 1,
                duration_samples: 128,
                waveform_bin_count: 16,
            },
        ])
        .expect("server host-edge media assets should reconcile");
    runtime
        .start_media_preview("asset:sha256:host-server-media-ready")
        .expect("server host-edge media preview should start");

    let host = ServerRuntimeHost::new(runtime);
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
        Some("asset:sha256:host-server-media-ready")
    );
    assert_eq!(
        report
            .observation
            .media_service_snapshot
            .last_invalidated_asset_id
            .as_deref(),
        Some("asset:sha256:host-server-media-missing")
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
        .find(|asset| asset.asset_id == "asset:sha256:host-server-media-ready")
        .and_then(|asset| asset.cache_path.as_deref())
    {
        let _ = fs::remove_file(path);
    }
}

#[test]
fn server_shared_host_edge_exports_runtime_analysis_metadata_truth() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::server(48_000, 512));
    runtime
        .handshake(signal_runtime::HandshakeRequest {
            client_version: "public-host-server-analysis-metadata".into(),
            anticipative_preferred: true,
            max_sample_rate_hint: Some(96_000),
        })
        .expect("server host-edge analysis-metadata handshake should succeed");
    runtime
        .configure(RuntimeConfigRequest::new(48_000, 256))
        .expect("server host-edge analysis-metadata configure should succeed");

    let ready_path = public_server_media_fixture_path("analysis-ready");
    let missing_path = public_server_media_fixture_path("analysis-missing");
    write_public_test_wav(&ready_path);

    runtime
        .reconcile_media_assets(vec![
            signal_runtime::RuntimeMediaAssetRegistration {
                asset_id: "asset:sha256:host-server-analysis-ready".into(),
                content_hash: "host-server-analysis-ready".into(),
                source_path: ready_path.display().to_string(),
                file_name: "host-server-analysis-ready.wav".into(),
                byte_size: fs::metadata(&ready_path)
                    .expect("public server analysis fixture should exist")
                    .len(),
                sample_rate_hz: 48_000,
                channel_count: 1,
                duration_samples: 128,
                waveform_bin_count: 16,
            },
            signal_runtime::RuntimeMediaAssetRegistration {
                asset_id: "asset:sha256:host-server-analysis-missing".into(),
                content_hash: "host-server-analysis-missing".into(),
                source_path: missing_path.display().to_string(),
                file_name: "host-server-analysis-missing.wav".into(),
                byte_size: 0,
                sample_rate_hz: 48_000,
                channel_count: 1,
                duration_samples: 128,
                waveform_bin_count: 16,
            },
        ])
        .expect("server host-edge analysis metadata assets should reconcile");

    let host = ServerRuntimeHost::new(runtime);
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
        .find(|descriptor| descriptor.asset_id == "asset:sha256:host-server-analysis-ready")
        .expect("server host-edge ready analysis descriptor");
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
        .find(|descriptor| descriptor.asset_id == "asset:sha256:host-server-analysis-missing")
        .expect("server host-edge invalidated analysis descriptor");
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
        .find(|asset| asset.asset_id == "asset:sha256:host-server-analysis-ready")
        .and_then(|asset| asset.cache_path.as_deref())
    {
        let _ = fs::remove_file(path);
    }
}

#[test]
fn server_shared_host_edge_exports_runtime_fault_diagnostic_truth() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::server(48_000, 512));
    runtime
        .set_safe_mode(signal_runtime::SafeModeRequest { enabled: true })
        .expect("server host-edge fault diagnostic safe mode should enable");
    runtime
        .render_offline_queue(vec![RuntimeOfflineRenderRequest {
            request_id: "render:host-server:fault-diagnostic".into(),
            timeline_start_samples: 0,
            duration_samples: 64,
            export_sample_rate_hz: 48_000,
            include_main_mix: true,
            artifact_root_path: None,
            stem_targets: Vec::new(),
            freeze_artifacts: Vec::new(),
        }])
        .expect("server host-edge fault diagnostic queue should defer");

    let host = ServerRuntimeHost::new(runtime);
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
fn server_shared_host_edge_exports_runtime_device_supervision_truth() {
    let mut recovering_runtime = SignalRuntime::new(RuntimeConfig::server(48_000, 512));
    recovering_runtime
        .handshake(signal_runtime::HandshakeRequest {
            client_version: "public-host-server-device-supervision-recovering".into(),
            anticipative_preferred: true,
            max_sample_rate_hint: Some(96_000),
        })
        .expect("public server device supervision recovering handshake should succeed");
    recovering_runtime
        .configure(RuntimeConfigRequest::new(48_000, 512))
        .expect("public server device supervision recovering configure should succeed");
    recovering_runtime
        .start()
        .expect("public server device supervision recovering start should succeed");
    recovering_runtime.record_watchdog_restart(WatchdogRestartRecord {
        sandbox_id: "public-host-server-device-supervision-watchdog".into(),
        trigger: RuntimeWatchdogTrigger::HeartbeatMisses,
        processing_epoch: 3,
    });
    let recovering_host = ServerRuntimeHost::new(recovering_runtime);
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
            .interruption_class,
        RuntimeInterruptionClass::Steady
    );

    let mut faulted_runtime = SignalRuntime::new(RuntimeConfig::server(48_000, 512));
    faulted_runtime
        .handshake(signal_runtime::HandshakeRequest {
            client_version: "public-host-server-device-supervision-faulted".into(),
            anticipative_preferred: true,
            max_sample_rate_hint: Some(96_000),
        })
        .expect("public server device supervision faulted handshake should succeed");
    faulted_runtime
        .configure(RuntimeConfigRequest::new(48_000, 512))
        .expect("public server device supervision faulted configure should succeed");
    faulted_runtime
        .start()
        .expect("public server device supervision faulted start should succeed");
    faulted_runtime.fail_runtime(RuntimeError::new(
        RuntimeErrorKind::HardwareFailure,
        "public server host device supervision fault",
    ));
    let faulted_host = ServerRuntimeHost::new(faulted_runtime);
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
fn server_shared_host_edge_exports_runtime_block_timing_truth() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::server(48_000, 48));
    runtime
        .handshake(signal_runtime::HandshakeRequest {
            client_version: "public-host-edge-block-timing".into(),
            anticipative_preferred: true,
            max_sample_rate_hint: Some(96_000),
        })
        .expect("server host-edge block timing handshake should succeed");
    runtime
        .configure(RuntimeConfigRequest::new(48_000, 48))
        .expect("server host-edge block timing configure should succeed");
    apply_public_capture_graph(&mut runtime, "graph:host-server:block-timing");
    runtime
        .process_engine_block(
            1,
            1,
            synthetic_stereo_block(SampleRate(48_000), FrameCount(48), 49),
        )
        .expect("server host-edge block timing block should process");

    let host = ServerRuntimeHost::new(runtime);
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
            .expect("server host-edge block timing should expose latest execution time")
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
fn server_shared_host_edge_exports_runtime_external_io_truth() {
    let runtime = SignalRuntime::new(RuntimeConfig::server(48_000, 512));
    let host = ServerRuntimeHost::new(runtime);
    let report = host.supervisor_report();

    assert_eq!(
        report.observation.external_io_snapshot.health_state,
        RuntimeExternalIoHealthState::Unavailable
    );
    assert_eq!(
        report.observation.external_io_snapshot.primary_role,
        RuntimeExternalIoPrimaryRole::Unavailable
    );
    assert_eq!(
        report.observation.external_io_snapshot.monitoring_state,
        RuntimeExternalIoMonitoringState::Unavailable
    );
    assert_eq!(
        report.observation.external_io_snapshot.monitoring_tap_point,
        RuntimeExternalIoMonitoringTapPoint::Unavailable
    );
    assert_eq!(
        report.observation.external_io_snapshot.loopback_state,
        RuntimeExternalIoLoopbackState::Unavailable
    );

    let rendered = report.render_json();
    assert!(rendered.contains("\"external_io_snapshot\":{"));
    assert!(rendered.contains("\"health_state\":\"Unavailable\""));
    assert!(rendered.contains("\"monitoring_state\":\"Unavailable\""));
    assert!(rendered.contains("\"loopback_state\":\"Unavailable\""));
}

#[test]
fn server_shared_host_edge_exports_runtime_linux_audio_backend_truth() {
    let runtime = SignalRuntime::new(RuntimeConfig::server(48_000, 512));
    let host = ServerRuntimeHost::new(runtime);
    let report = host.supervisor_report();

    assert_eq!(
        report
            .observation
            .external_io_snapshot
            .linux_backend_identity,
        signal_runtime::RuntimeLinuxAudioBackendIdentity::Unavailable
    );
    assert_eq!(
        report
            .observation
            .external_io_snapshot
            .linux_backend_portability,
        signal_runtime::RuntimeLinuxAudioBackendPortabilityBand::Unsupported
    );
    assert_eq!(
        report.observation.external_io_snapshot.fallback_state,
        signal_runtime::RuntimeHostClockFallbackState::Unconfigured
    );

    let rendered = report.render_json();
    assert!(rendered.contains("\"external_io_snapshot\":{"));
    assert!(rendered.contains("\"linux_backend_identity\":\"Unavailable\""));
    assert!(rendered.contains("\"linux_backend_portability\":\"Unsupported\""));
    assert!(rendered.contains("\"fallback_state\":\"Unconfigured\""));
}

#[test]
fn server_shared_host_edge_exports_runtime_linux_backend_clock_topology_truth() {
    let runtime = SignalRuntime::new(RuntimeConfig::server(48_000, 512));
    let host = ServerRuntimeHost::new(runtime);
    let report = host.supervisor_report();

    assert_eq!(
        report
            .observation
            .external_io_snapshot
            .linux_clocking_parity,
        signal_runtime::RuntimeLinuxAudioBackendClockingParityBand::Unsupported
    );
    assert_eq!(
        report.observation.external_io_snapshot.linux_duplex_parity,
        signal_runtime::RuntimeLinuxAudioBackendDuplexParityState::Unsupported
    );
    assert_eq!(
        report
            .observation
            .external_io_snapshot
            .linux_endpoint_topology_parity,
        signal_runtime::RuntimeLinuxAudioBackendEndpointTopologyParityState::Unsupported
    );
    assert_eq!(
        report.observation.external_io_snapshot.endpoint_topology,
        signal_runtime::RuntimeHostEndpointTopology::Unconfigured
    );

    let rendered = report.render_json();
    assert!(rendered.contains("\"linux_clocking_parity\":\"Unsupported\""));
    assert!(rendered.contains("\"linux_duplex_parity\":\"Unsupported\""));
    assert!(rendered.contains("\"linux_endpoint_topology_parity\":\"Unsupported\""));
    assert!(rendered.contains("\"endpoint_topology\":\"Unconfigured\""));
}

#[test]
fn server_shared_host_edge_exports_runtime_linux_live_ownership_truth() {
    let runtime = SignalRuntime::new(RuntimeConfig::server(48_000, 512));
    let host = ServerRuntimeHost::new(runtime);
    let report = host.supervisor_report();

    assert_eq!(
        report
            .observation
            .linux_backend_session_snapshot
            .backend_identity,
        signal_runtime::RuntimeLinuxAudioBackendIdentity::PipeWire
    );
    assert_eq!(
        report.observation.linux_backend_session_snapshot.ownership,
        signal_runtime::RuntimeLinuxBackendSessionOwnership::BackendManagedGraph
    );
    assert_eq!(
        report
            .observation
            .linux_backend_session_snapshot
            .lifecycle_state,
        signal_runtime::RuntimeLinuxBackendSessionLifecycleState::Running
    );
    assert_eq!(
        report
            .observation
            .linux_backend_session_snapshot
            .device_claim_posture,
        signal_runtime::RuntimeLinuxBackendDeviceClaimPosture::SharedGraph
    );
    assert_eq!(
        report
            .observation
            .linux_backend_session_snapshot
            .session_role,
        signal_runtime::RuntimeLinuxBackendSessionRole::PrimaryAudioIo
    );
    assert_eq!(
        report
            .observation
            .linux_backend_session_snapshot
            .ownership_fallback,
        signal_runtime::RuntimeLinuxBackendOwnershipFallbackState::BackendManagedGuarded
    );

    let rendered = report.render_json();
    assert!(rendered.contains("\"linux_backend_session_snapshot\":{"));
    assert!(rendered.contains("\"backend_identity\":\"PipeWire\""));
    assert!(rendered.contains("\"ownership\":\"BackendManagedGraph\""));
    assert!(rendered.contains("\"session_role\":\"PrimaryAudioIo\""));
}

#[test]
fn server_shared_host_edge_exports_runtime_jack_coordination_truth() {
    let runtime = SignalRuntime::new(RuntimeConfig::server(48_000, 512));
    let host = ServerRuntimeHost::new(runtime);
    let report = host.supervisor_report();

    assert_eq!(
        report
            .observation
            .jack_coordination_snapshot
            .transport_posture,
        RuntimeJackTransportPosture::Detached
    );
    assert_eq!(
        report.observation.jack_coordination_snapshot.graph_state,
        RuntimeJackGraphCoordinationState::AttachedGuarded
    );
    assert_eq!(
        report.observation.jack_coordination_snapshot.client_role,
        RuntimeJackClientRole::PrimaryAudioIo
    );
    assert_eq!(
        report.observation.jack_coordination_snapshot.guarded_state,
        RuntimeJackGuardedCoordinationState::GraphGuarded
    );

    let rendered = report.render_json();
    assert!(rendered.contains("\"jack_coordination_snapshot\":{"));
    assert!(rendered.contains("\"transport_posture\":\"Detached\""));
    assert!(rendered.contains("\"graph_state\":\"AttachedGuarded\""));
    assert!(rendered.contains("\"client_role\":\"PrimaryAudioIo\""));
    assert!(rendered.contains("\"guarded_state\":\"GraphGuarded\""));
}

#[test]
fn server_shared_host_edge_exports_runtime_pipewire_alsa_parity_truth() {
    let runtime = SignalRuntime::new(RuntimeConfig::server(48_000, 512));
    let host = ServerRuntimeHost::new(runtime);
    let report = host.supervisor_report();

    assert_eq!(
        report
            .observation
            .pipewire_alsa_parity_snapshot
            .session_role_parity,
        signal_runtime::RuntimePipeWireAlsaSessionRoleParity::PrimaryAudioIo
    );
    assert_eq!(
        report
            .observation
            .pipewire_alsa_parity_snapshot
            .device_claim_parity,
        signal_runtime::RuntimePipeWireAlsaDeviceClaimParity::SharedGraph
    );
    assert_eq!(
        report
            .observation
            .pipewire_alsa_parity_snapshot
            .stream_policy_parity,
        signal_runtime::RuntimePipeWireAlsaStreamPolicyParity::BackendManagedGraph
    );
    assert_eq!(
        report
            .observation
            .pipewire_alsa_parity_snapshot
            .guarded_state,
        signal_runtime::RuntimePipeWireAlsaGuardedParityState::ClockGuarded
    );

    let rendered = report.render_json();
    assert!(rendered.contains("\"pipewire_alsa_parity_snapshot\":{"));
    assert!(rendered.contains("\"session_role_parity\":\"PrimaryAudioIo\""));
    assert!(rendered.contains("\"device_claim_parity\":\"SharedGraph\""));
    assert!(rendered.contains("\"stream_policy_parity\":\"BackendManagedGraph\""));
}

#[test]
fn server_shared_host_edge_exports_runtime_lv2_extension_truth() {
    let runtime = SignalRuntime::new(RuntimeConfig::server(48_000, 512));
    let mut host = ServerRuntimeHost::new(runtime);
    host.start_plugin_scan(PluginScanRequest {
        roots: vec!["~/.lv2".into(), "/usr/lib/lv2".into()],
        formats: vec![PluginFormat::Lv2],
    })
    .expect("public server lv2 extension scan should succeed");
    host.ensure_plugin_sandbox(PluginSandboxSpec {
        sandbox_id: "public-server-lv2-extension".into(),
        plugin_format: PluginFormat::Lv2,
        plugin_type_id: Some("plugin:lv2:linux-synth".into()),
    })
    .expect("public server lv2 extension sandbox should succeed");

    let report = host.supervisor_report();
    assert_eq!(
        report.observation.lv2_extension_snapshot.plugin_type_count,
        4
    );
    assert_eq!(
        report
            .observation
            .lv2_extension_snapshot
            .worker_required_type_count,
        2
    );
    assert_eq!(
        report
            .observation
            .lv2_extension_snapshot
            .patch_supported_type_count,
        3
    );
    let record = report
        .observation
        .lv2_extension_snapshot
        .records
        .iter()
        .find(|record| record.plugin_type_id == "plugin:lv2:linux-synth")
        .expect("server lv2 extension record should be visible");
    assert_eq!(
        record.worker_posture,
        signal_runtime::RuntimeLv2WorkerPosture::WorkerRequiredAvailable
    );
    assert_eq!(
        record.urid_negotiation_posture,
        signal_runtime::RuntimeLv2UridNegotiationPosture::Negotiated
    );
    assert_eq!(
        record.patch_exchange_posture,
        signal_runtime::RuntimeLv2PatchExchangePosture::Supported
    );
    assert_eq!(
        record.extension_negotiation_state,
        signal_runtime::RuntimeLv2ExtensionNegotiationState::Negotiated
    );

    let rendered = report.render_json();
    assert!(rendered.contains("\"lv2_extension_snapshot\":{"));
    assert!(rendered.contains("\"worker_posture\":\"WorkerRequiredAvailable\""));
    assert!(rendered.contains("\"patch_exchange_posture\":\"Supported\""));
}

#[test]
fn server_shared_host_edge_exports_runtime_external_midi_truth() {
    let runtime = SignalRuntime::new(RuntimeConfig::server(48_000, 512));
    let host = ServerRuntimeHost::new(runtime);
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
        "signal-host-server"
    );
    assert_eq!(report.observation.external_midi_snapshot.device_count, 0);
    assert_eq!(report.observation.external_midi_snapshot.endpoint_count, 0);
    assert_eq!(
        report
            .observation
            .external_midi_snapshot
            .guarded_route_count,
        0
    );
    assert_eq!(
        report
            .observation
            .external_midi_snapshot
            .live_ownership
            .ownership_posture,
        signal_runtime::RuntimeExternalMidiLiveOwnershipPosture::NoLiveOwnership
    );
    assert_eq!(
        report
            .observation
            .external_midi_snapshot
            .live_ownership
            .attach_continuity,
        signal_runtime::RuntimeExternalMidiAttachContinuity::Detached
    );
    assert_eq!(
        report
            .observation
            .external_midi_snapshot
            .live_ownership
            .backend_parity,
        signal_runtime::RuntimeExternalMidiBackendParity::Guarded
    );
    assert_eq!(
        report
            .observation
            .external_midi_snapshot
            .live_ownership
            .guarded_parity_outcome,
        signal_runtime::RuntimeExternalMidiGuardedParityOutcome::BackendManaged
    );

    let rendered = report.render_json();
    assert!(rendered.contains("\"external_midi_snapshot\":{"));
    assert!(rendered.contains("\"live_ownership\":{"));
    assert!(rendered.contains("\"discovery_state\":\"Idle\""));
    assert!(rendered.contains("\"graph_state\":\"Empty\""));
    assert!(rendered.contains("\"ownership_posture\":\"NoLiveOwnership\""));
    assert!(rendered.contains("\"backend_parity\":\"Guarded\""));
    assert!(rendered.contains("\"guarded_parity_outcome\":\"BackendManaged\""));
    assert!(rendered.contains("\"provider_name\":\"signal-host-server\""));
}

#[test]
fn server_shared_host_edge_exports_runtime_multichannel_truth() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::server(48_000, 512));
    runtime
        .handshake(signal_runtime::HandshakeRequest {
            client_version: "public-host-server-multichannel".into(),
            anticipative_preferred: true,
            max_sample_rate_hint: Some(96_000),
        })
        .expect("server host-edge multichannel handshake should succeed");
    runtime
        .configure(RuntimeConfigRequest::new(48_000, 256))
        .expect("server host-edge multichannel configure should succeed");
    apply_public_multichannel_graph(&mut runtime, "graph:host-server:multichannel");
    let scan_handle = runtime.record_plugin_scan_request(&PluginScanRequest {
        roots: vec!["/usr/lib/vst3".into()],
        formats: vec![PluginFormat::Vst3],
    });
    runtime.record_plugin_scan_results(
        scan_handle,
        vec![signal_runtime::RuntimePluginDiscoveredTypeRecord {
            plugin_type_id: "plugin:vst3:host-server-multichannel".into(),
            plugin_id: "com.signal.host-server-multichannel".into(),
            vendor: "Signal".into(),
            name: "Signal Host Server Multichannel".into(),
            format: PluginFormat::Vst3,
            version: Some("1.0.0".into()),
            features: vec![signal_plugin::PluginFeature::Instrument],
            default_io_layout: signal_plugin::PluginIoLayout {
                audio_inputs: 0,
                audio_outputs: 6,
                midi_inputs: 1,
                midi_outputs: 0,
            },
            default_multichannel_io: signal_runtime::RuntimeMultichannelIoSummary::for_plugin_io(
                signal_plugin::PluginIoLayout {
                    audio_inputs: 0,
                    audio_outputs: 6,
                    midi_inputs: 1,
                    midi_outputs: 0,
                },
            ),
            complex_io_summary:
                signal_runtime::RuntimePluginComplexIoSummary::from_plugin_features_and_layout(
                    &[signal_plugin::PluginFeature::Instrument],
                    signal_plugin::PluginIoLayout {
                        audio_inputs: 0,
                        audio_outputs: 6,
                        midi_inputs: 1,
                        midi_outputs: 0,
                    },
                ),
            audio_bus_count: 1,
            parameter_count: 6,
            state_contract: signal_plugin::PluginStateContract {
                supports_snapshot: false,
                supports_reset: true,
                supports_bypass: false,
                exposes_latency: false,
                exposes_tail: true,
            },
            processing_contract: signal_plugin::PluginProcessingContract {
                max_block_frames: 1024,
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
            summary: "server multichannel boundary plugin".into(),
        }],
    );

    let host = ServerRuntimeHost::new(runtime);
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
    assert_eq!(
        report
            .observation
            .external_io_snapshot
            .io_layout
            .output_layout
            .channel_count,
        0
    );
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
fn server_shared_host_edge_exports_runtime_sidechain_truth() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::server(48_000, 512));
    runtime
        .handshake(signal_runtime::HandshakeRequest {
            client_version: "public-host-server-sidechain".into(),
            anticipative_preferred: true,
            max_sample_rate_hint: Some(96_000),
        })
        .expect("server host-edge sidechain handshake should succeed");
    runtime
        .configure(RuntimeConfigRequest::new(48_000, 256))
        .expect("server host-edge sidechain configure should succeed");
    apply_public_sidechain_graph(&mut runtime, "graph:host-server:sidechain");
    record_public_plugin_sandbox_ready(
        &mut runtime,
        "sandbox:host-server:sidechain",
        PluginFormat::Clap,
        "plugin:clap:host-server-sidechain-compressor",
        1,
    );

    let host = ServerRuntimeHost::new(runtime);
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
        .expect("server host-edge sidechain plugin stage should be exported");
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
fn server_shared_host_edge_exports_runtime_multi_bus_truth() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::server(48_000, 512));
    runtime
        .handshake(signal_runtime::HandshakeRequest {
            client_version: "public-host-server-multi-bus".into(),
            anticipative_preferred: true,
            max_sample_rate_hint: Some(96_000),
        })
        .expect("server host-edge multi-bus handshake should succeed");
    runtime
        .configure(RuntimeConfigRequest::new(48_000, 256))
        .expect("server host-edge multi-bus configure should succeed");
    apply_public_multi_bus_graph(&mut runtime, "graph:host-server:multi-bus");
    runtime
        .process_engine_block(
            3,
            5,
            synthetic_stereo_block(SampleRate(48_000), FrameCount(256), 2),
        )
        .expect("server host-edge multi-bus block should succeed");

    let host = ServerRuntimeHost::new(runtime);
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
fn server_shared_host_edge_exports_runtime_complex_io_truth() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::server(48_000, 512));
    runtime
        .handshake(signal_runtime::HandshakeRequest {
            client_version: "public-host-server-complex-io".into(),
            anticipative_preferred: true,
            max_sample_rate_hint: Some(96_000),
        })
        .expect("public server complex io handshake should succeed");
    runtime
        .configure(RuntimeConfigRequest::new(48_000, 256))
        .expect("public server complex io configure should succeed");
    apply_public_complex_io_graph(&mut runtime, "graph:host-server:complex-io");
    let scan_handle = runtime.record_plugin_scan_request(&PluginScanRequest {
        roots: vec!["/usr/lib/vst3".into()],
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
            graph_id: "graph:host-server:complex-io".into(),
            processing_epoch: 1,
            block_sequence: 1,
            renders: vec![
                signal_runtime::PluginNodeRender {
                    node_id: "plugin-multiout".into(),
                    sandbox_id: "sandbox:host-server:multiout".into(),
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
                    sandbox_id: "sandbox:host-server:bus-fx".into(),
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
        .expect("public server complex io render batch should apply");
    runtime
        .process_engine_block(
            5,
            7,
            synthetic_stereo_block(SampleRate(48_000), FrameCount(256), 5),
        )
        .expect("public server complex io block should process");

    let host = ServerRuntimeHost::new(runtime);
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
        record.plugin_type_id == "plugin:vst3:host-server-multiout"
            && record.complex_io_summary.multi_output_instrument
    }));
    assert!(discovery.discovered_types.iter().any(|record| {
        record.plugin_type_id == "plugin:vst3:host-server-bus-fx"
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
    let pin_matrix = &report.observation.plugin_pin_matrix_snapshot;
    assert_eq!(pin_matrix.plugin_type_count, 2);
    assert_eq!(pin_matrix.negotiated_type_count, 2);
    assert_eq!(pin_matrix.dynamic_negotiated_type_count, 2);
    assert!(pin_matrix.records.iter().any(|record| {
        record.plugin_type_id == "plugin:vst3:host-server-multiout"
            && record.pin_matrix_posture
                == signal_runtime::RuntimePluginPinMatrixPosture::Negotiated
            && record.dynamic_bus_negotiation_posture
                == signal_runtime::RuntimeDynamicBusNegotiationPosture::Negotiated
            && record
                .pin_group_identities
                .contains(&signal_runtime::RuntimePluginPinGroupIdentity::SecondaryProgramPath)
    }));
    assert!(pin_matrix.records.iter().any(|record| {
        record.plugin_type_id == "plugin:vst3:host-server-bus-fx"
            && record.fallback_outcome
                == signal_runtime::RuntimePluginNegotiationFallbackOutcome::GuardedDegradation
            && record
                .pin_group_identities
                .contains(&signal_runtime::RuntimePluginPinGroupIdentity::SidechainPath)
    }));

    let rendered = report.render_json();
    assert!(rendered.contains("\"plugin_discovery_snapshot\":{"));
    assert!(rendered.contains("\"plugin_pin_matrix_snapshot\":{"));
    assert!(rendered.contains("\"complex_io_summary\":{"));
    assert!(rendered.contains("\"pin_matrix_posture\":\"Negotiated\""));
    assert!(rendered.contains("\"multi_output_instrument\":true"));
    assert!(rendered.contains("\"bus_capable_fx_class\":\"SendReturnCapableFx\""));
}

#[test]
fn server_shared_host_edge_exports_runtime_spatial_truth() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::server(48_000, 512));
    runtime
        .handshake(signal_runtime::HandshakeRequest {
            client_version: "public-host-server-spatial".into(),
            anticipative_preferred: true,
            max_sample_rate_hint: Some(96_000),
        })
        .expect("public server spatial handshake should succeed");
    runtime
        .configure(RuntimeConfigRequest::new(48_000, 256))
        .expect("public server spatial configure should succeed");
    apply_public_spatial_graph(&mut runtime, "graph:host-server:spatial");

    let host = ServerRuntimeHost::new(runtime);
    let report = host.supervisor_report();
    let topology = &report.observation.execution_topology_summary;
    assert_eq!(topology.spatial_node_count, 2);
    assert_eq!(topology.active_spatial_node_count, 1);
    assert_eq!(topology.bypassed_spatial_node_count, 1);
    assert_eq!(topology.fallback_spatial_node_count, 1);
    assert_eq!(topology.surround_bed_spatial_node_count, 1);
    assert_eq!(topology.object_aware_spatial_node_count, 0);
    assert_eq!(topology.expanded_fallback_spatial_node_count, 1);
    assert_eq!(topology.immersive_spatial_node_count, 1);
    assert_eq!(topology.room_policy_aware_spatial_node_count, 0);
    assert_eq!(topology.fallback_room_policy_spatial_node_count, 1);
    assert_eq!(topology.deployment_spatial_node_count, 1);
    assert_eq!(topology.folded_down_spatial_node_count, 1);
    assert_eq!(topology.fallback_monitoring_scene_spatial_node_count, 1);
    assert_eq!(topology.renderer_capability_spatial_node_count, 1);
    assert_eq!(topology.negotiated_renderer_spatial_node_count, 0);
    assert_eq!(topology.immersive_export_spatial_node_count, 1);
    assert_eq!(topology.fallback_immersive_export_spatial_node_count, 1);
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
                        && spatial
                            .immersive_room_policy
                            .as_ref()
                            .is_some_and(|immersive| {
                                immersive.object_rendering_posture
                                    == RuntimeImmersiveObjectRenderingPosture::NotRequested
                                    && immersive.room_policy_class
                                        == RuntimeRoomPolicyClass::FallbackRoom
                                    && immersive.room_policy_authority
                                        == RuntimeRoomPolicyAuthority::RuntimeDefault
                                    && immersive.room_outcome
                                        == RuntimeImmersiveRoomOutcome::BypassRoomPolicy
                            })
                        && spatial
                            .deployment_monitoring
                            .as_ref()
                            .is_some_and(|monitoring| {
                                monitoring.deployment_class
                                    == RuntimeDeploymentClass::FallbackDeployment
                                    && monitoring.fold_down_policy
                                        == RuntimeFoldDownPolicy::FoldDownToReferenceBed
                                    && monitoring.monitoring_scene_class
                                        == RuntimeMonitoringSceneClass::FallbackScene
                                    && monitoring.monitoring_scene_authority
                                        == RuntimeMonitoringSceneAuthority::RuntimeDefault
                                    && monitoring.monitoring_outcome
                                        == RuntimeMonitoringOutcome::BypassMonitoringScene
                            })
                        && spatial.renderer_export.as_ref().is_some_and(|renderer| {
                            renderer.renderer_capability_posture
                                == RuntimeRendererCapabilityNegotiationPosture::FallbackNegotiation
                                && renderer.capability_authority
                                    == RuntimeRendererCapabilityAuthority::RuntimeDefault
                                && renderer.immersive_export_class
                                    == RuntimeImmersiveExportClass::FallbackExport
                                && renderer.export_authority
                                    == RuntimeImmersiveExportAuthority::RuntimeDefault
                                && renderer.export_outcome
                                    == RuntimeImmersiveExportOutcome::BypassImmersiveExport
                        })
                })
        }));

    let rendered = report.render_json();
    assert!(rendered.contains("\"spatial_node_count\":2"));
    assert!(rendered.contains("\"active_spatial_node_count\":1"));
    assert!(rendered.contains("\"surround_bed_spatial_node_count\":1"));
    assert!(rendered.contains("\"expanded_fallback_spatial_node_count\":1"));
    assert!(rendered.contains("\"immersive_spatial_node_count\":1"));
    assert!(rendered.contains("\"fallback_room_policy_spatial_node_count\":1"));
    assert!(rendered.contains("\"deployment_spatial_node_count\":1"));
    assert!(rendered.contains("\"folded_down_spatial_node_count\":1"));
    assert!(rendered.contains("\"fallback_monitoring_scene_spatial_node_count\":1"));
    assert!(rendered.contains("\"renderer_capability_spatial_node_count\":1"));
    assert!(rendered.contains("\"negotiated_renderer_spatial_node_count\":0"));
    assert!(rendered.contains("\"immersive_export_spatial_node_count\":1"));
    assert!(rendered.contains("\"fallback_immersive_export_spatial_node_count\":1"));
    assert!(rendered.contains("\"bed_class\":\"CanonicalSurroundBed\""));
    assert!(rendered.contains("\"mix_policy\":\"CollapseToBaselineSpatial\""));
    assert!(rendered.contains("\"execution_mode\":\"BalanceGroups\""));
    assert!(rendered.contains("\"fallback_outcome\":\"BypassSpatialProcessing\""));
    assert!(rendered.contains("\"expanded_fallback_outcome\":\"CollapseToBaselineSpatial\""));
    assert!(rendered.contains("\"immersive_room_policy\":{"));
    assert!(rendered.contains("\"room_policy_class\":\"FallbackRoom\""));
    assert!(rendered.contains("\"room_outcome\":\"BypassRoomPolicy\""));
    assert!(rendered.contains("\"deployment_monitoring\":{"));
    assert!(rendered.contains("\"deployment_class\":\"FallbackDeployment\""));
    assert!(rendered.contains("\"fold_down_policy\":\"FoldDownToReferenceBed\""));
    assert!(rendered.contains("\"monitoring_scene_class\":\"FallbackScene\""));
    assert!(rendered.contains("\"monitoring_outcome\":\"BypassMonitoringScene\""));
    assert!(rendered.contains("\"renderer_export\":{"));
    assert!(rendered.contains("\"renderer_capability_posture\":\"FallbackNegotiation\""));
    assert!(rendered.contains("\"immersive_export_class\":\"FallbackExport\""));
    assert!(rendered.contains("\"export_outcome\":\"BypassImmersiveExport\""));
}

#[test]
fn server_shared_host_edge_exports_runtime_stretch_truth() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::server(48_000, 512));
    runtime
        .handshake(signal_runtime::HandshakeRequest {
            client_version: "public-host-server-stretch".into(),
            anticipative_preferred: true,
            max_sample_rate_hint: Some(96_000),
        })
        .expect("public server stretch handshake should succeed");
    runtime
        .configure(RuntimeConfigRequest::new(48_000, 256))
        .expect("public server stretch configure should succeed");

    let ready_path = public_server_media_fixture_path("stretch-ready");
    write_public_test_wav(&ready_path);
    runtime
        .reconcile_media_assets(vec![signal_runtime::RuntimeMediaAssetRegistration {
            asset_id: "asset:sha256:host-server-stretch-ready".into(),
            content_hash: "host-server-stretch-ready".into(),
            source_path: ready_path.display().to_string(),
            file_name: "host-server-stretch-ready.wav".into(),
            byte_size: fs::metadata(&ready_path).unwrap().len(),
            sample_rate_hz: 48_000,
            channel_count: 1,
            duration_samples: 128,
            waveform_bin_count: 8,
        }])
        .expect("public server stretch media asset should reconcile");
    runtime
        .reconcile_warp_clips(vec![signal_runtime::RuntimeWarpClipRegistration {
            clip_id: "clip:host-server-stretch".into(),
            media_asset_id: Some("asset:sha256:host-server-stretch-ready".into()),
            mode: signal_runtime::RuntimeWarpMode::ElastiqueDraft,
            source_tempo_bpm: Some(120.0),
            anchor_timeline_samples: 0,
            start_samples: 0,
            duration_samples: 48_000,
        }])
        .expect("public server stretch warp clip should reconcile");
    runtime
        .reconcile_clip_processing_clips(vec![signal_runtime::RuntimeClipProcessingRegistration {
            clip_id: "clip:host-server-stretch".into(),
            media_asset_id: Some("asset:sha256:host-server-stretch-ready".into()),
            warp_mode: signal_runtime::RuntimeWarpMode::ElastiqueDraft,
            start_samples: 0,
            duration_samples: 48_000,
            fade_in: signal_runtime::RuntimeClipFadeEnvelope::default(),
            fade_out: signal_runtime::RuntimeClipFadeEnvelope::default(),
            clip_gain: signal_runtime::RuntimeClipGainEnvelope::default(),
        }])
        .expect("public server stretch clip-processing clip should reconcile");
    runtime
        .apply_transport_projection(signal_runtime::TransportProjection {
            playing: false,
            timeline_position_samples: 0,
            tempo_bpm: 180.0,
            loop_state: None,
        })
        .expect("public server stretch transport projection should apply");

    let host = ServerRuntimeHost::new(runtime);
    let report = host.supervisor_report();
    assert_eq!(report.observation.stretch_engine_snapshot.clip_count, 1);
    assert_eq!(
        report.observation.stretch_engine_snapshot.ready_clip_count,
        1
    );
    assert_eq!(
        report
            .observation
            .stretch_engine_snapshot
            .sample_domain_clip_count,
        1
    );
    assert_eq!(
        report.observation.stretch_engine_snapshot.clips[0].engine_class,
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
fn server_shared_host_edge_exports_runtime_marker_analysis_truth() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::server(48_000, 512));
    runtime
        .handshake(signal_runtime::HandshakeRequest {
            client_version: "public-host-server-marker-analysis".into(),
            anticipative_preferred: true,
            max_sample_rate_hint: Some(96_000),
        })
        .expect("public server marker-analysis handshake should succeed");
    runtime
        .configure(RuntimeConfigRequest::new(48_000, 256))
        .expect("public server marker-analysis configure should succeed");

    let ready_path = public_server_media_fixture_path("marker-analysis-ready");
    write_public_transient_test_wav(&ready_path);
    runtime
        .reconcile_media_assets(vec![signal_runtime::RuntimeMediaAssetRegistration {
            asset_id: "asset:sha256:host-server-marker-analysis-ready".into(),
            content_hash: "host-server-marker-analysis-ready".into(),
            source_path: ready_path.display().to_string(),
            file_name: "host-server-marker-analysis-ready.wav".into(),
            byte_size: fs::metadata(&ready_path).unwrap().len(),
            sample_rate_hz: 48_000,
            channel_count: 1,
            duration_samples: 48_000,
            waveform_bin_count: 8,
        }])
        .expect("public server marker-analysis media asset should reconcile");
    runtime
        .reconcile_warp_clips(vec![signal_runtime::RuntimeWarpClipRegistration {
            clip_id: "clip:host-server-marker-analysis".into(),
            media_asset_id: Some("asset:sha256:host-server-marker-analysis-ready".into()),
            mode: signal_runtime::RuntimeWarpMode::ElastiqueDraft,
            source_tempo_bpm: Some(120.0),
            anchor_timeline_samples: 0,
            start_samples: 0,
            duration_samples: 48_000,
        }])
        .expect("public server marker-analysis warp clip should reconcile");
    runtime
        .reconcile_clip_processing_clips(vec![signal_runtime::RuntimeClipProcessingRegistration {
            clip_id: "clip:host-server-marker-analysis".into(),
            media_asset_id: Some("asset:sha256:host-server-marker-analysis-ready".into()),
            warp_mode: signal_runtime::RuntimeWarpMode::ElastiqueDraft,
            start_samples: 0,
            duration_samples: 48_000,
            fade_in: signal_runtime::RuntimeClipFadeEnvelope::default(),
            fade_out: signal_runtime::RuntimeClipFadeEnvelope::default(),
            clip_gain: signal_runtime::RuntimeClipGainEnvelope::default(),
        }])
        .expect("public server marker-analysis clip-processing clip should reconcile");
    runtime
        .apply_transport_projection(signal_runtime::TransportProjection {
            playing: false,
            timeline_position_samples: 0,
            tempo_bpm: 180.0,
            loop_state: None,
        })
        .expect("public server marker-analysis transport projection should apply");

    let host = ServerRuntimeHost::new(runtime);
    let report = host.supervisor_report();
    assert_eq!(report.observation.marker_analysis_snapshot.clip_count, 1);
    assert_eq!(
        report.observation.marker_analysis_snapshot.ready_clip_count,
        1
    );
    assert_eq!(
        report
            .observation
            .marker_analysis_snapshot
            .tempo_assist_ready_clip_count,
        1
    );
    assert!(
        report
            .observation
            .marker_analysis_snapshot
            .warp_marker_count
            > 0
    );
    assert!(
        report
            .observation
            .marker_analysis_snapshot
            .transient_anchor_count
            > 0
    );
    assert_eq!(
        report.observation.marker_analysis_snapshot.clips[0].tempo_assist_posture,
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
fn server_shared_host_edge_exports_runtime_transform_artifact_truth() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::server(48_000, 512));
    runtime
        .handshake(signal_runtime::HandshakeRequest {
            client_version: "public-host-server-transform-artifact".into(),
            anticipative_preferred: true,
            max_sample_rate_hint: Some(96_000),
        })
        .expect("public server transform-artifact handshake should succeed");
    runtime
        .configure(RuntimeConfigRequest::new(48_000, 256))
        .expect("public server transform-artifact configure should succeed");

    let ready_path = public_server_media_fixture_path("transform-artifact-ready");
    write_public_transient_test_wav(&ready_path);
    runtime
        .reconcile_media_assets(vec![signal_runtime::RuntimeMediaAssetRegistration {
            asset_id: "asset:sha256:host-server-transform-artifact-ready".into(),
            content_hash: "host-server-transform-artifact-ready".into(),
            source_path: ready_path.display().to_string(),
            file_name: "host-server-transform-artifact-ready.wav".into(),
            byte_size: fs::metadata(&ready_path).unwrap().len(),
            sample_rate_hz: 48_000,
            channel_count: 1,
            duration_samples: 48_000,
            waveform_bin_count: 8,
        }])
        .expect("public server transform-artifact media asset should reconcile");
    runtime
        .reconcile_warp_clips(vec![signal_runtime::RuntimeWarpClipRegistration {
            clip_id: "clip:host-server-transform-artifact".into(),
            media_asset_id: Some("asset:sha256:host-server-transform-artifact-ready".into()),
            mode: signal_runtime::RuntimeWarpMode::ElastiqueDraft,
            source_tempo_bpm: Some(120.0),
            anchor_timeline_samples: 0,
            start_samples: 0,
            duration_samples: 48_000,
        }])
        .expect("public server transform-artifact warp clip should reconcile");
    runtime
        .reconcile_clip_processing_clips(vec![signal_runtime::RuntimeClipProcessingRegistration {
            clip_id: "clip:host-server-transform-artifact".into(),
            media_asset_id: Some("asset:sha256:host-server-transform-artifact-ready".into()),
            warp_mode: signal_runtime::RuntimeWarpMode::ElastiqueDraft,
            start_samples: 0,
            duration_samples: 48_000,
            fade_in: signal_runtime::RuntimeClipFadeEnvelope::default(),
            fade_out: signal_runtime::RuntimeClipFadeEnvelope::default(),
            clip_gain: signal_runtime::RuntimeClipGainEnvelope::default(),
        }])
        .expect("public server transform-artifact clip-processing clip should reconcile");
    runtime
        .apply_transport_projection(signal_runtime::TransportProjection {
            playing: false,
            timeline_position_samples: 0,
            tempo_bpm: 180.0,
            loop_state: None,
        })
        .expect("public server transform-artifact transport projection should apply");

    let host = ServerRuntimeHost::new(runtime);
    let report = host.supervisor_report();
    assert_eq!(report.observation.transform_artifact_snapshot.clip_count, 1);
    assert_eq!(
        report
            .observation
            .transform_artifact_snapshot
            .ready_clip_count,
        1
    );
    assert_eq!(
        report
            .observation
            .transform_artifact_snapshot
            .reusable_clip_count,
        1
    );
    assert_eq!(
        report
            .observation
            .transform_artifact_snapshot
            .transform_persistence
            .persistence_posture,
        signal_runtime::RuntimeTransformPersistencePosture::AssetScopedTransformPersistence
    );
    assert_eq!(
        report.observation.transform_artifact_snapshot.clips[0].reuse_state,
        signal_runtime::RuntimeTransformArtifactReuseState::Reusable
    );

    let rendered = report.render_json();
    assert!(rendered.contains("\"transform_artifact_snapshot\":{"));
    assert!(rendered.contains("\"clip_count\":1"));
    assert!(rendered.contains("\"reusable_clip_count\":1"));
    assert!(rendered.contains("\"reuse_state\":\"Reusable\""));
    assert!(rendered.contains("\"persistence_posture\":\"AssetScopedTransformPersistence\""));

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
fn server_shared_host_edge_exports_runtime_preview_transform_truth() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::server(48_000, 512));
    runtime
        .handshake(signal_runtime::HandshakeRequest {
            client_version: "public-host-server-preview-transform".into(),
            anticipative_preferred: true,
            max_sample_rate_hint: Some(96_000),
        })
        .expect("public server preview-transform handshake should succeed");
    runtime
        .configure(RuntimeConfigRequest::new(48_000, 256))
        .expect("public server preview-transform configure should succeed");

    let ready_path = public_server_media_fixture_path("preview-transform-ready");
    write_public_transient_test_wav(&ready_path);
    runtime
        .reconcile_media_assets(vec![signal_runtime::RuntimeMediaAssetRegistration {
            asset_id: "asset:sha256:host-server-preview-transform-ready".into(),
            content_hash: "host-server-preview-transform-ready".into(),
            source_path: ready_path.display().to_string(),
            file_name: "host-server-preview-transform-ready.wav".into(),
            byte_size: fs::metadata(&ready_path).unwrap().len(),
            sample_rate_hz: 48_000,
            channel_count: 1,
            duration_samples: 48_000,
            waveform_bin_count: 8,
        }])
        .expect("public server preview-transform media asset should reconcile");
    runtime
        .reconcile_warp_clips(vec![signal_runtime::RuntimeWarpClipRegistration {
            clip_id: "clip:host-server-preview-transform".into(),
            media_asset_id: Some("asset:sha256:host-server-preview-transform-ready".into()),
            mode: signal_runtime::RuntimeWarpMode::ElastiqueDraft,
            source_tempo_bpm: Some(120.0),
            anchor_timeline_samples: 0,
            start_samples: 0,
            duration_samples: 48_000,
        }])
        .expect("public server preview-transform warp clip should reconcile");
    runtime
        .reconcile_clip_processing_clips(vec![signal_runtime::RuntimeClipProcessingRegistration {
            clip_id: "clip:host-server-preview-transform".into(),
            media_asset_id: Some("asset:sha256:host-server-preview-transform-ready".into()),
            warp_mode: signal_runtime::RuntimeWarpMode::ElastiqueDraft,
            start_samples: 0,
            duration_samples: 48_000,
            fade_in: signal_runtime::RuntimeClipFadeEnvelope::default(),
            fade_out: signal_runtime::RuntimeClipFadeEnvelope::default(),
            clip_gain: signal_runtime::RuntimeClipGainEnvelope::default(),
        }])
        .expect("public server preview-transform clip-processing clip should reconcile");
    runtime
        .apply_transport_projection(signal_runtime::TransportProjection {
            playing: false,
            timeline_position_samples: 0,
            tempo_bpm: 180.0,
            loop_state: None,
        })
        .expect("public server preview-transform transport projection should apply");
    runtime
        .start_media_preview("asset:sha256:host-server-preview-transform-ready")
        .expect("public server preview-transform media preview should start");

    let host = ServerRuntimeHost::new(runtime);
    let report = host.supervisor_report();
    assert_eq!(report.observation.preview_transform_snapshot.clip_count, 1);
    assert_eq!(
        report
            .observation
            .preview_transform_snapshot
            .ready_clip_count,
        1
    );
    assert_eq!(
        report
            .observation
            .preview_transform_snapshot
            .active_audition_clip_count,
        1
    );
    assert_eq!(
        report
            .observation
            .preview_transform_snapshot
            .preview_device_policy
            .routing_posture,
        signal_runtime::RuntimePreviewOutputRoutingPosture::GuardedPreviewOutputRouting
    );
    assert_eq!(
        report
            .observation
            .preview_transform_snapshot
            .preview_workflow
            .queue_posture,
        signal_runtime::RuntimePreviewBrowserQueuePosture::SingleActivePreviewQueue
    );
    assert_eq!(
        report
            .observation
            .preview_transform_snapshot
            .preview_workflow
            .transform_scheduling_outcome,
        signal_runtime::RuntimePreviewTransformSchedulingOutcome::PreferArtifactBackedPreview
    );
    assert_eq!(
        report
            .observation
            .preview_transform_snapshot
            .artifact_backed_clip_count,
        1
    );
    assert_eq!(
        report.observation.preview_transform_snapshot.clips[0].service_class,
        signal_runtime::RuntimePreviewTransformServiceClass::ArtifactBacked
    );

    let rendered = report.render_json();
    assert!(rendered.contains("\"preview_transform_snapshot\":{"));
    assert!(rendered.contains("\"active_audition_clip_count\":1"));
    assert!(rendered.contains("\"artifact_backed_clip_count\":1"));
    assert!(rendered.contains("\"service_class\":\"ArtifactBacked\""));
    assert!(rendered.contains("\"routing_posture\":\"GuardedPreviewOutputRouting\""));
    assert!(rendered.contains("\"queue_posture\":\"SingleActivePreviewQueue\""));

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
fn server_shared_host_edge_exports_runtime_critical_path_truth() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::server(48_000, 48));
    runtime
        .handshake(signal_runtime::HandshakeRequest {
            client_version: "public-host-edge-critical-path".into(),
            anticipative_preferred: true,
            max_sample_rate_hint: Some(96_000),
        })
        .expect("server host-edge critical-path handshake should succeed");
    runtime
        .configure(RuntimeConfigRequest::new(48_000, 48))
        .expect("server host-edge critical-path configure should succeed");
    apply_public_capture_graph(&mut runtime, "graph:host-server:critical-path");
    runtime
        .process_engine_block(
            1,
            1,
            synthetic_stereo_block(SampleRate(48_000), FrameCount(48), 53),
        )
        .expect("server host-edge critical-path block should process");

    let host = ServerRuntimeHost::new(runtime);
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
        .expect("server host-edge critical-path lane should resolve to a typed worker summary");
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
fn server_shared_host_edge_exports_runtime_deferred_work_policy_truth() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::server(48_000, 48));
    runtime
        .handshake(signal_runtime::HandshakeRequest {
            client_version: "public-host-edge-deferred-work".into(),
            anticipative_preferred: true,
            max_sample_rate_hint: Some(96_000),
        })
        .expect("server host-edge deferred-work handshake should succeed");
    runtime
        .configure(RuntimeConfigRequest::new(48_000, 48))
        .expect("server host-edge deferred-work configure should succeed");
    runtime
        .purge_offline_render_artifacts(RuntimeOfflineRenderPurgeRequest {
            request_id: String::new(),
            artifact_root_path: None,
            report_path: None,
        })
        .expect_err("empty purge request id should record terminal deferred-work policy");

    let host = ServerRuntimeHost::new(runtime);
    let report = host.supervisor_report();
    let receipt = report
        .observation
        .last_deferred_service_receipt
        .as_ref()
        .expect("server host-edge report should expose deferred-work policy receipt");
    assert_eq!(receipt.decision, RuntimeDeferredServiceDecision::Abort);
    assert_eq!(receipt.reason, RuntimeDeferredServiceReason::InvalidRequest);
    assert_eq!(
        receipt.priority_band,
        RuntimeDeferredServicePriorityBand::Maintenance
    );
    assert_eq!(receipt.blocking_priority_band, None);
    assert_eq!(
        receipt.cancellation_cause,
        Some(RuntimeDeferredServiceCancellationCause::InvalidRequest)
    );
    assert_eq!(receipt.cancelled_work_item_count, 1);

    let performance = report.performance_snapshot();
    assert_eq!(
        performance.background_service_decision,
        Some(RuntimeDeferredServiceDecision::Abort)
    );
    assert_eq!(
        performance.background_service_reason,
        Some(RuntimeDeferredServiceReason::InvalidRequest)
    );
    assert_eq!(
        performance.background_service_priority_band,
        Some(RuntimeDeferredServicePriorityBand::Maintenance)
    );
    assert_eq!(
        performance.background_service_cancellation_cause,
        Some(RuntimeDeferredServiceCancellationCause::InvalidRequest)
    );
    assert_eq!(performance.background_service_cancelled_work_item_count, 1);

    let rendered = report.render_json();
    assert!(rendered.contains("\"last_deferred_service\":{"));
    assert!(rendered.contains("\"priority_band\":\"Maintenance\""));
    assert!(rendered.contains("\"cancellation_cause\":\"InvalidRequest\""));
}

#[test]
fn server_shared_host_edge_exports_restartable_and_terminal_offline_render_session_truth() {
    let mut restartable_runtime = SignalRuntime::new(RuntimeConfig::server(48_000, 512));
    restartable_runtime
        .handshake(signal_runtime::HandshakeRequest {
            client_version: "public-host-edge-render".into(),
            anticipative_preferred: true,
            max_sample_rate_hint: Some(96_000),
        })
        .unwrap();
    restartable_runtime
        .configure(RuntimeConfigRequest::new(48_000, 512))
        .unwrap();
    apply_public_render_graph(
        &mut restartable_runtime,
        "graph:host-server:render-restartable",
    );
    restartable_runtime.start().unwrap();
    restartable_runtime
        .begin_offline_render_execution(RuntimeOfflineRenderRequest {
            request_id: "render:host-server:restartable".into(),
            timeline_start_samples: 0,
            duration_samples: 512,
            export_sample_rate_hz: 48_000,
            include_main_mix: true,
            artifact_root_path: None,
            stem_targets: Vec::new(),
            freeze_artifacts: Vec::new(),
        })
        .unwrap();
    restartable_runtime
        .advance_offline_render_execution("render:host-server:restartable")
        .unwrap();
    restartable_runtime
        .stop(StopReason::DeviceReconfigure)
        .unwrap();
    restartable_runtime
        .restart(RestartRequest { reconfigure: None })
        .unwrap();

    let restartable_host = ServerRuntimeHost::new(restartable_runtime);
    let restartable_report = restartable_host.supervisor_report();
    assert_eq!(
        restartable_report
            .observation
            .offline_render_session_snapshot
            .active_sessions
            .first()
            .map(|session| session.interruption_class),
        Some(RuntimeInterruptionClass::Restartable)
    );

    let mut terminal_runtime = SignalRuntime::new(RuntimeConfig::server(48_000, 512));
    terminal_runtime
        .handshake(signal_runtime::HandshakeRequest {
            client_version: "public-host-edge-render".into(),
            anticipative_preferred: true,
            max_sample_rate_hint: Some(96_000),
        })
        .unwrap();
    terminal_runtime
        .configure(RuntimeConfigRequest::new(48_000, 512))
        .unwrap();
    apply_public_render_graph(&mut terminal_runtime, "graph:host-server:render-terminal");
    terminal_runtime
        .begin_offline_render_execution(RuntimeOfflineRenderRequest {
            request_id: "render:host-server:terminal".into(),
            timeline_start_samples: 0,
            duration_samples: 512,
            export_sample_rate_hz: 48_000,
            include_main_mix: true,
            artifact_root_path: Some("/dev/null/signal-host-server-render-terminal".into()),
            stem_targets: Vec::new(),
            freeze_artifacts: Vec::new(),
        })
        .unwrap();
    let mut terminal_error_observed = false;
    for _ in 0..16 {
        match terminal_runtime.advance_offline_render_execution("render:host-server:terminal") {
            Ok(_) => continue,
            Err(_) => {
                terminal_error_observed = true;
                break;
            }
        }
    }
    assert!(terminal_error_observed);

    let terminal_host = ServerRuntimeHost::new(terminal_runtime);
    let terminal_report = terminal_host.supervisor_report();
    assert_eq!(
        terminal_report
            .observation
            .offline_render_session_snapshot
            .last_session
            .as_ref()
            .map(|session| session.state),
        Some(RuntimeOfflineRenderExecutionState::Failed)
    );
    assert_eq!(
        terminal_report
            .observation
            .offline_render_session_snapshot
            .last_session
            .as_ref()
            .map(|session| session.interruption_class),
        Some(RuntimeInterruptionClass::Terminal)
    );
    let rendered = terminal_report.render_json();
    assert!(rendered.contains("\"offline_render_session_snapshot\":{"));
    assert!(rendered.contains("\"state\":\"Failed\""));
    assert!(rendered.contains("\"interruption_class\":\"Terminal\""));
}
