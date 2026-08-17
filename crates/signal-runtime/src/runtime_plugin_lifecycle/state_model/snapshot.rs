use super::*;

impl RuntimePluginLifecycleStateModel {
    pub(crate) fn snapshot(
        &self,
        policy: &RuntimePluginPlacementPolicy,
        boundary_stage_counts: &HashMap<String, usize>,
        discovered_types: &[RuntimePluginDiscoveredTypeRecord],
        platform_coverage: &[RuntimePluginFormatPlatformCoverageRecord],
    ) -> RuntimePluginLifecycleSnapshot {
        let mut group_counts = HashMap::new();
        for sandbox in self.sandboxes.values() {
            let placement = runtime_plugin_placement_decision(sandbox, policy);
            if placement.outcome == RuntimePluginIsolationOutcome::SharedSandbox {
                *group_counts.entry(placement.sandbox_group_key).or_insert(0) += 1;
            }
        }
        let sandboxes = self
            .sandboxes
            .values()
            .map(|sandbox| {
                let placement = runtime_plugin_placement_decision(sandbox, policy);
                let graph_count = boundary_stage_counts
                    .get(sandbox.sandbox_id.as_str())
                    .copied()
                    .unwrap_or(1);
                let group_count = group_counts
                    .get(&placement.sandbox_group_key)
                    .copied()
                    .unwrap_or(1);
                let member_count =
                    if placement.outcome == RuntimePluginIsolationOutcome::SharedSandbox {
                        graph_count.max(group_count)
                    } else {
                        graph_count
                    };
                runtime_plugin_sandbox_snapshot(sandbox, policy, member_count)
            })
            .collect::<Vec<_>>();
        let snapshot = RuntimePluginLifecycleSnapshot {
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
        };
        snapshot
    }
}
