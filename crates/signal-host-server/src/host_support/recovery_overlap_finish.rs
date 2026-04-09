use signal_plugin_clap::{ClapBlockProtocol, ClapSandboxLifecycleHarness};
use signal_runtime::{
    handle_recovery_overlap_old_transport_teardown, RecoveryOverlapOldTransportTeardownOutcome,
    RuntimeError, RuntimeSupervisorApi,
};

use super::super::{RecoveryFailureInjection, ServerRuntimeHost};
use super::{
    lifecycle_stage_for_request, record_runtime_fault, runtime_error_from_failure,
    LifecycleRunSummary,
};

pub(crate) struct RecoveryOverlapTransition<'a> {
    pub(crate) sandbox_id: &'a str,
    pub(crate) lifecycle: &'a mut ClapSandboxLifecycleHarness,
    pub(crate) run: &'a LifecycleRunSummary,
    pub(crate) failure: Option<RecoveryFailureInjection>,
    pub(crate) replacement_lifecycle: &'a mut ClapSandboxLifecycleHarness,
    pub(crate) replacement_run: &'a LifecycleRunSummary,
}

impl ServerRuntimeHost {
    pub(crate) fn finish_recovery_overlap_transition(
        &mut self,
        protocol: &ClapBlockProtocol,
        transition: RecoveryOverlapTransition<'_>,
    ) -> Result<(), RuntimeError> {
        let RecoveryOverlapTransition {
            sandbox_id,
            lifecycle,
            run,
            failure,
            replacement_lifecycle,
            replacement_run,
        } = transition;
        let current_transport = run.transport.as_ref().ok_or_else(|| {
            RuntimeError::new(
                signal_runtime::RuntimeErrorKind::ResourceUnavailable,
                "lifecycle completed without brokered shared-memory transport",
            )
        })?;

        for request in protocol.teardown_sequence(sandbox_id, run.processing_epoch) {
            match lifecycle.handle(request.clone()) {
                Ok(_) => {
                    if let Some(stage) = lifecycle_stage_for_request(&request.payload) {
                        self.runtime.record_plugin_sandbox_lifecycle(
                            sandbox_id,
                            stage,
                            Some(run.processing_epoch),
                        );
                    }
                }
                Err(failure) => {
                    record_runtime_fault(&mut self.runtime, &failure);
                    self.rollback_replacement_recovery_session(
                        protocol,
                        sandbox_id,
                        replacement_lifecycle,
                        replacement_run,
                    );
                    self.runtime.set_active_plugin_sandboxes(0);
                    return Err(runtime_error_from_failure(&failure));
                }
            }
        }
        self.runtime.set_active_plugin_sandboxes(1);
        if let Err(error) = self.teardown_plugin_sandbox(sandbox_id) {
            self.rollback_replacement_recovery_session(
                protocol,
                sandbox_id,
                replacement_lifecycle,
                replacement_run,
            );
            self.runtime.set_active_plugin_sandboxes(0);
            return Err(error);
        }
        let deferred_teardown_failure = matches!(
            failure,
            Some(RecoveryFailureInjection::DeferredOldTransportTeardown)
        );
        let destroy_result = if deferred_teardown_failure {
            Ok(())
        } else {
            self.broker
                .destroy_region(current_transport)
                .map_err(|error| error.to_string())
        };
        let injected_old_transport_teardown_failure = matches!(
            failure,
            Some(RecoveryFailureInjection::OldTransportTeardown)
        );
        let transport_teardown_result =
            if deferred_teardown_failure || injected_old_transport_teardown_failure {
                Ok(())
            } else {
                lifecycle
                    .teardown_active_transport()
                    .map_err(|error| error.to_string())
            };
        match handle_recovery_overlap_old_transport_teardown(
            &mut self.runtime,
            sandbox_id,
            run.shared_memory_lease_id.as_str(),
            current_transport.region_id.as_str(),
            run.processing_epoch,
            run.last_block_sequence,
            deferred_teardown_failure,
            destroy_result,
            injected_old_transport_teardown_failure,
            transport_teardown_result,
        ) {
            RecoveryOverlapOldTransportTeardownOutcome::Continue => {}
            RecoveryOverlapOldTransportTeardownOutcome::RollbackKeepReplacement(error) => {
                self.rollback_replacement_recovery_session(
                    protocol,
                    sandbox_id,
                    replacement_lifecycle,
                    replacement_run,
                );
                self.runtime.set_active_plugin_sandboxes(1);
                return Err(error);
            }
            RecoveryOverlapOldTransportTeardownOutcome::RollbackClearOverlap(error) => {
                self.rollback_replacement_recovery_session(
                    protocol,
                    sandbox_id,
                    replacement_lifecycle,
                    replacement_run,
                );
                self.runtime.set_active_plugin_sandboxes(0);
                return Err(error);
            }
        }
        self.complete_recovery_overlap_restart(
            protocol,
            sandbox_id,
            failure,
            replacement_lifecycle,
            replacement_run,
        )
    }
}
