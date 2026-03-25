use signal_plugin_clap::{ClapBlockProtocol, ClapSandboxLifecycleHarness};
use signal_runtime::RuntimeError;

use super::super::{LifecycleRunSummary, RecoveryFailureInjection, ServerRuntimeHost};

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
        self.runtime.set_active_plugin_sandboxes(2);
        if matches!(
            failure,
            Some(RecoveryFailureInjection::CompetingOverlapAttach)
        ) {
            let mut competing_lifecycle = ClapSandboxLifecycleHarness::default();
            let contention_error = match self.run_lifecycle(
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
                    RuntimeError::new(
                        signal_runtime::RuntimeErrorKind::ResourceUnavailable,
                        "expected overlapping replacement attach contention",
                    )
                }
                Err(error) => error,
            };
            self.rollback_replacement_recovery_session(
                protocol,
                sandbox_id,
                replacement_lifecycle,
                &replacement_run,
            );
            return Err(contention_error);
        }
        Ok(replacement_run)
    }
}
