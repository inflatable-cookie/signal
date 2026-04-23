use super::*;

/// State of the audio device restart attempt.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RuntimeDeviceRestartState {
    /// No restart is needed; the device is healthy.
    #[default]
    Unneeded,
    /// A restart is in progress.
    Attempting,
    /// Device was successfully restarted.
    Recovered,
    /// All restart attempts have been exhausted.
    Exhausted,
    /// Device faulted and cannot be restarted.
    Faulted,
}

/// Fault boundary state for the audio hardware path.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RuntimeDeviceFaultBoundaryState {
    /// No fault boundary is active.
    #[default]
    Clear,
    /// A fault boundary is present but the device can be restarted.
    Restartable,
    /// Restart attempts exhausted; the boundary is at capacity.
    Exhausted,
    /// Unrecoverable fault boundary.
    Faulted,
}

/// Overall hardware supervision health.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RuntimeDeviceSupervisionState {
    /// Device is stable and healthy.
    #[default]
    Stable,
    /// Device is undergoing recovery.
    Recovering,
    /// Recovery exhausted; device is no longer usable.
    Exhausted,
    /// Device has faulted and cannot be recovered.
    Faulted,
}

/// Synthesised snapshot of hardware supervision state derived from host I/O and
/// fault status.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeDeviceSupervisionSnapshot {
    /// Overall supervision health.
    pub state: RuntimeDeviceSupervisionState,
    /// Restart attempt state.
    pub restart_state: RuntimeDeviceRestartState,
    /// Fault boundary state.
    pub fault_boundary: RuntimeDeviceFaultBoundaryState,
    /// Runtime recovery trajectory.
    pub recovery_state: RuntimeRecoveryState,
    /// Interruption class derived from this supervision state.
    pub interruption_class: RuntimeInterruptionClass,
    /// Primary fault cause if a fault is active.
    pub primary_fault_cause: Option<RuntimeFaultCause>,
    /// Whether safe mode is engaged.
    pub safe_mode_enabled: bool,
    /// Whether a device loss event is currently active.
    pub device_loss_active: bool,
    /// Configured active output device identifier, if known.
    pub active_output_device: Option<String>,
    /// Runtime device identifier, if available.
    pub device_id: Option<String>,
    /// Runtime device name, if available.
    pub device_name: Option<String>,
    /// Restart policy configured for the host, if known.
    pub restart_policy: Option<RuntimeHostRestartPolicy>,
    /// Backend health reported by the host, if available.
    pub backend_health: Option<BackendHealth>,
    /// Audio stream state reported by the host, if available.
    pub stream_state: Option<RuntimeHostAudioStreamState>,
    /// Total number of device-loss events since startup.
    pub device_loss_count: u64,
    /// Total number of device restart attempts, if the host reports it.
    pub restart_attempt_count: Option<u64>,
    /// Total number of failed restart attempts, if the host reports it.
    pub restart_failure_count: Option<u64>,
    /// Total watchdog-triggered restarts since startup.
    pub watchdog_restart_count: u32,
    /// What triggered the last watchdog event, if any.
    pub last_watchdog_trigger: Option<RuntimeWatchdogTrigger>,
    /// Human-readable summary of this snapshot.
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
    /// Captures a hardware supervision snapshot from the current supervision, fault, and host I/O state.
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
