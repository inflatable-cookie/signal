use super::*;

impl SignalRuntime {
    pub(crate) fn plugin_lifecycle_snapshot(&self) -> RuntimePluginLifecycleSnapshot {
        let boundary_counts = runtime_plugin_boundary_counts(&self.plan.planned_nodes);
        self.plugin_lifecycle.snapshot(
            &self.plugin_placement_policy,
            &boundary_counts.sandbox_stage_counts,
            &self.plugin_discovery.discovered_types,
            &self.plugin_discovery.platform_coverage,
        )
    }
}
