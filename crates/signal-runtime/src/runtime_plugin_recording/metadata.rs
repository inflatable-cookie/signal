use super::*;

impl SignalRuntime {
    /// Records the plugin sandbox specification into the lifecycle model.
    pub fn record_plugin_sandbox_spec(&mut self, spec: &PluginSandboxSpec) {
        self.plugin_lifecycle.record_spec(spec);
    }
}
