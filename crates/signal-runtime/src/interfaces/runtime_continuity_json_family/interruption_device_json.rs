use super::*;

pub(crate) fn json_runtime_interruption_summary(summary: &RuntimeInterruptionSummary) -> String {
    format!(
        concat!(
            "{{",
            "\"active\":{},",
            "\"class\":{},",
            "\"rebindable\":{},",
            "\"recovery_state\":{},",
            "\"primary_fault_cause\":{},",
            "\"safe_mode_enabled\":{},",
            "\"deferred_service_class\":{},",
            "\"deferred_service_decision\":{},",
            "\"summary\":{}",
            "}}"
        ),
        summary.active,
        json_string(&format!("{:?}", summary.class)),
        summary.rebindable,
        json_string(&format!("{:?}", summary.recovery_state)),
        json_option_string(
            summary
                .primary_fault_cause
                .map(|value| format!("{value:?}"))
                .as_deref(),
        ),
        summary.safe_mode_enabled,
        json_option_string(
            summary
                .deferred_service_class
                .map(|value| format!("{value:?}"))
                .as_deref(),
        ),
        json_option_string(
            summary
                .deferred_service_decision
                .map(|value| format!("{value:?}"))
                .as_deref(),
        ),
        json_option_string(Some(summary.summary.as_str())),
    )
}

pub(crate) fn json_runtime_device_supervision_snapshot(
    snapshot: &RuntimeDeviceSupervisionSnapshot,
) -> String {
    format!(
        concat!(
            "{{",
            "\"state\":{},",
            "\"restart_state\":{},",
            "\"fault_boundary\":{},",
            "\"recovery_state\":{},",
            "\"interruption_class\":{},",
            "\"primary_fault_cause\":{},",
            "\"safe_mode_enabled\":{},",
            "\"device_loss_active\":{},",
            "\"active_output_device\":{},",
            "\"device_id\":{},",
            "\"device_name\":{},",
            "\"restart_policy\":{},",
            "\"backend_health\":{},",
            "\"stream_state\":{},",
            "\"device_loss_count\":{},",
            "\"restart_attempt_count\":{},",
            "\"restart_failure_count\":{},",
            "\"watchdog_restart_count\":{},",
            "\"last_watchdog_trigger\":{},",
            "\"summary\":{}",
            "}}"
        ),
        json_string(&format!("{:?}", snapshot.state)),
        json_string(&format!("{:?}", snapshot.restart_state)),
        json_string(&format!("{:?}", snapshot.fault_boundary)),
        json_string(&format!("{:?}", snapshot.recovery_state)),
        json_string(&format!("{:?}", snapshot.interruption_class)),
        json_option_string(
            snapshot
                .primary_fault_cause
                .map(|value| format!("{value:?}"))
                .as_deref(),
        ),
        snapshot.safe_mode_enabled,
        snapshot.device_loss_active,
        json_option_string(snapshot.active_output_device.as_deref()),
        json_option_string(snapshot.device_id.as_deref()),
        json_option_string(snapshot.device_name.as_deref()),
        json_option_string(
            snapshot
                .restart_policy
                .map(|value| format!("{value:?}"))
                .as_deref(),
        ),
        json_option_string(
            snapshot
                .backend_health
                .map(|value| format!("{value:?}"))
                .as_deref(),
        ),
        json_option_string(
            snapshot
                .stream_state
                .map(|value| format!("{value:?}"))
                .as_deref(),
        ),
        snapshot.device_loss_count,
        json_option_u64(snapshot.restart_attempt_count),
        json_option_u64(snapshot.restart_failure_count),
        snapshot.watchdog_restart_count,
        json_option_string(
            snapshot
                .last_watchdog_trigger
                .map(|value| format!("{value:?}"))
                .as_deref(),
        ),
        json_option_string(Some(snapshot.summary.as_str())),
    )
}
