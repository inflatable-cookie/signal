use super::*;

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

#[derive(Clone)]
struct RuntimePlannedGraphNodeTopologyEndpoint<'a> {
    node_id: &'a str,
    topology_role: GraphNodeTopologyRole,
    input_bus_id: &'a str,
    output_bus_id: &'a str,
    input_bus_intent: RuntimeBusIntent,
    output_bus_intent: RuntimeBusIntent,
    bus_group_id: Option<&'a str>,
    send_return_id: Option<&'a str>,
}

fn runtime_auxiliary_path_for_connection(
    source: &RuntimePlannedGraphNodeTopologyEndpoint<'_>,
    target: &RuntimePlannedGraphNodeTopologyEndpoint<'_>,
) -> Option<(
    RuntimeAuxiliaryPathKind,
    String,
    RuntimeBusRole,
    RuntimeBusIntent,
)> {
    if let Some(send_return_id) = source.send_return_id.or(target.send_return_id) {
        return Some((
            RuntimeAuxiliaryPathKind::SendReturn,
            format!("send_return:{send_return_id}"),
            RuntimeBusRole::AuxSend,
            RuntimeBusIntent::AuxSend,
        ));
    }
    if let Some(bus_group_id) = source.bus_group_id.or(target.bus_group_id) {
        return Some((
            RuntimeAuxiliaryPathKind::Submix,
            format!("bus_group:{bus_group_id}"),
            RuntimeBusRole::Submix,
            RuntimeBusIntent::MainProgram,
        ));
    }
    let source_role = runtime_bus_role_for_endpoint(source.topology_role, source.output_bus_intent);
    let target_role = runtime_bus_role_for_endpoint(target.topology_role, target.input_bus_intent);
    if source_role == RuntimeBusRole::AnalysisTap || target_role == RuntimeBusRole::AnalysisTap {
        return Some((
            RuntimeAuxiliaryPathKind::Analysis,
            format!("analysis:{}", source.output_bus_id),
            RuntimeBusRole::AnalysisTap,
            RuntimeBusIntent::AnalysisTap,
        ));
    }
    None
}

