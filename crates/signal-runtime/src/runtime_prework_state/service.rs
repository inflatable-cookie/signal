use super::super::*;

impl RuntimeEngineState {
    pub(crate) fn classify_prework_service_semantic_policy(
        graph: &ExecutableGraph,
        anticipative_enabled: bool,
        active_plugin_sandboxes: u32,
    ) -> RuntimePreworkServiceSemanticPolicy {
        if !anticipative_enabled {
            return RuntimePreworkServiceSemanticPolicy::Balanced;
        }

        let planning = graph.planning_summary(anticipative_enabled);
        if planning.anticipative_eligible_node_count == 0 {
            return RuntimePreworkServiceSemanticPolicy::Balanced;
        }

        if graph.plugin_backed_node_count() > 0 && active_plugin_sandboxes > 0 {
            return RuntimePreworkServiceSemanticPolicy::PluginConstrained;
        }

        if graph.total_latency_samples() >= PREWORK_LATENCY_FOCUSED_THRESHOLD_SAMPLES
            || graph.max_node_latency_samples() >= PREWORK_LATENCY_FOCUSED_THRESHOLD_SAMPLES
        {
            RuntimePreworkServiceSemanticPolicy::LatencyFocused
        } else {
            RuntimePreworkServiceSemanticPolicy::Balanced
        }
    }

    pub(crate) fn set_prework_service_pressure(&mut self, pressure: RuntimePreworkServicePressure) {
        self.snapshot.prework_service_pressure = pressure;
    }

    pub(crate) fn set_prework_service_plugin_state(
        &mut self,
        active_plugin_sandboxes: u32,
        bound_plugin_sandboxes: usize,
        active_bound_plugin_sandboxes: usize,
        degraded_bound_plugin_sandboxes: usize,
        missing_bound_plugin_sandboxes: usize,
        plugin_gate_active: bool,
    ) {
        self.snapshot.prework_service_active_plugin_sandboxes = active_plugin_sandboxes;
        self.snapshot.prework_service_bound_plugin_sandboxes = bound_plugin_sandboxes;
        self.snapshot.prework_service_active_bound_plugin_sandboxes = active_bound_plugin_sandboxes;
        self.snapshot
            .prework_service_degraded_bound_plugin_sandboxes = degraded_bound_plugin_sandboxes;
        self.snapshot.prework_service_missing_bound_plugin_sandboxes =
            missing_bound_plugin_sandboxes;
        self.snapshot.prework_service_plugin_gate_active = plugin_gate_active;
    }

    pub(crate) fn set_prework_service_transport_state(
        &mut self,
        recovery_overlap_sessions: usize,
        lingering_sessions: usize,
        detach_faulted_sessions: usize,
        transport_gate_active: bool,
    ) {
        self.snapshot.prework_service_recovery_overlap_sessions = recovery_overlap_sessions;
        self.snapshot.prework_service_lingering_sessions = lingering_sessions;
        self.snapshot.prework_service_detach_faulted_sessions = detach_faulted_sessions;
        self.snapshot.prework_service_transport_gate_active = transport_gate_active;
    }

    pub(crate) fn transition_prework_service_state(
        &mut self,
        state: RuntimePreworkServiceState,
        processing_epoch: Option<u64>,
    ) {
        let previous = self.snapshot.prework_service_state;
        if previous == state {
            return;
        }
        if state == RuntimePreworkServiceState::Paused {
            self.snapshot.prework_service_pause_count =
                self.snapshot.prework_service_pause_count.saturating_add(1);
        }
        if previous == RuntimePreworkServiceState::Paused
            && state == RuntimePreworkServiceState::Servicing
        {
            self.snapshot.prework_service_resume_count =
                self.snapshot.prework_service_resume_count.saturating_add(1);
        }
        if state == RuntimePreworkServiceState::Starved {
            self.snapshot.prework_service_starvation_count = self
                .snapshot
                .prework_service_starvation_count
                .saturating_add(1);
        }
        self.snapshot.prework_service_state = state;
        if let Some(processing_epoch) = processing_epoch {
            self.snapshot.last_prework_service_processing_epoch = Some(processing_epoch);
        }
    }

    pub(crate) fn record_prework_service_cycle(
        &mut self,
        processing_epoch: u64,
        cycle_count: usize,
        budget_per_cycle: usize,
        prepared_targets: usize,
    ) {
        self.snapshot.prework_service_cycle_count = self
            .snapshot
            .prework_service_cycle_count
            .saturating_add(cycle_count as u64);
        self.snapshot.prework_service_prepared_targets = self
            .snapshot
            .prework_service_prepared_targets
            .saturating_add(prepared_targets as u64);
        self.snapshot.last_prework_service_processing_epoch = Some(processing_epoch);
        self.snapshot.last_prework_service_cycle_count = cycle_count;
        self.snapshot.last_prework_service_budget_per_cycle = Some(budget_per_cycle);
        self.snapshot.last_prework_service_prepared_targets = prepared_targets;
        self.update_prework_queue_snapshot(
            None,
            self.snapshot.prework_cache_state == RuntimePreworkCacheState::Invalidated,
        );
    }

    pub(crate) fn record_prework_service_request(
        &mut self,
        requested_cycles: usize,
        effective_cycles: usize,
        requested_budget_per_cycle: usize,
        effective_budget_per_cycle: usize,
    ) {
        self.snapshot.last_prework_service_requested_cycles = requested_cycles;
        self.snapshot.last_prework_service_effective_cycles = effective_cycles;
        self.snapshot.last_prework_service_budget_per_cycle = Some(requested_budget_per_cycle);
        self.snapshot
            .last_prework_service_effective_budget_per_cycle = Some(effective_budget_per_cycle);
        if effective_cycles < requested_cycles
            || effective_budget_per_cycle < requested_budget_per_cycle
        {
            self.snapshot.prework_service_throttle_count = self
                .snapshot
                .prework_service_throttle_count
                .saturating_add(1);
        }
    }

    pub(crate) fn record_prework_service_yield(
        &mut self,
        processing_epoch: u64,
        requested_cycles: usize,
        requested_budget_per_cycle: usize,
    ) {
        self.snapshot.prework_service_yield_count =
            self.snapshot.prework_service_yield_count.saturating_add(1);
        self.snapshot.last_prework_service_processing_epoch = Some(processing_epoch);
        self.snapshot.last_prework_service_requested_cycles = requested_cycles;
        self.snapshot.last_prework_service_effective_cycles = 0;
        self.snapshot.last_prework_service_cycle_count = 0;
        self.snapshot.last_prework_service_budget_per_cycle = Some(requested_budget_per_cycle);
        self.snapshot
            .last_prework_service_effective_budget_per_cycle = Some(0);
        self.snapshot.last_prework_service_prepared_targets = 0;
    }

    pub(crate) fn record_last_serviced_pending_target(
        &mut self,
        target: &RuntimePendingPreworkTarget,
    ) {
        self.snapshot.last_prework_serviced_target_block_sequence =
            Some(target.target_block_sequence);
        self.snapshot.last_prework_serviced_backlog_class = Some(target.backlog_class);
    }
}
