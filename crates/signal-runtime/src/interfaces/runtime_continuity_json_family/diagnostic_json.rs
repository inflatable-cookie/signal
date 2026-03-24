use super::*;

pub(crate) fn json_runtime_degradation_summary(summary: &RuntimeDegradationSummary) -> String {
    format!(
        concat!(
            "{{",
            "\"readiness_degraded\":{},",
            "\"safe_mode_enabled\":{},",
            "\"xrun_count\":{},",
            "\"plugin_fault_count\":{},",
            "\"transport_fault_event_count\":{},",
            "\"broker_failure_event_count\":{},",
            "\"sandbox_operation_failure_event_count\":{},",
            "\"recovery_event_count\":{},",
            "\"active_plugin_sandboxes\":{},",
            "\"recovery_overlap_sessions\":{},",
            "\"lingering_sessions\":{},",
            "\"degraded_bound_plugin_sandboxes\":{},",
            "\"missing_bound_plugin_sandboxes\":{},",
            "\"detach_faulted_sessions\":{},",
            "\"transport_gate_active\":{},",
            "\"plugin_gate_active\":{},",
            "\"last_watchdog_trigger\":{}",
            "}}"
        ),
        summary.readiness_degraded,
        summary.safe_mode_enabled,
        summary.xrun_count,
        summary.plugin_fault_count,
        summary.transport_fault_event_count,
        summary.broker_failure_event_count,
        summary.sandbox_operation_failure_event_count,
        summary.recovery_event_count,
        summary.active_plugin_sandboxes,
        summary.recovery_overlap_sessions,
        summary.lingering_sessions,
        summary.degraded_bound_plugin_sandboxes,
        summary.missing_bound_plugin_sandboxes,
        summary.detach_faulted_sessions,
        summary.transport_gate_active,
        summary.plugin_gate_active,
        json_option_string(
            summary
                .last_watchdog_trigger
                .map(|value| format!("{value:?}"))
                .as_deref()
        ),
    )
}

pub(crate) fn json_runtime_fault_status(snapshot: &RuntimeFaultStatusSnapshot) -> String {
    format!(
        concat!(
            "{{",
            "\"recovery_state\":{},",
            "\"primary_fault_cause\":{},",
            "\"active_fault_count\":{},",
            "\"xrun_overload_active\":{},",
            "\"plugin_fault_active\":{},",
            "\"watchdog_active\":{},",
            "\"device_loss_active\":{},",
            "\"transport_fault_active\":{},",
            "\"missing_plugin_binding_active\":{},",
            "\"safe_mode_enabled\":{},",
            "\"restart_count\":{},",
            "\"watchdog_restart_count\":{},",
            "\"plugin_fault_count\":{},",
            "\"transport_faulted_session_count\":{},",
            "\"device_loss_count\":{},",
            "\"summary\":{}",
            "}}"
        ),
        json_string(&format!("{:?}", snapshot.recovery_state)),
        json_option_string(
            snapshot
                .primary_fault_cause
                .map(|value| format!("{value:?}"))
                .as_deref(),
        ),
        snapshot.active_fault_count,
        snapshot.xrun_overload_active,
        snapshot.plugin_fault_active,
        snapshot.watchdog_active,
        snapshot.device_loss_active,
        snapshot.transport_fault_active,
        snapshot.missing_plugin_binding_active,
        snapshot.safe_mode_enabled,
        snapshot.restart_count,
        snapshot.watchdog_restart_count,
        snapshot.plugin_fault_count,
        snapshot.transport_faulted_session_count,
        snapshot.device_loss_count,
        json_option_string(Some(snapshot.summary.as_str())),
    )
}

pub(crate) fn json_runtime_fault_contribution_receipt(
    receipt: &RuntimeFaultContributionReceipt,
) -> String {
    format!(
        concat!(
            "{{",
            "\"family\":{},",
            "\"authority\":{},",
            "\"active\":{},",
            "\"event_count\":{},",
            "\"detail\":{},",
            "\"summary\":{}",
            "}}"
        ),
        json_string(&format!("{:?}", receipt.family)),
        json_string(&format!("{:?}", receipt.authority)),
        receipt.active,
        receipt.event_count,
        json_option_string(receipt.detail.as_deref()),
        json_option_string(Some(receipt.summary.as_str())),
    )
}

pub(crate) fn json_runtime_fault_diagnostic_receipt(
    receipt: &RuntimeFaultDiagnosticReceipt,
) -> String {
    let contributions = receipt
        .contributions
        .iter()
        .map(json_runtime_fault_contribution_receipt)
        .collect::<Vec<_>>()
        .join(",");
    format!(
        concat!(
            "{{",
            "\"primary_family\":{},",
            "\"primary_fault_cause\":{},",
            "\"interruption_class\":{},",
            "\"recovery_state\":{},",
            "\"safe_mode_enabled\":{},",
            "\"rebindable\":{},",
            "\"contributions\":[{}],",
            "\"summary\":{}",
            "}}"
        ),
        json_option_string(
            receipt
                .primary_family
                .map(|value| format!("{value:?}"))
                .as_deref()
        ),
        json_option_string(
            receipt
                .primary_fault_cause
                .map(|value| format!("{value:?}"))
                .as_deref()
        ),
        json_string(&format!("{:?}", receipt.interruption_class)),
        json_string(&format!("{:?}", receipt.recovery_state)),
        receipt.safe_mode_enabled,
        receipt.rebindable,
        contributions,
        json_option_string(Some(receipt.summary.as_str())),
    )
}
