use super::super::*;

pub(crate) fn apply_public_multichannel_graph(runtime: &mut SignalRuntime, graph_id: &str) {
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

pub(crate) fn apply_public_sidechain_graph(runtime: &mut SignalRuntime, graph_id: &str) {
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
                            channels: ChannelLayout::Stereo,
                        },
                        ..GraphNodeBufferContractProjection::default()
                    },
                    topology: GraphNodeTopologyProjection {
                        role: Some(GraphNodeTopologyRole::Send),
                        track_lane_id: Some("track:host-local:sidechain".into()),
                        bus_group_id: Some("mix:host-local:sidechain".into()),
                        console_group_id: None,
                        send_return_id: Some("send:return:host-local:compressor".into()),
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
                            source_id: "kick-sidechain".into(),
                            source_bus_id: Some("bus:sidechain:kick".into()),
                            target_kind: RuntimeSecondaryInputTargetKind::NodeInput,
                            target_id: "compressor".into(),
                            target_bus_id: "plugin:compressor:sidechain".into(),
                            attachment_policy: RuntimeSecondaryInputAttachmentPolicy::Required,
                            fallback_outcome:
                                RuntimeSecondaryInputFallbackOutcome::SafeModeDegradation,
                        }),
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
}

pub(crate) fn apply_public_multi_bus_graph(runtime: &mut SignalRuntime, graph_id: &str) {
    runtime
        .apply_graph_projection(GraphProjection {
            graph_id: graph_id.into(),
            node_count: 4,
            nodes: vec![
                GraphNodeProjection {
                    node_id: "drums".into(),
                    execution_class: GraphNodeExecutionClass::Stateful,
                    latency_samples: 0,
                    stages: vec![GraphStageSpec::Gain { linear: 0.88 }],
                },
                GraphNodeProjection {
                    node_id: "music".into(),
                    execution_class: GraphNodeExecutionClass::Stateful,
                    latency_samples: 0,
                    stages: vec![GraphStageSpec::Gain { linear: 0.9 }],
                },
                GraphNodeProjection {
                    node_id: "drum-bus".into(),
                    execution_class: GraphNodeExecutionClass::PluginBacked,
                    latency_samples: 16,
                    stages: vec![GraphStageSpec::HardClip { threshold: 0.8 }],
                },
                GraphNodeProjection {
                    node_id: "master".into(),
                    execution_class: GraphNodeExecutionClass::LatencyBearing,
                    latency_samples: 24,
                    stages: vec![GraphStageSpec::HardClip { threshold: 0.75 }],
                },
            ],
        })
        .expect("public host-edge multi-bus graph should apply");
}
