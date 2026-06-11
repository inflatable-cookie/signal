use super::*;

/// Nodes and topology roles assigned to one execution lane.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeExecutionLaneSummary {
    /// The execution lane this summary describes.
    pub lane: GraphExecutionLane,
    /// Ordered planning groups contained in this lane.
    pub groups: Vec<GraphNodePlanningGroup>,
    /// IDs of all nodes assigned to this lane.
    pub node_ids: Vec<String>,
    /// Topology roles present among this lane's nodes.
    pub topology_roles: Vec<GraphNodeTopologyRole>,
    /// Track lane IDs that have nodes in this execution lane.
    pub track_lane_ids: Vec<String>,
    /// Bus group IDs that have nodes in this execution lane.
    pub bus_group_ids: Vec<String>,
    /// Console group IDs that have nodes in this execution lane.
    pub console_group_ids: Vec<String>,
    /// Send/return IDs that have nodes in this execution lane.
    pub send_return_ids: Vec<String>,
}

/// Aggregated plugin chain summary for a set of routed graph nodes.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RuntimeRoutedPluginChainSummary {
    /// Number of chains included in this summary.
    pub chain_count: usize,
    /// Total number of stages across all included chains.
    pub stage_count: usize,
    /// Number of stages waiting for a first render.
    pub pending_render_stage_count: usize,
    /// Number of stages whose output is still settling.
    pub settling_stage_count: usize,
    /// Number of stages using latency compensation.
    pub compensated_stage_count: usize,
    /// Number of stages in a degraded state.
    pub degraded_stage_count: usize,
    /// Number of stages that are bypassed.
    pub bypassed_stage_count: usize,
    /// Number of stages with a missing plugin binding.
    pub missing_binding_stage_count: usize,
    /// Total planned (declared) latency across all stages in samples.
    pub total_planned_latency_samples: u32,
    /// Total realized (measured) latency across all stages in samples.
    pub total_realized_latency_samples: u32,
    /// Total tail length across all stages in samples.
    pub total_tail_samples: u32,
    /// IDs of all included chains.
    pub chain_ids: Vec<String>,
    /// IDs of all nodes covered by included chains.
    pub node_ids: Vec<String>,
    /// IDs of all sandbox instances present in included chains.
    pub sandbox_ids: Vec<String>,
    /// Full chain summaries for each included chain.
    pub chains: Vec<RuntimePluginExecutionChainSummary>,
}

impl RuntimeRoutedPluginChainSummary {
    fn include_chain(&mut self, chain: &RuntimePluginExecutionChainSummary) {
        if !self.chain_ids.contains(&chain.chain_id) {
            self.chain_count = self.chain_count.saturating_add(1);
            self.chain_ids.push(chain.chain_id.clone());
            self.chains.push(chain.clone());
        }
        self.stage_count = self.stage_count.saturating_add(chain.stage_count);
        self.pending_render_stage_count = self
            .pending_render_stage_count
            .saturating_add(chain.pending_render_stage_count);
        self.settling_stage_count = self
            .settling_stage_count
            .saturating_add(chain.settling_stage_count);
        self.compensated_stage_count = self
            .compensated_stage_count
            .saturating_add(chain.compensated_stage_count);
        self.degraded_stage_count = self
            .degraded_stage_count
            .saturating_add(chain.degraded_stage_count);
        self.bypassed_stage_count = self
            .bypassed_stage_count
            .saturating_add(chain.bypassed_stage_count);
        self.missing_binding_stage_count = self
            .missing_binding_stage_count
            .saturating_add(chain.missing_binding_stage_count);
        self.total_planned_latency_samples = self
            .total_planned_latency_samples
            .saturating_add(chain.total_planned_latency_samples);
        self.total_realized_latency_samples = self
            .total_realized_latency_samples
            .saturating_add(chain.total_realized_latency_samples);
        self.total_tail_samples = self
            .total_tail_samples
            .saturating_add(chain.total_tail_samples);
        for stage in &chain.stages {
            if !self.node_ids.contains(&stage.node_id) {
                self.node_ids.push(stage.node_id.clone());
            }
            if let Some(sandbox_id) = &stage.sandbox_id {
                if !self.sandbox_ids.contains(sandbox_id) {
                    self.sandbox_ids.push(sandbox_id.clone());
                }
            }
        }
    }
}

/// Topology summary for one mixer track lane.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeMixerTrackLaneSummary {
    /// Track lane ID this summary describes.
    pub track_lane_id: String,
    /// IDs of all nodes in this track lane.
    pub node_ids: Vec<String>,
    /// Bus group IDs this track lane routes through.
    pub bus_group_ids: Vec<String>,
    /// Input bus IDs used by this track lane's nodes.
    pub input_bus_ids: Vec<String>,
    /// Output bus IDs used by this track lane's nodes.
    pub output_bus_ids: Vec<String>,
    /// Aggregated plugin chain summary for this track lane.
    pub plugin_chain: RuntimeRoutedPluginChainSummary,
}

