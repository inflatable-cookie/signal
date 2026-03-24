use super::*;

fn json_runtime_plugin_pin_matrix_record(record: &RuntimePluginPinMatrixRecord) -> String {
    format!(
        concat!(
            "{{",
            "\"plugin_type_id\":{},",
            "\"plugin_id\":{},",
            "\"pin_group_identities\":{},",
            "\"pin_matrix_posture\":{},",
            "\"dynamic_bus_negotiation_posture\":{},",
            "\"fallback_outcome\":{},",
            "\"strongest_lifecycle_state\":{},",
            "\"stage_count\":{},",
            "\"active_stage_count\":{},",
            "\"summary\":{}",
            "}}"
        ),
        json_option_string(Some(record.plugin_type_id.as_str())),
        json_option_string(Some(record.plugin_id.as_str())),
        json_runtime_plugin_pin_group_identity_vec(&record.pin_group_identities),
        json_string(&format!("{:?}", record.pin_matrix_posture)),
        json_string(&format!("{:?}", record.dynamic_bus_negotiation_posture)),
        json_string(&format!("{:?}", record.fallback_outcome)),
        json_option_string(
            record
                .strongest_lifecycle_state
                .map(|state| format!("{state:?}"))
                .as_deref(),
        ),
        record.stage_count,
        record.active_stage_count,
        json_option_string(Some(record.summary.as_str())),
    )
}

pub(crate) fn json_runtime_plugin_pin_matrix_snapshot(
    snapshot: &RuntimePluginPinMatrixSnapshot,
) -> String {
    format!(
        concat!(
            "{{",
            "\"plugin_type_count\":{},",
            "\"negotiated_type_count\":{},",
            "\"guarded_type_count\":{},",
            "\"unavailable_type_count\":{},",
            "\"dynamic_negotiated_type_count\":{},",
            "\"dynamic_guarded_type_count\":{},",
            "\"records\":{},",
            "\"summary\":{}",
            "}}"
        ),
        snapshot.plugin_type_count,
        snapshot.negotiated_type_count,
        snapshot.guarded_type_count,
        snapshot.unavailable_type_count,
        snapshot.dynamic_negotiated_type_count,
        snapshot.dynamic_guarded_type_count,
        format!(
            "[{}]",
            snapshot
                .records
                .iter()
                .map(json_runtime_plugin_pin_matrix_record)
                .collect::<Vec<_>>()
                .join(",")
        ),
        json_option_string(Some(snapshot.summary.as_str())),
    )
}

