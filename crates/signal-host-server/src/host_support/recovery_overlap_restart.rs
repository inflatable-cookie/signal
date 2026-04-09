use signal_plugin_clap::{ClapBlockProtocol, ClapSandboxLifecycleHarness};
use signal_runtime::{
    complete_recovery_overlap_restart, complete_recovery_overlap_restart_or_rollback,
    rollback_recovery_overlap, RuntimeError, RuntimeLifecycleApi, RuntimeSupervisorApi,
};

use super::super::{RecoveryFailureInjection, ServerRuntimeHost};
use super::LifecycleRunSummary;

impl ServerRuntimeHost {
    pub(crate) fn complete_recovery_overlap_restart(
        &mut self,
        protocol: &ClapBlockProtocol,
        sandbox_id: &str,
        failure: Option<RecoveryFailureInjection>,
        replacement_lifecycle: &mut ClapSandboxLifecycleHarness,
        replacement_run: &LifecycleRunSummary,
    ) -> Result<(), RuntimeError> {
        let restart_result = self.restart_plugin_sandbox(sandbox_id);
        if restart_result.is_ok() {
            complete_recovery_overlap_restart(&mut self.runtime, sandbox_id, None, None);
        }
        let inject_replacement_start_failure =
            matches!(failure, Some(RecoveryFailureInjection::ReplacementStart));
        let start_result = if restart_result.is_ok() && !inject_replacement_start_failure {
            Some(self.runtime.start())
        } else {
            None
        };
        if let Err(error) = complete_recovery_overlap_restart_or_rollback(
            restart_result,
            inject_replacement_start_failure,
            start_result,
        ) {
            self.rollback_replacement_recovery_session(
                protocol,
                sandbox_id,
                replacement_lifecycle,
                replacement_run,
            );
            rollback_recovery_overlap(&mut self.runtime);
            return Err(error);
        }
        complete_recovery_overlap_restart(
            &mut self.runtime,
            sandbox_id,
            replacement_run
                .transport
                .as_ref()
                .map(|_| replacement_run.shared_memory_lease_id.as_str()),
            replacement_run
                .transport
                .as_ref()
                .map(|transport| transport.region_id.as_str()),
        );
        Ok(())
    }
}
