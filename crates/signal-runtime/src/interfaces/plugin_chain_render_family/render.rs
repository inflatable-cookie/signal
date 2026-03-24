use super::*;

pub(crate) fn format_runtime_plugin_chain_snapshot_compact(
    snapshot: &RuntimePluginChainSnapshot,
) -> String {
    format!(
        " plugin_chains={}/{} plugin_chain_placement={}/{}/{} plugin_chain_pending={} plugin_chain_settling={} plugin_chain_compensated={} plugin_chain_degraded={} plugin_chain_bypassed={} plugin_chain_missing={} plugin_chain_rebindable={} plugin_chain_terminal={} plugin_chain_latency={}/{} plugin_chain_tail={}",
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
        snapshot.total_realized_latency_samples,
        snapshot.total_planned_latency_samples,
        snapshot.total_tail_samples,
    )
}

pub(crate) fn format_runtime_plugin_recall_snapshot_compact(
    snapshot: &RuntimePluginRecallSnapshot,
) -> String {
    format!(
        "{:?}/sandbox={:?}/plugin={:?}/{:?}/lifecycle={:?}/{:?}/{:?}/readiness={:?}/recoveries={}/restarts={}/faults={}/fault_kind={:?}/stop_reason={:?}/portability={:?}/preset={:?}/ara={}/degraded={:?}",
        snapshot.state,
        snapshot.payload.sandbox_id.as_deref(),
        snapshot.payload.plugin_type_id.as_deref(),
        snapshot.payload.plugin_format,
        snapshot.payload.lifecycle_state,
        snapshot.payload.lifecycle_stage,
        snapshot.payload.transport_stage,
        snapshot.payload.readiness_state.as_deref(),
        snapshot.payload.recovery_count,
        snapshot.payload.restart_count,
        snapshot.payload.fault_count,
        snapshot.payload.last_fault_kind,
        snapshot.payload.last_stop_reason,
        snapshot.payload.interchange.portability_class,
        snapshot
            .payload
            .interchange
            .preset_descriptor
            .as_ref()
            .and_then(|descriptor| descriptor.label.as_deref()),
        snapshot.payload.ara_context.is_some(),
        &snapshot.payload.degraded_reasons,
    )
}

pub(crate) fn format_runtime_plugin_chain_snapshot_multiline(
    snapshot: &RuntimePluginChainSnapshot,
) -> String {
    let chain_lines = snapshot
        .chains
        .iter()
        .enumerate()
        .map(|(chain_index, chain)| {
            let stage_lines = chain
                .stages
                .iter()
                .enumerate()
                .map(|(stage_index, stage)| {
                    format!(
                        "\nplugin_chain_{}_stage_{}={}/sandbox={:?}/group={:?}/placement={:?}/rule={:?}/members={}/continuity={:?}/rebindable={}/secondary_input={:?}/lifecycle={:?}/{:?}/transport={:?}/recall={}/compensation={:?}/latency={}/{:?}/{:?}/bypassed={}/active_transport={}/degraded_reasons={:?}",
                        chain_index,
                        stage_index,
                        stage.node_id,
                        stage.sandbox_id,
                        stage.sandbox_group_key,
                        stage.placement_outcome,
                        stage.placement_rule_id,
                        stage.shared_boundary_member_count,
                        stage.continuity_class,
                        stage.rebindable,
                        stage.secondary_input
                            .as_ref()
                            .map(|secondary_input| secondary_input.summary.as_str()),
                        stage.lifecycle_state,
                        stage.lifecycle_stage,
                        stage.transport_stage,
                        format_runtime_plugin_recall_snapshot_compact(&stage.recall),
                        stage.compensation_state,
                        stage.planned_latency_samples,
                        stage.realized_latency_samples,
                        stage.tail_samples,
                        stage.bypassed,
                        stage.active_transport,
                        stage.degraded_reasons,
                    )
                })
                .collect::<String>();
            format!(
                "\nplugin_chain_{}={}/track={:?}/bus={:?}/console={:?}/send_return={:?}/stages={}/shared={}/isolated={}/in_process={}/pending={}/settling={}/compensated={}/degraded={}/bypassed={}/missing={}/rebindable={}/terminal={}/latency={}/{}/{}{}",
                chain_index,
                chain.chain_id,
                chain.track_lane_id,
                chain.bus_group_id,
                chain.console_group_id,
                chain.send_return_id,
                chain.stage_count,
                chain.shared_sandbox_stage_count,
                chain.isolated_sandbox_stage_count,
                chain.in_process_stage_count,
                chain.pending_render_stage_count,
                chain.settling_stage_count,
                chain.compensated_stage_count,
                chain.degraded_stage_count,
                chain.bypassed_stage_count,
                chain.missing_binding_stage_count,
                chain.rebindable_stage_count,
                chain.terminal_stage_count,
                chain.total_planned_latency_samples,
                chain.total_realized_latency_samples,
                chain.total_tail_samples,
                stage_lines,
            )
        })
        .collect::<String>();
    format!(
        "\nplugin_chain_count={}\nplugin_chain_stage_count={}\nplugin_chain_shared_sandbox_stage_count={}\nplugin_chain_isolated_sandbox_stage_count={}\nplugin_chain_in_process_stage_count={}\nplugin_chain_pending_render_stage_count={}\nplugin_chain_settling_stage_count={}\nplugin_chain_compensated_stage_count={}\nplugin_chain_degraded_stage_count={}\nplugin_chain_bypassed_stage_count={}\nplugin_chain_missing_binding_stage_count={}\nplugin_chain_rebindable_stage_count={}\nplugin_chain_terminal_stage_count={}\nplugin_chain_total_planned_latency_samples={}\nplugin_chain_total_realized_latency_samples={}\nplugin_chain_total_tail_samples={}{}",
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
        chain_lines,
    )
}

pub(crate) fn format_runtime_routed_plugin_chain_summary_compact(
    summary: &RuntimeRoutedPluginChainSummary,
) -> String {
    format!(
        "{}/{}/{}/{}/{}/{}/{}/{}/{}/{}/{}",
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
    )
}
