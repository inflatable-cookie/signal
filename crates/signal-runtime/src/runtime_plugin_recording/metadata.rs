use super::*;

impl SignalRuntime {
    /// Records the plugin sandbox specification into the lifecycle model.
    pub fn record_plugin_sandbox_spec(&mut self, spec: &PluginSandboxSpec) {
        self.plugin_lifecycle.record_spec(spec);
    }

    /// Ensure a SharedSandbox placement rule for `plugin_type_id`.
    ///
    /// When the rule omits `sandbox_group_key`, snapshots default to
    /// `plugin:{plugin_type_id}`. Does not require the runtime to be
    /// configured; factory construction records placement before handshake.
    pub fn ensure_shared_sandbox_placement(&mut self, plugin_type_id: &str) {
        let rule_id = format!("shared-sandbox:{plugin_type_id}");
        if self
            .plugin_placement_policy
            .rules
            .iter()
            .any(|rule| rule.rule_id == rule_id)
        {
            return;
        }
        self.plugin_placement_policy
            .rules
            .push(RuntimePluginPlacementRule {
                rule_id,
                matcher: RuntimePluginPlacementRuleMatcher::PluginTypeId(
                    plugin_type_id.to_string(),
                ),
                outcome: RuntimePluginIsolationOutcome::SharedSandbox,
                sandbox_group_key: None,
            });
    }
}
