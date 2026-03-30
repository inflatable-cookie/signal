use super::*;

#[test]
fn runtime_graph_contract_projection_updates_execution_topology_for_projected_graphs() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
    handshake_and_configure_with_disabled_forecast(&mut runtime, true);
    runtime
        .apply_graph_projection(GraphProjection {
            graph_id: "graph:runtime:projected-topology".into(),
            node_count: 4,
            nodes: vec![
                GraphNodeProjection {
                    node_id: "track-input".into(),
                    execution_class: GraphNodeExecutionClass::Stateful,
                    latency_samples: 0,
                    stages: vec![GraphStageSpec::Gain { linear: 0.8 }],
                },
                GraphNodeProjection {
                    node_id: "plugin-insert".into(),
                    execution_class: GraphNodeExecutionClass::PluginBacked,
                    latency_samples: 0,
                    stages: vec![GraphStageSpec::HardClip { threshold: 0.82 }],
                },
                GraphNodeProjection {
                    node_id: "bus-main".into(),
                    execution_class: GraphNodeExecutionClass::Stateful,
                    latency_samples: 0,
                    stages: vec![GraphStageSpec::Gain { linear: 0.95 }],
                },
                GraphNodeProjection {
                    node_id: "output-main".into(),
                    execution_class: GraphNodeExecutionClass::LatencyBearing,
                    latency_samples: 24,
                    stages: vec![GraphStageSpec::StereoBalance { balance: -0.15 }],
                },
            ],
        })
        .expect("apply projected graph");
    runtime
        .apply_graph_contract_projection(GraphContractProjection {
            graph_id: "graph:runtime:projected-topology".into(),
            contract_count: 4,
            nodes: vec![
                GraphNodeContractProjection {
                    node_id: "track-input".into(),
                    buffer_contract: GraphNodeBufferContractProjection {
                        input: GraphNodeBusEndpointProjection {
                            bus_id: "main:in".into(),
                            channels: ChannelLayout::Stereo,
                        },
                        output: GraphNodeBusEndpointProjection {
                            bus_id: "bus:track:lead".into(),
                            channels: ChannelLayout::Stereo,
                        },
                        ..GraphNodeBufferContractProjection::default()
                    },
                    topology: GraphNodeTopologyProjection {
                        role: Some(GraphNodeTopologyRole::TrackLane),
                        track_lane_id: Some("track:lead".into()),
                        bus_group_id: Some("mix:tracks".into()),
                        console_group_id: None,
                        send_return_id: None,
                    },
                },
                GraphNodeContractProjection {
                    node_id: "plugin-insert".into(),
                    buffer_contract: GraphNodeBufferContractProjection {
                        input: GraphNodeBusEndpointProjection {
                            bus_id: "bus:track:lead".into(),
                            channels: ChannelLayout::Stereo,
                        },
                        output: GraphNodeBusEndpointProjection {
                            bus_id: "bus:mix:tracks".into(),
                            channels: ChannelLayout::Stereo,
                        },
                        ..GraphNodeBufferContractProjection::default()
                    },
                    topology: GraphNodeTopologyProjection {
                        role: Some(GraphNodeTopologyRole::TrackLane),
                        track_lane_id: Some("track:lead".into()),
                        bus_group_id: Some("mix:tracks".into()),
                        console_group_id: None,
                        send_return_id: None,
                    },
                },
                GraphNodeContractProjection {
                    node_id: "bus-main".into(),
                    buffer_contract: GraphNodeBufferContractProjection {
                        input: GraphNodeBusEndpointProjection {
                            bus_id: "bus:mix:tracks".into(),
                            channels: ChannelLayout::Stereo,
                        },
                        output: GraphNodeBusEndpointProjection {
                            bus_id: "bus:console:main".into(),
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
                    node_id: "output-main".into(),
                    buffer_contract: GraphNodeBufferContractProjection {
                        input: GraphNodeBusEndpointProjection {
                            bus_id: "bus:console:main".into(),
                            channels: ChannelLayout::Stereo,
                        },
                        output: GraphNodeBusEndpointProjection {
                            bus_id: "main:out".into(),
                            channels: ChannelLayout::Stereo,
                        },
                        ..GraphNodeBufferContractProjection::default()
                    },
                    topology: GraphNodeTopologyProjection {
                        role: Some(GraphNodeTopologyRole::ConsoleNode),
                        track_lane_id: None,
                        bus_group_id: None,
                        console_group_id: Some("console:main".into()),
                        send_return_id: None,
                    },
                },
            ],
        })
        .expect("apply projected graph contracts");
    runtime
        .apply_plugin_backed_node_bindings(PluginBackedNodeBindingProjection {
            graph_id: "graph:runtime:projected-topology".into(),
            bindings: vec![PluginBackedNodeBinding {
                node_id: "plugin-insert".into(),
                sandbox_id: "sandbox:lead".into(),
            }],
        })
        .expect("apply plugin bindings");

    let block = synthetic_stereo_block(SampleRate(48_000), FrameCount(256), 1);
    runtime
        .process_engine_block(1, 1, block)
        .expect("process projected topology block");

    let observation = RuntimeObservationReport::capture(&runtime, &RuntimeEventRecorder::default());
    assert_eq!(observation.execution_topology_summary.node_count, 4);
    assert_eq!(
        observation.execution_topology_summary.track_lane_node_count,
        2
    );
    assert_eq!(observation.execution_topology_summary.bus_node_count, 1);
    assert_eq!(observation.execution_topology_summary.console_node_count, 1);
    assert_eq!(
        observation
            .execution_topology_summary
            .track_lane_group_count,
        1
    );
    assert_eq!(observation.execution_topology_summary.bus_group_count, 2);
    assert_eq!(
        observation.execution_topology_summary.console_group_count,
        1
    );
    assert_eq!(observation.execution_topology_summary.track_lanes.len(), 1);
    assert_eq!(observation.execution_topology_summary.bus_groups.len(), 2);
    assert_eq!(
        observation.execution_topology_summary.console_groups.len(),
        1
    );
    assert_eq!(
        observation
            .execution_topology_summary
            .plugin_chain
            .chain_count,
        1
    );
    assert_eq!(
        observation
            .execution_topology_summary
            .plugin_chain
            .stage_count,
        1
    );
    assert_eq!(
        observation
            .execution_topology_summary
            .plugin_chain
            .pending_render_stage_count,
        1
    );
    assert!(observation
        .execution_topology_summary
        .track_lanes
        .iter()
        .any(|track_lane| {
            track_lane.track_lane_id == "track:lead"
                && track_lane.bus_group_ids == vec!["mix:tracks".to_string()]
                && track_lane.plugin_chain.chain_count == 1
                && track_lane.plugin_chain.pending_render_stage_count == 1
                && track_lane
                    .output_bus_ids
                    .contains(&"bus:track:lead".to_string())
                && track_lane
                    .output_bus_ids
                    .contains(&"bus:mix:tracks".to_string())
        }));
    assert!(observation
        .execution_topology_summary
        .nodes
        .iter()
        .any(|node| {
            node.node_id == "track-input"
                && node.topology_role == GraphNodeTopologyRole::TrackLane
                && node.track_lane_id.as_deref() == Some("track:lead")
                && node.output_bus_id == "bus:track:lead"
        }));
    assert!(observation
        .execution_topology_summary
        .nodes
        .iter()
        .any(|node| {
            node.node_id == "plugin-insert"
                && node.plugin_sandbox_id.as_deref() == Some("sandbox:lead")
                && node.plugin_recall_state == Some(RuntimePluginRecallState::Cold)
                && node.plugin_compensation_state
                    == Some(RuntimePluginCompensationState::PendingRender)
                && node.plugin_realized_latency_samples.is_none()
                && node.input_bus_id == "bus:track:lead"
                && node.output_bus_id == "bus:mix:tracks"
        }));
    assert!(observation
        .execution_topology_summary
        .nodes
        .iter()
        .any(|node| {
            node.node_id == "bus-main"
                && node.topology_role == GraphNodeTopologyRole::Bus
                && node.bus_group_id.as_deref() == Some("mix:master")
        }));
    assert!(observation
        .execution_topology_summary
        .nodes
        .iter()
        .any(|node| {
            node.node_id == "output-main"
                && node.topology_role == GraphNodeTopologyRole::ConsoleNode
                && node.console_group_id.as_deref() == Some("console:main")
                && node.input_bus_id == "bus:console:main"
                && node.output_bus_id == "main:out"
        }));
    let supervisor = RuntimeSupervisorReport::capture(&runtime, &RuntimeEventRecorder::default());
    assert!(supervisor
        .render_multiline()
        .contains("execution_topology_summary_plugin_chain=1/1/1/0/0/0/0/0/0/0/0"));
    let json = supervisor.render_json();
    assert!(json.contains("\"plugin_chain\":{\"chain_count\":1"));
    assert!(json.contains("\"plugin_recall_state\":\"Cold\""));
    assert!(json.contains("\"plugin_compensation_state\":\"PendingRender\""));
}
