use super::*;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RuntimeDeviceRestartState {
    #[default]
    Unneeded,
    Attempting,
    Recovered,
    Exhausted,
    Faulted,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RuntimeDeviceFaultBoundaryState {
    #[default]
    Clear,
    Restartable,
    Exhausted,
    Faulted,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RuntimeDeviceSupervisionState {
    #[default]
    Stable,
    Recovering,
    Exhausted,
    Faulted,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeDeviceSupervisionSnapshot {
    pub state: RuntimeDeviceSupervisionState,
    pub restart_state: RuntimeDeviceRestartState,
    pub fault_boundary: RuntimeDeviceFaultBoundaryState,
    pub recovery_state: RuntimeRecoveryState,
    pub interruption_class: RuntimeInterruptionClass,
    pub primary_fault_cause: Option<RuntimeFaultCause>,
    pub safe_mode_enabled: bool,
    pub device_loss_active: bool,
    pub active_output_device: Option<String>,
    pub device_id: Option<String>,
    pub device_name: Option<String>,
    pub restart_policy: Option<RuntimeHostRestartPolicy>,
    pub backend_health: Option<BackendHealth>,
    pub stream_state: Option<RuntimeHostAudioStreamState>,
    pub device_loss_count: u64,
    pub restart_attempt_count: Option<u64>,
    pub restart_failure_count: Option<u64>,
    pub watchdog_restart_count: u32,
    pub last_watchdog_trigger: Option<RuntimeWatchdogTrigger>,
    pub summary: String,
}

impl Default for RuntimeDeviceSupervisionSnapshot {
    fn default() -> Self {
        Self {
            state: RuntimeDeviceSupervisionState::Stable,
            restart_state: RuntimeDeviceRestartState::Unneeded,
            fault_boundary: RuntimeDeviceFaultBoundaryState::Clear,
            recovery_state: RuntimeRecoveryState::Steady,
            interruption_class: RuntimeInterruptionClass::Steady,
            primary_fault_cause: None,
            safe_mode_enabled: false,
            device_loss_active: false,
            active_output_device: None,
            device_id: None,
            device_name: None,
            restart_policy: None,
            backend_health: None,
            stream_state: None,
            device_loss_count: 0,
            restart_attempt_count: None,
            restart_failure_count: None,
            watchdog_restart_count: 0,
            last_watchdog_trigger: None,
            summary: "state=Stable restart_state=Unneeded fault_boundary=Clear interruption_class=Steady recovery_state=Steady device_loss_active=false safe_mode=false device_loss_count=0".to_string(),
        }
    }
}

impl RuntimeDeviceSupervisionSnapshot {
    pub fn capture(
        effective_config: &EffectiveRuntimeConfig,
        supervision_snapshot: &RuntimeSupervisionSnapshot,
        fault_status: &RuntimeFaultStatusSnapshot,
        interruption_summary: &RuntimeInterruptionSummary,
        host_io: Option<&RuntimeHostIoSummary>,
    ) -> Self {
        let device_loss_count = host_io
            .map(|host_io| host_io.hardware.device_loss_count)
            .unwrap_or(fault_status.device_loss_count);
        let restart_attempt_count = host_io.map(|host_io| host_io.hardware.restart_attempt_count);
        let restart_failure_count = host_io.map(|host_io| host_io.hardware.restart_failure_count);
        let primary_fault_cause = fault_status.primary_fault_cause;

        let host_reports_restart_failure = restart_failure_count.unwrap_or(0) > 0;
        let host_reports_device_restart_boundary = host_io
            .map(|host_io| {
                host_io.hardware.device_loss_count > 0
                    || host_io.hardware.backend_health == BackendHealth::Degraded
                    || host_io.audio_pump.stream_state == RuntimeHostAudioStreamState::Faulted
            })
            .unwrap_or(false);

        let restart_state = if fault_status.recovery_state == RuntimeRecoveryState::Faulted {
            RuntimeDeviceRestartState::Faulted
        } else if host_reports_restart_failure
            && (matches!(primary_fault_cause, Some(RuntimeFaultCause::DeviceLoss))
                || fault_status.device_loss_active
                || device_loss_count > 0
                || host_reports_device_restart_boundary)
        {
            RuntimeDeviceRestartState::Exhausted
        } else if fault_status.device_loss_active
            || host_io
                .map(|host_io| {
                    host_io.hardware.backend_health == BackendHealth::Recovering
                        || host_io.audio_pump.stream_state == RuntimeHostAudioStreamState::Faulted
                })
                .unwrap_or(false)
        {
            RuntimeDeviceRestartState::Attempting
        } else if device_loss_count > 0
            || restart_attempt_count.unwrap_or(0) > 0
            || supervision_snapshot.watchdog_restart_count > 0
        {
            RuntimeDeviceRestartState::Recovered
        } else {
            RuntimeDeviceRestartState::Unneeded
        };

        let state = match restart_state {
            RuntimeDeviceRestartState::Faulted => RuntimeDeviceSupervisionState::Faulted,
            RuntimeDeviceRestartState::Exhausted => RuntimeDeviceSupervisionState::Exhausted,
            RuntimeDeviceRestartState::Attempting => RuntimeDeviceSupervisionState::Recovering,
            RuntimeDeviceRestartState::Recovered
                if supervision_snapshot.safe_mode_enabled
                    || interruption_summary.active
                    || fault_status.device_loss_active =>
            {
                RuntimeDeviceSupervisionState::Recovering
            }
            RuntimeDeviceRestartState::Recovered | RuntimeDeviceRestartState::Unneeded => {
                RuntimeDeviceSupervisionState::Stable
            }
        };

        let fault_boundary = match restart_state {
            RuntimeDeviceRestartState::Faulted => RuntimeDeviceFaultBoundaryState::Faulted,
            RuntimeDeviceRestartState::Exhausted => RuntimeDeviceFaultBoundaryState::Exhausted,
            RuntimeDeviceRestartState::Attempting
                if matches!(primary_fault_cause, Some(RuntimeFaultCause::DeviceLoss))
                    || fault_status.device_loss_active =>
            {
                RuntimeDeviceFaultBoundaryState::Restartable
            }
            _ => RuntimeDeviceFaultBoundaryState::Clear,
        };

        let mut snapshot = Self {
            state,
            restart_state,
            fault_boundary,
            recovery_state: fault_status.recovery_state,
            interruption_class: interruption_summary.class,
            primary_fault_cause,
            safe_mode_enabled: supervision_snapshot.safe_mode_enabled,
            device_loss_active: fault_status.device_loss_active,
            active_output_device: effective_config.active_output_device.clone(),
            device_id: host_io.map(|host_io| host_io.hardware.device_id.clone()),
            device_name: host_io.map(|host_io| host_io.hardware.device_name.clone()),
            restart_policy: host_io.map(|host_io| host_io.clocking.restart_policy),
            backend_health: host_io.map(|host_io| host_io.hardware.backend_health),
            stream_state: host_io.map(|host_io| host_io.audio_pump.stream_state),
            device_loss_count,
            restart_attempt_count,
            restart_failure_count,
            watchdog_restart_count: supervision_snapshot.watchdog_restart_count,
            last_watchdog_trigger: supervision_snapshot.last_watchdog_trigger,
            summary: String::new(),
        };
        snapshot.summary = format!(
            "state={:?} restart={:?} boundary={:?} recovery={:?} interruption={:?} primary={:?} safe_mode={} device_loss_active={} device_losses={} restart_attempts={:?} restart_failures={:?} watchdog_restarts={} backend_health={:?} stream_state={:?}",
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
            snapshot.backend_health,
            snapshot.stream_state,
        );
        snapshot
    }
}
