use super::super::*;

pub struct TopologyLanesResult {
    pub lanes: Vec<RuntimeExecutionLaneSummary>,
    pub track_lane_ids: std::collections::BTreeSet<String>,
    pub bus_group_ids: std::collections::BTreeSet<String>,
    pub send_return_ids: std::collections::BTreeSet<String>,
    pub console_group_ids: std::collections::BTreeSet<String>,
}

pub fn build_topology_lanes(snapshot: &RuntimeEngineBlockSnapshot) -> TopologyLanesResult {
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

    TopologyLanesResult {
        lanes,
        track_lane_ids,
        bus_group_ids,
        send_return_ids,
        console_group_ids,
    }
}
