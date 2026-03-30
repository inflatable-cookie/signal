use super::*;

pub(super) fn apply_plugin_continuity_graph(
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
                    stages: vec![GraphStageSpec::HardClip { threshold: 0.7 }],
                })
                .collect(),
        })
        .expect("plugin continuity graph should apply");
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
                        track_lane_id: Some("track:plugin-continuity".into()),
                        bus_group_id: Some("mix:tracks".into()),
                        console_group_id: None,
                        send_return_id: None,
                    },
                })
                .collect(),
        })
        .expect("plugin continuity contracts should apply");
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
        .expect("plugin continuity bindings should apply");
}

pub(super) fn record_ready_plugin_sandbox(
    runtime: &mut SignalRuntime,
    sandbox_id: &str,
    plugin_format: PluginFormat,
    plugin_type_id: &str,
    processing_epoch: u64,
) {
    runtime.record_plugin_sandbox_spec(&PluginSandboxSpec {
        sandbox_id: sandbox_id.into(),
        plugin_format,
        plugin_type_id: Some(plugin_type_id.into()),
    });
    runtime.record_plugin_sandbox_lifecycle(
        sandbox_id,
        PluginSandboxLifecycleStage::InstancePrepared,
        Some(processing_epoch),
    );
    runtime.record_plugin_sandbox_transport(
        sandbox_id,
        format!("lease-{sandbox_id}"),
        format!("region-{sandbox_id}"),
        PluginSandboxTransportStage::Attached,
        Some(processing_epoch),
        None,
    );
}

pub(super) fn apply_latency_runtime_graph(runtime: &mut SignalRuntime, graph_id: &str) {
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
        .unwrap();
}

pub(super) fn install_scheduler_topology_runtime_graph(
    runtime: &mut SignalRuntime,
    graph_id: &str,
    track_lane_ids: &[&str],
    include_missing_track_lane_id: bool,
) {
    let mut nodes = vec![GraphNodeSpec {
        node_id: "lookahead".into(),
        execution_class: GraphNodeExecutionClass::LatencyBearing,
        latency_samples: 32,
        tail_samples: 0,
        buffer_contract: GraphNodeBufferContract {
            input: GraphNodeBusEndpoint::new("main:in", ChannelLayout::Stereo),
            output: GraphNodeBusEndpoint::new("bus:lookahead", ChannelLayout::Stereo),
            ..GraphNodeBufferContract::default()
        },
        topology: GraphNodeTopologyMetadata {
            role: Some(GraphNodeTopologyRole::Utility),
            track_lane_id: None,
            bus_group_id: None,
            console_group_id: None,
            send_return_id: None,
        },
        stages: vec![GraphStageSpec::Gain { linear: 0.5 }],
    }];

    for (index, lane_id) in track_lane_ids.iter().enumerate() {
        nodes.push(GraphNodeSpec {
            node_id: format!("track-{index}"),
            execution_class: GraphNodeExecutionClass::Stateful,
            latency_samples: 0,
            tail_samples: 0,
            buffer_contract: GraphNodeBufferContract {
                input: GraphNodeBusEndpoint::new("main:in", ChannelLayout::Stereo),
                output: GraphNodeBusEndpoint::new("bus:tracks", ChannelLayout::Stereo),
                ..GraphNodeBufferContract::default()
            },
            topology: GraphNodeTopologyMetadata {
                role: Some(GraphNodeTopologyRole::TrackLane),
                track_lane_id: Some((*lane_id).into()),
                bus_group_id: Some("mix:tracks".into()),
                console_group_id: None,
                send_return_id: None,
            },
            stages: vec![GraphStageSpec::Gain { linear: 0.8 }],
        });
    }

    if include_missing_track_lane_id {
        nodes.push(GraphNodeSpec {
            node_id: "track-missing".into(),
            execution_class: GraphNodeExecutionClass::Stateful,
            latency_samples: 0,
            tail_samples: 0,
            buffer_contract: GraphNodeBufferContract {
                input: GraphNodeBusEndpoint::new("main:in", ChannelLayout::Stereo),
                output: GraphNodeBusEndpoint::new("bus:tracks", ChannelLayout::Stereo),
                ..GraphNodeBufferContract::default()
            },
            topology: GraphNodeTopologyMetadata {
                role: Some(GraphNodeTopologyRole::TrackLane),
                track_lane_id: None,
                bus_group_id: Some("mix:tracks".into()),
                console_group_id: None,
                send_return_id: None,
            },
            stages: vec![GraphStageSpec::Gain { linear: 0.7 }],
        });
    }

    nodes.push(GraphNodeSpec {
        node_id: "bus-main".into(),
        execution_class: GraphNodeExecutionClass::Stateful,
        latency_samples: 0,
        tail_samples: 0,
        buffer_contract: GraphNodeBufferContract {
            input: GraphNodeBusEndpoint::new("bus:tracks", ChannelLayout::Stereo),
            output: GraphNodeBusEndpoint::new("bus:master", ChannelLayout::Stereo),
            ..GraphNodeBufferContract::default()
        },
        topology: GraphNodeTopologyMetadata {
            role: Some(GraphNodeTopologyRole::Bus),
            track_lane_id: None,
            bus_group_id: Some("mix:master".into()),
            console_group_id: None,
            send_return_id: None,
        },
        stages: vec![GraphStageSpec::HardClip { threshold: 0.9 }],
    });

    nodes.push(GraphNodeSpec {
        node_id: "console-main".into(),
        execution_class: GraphNodeExecutionClass::PureTransform,
        latency_samples: 0,
        tail_samples: 0,
        buffer_contract: GraphNodeBufferContract {
            input: GraphNodeBusEndpoint::new("bus:master", ChannelLayout::Stereo),
            output: GraphNodeBusEndpoint::new("main:out", ChannelLayout::Stereo),
            ..GraphNodeBufferContract::default()
        },
        topology: GraphNodeTopologyMetadata {
            role: Some(GraphNodeTopologyRole::ConsoleNode),
            track_lane_id: None,
            bus_group_id: None,
            console_group_id: Some("console:main".into()),
            send_return_id: None,
        },
        stages: vec![GraphStageSpec::HardClip { threshold: 0.8 }],
    });

    runtime.engine.graph = Some(ExecutableGraph::new(graph_id, nodes));
    runtime
        .engine
        .refresh_planning(runtime.anticipative_enabled);
    runtime.refresh_scheduler_topology_summary();
}
