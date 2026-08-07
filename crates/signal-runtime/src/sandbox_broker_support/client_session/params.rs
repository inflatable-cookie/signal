use super::super::types::*;

impl SandboxBrokerClientSession {
    /// Sends one normalized 0..1 parameter write (g12.023). The child
    /// queues it on the format's audio-thread-correct set path; delivery
    /// is block-boundary.
    pub fn set_parameter(&mut self, parameter_id: u32, normalized: f32) -> std::io::Result<String> {
        self.set_parameters(&[(parameter_id, normalized)])
    }

    /// Sends a batched `(parameter_id, normalized 0..1)` write (g12.023):
    /// one `set-params` command, one `param_set` receipt for the whole
    /// batch.
    pub fn set_parameters(&mut self, changes: &[(u32, f32)]) -> std::io::Result<String> {
        if changes.is_empty() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "set_parameters requires at least one change",
            ));
        }
        let blob = changes
            .iter()
            .map(|(parameter_id, normalized)| format!("{parameter_id}:{normalized}"))
            .collect::<Vec<_>>()
            .join(";");
        self.simple_plugin_command(
            &format!("set-params {blob}"),
            SandboxBrokerReceiptState::ParamSet,
        )
    }
}
