use signal_plugin::CompletionState;
use signal_plugin_clap::{ClapBlockProtocol, ClapSandboxLifecycleHarness};
use signal_runtime::{RecoveryRestartIntent, RuntimeError};

use super::super::{
    FaultInjection, LocalRuntimeHost, RecoveryFailureInjection, STEADY_STATE_BLOCKS,
    WATCHDOG_TRIGGER_WINDOW_BLOCKS,
};
use super::demo::LocalDemoPluginSandboxAssembly;
use super::lifecycle_run::LifecycleRunSummary;
use super::{
    build_fault_envelope, record_runtime_fault, RepeatedWatchdogRecoveryPlan,
    TimeoutRecoveryRetryPlan,
};

impl LocalRuntimeHost {
    pub(crate) fn apply_boot_fault_recovery(
        &mut self,
        protocol: &ClapBlockProtocol,
        sandbox: &LocalDemoPluginSandboxAssembly,
        run: &mut LifecycleRunSummary,
        lifecycle: &mut ClapSandboxLifecycleHarness,
        fault: FaultInjection,
    ) -> Result<bool, RuntimeError> {
        match fault {
            FaultInjection::Timeout => {
                for _ in 0..WATCHDOG_TRIGGER_WINDOW_BLOCKS {
                    let block_sequence = self.runtime.allocate_block_sequence();
                    let timeout_result = self.run_realtime_cycle(
                        protocol,
                        run,
                        block_sequence,
                        lifecycle,
                        false,
                        true,
                    )?;
                    if let Some(outcome) = timeout_result {
                        if outcome.result.slot.state == CompletionState::TimedOut
                            && run.current_watchdog_triggered
                        {
                            let failure = build_fault_envelope(
                                sandbox.request.sandbox_id.as_str(),
                                "instance:local:default",
                                &run.shared_memory_lease_id,
                                run.processing_epoch,
                                fault,
                            );
                            record_runtime_fault(&mut self.runtime, &failure);
                            self.runtime.increment_xruns();
                            *run = self.handle_watchdog_recovery(
                                protocol,
                                &sandbox.request.sandbox_id,
                                lifecycle,
                                run,
                                None,
                            )?;
                            break;
                        }
                    }
                }
            }
            FaultInjection::Crash => {
                let crash_sequence = self.runtime.allocate_block_sequence();
                let _ = self.run_realtime_cycle(
                    protocol,
                    run,
                    crash_sequence,
                    lifecycle,
                    false,
                    false,
                )?;
                let failure = build_fault_envelope(
                    &sandbox.request.sandbox_id,
                    "instance:local:default",
                    &run.shared_memory_lease_id,
                    run.processing_epoch,
                    fault,
                );
                record_runtime_fault(&mut self.runtime, &failure);
                *run = self.recover_sandbox(
                    protocol,
                    &sandbox.request.sandbox_id,
                    lifecycle,
                    run,
                    RecoveryRestartIntent::CrashRecovery,
                    None,
                )?;
            }
            FaultInjection::HeartbeatMiss => {
                for _ in 0..WATCHDOG_TRIGGER_WINDOW_BLOCKS {
                    let block_sequence = self.runtime.allocate_block_sequence();
                    let outcome = self.run_realtime_cycle(
                        protocol,
                        run,
                        block_sequence,
                        lifecycle,
                        true,
                        false,
                    )?;
                    if outcome.is_none() && run.current_watchdog_triggered {
                        let failure = build_fault_envelope(
                            &sandbox.request.sandbox_id,
                            "instance:local:default",
                            &run.shared_memory_lease_id,
                            run.processing_epoch,
                            fault,
                        );
                        record_runtime_fault(&mut self.runtime, &failure);
                        *run = self.handle_watchdog_recovery(
                            protocol,
                            &sandbox.request.sandbox_id,
                            lifecycle,
                            run,
                            None,
                        )?;
                        break;
                    }
                }
            }
            FaultInjection::DeviceLoss => {
                self.execute_block_sequence(protocol, run, 2, lifecycle, false)?;
                self.handle_device_loss_transition(false)?;
                self.execute_block_sequence(
                    protocol,
                    run,
                    STEADY_STATE_BLOCKS.saturating_sub(2),
                    lifecycle,
                    false,
                )?;
                return Ok(true);
            }
            FaultInjection::DeviceLossRestartFailure => {
                self.execute_block_sequence(protocol, run, 2, lifecycle, false)?;
                self.handle_device_loss_transition(true)?;
            }
            FaultInjection::RecoveryTeardownFailure => {
                self.apply_timeout_recovery_failure(
                    protocol,
                    sandbox,
                    run,
                    lifecycle,
                    RecoveryFailureInjection::OldTransportTeardown,
                    "expected injected recovery teardown failure",
                )?;
            }
            FaultInjection::RecoveryDeferredTeardownFailure => {
                self.apply_timeout_recovery_failure(
                    protocol,
                    sandbox,
                    run,
                    lifecycle,
                    RecoveryFailureInjection::DeferredOldTransportTeardown,
                    "expected deferred teardown recovery failure",
                )?;
            }
            FaultInjection::RecoveryDeferredTeardownThenCleanup => {
                self.apply_retrying_timeout_recovery(
                    protocol,
                    sandbox,
                    run,
                    lifecycle,
                    TimeoutRecoveryRetryPlan {
                        failures: &[RecoveryFailureInjection::DeferredOldTransportTeardown],
                        terminal_detail: "expected deferred teardown recovery failure",
                        recover_after_failures: true,
                    },
                )?;
            }
            FaultInjection::RecoveryDeferredTeardownCleanupRetry => {
                self.apply_retrying_timeout_recovery(
                    protocol,
                    sandbox,
                    run,
                    lifecycle,
                    TimeoutRecoveryRetryPlan {
                        failures: &[
                            RecoveryFailureInjection::DeferredOldTransportTeardown,
                            RecoveryFailureInjection::LingeringCleanupTeardown,
                        ],
                        terminal_detail: "expected lingering cleanup retry failure",
                        recover_after_failures: true,
                    },
                )?;
            }
            FaultInjection::RecoveryRestartFailure => {
                self.apply_timeout_recovery_failure(
                    protocol,
                    sandbox,
                    run,
                    lifecycle,
                    RecoveryFailureInjection::ReplacementStart,
                    "expected injected replacement start failure",
                )?;
            }
            FaultInjection::RecoveryOverlapContention => {
                self.apply_timeout_recovery_failure(
                    protocol,
                    sandbox,
                    run,
                    lifecycle,
                    RecoveryFailureInjection::CompetingOverlapAttach,
                    "expected injected overlap contention failure",
                )?;
            }
            FaultInjection::RecoveryInterleavedFailures => {
                self.apply_interleaved_timeout_recovery(protocol, sandbox, run, lifecycle)?;
            }
            FaultInjection::EscalatingHeartbeatMisses { restart_episodes } => {
                self.apply_repeated_watchdog_recovery(
                    protocol,
                    sandbox,
                    run,
                    lifecycle,
                    RepeatedWatchdogRecoveryPlan {
                        restart_episodes,
                        mixed_faults: false,
                    },
                )?;
            }
            FaultInjection::MixedWatchdogEpisodes { restart_episodes } => {
                self.apply_repeated_watchdog_recovery(
                    protocol,
                    sandbox,
                    run,
                    lifecycle,
                    RepeatedWatchdogRecoveryPlan {
                        restart_episodes,
                        mixed_faults: true,
                    },
                )?;
            }
        }
        Ok(false)
    }
}