fn derive_runtime_bus_connections(
    planned_nodes: &[RuntimePlannedGraphNode],
) -> (
    Vec<RuntimeBusConnectionSummary>,
    Vec<RuntimeAuxiliaryPathSummary>,
) {
    let mut producers_by_bus =
        std::collections::BTreeMap::<&str, Vec<RuntimePlannedGraphNodeTopologyEndpoint<'_>>>::new();
    for node in planned_nodes {
        producers_by_bus
            .entry(node.output_bus_id.as_str())
            .or_default()
            .push(RuntimePlannedGraphNodeTopologyEndpoint {
                node_id: node.node_id.as_str(),
                topology_role: node.topology_role,
                input_bus_id: node.input_bus_id.as_str(),
                output_bus_id: node.output_bus_id.as_str(),
                input_bus_intent: node.input_bus_intent,
                output_bus_intent: node.output_bus_intent,
                bus_group_id: node.bus_group_id.as_deref(),
                send_return_id: node.send_return_id.as_deref(),
            });
    }

    let mut connections = Vec::new();
    let mut auxiliary_paths =
        std::collections::BTreeMap::<String, RuntimeAuxiliaryPathSummary>::new();

    for node in planned_nodes {
        let Some(producers) = producers_by_bus.get(node.input_bus_id.as_str()) else {
            continue;
        };
        let target = RuntimePlannedGraphNodeTopologyEndpoint {
            node_id: node.node_id.as_str(),
            topology_role: node.topology_role,
            input_bus_id: node.input_bus_id.as_str(),
            output_bus_id: node.output_bus_id.as_str(),
            input_bus_intent: node.input_bus_intent,
            output_bus_intent: node.output_bus_intent,
            bus_group_id: node.bus_group_id.as_deref(),
            send_return_id: node.send_return_id.as_deref(),
        };
        for source in producers {
            let auxiliary_path = runtime_auxiliary_path_for_connection(source, &target);
            let source_bus_role =
                runtime_bus_role_for_endpoint(source.topology_role, source.output_bus_intent);
            let target_bus_role =
                runtime_bus_role_for_endpoint(target.topology_role, target.input_bus_intent);
            let connection_id = format!(
                "{}:{}->{}:{}",
                source.node_id, source.output_bus_id, target.node_id, target.input_bus_id
            );
            let attachment_class = RuntimeBusConnectionAttachmentClass::Required;
            let fallback_outcome = RuntimeBusConnectionFallbackOutcome::NoFallback;
            let summary = format!(
                "connection={} source={}:{}/{:?} target={}:{}/{:?} path={:?} attachment={:?} fallback={:?}",
                connection_id,
                source.node_id,
                source.output_bus_id,
                source_bus_role,
                target.node_id,
                target.input_bus_id,
                target_bus_role,
                auxiliary_path.as_ref().map(|(kind, path_id, _, _)| format!("{kind:?}:{path_id}")),
                attachment_class,
                fallback_outcome,
            );
            connections.push(RuntimeBusConnectionSummary {
                connection_id: connection_id.clone(),
                source_node_id: source.node_id.into(),
                source_bus_id: source.output_bus_id.into(),
                source_bus_role,
                target_node_id: target.node_id.into(),
                target_bus_id: target.input_bus_id.into(),
                target_bus_role,
                auxiliary_path_kind: auxiliary_path.as_ref().map(|(kind, _, _, _)| *kind),
                auxiliary_path_id: auxiliary_path
                    .as_ref()
                    .map(|(_, path_id, _, _)| path_id.clone()),
                attachment_class,
                fallback_outcome,
                summary,
            });

            if let Some((path_kind, auxiliary_path_id, bus_role, material_bus_intent)) =
                auxiliary_path
            {
                let path = auxiliary_paths
                    .entry(auxiliary_path_id.clone())
                    .or_insert_with(|| RuntimeAuxiliaryPathSummary {
                        auxiliary_path_id: auxiliary_path_id.clone(),
                        path_kind,
                        bus_role,
                        material_bus_intent,
                        source_node_ids: Vec::new(),
                        target_node_ids: Vec::new(),
                        bus_ids: Vec::new(),
                        connection_ids: Vec::new(),
                        attachment_class,
                        fallback_outcome,
                        summary: String::new(),
                    });
                if !path.source_node_ids.contains(&source.node_id.to_string()) {
                    path.source_node_ids.push(source.node_id.to_string());
                }
                if !path.target_node_ids.contains(&target.node_id.to_string()) {
                    path.target_node_ids.push(target.node_id.to_string());
                }
                if !path.bus_ids.contains(&source.output_bus_id.to_string()) {
                    path.bus_ids.push(source.output_bus_id.to_string());
                }
                if !path.bus_ids.contains(&target.input_bus_id.to_string()) {
                    path.bus_ids.push(target.input_bus_id.to_string());
                }
                if !path.connection_ids.contains(&connection_id) {
                    path.connection_ids.push(connection_id.clone());
                }
            }
        }
    }

    let mut auxiliary_paths = auxiliary_paths.into_values().collect::<Vec<_>>();
    for path in &mut auxiliary_paths {
        path.summary = format!(
            "path={} kind={:?} role={:?} material={:?} sources={:?} targets={:?} buses={:?} connections={} attachment={:?} fallback={:?}",
            path.auxiliary_path_id,
            path.path_kind,
            path.bus_role,
            path.material_bus_intent,
            path.source_node_ids,
            path.target_node_ids,
            path.bus_ids,
            path.connection_ids.len(),
            path.attachment_class,
            path.fallback_outcome,
        );
    }

    (connections, auxiliary_paths)
}

