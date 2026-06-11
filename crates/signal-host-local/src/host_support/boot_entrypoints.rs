use signal_hardware::{HardwareConfigRequest, HardwareNegotiationError, HardwareStreamConfig};
use signal_runtime::{
    BackendPolicyOverride, RuntimeError, RuntimeProjectionApi, RuntimeSupervisorApi,
};

use super::super::{LocalRuntimeHost, LocalRuntimeHostSummary};

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

    /// Boots the local host.
    pub fn boot_default(&mut self) -> Result<LocalRuntimeHostSummary, RuntimeError> {
        self.boot_local()
    }
}
