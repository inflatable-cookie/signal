use super::*;

impl ExecutableGraph {
    /// Build the execution schedule for the current plan.
    ///
    /// Assigns each node to a planning group, forms phases, and produces an
    /// ordered list of lane dispatches. Pass `anticipative_enabled: true` to
    /// allow anticipative-eligible nodes to be placed on the anticipative lane;
    /// when `false` they fall back to the stateful-realtime lane.
    pub fn planning_summary(&self, anticipative_enabled: bool) -> GraphPlanningSummary {
        let planned_nodes = self
            .plan
            .nodes
            .iter()
            .map(|node| GraphPlannedNode {
                node_id: node.node_id.clone(),
                execution_class: node.execution_class,
                group: planning_group_for_node(node, anticipative_enabled),
                latency_samples: node.latency_samples,
            })
            .collect::<Vec<_>>();
        let phase_order = planning_phase_order(&planned_nodes);
        let phases = phase_order
            .iter()
            .copied()
            .map(|group| GraphPlannedPhase {
                group,
                node_ids: planned_nodes
                    .iter()
                    .filter(|node| node.group == group)
                    .map(|node| node.node_id.clone())
                    .collect(),
            })
            .collect::<Vec<_>>();
        let lane_order = planning_lane_order(&planned_nodes);
        let dispatches = lane_order
            .iter()
            .copied()
            .map(|lane| GraphLaneDispatch {
                lane,
                phase_order: phase_order
                    .iter()
                    .copied()
                    .filter(|group| planning_lane_for_group(*group) == lane)
                    .collect(),
            })
            .collect::<Vec<_>>();

        GraphPlanningSummary {
            inline_realtime_node_count: planned_nodes
                .iter()
                .filter(|node| node.group == GraphNodePlanningGroup::InlineRealtime)
                .count(),
            stateful_realtime_node_count: planned_nodes
                .iter()
                .filter(|node| node.group == GraphNodePlanningGroup::StatefulRealtime)
                .count(),
            anticipative_eligible_node_count: planned_nodes
                .iter()
                .filter(|node| node.group == GraphNodePlanningGroup::AnticipativeEligible)
                .count(),
            plugin_backed_node_count: planned_nodes
                .iter()
                .filter(|node| node.execution_class == GraphNodeExecutionClass::PluginBacked)
                .count(),
            phase_count: phase_order.len(),
            anticipative_phase_count: phase_order
                .iter()
                .filter(|group| **group == GraphNodePlanningGroup::AnticipativeEligible)
                .count(),
            phase_order,
            lane_count: lane_order.len(),
            anticipative_lane_count: lane_order
                .iter()
                .filter(|lane| **lane == GraphExecutionLane::Anticipative)
                .count(),
            lane_order,
            dispatch_count: dispatches.len(),
            dispatch_boundary_count: dispatches.len().saturating_sub(1),
            dispatches,
            phases,
            planned_nodes,
        }
    }
}

pub(crate) fn planning_group_for_node(
    node: &GraphNodeSpec,
    anticipative_enabled: bool,
) -> GraphNodePlanningGroup {
    match node.execution_class {
        GraphNodeExecutionClass::PureTransform => GraphNodePlanningGroup::InlineRealtime,
        GraphNodeExecutionClass::Stateful | GraphNodeExecutionClass::PluginBacked => {
            GraphNodePlanningGroup::StatefulRealtime
        }
        GraphNodeExecutionClass::LatencyBearing if anticipative_enabled => {
            GraphNodePlanningGroup::AnticipativeEligible
        }
        GraphNodeExecutionClass::LatencyBearing => GraphNodePlanningGroup::StatefulRealtime,
    }
}

pub(crate) fn planning_phase_order(nodes: &[GraphPlannedNode]) -> Vec<GraphNodePlanningGroup> {
    [
        GraphNodePlanningGroup::InlineRealtime,
        GraphNodePlanningGroup::StatefulRealtime,
        GraphNodePlanningGroup::AnticipativeEligible,
    ]
    .into_iter()
    .filter(|group| nodes.iter().any(|node| node.group == *group))
    .collect()
}

pub(crate) fn planning_lane_order(nodes: &[GraphPlannedNode]) -> Vec<GraphExecutionLane> {
    [
        GraphExecutionLane::Anticipative,
        GraphExecutionLane::Realtime,
    ]
    .into_iter()
    .filter(|lane| {
        nodes
            .iter()
            .any(|node| planning_lane_for_group(node.group) == *lane)
    })
    .collect()
}

pub(crate) fn planning_lane_for_group(group: GraphNodePlanningGroup) -> GraphExecutionLane {
    match group {
        GraphNodePlanningGroup::AnticipativeEligible => GraphExecutionLane::Anticipative,
        GraphNodePlanningGroup::InlineRealtime | GraphNodePlanningGroup::StatefulRealtime => {
            GraphExecutionLane::Realtime
        }
    }
}