fn json_runtime_plugin_sandbox_snapshot(snapshot: &RuntimePluginSandboxSnapshot) -> String {
    let plugin_format = snapshot.plugin_format.map(|format| format!("{format:?}"));
    let state = format!("{:?}", snapshot.state);
    let continuity_class = format!("{:?}", snapshot.continuity_class);
    let lifecycle_stage = snapshot.lifecycle_stage.map(|stage| format!("{stage:?}"));
    let transport_stage = snapshot.transport_stage.map(|stage| format!("{stage:?}"));
    let last_fault_kind = snapshot.last_fault_kind.map(|kind| format!("{kind:?}"));
    let last_restart_intent = snapshot
        .last_restart_intent
        .map(|intent| format!("{intent:?}"));
    let last_stop_reason = snapshot
        .last_stop_reason
        .map(|reason| format!("{reason:?}"));
    format!(
        concat!(
            "{{",
            "\"sandbox_id\":{},",
            "\"sandbox_group_key\":{},",
            "\"plugin_type_id\":{},",
            "\"plugin_format\":{},",
            "\"instance_id\":{},",
            "\"preset_descriptor\":{},",
            "\"ara_context\":{},",
            "\"placement_outcome\":\"{:?}\",",
            "\"placement_rule_id\":{},",
            "\"shared_boundary_member_count\":{},",
            "\"continuity_class\":{},",
            "\"rebindable\":{},",
            "\"state\":{},",
            "\"lifecycle_stage\":{},",
            "\"transport_stage\":{},",
            "\"active\":{},",
            "\"active_transport\":{},",
            "\"restart_count\":{},",
            "\"recovery_count\":{},",
            "\"fault_count\":{},",
            "\"last_fault_kind\":{},",
            "\"last_fault_detail\":{},",
            "\"last_restart_intent\":{},",
            "\"last_stop_reason\":{},",
            "\"last_processing_epoch\":{},",
            "\"readiness_state\":{},",
            "\"degraded_reasons\":{},",
            "\"active_lease_id\":{},",
            "\"active_region_id\":{},",
            "\"summary\":{}",
            "}}"
        ),
        json_option_string(Some(snapshot.sandbox_id.as_str())),
        json_option_string(Some(snapshot.sandbox_group_key.as_str())),
        json_option_string(snapshot.plugin_type_id.as_deref()),
        json_option_string(plugin_format.as_deref()),
        json_option_string(snapshot.instance_id.as_deref()),
        snapshot
            .preset_descriptor
            .as_ref()
            .map_or_else(|| "null".into(), json_runtime_plugin_preset_descriptor),
        snapshot
            .ara_context
            .as_ref()
            .map_or_else(|| "null".into(), json_runtime_plugin_ara_context_snapshot),
        snapshot.placement_outcome,
        json_option_string(snapshot.placement_rule_id.as_deref()),
        snapshot.shared_boundary_member_count,
        json_option_string(Some(continuity_class.as_str())),
        snapshot.rebindable,
        json_option_string(Some(state.as_str())),
        json_option_string(lifecycle_stage.as_deref()),
        json_option_string(transport_stage.as_deref()),
        snapshot.active,
        snapshot.active_transport,
        snapshot.restart_count,
        snapshot.recovery_count,
        snapshot.fault_count,
        json_option_string(last_fault_kind.as_deref()),
        json_option_string(snapshot.last_fault_detail.as_deref()),
        json_option_string(last_restart_intent.as_deref()),
        json_option_string(last_stop_reason.as_deref()),
        json_option_u64(snapshot.last_processing_epoch),
        json_option_string(snapshot.readiness_state.as_deref()),
        json_string_vec(&snapshot.degraded_reasons),
        json_option_string(snapshot.active_lease_id.as_deref()),
        json_option_string(snapshot.active_region_id.as_deref()),
        json_option_string(Some(snapshot.summary.as_str())),
    )
}

fn json_runtime_plugin_sandbox_snapshot_vec(sandboxes: &[RuntimePluginSandboxSnapshot]) -> String {
    format!(
        "[{}]",
        sandboxes
            .iter()
            .map(json_runtime_plugin_sandbox_snapshot)
            .collect::<Vec<_>>()
            .join(",")
    )
}

pub(crate) fn json_runtime_plugin_lifecycle_snapshot(
    snapshot: &RuntimePluginLifecycleSnapshot,
) -> String {
    format!(
        concat!(
            "{{",
            "\"sandbox_count\":{},",
            "\"active_sandbox_count\":{},",
            "\"shared_sandbox_count\":{},",
            "\"isolated_sandbox_count\":{},",
            "\"ready_sandbox_count\":{},",
            "\"booting_sandbox_count\":{},",
            "\"degraded_sandbox_count\":{},",
            "\"faulted_sandbox_count\":{},",
            "\"restarting_sandbox_count\":{},",
            "\"quarantined_sandbox_count\":{},",
            "\"stopped_sandbox_count\":{},",
            "\"rebindable_sandbox_count\":{},",
            "\"terminal_sandbox_count\":{},",
            "\"parity_coverage\":{},",
            "\"sandboxes\":{},",
            "\"summary\":{}",
            "}}"
        ),
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
        json_runtime_plugin_parity_coverage_vec(&snapshot.parity_coverage),
        json_runtime_plugin_sandbox_snapshot_vec(&snapshot.sandboxes),
        json_option_string(Some(snapshot.summary.as_str())),
    )
}
