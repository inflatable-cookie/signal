use signal_plugin::CompletionState;
use signal_plugin_clap::{ClapBlockProtocol, ClapSandboxLifecycleHarness};
use signal_runtime::{RuntimeError, RuntimeErrorKind};

use super::super::INTER_EPISODE_CONTINUITY_BLOCKS;
use super::super::{
    FaultInjection, RecoveryFailureInjection, ServerRuntimeHost, WATCHDOG_TRIGGER_WINDOW_BLOCKS,
};
use super::{
    build_fault_envelope, record_runtime_fault, LifecycleRunSummary,
    ServerDemoPluginSandboxAssembly,
};

impl ServerRuntimeHost {
    pub(super) fn apply_timeout_recovery_failure(
        &mut self,
        protocol: &ClapBlockProtocol,
        sandbox: &ServerDemoPluginSandboxAssembly,
        run: &mut LifecycleRunSummary,
        lifecycle: &mut ClapSandboxLifecycleHarness,
        failure: RecoveryFailureInjection,
        detail: &str,
    ) -> Result<(), RuntimeError> {
        self.walk_timeout_watchdog(protocol, sandbox, run, lifecycle, |this, run, lifecycle| {
            match this.handle_watchdog_recovery(
                protocol,
                &sandbox.request.sandbox_id,
                lifecycle,
                run,
                Some(failure),
            ) {
                Ok(_) => Err(RuntimeError::new(
                    RuntimeErrorKind::ResourceUnavailable,
                    detail,
                )),
                Err(error) => Err(error),
            }
        })
    }

    pub(super) fn apply_retrying_timeout_recovery(
        &mut self,
        protocol: &ClapBlockProtocol,
        sandbox: &ServerDemoPluginSandboxAssembly,
        run: &mut LifecycleRunSummary,
        lifecycle: &mut ClapSandboxLifecycleHarness,
        failures: &[RecoveryFailureInjection],
        terminal_detail: &str,
        recover_after_failures: bool,
    ) -> Result<(), RuntimeError> {
        self.walk_timeout_watchdog(protocol, sandbox, run, lifecycle, |this, run, lifecycle| {
            for (index, failure) in failures.iter().copied().enumerate() {
                let result = this.handle_watchdog_recovery(
                    protocol,
                    &sandbox.request.sandbox_id,
                    lifecycle,
                    run,
                    Some(failure),
                );
                if result.is_ok() {
                    let detail = if index + 1 == failures.len() {
                        terminal_detail
                    } else {
                        "expected deferred teardown recovery failure"
                    };
                    return Err(RuntimeError::new(
                        RuntimeErrorKind::ResourceUnavailable,
                        detail,
                    ));
                }
            }
            if recover_after_failures {
                *run = this.handle_watchdog_recovery(
                    protocol,
                    &sandbox.request.sandbox_id,
                    lifecycle,
                    run,
                    None,
                )?;
            }
            Ok(())
        })
    }

    pub(super) fn apply_interleaved_timeout_recovery(
        &mut self,
        protocol: &ClapBlockProtocol,
        sandbox: &ServerDemoPluginSandboxAssembly,
        run: &mut LifecycleRunSummary,
        lifecycle: &mut ClapSandboxLifecycleHarness,
    ) -> Result<(), RuntimeError> {
        self.walk_timeout_watchdog(protocol, sandbox, run, lifecycle, |this, run, lifecycle| {
            let first_error = this.handle_watchdog_recovery(
                protocol,
                &sandbox.request.sandbox_id,
                lifecycle,
                run,
                Some(RecoveryFailureInjection::DeferredOldTransportTeardown),
            );
            if first_error.is_ok() {
                return Err(RuntimeError::new(
                    RuntimeErrorKind::ResourceUnavailable,
                    "expected deferred teardown recovery failure",
                ));
            }
            match this.handle_watchdog_recovery(
                protocol,
                &sandbox.request.sandbox_id,
                lifecycle,
                run,
                Some(RecoveryFailureInjection::CompetingOverlapAttach),
            ) {
                Ok(_) => Err(RuntimeError::new(
                    RuntimeErrorKind::ResourceUnavailable,
                    "expected interleaved recovery failures",
                )),
                Err(error) => Err(error),
            }
        })
    }

