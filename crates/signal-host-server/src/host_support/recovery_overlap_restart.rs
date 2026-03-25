use signal_plugin_clap::{ClapBlockProtocol, ClapSandboxLifecycleHarness};
use signal_runtime::{RuntimeError, RuntimeLifecycleApi, RuntimeSupervisorApi};

use super::super::{LifecycleRunSummary, RecoveryFailureInjection, ServerRuntimeHost};

impl ServerRuntimeHost {
    pub(crate) fn complete_recovery_overlap_restart(
        &mut self,
        protocol: &ClapBlockProtocol,
        sandbox_id: &str,
        failure: Option<RecoveryFailureInjection>,
        replacement_lifecycle: &mut ClapSandboxLifecycleHarness,
        replacement_run: &LifecycleRunSummary,
    ) -> Result<(), RuntimeError> {
        if let Err(error) = self.restart_plugin_sandbox(sandbox_id) {
            self.rollback_replacement_recovery_session(
                protocol,
                sandbox_id,
                replacement_lifecycle,
                replacement_run,
            );
            self.runtime.set_active_plugin_sandboxes(0);
            return Err(error);
        }
        self.runtime.set_active_plugin_sandboxes(1);
        if matches!(failure, Some(RecoveryFailureInjection::ReplacementStart)) {
            self.rollback_replacement_recovery_session(
                protocol,
                sandbox_id,
                replacement_lifecycle,
                replacement_run,
            );
            self.runtime.set_active_plugin_sandboxes(0);
            return Err(RuntimeError::new(
                signal_runtime::RuntimeErrorKind::ResourceUnavailable,
                "injected replacement start failure during overlap recovery",
            ));
        }
        if let Err(error) = self.runtime.start() {
            self.rollback_replacement_recovery_session(
                protocol,
                sandbox_id,
                replacement_lifecycle,
                replacement_run,
            );
            self.runtime.set_active_plugin_sandboxes(0);
            return Err(error);
        }
        Ok(())
    }
}
