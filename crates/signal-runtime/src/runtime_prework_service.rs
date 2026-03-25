use super::*;

impl SignalRuntime {
    pub(crate) fn service_prework_lane_with_policy(
        &mut self,
        processing_epoch: u64,
        cycles: usize,
        budget_per_cycle: usize,
    ) -> Result<usize, RuntimeError> {
        if !self.control.configured {
            return Ok(0);
        }
        if self.prework_forecast_mode == RuntimePreworkForecastMode::Disabled
            || !self.engine.snapshot.prework_cache_enabled
        {
            self.reconcile_prework_service_state(Some(processing_epoch));
            return Ok(0);
        }
        if !self.control.running {
            self.reconcile_prework_service_state(Some(processing_epoch));
            return Ok(0);
        }
        if self.engine.pending_prework_targets.is_empty() {
            self.reconcile_prework_service_state(Some(processing_epoch));
            return Ok(0);
        }
        self.recompute_prework_service_policy_snapshot();
        let pressure = self.engine.snapshot.prework_service_pressure;
        let semantic_policy = self.engine.snapshot.prework_service_semantic_policy;
        let transport_condition = self.current_prework_transport_condition();
        let widened_budget_per_cycle =
            budget_per_cycle.saturating_mul(self.multicore_prework_budget_scale().max(1));
        if self.engine.snapshot.prework_service_plugin_gate_active
            || self.engine.snapshot.prework_service_transport_gate_active
        {
            self.engine
                .record_prework_service_yield(processing_epoch, cycles, budget_per_cycle);
            self.engine.transition_prework_service_state(
                RuntimePreworkServiceState::Yielding,
                Some(processing_epoch),
            );
            return Ok(0);
        }
        let (effective_cycles, effective_budget_per_cycle, max_backlog_class) = match pressure {
            RuntimePreworkServicePressure::Normal => (
                cycles,
                widened_budget_per_cycle,
                RuntimePreworkBacklogClass::Deferred,
            ),
            RuntimePreworkServicePressure::Elevated => match semantic_policy {
                RuntimePreworkServiceSemanticPolicy::Balanced => (
                    cycles.min(1),
                    widened_budget_per_cycle.min(1),
                    RuntimePreworkBacklogClass::NearTerm,
                ),
                RuntimePreworkServiceSemanticPolicy::PluginConstrained => (
                    cycles.min(1),
                    widened_budget_per_cycle.min(1),
                    RuntimePreworkBacklogClass::Immediate,
                ),
                RuntimePreworkServiceSemanticPolicy::LatencyFocused => (
                    cycles.min(1),
                    widened_budget_per_cycle.min(2),
                    RuntimePreworkBacklogClass::Deferred,
                ),
            },
            RuntimePreworkServicePressure::Critical => {
                (0, 0, RuntimePreworkBacklogClass::Immediate)
            }
        };
        let (effective_cycles, effective_budget_per_cycle, max_backlog_class) = transport_condition
            .reduce_service_scope(
                effective_cycles,
                effective_budget_per_cycle,
                max_backlog_class,
            );
        if pressure == RuntimePreworkServicePressure::Critical {
            self.engine
                .record_prework_service_yield(processing_epoch, cycles, budget_per_cycle);
            self.engine.transition_prework_service_state(
                RuntimePreworkServiceState::Yielding,
                Some(processing_epoch),
            );
            return Ok(0);
        }
        self.engine.record_prework_service_request(
            cycles,
            effective_cycles,
            budget_per_cycle,
            effective_budget_per_cycle,
        );
        if effective_cycles == 0 || effective_budget_per_cycle == 0 {
            self.engine.transition_prework_service_state(
                RuntimePreworkServiceState::Starved,
                Some(processing_epoch),
            );
            return Ok(0);
        }

        self.engine.transition_prework_service_state(
            RuntimePreworkServiceState::Servicing,
            Some(processing_epoch),
        );
        let mut total_prepared = 0usize;
        let mut executed_cycles = 0usize;
        for _ in 0..effective_cycles {
            executed_cycles = executed_cycles.saturating_add(1);
            total_prepared = total_prepared.saturating_add(self.service_pending_prework_cycle(
                processing_epoch,
                effective_budget_per_cycle,
                max_backlog_class,
            )?);
            if self.engine.pending_prework_targets.is_empty() {
                break;
            }
        }
        self.engine.record_prework_service_cycle(
            processing_epoch,
            executed_cycles,
            budget_per_cycle,
            total_prepared,
        );
        if !self.engine.pending_prework_targets.is_empty() && total_prepared == 0 {
            if pressure == RuntimePreworkServicePressure::Elevated {
                self.engine.record_prework_service_yield(
                    processing_epoch,
                    cycles,
                    budget_per_cycle,
                );
                self.engine.transition_prework_service_state(
                    RuntimePreworkServiceState::Yielding,
                    Some(processing_epoch),
                );
            } else {
                self.engine.transition_prework_service_state(
                    RuntimePreworkServiceState::Starved,
                    Some(processing_epoch),
                );
            }
        } else {
            self.reconcile_prework_service_state(Some(processing_epoch));
        }
        Ok(total_prepared)
    }

    pub(crate) fn prime_pending_prework_targets(
        &mut self,
        processing_epoch: u64,
        budget: usize,
        max_backlog_class: RuntimePreworkBacklogClass,
    ) -> Result<usize, RuntimeError> {
        if budget == 0 || !self.control.configured {
            return Ok(0);
        }
        let prepared =
            self.service_pending_prework_cycle(processing_epoch, budget, max_backlog_class)?;
        if prepared > 0 {
            self.engine.transition_prework_service_state(
                RuntimePreworkServiceState::Pending,
                Some(processing_epoch),
            );
        }
        Ok(prepared)
    }
}
