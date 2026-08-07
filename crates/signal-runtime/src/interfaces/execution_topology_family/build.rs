use super::*;

pub(super) struct TopologyNodesResult {
    pub(super) nodes: Vec<RuntimeExecutionNodeSummary>,
    pub(super) track_lanes: Vec<RuntimeMixerTrackLaneSummary>,
    pub(super) bus_groups: Vec<RuntimeMixerBusGroupSummary>,
    pub(super) console_groups: Vec<RuntimeMixerConsoleGroupSummary>,
    pub(super) send_returns: Vec<RuntimeMixerSendReturnSummary>,
    pub(super) utility_node_count: usize,
    pub(super) track_lane_node_count: usize,
    pub(super) bus_node_count: usize,
    pub(super) send_return_node_count: usize,
    pub(super) console_node_count: usize,
}

pub(super) fn build_topology_nodes(
    planned_nodes: &[RuntimePlannedGraphNode],
) -> TopologyNodesResult {
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
