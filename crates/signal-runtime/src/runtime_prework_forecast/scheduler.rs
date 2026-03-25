use crate::{
    RuntimeError, RuntimePreworkBacklogClass, RuntimePreworkForecastMode,
    RuntimePreworkForecastPolicy, SignalRuntime,
};

impl SignalRuntime {
    pub fn prime_engine_prework_window_with_forecast(
        &mut self,
        processing_epoch: u64,
        current_block_sequence: u64,
        policy: &RuntimePreworkForecastPolicy,
    ) -> Result<usize, RuntimeError> {
        self.reconcile_prework_window_with_forecast(current_block_sequence, policy);
        let admitted = if self.control.running {
            let requested_cycles = self.multicore_prework_requested_cycles(1);
            self.service_prework_lane_with_policy(
                processing_epoch,
                requested_cycles,
                policy.prepare_budget_per_cycle,
            )?
        } else {
            self.prime_pending_prework_targets(
                processing_epoch,
                policy.prepare_budget_per_cycle,
                RuntimePreworkBacklogClass::Deferred,
            )?
        };
        self.reconcile_prework_service_state(Some(processing_epoch));
        Ok(admitted)
    }

    pub(crate) fn enforce_scheduler_after_engine_block(
        &mut self,
        processing_epoch: u64,
        current_block_sequence: u64,
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
        let Some(policy) = self.prework_forecast_policy.clone() else {
            self.reconcile_prework_service_state(Some(processing_epoch));
            return Ok(0);
        };

        self.reconcile_prework_window_with_forecast(current_block_sequence, &policy);
        if self.control.running {
            let requested_cycles = self.multicore_prework_requested_cycles(1);
            self.service_prework_lane_with_policy(
                processing_epoch,
                requested_cycles,
                policy.prepare_budget_per_cycle,
            )
        } else {
            let prepared = self.prime_pending_prework_targets(
                processing_epoch,
                policy.prepare_budget_per_cycle,
                RuntimePreworkBacklogClass::Deferred,
            )?;
            self.reconcile_prework_service_state(Some(processing_epoch));
            Ok(prepared)
        }
    }
}
