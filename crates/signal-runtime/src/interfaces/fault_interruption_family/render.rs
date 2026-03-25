use super::*;

pub(crate) fn format_runtime_fault_status_compact(snapshot: &RuntimeFaultStatusSnapshot) -> String {
    format!(
        " fault_status={:?}/safe_mode={} active={} xrun={} plugin={} watchdog={} device_loss={} transport={} binding={} restarts={}/{} plugin_faults={} transport_faulted_sessions={} device_losses={}",
        snapshot.recovery_state,
        snapshot.safe_mode_enabled,
        snapshot.active_fault_count,
        snapshot.xrun_overload_active,
        snapshot.plugin_fault_active,
        snapshot.watchdog_active,
        snapshot.device_loss_active,
        snapshot.transport_fault_active,
        snapshot.missing_plugin_binding_active,
        snapshot.restart_count,
        snapshot.watchdog_restart_count,
        snapshot.plugin_fault_count,
        snapshot.transport_faulted_session_count,
        snapshot.device_loss_count,
    )
}

pub(crate) fn format_runtime_fault_diagnostic_receipt_compact(
    receipt: &RuntimeFaultDiagnosticReceipt,
) -> String {
    format!(
        " fault_diagnostic={:?}/{:?}/{:?} recovery={:?} safe_mode={} rebindable={} contributions={}",
        receipt.primary_family,
        receipt.primary_fault_cause,
        receipt.interruption_class,
        receipt.recovery_state,
        receipt.safe_mode_enabled,
        receipt.rebindable,
        receipt.contributions.len(),
    )
}

pub(crate) fn format_runtime_fault_status_multiline(
    snapshot: &RuntimeFaultStatusSnapshot,
) -> String {
    format!(
        concat!(
            "\nfault_status_recovery_state={:?}",
            "\nfault_status_primary_fault_cause={:?}",
            "\nfault_status_active_fault_count={}",
            "\nfault_status_xrun_overload_active={}",
            "\nfault_status_plugin_fault_active={}",
            "\nfault_status_watchdog_active={}",
            "\nfault_status_device_loss_active={}",
            "\nfault_status_transport_fault_active={}",
            "\nfault_status_missing_plugin_binding_active={}",
            "\nfault_status_safe_mode_enabled={}",
            "\nfault_status_restart_count={}",
            "\nfault_status_watchdog_restart_count={}",
            "\nfault_status_plugin_fault_count={}",
            "\nfault_status_transport_faulted_session_count={}",
            "\nfault_status_device_loss_count={}",
            "\nfault_status_summary={}",
        ),
        snapshot.recovery_state,
        snapshot.primary_fault_cause,
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
        snapshot.summary,
    )
}

pub(crate) fn format_runtime_fault_diagnostic_receipt_multiline(
    receipt: &RuntimeFaultDiagnosticReceipt,
) -> String {
    let contributions = receipt
        .contributions
        .iter()
        .map(|contribution| contribution.summary.as_str())
        .collect::<Vec<_>>()
        .join(" | ");
    format!(
        concat!(
            "\nfault_diagnostic_primary_family={:?}",
            "\nfault_diagnostic_primary_fault_cause={:?}",
            "\nfault_diagnostic_interruption_class={:?}",
            "\nfault_diagnostic_recovery_state={:?}",
            "\nfault_diagnostic_safe_mode_enabled={}",
            "\nfault_diagnostic_rebindable={}",
            "\nfault_diagnostic_contribution_count={}",
            "\nfault_diagnostic_contributions={}",
            "\nfault_diagnostic_summary={}",
        ),
        receipt.primary_family,
        receipt.primary_fault_cause,
        receipt.interruption_class,
        receipt.recovery_state,
        receipt.safe_mode_enabled,
        receipt.rebindable,
        receipt.contributions.len(),
        if contributions.is_empty() {
            "none"
        } else {
            contributions.as_str()
        },
        receipt.summary,
    )
}

pub(crate) fn format_runtime_interruption_summary_compact(
    summary: &RuntimeInterruptionSummary,
) -> String {
    format!(
        " interruption={:?}/active={} rebindable={} recovery={:?} primary={:?} deferred={:?}/{:?}",
        summary.class,
        summary.active,
        summary.rebindable,
        summary.recovery_state,
        summary.primary_fault_cause,
        summary.deferred_service_class,
        summary.deferred_service_decision,
    )
}

pub(crate) fn format_runtime_interruption_summary_multiline(
    summary: &RuntimeInterruptionSummary,
) -> String {
    format!(
        concat!(
            "\ninterruption_active={}",
            "\ninterruption_class={:?}",
            "\ninterruption_rebindable={}",
            "\ninterruption_recovery_state={:?}",
            "\ninterruption_primary_fault_cause={:?}",
            "\ninterruption_safe_mode_enabled={}",
            "\ninterruption_deferred_service_class={:?}",
            "\ninterruption_deferred_service_decision={:?}",
            "\ninterruption_summary={}",
        ),
        summary.active,
        summary.class,
        summary.rebindable,
        summary.recovery_state,
        summary.primary_fault_cause,
        summary.safe_mode_enabled,
        summary.deferred_service_class,
        summary.deferred_service_decision,
        summary.summary,
    )
}

pub(crate) fn format_runtime_degradation_summary_compact(
    summary: &RuntimeDegradationSummary,
) -> String {
    format!(
        " degradation_summary_state={}/{} degradation_summary_faults={}/{}/{}/{} degradation_summary_recovery={} degradation_summary_sessions={}/{}/{}/{}/{} degradation_summary_gates={}/{} degradation_summary_last_watchdog={:?}",
        summary.readiness_degraded,
        summary.safe_mode_enabled,
        summary.plugin_fault_count,
        summary.transport_fault_event_count,
        summary.broker_failure_event_count,
        summary.sandbox_operation_failure_event_count,
        summary.recovery_event_count,
        summary.recovery_overlap_sessions,
        summary.lingering_sessions,
        summary.degraded_bound_plugin_sandboxes,
        summary.missing_bound_plugin_sandboxes,
        summary.detach_faulted_sessions,
        summary.plugin_gate_active,
        summary.transport_gate_active,
        summary.last_watchdog_trigger,
    )
}

pub(crate) fn format_runtime_degradation_summary_multiline(
    summary: &RuntimeDegradationSummary,
) -> String {
    format!(
        "\ndegradation_summary_readiness_degraded={}\ndegradation_summary_safe_mode_enabled={}\ndegradation_summary_xruns={}\ndegradation_summary_plugin_faults={}\ndegradation_summary_transport_fault_events={}\ndegradation_summary_broker_failure_events={}\ndegradation_summary_sandbox_operation_failure_events={}\ndegradation_summary_recovery_events={}\ndegradation_summary_active_plugin_sandboxes={}\ndegradation_summary_recovery_overlap_sessions={}\ndegradation_summary_lingering_sessions={}\ndegradation_summary_degraded_bound_plugin_sandboxes={}\ndegradation_summary_missing_bound_plugin_sandboxes={}\ndegradation_summary_detach_faulted_sessions={}\ndegradation_summary_plugin_gate_active={}\ndegradation_summary_transport_gate_active={}\ndegradation_summary_last_watchdog_trigger={:?}",
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
        summary.plugin_gate_active,
        summary.transport_gate_active,
        summary.last_watchdog_trigger,
    )
}
