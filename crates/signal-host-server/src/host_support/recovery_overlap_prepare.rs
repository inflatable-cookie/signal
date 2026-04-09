use signal_plugin_clap::{ClapBlockProtocol, ClapSandboxLifecycleHarness};
use signal_runtime::{
    begin_recovery_overlap, handle_overlap_prepare_contention, rollback_recovery_overlap,
    RuntimeError,
};

use super::super::{RecoveryFailureInjection, ServerRuntimeHost};
use super::LifecycleRunSummary;

impl ServerRuntimeHost {
    pub(crate) fn prepare_replacement_recovery_session(
        &mut self,
        protocol: &ClapBlockProtocol,
        sandbox_id: &str,
        run: &LifecycleRunSummary,
        next_epoch: u64,
        failure: Option<RecoveryFailureInjection>,
        replacement_lifecycle: &mut ClapSandboxLifecycleHarness,
    ) -> Result<LifecycleRunSummary, RuntimeError> {
        let mut replacement_run =
            self.run_lifecycle(protocol, sandbox_id, next_epoch, replacement_lifecycle)?;
        replacement_run.apply_recovery_history(run.recovery_history());
        begin_recovery_overlap(&mut self.runtime);
        let contention_requested = matches!(
            failure,
            Some(RecoveryFailureInjection::CompetingOverlapAttach)
        );
        let competing_attach_result = if contention_requested {
            let mut competing_lifecycle = ClapSandboxLifecycleHarness::default();
            match self.run_lifecycle(
                protocol,
                sandbox_id,
                next_epoch.saturating_add(1),
                &mut competing_lifecycle,
            ) {
                Ok(competing_run) => {
                    self.rollback_replacement_recovery_session(
                        protocol,
                        sandbox_id,
                        &mut competing_lifecycle,
                        &competing_run,
                    );
                    Ok(())
                }
                Err(error) => Err(error),
            }
        } else {
            Ok(())
        };
        if let Err(error) =
            handle_overlap_prepare_contention(contention_requested, competing_attach_result)
        {
            self.rollback_replacement_recovery_session(
                protocol,
                sandbox_id,
                replacement_lifecycle,
                &replacement_run,
            );
            rollback_recovery_overlap(&mut self.runtime);
            return Err(error);
        }
        Ok(replacement_run)
    }
}
