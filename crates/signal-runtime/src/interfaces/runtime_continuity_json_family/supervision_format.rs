use super::*;

pub(crate) fn format_runtime_device_supervision_snapshot_compact(
    snapshot: &RuntimeDeviceSupervisionSnapshot,
) -> String {
    format!(
        " device_supervision={:?}/{:?}/{:?} recovery={:?} interruption={:?} primary={:?} safe_mode={} device_loss_active={} device_losses={} restart_attempts={:?} restart_failures={:?} watchdog_restarts={}",
        snapshot.state,
        snapshot.restart_state,
        snapshot.fault_boundary,
        snapshot.recovery_state,
        snapshot.interruption_class,
        snapshot.primary_fault_cause,
        snapshot.safe_mode_enabled,
        snapshot.device_loss_active,
        snapshot.device_loss_count,
        snapshot.restart_attempt_count,
        snapshot.restart_failure_count,
        snapshot.watchdog_restart_count,
    )
}

pub(crate) fn format_runtime_device_supervision_snapshot_multiline(
    snapshot: &RuntimeDeviceSupervisionSnapshot,
) -> String {
    format!(
        concat!(
            "\ndevice_supervision_state={:?}",
            "\ndevice_supervision_restart_state={:?}",
            "\ndevice_supervision_fault_boundary={:?}",
            "\ndevice_supervision_recovery_state={:?}",
            "\ndevice_supervision_interruption_class={:?}",
            "\ndevice_supervision_primary_fault_cause={:?}",
            "\ndevice_supervision_safe_mode_enabled={}",
            "\ndevice_supervision_device_loss_active={}",
            "\ndevice_supervision_active_output_device={:?}",
            "\ndevice_supervision_device_id={:?}",
            "\ndevice_supervision_device_name={:?}",
            "\ndevice_supervision_restart_policy={:?}",
            "\ndevice_supervision_backend_health={:?}",
            "\ndevice_supervision_stream_state={:?}",
            "\ndevice_supervision_device_loss_count={}",
            "\ndevice_supervision_restart_attempt_count={:?}",
            "\ndevice_supervision_restart_failure_count={:?}",
            "\ndevice_supervision_watchdog_restart_count={}",
            "\ndevice_supervision_last_watchdog_trigger={:?}",
            "\ndevice_supervision_summary={}",
        ),
        snapshot.state,
        snapshot.restart_state,
        snapshot.fault_boundary,
        snapshot.recovery_state,
        snapshot.interruption_class,
        snapshot.primary_fault_cause,
        snapshot.safe_mode_enabled,
        snapshot.device_loss_active,
        snapshot.active_output_device,
        snapshot.device_id,
        snapshot.device_name,
        snapshot.restart_policy,
        snapshot.backend_health,
        snapshot.stream_state,
        snapshot.device_loss_count,
        snapshot.restart_attempt_count,
        snapshot.restart_failure_count,
        snapshot.watchdog_restart_count,
        snapshot.last_watchdog_trigger,
        snapshot.summary,
    )
}
