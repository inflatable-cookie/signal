use signal_graph::{GraphNodeExecutionClass, GraphNodeTopologyRole, GraphStageSpec};
use signal_primitives::ChannelLayout;
use signal_runtime::{
    GraphContractProjection, GraphNodeBufferContractProjection, GraphNodeBusEndpointProjection,
    GraphNodeContractProjection, GraphNodeProjection, GraphNodeTopologyProjection, GraphProjection,
};

use super::super::LOCAL_DEMO_GRAPH_ID;

pub(crate) fn local_demo_graph_projection() -> GraphProjection {
    GraphProjection {
        graph_id: LOCAL_DEMO_GRAPH_ID.into(),
        node_count: 4,
        nodes: vec![
            GraphNodeProjection {
                node_id: "track-input".into(),
                execution_class: GraphNodeExecutionClass::LatencyBearing,
                latency_samples: 24,
                stages: vec![
                    GraphStageSpec::Gain { linear: 0.75 },
                    GraphStageSpec::Bias { amount: 0.05 },
                    GraphStageSpec::TanhDrive { drive: 1.35 },
                ],
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
                stages: vec![GraphStageSpec::Gain { linear: 0.9 }],
            },
            GraphNodeProjection {
                node_id: "output-main".into(),
                execution_class: GraphNodeExecutionClass::Stateful,
                latency_samples: 0,
                stages: vec![
                    GraphStageSpec::StereoBalance { balance: -0.2 },
                    GraphStageSpec::HardClip { threshold: 0.8 },
                ],
            },
        ],
    }
}

pub(crate) fn local_demo_graph_contract_projection(graph_id: &str) -> GraphContractProjection {
    GraphContractProjection {
        graph_id: graph_id.into(),
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
    }
}
