use super::*;

/// Builds per-lane instrumentation summaries from the planned nodes in a block snapshot.
pub fn runtime_worker_lane_instrumentation_summaries(
    engine_block_snapshot: &RuntimeEngineBlockSnapshot,
) -> Vec<RuntimeWorkerLaneInstrumentationSummary> {
    let mut lane_order = engine_block_snapshot.lane_order.clone();
    for node in &engine_block_snapshot.planned_nodes {
        let lane = runtime_lane_for_group(node.group);
        if !lane_order.contains(&lane) {
            lane_order.push(lane);
        }
    }

    let mut summaries = Vec::new();
    for lane in lane_order {
        let mut node_count = 0usize;
        let mut plugin_backed_node_count = 0usize;
        let mut planning_groups = Vec::new();
        let mut total_latency_samples = 0u32;
        let mut max_node_latency_samples = 0u32;

        for node in engine_block_snapshot
            .planned_nodes
            .iter()
            .filter(|node| runtime_lane_for_group(node.group) == lane)
        {
            node_count = node_count.saturating_add(1);
            if matches!(node.execution_class, GraphNodeExecutionClass::PluginBacked) {
                plugin_backed_node_count = plugin_backed_node_count.saturating_add(1);
            }
            if !planning_groups.contains(&node.group) {
                planning_groups.push(node.group);
            }
            total_latency_samples = total_latency_samples.saturating_add(node.latency_samples);
            max_node_latency_samples = max_node_latency_samples.max(node.latency_samples);
        }

        if node_count > 0 {
            summaries.push(RuntimeWorkerLaneInstrumentationSummary {
                lane,
                node_count,
                plugin_backed_node_count,
                planning_group_count: planning_groups.len(),
                total_latency_samples,
                max_node_latency_samples,
            });
        }
    }

    summaries
}

/// Returns the display name string for a graph node planning group.
pub fn runtime_graph_node_planning_group_name(group: GraphNodePlanningGroup) -> &'static str {
    match group {
        GraphNodePlanningGroup::InlineRealtime => "InlineRealtime",
        GraphNodePlanningGroup::StatefulRealtime => "StatefulRealtime",
        GraphNodePlanningGroup::AnticipativeEligible => "AnticipativeEligible",
    }
}

/// Returns the display name string for a graph node topology role.
pub fn runtime_graph_node_topology_role_name(role: GraphNodeTopologyRole) -> &'static str {
    match role {
        GraphNodeTopologyRole::Utility => "Utility",
        GraphNodeTopologyRole::TrackLane => "TrackLane",
        GraphNodeTopologyRole::Bus => "Bus",
        GraphNodeTopologyRole::Send => "Send",
        GraphNodeTopologyRole::Return => "Return",
        GraphNodeTopologyRole::ConsoleNode => "ConsoleNode",
    }
}

pub(crate) fn runtime_execution_lane_name(lane: GraphExecutionLane) -> &'static str {
    match lane {
        GraphExecutionLane::Realtime => "Realtime",
        GraphExecutionLane::Anticipative => "Anticipative",
    }
}

/// Returns the display name string for a prework backlog class.
pub fn runtime_prework_backlog_class_name(value: RuntimePreworkBacklogClass) -> &'static str {
    match value {
        RuntimePreworkBacklogClass::Immediate => "Immediate",
        RuntimePreworkBacklogClass::NearTerm => "NearTerm",
        RuntimePreworkBacklogClass::Deferred => "Deferred",
    }
}

impl RuntimeObservationReport {
    /// Captures a performance snapshot from this observation report.
    pub fn performance_snapshot(&self) -> RuntimePerformanceSnapshot {
        RuntimePerformanceSnapshot::capture(
            &self.effective_config,
            &self.diagnostics_snapshot,
            &self.engine_block_snapshot,
            self.last_deferred_service_receipt.as_ref(),
        )
    }
}

impl RuntimeSupervisorReport {
    /// Captures a performance snapshot from the observation within this supervisor report.
    pub fn performance_snapshot(&self) -> RuntimePerformanceSnapshot {
        self.observation.performance_snapshot()
    }
}
