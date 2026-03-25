use super::super::*;

impl SignalRuntime {
    pub(crate) fn recompute_prework_service_policy_snapshot(&mut self) {
        let binding_summary = self.summarize_plugin_backed_bindings();
        let transport_condition = self.current_prework_transport_condition();
        let semantic_policy = self
            .engine
            .graph
            .as_ref()
            .map(|graph| {
                RuntimeEngineState::classify_prework_service_semantic_policy(
                    graph,
                    self.anticipative_enabled,
                    if !binding_summary.bound_sandbox_ids.is_empty() {
                        binding_summary.active_bound_sandboxes as u32
                            + binding_summary.degraded_bound_sandboxes as u32
                            + binding_summary.missing_bound_sandboxes as u32
                    } else {
                        self.diagnostics.active_plugin_sandboxes
                    },
                )
            })
            .unwrap_or(RuntimePreworkServiceSemanticPolicy::Balanced);
        let plugin_gate_active = matches!(
            semantic_policy,
            RuntimePreworkServiceSemanticPolicy::PluginConstrained
        ) && self.engine.snapshot.prework_service_pressure
            != RuntimePreworkServicePressure::Normal
            && if !binding_summary.bound_sandbox_ids.is_empty() {
                binding_summary.degraded_bound_sandboxes > 0
                    || binding_summary.missing_bound_sandboxes > 0
                    || binding_summary.active_bound_sandboxes > 1
            } else {
                self.diagnostics.active_plugin_sandboxes > 1
            };
        let transport_gate_active =
            transport_condition.gate_active(self.engine.snapshot.prework_service_pressure);
        self.engine.snapshot.prework_service_semantic_policy = semantic_policy;
        self.engine.set_prework_service_plugin_state(
            self.diagnostics.active_plugin_sandboxes,
            binding_summary.bound_sandbox_ids.len(),
            binding_summary.active_bound_sandboxes,
            binding_summary.degraded_bound_sandboxes,
            binding_summary.missing_bound_sandboxes,
            plugin_gate_active,
        );
        self.engine.set_prework_service_transport_state(
            transport_condition.recovery_overlap_sessions,
            transport_condition.lingering_sessions,
            transport_condition.detach_faulted_sessions,
            transport_gate_active,
        );
    }

    pub(crate) fn current_prework_transport_condition(&self) -> RuntimePreworkTransportCondition {
        RuntimePreworkTransportCondition {
            recovery_overlap_sessions: self.transport_concurrency.recovery_overlap_session_count(),
            lingering_sessions: self.transport_concurrency.lingering_session_count(),
            detach_faulted_sessions: self.transport_concurrency.detach_faulted_session_count(),
        }
    }

    pub(crate) fn multicore_prework_budget_scale(&self) -> usize {
        let Some(schedule) = self.applied_schedule.as_ref() else {
            return 1;
        };
        if schedule.stream_count <= 1
            || !self.engine.snapshot.scheduler_topology.compatible
            || !self.engine.snapshot.anticipative_planning_enabled
            || self.engine.snapshot.anticipative_lane_count == 0
        {
            return 1;
        }
        schedule.stream_count
    }

    pub(crate) fn multicore_prework_requested_cycles(&self, cycles: usize) -> usize {
        cycles.saturating_mul(self.multicore_prework_budget_scale().max(1))
    }

    pub(crate) fn refresh_prework_service_policy_and_state(
        &mut self,
        processing_epoch: Option<u64>,
    ) {
        self.recompute_prework_service_policy_snapshot();
        self.reconcile_prework_service_state(processing_epoch);
    }
}
