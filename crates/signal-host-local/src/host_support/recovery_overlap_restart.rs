use signal_runtime::{RuntimeError, RuntimeLifecycleApi, RuntimeSupervisorApi};

use super::super::{LocalRuntimeHost, RecoveryFailureInjection};
use super::LifecycleRunSummary;

impl LocalRuntimeHost {
    pub(crate) fn complete_recovery_overlap_restart(
        &mut self,
        protocol: &signal_plugin_clap::ClapBlockProtocol,
        sandbox_id: &str,
        failure: Option<RecoveryFailureInjection>,
        replacement_lifecycle: &mut signal_plugin_clap::ClapSandboxLifecycleHarness,
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
        if let Some(transport) = replacement_run.transport.as_ref() {
            self.runtime.promote_transport_session_to_steady_state(
                sandbox_id,
                replacement_run.shared_memory_lease_id.as_str(),
                transport.region_id.as_str(),
            );
        }
        Ok(())
    }
}
