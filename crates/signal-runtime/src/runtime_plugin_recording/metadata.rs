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
        mut descriptor: RuntimePluginPresetDescriptor,
    ) {
        let sandbox_id = sandbox_id.into();
        if descriptor.summary.is_empty() {
            descriptor.summary = format!(
                "preset={:?} label={:?} origin={:?}",
                descriptor.preset_id.as_deref(),
                descriptor.label.as_deref(),
                descriptor.origin,
            );
        }
        self.plugin_lifecycle
            .record_preset_descriptor(sandbox_id.as_str(), descriptor);
    }

    /// Records an ARA context snapshot for the given sandbox, filling in any missing summary fields.
    pub fn record_plugin_ara_context(
        &mut self,
        sandbox_id: impl Into<String>,
        mut context: RuntimePluginAraContextSnapshot,
    ) {
        let sandbox_id = sandbox_id.into();
        if let Some(document_context) = context.document_context.as_mut() {
            if document_context.summary.is_empty() {
                document_context.summary = format!(
                    "document={} label={:?}",
                    document_context.document_id,
                    document_context.display_label.as_deref(),
                );
            }
        }
        if let Some(source_context) = context.source_context.as_mut() {
            if source_context.summary.is_empty() {
                source_context.summary = format!(
                    "source={} label={:?}",
                    source_context.source_id,
                    source_context.display_label.as_deref(),
                );
            }
        }
        if let Some(region_context) = context.region_context.as_mut() {
            if region_context.summary.is_empty() {
                region_context.summary = format!(
                    "region={} label={:?} start={:?} duration={:?}",
                    region_context.region_id,
                    region_context.display_label.as_deref(),
                    region_context.timeline_start_samples,
                    region_context.duration_samples,
                );
            }
        }
        if context.summary.is_empty() {
            context.summary = format!(
                "class={:?} document={} source={} region={}",
                context.portability_class,
                context.document_context.is_some(),
                context.source_context.is_some(),
                context.region_context.is_some(),
            );
        }
        self.plugin_lifecycle
            .record_ara_context(sandbox_id.as_str(), context);
    }
}
