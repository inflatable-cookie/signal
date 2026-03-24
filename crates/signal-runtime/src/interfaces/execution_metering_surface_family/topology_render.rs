use super::*;

pub(crate) fn format_runtime_execution_topology_summary_compact(
    summary: &RuntimeExecutionTopologySummary,
) -> String {
    let lane_shapes = summary
        .lanes
        .iter()
        .map(|lane| format!("{:?}:{}", lane.lane, lane.node_ids.len()))
        .collect::<Vec<_>>()
        .join("|");
    format!(
        " execution_topology_summary_nodes={} execution_topology_summary_roles={}/{}/{}/{}/{} execution_topology_summary_groups={}/{}/{} execution_topology_summary_secondary_inputs={}/{}/{}/{}/{} execution_topology_summary_bus_connections={} execution_topology_summary_auxiliary_paths={} execution_topology_summary_plugin_chain={} execution_topology_summary_lanes={} execution_topology_summary_lane_shapes={}",
        summary.node_count,
        summary.utility_node_count,
        summary.track_lane_node_count,
        summary.bus_node_count,
        summary.send_return_node_count,
        summary.console_node_count,
        summary.track_lane_group_count,
        summary.bus_group_count,
        summary.console_group_count,
        summary.secondary_input_count,
        summary.required_secondary_input_count,
        summary.optional_secondary_input_count,
        summary.disabled_secondary_input_count,
        summary.terminal_fallback_secondary_input_count,
        summary.bus_connection_count,
        summary.auxiliary_path_count,
        format_runtime_routed_plugin_chain_summary_compact(&summary.plugin_chain),
        summary.lane_count,
        lane_shapes,
    )
}

