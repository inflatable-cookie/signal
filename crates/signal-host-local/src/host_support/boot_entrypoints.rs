use signal_hardware::{
    HardwareConfigRequest, HardwareNegotiationError, HardwareNegotiationErrorKind,
    HardwareStreamConfig, HardwareStreamRequest,
};
use signal_runtime::{
    BackendPolicyOverride, RuntimeError, RuntimeProjectionApi, RuntimeSupervisorApi,
};

use super::super::{LocalRuntimeHost, LocalRuntimeHostSummary};

impl LocalRuntimeHost {
    pub(crate) fn prepare_default_output_hardware(
        &mut self,
    ) -> Result<HardwareStreamConfig, RuntimeError> {
        let sample_rate = self.runtime.config().sample_rate.0;
        let buffer_size = self.runtime.config().graph.block_size;
        let device = self.hardware.default_output_device().ok_or_else(|| {
            Self::runtime_error_from_hardware_negotiation(HardwareNegotiationError::new(
                HardwareNegotiationErrorKind::DeviceUnavailable,
                "no default output device is currently available",
            ))
        })?;
        let request =
            HardwareStreamRequest::new_output(device.device_id.clone(), sample_rate, buffer_size);
        let stream = self
            .hardware
            .negotiate_stream(&request)
            .map_err(Self::runtime_error_from_hardware_negotiation)?;
        let hardware_request =
            HardwareConfigRequest::from_stream(&stream, self.hardware.policy_record().tier);
        self.runtime.apply_hardware_config(hardware_request)?;
        self.runtime
            .set_active_output_device(stream.device.device_id.clone());
        self.set_backend_policy(BackendPolicyOverride {
            tier: hardware_request.backend_policy,
        })?;
        self.runtime
            .set_backend_policy_tier(hardware_request.backend_policy);
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
