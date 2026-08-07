use super::build::build_topology_nodes;
use super::*;

/// Execution topology summary derived from the applied graph plan: lanes,
/// track lanes, bus groups, console groups, send/returns, and plugin chains.
///
/// Obtained from `RuntimeObservationApi::get_execution_topology_summary()`.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RuntimeExecutionTopologySummary {
    /// Total number of nodes in the topology.
    pub node_count: usize,
    /// Number of utility (non-mixer) nodes.
    pub utility_node_count: usize,
    /// Number of nodes assigned to track lanes.
    pub track_lane_node_count: usize,
    /// Number of nodes assigned to bus groups.
    pub bus_node_count: usize,
    /// Number of nodes assigned to send/return pairs.
    pub send_return_node_count: usize,
    /// Number of nodes assigned to console groups.
    pub console_node_count: usize,
    /// Total number of execution lanes.
    pub lane_count: usize,
    /// Number of distinct track lane groups.
    pub track_lane_group_count: usize,
    /// Number of distinct bus groups.
    pub bus_group_count: usize,
    /// Number of distinct send/return groups.
    pub send_return_group_count: usize,
    /// Number of distinct console groups.
    pub console_group_count: usize,
    /// Summaries for each execution lane.
    pub lanes: Vec<RuntimeExecutionLaneSummary>,
    /// Summaries for each mixer track lane.
    pub track_lanes: Vec<RuntimeMixerTrackLaneSummary>,
    /// Summaries for each mixer bus group.
    pub bus_groups: Vec<RuntimeMixerBusGroupSummary>,
    /// Summaries for each mixer console group.
    pub console_groups: Vec<RuntimeMixerConsoleGroupSummary>,
    /// Summaries for each send/return pair.
    pub send_returns: Vec<RuntimeMixerSendReturnSummary>,
    /// Per-node topology summaries.
    pub nodes: Vec<RuntimeExecutionNodeSummary>,
    /// Aggregated plugin chain summary for the whole topology.
    pub plugin_chain: RuntimeRoutedPluginChainSummary,
}

impl RuntimeExecutionTopologySummary {
    /// Builds a topology summary from the applied execution plan.
    pub(crate) fn from_plan(
        lane_order: &[GraphExecutionLane],
        planned_nodes: &[RuntimePlannedGraphNode],
    ) -> Self {
        let mut track_lane_ids = std::collections::BTreeSet::new();
        let mut bus_group_id_set = std::collections::BTreeSet::new();
        let mut send_return_id_set = std::collections::BTreeSet::new();
        let mut console_group_id_set = std::collections::BTreeSet::new();
        let mut lanes = Vec::new();

        for lane in lane_order {
            let mut groups = Vec::new();
            let mut node_ids = Vec::new();
            let mut topology_roles = Vec::new();
            let mut lane_ids = Vec::new();
            let mut bus_groups = Vec::new();
            let mut console_groups = Vec::new();
            let mut send_returns = Vec::new();

            for node in planned_nodes
                .iter()
                .filter(|node| runtime_lane_for_group(node.group) == *lane)
            {
                if !groups.contains(&node.group) {
                    groups.push(node.group);
                }
                node_ids.push(node.node_id.clone());
                if !topology_roles.contains(&node.topology_role) {
                    topology_roles.push(node.topology_role);
                }
                if let Some(track_lane_id) = &node.track_lane_id {
                    if !lane_ids.contains(track_lane_id) {
                        lane_ids.push(track_lane_id.clone());
                    }
                    track_lane_ids.insert(track_lane_id.clone());
                }
                if let Some(bus_group_id) = &node.bus_group_id {
                    if !bus_groups.contains(bus_group_id) {
                        bus_groups.push(bus_group_id.clone());
                    }
                    bus_group_id_set.insert(bus_group_id.clone());
                }
                if let Some(console_group_id) = &node.console_group_id {
                    if !console_groups.contains(console_group_id) {
                        console_groups.push(console_group_id.clone());
                    }
                    console_group_id_set.insert(console_group_id.clone());
                }
                if let Some(send_return_id) = &node.send_return_id {
                    if !send_returns.contains(send_return_id) {
                        send_returns.push(send_return_id.clone());
                    }
                    send_return_id_set.insert(send_return_id.clone());
                }
            }

            lanes.push(RuntimeExecutionLaneSummary {
                lane: *lane,
                groups,
                node_ids,
                topology_roles,
                track_lane_ids: lane_ids,
                bus_group_ids: bus_groups,
                console_group_ids: console_groups,
                send_return_ids: send_returns,
            });
        }

        let nodes_result = build_topology_nodes(planned_nodes);

        Self {
            node_count: planned_nodes.len(),
            utility_node_count: nodes_result.utility_node_count,
            track_lane_node_count: nodes_result.track_lane_node_count,
            bus_node_count: nodes_result.bus_node_count,
            send_return_node_count: nodes_result.send_return_node_count,
            console_node_count: nodes_result.console_node_count,
            lane_count: lanes.len(),
            track_lane_group_count: track_lane_ids.len(),
            bus_group_count: bus_group_id_set.len(),
            send_return_group_count: send_return_id_set.len(),
            console_group_count: console_group_id_set.len(),
            lanes,
            track_lanes: nodes_result.track_lanes,
            bus_groups: nodes_result.bus_groups,
            console_groups: nodes_result.console_groups,
            send_returns: nodes_result.send_returns,
            nodes: nodes_result.nodes,
            plugin_chain: RuntimeRoutedPluginChainSummary::default(),
        }
    }

    /// Enriches the topology summary with plugin chain data.
    pub(crate) fn with_plugin_chain_snapshot(
        mut self,
        snapshot: &RuntimePluginChainSnapshot,
    ) -> Self {
        for chain in &snapshot.chains {
            self.plugin_chain.include_chain(chain);
            if let Some(track_lane_id) = chain.track_lane_id.as_deref() {
                if let Some(summary) = self
                    .track_lanes
                    .iter_mut()
                    .find(|summary| summary.track_lane_id == track_lane_id)
                {
                    summary.plugin_chain.include_chain(chain);
                }
            }
            if let Some(bus_group_id) = chain.bus_group_id.as_deref() {
                if let Some(summary) = self
                    .bus_groups
                    .iter_mut()
                    .find(|summary| summary.bus_group_id == bus_group_id)
                {
                    summary.plugin_chain.include_chain(chain);
                }
            }
            if let Some(console_group_id) = chain.console_group_id.as_deref() {
                if let Some(summary) = self
                    .console_groups
                    .iter_mut()
                    .find(|summary| summary.console_group_id == console_group_id)
                {
                    summary.plugin_chain.include_chain(chain);
                }
            }
            if let Some(send_return_id) = chain.send_return_id.as_deref() {
                if let Some(summary) = self
                    .send_returns
                    .iter_mut()
                    .find(|summary| summary.send_return_id == send_return_id)
                {
                    summary.plugin_chain.include_chain(chain);
                }
            }
        }
        self
    }
}
