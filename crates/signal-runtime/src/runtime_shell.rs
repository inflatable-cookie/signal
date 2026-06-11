use super::*;

impl core::fmt::Debug for SignalRuntime {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("SignalRuntime")
            .field("config", &self.config)
            .field("readiness", &self.readiness)
            .field("safe_mode_enabled", &self.safe_mode_enabled)
            .field("anticipative_enabled", &self.anticipative_enabled)
            .field("active_output_device", &self.active_output_device)
            .field("projection_epoch", &self.projection_epoch)
            .field("control", &self.control)
            .field("plan", &self.plan)
            .field("diagnostics", &self.diagnostics)
            .field("supervision", &self.supervision)
            .finish()
    }
}
