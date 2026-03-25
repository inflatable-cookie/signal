use super::*;

impl SignalRuntime {
    pub(crate) fn diagnostics_snapshot(&self) -> RuntimeDiagnosticsSnapshot {
        RuntimeDiagnosticsSnapshot {
            cpu_load_percent: self.diagnostics.cpu_load_percent,
            xruns: self.diagnostics.xruns,
            graph_latency_ms: self.diagnostics.graph_latency_ms,
            active_plugin_sandboxes: self.diagnostics.active_plugin_sandboxes,
            backend_policy_tier: self.diagnostics.backend_policy_tier,
            topology_compatible: self.engine.snapshot.scheduler_topology.compatible,
            topology_issue_count: self.engine.snapshot.scheduler_topology.issues.len(),
            degraded_bound_plugin_sandboxes: self
                .engine
                .snapshot
                .prework_service_degraded_bound_plugin_sandboxes,
            missing_bound_plugin_sandboxes: self
                .engine
                .snapshot
                .prework_service_missing_bound_plugin_sandboxes,
            last_output_peak: self.metering.snapshot.main_output_peak_level,
            last_output_rms: self.metering.snapshot.main_output_rms_level,
            momentary_loudness_lufs: self.metering.snapshot.momentary_loudness_lufs,
            short_term_loudness_lufs: self.metering.snapshot.short_term_loudness_lufs,
            integrated_loudness_lufs: self.metering.snapshot.integrated_loudness_lufs,
        }
    }
}
