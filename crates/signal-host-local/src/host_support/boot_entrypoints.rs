use signal_hardware::{HardwareConfigRequest, HardwareNegotiationError, HardwareStreamConfig};
use signal_runtime::{
    BackendPolicyOverride, RuntimeError, RuntimeProjectionApi, RuntimeSupervisorApi,
};

use super::super::{
    FaultInjection, LocalRuntimeHost, LocalRuntimeHostSummary, SOAK_RESTART_EPISODES,
};

impl LocalRuntimeHost {
    pub(crate) fn prepare_default_output_hardware(
        &mut self,
    ) -> Result<HardwareStreamConfig, RuntimeError> {
        let stream = self
            .coreaudio
            .default_output_stream(
                self.runtime.config().sample_rate.0,
                self.runtime.config().graph.block_size,
            )
            .map_err(Self::runtime_error_from_hardware_negotiation)?;
        let hardware_request =
            HardwareConfigRequest::from_stream(&stream, self.coreaudio.policy_tier());
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

    pub fn boot_default(&mut self) -> Result<LocalRuntimeHostSummary, RuntimeError> {
        self.boot_with_fault_recovery(None)
    }

    #[cfg_attr(not(test), allow(dead_code))]
    pub fn boot_with_timeout_recovery(&mut self) -> Result<LocalRuntimeHostSummary, RuntimeError> {
        self.boot_with_fault_recovery(Some(FaultInjection::Timeout))
    }

    #[cfg_attr(not(test), allow(dead_code))]
    pub fn boot_with_crash_recovery(&mut self) -> Result<LocalRuntimeHostSummary, RuntimeError> {
        self.boot_with_fault_recovery(Some(FaultInjection::Crash))
    }

    #[cfg_attr(not(test), allow(dead_code))]
    pub fn boot_with_heartbeat_miss_recovery(
        &mut self,
    ) -> Result<LocalRuntimeHostSummary, RuntimeError> {
        self.boot_with_fault_recovery(Some(FaultInjection::HeartbeatMiss))
    }

    #[cfg_attr(not(test), allow(dead_code))]
    pub fn boot_with_recovery_teardown_failure(
        &mut self,
    ) -> Result<LocalRuntimeHostSummary, RuntimeError> {
        self.boot_with_fault_recovery(Some(FaultInjection::RecoveryTeardownFailure))
    }

    #[cfg_attr(not(test), allow(dead_code))]
    pub fn boot_with_recovery_deferred_teardown_failure(
        &mut self,
    ) -> Result<LocalRuntimeHostSummary, RuntimeError> {
        self.boot_with_fault_recovery(Some(FaultInjection::RecoveryDeferredTeardownFailure))
    }

    #[cfg_attr(not(test), allow(dead_code))]
    pub fn boot_with_recovery_deferred_teardown_then_cleanup(
        &mut self,
    ) -> Result<LocalRuntimeHostSummary, RuntimeError> {
        self.boot_with_fault_recovery(Some(FaultInjection::RecoveryDeferredTeardownThenCleanup))
    }

    #[cfg_attr(not(test), allow(dead_code))]
    pub fn boot_with_recovery_deferred_teardown_cleanup_retry(
        &mut self,
    ) -> Result<LocalRuntimeHostSummary, RuntimeError> {
        self.boot_with_fault_recovery(Some(FaultInjection::RecoveryDeferredTeardownCleanupRetry))
    }

    #[cfg_attr(not(test), allow(dead_code))]
    pub fn boot_with_recovery_restart_failure(
        &mut self,
    ) -> Result<LocalRuntimeHostSummary, RuntimeError> {
        self.boot_with_fault_recovery(Some(FaultInjection::RecoveryRestartFailure))
    }

    #[cfg_attr(not(test), allow(dead_code))]
    pub fn boot_with_recovery_overlap_contention(
        &mut self,
    ) -> Result<LocalRuntimeHostSummary, RuntimeError> {
        self.boot_with_fault_recovery(Some(FaultInjection::RecoveryOverlapContention))
    }

    #[cfg_attr(not(test), allow(dead_code))]
    pub fn boot_with_recovery_interleaved_failures(
        &mut self,
    ) -> Result<LocalRuntimeHostSummary, RuntimeError> {
        self.boot_with_fault_recovery(Some(FaultInjection::RecoveryInterleavedFailures))
    }

    #[cfg_attr(not(test), allow(dead_code))]
    pub fn boot_with_escalating_heartbeat_failures(
        &mut self,
    ) -> Result<LocalRuntimeHostSummary, RuntimeError> {
        self.boot_with_fault_recovery(Some(FaultInjection::EscalatingHeartbeatMisses {
            restart_episodes: 2,
        }))
    }

    #[cfg_attr(not(test), allow(dead_code))]
    pub fn boot_with_watchdog_soak(&mut self) -> Result<LocalRuntimeHostSummary, RuntimeError> {
        self.boot_with_fault_recovery(Some(FaultInjection::EscalatingHeartbeatMisses {
            restart_episodes: SOAK_RESTART_EPISODES,
        }))
    }

    pub fn boot_with_device_loss_recovery(
        &mut self,
    ) -> Result<LocalRuntimeHostSummary, RuntimeError> {
        self.boot_with_fault_recovery(Some(FaultInjection::DeviceLoss))
    }

    pub fn boot_with_device_loss_restart_failure(
        &mut self,
    ) -> Result<LocalRuntimeHostSummary, RuntimeError> {
        self.boot_with_fault_recovery(Some(FaultInjection::DeviceLossRestartFailure))
    }

    #[cfg_attr(not(test), allow(dead_code))]
    pub fn boot_with_mixed_watchdog_soak(
        &mut self,
    ) -> Result<LocalRuntimeHostSummary, RuntimeError> {
        self.boot_with_fault_recovery(Some(FaultInjection::MixedWatchdogEpisodes {
            restart_episodes: SOAK_RESTART_EPISODES,
        }))
    }
}
