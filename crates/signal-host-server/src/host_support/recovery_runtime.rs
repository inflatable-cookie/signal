use signal_plugin_clap::{ClapBlockProtocol, ClapSandboxLifecycleHarness};
use signal_runtime::{
    RecoveryRestartIntent, RuntimeError, RuntimeLifecycleApi, RuntimeObservationApi,
    RuntimeSupervisorApi, StopReason,
};

use super::super::{RecoveryFailureInjection, ServerRuntimeHost};
use super::{LifecycleRunSummary, RecoveryHistory};

pub(crate) struct LingeringSessionRecovery<'a> {
    pub(crate) sandbox_id: &'a str,
    pub(crate) lifecycle: &'a mut ClapSandboxLifecycleHarness,
    pub(crate) run: &'a LifecycleRunSummary,
    pub(crate) prior_history: RecoveryHistory,
    pub(crate) next_epoch: u64,
    pub(crate) failure: Option<RecoveryFailureInjection>,
}

impl ServerRuntimeHost {
    pub(crate) fn recover_from_lingering_session(
        &mut self,
        protocol: &ClapBlockProtocol,
        recovery: LingeringSessionRecovery<'_>,
    ) -> Result<LifecycleRunSummary, RuntimeError> {
        let LingeringSessionRecovery {
            sandbox_id,
            lifecycle,
            run,
            prior_history,
            next_epoch,
            failure,
        } = recovery;
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
        self.reconcile_late_lingering_sessions_after_start(sandbox_id, &restarted_run);

        *lifecycle = restarted_lifecycle;
        Ok(restarted_run)
    }

    pub(crate) fn stop_runtime_for_recovery(&mut self) -> Result<(), RuntimeError> {
        if self.runtime.get_control_snapshot().running {
            self.runtime.stop(StopReason::DegradedModeRecovery)
        } else {
            Ok(())
        }
    }

    pub(crate) fn handle_watchdog_recovery(
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