pub(crate) fn format_runtime_execution_topology_summary_multiline(
    summary: &RuntimeExecutionTopologySummary,
) -> String {
    let lane_lines = summary
        .lanes
        .iter()
        .enumerate()
        .map(|(index, lane)| {
            format!(
                "\nexecution_topology_summary_lane_{}={:?}/groups={:?}/nodes={:?}/roles={:?}/track_lanes={:?}/bus_groups={:?}/console_groups={:?}/send_returns={:?}",
                index,
                lane.lane,
                lane.groups,
                lane.node_ids,
                lane.topology_roles,
                lane.track_lane_ids,
                lane.bus_group_ids,
                lane.console_group_ids,
                lane.send_return_ids,
            )
        })
        .collect::<String>();
    let track_lane_lines = summary
        .track_lanes
        .iter()
        .enumerate()
        .map(|(index, track_lane)| {
            format!(
                "\nexecution_topology_summary_track_lane_{}={}/nodes={:?}/bus_groups={:?}/input={:?}/output={:?}/plugin_chain={}",
                index,
                track_lane.track_lane_id,
                track_lane.node_ids,
                track_lane.bus_group_ids,
                track_lane.input_bus_ids,
                track_lane.output_bus_ids,
                format_runtime_routed_plugin_chain_summary_compact(&track_lane.plugin_chain),
            )
        })
        .collect::<String>();
    let bus_group_lines = summary
        .bus_groups
        .iter()
        .enumerate()
        .map(|(index, bus_group)| {
            format!(
                "\nexecution_topology_summary_bus_group_{}={}/roles={:?}/nodes={:?}/input={:?}/output={:?}/plugin_chain={}",
                index,
                bus_group.bus_group_id,
                bus_group.topology_roles,
                bus_group.node_ids,
                bus_group.input_bus_ids,
                bus_group.output_bus_ids,
                format_runtime_routed_plugin_chain_summary_compact(&bus_group.plugin_chain),
            )
        })
        .collect::<String>();
    let console_group_lines = summary
        .console_groups
        .iter()
        .enumerate()
        .map(|(index, console_group)| {
            format!(
                "\nexecution_topology_summary_console_group_{}={}/nodes={:?}/input={:?}/output={:?}/plugin_chain={}",
                index,
                console_group.console_group_id,
                console_group.node_ids,
                console_group.input_bus_ids,
                console_group.output_bus_ids,
                format_runtime_routed_plugin_chain_summary_compact(&console_group.plugin_chain),
            )
        })
        .collect::<String>();
    let send_return_lines = summary
        .send_returns
        .iter()
        .enumerate()
        .map(|(index, send_return)| {
            format!(
                "\nexecution_topology_summary_send_return_{}={}/sends={:?}/returns={:?}/input={:?}/output={:?}/plugin_chain={}",
                index,
                send_return.send_return_id,
                send_return.send_node_ids,
                send_return.return_node_ids,
                send_return.input_bus_ids,
                send_return.output_bus_ids,
                format_runtime_routed_plugin_chain_summary_compact(&send_return.plugin_chain),
            )
        })
        .collect::<String>();
    let node_lines = summary
        .nodes
        .iter()
        .enumerate()
        .map(|(index, node)| {
            format!(
                "\nexecution_topology_summary_node_{}={}/{:?}/{:?}/{:?}/track_lane_id={:?}/bus_group_id={:?}/console_group_id={:?}/send_return_id={:?}/input={}/output={}/secondary_input={:?}/plugin={:?}/plugin_recall={:?}/plugin_recall_payload={:?}/plugin_compensation={:?}/plugin_realized_latency={:?}/plugin_tail={:?}",
                index,
                node.node_id,
                node.lane,
                node.group,
                node.topology_role,
                node.track_lane_id,
                node.bus_group_id,
                node.console_group_id,
                node.send_return_id,
                node.input_bus_id,
                node.output_bus_id,
                node.secondary_input
                    .as_ref()
                    .map(|secondary_input| secondary_input.summary.as_str()),
                node.plugin_sandbox_id,
                node.plugin_recall_state,
                node.plugin_recall
                    .as_ref()
                    .map(format_runtime_plugin_recall_snapshot_compact),
                node.plugin_compensation_state,
                node.plugin_realized_latency_samples,
                node.plugin_tail_samples,
            )
        })
        .collect::<String>();
    let bus_connection_lines = summary
        .bus_connections
        .iter()
        .enumerate()
        .map(|(index, connection)| {
            format!(
                "\nexecution_topology_summary_bus_connection_{}={}",
                index, connection.summary
            )
        })
        .collect::<String>();
    let auxiliary_path_lines = summary
        .auxiliary_paths
        .iter()
        .enumerate()
        .map(|(index, path)| {
            format!(
                "\nexecution_topology_summary_auxiliary_path_{}={}",
                index, path.summary
            )
        })
        .collect::<String>();
    format!(
        "\nexecution_topology_summary_node_count={}\nexecution_topology_summary_utility_nodes={}\nexecution_topology_summary_track_lane_nodes={}\nexecution_topology_summary_bus_nodes={}\nexecution_topology_summary_send_return_nodes={}\nexecution_topology_summary_console_nodes={}\nexecution_topology_summary_lane_count={}\nexecution_topology_summary_track_lane_groups={}\nexecution_topology_summary_bus_groups={}\nexecution_topology_summary_send_return_groups={}\nexecution_topology_summary_console_groups={}\nexecution_topology_summary_secondary_input_count={}\nexecution_topology_summary_required_secondary_input_count={}\nexecution_topology_summary_optional_secondary_input_count={}\nexecution_topology_summary_disabled_secondary_input_count={}\nexecution_topology_summary_terminal_fallback_secondary_input_count={}\nexecution_topology_summary_bus_connection_count={}\nexecution_topology_summary_auxiliary_path_count={}\nexecution_topology_summary_plugin_chain={}{}{}{}{}{}{}{}{}",
        summary.node_count,
        summary.utility_node_count,
        summary.track_lane_node_count,
        summary.bus_node_count,
        summary.send_return_node_count,
        summary.console_node_count,
        summary.lane_count,
        summary.track_lane_group_count,
        summary.bus_group_count,
        summary.send_return_group_count,
        summary.console_group_count,
        summary.secondary_input_count,
        summary.required_secondary_input_count,
        summary.optional_secondary_input_count,
        summary.disabled_secondary_input_count,
        summary.terminal_fallback_secondary_input_count,
        summary.bus_connection_count,
        summary.auxiliary_path_count,
        format_runtime_routed_plugin_chain_summary_compact(&summary.plugin_chain),
        lane_lines,
        track_lane_lines,
        bus_group_lines,
        console_group_lines,
        send_return_lines,
        node_lines,
        bus_connection_lines,
        auxiliary_path_lines,
    )
}
