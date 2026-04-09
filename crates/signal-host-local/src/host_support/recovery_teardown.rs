use signal_plugin_clap::{ClapBlockProtocol, ClapSandboxLifecycleHarness};
use signal_runtime::{finalize_brokered_recovery_transport_detach, RuntimeSupervisorApi};

use super::super::LocalRuntimeHost;
use super::{lifecycle_stage_for_request, record_runtime_fault, LifecycleRunSummary};

impl LocalRuntimeHost {
    fn teardown_recovery_session(
        &mut self,
        protocol: &ClapBlockProtocol,
        sandbox_id: &str,
        lifecycle: &mut ClapSandboxLifecycleHarness,
        run: &LifecycleRunSummary,
        detail: &str,
        teardown_plugin_sandbox: bool,
    ) {
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
                Err(failure) => record_runtime_fault(&mut self.runtime, &failure),
            }
        }

        let Some(transport) = run.transport.as_ref() else {
            return;
        };

        if teardown_plugin_sandbox {
            let _ = self.teardown_plugin_sandbox(sandbox_id);
        }

        let destroy_error = self
            .broker
            .destroy_region(transport)
            .err()
            .map(|error| error.to_string());
        let teardown_error = lifecycle
            .teardown_active_transport()
            .err()
            .map(|error| error.to_string());

        finalize_brokered_recovery_transport_detach(
            &mut self.runtime,
            sandbox_id,
            run.shared_memory_lease_id.as_str(),
            transport.region_id.as_str(),
            run.processing_epoch,
            run.last_block_sequence,
            detail,
            false,
            destroy_error,
            teardown_error,
        );
    }

    pub(crate) fn abort_origin_recovery_session(
        &mut self,
        protocol: &ClapBlockProtocol,
        sandbox_id: &str,
        lifecycle: &mut ClapSandboxLifecycleHarness,
        run: &LifecycleRunSummary,
    ) {
        self.teardown_recovery_session(
            protocol,
            sandbox_id,
            lifecycle,
            run,
            "origin recovery abort",
            true,
        );
    }

    pub(crate) fn rollback_replacement_recovery_session(
        &mut self,
        protocol: &ClapBlockProtocol,
        sandbox_id: &str,
        lifecycle: &mut ClapSandboxLifecycleHarness,
        run: &LifecycleRunSummary,
    ) {
        self.teardown_recovery_session(
            protocol,
            sandbox_id,
            lifecycle,
            run,
            "replacement rollback",
            false,
        );
    }
}
