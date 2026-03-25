use signal_plugin_clap::{ClapBlockProtocol, ClapSandboxLifecycleHarness};
use signal_runtime::{RecoveryRestartIntent, RuntimeError, StopReason};

use super::super::{LocalRuntimeHost, RecoveryFailureInjection};
use super::{LifecycleRunSummary, RecoveryHistory};

impl LocalRuntimeHost {
    fn recover_from_lingering_session(
        &mut self,
        protocol: &ClapBlockProtocol,
        sandbox_id: &str,
        lifecycle: &mut ClapSandboxLifecycleHarness,
        run: &LifecycleRunSummary,
        prior_history: RecoveryHistory,
        next_epoch: u64,
        failure: Option<RecoveryFailureInjection>,
    ) -> Result<LifecycleRunSummary, RuntimeError> {
        self.cleanup_lingering_origin_transport(sandbox_id, lifecycle, run, failure)?;
        self.cleanup_orphan_lingering_sessions_for_sandbox(
            sandbox_id,
            run.processing_epoch,
            Some(run.shared_memory_lease_id.as_str()),
            run.transport
                .as_ref()
                .map(|transport| transport.region_id.as_str()),
            signal_runtime::LingeringCleanupMode::StrictPreAttach,
        )?;
        self.runtime.set_active_plugin_sandboxes(0);
        self.restart_plugin_sandbox(sandbox_id)?;
        self.runtime.set_active_plugin_sandboxes(1);

        let mut restarted_lifecycle = ClapSandboxLifecycleHarness::default();
        let mut restarted_run =
            self.run_lifecycle(protocol, sandbox_id, next_epoch, &mut restarted_lifecycle)?;
        restarted_run.apply_recovery_history(prior_history);

        if let Err(error) = self.runtime.start() {
            self.rollback_replacement_recovery_session(
                protocol,
                sandbox_id,
                &mut restarted_lifecycle,
                &restarted_run,
            );
            self.runtime.set_active_plugin_sandboxes(0);
            return Err(error);
        }
        if let Some(transport) = restarted_run.transport.as_ref() {
            self.runtime.promote_transport_session_to_steady_state(
                sandbox_id,
                restarted_run.shared_memory_lease_id.as_str(),
                transport.region_id.as_str(),
            );
        }
        self.reconcile_late_lingering_sessions_after_start(sandbox_id, &restarted_run);

        *lifecycle = restarted_lifecycle;
        Ok(restarted_run)
    }

    fn stop_runtime_with_reason(&mut self, reason: StopReason) -> Result<(), RuntimeError> {
        if self.runtime.get_control_snapshot().running {
            self.audio_pump.stop();
            self.supervisor.last_stop_reason = Some(reason);
            self.runtime.stop(reason)
        } else {
            Ok(())
        }
    }

    fn stop_runtime_for_recovery(&mut self) -> Result<(), RuntimeError> {
        self.stop_runtime_with_reason(StopReason::DegradedModeRecovery)
    }

    fn handle_device_loss_transition(
        &mut self,
        restart_should_fail: bool,
    ) -> Result<(), RuntimeError> {
        self.coreaudio
            .simulate_device_loss("simulated CoreAudio device disconnect");
        self.stop_runtime_with_reason(StopReason::DeviceReconfigure)?;
        self.audio_pump.fault();
        self.coreaudio
            .simulate_restart_attempt("simulated CoreAudio device restart attempt");

        if restart_should_fail {
            self.coreaudio
                .simulate_restart_failure("simulated CoreAudio device restart failure");
            return Err(RuntimeError::new(
                signal_runtime::RuntimeErrorKind::HardwareFailure,
                "simulated device-loss restart failure",
            ));
        }

        self.prepare_default_output_hardware()?;
        self.runtime.start()?;
        self.coreaudio.mark_recovered();
        Ok(())
    }

    fn handle_watchdog_recovery(
        &mut self,
        protocol: &ClapBlockProtocol,
        sandbox_id: &str,
        lifecycle: &mut ClapSandboxLifecycleHarness,
        run: &LifecycleRunSummary,
        failure: Option<RecoveryFailureInjection>,
    ) -> Result<LifecycleRunSummary, RuntimeError> {
        self.recover_sandbox(
            protocol,
            sandbox_id,
            lifecycle,
            run,
            RecoveryRestartIntent::WatchdogRecovery,
            failure,
        )
    }
}
