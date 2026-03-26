use signal_plugin::WatchdogOutcome;
use signal_plugin_clap::{BrokeredBlockOutcome, ClapBlockProtocol, ClapSandboxLifecycleHarness};
use signal_runtime::{
    HeartbeatCycleStage, RuntimeError, RuntimePreworkServicePressure, RuntimeProjectionApi,
    WatchdogRestartRecord,
};

use super::super::ServerRuntimeHost;
use super::{
    plugin_instance_state_record_from_response, record_runtime_fault, runtime_error_from_failure,
    runtime_watchdog_trigger, LifecycleRunSummary,
};

impl ServerRuntimeHost {
    pub(crate) fn execute_block_sequence(
        &mut self,
        protocol: &ClapBlockProtocol,
        run: &mut LifecycleRunSummary,
        block_count: u64,
        lifecycle: &mut ClapSandboxLifecycleHarness,
        simulate_timeout: bool,
    ) -> Result<(), RuntimeError> {
        let mut last_completed_block_sequence = None;
        for block_offset in 0..block_count {
            let block_sequence = self
                .runtime
                .next_planned_prework_block_sequence(last_completed_block_sequence)
                .unwrap_or_else(|| self.runtime.allocate_block_sequence());
            let should_timeout = simulate_timeout && block_offset == 0;
            let _ = self.run_realtime_cycle(
                protocol,
                run,
                block_sequence,
                lifecycle,
                false,
                should_timeout,
            )?;
            last_completed_block_sequence = Some(block_sequence);
        }
        Ok(())
    }

    pub(crate) fn run_realtime_cycle(
        &mut self,
        protocol: &ClapBlockProtocol,
        run: &mut LifecycleRunSummary,
        block_sequence: u64,
        lifecycle: &mut ClapSandboxLifecycleHarness,
        simulate_heartbeat_miss: bool,
        simulate_timeout: bool,
    ) -> Result<Option<BrokeredBlockOutcome>, RuntimeError> {
        self.runtime
            .set_prework_service_pressure(self.prework_service_pressure(
                run,
                simulate_heartbeat_miss,
                simulate_timeout,
            ))?;
        let _ = self.runtime.service_prework_lane(run.processing_epoch, 1)?;
        if !self.poll_heartbeat(
            protocol,
            run,
            lifecycle,
            simulate_heartbeat_miss,
            Some(block_sequence),
        )? {
            return Ok(None);
        }

        let result =
            self.execute_block(protocol, run, block_sequence, lifecycle, simulate_timeout)?;
        self.runtime
            .set_prework_service_pressure(self.prework_service_pressure(
                run,
                false,
                simulate_timeout && run.current_watchdog_triggered,
            ))?;
        let _ = self.runtime.service_prework_lane(run.processing_epoch, 1)?;
        Ok(Some(result))
    }

    pub(crate) fn prework_service_pressure(
        &self,
        run: &LifecycleRunSummary,
        simulate_heartbeat_miss: bool,
        simulate_timeout: bool,
    ) -> RuntimePreworkServicePressure {
        if simulate_heartbeat_miss || simulate_timeout || run.current_watchdog_triggered {
            RuntimePreworkServicePressure::Critical
        } else if run.watchdog_triggered {
            RuntimePreworkServicePressure::Elevated
        } else {
            RuntimePreworkServicePressure::Normal
        }
    }

    pub(crate) fn poll_heartbeat(
        &mut self,
        protocol: &ClapBlockProtocol,
        run: &mut LifecycleRunSummary,
        lifecycle: &mut ClapSandboxLifecycleHarness,
        simulate_miss: bool,
        block_sequence: Option<u64>,
    ) -> Result<bool, RuntimeError> {
        if simulate_miss {
            self.runtime.record_heartbeat_cycle(
                run.sandbox_id.as_str(),
                HeartbeatCycleStage::Missed,
                Some(run.processing_epoch),
                block_sequence,
            );
            run.heartbeat_misses = run.heartbeat_misses.saturating_add(1);
            if let WatchdogOutcome::RestartRequired {
                reason,
                consecutive_misses: _,
            } = run.watchdog.record_heartbeat_miss()
            {
                run.watchdog_triggered = true;
                run.watchdog_trigger_reason = Some(reason);
                run.current_watchdog_triggered = true;
                self.runtime.record_watchdog_restart(WatchdogRestartRecord {
                    sandbox_id: run.sandbox_id.clone(),
                    trigger: runtime_watchdog_trigger(reason),
                    processing_epoch: run.processing_epoch,
                });
            }
            return Ok(false);
        }

        self.runtime.record_heartbeat_cycle(
            run.sandbox_id.as_str(),
            HeartbeatCycleStage::Requested,
            Some(run.processing_epoch),
            block_sequence,
        );
        let heartbeat = lifecycle
            .handle(protocol.heartbeat_request(run.sandbox_id.as_str(), Some(run.processing_epoch)))
            .map_err(|failure| {
                record_runtime_fault(&mut self.runtime, &failure);
                runtime_error_from_failure(&failure)
            })?;
        if let Some(instance_state) = plugin_instance_state_record_from_response(
            run.sandbox_id.as_str(),
            Some(run.processing_epoch),
            &heartbeat,
        ) {
            self.runtime
                .record_plugin_sandbox_instance_state(instance_state.clone());
            run.last_plugin_state = Some(instance_state);
        }
        self.runtime.record_heartbeat_cycle(
            run.sandbox_id.as_str(),
            HeartbeatCycleStage::Responded,
            Some(run.processing_epoch),
            block_sequence,
        );
        run.control_requests = run.control_requests.saturating_add(1);
        run.control_responses = run.control_responses.saturating_add(1);
        run.heartbeat_responses = run.heartbeat_responses.saturating_add(1);
        run.last_control_message = heartbeat.message.name.clone();
        run.watchdog.record_heartbeat_response();
        if let Some(sequence) = block_sequence {
            run.last_block_sequence = sequence;
        }
        Ok(true)
    }
}
