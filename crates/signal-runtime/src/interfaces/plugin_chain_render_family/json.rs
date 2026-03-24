use super::*;

fn json_runtime_plugin_chain_stage_snapshot(snapshot: &RuntimePluginChainStageSnapshot) -> String {
    let lifecycle_state = snapshot.lifecycle_state.map(|state| format!("{state:?}"));
    let lifecycle_stage = snapshot.lifecycle_stage.map(|stage| format!("{stage:?}"));
    let transport_stage = snapshot.transport_stage.map(|stage| format!("{stage:?}"));
    format!(
        concat!(
            "{{",
            "\"node_id\":{},",
            "\"stage_index\":{},",
            "\"sandbox_id\":{},",
            "\"sandbox_group_key\":{},",
            "\"track_lane_id\":{},",
            "\"bus_group_id\":{},",
            "\"console_group_id\":{},",
            "\"send_return_id\":{},",
            "\"placement_outcome\":\"{:?}\",",
            "\"placement_rule_id\":{},",
            "\"shared_boundary_member_count\":{},",
            "\"continuity_class\":\"{:?}\",",
            "\"rebindable\":{},",
            "\"io_layout\":{},",
            "\"complex_io_summary\":{},",
            "\"secondary_input\":{},",
            "\"spatial_execution\":{},",
            "\"lifecycle_state\":{},",
            "\"lifecycle_stage\":{},",
            "\"transport_stage\":{},",
            "\"recall_state\":\"{:?}\",",
            "\"recall\":{},",
            "\"compensation_state\":\"{:?}\",",
            "\"planned_latency_samples\":{},",
            "\"realized_latency_samples\":{},",
            "\"tail_samples\":{},",
            "\"bypassed\":{},",
            "\"active_transport\":{},",
            "\"degraded_reasons\":{},",
            "\"summary\":{}",
            "}}"
        ),
        json_option_string(Some(snapshot.node_id.as_str())),
        snapshot.stage_index,
        json_option_string(snapshot.sandbox_id.as_deref()),
        json_option_string(snapshot.sandbox_group_key.as_deref()),
        json_option_string(snapshot.track_lane_id.as_deref()),
        json_option_string(snapshot.bus_group_id.as_deref()),
        json_option_string(snapshot.console_group_id.as_deref()),
        json_option_string(snapshot.send_return_id.as_deref()),
        snapshot.placement_outcome,
        json_option_string(snapshot.placement_rule_id.as_deref()),
        snapshot.shared_boundary_member_count,
        snapshot.continuity_class,
        snapshot.rebindable,
        json_runtime_multichannel_io_summary(&snapshot.io_layout),
        json_runtime_plugin_complex_io_summary(&snapshot.complex_io_summary),
        snapshot
            .secondary_input
            .as_ref()
            .map_or_else(|| "null".into(), json_runtime_secondary_input_route_summary,),
        snapshot
            .spatial_execution
            .as_ref()
            .map_or_else(|| "null".into(), json_runtime_spatial_execution_summary),
        json_option_string(lifecycle_state.as_deref()),
        json_option_string(lifecycle_stage.as_deref()),
        json_option_string(transport_stage.as_deref()),
        snapshot.recall_state,
        json_runtime_plugin_recall_snapshot(&snapshot.recall),
        snapshot.compensation_state,
        snapshot.planned_latency_samples,
        json_option_u32(snapshot.realized_latency_samples),
        json_option_u32(snapshot.tail_samples),
        snapshot.bypassed,
        snapshot.active_transport,
        json_string_vec(&snapshot.degraded_reasons),
        json_option_string(Some(snapshot.summary.as_str())),
    )
}

fn json_runtime_plugin_chain_stage_snapshot_vec(
    snapshots: &[RuntimePluginChainStageSnapshot],
) -> String {
    let joined = snapshots
        .iter()
        .map(json_runtime_plugin_chain_stage_snapshot)
        .collect::<Vec<_>>()
        .join(",");
    format!("[{joined}]")
}

