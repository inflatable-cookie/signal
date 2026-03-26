use signal_plugin_clap::{ClapBlockProtocol, ClapSandboxLifecycleHarness};
use signal_runtime::{
    BrokerFailureStage, PluginSandboxLifecycleStage, PluginSandboxTransportStage, RuntimeError,
    RuntimeSupervisorApi,
};

use super::super::{RecoveryFailureInjection, ServerRuntimeHost};
use super::{
    lifecycle_stage_for_request, record_runtime_fault, runtime_error_from_failure,
    runtime_error_from_io, LifecycleRunSummary,
};

impl ServerRuntimeHost {
    pub(crate) fn finish_recovery_overlap_transition(
        &mut self,
        protocol: &ClapBlockProtocol,
        sandbox_id: &str,
        lifecycle: &mut ClapSandboxLifecycleHarness,
        run: &LifecycleRunSummary,
        failure: Option<RecoveryFailureInjection>,
        replacement_lifecycle: &mut ClapSandboxLifecycleHarness,
        replacement_run: &LifecycleRunSummary,
    ) -> Result<(), RuntimeError> {
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
        self.runtime.record_plugin_sandbox_transport(
            sandbox_id,
            run.shared_memory_lease_id.as_str(),
            current_transport.region_id.as_str(),
            PluginSandboxTransportStage::DetachRequested,
            Some(run.processing_epoch),
            None,
        );
        if matches!(
            failure,
            Some(RecoveryFailureInjection::DeferredOldTransportTeardown)
        ) {
            let error =
                std::io::Error::other("deferred old transport teardown during recovery retry");
            self.runtime.record_broker_failure(
                sandbox_id,
                Some(run.shared_memory_lease_id.clone()),
                Some(run.processing_epoch),
                Some(run.last_block_sequence),
                BrokerFailureStage::TransportTeardown,
                error.to_string(),
            );
            self.runtime.record_plugin_sandbox_transport(
                sandbox_id,
                run.shared_memory_lease_id.as_str(),
                current_transport.region_id.as_str(),
                PluginSandboxTransportStage::DetachFault,
                Some(run.processing_epoch),
                Some(error.to_string()),
            );
            self.rollback_replacement_recovery_session(
                protocol,
                sandbox_id,
                replacement_lifecycle,
                replacement_run,
            );
            self.runtime.set_active_plugin_sandboxes(1);
            return Err(runtime_error_from_io(error));
        }
        if let Err(error) = self.broker.destroy_region(current_transport) {
            self.runtime.record_broker_failure(
                sandbox_id,
                Some(run.shared_memory_lease_id.clone()),
                Some(run.processing_epoch),
                Some(run.last_block_sequence),
                BrokerFailureStage::TransportDestroy,
                error.to_string(),
            );
            self.runtime.record_plugin_sandbox_transport(
                sandbox_id,
                run.shared_memory_lease_id.as_str(),
                current_transport.region_id.as_str(),
                PluginSandboxTransportStage::DetachFault,
                Some(run.processing_epoch),
                Some(error.to_string()),
            );
            self.runtime.end_transport_session(
                sandbox_id,
                run.shared_memory_lease_id.as_str(),
                current_transport.region_id.as_str(),
            );
            self.rollback_replacement_recovery_session(
                protocol,
                sandbox_id,
                replacement_lifecycle,
                replacement_run,
            );
            self.runtime.set_active_plugin_sandboxes(0);
            return Err(runtime_error_from_io(error));
        }
        if matches!(
            failure,
            Some(RecoveryFailureInjection::OldTransportTeardown)
        ) {
            let error = std::io::Error::other(
                "injected old transport teardown failure during overlap recovery",
            );
            self.runtime.record_broker_failure(
                sandbox_id,
                Some(run.shared_memory_lease_id.clone()),
                Some(run.processing_epoch),
                Some(run.last_block_sequence),
                BrokerFailureStage::TransportTeardown,
                error.to_string(),
            );
            self.runtime.record_plugin_sandbox_transport(
                sandbox_id,
                run.shared_memory_lease_id.as_str(),
                current_transport.region_id.as_str(),
                PluginSandboxTransportStage::DetachFault,
                Some(run.processing_epoch),
                Some(error.to_string()),
            );
            self.runtime.end_transport_session(
                sandbox_id,
                run.shared_memory_lease_id.as_str(),
                current_transport.region_id.as_str(),
            );
            self.rollback_replacement_recovery_session(
                protocol,
                sandbox_id,
                replacement_lifecycle,
                replacement_run,
            );
            self.runtime.set_active_plugin_sandboxes(0);
            return Err(runtime_error_from_io(error));
        }
        if let Err(error) = lifecycle.teardown_active_transport() {
            self.runtime.record_broker_failure(
                sandbox_id,
                Some(run.shared_memory_lease_id.clone()),
                Some(run.processing_epoch),
                Some(run.last_block_sequence),
                BrokerFailureStage::TransportTeardown,
                error.to_string(),
            );
            self.runtime.record_plugin_sandbox_transport(
                sandbox_id,
                run.shared_memory_lease_id.as_str(),
                current_transport.region_id.as_str(),
                PluginSandboxTransportStage::DetachFault,
                Some(run.processing_epoch),
                Some(error.to_string()),
            );
            self.runtime.end_transport_session(
                sandbox_id,
                run.shared_memory_lease_id.as_str(),
                current_transport.region_id.as_str(),
            );
            self.rollback_replacement_recovery_session(
                protocol,
                sandbox_id,
                replacement_lifecycle,
                replacement_run,
            );
            self.runtime.set_active_plugin_sandboxes(0);
            return Err(runtime_error_from_io(error));
        }
        self.runtime.record_plugin_sandbox_transport(
            sandbox_id,
            run.shared_memory_lease_id.as_str(),
            current_transport.region_id.as_str(),
            PluginSandboxTransportStage::Detached,
            Some(run.processing_epoch),
            None,
        );
        self.runtime.end_transport_session(
            sandbox_id,
            run.shared_memory_lease_id.as_str(),
            current_transport.region_id.as_str(),
        );
        self.runtime.record_plugin_sandbox_lifecycle(
            sandbox_id,
            PluginSandboxLifecycleStage::TransportTornDown,
            Some(run.processing_epoch),
        );
        self.complete_recovery_overlap_restart(
            protocol,
            sandbox_id,
            failure,
            replacement_lifecycle,
            replacement_run,
        )
    }
}
