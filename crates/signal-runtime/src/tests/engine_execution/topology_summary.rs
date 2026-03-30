use super::super::*;

#[test]
fn runtime_execution_topology_summarizes_send_return_routes_explicitly() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
    handshake_and_configure_with_disabled_forecast(&mut runtime, true);
    runtime
        .apply_graph_projection(GraphProjection {
            graph_id: "graph:runtime:send-return-summary".into(),
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
        .expect("apply projected graph");
    runtime
        .apply_graph_contract_projection(GraphContractProjection {
            graph_id: "graph:runtime:send-return-summary".into(),
            contract_count: 5,
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
                    node_id: "bus-dry".into(),
                    buffer_contract: GraphNodeBufferContractProjection {
                        input: GraphNodeBusEndpointProjection {
                            bus_id: "bus:track:lead".into(),
                            channels: ChannelLayout::Stereo,
                        },
                        output: GraphNodeBusEndpointProjection {
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
                        input: GraphNodeBusEndpointProjection {
                            bus_id: "bus:track:lead".into(),
                            channels: ChannelLayout::Stereo,
                        },
                        output: GraphNodeBusEndpointProjection {
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
                        input: GraphNodeBusEndpointProjection {
                            bus_id: "bus:fx:plate".into(),
                            channels: ChannelLayout::Stereo,
                        },
                        output: GraphNodeBusEndpointProjection {
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
                        input: GraphNodeBusEndpointProjection {
                            bus_id: "bus:mix:master".into(),
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

    let block = synthetic_stereo_block(SampleRate(48_000), FrameCount(256), 2);
    runtime
        .process_engine_block(3, 5, block)
        .expect("process send return topology block");

    let metering = runtime.get_metering_snapshot();
    assert_eq!(metering.send_returns.len(), 1);
    assert_eq!(metering.bus_connection_count, 5);
    assert_eq!(metering.auxiliary_path_count, 3);
    assert!(metering.send_returns.iter().any(|send_return| {
        send_return.send_return_id == "fx:plate"
            && send_return.aggregate.meter_count == 2
            && send_return
                .aggregate
                .metered_bus_ids
                .contains(&"bus:fx:plate".to_string())
            && send_return
                .aggregate
                .metered_bus_ids
                .contains(&"bus:mix:master".to_string())
    }));
    assert!(metering.bus_connections.iter().any(|connection| {
        connection.connection_id == "send-fx:bus:fx:plate->return-fx:bus:fx:plate"
            && connection.source_bus_role == crate::RuntimeBusRole::AuxSend
            && connection.target_bus_role == crate::RuntimeBusRole::AuxReturn
            && connection.auxiliary_path_kind == Some(crate::RuntimeAuxiliaryPathKind::SendReturn)
            && connection.auxiliary_path_id.as_deref() == Some("send_return:fx:plate")
    }));
    assert!(metering.auxiliary_paths.iter().any(|path| {
        path.auxiliary_path_id == "send_return:fx:plate"
            && path.path_kind == crate::RuntimeAuxiliaryPathKind::SendReturn
            && path.bus_role == crate::RuntimeBusRole::AuxSend
            && path
                .connection_ids
                .contains(&"send-fx:bus:fx:plate->return-fx:bus:fx:plate".to_string())
    }));
    assert!(metering.auxiliary_paths.iter().any(|path| {
        path.auxiliary_path_id == "bus_group:mix:master"
            && path.path_kind == crate::RuntimeAuxiliaryPathKind::Submix
            && path.bus_role == crate::RuntimeBusRole::Submix
            && path.source_node_ids.contains(&"bus-dry".to_string())
            && path.target_node_ids.contains(&"output-main".to_string())
    }));

    let observation = RuntimeObservationReport::capture(&runtime, &RuntimeEventRecorder::default());
    assert_eq!(
        observation
            .execution_topology_summary
            .send_return_node_count,
        2
    );
    assert_eq!(
        observation
            .execution_topology_summary
            .send_return_group_count,
        1
    );
    assert_eq!(
        observation.execution_topology_summary.bus_connection_count,
        5
    );
    assert_eq!(
        observation.execution_topology_summary.auxiliary_path_count,
        3
    );
    assert_eq!(observation.execution_topology_summary.send_returns.len(), 1);
    assert_eq!(observation.metering_snapshot.send_returns.len(), 1);
    assert_eq!(observation.metering_snapshot.bus_connection_count, 5);
    assert_eq!(observation.metering_snapshot.auxiliary_path_count, 3);
    assert!(observation
        .execution_topology_summary
        .send_returns
        .iter()
        .any(|send_return| {
            send_return.send_return_id == "fx:plate"
                && send_return.send_node_ids == vec!["send-fx".to_string()]
                && send_return.return_node_ids == vec!["return-fx".to_string()]
                && send_return
                    .input_bus_ids
                    .contains(&"bus:track:lead".to_string())
                && send_return
                    .input_bus_ids
                    .contains(&"bus:fx:plate".to_string())
                && send_return
                    .output_bus_ids
                    .contains(&"bus:fx:plate".to_string())
                && send_return
                    .output_bus_ids
                    .contains(&"bus:mix:master".to_string())
        }));
    assert!(observation
        .execution_topology_summary
        .bus_connections
        .iter()
        .any(|connection| {
            connection.connection_id == "send-fx:bus:fx:plate->return-fx:bus:fx:plate"
                && connection.source_bus_role == crate::RuntimeBusRole::AuxSend
                && connection.target_bus_role == crate::RuntimeBusRole::AuxReturn
        }));
    assert!(observation
        .execution_topology_summary
        .auxiliary_paths
        .iter()
        .any(|path| {
            path.auxiliary_path_id == "send_return:fx:plate"
                && path
                    .connection_ids
                    .contains(&"send-fx:bus:fx:plate->return-fx:bus:fx:plate".to_string())
                && path
                    .connection_ids
                    .contains(&"return-fx:bus:mix:master->output-main:bus:mix:master".to_string())
        }));
    let supervisor = RuntimeSupervisorReport::capture(&runtime, &RuntimeEventRecorder::default());
    assert!(supervisor
        .render_multiline()
        .contains("metering_snapshot_send_return_0=fx:plate"));
    assert!(supervisor
        .render_multiline()
        .contains("execution_topology_summary_bus_connection_count=5"));
    assert!(supervisor
        .render_multiline()
        .contains("execution_topology_summary_auxiliary_path_0="));
    let json = supervisor.render_json();
    assert!(json.contains("\"metering_snapshot\":{\"meter_count\":"));
    assert!(json.contains("\"send_return_group_count\":1"));
    assert!(json.contains("\"send_returns\":["));
    assert!(json.contains("\"send_return_id\":\"fx:plate\""));
    assert!(json.contains("\"bus_connection_count\":5"));
    assert!(json.contains("\"auxiliary_path_count\":3"));
    assert!(json.contains("\"connection_id\":\"send-fx:bus:fx:plate->return-fx:bus:fx:plate\""));
    assert!(json.contains("\"auxiliary_path_id\":\"send_return:fx:plate\""));
}
