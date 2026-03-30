use super::*;

impl RuntimePluginLifecycleStateModel {
    pub(crate) fn snapshot(
        &self,
        policy: &RuntimePluginPlacementPolicy,
        boundary_stage_counts: &HashMap<String, usize>,
        discovered_types: &[RuntimePluginDiscoveredTypeRecord],
        platform_coverage: &[RuntimePluginFormatPlatformCoverageRecord],
    ) -> RuntimePluginLifecycleSnapshot {
        let sandboxes = self
            .sandboxes
            .values()
            .map(|sandbox| {
                runtime_plugin_sandbox_snapshot(
                    sandbox,
                    policy,
                    boundary_stage_counts
                        .get(sandbox.sandbox_id.as_str())
                        .copied()
                        .unwrap_or(1),
                )
            })
            .collect::<Vec<_>>();
        let mut snapshot = RuntimePluginLifecycleSnapshot {
            sandbox_count: sandboxes.len(),
            active_sandbox_count: self.active_sandbox_count,
            shared_sandbox_count: sandboxes
                .iter()
                .filter(|sandbox| {
                    sandbox.placement_outcome == RuntimePluginIsolationOutcome::SharedSandbox
                })
                .count(),
            isolated_sandbox_count: sandboxes
                .iter()
                .filter(|sandbox| {
                    sandbox.placement_outcome == RuntimePluginIsolationOutcome::IsolatedSandbox
                })
                .count(),
            ready_sandbox_count: sandboxes
                .iter()
                .filter(|sandbox| sandbox.state == RuntimePluginLifecycleState::Ready)
                .count(),
            booting_sandbox_count: sandboxes
                .iter()
                .filter(|sandbox| sandbox.state == RuntimePluginLifecycleState::Booting)
                .count(),
            degraded_sandbox_count: sandboxes
                .iter()
                .filter(|sandbox| sandbox.state == RuntimePluginLifecycleState::Degraded)
                .count(),
            faulted_sandbox_count: sandboxes
                .iter()
                .filter(|sandbox| sandbox.state == RuntimePluginLifecycleState::Faulted)
                .count(),
            restarting_sandbox_count: sandboxes
                .iter()
                .filter(|sandbox| sandbox.state == RuntimePluginLifecycleState::Restarting)
                .count(),
            quarantined_sandbox_count: sandboxes
                .iter()
                .filter(|sandbox| sandbox.state == RuntimePluginLifecycleState::Quarantined)
                .count(),
            stopped_sandbox_count: sandboxes
                .iter()
                .filter(|sandbox| sandbox.state == RuntimePluginLifecycleState::Stopped)
                .count(),
            rebindable_sandbox_count: sandboxes
                .iter()
                .filter(|sandbox| sandbox.rebindable)
                .count(),
            terminal_sandbox_count: sandboxes
                .iter()
                .filter(|sandbox| sandbox.continuity_class == RuntimeInterruptionClass::Terminal)
                .count(),
            parity_coverage: runtime_plugin_parity_coverage(
                discovered_types,
                &sandboxes,
                policy,
                platform_coverage,
            ),
            sandboxes,
            summary: String::new(),
        };
        snapshot.summary = format!(
            "sandboxes={} active={} shared={} isolated={} ready={} booting={} degraded={} faulted={} restarting={} quarantined={} stopped={} rebindable={} terminal={}",
            snapshot.sandbox_count,
            snapshot.active_sandbox_count,
            snapshot.shared_sandbox_count,
            snapshot.isolated_sandbox_count,
            snapshot.ready_sandbox_count,
            snapshot.booting_sandbox_count,
            snapshot.degraded_sandbox_count,
            snapshot.faulted_sandbox_count,
            snapshot.restarting_sandbox_count,
            snapshot.quarantined_sandbox_count,
            snapshot.stopped_sandbox_count,
            snapshot.rebindable_sandbox_count,
            snapshot.terminal_sandbox_count,
        );
        snapshot
    }
}
