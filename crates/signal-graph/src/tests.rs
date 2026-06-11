use super::{
    ExecutableGraph, GraphExecutionLane, GraphNodeBufferContract, GraphNodeBusEndpoint,
    GraphNodeExecutionClass, GraphNodePlanningGroup, GraphNodeSpec, GraphNodeTopologyMetadata,
    GraphNodeTopologyRole, GraphStageSpec,
};
use signal_primitives::ChannelLayout;

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

fn demo_graph() -> ExecutableGraph {
    ExecutableGraph::new(
        "plan-demo",
        vec![
            test_node(
                "lookahead",
                GraphNodeExecutionClass::LatencyBearing,
                24,
                vec![GraphStageSpec::Gain { linear: 0.5 }],
            ),
            test_node(
                "drive",
                GraphNodeExecutionClass::Stateful,
                0,
                vec![GraphStageSpec::TanhDrive { drive: 1.2 }],
            ),
            test_node(
                "trim",
                GraphNodeExecutionClass::PureTransform,
                0,
                vec![GraphStageSpec::Gain { linear: 0.9 }],
            ),
        ],
    )
}

#[test]
fn planning_summary_assigns_groups_and_lanes() {
    let graph = demo_graph();
    let planning = graph.planning_summary(true);

    assert_eq!(planning.planned_nodes.len(), 3);
    assert_eq!(planning.anticipative_eligible_node_count, 1);
    assert_eq!(planning.stateful_realtime_node_count, 1);
    assert_eq!(planning.inline_realtime_node_count, 1);
    assert_eq!(
        planning.lane_order,
        vec![
            GraphExecutionLane::Anticipative,
            GraphExecutionLane::Realtime
        ]
    );
    let lookahead = planning
        .planned_nodes
        .iter()
        .find(|node| node.node_id == "lookahead")
        .expect("lookahead node planned");
    assert_eq!(
        lookahead.group,
        GraphNodePlanningGroup::AnticipativeEligible
    );
    assert_eq!(lookahead.latency_samples, 24);
}

#[test]
fn planning_summary_without_anticipative_lane_collapses_to_realtime() {
    let graph = demo_graph();
    let planning = graph.planning_summary(false);

    assert_eq!(planning.anticipative_eligible_node_count, 0);
    assert_eq!(planning.lane_order, vec![GraphExecutionLane::Realtime]);
}

#[test]
fn graph_metrics_report_declared_latency_and_counts() {
    let graph = demo_graph();
    assert_eq!(graph.node_count(), 3);
    assert_eq!(graph.stage_count(), 3);
    assert_eq!(graph.total_latency_samples(), 24);
    assert_eq!(graph.max_node_latency_samples(), 24);
    assert_eq!(graph.stateful_node_count(), 2);
    assert_eq!(graph.latency_node_count(), 1);
}

#[test]
fn contract_summary_reports_routing_roles_and_issues() {
    let mut node = test_node(
        "track",
        GraphNodeExecutionClass::Stateful,
        0,
        vec![GraphStageSpec::Gain { linear: 1.0 }],
    );
    node.buffer_contract = GraphNodeBufferContract {
        input: GraphNodeBusEndpoint::new("main:in", ChannelLayout::Stereo),
        output: GraphNodeBusEndpoint::new("bus:track", ChannelLayout::Stereo),
        ..GraphNodeBufferContract::default()
    };
    node.topology = GraphNodeTopologyMetadata {
        role: Some(GraphNodeTopologyRole::TrackLane),
        track_lane_id: Some("track:1".into()),
        bus_group_id: None,
        console_group_id: None,
        send_return_id: None,
    };
    let graph = ExecutableGraph::new("plan-contract", vec![node]);

    let summary = graph.contract_summary();
    assert_eq!(summary.node_contracts.len(), 1);
    assert_eq!(summary.track_lane_node_count, 1);
    assert!(summary.issues.is_empty());
    let contract = &summary.node_contracts[0];
    assert_eq!(contract.input_bus_id, "main:in");
    assert_eq!(contract.output_bus_id, "bus:track");
    assert_eq!(contract.topology_role, GraphNodeTopologyRole::TrackLane);
}
