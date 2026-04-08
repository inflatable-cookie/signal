use super::super::*;

pub struct TopologyNodesResult {
    pub nodes: Vec<RuntimeExecutionNodeSummary>,
    pub track_lanes: Vec<RuntimeMixerTrackLaneSummary>,
    pub bus_groups: Vec<RuntimeMixerBusGroupSummary>,
    pub console_groups: Vec<RuntimeMixerConsoleGroupSummary>,
    pub send_returns: Vec<RuntimeMixerSendReturnSummary>,
    pub secondary_inputs: Vec<RuntimeSecondaryInputRouteSummary>,
    pub utility_node_count: usize,
    pub track_lane_node_count: usize,
    pub bus_node_count: usize,
    pub send_return_node_count: usize,
    pub console_node_count: usize,
    pub required_secondary_input_count: usize,
    pub optional_secondary_input_count: usize,
    pub disabled_secondary_input_count: usize,
    pub terminal_fallback_secondary_input_count: usize,
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
}

pub fn build_topology_nodes(planned_nodes: &[RuntimePlannedGraphNode]) -> TopologyNodesResult {
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
                if immersive_room_policy.room_policy_class == RuntimeRoomPolicyClass::FallbackRoom {
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

    TopologyNodesResult {
        nodes,
        track_lanes: track_lanes_by_id.into_values().collect(),
        bus_groups: bus_groups_by_id.into_values().collect(),
        console_groups: console_groups_by_id.into_values().collect(),
        send_returns: send_returns_by_id.into_values().collect(),
        secondary_inputs,
        utility_node_count,
        track_lane_node_count,
        bus_node_count,
        send_return_node_count,
        console_node_count,
        required_secondary_input_count,
        optional_secondary_input_count,
        disabled_secondary_input_count,
        terminal_fallback_secondary_input_count,
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
    }
}
