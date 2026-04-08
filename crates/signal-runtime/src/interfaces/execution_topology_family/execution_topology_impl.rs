use super::super::*;
use super::execution_topology_bus_connections::derive_runtime_bus_connections;
use super::execution_topology_lanes::build_topology_lanes;
use super::execution_topology_nodes::build_topology_nodes;

impl RuntimeExecutionTopologySummary {
    pub fn from_snapshot(snapshot: &RuntimeEngineBlockSnapshot) -> Self {
        let lanes_result = build_topology_lanes(snapshot);
        let nodes_result = build_topology_nodes(&snapshot.planned_nodes);
        let (bus_connections, auxiliary_paths) =
            derive_runtime_bus_connections(&snapshot.planned_nodes);

        Self {
            node_count: snapshot.planned_nodes.len(),
            utility_node_count: nodes_result.utility_node_count,
            track_lane_node_count: nodes_result.track_lane_node_count,
            bus_node_count: nodes_result.bus_node_count,
            send_return_node_count: nodes_result.send_return_node_count,
            console_node_count: nodes_result.console_node_count,
            lane_count: lanes_result.lanes.len(),
            track_lane_group_count: lanes_result.track_lane_ids.len(),
            bus_group_count: lanes_result.bus_group_ids.len(),
            send_return_group_count: lanes_result.send_return_ids.len(),
            console_group_count: lanes_result.console_group_ids.len(),
            secondary_input_count: nodes_result.secondary_inputs.len(),
            required_secondary_input_count: nodes_result.required_secondary_input_count,
            optional_secondary_input_count: nodes_result.optional_secondary_input_count,
            disabled_secondary_input_count: nodes_result.disabled_secondary_input_count,
            terminal_fallback_secondary_input_count: nodes_result
                .terminal_fallback_secondary_input_count,
            bus_connection_count: bus_connections.len(),
            auxiliary_path_count: auxiliary_paths.len(),
            spatial_node_count: nodes_result.spatial_node_count,
            active_spatial_node_count: nodes_result.active_spatial_node_count,
            bypassed_spatial_node_count: nodes_result.bypassed_spatial_node_count,
            fallback_spatial_node_count: nodes_result.fallback_spatial_node_count,
            surround_bed_spatial_node_count: nodes_result.surround_bed_spatial_node_count,
            object_aware_spatial_node_count: nodes_result.object_aware_spatial_node_count,
            expanded_fallback_spatial_node_count: nodes_result
                .expanded_fallback_spatial_node_count,
            immersive_spatial_node_count: nodes_result.immersive_spatial_node_count,
            room_policy_aware_spatial_node_count: nodes_result
                .room_policy_aware_spatial_node_count,
            fallback_room_policy_spatial_node_count: nodes_result
                .fallback_room_policy_spatial_node_count,
            deployment_spatial_node_count: nodes_result.deployment_spatial_node_count,
            folded_down_spatial_node_count: nodes_result.folded_down_spatial_node_count,
            fallback_monitoring_scene_spatial_node_count: nodes_result
                .fallback_monitoring_scene_spatial_node_count,
            renderer_capability_spatial_node_count: nodes_result
                .renderer_capability_spatial_node_count,
            negotiated_renderer_spatial_node_count: nodes_result
                .negotiated_renderer_spatial_node_count,
            immersive_export_spatial_node_count: nodes_result.immersive_export_spatial_node_count,
            fallback_immersive_export_spatial_node_count: nodes_result
                .fallback_immersive_export_spatial_node_count,
            lanes: lanes_result.lanes,
            track_lanes: nodes_result.track_lanes,
            bus_groups: nodes_result.bus_groups,
            console_groups: nodes_result.console_groups,
            send_returns: nodes_result.send_returns,
            secondary_inputs: nodes_result.secondary_inputs,
            bus_connections,
            auxiliary_paths,
            nodes: nodes_result.nodes,
            plugin_chain: RuntimeRoutedPluginChainSummary::default(),
        }
    }
}
