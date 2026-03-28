use super::{
    synthetic_stereo_block, AudioBuffer, ChannelLayout, ExecutableGraph, FrameCount,
    GraphDynamicStageStateModel, GraphExecutionContext, GraphExecutionLane, GraphExecutionRequest,
    GraphNodeBufferContract, GraphNodeBusEndpoint, GraphNodeExecutionClass, GraphNodePlanningGroup,
    GraphNodeSpec, GraphNodeTopologyMetadata, GraphNodeTopologyRole, GraphStageSpec, SampleRate,
};

fn test_node(
    node_id: &str,
    execution_class: GraphNodeExecutionClass,
    latency_samples: u32,
    stages: Vec<GraphStageSpec>,
) -> GraphNodeSpec {
    GraphNodeSpec {
        node_id: node_id.into(),
        execution_class,
        latency_samples,
        tail_samples: 0,
        buffer_contract: GraphNodeBufferContract::default(),
        topology: GraphNodeTopologyMetadata::default(),
        stages,
    }
}

fn routed_node(
    node_id: &str,
    input_bus: &str,
    input_channels: ChannelLayout,
    output_bus: &str,
    output_channels: ChannelLayout,
    role: GraphNodeTopologyRole,
    stages: Vec<GraphStageSpec>,
) -> GraphNodeSpec {
    let topology = match role {
        GraphNodeTopologyRole::Utility => GraphNodeTopologyMetadata {
            role: Some(role),
            track_lane_id: None,
            bus_group_id: None,
            console_group_id: None,
            send_return_id: None,
        },
        GraphNodeTopologyRole::TrackLane => GraphNodeTopologyMetadata {
            role: Some(role),
            track_lane_id: Some(node_id.into()),
            bus_group_id: None,
            console_group_id: None,
            send_return_id: None,
        },
        GraphNodeTopologyRole::Bus => GraphNodeTopologyMetadata {
            role: Some(role),
            track_lane_id: None,
            bus_group_id: Some(node_id.into()),
            console_group_id: None,
            send_return_id: None,
        },
        GraphNodeTopologyRole::Send | GraphNodeTopologyRole::Return => GraphNodeTopologyMetadata {
            role: Some(role),
            track_lane_id: None,
            bus_group_id: None,
            console_group_id: None,
            send_return_id: Some(node_id.into()),
        },
        GraphNodeTopologyRole::ConsoleNode => GraphNodeTopologyMetadata {
            role: Some(role),
            track_lane_id: None,
            bus_group_id: None,
            console_group_id: Some(node_id.into()),
            send_return_id: None,
        },
    };
    GraphNodeSpec {
        buffer_contract: GraphNodeBufferContract {
            input: GraphNodeBusEndpoint::new(input_bus, input_channels),
            output: GraphNodeBusEndpoint::new(output_bus, output_channels),
            ..GraphNodeBufferContract::default()
        },
        topology,
        ..test_node(node_id, GraphNodeExecutionClass::Stateful, 0, stages)
    }
}

#[test]
fn mono_mixdown_averages_channels() {
    let audio = AudioBuffer::from_interleaved(
        SampleRate(48_000),
        ChannelLayout::Stereo,
        vec![1.0, -1.0, 0.25, 0.75],
    );

    assert_eq!(audio.to_mono(), vec![0.0, 0.5]);
}

