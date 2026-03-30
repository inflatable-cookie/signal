use super::super::*;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeExecutionLaneSummary {
    pub lane: GraphExecutionLane,
    pub groups: Vec<GraphNodePlanningGroup>,
    pub node_ids: Vec<String>,
    pub topology_roles: Vec<GraphNodeTopologyRole>,
    pub track_lane_ids: Vec<String>,
    pub bus_group_ids: Vec<String>,
    pub console_group_ids: Vec<String>,
    pub send_return_ids: Vec<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RuntimeRoutedPluginChainSummary {
    pub chain_count: usize,
    pub stage_count: usize,
    pub pending_render_stage_count: usize,
    pub settling_stage_count: usize,
    pub compensated_stage_count: usize,
    pub degraded_stage_count: usize,
    pub bypassed_stage_count: usize,
    pub missing_binding_stage_count: usize,
    pub total_planned_latency_samples: u32,
    pub total_realized_latency_samples: u32,
    pub total_tail_samples: u32,
    pub chain_ids: Vec<String>,
    pub node_ids: Vec<String>,
    pub sandbox_ids: Vec<String>,
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

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeMixerTrackLaneSummary {
    pub track_lane_id: String,
    pub node_ids: Vec<String>,
    pub bus_group_ids: Vec<String>,
    pub input_bus_ids: Vec<String>,
    pub output_bus_ids: Vec<String>,
    pub plugin_chain: RuntimeRoutedPluginChainSummary,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeMixerBusGroupSummary {
    pub bus_group_id: String,
    pub topology_roles: Vec<GraphNodeTopologyRole>,
    pub node_ids: Vec<String>,
    pub input_bus_ids: Vec<String>,
    pub output_bus_ids: Vec<String>,
    pub plugin_chain: RuntimeRoutedPluginChainSummary,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeMixerConsoleGroupSummary {
    pub console_group_id: String,
    pub node_ids: Vec<String>,
    pub input_bus_ids: Vec<String>,
    pub output_bus_ids: Vec<String>,
    pub plugin_chain: RuntimeRoutedPluginChainSummary,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeMixerSendReturnSummary {
    pub send_return_id: String,
    pub send_node_ids: Vec<String>,
    pub return_node_ids: Vec<String>,
    pub input_bus_ids: Vec<String>,
    pub output_bus_ids: Vec<String>,
    pub plugin_chain: RuntimeRoutedPluginChainSummary,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeExecutionNodeSummary {
    pub node_id: String,
    pub lane: GraphExecutionLane,
    pub group: GraphNodePlanningGroup,
    pub execution_class: GraphNodeExecutionClass,
    pub topology_role: GraphNodeTopologyRole,
    pub track_lane_id: Option<String>,
    pub bus_group_id: Option<String>,
    pub console_group_id: Option<String>,
    pub send_return_id: Option<String>,
    pub input_bus_id: String,
    pub output_bus_id: String,
    pub input_channels: ChannelLayout,
    pub output_channels: ChannelLayout,
    pub input_layout: RuntimeMultichannelLayoutSummary,
    pub output_layout: RuntimeMultichannelLayoutSummary,
    pub input_bus_intent: RuntimeBusIntent,
    pub output_bus_intent: RuntimeBusIntent,
    pub secondary_input: Option<RuntimeSecondaryInputRouteSummary>,
    pub spatial_execution: Option<RuntimeSpatialExecutionSummary>,
    pub plugin_sandbox_id: Option<String>,
    pub plugin_recall_state: Option<RuntimePluginRecallState>,
    pub plugin_recall: Option<RuntimePluginRecallSnapshot>,
    pub plugin_compensation_state: Option<RuntimePluginCompensationState>,
    pub plugin_realized_latency_samples: Option<u32>,
    pub plugin_tail_samples: Option<u32>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RuntimeExecutionTopologySummary {
    pub node_count: usize,
    pub utility_node_count: usize,
    pub track_lane_node_count: usize,
    pub bus_node_count: usize,
    pub send_return_node_count: usize,
    pub console_node_count: usize,
    pub lane_count: usize,
    pub track_lane_group_count: usize,
    pub bus_group_count: usize,
    pub send_return_group_count: usize,
    pub console_group_count: usize,
    pub secondary_input_count: usize,
    pub required_secondary_input_count: usize,
    pub optional_secondary_input_count: usize,
    pub disabled_secondary_input_count: usize,
    pub terminal_fallback_secondary_input_count: usize,
    pub bus_connection_count: usize,
    pub auxiliary_path_count: usize,
    pub spatial_node_count: usize,
    pub active_spatial_node_count: usize,
    pub bypassed_spatial_node_count: usize,
    pub fallback_spatial_node_count: usize,
    pub surround_bed_spatial_node_count: usize,
    pub object_aware_spatial_node_count: usize,
    pub expanded_fallback_spatial_node_count: usize,
    pub immersive_spatial_node_count: usize,
    pub room_policy_aware_spatial_node_count: usize,
    pub fallback_room_policy_spatial_node_count: usize,
    pub deployment_spatial_node_count: usize,
    pub folded_down_spatial_node_count: usize,
    pub fallback_monitoring_scene_spatial_node_count: usize,
    pub renderer_capability_spatial_node_count: usize,
    pub negotiated_renderer_spatial_node_count: usize,
    pub immersive_export_spatial_node_count: usize,
    pub fallback_immersive_export_spatial_node_count: usize,
    pub lanes: Vec<RuntimeExecutionLaneSummary>,
    pub track_lanes: Vec<RuntimeMixerTrackLaneSummary>,
    pub bus_groups: Vec<RuntimeMixerBusGroupSummary>,
    pub console_groups: Vec<RuntimeMixerConsoleGroupSummary>,
    pub send_returns: Vec<RuntimeMixerSendReturnSummary>,
    pub secondary_inputs: Vec<RuntimeSecondaryInputRouteSummary>,
    pub bus_connections: Vec<RuntimeBusConnectionSummary>,
    pub auxiliary_paths: Vec<RuntimeAuxiliaryPathSummary>,
    pub nodes: Vec<RuntimeExecutionNodeSummary>,
    pub plugin_chain: RuntimeRoutedPluginChainSummary,
}