/// Topology summary for one mixer bus group.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeMixerBusGroupSummary {
    /// Bus group ID this summary describes.
    pub bus_group_id: String,
    /// Topology roles present in this bus group.
    pub topology_roles: Vec<GraphNodeTopologyRole>,
    /// IDs of all nodes in this bus group.
    pub node_ids: Vec<String>,
    /// Input bus IDs used by this bus group's nodes.
    pub input_bus_ids: Vec<String>,
    /// Output bus IDs used by this bus group's nodes.
    pub output_bus_ids: Vec<String>,
    /// Aggregated plugin chain summary for this bus group.
    pub plugin_chain: RuntimeRoutedPluginChainSummary,
}

/// Topology summary for one mixer console group.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeMixerConsoleGroupSummary {
    /// Console group ID this summary describes.
    pub console_group_id: String,
    /// IDs of all nodes in this console group.
    pub node_ids: Vec<String>,
    /// Input bus IDs used by this console group's nodes.
    pub input_bus_ids: Vec<String>,
    /// Output bus IDs used by this console group's nodes.
    pub output_bus_ids: Vec<String>,
    /// Aggregated plugin chain summary for this console group.
    pub plugin_chain: RuntimeRoutedPluginChainSummary,
}

/// Topology summary for one send/return pair.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeMixerSendReturnSummary {
    /// Send/return ID this summary describes.
    pub send_return_id: String,
    /// IDs of all send nodes in this pair.
    pub send_node_ids: Vec<String>,
    /// IDs of all return nodes in this pair.
    pub return_node_ids: Vec<String>,
    /// Input bus IDs used by this pair's nodes.
    pub input_bus_ids: Vec<String>,
    /// Output bus IDs used by this pair's nodes.
    pub output_bus_ids: Vec<String>,
    /// Aggregated plugin chain summary for this pair.
    pub plugin_chain: RuntimeRoutedPluginChainSummary,
}

/// Per-node summary within the execution topology.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeExecutionNodeSummary {
    /// Unique identifier of this graph node.
    pub node_id: String,
    /// Execution lane this node is assigned to.
    pub lane: GraphExecutionLane,
    /// Planning group this node belongs to within its lane.
    pub group: GraphNodePlanningGroup,
    /// Execution class of this node.
    pub execution_class: GraphNodeExecutionClass,
    /// Topology role of this node.
    pub topology_role: GraphNodeTopologyRole,
    /// Track lane ID this node belongs to, if any.
    pub track_lane_id: Option<String>,
    /// Bus group ID this node belongs to, if any.
    pub bus_group_id: Option<String>,
    /// Console group ID this node belongs to, if any.
    pub console_group_id: Option<String>,
    /// Send/return ID this node belongs to, if any.
    pub send_return_id: Option<String>,
    /// ID of this node's primary input bus.
    pub input_bus_id: String,
    /// ID of this node's primary output bus.
    pub output_bus_id: String,
    /// Sandbox ID of the plugin backing this node, if any.
    pub plugin_sandbox_id: Option<String>,
}

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

struct TopologyNodesResult {
    nodes: Vec<RuntimeExecutionNodeSummary>,
    track_lanes: Vec<RuntimeMixerTrackLaneSummary>,
    bus_groups: Vec<RuntimeMixerBusGroupSummary>,
    console_groups: Vec<RuntimeMixerConsoleGroupSummary>,
    send_returns: Vec<RuntimeMixerSendReturnSummary>,
    utility_node_count: usize,
    track_lane_node_count: usize,
    bus_node_count: usize,
    send_return_node_count: usize,
    console_node_count: usize,
}