    pub(super) fn apply_repeated_watchdog_recovery(
        &mut self,
        protocol: &ClapBlockProtocol,
        sandbox: &ServerDemoPluginSandboxAssembly,
        run: &mut LifecycleRunSummary,
        lifecycle: &mut ClapSandboxLifecycleHarness,
        restart_episodes: u32,
        mixed_faults: bool,
    ) -> Result<(), RuntimeError> {
        for restart_episode in 0..restart_episodes {
            let episode_fault = if mixed_faults && restart_episode % 2 == 1 {
                FaultInjection::Timeout
            } else {
                FaultInjection::HeartbeatMiss
            };
            for _ in 0..WATCHDOG_TRIGGER_WINDOW_BLOCKS {
                let block_sequence = self.runtime.allocate_block_sequence();
                let (simulate_heartbeat_miss, simulate_timeout) = match episode_fault {
                    FaultInjection::HeartbeatMiss => (true, false),
                    FaultInjection::Timeout => (false, true),
                    _ => (false, false),
                };
                let outcome = self.run_realtime_cycle(
                    protocol,
                    run,
                    block_sequence,
                    lifecycle,
                    simulate_heartbeat_miss,
                    simulate_timeout,
                )?;
                let watchdog_fired = match episode_fault {
                    FaultInjection::HeartbeatMiss => {
                        outcome.is_none() && run.current_watchdog_triggered
                    }
                    FaultInjection::Timeout => outcome.as_ref().is_some_and(|outcome| {
                        outcome.result.slot.state == CompletionState::TimedOut
                            && run.current_watchdog_triggered
                    }),
                    _ => false,
                };
                if watchdog_fired {
                    let failure = build_fault_envelope(
                        &sandbox.request.sandbox_id,
                        "instance:server:default",
                        &run.shared_memory_lease_id,
                        run.processing_epoch,
                        episode_fault,
                    );
                    record_runtime_fault(&mut self.runtime, &failure);
                    if matches!(episode_fault, FaultInjection::Timeout) {
                        self.runtime.increment_xruns();
                    }
                    *run = self.handle_watchdog_recovery(
                        protocol,
                        &sandbox.request.sandbox_id,
                        lifecycle,
                        run,
                        None,
                    )?;
                    if restart_episode + 1 < restart_episodes {
                        self.execute_block_sequence(
                            protocol,
                            run,
                            INTER_EPISODE_CONTINUITY_BLOCKS,
                            lifecycle,
                            false,
                        )?;
                    }
                    break;
                }
            }
        }
        Ok(())
    }

    pub(super) fn walk_timeout_watchdog<F>(
        &mut self,
        protocol: &ClapBlockProtocol,
        sandbox: &ServerDemoPluginSandboxAssembly,
        run: &mut LifecycleRunSummary,
        lifecycle: &mut ClapSandboxLifecycleHarness,
        on_trigger: F,
    ) -> Result<(), RuntimeError>
    where
        F: FnOnce(
            &mut ServerRuntimeHost,
            &mut LifecycleRunSummary,
            &mut ClapSandboxLifecycleHarness,
        ) -> Result<(), RuntimeError>,
    {
        let mut on_trigger = Some(on_trigger);
        for _ in 0..WATCHDOG_TRIGGER_WINDOW_BLOCKS {
            let block_sequence = self.runtime.allocate_block_sequence();
            let timeout_result =
                self.run_realtime_cycle(protocol, run, block_sequence, lifecycle, false, true)?;
            if let Some(outcome) = timeout_result {
                if outcome.result.slot.state == CompletionState::TimedOut
                    && run.current_watchdog_triggered
                {
                    let failure = build_fault_envelope(
                        &sandbox.request.sandbox_id,
                        "instance:server:default",
                        &run.shared_memory_lease_id,
                        run.processing_epoch,
                        FaultInjection::Timeout,
                    );
                    record_runtime_fault(&mut self.runtime, &failure);
                    self.runtime.increment_xruns();
                    if let Some(on_trigger) = on_trigger.take() {
                        return on_trigger(self, run, lifecycle);
                    }
                }
            }
        }
        Ok(())
    }
}
