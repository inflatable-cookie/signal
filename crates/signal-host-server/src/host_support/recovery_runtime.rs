use signal_plugin_clap::{ClapBlockProtocol, ClapSandboxLifecycleHarness};
use signal_runtime::{
    complete_lingering_recovery_restart_or_rollback, rollback_recovery_overlap,
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
        rollback_recovery_overlap(&mut self.runtime);
        let restart_result = self.restart_plugin_sandbox(sandbox_id);

        let mut restarted_lifecycle = ClapSandboxLifecycleHarness::default();
        let mut restarted_run =
            self.run_lifecycle(protocol, sandbox_id, next_epoch, &mut restarted_lifecycle)?;
        restarted_run.apply_recovery_history(prior_history);
        let start_result = self.runtime.start();

        if let Err(error) = complete_lingering_recovery_restart_or_rollback(
            &mut self.runtime,
            sandbox_id,
            restart_result,
            restarted_run.transport.as_ref().map(|transport| {
                (
                    restarted_run.shared_memory_lease_id.as_str(),
                    transport.region_id.as_str(),
                )
            }),
            start_result,
        ) {
            self.rollback_replacement_recovery_session(
                protocol,
                sandbox_id,
                &mut restarted_lifecycle,
                &restarted_run,
            );
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