impl RuntimeExecutionTopologySummary {
    pub fn from_snapshot(snapshot: &RuntimeEngineBlockSnapshot) -> Self {
        let mut track_lane_ids = std::collections::BTreeSet::new();
        let mut bus_group_ids = std::collections::BTreeSet::new();
        let mut send_return_ids = std::collections::BTreeSet::new();
        let mut console_group_ids = std::collections::BTreeSet::new();
        let mut lanes = Vec::new();

        for lane in &snapshot.lane_order {
            let mut groups = Vec::new();
            let mut node_ids = Vec::new();
            let mut topology_roles = Vec::new();
            let mut lane_ids = Vec::new();
            let mut bus_groups = Vec::new();
            let mut console_groups = Vec::new();
            let mut send_returns = Vec::new();

            for node in snapshot
                .planned_nodes
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
                    bus_group_ids.insert(bus_group_id.clone());
                }
                if let Some(console_group_id) = &node.console_group_id {
                    if !console_groups.contains(console_group_id) {
                        console_groups.push(console_group_id.clone());
                    }
                    console_group_ids.insert(console_group_id.clone());
                }
                if let Some(send_return_id) = &node.send_return_id {
                    if !send_returns.contains(send_return_id) {
                        send_returns.push(send_return_id.clone());
                    }
                    send_return_ids.insert(send_return_id.clone());
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

        let mut track_lanes_by_id =
            std::collections::BTreeMap::<String, RuntimeMixerTrackLaneSummary>::new();
        let mut bus_groups_by_id =
            std::collections::BTreeMap::<String, RuntimeMixerBusGroupSummary>::new();
        let mut console_groups_by_id =
            std::collections::BTreeMap::<String, RuntimeMixerConsoleGroupSummary>::new();
        let mut send_returns_by_id =
            std::collections::BTreeMap::<String, RuntimeMixerSendReturnSummary>::new();

        let mut nodes = Vec::with_capacity(snapshot.planned_nodes.len());
        let mut utility_node_count = 0usize;
        let mut track_lane_node_count = 0usize;
        let mut bus_node_count = 0usize;
        let mut send_return_node_count = 0usize;
        let mut console_node_count = 0usize;
        let mut secondary_inputs = Vec::new();
        let mut required_secondary_input_count = 0usize;
        let mut optional_secondary_input_count = 0usize;
        let mut disabled_secondary_input_count = 0usize;
        let mut terminal_fallback_secondary_input_count = 0usize;
        let mut spatial_node_count = 0usize;
        let mut active_spatial_node_count = 0usize;
        let mut bypassed_spatial_node_count = 0usize;
        let mut fallback_spatial_node_count = 0usize;
        let mut surround_bed_spatial_node_count = 0usize;
        let mut object_aware_spatial_node_count = 0usize;
        let mut expanded_fallback_spatial_node_count = 0usize;
        let mut immersive_spatial_node_count = 0usize;
        let mut room_policy_aware_spatial_node_count = 0usize;
        let mut fallback_room_policy_spatial_node_count = 0usize;
        let mut deployment_spatial_node_count = 0usize;
        let mut folded_down_spatial_node_count = 0usize;
        let mut fallback_monitoring_scene_spatial_node_count = 0usize;
        let mut renderer_capability_spatial_node_count = 0usize;
        let mut negotiated_renderer_spatial_node_count = 0usize;
        let mut immersive_export_spatial_node_count = 0usize;
        let mut fallback_immersive_export_spatial_node_count = 0usize;

        for node in &snapshot.planned_nodes {
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
            if let Some(secondary_input) = &node.secondary_input {
                secondary_inputs.push(secondary_input.clone());
                match secondary_input.attachment_policy {
                    RuntimeSecondaryInputAttachmentPolicy::Required => {
                        required_secondary_input_count += 1;
                    }
                    RuntimeSecondaryInputAttachmentPolicy::Optional => {
                        optional_secondary_input_count += 1;
                    }
                    RuntimeSecondaryInputAttachmentPolicy::Disabled => {
                        disabled_secondary_input_count += 1;
                    }
                }
                if secondary_input.fallback_outcome
                    == RuntimeSecondaryInputFallbackOutcome::TerminalRoutingFailure
                {
                    terminal_fallback_secondary_input_count += 1;
                }
            }
            if let Some(spatial_execution) = &node.spatial_execution {
                spatial_node_count += 1;
                if spatial_execution.execution_mode == RuntimeSpatialExecutionMode::Bypassed {
                    bypassed_spatial_node_count += 1;
                } else {
                    active_spatial_node_count += 1;
                }
                if spatial_execution.fallback_outcome.is_some() {
                    fallback_spatial_node_count += 1;
                }
                if spatial_execution.bed_class == RuntimeSpatialBedClass::CanonicalSurroundBed {
                    surround_bed_spatial_node_count += 1;
                }
                if spatial_execution.object_count > 0 || spatial_execution.object_role.is_some() {
                    object_aware_spatial_node_count += 1;
                }
                if spatial_execution.expanded_fallback_outcome.is_some() {
                    expanded_fallback_spatial_node_count += 1;
                }
                if let Some(immersive_room_policy) = &spatial_execution.immersive_room_policy {
                    immersive_spatial_node_count += 1;
                    if immersive_room_policy.object_rendering_posture
                        == RuntimeImmersiveObjectRenderingPosture::RoomPolicyAware
                    {
                        room_policy_aware_spatial_node_count += 1;
                    }
                    if immersive_room_policy.room_policy_class
                        == RuntimeRoomPolicyClass::FallbackRoom
                    {
                        fallback_room_policy_spatial_node_count += 1;
                    }
                }
                if let Some(deployment_monitoring) = &spatial_execution.deployment_monitoring {
                    deployment_spatial_node_count += 1;
                    if matches!(
                        deployment_monitoring.fold_down_policy,
                        RuntimeFoldDownPolicy::FoldDownToReferenceBed
                            | RuntimeFoldDownPolicy::FoldDownToStereoMonitoring
                            | RuntimeFoldDownPolicy::FoldDownToPortablePreview
                    ) {
                        folded_down_spatial_node_count += 1;
                    }
                    if deployment_monitoring.monitoring_scene_class
                        == RuntimeMonitoringSceneClass::FallbackScene
                    {
                        fallback_monitoring_scene_spatial_node_count += 1;
                    }
                }
                if let Some(renderer_export) = &spatial_execution.renderer_export {
                    renderer_capability_spatial_node_count += 1;
                    if renderer_export.renderer_capability_posture
                        == RuntimeRendererCapabilityNegotiationPosture::NegotiatedCompatible
                    {
                        negotiated_renderer_spatial_node_count += 1;
                    }
                    if renderer_export.immersive_export_class
                        != RuntimeImmersiveExportClass::NoImmersiveExport
                    {
                        immersive_export_spatial_node_count += 1;
                    }
                    if renderer_export.immersive_export_class
                        == RuntimeImmersiveExportClass::FallbackExport
                    {
                        fallback_immersive_export_spatial_node_count += 1;
                    }
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
                input_channels: node.input_channels,
                output_channels: node.output_channels,
                input_layout: node.input_layout.clone(),
                output_layout: node.output_layout.clone(),
                input_bus_intent: node.input_bus_intent,
                output_bus_intent: node.output_bus_intent,
                secondary_input: node.secondary_input.clone(),
                spatial_execution: node.spatial_execution.clone(),
                plugin_sandbox_id: node.plugin_sandbox_id.clone(),
                plugin_recall_state: None,
                plugin_recall: None,
                plugin_compensation_state: None,
                plugin_realized_latency_samples: None,
                plugin_tail_samples: None,
            });
        }
        let (bus_connections, auxiliary_paths) =
            derive_runtime_bus_connections(&snapshot.planned_nodes);

        Self {
            node_count: snapshot.planned_nodes.len(),
            utility_node_count,
            track_lane_node_count,
            bus_node_count,
            send_return_node_count,
            console_node_count,
            lane_count: lanes.len(),
            track_lane_group_count: track_lane_ids.len(),
            bus_group_count: bus_group_ids.len(),
            send_return_group_count: send_return_ids.len(),
            console_group_count: console_group_ids.len(),
            secondary_input_count: secondary_inputs.len(),
            required_secondary_input_count,
            optional_secondary_input_count,
            disabled_secondary_input_count,
            terminal_fallback_secondary_input_count,
            bus_connection_count: bus_connections.len(),
            auxiliary_path_count: auxiliary_paths.len(),
            spatial_node_count,
            active_spatial_node_count,
            bypassed_spatial_node_count,
            fallback_spatial_node_count,
            surround_bed_spatial_node_count,
            object_aware_spatial_node_count,
            expanded_fallback_spatial_node_count,
            immersive_spatial_node_count,
            room_policy_aware_spatial_node_count,
            fallback_room_policy_spatial_node_count,
            deployment_spatial_node_count,
            folded_down_spatial_node_count,
            fallback_monitoring_scene_spatial_node_count,
            renderer_capability_spatial_node_count,
            negotiated_renderer_spatial_node_count,
            immersive_export_spatial_node_count,
            fallback_immersive_export_spatial_node_count,
            lanes,
            track_lanes: track_lanes_by_id.into_values().collect(),
            bus_groups: bus_groups_by_id.into_values().collect(),
            console_groups: console_groups_by_id.into_values().collect(),
            send_returns: send_returns_by_id.into_values().collect(),
            secondary_inputs,
            bus_connections,
            auxiliary_paths,
            nodes,
            plugin_chain: RuntimeRoutedPluginChainSummary::default(),
        }
    }

    pub fn with_plugin_chain_snapshot(mut self, snapshot: &RuntimePluginChainSnapshot) -> Self {
        let mut stage_by_node =
            std::collections::BTreeMap::<&str, &RuntimePluginChainStageSnapshot>::new();
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
            for stage in &chain.stages {
                stage_by_node.insert(stage.node_id.as_str(), stage);
            }
        }

        for node in &mut self.nodes {
            if let Some(stage) = stage_by_node.get(node.node_id.as_str()) {
                node.spatial_execution = stage.spatial_execution.clone();
                node.plugin_recall_state = Some(stage.recall_state);
                node.plugin_recall = Some(stage.recall.clone());
                node.plugin_compensation_state = Some(stage.compensation_state);
                node.plugin_realized_latency_samples = stage.realized_latency_samples;
                node.plugin_tail_samples = stage.tail_samples;
            }
        }

        self
    }
}
