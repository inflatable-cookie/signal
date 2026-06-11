use signal_hardware::{HardwareConfigRequest, HardwareNegotiationError, HardwareStreamConfig};
use signal_runtime::{
    BackendPolicyOverride, RuntimeError, RuntimeProjectionApi, RuntimeSupervisorApi,
};

use super::super::{FaultInjection, LocalRuntimeHost, LocalRuntimeHostSummary};

impl LocalRuntimeHost {
    pub(crate) fn prepare_default_output_hardware(
        &mut self,
    ) -> Result<HardwareStreamConfig, RuntimeError> {
        let stream = self
            .hardware
            .default_output_stream(
                self.runtime.config().sample_rate.0,
                self.runtime.config().graph.block_size,
            )
            .map_err(Self::runtime_error_from_hardware_negotiation)?;
        let hardware_request =
            HardwareConfigRequest::from_stream(&stream, self.hardware.policy_tier());
        self.runtime.apply_hardware_config(hardware_request)?;
        self.runtime
            .set_active_output_device(stream.device.device_id.clone());
        self.set_backend_policy(BackendPolicyOverride {
            tier: hardware_request.backend_policy,
        })?;
        self.runtime
            .set_backend_policy_tier(hardware_request.backend_policy);
        self.audio_pump.reset_for_stream(&stream);
        self.active_output_stream = Some(stream.clone());
        Ok(stream)
    }

    pub(crate) fn runtime_error_from_hardware_negotiation(
        error: HardwareNegotiationError,
    ) -> RuntimeError {
        RuntimeError::new(
            signal_runtime::RuntimeErrorKind::InvalidRequest,
            format!("hardware negotiation failed: {}", error.message),
        )
    }

    /// Boots the local host with no fault injection.
    pub fn boot_default(&mut self) -> Result<LocalRuntimeHostSummary, RuntimeError> {
        self.boot_with_fault_recovery(None)
    }

    /// Boots the local host and exercises sandbox crash recovery.
    pub fn boot_with_crash_recovery(&mut self) -> Result<LocalRuntimeHostSummary, RuntimeError> {
        self.boot_with_fault_recovery(Some(FaultInjection::Crash))
    }

    /// Boots the local host and exercises deferred recovery teardown failure handling.
    pub fn boot_with_recovery_deferred_teardown_failure(
        &mut self,
    ) -> Result<LocalRuntimeHostSummary, RuntimeError> {
        self.boot_with_fault_recovery(Some(FaultInjection::RecoveryDeferredTeardownFailure))
    }

    /// Boots the local host and exercises deferred teardown cleanup retry handling.
    pub fn boot_with_recovery_deferred_teardown_cleanup_retry(
        &mut self,
    ) -> Result<LocalRuntimeHostSummary, RuntimeError> {
        self.boot_with_fault_recovery(Some(FaultInjection::RecoveryDeferredTeardownCleanupRetry))
    }

    /// Boots the local host and exercises recovery overlap contention handling.
    pub fn boot_with_recovery_overlap_contention(
        &mut self,
    ) -> Result<LocalRuntimeHostSummary, RuntimeError> {
        self.boot_with_fault_recovery(Some(FaultInjection::RecoveryOverlapContention))
    }

    /// Boots the local host and exercises audio device loss recovery.
    pub fn boot_with_device_loss_recovery(
        &mut self,
    ) -> Result<LocalRuntimeHostSummary, RuntimeError> {
        self.boot_with_fault_recovery(Some(FaultInjection::DeviceLoss))
    }

    /// Boots the local host and exercises audio device loss followed by restart failure.
    pub fn boot_with_device_loss_restart_failure(
        &mut self,
    ) -> Result<LocalRuntimeHostSummary, RuntimeError> {
        self.boot_with_fault_recovery(Some(FaultInjection::DeviceLossRestartFailure))
    }
}

/// Fault-injection boot entry points consumed only by this crate's unit tests.
#[cfg(test)]
impl LocalRuntimeHost {
    /// Boots the local host and exercises sandbox timeout recovery.
    pub(crate) fn boot_with_timeout_recovery(
        &mut self,
    ) -> Result<LocalRuntimeHostSummary, RuntimeError> {
        self.boot_with_fault_recovery(Some(FaultInjection::Timeout))
    }

    /// Boots the local host and exercises heartbeat-miss watchdog recovery.
    pub(crate) fn boot_with_heartbeat_miss_recovery(
        &mut self,
    ) -> Result<LocalRuntimeHostSummary, RuntimeError> {
        self.boot_with_fault_recovery(Some(FaultInjection::HeartbeatMiss))
    }

    /// Boots the local host and exercises recovery teardown failure handling.
    pub(crate) fn boot_with_recovery_teardown_failure(
        &mut self,
    ) -> Result<LocalRuntimeHostSummary, RuntimeError> {
        self.boot_with_fault_recovery(Some(FaultInjection::RecoveryTeardownFailure))
    }

    /// Boots the local host and exercises deferred teardown followed by cleanup.
    pub(crate) fn boot_with_recovery_deferred_teardown_then_cleanup(
        &mut self,
    ) -> Result<LocalRuntimeHostSummary, RuntimeError> {
        self.boot_with_fault_recovery(Some(FaultInjection::RecoveryDeferredTeardownThenCleanup))
    }

    /// Boots the local host and exercises recovery restart failure handling.
    pub(crate) fn boot_with_recovery_restart_failure(
        &mut self,
    ) -> Result<LocalRuntimeHostSummary, RuntimeError> {
        self.boot_with_fault_recovery(Some(FaultInjection::RecoveryRestartFailure))
    }

    /// Boots the local host and exercises interleaved failure recovery handling.
    pub(crate) fn boot_with_recovery_interleaved_failures(
        &mut self,
    ) -> Result<LocalRuntimeHostSummary, RuntimeError> {
        self.boot_with_fault_recovery(Some(FaultInjection::RecoveryInterleavedFailures))
    }

    /// Boots the local host and exercises two rounds of escalating heartbeat-miss recovery.
    pub(crate) fn boot_with_escalating_heartbeat_failures(
        &mut self,
    ) -> Result<LocalRuntimeHostSummary, RuntimeError> {
        self.boot_with_fault_recovery(Some(FaultInjection::EscalatingHeartbeatMisses {
            restart_episodes: 2,
        }))
    }

    /// Boots the local host and runs a full soak of escalating heartbeat-miss recovery episodes.
    pub(crate) fn boot_with_watchdog_soak(
        &mut self,
    ) -> Result<LocalRuntimeHostSummary, RuntimeError> {
        self.boot_with_fault_recovery(Some(FaultInjection::EscalatingHeartbeatMisses {
            restart_episodes: super::SOAK_RESTART_EPISODES,
        }))
    }

    /// Boots the local host and runs a full soak of mixed watchdog recovery episodes.
    pub(crate) fn boot_with_mixed_watchdog_soak(
        &mut self,
    ) -> Result<LocalRuntimeHostSummary, RuntimeError> {
        self.boot_with_fault_recovery(Some(FaultInjection::MixedWatchdogEpisodes {
            restart_episodes: super::SOAK_RESTART_EPISODES,
        }))
    }
}
