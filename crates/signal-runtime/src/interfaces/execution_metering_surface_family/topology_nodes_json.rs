use super::*;

pub(crate) fn json_runtime_execution_topology_nodes(
    nodes: &[RuntimeExecutionNodeSummary],
) -> String {
    format!(
        "[{}]",
        nodes
            .iter()
            .map(|node| {
                format!(
                    concat!(
                        "{{",
                        "\"node_id\":{},",
                        "\"lane\":{},",
                        "\"group\":{},",
                        "\"execution_class\":{},",
                        "\"topology_role\":{},",
                        "\"track_lane_id\":{},",
                        "\"bus_group_id\":{},",
                        "\"console_group_id\":{},",
                        "\"send_return_id\":{},",
                        "\"input_bus_id\":{},",
                        "\"output_bus_id\":{},",
                        "\"input_channels\":{},",
                        "\"output_channels\":{},",
                        "\"input_layout\":{},",
                        "\"output_layout\":{},",
                        "\"input_bus_intent\":{},",
                        "\"output_bus_intent\":{},",
                        "\"secondary_input\":{},",
                        "\"spatial_execution\":{},",
                        "\"plugin_sandbox_id\":{},",
                        "\"plugin_recall_state\":{},",
                        "\"plugin_recall\":{},",
                        "\"plugin_compensation_state\":{},",
                        "\"plugin_realized_latency_samples\":{},",
                        "\"plugin_tail_samples\":{}",
                        "}}"
                    ),
                    json_option_string(Some(node.node_id.as_str())),
                    json_option_string(Some(match node.lane {
                        GraphExecutionLane::Realtime => "Realtime",
                        GraphExecutionLane::Anticipative => "Anticipative",
                    })),
                    json_option_string(Some(match node.group {
                        GraphNodePlanningGroup::InlineRealtime => "InlineRealtime",
                        GraphNodePlanningGroup::StatefulRealtime => "StatefulRealtime",
                        GraphNodePlanningGroup::AnticipativeEligible => "AnticipativeEligible",
                    })),
                    json_option_string(Some(match node.execution_class {
                        GraphNodeExecutionClass::PureTransform => "PureTransform",
                        GraphNodeExecutionClass::Stateful => "Stateful",
                        GraphNodeExecutionClass::LatencyBearing => "LatencyBearing",
                        GraphNodeExecutionClass::PluginBacked => "PluginBacked",
                    })),
                    json_option_string(Some(match node.topology_role {
                        GraphNodeTopologyRole::Utility => "Utility",
                        GraphNodeTopologyRole::TrackLane => "TrackLane",
                        GraphNodeTopologyRole::Bus => "Bus",
                        GraphNodeTopologyRole::Send => "Send",
                        GraphNodeTopologyRole::Return => "Return",
                        GraphNodeTopologyRole::ConsoleNode => "ConsoleNode",
                    })),
                    json_option_string(node.track_lane_id.as_deref()),
                    json_option_string(node.bus_group_id.as_deref()),
                    json_option_string(node.console_group_id.as_deref()),
                    json_option_string(node.send_return_id.as_deref()),
                    json_option_string(Some(node.input_bus_id.as_str())),
                    json_option_string(Some(node.output_bus_id.as_str())),
                    json_option_string(Some(&format!("{:?}", node.input_channels))),
                    json_option_string(Some(&format!("{:?}", node.output_channels))),
                    json_runtime_multichannel_layout_summary(&node.input_layout),
                    json_runtime_multichannel_layout_summary(&node.output_layout),
                    json_runtime_bus_intent(node.input_bus_intent),
                    json_runtime_bus_intent(node.output_bus_intent),
                    node.secondary_input
                        .as_ref()
                        .map_or_else(|| "null".into(), json_runtime_secondary_input_route_summary),
                    node.spatial_execution
                        .as_ref()
                        .map_or_else(|| "null".into(), json_runtime_spatial_execution_summary),
                    json_option_string(node.plugin_sandbox_id.as_deref()),
                    json_option_string(
                        node.plugin_recall_state
                            .map(|state| format!("{state:?}"))
                            .as_deref(),
                    ),
                    node.plugin_recall
                        .as_ref()
                        .map_or_else(|| "null".into(), json_runtime_plugin_recall_snapshot),
                    json_option_string(
                        node.plugin_compensation_state
                            .map(|state| format!("{state:?}"))
                            .as_deref(),
                    ),
                    json_option_u32(node.plugin_realized_latency_samples),
                    json_option_u32(node.plugin_tail_samples),
                )
            })
            .collect::<Vec<_>>()
            .join(",")
    )
}
