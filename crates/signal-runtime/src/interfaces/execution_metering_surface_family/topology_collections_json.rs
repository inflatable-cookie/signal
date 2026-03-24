use super::*;

pub(crate) fn json_runtime_execution_topology_lanes(
    lanes: &[RuntimeExecutionLaneSummary],
) -> String {
    format!(
        "[{}]",
        lanes
            .iter()
            .map(|lane| {
                format!(
                    concat!(
                        "{{",
                        "\"lane\":{},",
                        "\"groups\":{},",
                        "\"node_ids\":{},",
                        "\"topology_roles\":{},",
                        "\"track_lane_ids\":{},",
                        "\"bus_group_ids\":{},",
                        "\"console_group_ids\":{},",
                        "\"send_return_ids\":{}",
                        "}}"
                    ),
                    json_option_string(Some(match lane.lane {
                        GraphExecutionLane::Realtime => "Realtime",
                        GraphExecutionLane::Anticipative => "Anticipative",
                    })),
                    json_runtime_planning_group_order(&lane.groups),
                    json_string_vec(&lane.node_ids),
                    json_runtime_topology_role_vec(&lane.topology_roles),
                    json_string_vec(&lane.track_lane_ids),
                    json_string_vec(&lane.bus_group_ids),
                    json_string_vec(&lane.console_group_ids),
                    json_string_vec(&lane.send_return_ids),
                )
            })
            .collect::<Vec<_>>()
            .join(",")
    )
}

pub(crate) fn json_runtime_mixer_track_lanes(
    track_lanes: &[RuntimeMixerTrackLaneSummary],
) -> String {
    format!(
        "[{}]",
        track_lanes
            .iter()
            .map(|track_lane| {
                format!(
                    concat!(
                        "{{",
                        "\"track_lane_id\":{},",
                        "\"node_ids\":{},",
                        "\"bus_group_ids\":{},",
                        "\"input_bus_ids\":{},",
                        "\"output_bus_ids\":{},",
                        "\"plugin_chain\":{}",
                        "}}"
                    ),
                    json_option_string(Some(track_lane.track_lane_id.as_str())),
                    json_string_vec(&track_lane.node_ids),
                    json_string_vec(&track_lane.bus_group_ids),
                    json_string_vec(&track_lane.input_bus_ids),
                    json_string_vec(&track_lane.output_bus_ids),
                    json_runtime_routed_plugin_chain_summary(&track_lane.plugin_chain),
                )
            })
            .collect::<Vec<_>>()
            .join(",")
    )
}

pub(crate) fn json_runtime_mixer_bus_groups(bus_groups: &[RuntimeMixerBusGroupSummary]) -> String {
    format!(
        "[{}]",
        bus_groups
            .iter()
            .map(|bus_group| {
                format!(
                    concat!(
                        "{{",
                        "\"bus_group_id\":{},",
                        "\"topology_roles\":{},",
                        "\"node_ids\":{},",
                        "\"input_bus_ids\":{},",
                        "\"output_bus_ids\":{},",
                        "\"plugin_chain\":{}",
                        "}}"
                    ),
                    json_option_string(Some(bus_group.bus_group_id.as_str())),
                    json_runtime_topology_role_vec(&bus_group.topology_roles),
                    json_string_vec(&bus_group.node_ids),
                    json_string_vec(&bus_group.input_bus_ids),
                    json_string_vec(&bus_group.output_bus_ids),
                    json_runtime_routed_plugin_chain_summary(&bus_group.plugin_chain),
                )
            })
            .collect::<Vec<_>>()
            .join(",")
    )
}

pub(crate) fn json_runtime_mixer_console_groups(
    console_groups: &[RuntimeMixerConsoleGroupSummary],
) -> String {
    format!(
        "[{}]",
        console_groups
            .iter()
            .map(|console_group| {
                format!(
                    concat!(
                        "{{",
                        "\"console_group_id\":{},",
                        "\"node_ids\":{},",
                        "\"input_bus_ids\":{},",
                        "\"output_bus_ids\":{},",
                        "\"plugin_chain\":{}",
                        "}}"
                    ),
                    json_option_string(Some(console_group.console_group_id.as_str())),
                    json_string_vec(&console_group.node_ids),
                    json_string_vec(&console_group.input_bus_ids),
                    json_string_vec(&console_group.output_bus_ids),
                    json_runtime_routed_plugin_chain_summary(&console_group.plugin_chain),
                )
            })
            .collect::<Vec<_>>()
            .join(",")
    )
}

pub(crate) fn json_runtime_mixer_send_returns(
    send_returns: &[RuntimeMixerSendReturnSummary],
) -> String {
    format!(
        "[{}]",
        send_returns
            .iter()
            .map(|send_return| {
                format!(
                    concat!(
                        "{{",
                        "\"send_return_id\":{},",
                        "\"send_node_ids\":{},",
                        "\"return_node_ids\":{},",
                        "\"input_bus_ids\":{},",
                        "\"output_bus_ids\":{},",
                        "\"plugin_chain\":{}",
                        "}}"
                    ),
                    json_option_string(Some(send_return.send_return_id.as_str())),
                    json_string_vec(&send_return.send_node_ids),
                    json_string_vec(&send_return.return_node_ids),
                    json_string_vec(&send_return.input_bus_ids),
                    json_string_vec(&send_return.output_bus_ids),
                    json_runtime_routed_plugin_chain_summary(&send_return.plugin_chain),
                )
            })
            .collect::<Vec<_>>()
            .join(",")
    )
}

pub(crate) fn json_runtime_planning_group_order(groups: &[GraphNodePlanningGroup]) -> String {
    format!(
        "[{}]",
        groups
            .iter()
            .map(|group| {
                json_option_string(Some(match group {
                    GraphNodePlanningGroup::InlineRealtime => "InlineRealtime",
                    GraphNodePlanningGroup::StatefulRealtime => "StatefulRealtime",
                    GraphNodePlanningGroup::AnticipativeEligible => "AnticipativeEligible",
                }))
            })
            .collect::<Vec<_>>()
            .join(",")
    )
}

pub(crate) fn json_runtime_topology_role_vec(roles: &[GraphNodeTopologyRole]) -> String {
    format!(
        "[{}]",
        roles
            .iter()
            .map(|role| {
                json_option_string(Some(match role {
                    GraphNodeTopologyRole::Utility => "Utility",
                    GraphNodeTopologyRole::TrackLane => "TrackLane",
                    GraphNodeTopologyRole::Bus => "Bus",
                    GraphNodeTopologyRole::Send => "Send",
                    GraphNodeTopologyRole::Return => "Return",
                    GraphNodeTopologyRole::ConsoleNode => "ConsoleNode",
                }))
            })
            .collect::<Vec<_>>()
            .join(",")
    )
}