#[test]
fn executable_graph_processes_buffer_and_reports_metrics() {
    let mut buffer = synthetic_stereo_block(SampleRate(48_000), FrameCount(4), 2);
    let graph = ExecutableGraph::new(
        "graph:test",
        vec![
            GraphNodeSpec {
                execution_class: GraphNodeExecutionClass::PureTransform,
                ..routed_node(
                    "pre",
                    "main:in",
                    ChannelLayout::Stereo,
                    "bus:pre",
                    ChannelLayout::Stereo,
                    GraphNodeTopologyRole::TrackLane,
                    vec![
                        GraphStageSpec::Gain { linear: 0.5 },
                        GraphStageSpec::Bias { amount: 0.25 },
                        GraphStageSpec::TanhDrive { drive: 1.5 },
                    ],
                )
            },
            GraphNodeSpec {
                execution_class: GraphNodeExecutionClass::LatencyBearing,
                latency_samples: 24,
                ..routed_node(
                    "post",
                    "bus:pre",
                    ChannelLayout::Stereo,
                    "main:out",
                    ChannelLayout::Stereo,
                    GraphNodeTopologyRole::ConsoleNode,
                    vec![
                        GraphStageSpec::StereoBalance { balance: -0.25 },
                        GraphStageSpec::HardClip { threshold: 0.4 },
                    ],
                )
            },
        ],
    );

    let report = graph.process(&mut buffer);

    assert_eq!(report.graph_id, "graph:test");
    assert_eq!(report.node_count, 2);
    assert_eq!(report.stateful_node_count, 1);
    assert_eq!(report.latency_node_count, 1);
    assert_eq!(report.contract_issue_count, 0);
    assert_eq!(report.silence_clear_node_count, 0);
    assert_eq!(report.adaptive_channel_node_count, 0);
    assert_eq!(report.resettable_node_count, 0);
    assert_eq!(report.scratch_buffer_count, 0);
    assert_eq!(report.direct_edge_count, 1);
    assert_eq!(report.fan_in_bus_count, 0);
    assert_eq!(report.fan_out_bus_count, 0);
    assert_eq!(report.phase_count, 2);
    assert_eq!(report.anticipative_phase_count, 0);
    assert_eq!(report.lane_count, 1);
    assert_eq!(report.anticipative_lane_count, 0);
    assert_eq!(report.lane_order, vec![GraphExecutionLane::Realtime]);
    assert_eq!(report.dispatch_count, 1);
    assert_eq!(report.dispatch_boundary_count, 0);
    assert_eq!(report.dispatch_order, vec![GraphExecutionLane::Realtime]);
    assert_eq!(report.prepared_dispatch_count, 0);
    assert_eq!(report.realtime_dispatch_count, 1);
    assert_eq!(report.dispatch_handoff_count, 0);
    assert_eq!(report.prework_output_peak, None);
    assert!(report.realtime_input_peak.is_some());
    assert_eq!(
        report.phase_order,
        vec![
            GraphNodePlanningGroup::InlineRealtime,
            GraphNodePlanningGroup::StatefulRealtime,
        ]
    );
    assert_eq!(report.stage_count, 5);
    assert_eq!(report.dynamic_kernel_stage_count, 0);
    assert_eq!(
        report.dynamic_stage_state_model,
        GraphDynamicStageStateModel::RebuiltPerBlock
    );
    assert_eq!(report.total_latency_samples, 24);
    assert_eq!(report.max_node_latency_samples, 24);
    assert_eq!(report.total_tail_samples, 0);
    assert_eq!(report.max_node_tail_samples, 0);
    assert_eq!(report.output_latency_samples, 24);
    assert_eq!(report.max_bus_latency_samples, 24);
    assert_eq!(report.frame_count, 4);
    assert_eq!(report.channel_count, 2);
    assert!(report.output_peak <= 0.4);
    assert!(report.output_rms > 0.0);
    assert!(report.first_output_sample.is_some());
}

#[test]
fn stereo_balance_stage_scales_channels_as_expected() {
    let mut buffer = AudioBuffer::from_interleaved(
        SampleRate(48_000),
        ChannelLayout::Stereo,
        vec![1.0, 1.0, 0.5, 0.5],
    );
    let graph = ExecutableGraph::new(
        "graph:stereo-balance",
        vec![test_node(
            "balance",
            GraphNodeExecutionClass::PureTransform,
            0,
            vec![GraphStageSpec::StereoBalance { balance: 0.5 }],
        )],
    );

    let report = graph.process(&mut buffer);

    assert_eq!(buffer.samples(), &[0.5, 1.0, 0.25, 0.5]);
    assert_eq!(report.node_count, 1);
    assert_eq!(report.phase_count, 1);
    assert_eq!(report.lane_count, 1);
    assert_eq!(report.dispatch_count, 1);
    assert_eq!(report.prepared_dispatch_count, 0);
    assert_eq!(report.realtime_dispatch_count, 1);
    assert_eq!(report.dispatch_handoff_count, 0);
    assert_eq!(report.stage_count, 1);
}

#[test]
fn executable_graph_carries_execution_context() {
    let graph = ExecutableGraph::new(
        "graph:context",
        vec![test_node(
            "gain",
            GraphNodeExecutionClass::Stateful,
            0,
            vec![GraphStageSpec::Gain { linear: 0.5 }],
        )],
    );
    let context = GraphExecutionContext {
        processing_epoch: 3,
        block_sequence: 17,
        projection_epoch: 2,
        parameter_epoch: 23,
        configured_block_size: 256,
        anticipative_enabled: true,
        transport_playing: true,
        transport_tempo_bpm: 128.0,
        timeline_position_samples: 512,
    };

    let (_buffer, report) = graph.execute(GraphExecutionRequest {
        context: context.clone(),
        buffer: synthetic_stereo_block(SampleRate(48_000), FrameCount(4), 4),
        parameter_batch: None,
    });

    assert_eq!(report.context, context);
    assert_eq!(report.graph_id, "graph:context");
    assert_eq!(report.node_count, 1);
    assert_eq!(report.stateful_node_count, 1);
    assert_eq!(report.prepared_dispatch_count, 0);
    assert_eq!(report.realtime_dispatch_count, 1);
    assert_eq!(report.dispatch_handoff_count, 0);
}
