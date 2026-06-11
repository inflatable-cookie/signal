use super::{GraphExecutionLane, GraphNodeExecutionClass, GraphNodePlanningGroup};

/// A single node as resolved by the planner, including its scheduling group.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GraphPlannedNode {
    /// Node identifier, matching [`GraphNodeSpec::node_id`].
    pub node_id: String,
    /// Execution class used to derive the planning group.
    pub execution_class: GraphNodeExecutionClass,
    /// Planning group this node was assigned to.
    pub group: GraphNodePlanningGroup,
    /// Latency this node introduces, in samples.
    pub latency_samples: u32,
}

/// A planning phase: a group of nodes that share the same
/// [`GraphNodePlanningGroup`] and can run together within a lane dispatch.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GraphPlannedPhase {
    /// Planning group for all nodes in this phase.
    pub group: GraphNodePlanningGroup,
    /// Ordered list of node IDs in this phase.
    pub node_ids: Vec<String>,
}

/// A dispatch unit: one execution lane with the ordered phases it will run.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GraphLaneDispatch {
    /// Lane on which this dispatch runs.
    pub lane: GraphExecutionLane,
    /// Phase groups to execute within this dispatch, in order.
    pub phase_order: Vec<GraphNodePlanningGroup>,
}

/// Aggregate result of the graph planner: node group counts, phase/lane
/// ordering, and the full dispatch schedule for one block.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct GraphPlanningSummary {
    /// Number of nodes placed in the [`GraphNodePlanningGroup::InlineRealtime`] group.
    pub inline_realtime_node_count: usize,
    /// Number of nodes placed in the [`GraphNodePlanningGroup::StatefulRealtime`] group.
    pub stateful_realtime_node_count: usize,
    /// Number of nodes placed in the [`GraphNodePlanningGroup::AnticipativeEligible`] group.
    pub anticipative_eligible_node_count: usize,
    /// Number of plugin-backed nodes across all groups.
    pub plugin_backed_node_count: usize,
    /// Total number of phases across all dispatches.
    pub phase_count: usize,
    /// Number of phases that run on the anticipative lane.
    pub anticipative_phase_count: usize,
    /// Ordered sequence of planning groups as they appear in the schedule.
    pub phase_order: Vec<GraphNodePlanningGroup>,
    /// Total number of execution lanes in the schedule.
    pub lane_count: usize,
    /// Number of lanes assigned to the anticipative execution path.
    pub anticipative_lane_count: usize,
    /// Ordered sequence of execution lanes in the schedule.
    pub lane_order: Vec<GraphExecutionLane>,
    /// Total number of lane dispatches in the schedule.
    pub dispatch_count: usize,
    /// Number of dispatch boundaries (transitions between realtime and
    /// anticipative lanes).
    pub dispatch_boundary_count: usize,
    /// Full ordered list of lane dispatches to execute this block.
    pub dispatches: Vec<GraphLaneDispatch>,
    /// Full ordered list of planning phases.
    pub phases: Vec<GraphPlannedPhase>,
    /// All planned nodes with their resolved scheduling metadata.
    pub planned_nodes: Vec<GraphPlannedNode>,
}
