use super::super::*;

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
    /// Number of stages whose output is still settling (e.g. after a parameter change).
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
    pub track_lane_id: String,
    pub node_ids: Vec<String>,
    pub bus_group_ids: Vec<String>,
    pub input_bus_ids: Vec<String>,
    pub output_bus_ids: Vec<String>,
    pub plugin_chain: RuntimeRoutedPluginChainSummary,
}

/// Topology summary for one mixer bus group.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeMixerBusGroupSummary {
    pub bus_group_id: String,
    pub topology_roles: Vec<GraphNodeTopologyRole>,
    pub node_ids: Vec<String>,
    pub input_bus_ids: Vec<String>,
    pub output_bus_ids: Vec<String>,
    pub plugin_chain: RuntimeRoutedPluginChainSummary,
}

/// Topology summary for one mixer console group.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeMixerConsoleGroupSummary {
    pub console_group_id: String,
    pub node_ids: Vec<String>,
    pub input_bus_ids: Vec<String>,
    pub output_bus_ids: Vec<String>,
    pub plugin_chain: RuntimeRoutedPluginChainSummary,
}

/// Topology summary for one send/return pair.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeMixerSendReturnSummary {
    pub send_return_id: String,
    pub send_node_ids: Vec<String>,
    pub return_node_ids: Vec<String>,
    pub input_bus_ids: Vec<String>,
    pub output_bus_ids: Vec<String>,
    pub plugin_chain: RuntimeRoutedPluginChainSummary,
}

/// Per-node summary within the execution topology, including routing,
/// plugin state, and latency.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeExecutionNodeSummary {
    /// Unique identifier of this graph node.
    pub node_id: String,
    /// Execution lane this node is assigned to.
    pub lane: GraphExecutionLane,
    /// Planning group this node belongs to within its lane.
    pub group: GraphNodePlanningGroup,
    /// Execution class (realtime, anticipative, etc.) of this node.
    pub execution_class: GraphNodeExecutionClass,
    /// Topology role (track lane, bus group, console group, etc.) of this node.
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
    /// Channel layout of the input bus.
    pub input_channels: ChannelLayout,
    /// Channel layout of the output bus.
    pub output_channels: ChannelLayout,
    /// Detailed multichannel layout summary for the input bus.
    pub input_layout: RuntimeMultichannelLayoutSummary,
    /// Detailed multichannel layout summary for the output bus.
    pub output_layout: RuntimeMultichannelLayoutSummary,
    /// Bus intent (main program, aux, sidechain, etc.) for the input bus.
    pub input_bus_intent: RuntimeBusIntent,
    /// Bus intent for the output bus.
    pub output_bus_intent: RuntimeBusIntent,
    /// Resolved secondary (sidechain/aux) input connection, if any.
    pub secondary_input: Option<RuntimeSecondaryInputRouteSummary>,
    /// Spatial execution summary for spatialisation nodes, if applicable.
    pub spatial_execution: Option<RuntimeSpatialExecutionSummary>,
    /// Sandbox ID of the plugin backing this node, if any.
    pub plugin_sandbox_id: Option<String>,
    /// Recall state of the plugin at this node, if known.
    pub plugin_recall_state: Option<RuntimePluginRecallState>,
    /// Full recall snapshot for the plugin at this node, if known.
    pub plugin_recall: Option<RuntimePluginRecallSnapshot>,
    /// Latency compensation state of the plugin at this node, if applicable.
    pub plugin_compensation_state: Option<RuntimePluginCompensationState>,
    /// Realized (measured) latency introduced by the plugin in samples, if known.
    pub plugin_realized_latency_samples: Option<u32>,
    /// Plugin tail length in samples, if known.
    pub plugin_tail_samples: Option<u32>,
}

/// Full execution topology summary: lanes, track lanes, bus groups, console
/// groups, send/returns, secondary inputs, bus connections, and spatial nodes.
///
/// Obtained from `RuntimeObservationApi::get_execution_topology_summary()`.
/// Used for host-side topology inspection and test assertions.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RuntimeExecutionTopologySummary {
    /// Total number of nodes in the topology.
    pub node_count: usize,
    /// Number of utility (non-audio-producing) nodes.
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
    /// Total number of secondary (sidechain/aux) input connections.
    pub secondary_input_count: usize,
    /// Number of secondary inputs with a `Required` attachment policy.
    pub required_secondary_input_count: usize,
    /// Number of secondary inputs with an `Optional` attachment policy.
    pub optional_secondary_input_count: usize,
    /// Number of secondary inputs with a `Disabled` attachment policy.
    pub disabled_secondary_input_count: usize,
    /// Number of secondary inputs whose fallback outcome is terminal.
    pub terminal_fallback_secondary_input_count: usize,
    /// Total number of bus connection edges.
    pub bus_connection_count: usize,
    /// Total number of auxiliary routing paths.
    pub auxiliary_path_count: usize,
    /// Total number of spatial processing nodes.
    pub spatial_node_count: usize,
    /// Number of active (non-bypassed) spatial nodes.
    pub active_spatial_node_count: usize,
    /// Number of bypassed spatial nodes.
    pub bypassed_spatial_node_count: usize,
    /// Number of spatial nodes operating in fallback mode.
    pub fallback_spatial_node_count: usize,
    /// Number of surround-bed spatial nodes.
    pub surround_bed_spatial_node_count: usize,
    /// Number of object-aware spatial nodes.
    pub object_aware_spatial_node_count: usize,
    /// Number of spatial nodes using an expanded fallback path.
    pub expanded_fallback_spatial_node_count: usize,
    /// Number of immersive (3D) spatial nodes.
    pub immersive_spatial_node_count: usize,
    /// Number of spatial nodes that respond to room policy.
    pub room_policy_aware_spatial_node_count: usize,
    /// Number of spatial nodes falling back to a room policy.
    pub fallback_room_policy_spatial_node_count: usize,
    /// Number of deployment-configured spatial nodes.
    pub deployment_spatial_node_count: usize,
    /// Number of spatial nodes that fold down to a lower channel count.
    pub folded_down_spatial_node_count: usize,
    /// Number of spatial nodes falling back to a monitoring scene.
    pub fallback_monitoring_scene_spatial_node_count: usize,
    /// Number of spatial nodes with renderer capability negotiation.
    pub renderer_capability_spatial_node_count: usize,
    /// Number of spatial nodes with a negotiated renderer.
    pub negotiated_renderer_spatial_node_count: usize,
    /// Number of spatial nodes performing immersive export.
    pub immersive_export_spatial_node_count: usize,
    /// Number of spatial nodes falling back in immersive export.
    pub fallback_immersive_export_spatial_node_count: usize,
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
    /// Resolved secondary input connections.
    pub secondary_inputs: Vec<RuntimeSecondaryInputRouteSummary>,
    /// Bus connection edge summaries.
    pub bus_connections: Vec<RuntimeBusConnectionSummary>,
    /// Auxiliary routing path summaries.
    pub auxiliary_paths: Vec<RuntimeAuxiliaryPathSummary>,
    /// Per-node topology summaries.
    pub nodes: Vec<RuntimeExecutionNodeSummary>,
    /// Aggregated plugin chain summary for the whole topology.
    pub plugin_chain: RuntimeRoutedPluginChainSummary,
}
