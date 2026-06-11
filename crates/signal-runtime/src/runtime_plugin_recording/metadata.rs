use super::*;

impl SignalRuntime {
    /// Records the plugin sandbox specification into the lifecycle model.
    pub fn record_plugin_sandbox_spec(&mut self, spec: &PluginSandboxSpec) {
        self.plugin_lifecycle.record_spec(spec);
    }

    /// Records a plugin preset descriptor for the given sandbox.
    pub fn record_plugin_preset_descriptor(
        &mut self,
        sandbox_id: impl Into<String>,
        descriptor: RuntimePluginPresetDescriptor,
    ) {
        let sandbox_id = sandbox_id.into();
        self.plugin_lifecycle
            .record_preset_descriptor(sandbox_id.as_str(), descriptor);
    }

    /// Records an ARA context snapshot for the given sandbox.
    pub fn record_plugin_ara_context(
        &mut self,
        sandbox_id: impl Into<String>,
        context: RuntimePluginAraContextSnapshot,
    ) {
        let sandbox_id = sandbox_id.into();
        self.plugin_lifecycle
            .record_ara_context(sandbox_id.as_str(), context);
    }
}
