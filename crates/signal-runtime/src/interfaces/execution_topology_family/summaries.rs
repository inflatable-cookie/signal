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
    pub(super) fn include_chain(&mut self, chain: &RuntimePluginExecutionChainSummary) {
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