fn build_topology_nodes(planned_nodes: &[RuntimePlannedGraphNode]) -> TopologyNodesResult {
    let mut track_lanes_by_id =
        std::collections::BTreeMap::<String, RuntimeMixerTrackLaneSummary>::new();
    let mut bus_groups_by_id =
        std::collections::BTreeMap::<String, RuntimeMixerBusGroupSummary>::new();
    let mut console_groups_by_id =
        std::collections::BTreeMap::<String, RuntimeMixerConsoleGroupSummary>::new();
    let mut send_returns_by_id =
        std::collections::BTreeMap::<String, RuntimeMixerSendReturnSummary>::new();

    let mut nodes = Vec::with_capacity(planned_nodes.len());
    let mut utility_node_count = 0usize;
    let mut track_lane_node_count = 0usize;
    let mut bus_node_count = 0usize;
    let mut send_return_node_count = 0usize;
    let mut console_node_count = 0usize;

    for node in planned_nodes {
        match node.topology_role {
            GraphNodeTopologyRole::Utility => utility_node_count += 1,
            GraphNodeTopologyRole::TrackLane => track_lane_node_count += 1,
            GraphNodeTopologyRole::Bus => bus_node_count += 1,
            GraphNodeTopologyRole::Send | GraphNodeTopologyRole::Return => {
                send_return_node_count += 1;
            }
            GraphNodeTopologyRole::ConsoleNode => console_node_count += 1,
        }
        if let Some(track_lane_id) = &node.track_lane_id {
            let summary = track_lanes_by_id
                .entry(track_lane_id.clone())
                .or_insert_with(|| RuntimeMixerTrackLaneSummary {
                    track_lane_id: track_lane_id.clone(),
                    node_ids: Vec::new(),
                    bus_group_ids: Vec::new(),
                    input_bus_ids: Vec::new(),
                    output_bus_ids: Vec::new(),
                    plugin_chain: RuntimeRoutedPluginChainSummary::default(),
                });
            summary.node_ids.push(node.node_id.clone());
            if let Some(bus_group_id) = &node.bus_group_id {
                if !summary.bus_group_ids.contains(bus_group_id) {
                    summary.bus_group_ids.push(bus_group_id.clone());
                }
            }
            if !summary.input_bus_ids.contains(&node.input_bus_id) {
                summary.input_bus_ids.push(node.input_bus_id.clone());
            }
            if !summary.output_bus_ids.contains(&node.output_bus_id) {
                summary.output_bus_ids.push(node.output_bus_id.clone());
            }
        }
        if let Some(bus_group_id) = &node.bus_group_id {
            let summary = bus_groups_by_id
                .entry(bus_group_id.clone())
                .or_insert_with(|| RuntimeMixerBusGroupSummary {
                    bus_group_id: bus_group_id.clone(),
                    topology_roles: Vec::new(),
                    node_ids: Vec::new(),
                    input_bus_ids: Vec::new(),
                    output_bus_ids: Vec::new(),
                    plugin_chain: RuntimeRoutedPluginChainSummary::default(),
                });
            if !summary.topology_roles.contains(&node.topology_role) {
                summary.topology_roles.push(node.topology_role);
            }
            summary.node_ids.push(node.node_id.clone());
            if !summary.input_bus_ids.contains(&node.input_bus_id) {
                summary.input_bus_ids.push(node.input_bus_id.clone());
            }
            if !summary.output_bus_ids.contains(&node.output_bus_id) {
                summary.output_bus_ids.push(node.output_bus_id.clone());
            }
        }
        if let Some(console_group_id) = &node.console_group_id {
            let summary = console_groups_by_id
                .entry(console_group_id.clone())
                .or_insert_with(|| RuntimeMixerConsoleGroupSummary {
                    console_group_id: console_group_id.clone(),
                    node_ids: Vec::new(),
                    input_bus_ids: Vec::new(),
                    output_bus_ids: Vec::new(),
                    plugin_chain: RuntimeRoutedPluginChainSummary::default(),
                });
            summary.node_ids.push(node.node_id.clone());
            if !summary.input_bus_ids.contains(&node.input_bus_id) {
                summary.input_bus_ids.push(node.input_bus_id.clone());
            }
            if !summary.output_bus_ids.contains(&node.output_bus_id) {
                summary.output_bus_ids.push(node.output_bus_id.clone());
            }
        }
        if let Some(send_return_id) = &node.send_return_id {
            let summary = send_returns_by_id
                .entry(send_return_id.clone())
                .or_insert_with(|| RuntimeMixerSendReturnSummary {
                    send_return_id: send_return_id.clone(),
                    send_node_ids: Vec::new(),
                    return_node_ids: Vec::new(),
                    input_bus_ids: Vec::new(),
                    output_bus_ids: Vec::new(),
                    plugin_chain: RuntimeRoutedPluginChainSummary::default(),
                });
            match node.topology_role {
                GraphNodeTopologyRole::Send => summary.send_node_ids.push(node.node_id.clone()),
                GraphNodeTopologyRole::Return => {
                    summary.return_node_ids.push(node.node_id.clone());
                }
                _ => {}
            }
            if !summary.input_bus_ids.contains(&node.input_bus_id) {
                summary.input_bus_ids.push(node.input_bus_id.clone());
            }
            if !summary.output_bus_ids.contains(&node.output_bus_id) {
                summary.output_bus_ids.push(node.output_bus_id.clone());
            }
        }
        nodes.push(RuntimeExecutionNodeSummary {
            node_id: node.node_id.clone(),
            lane: runtime_lane_for_group(node.group),
            group: node.group,
            execution_class: node.execution_class,
            topology_role: node.topology_role,
            track_lane_id: node.track_lane_id.clone(),
            bus_group_id: node.bus_group_id.clone(),
            console_group_id: node.console_group_id.clone(),
            send_return_id: node.send_return_id.clone(),
            input_bus_id: node.input_bus_id.clone(),
            output_bus_id: node.output_bus_id.clone(),
            plugin_sandbox_id: node.plugin_sandbox_id.clone(),
        });
    }

    TopologyNodesResult {
        nodes,
        track_lanes: track_lanes_by_id.into_values().collect(),
        bus_groups: bus_groups_by_id.into_values().collect(),
        console_groups: console_groups_by_id.into_values().collect(),
        send_returns: send_returns_by_id.into_values().collect(),
        utility_node_count,
        track_lane_node_count,
        bus_node_count,
        send_return_node_count,
        console_node_count,
    }
}