fn json_runtime_plugin_execution_chain_summary(
    summary: &RuntimePluginExecutionChainSummary,
) -> String {
    format!(
        concat!(
            "{{",
            "\"chain_id\":{},",
            "\"track_lane_id\":{},",
            "\"bus_group_id\":{},",
            "\"console_group_id\":{},",
            "\"send_return_id\":{},",
            "\"stage_count\":{},",
            "\"shared_sandbox_stage_count\":{},",
            "\"isolated_sandbox_stage_count\":{},",
            "\"in_process_stage_count\":{},",
            "\"pending_render_stage_count\":{},",
            "\"settling_stage_count\":{},",
            "\"compensated_stage_count\":{},",
            "\"degraded_stage_count\":{},",
            "\"bypassed_stage_count\":{},",
            "\"missing_binding_stage_count\":{},",
            "\"rebindable_stage_count\":{},",
            "\"terminal_stage_count\":{},",
            "\"total_planned_latency_samples\":{},",
            "\"total_realized_latency_samples\":{},",
            "\"total_tail_samples\":{},",
            "\"stages\":{},",
            "\"summary\":{}",
            "}}"
        ),
        json_option_string(Some(summary.chain_id.as_str())),
        json_option_string(summary.track_lane_id.as_deref()),
        json_option_string(summary.bus_group_id.as_deref()),
        json_option_string(summary.console_group_id.as_deref()),
        json_option_string(summary.send_return_id.as_deref()),
        summary.stage_count,
        summary.shared_sandbox_stage_count,
        summary.isolated_sandbox_stage_count,
        summary.in_process_stage_count,
        summary.pending_render_stage_count,
        summary.settling_stage_count,
        summary.compensated_stage_count,
        summary.degraded_stage_count,
        summary.bypassed_stage_count,
        summary.missing_binding_stage_count,
        summary.rebindable_stage_count,
        summary.terminal_stage_count,
        summary.total_planned_latency_samples,
        summary.total_realized_latency_samples,
        summary.total_tail_samples,
        json_runtime_plugin_chain_stage_snapshot_vec(&summary.stages),
        json_option_string(Some(summary.summary.as_str())),
    )
}

fn json_runtime_plugin_execution_chain_summary_vec(
    summaries: &[RuntimePluginExecutionChainSummary],
) -> String {
    let joined = summaries
        .iter()
        .map(json_runtime_plugin_execution_chain_summary)
        .collect::<Vec<_>>()
        .join(",");
    format!("[{joined}]")
}

pub(crate) fn json_runtime_plugin_chain_snapshot(snapshot: &RuntimePluginChainSnapshot) -> String {
    format!(
        concat!(
            "{{",
            "\"chain_count\":{},",
            "\"stage_count\":{},",
            "\"shared_sandbox_stage_count\":{},",
            "\"isolated_sandbox_stage_count\":{},",
            "\"in_process_stage_count\":{},",
            "\"pending_render_stage_count\":{},",
            "\"settling_stage_count\":{},",
            "\"compensated_stage_count\":{},",
            "\"degraded_stage_count\":{},",
            "\"bypassed_stage_count\":{},",
            "\"missing_binding_stage_count\":{},",
            "\"rebindable_stage_count\":{},",
            "\"terminal_stage_count\":{},",
            "\"total_planned_latency_samples\":{},",
            "\"total_realized_latency_samples\":{},",
            "\"total_tail_samples\":{},",
            "\"chains\":{},",
            "\"summary\":{}",
            "}}"
        ),
        snapshot.chain_count,
        snapshot.stage_count,
        snapshot.shared_sandbox_stage_count,
        snapshot.isolated_sandbox_stage_count,
        snapshot.in_process_stage_count,
        snapshot.pending_render_stage_count,
        snapshot.settling_stage_count,
        snapshot.compensated_stage_count,
        snapshot.degraded_stage_count,
        snapshot.bypassed_stage_count,
        snapshot.missing_binding_stage_count,
        snapshot.rebindable_stage_count,
        snapshot.terminal_stage_count,
        snapshot.total_planned_latency_samples,
        snapshot.total_realized_latency_samples,
        snapshot.total_tail_samples,
        json_runtime_plugin_execution_chain_summary_vec(&snapshot.chains),
        json_option_string(Some(snapshot.summary.as_str())),
    )
}

pub(crate) fn json_runtime_routed_plugin_chain_summary(
    summary: &RuntimeRoutedPluginChainSummary,
) -> String {
    format!(
        concat!(
            "{{",
            "\"chain_count\":{},",
            "\"stage_count\":{},",
            "\"pending_render_stage_count\":{},",
            "\"settling_stage_count\":{},",
            "\"compensated_stage_count\":{},",
            "\"degraded_stage_count\":{},",
            "\"bypassed_stage_count\":{},",
            "\"missing_binding_stage_count\":{},",
            "\"total_planned_latency_samples\":{},",
            "\"total_realized_latency_samples\":{},",
            "\"total_tail_samples\":{},",
            "\"chain_ids\":{},",
            "\"node_ids\":{},",
            "\"sandbox_ids\":{}",
            "}}"
        ),
        summary.chain_count,
        summary.stage_count,
        summary.pending_render_stage_count,
        summary.settling_stage_count,
        summary.compensated_stage_count,
        summary.degraded_stage_count,
        summary.bypassed_stage_count,
        summary.missing_binding_stage_count,
        summary.total_planned_latency_samples,
        summary.total_realized_latency_samples,
        summary.total_tail_samples,
        json_string_vec(&summary.chain_ids),
        json_string_vec(&summary.node_ids),
        json_string_vec(&summary.sandbox_ids),
    )
}
