use super::*;

pub(crate) fn format_runtime_plugin_lifecycle_snapshot_compact(
    snapshot: &RuntimePluginLifecycleSnapshot,
) -> String {
    format!(
        " plugin_sandboxes={}/{} plugin_sandbox_placement={}/{} plugin_sandbox_rebindable={} plugin_sandbox_terminal={} plugin_parity_formats={}",
        snapshot.sandbox_count,
        snapshot.active_sandbox_count,
        snapshot.shared_sandbox_count,
        snapshot.isolated_sandbox_count,
        snapshot.rebindable_sandbox_count,
        snapshot.terminal_sandbox_count,
        snapshot.parity_coverage.len(),
    )
}

pub(crate) fn format_runtime_plugin_lifecycle_snapshot_multiline(
    snapshot: &RuntimePluginLifecycleSnapshot,
) -> String {
    let parity_coverage_lines = snapshot
        .parity_coverage
        .iter()
        .enumerate()
        .map(|(index, parity)| {
            format!(
                "\nplugin_lifecycle_parity_coverage_{}={:?}/{:?}/linux={:?}/linux_supported={}/linux_policy={:?}/linux_strict_default={}/supported={:?}/unsupported={:?}/types={}/prepare_capable={}/activate_capable={}/sandboxes={}/in_process={}/shared={}/isolated={}/ready={}/restarting={}/rebindable={}/degraded={}/faulted={}/quarantined={}/terminal={}/transport_active={}/placement_rules={}",
                index,
                parity.format,
                parity.parity_band,
                parity.linux_parity_band,
                parity.linux_supported,
                parity.linux_preferred_sandbox_outcome,
                parity.linux_strict_sandbox_default,
                parity.supported_platforms,
                parity.unsupported_platforms,
                parity.discovered_type_count,
                parity.prepare_capable_type_count,
                parity.activate_capable_type_count,
                parity.sandbox_count,
                parity.in_process_sandbox_count,
                parity.shared_sandbox_count,
                parity.isolated_sandbox_count,
                parity.ready_sandbox_count,
                parity.restarting_sandbox_count,
                parity.rebindable_sandbox_count,
                parity.degraded_sandbox_count,
                parity.faulted_sandbox_count,
                parity.quarantined_sandbox_count,
                parity.terminal_sandbox_count,
                parity.active_transport_count,
                parity.explicit_placement_rule_count,
            )
        })
        .collect::<String>();
    let sandbox_lines = snapshot
        .sandboxes
        .iter()
        .enumerate()
        .map(|(index, sandbox)| {
            format!(
                "\nplugin_sandbox_{}={}/group={}/placement={:?}/rule={:?}/members={}/continuity={:?}/rebindable={}/state={:?}/lifecycle={:?}/transport={:?}/preset={:?}/ara={}/ready={:?}/restarts={}/recoveries={}/faults={}/active={}/transport_active={}/degraded={:?}",
                index,
                sandbox.sandbox_id,
                sandbox.sandbox_group_key,
                sandbox.placement_outcome,
                sandbox.placement_rule_id,
                sandbox.shared_boundary_member_count,
                sandbox.continuity_class,
                sandbox.rebindable,
                sandbox.state,
                sandbox.lifecycle_stage,
                sandbox.transport_stage,
                sandbox
                    .preset_descriptor
                    .as_ref()
                    .and_then(|descriptor| descriptor.label.as_deref()),
                sandbox.ara_context.is_some(),
                sandbox.readiness_state,
                sandbox.restart_count,
                sandbox.recovery_count,
                sandbox.fault_count,
                sandbox.active,
                sandbox.active_transport,
                sandbox.degraded_reasons,
            )
        })
        .collect::<String>();
    format!(
        "\nplugin_sandbox_count={}\nplugin_active_sandbox_count={}\nplugin_shared_sandbox_count={}\nplugin_isolated_sandbox_count={}\nplugin_ready_sandbox_count={}\nplugin_booting_sandbox_count={}\nplugin_degraded_sandbox_count={}\nplugin_faulted_sandbox_count={}\nplugin_restarting_sandbox_count={}\nplugin_quarantined_sandbox_count={}\nplugin_stopped_sandbox_count={}\nplugin_rebindable_sandbox_count={}\nplugin_terminal_sandbox_count={}{}{}",
        snapshot.sandbox_count,
        snapshot.active_sandbox_count,
        snapshot.shared_sandbox_count,
        snapshot.isolated_sandbox_count,
        snapshot.ready_sandbox_count,
        snapshot.booting_sandbox_count,
        snapshot.degraded_sandbox_count,
        snapshot.faulted_sandbox_count,
        snapshot.restarting_sandbox_count,
        snapshot.quarantined_sandbox_count,
        snapshot.stopped_sandbox_count,
        snapshot.rebindable_sandbox_count,
        snapshot.terminal_sandbox_count,
        parity_coverage_lines,
        sandbox_lines,
    )
}

pub(crate) fn format_runtime_plugin_pin_matrix_snapshot_compact(
    snapshot: &RuntimePluginPinMatrixSnapshot,
) -> String {
    format!(
        " plugin_pin_matrix=types={} negotiated={} guarded={} unavailable={} dynamic_negotiated={} dynamic_guarded={}",
        snapshot.plugin_type_count,
        snapshot.negotiated_type_count,
        snapshot.guarded_type_count,
        snapshot.unavailable_type_count,
        snapshot.dynamic_negotiated_type_count,
        snapshot.dynamic_guarded_type_count,
    )
}

pub(crate) fn format_runtime_plugin_pin_matrix_snapshot_multiline(
    snapshot: &RuntimePluginPinMatrixSnapshot,
) -> String {
    let record_lines = snapshot
        .records
        .iter()
        .enumerate()
        .map(|(index, record)| {
            format!(
                "\nplugin_pin_matrix_record_{}={}/plugin_id={}/pin_groups={:?}/matrix={:?}/dynamic={:?}/fallback={:?}/stages={}/active={}/lifecycle={:?}",
                index,
                record.plugin_type_id,
                record.plugin_id,
                record.pin_group_identities,
                record.pin_matrix_posture,
                record.dynamic_bus_negotiation_posture,
                record.fallback_outcome,
                record.stage_count,
                record.active_stage_count,
                record.strongest_lifecycle_state,
            )
        })
        .collect::<String>();
    format!(
        "\nplugin_pin_matrix_type_count={}\nplugin_pin_matrix_negotiated_type_count={}\nplugin_pin_matrix_guarded_type_count={}\nplugin_pin_matrix_unavailable_type_count={}\nplugin_pin_matrix_dynamic_negotiated_type_count={}\nplugin_pin_matrix_dynamic_guarded_type_count={}\nplugin_pin_matrix_summary={}{}",
        snapshot.plugin_type_count,
        snapshot.negotiated_type_count,
        snapshot.guarded_type_count,
        snapshot.unavailable_type_count,
        snapshot.dynamic_negotiated_type_count,
        snapshot.dynamic_guarded_type_count,
        snapshot.summary,
        record_lines,
    )
}
