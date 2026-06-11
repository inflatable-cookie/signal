use super::*;

impl SignalRuntime {
    pub(crate) fn require_handshake(&self) -> Result<(), RuntimeError> {
        if self.control.handshaken {
            Ok(())
        } else {
            Err(RuntimeError::new(
                RuntimeErrorKind::InvalidState,
                "runtime must be handshaken before control requests",
            ))
        }
    }

    pub(crate) fn require_configured(&self) -> Result<(), RuntimeError> {
        if self.control.configured {
            Ok(())
        } else {
            Err(RuntimeError::new(
                RuntimeErrorKind::InvalidState,
                "runtime must be configured before this request",
            ))
        }
    }

    pub(crate) fn apply_hardware_config_state(
        &mut self,
        request: HardwareConfigRequest,
    ) -> Result<(), RuntimeError> {
        self.require_handshake()?;
        self.require_configured()?;
        if request.buffer_size == 0 || request.sample_rate.0 == 0 {
            return Err(RuntimeError::new(
                RuntimeErrorKind::InvalidRequest,
                "hardware config sample_rate and buffer_size must be non-zero",
            ));
        }

        self.config.sample_rate = request.sample_rate;
        self.config.graph.block_size = request.buffer_size;
        self.diagnostics.backend_policy_tier = request.backend_policy;
        self.emit(RuntimeEvent::EffectiveConfigChanged(
            self.get_effective_config(),
        ));
        Ok(())
    }
}
