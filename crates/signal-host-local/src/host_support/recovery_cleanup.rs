use signal_ipc::SharedMemoryTransportPayload;
use signal_plugin_clap::ClapSandboxLifecycleHarness;
use signal_runtime::{
    BrokerFailureStage, LingeringCleanupMode, PluginSandboxLifecycleStage,
    PluginSandboxTransportStage, RuntimeError,
};

use super::super::{LocalRuntimeHost, RecoveryFailureInjection};
use super::{record_runtime_fault, runtime_error_from_io, LifecycleRunSummary};

impl LocalRuntimeHost {
    fn session_is_lingering(&self, sandbox_id: &str, lease_id: &str, region_id: &str) -> bool {
        self.runtime
            .get_transport_concurrency_snapshot()
            .active_sessions
            .iter()
            .find(|session| {
                session.sandbox_id == sandbox_id
                    && session.lease_id == lease_id
                    && session.region_id == region_id
            })
            .is_some_and(|session| {
                matches!(
                    session.state,
                    signal_runtime::TransportSessionState::DetachRequested
                        | signal_runtime::TransportSessionState::DetachFaulted
                )
            })
    }

    fn cleanup_orphan_lingering_sessions_for_sandbox(
        &mut self,
        sandbox_id: &str,
        processing_epoch: u64,
        exclude_lease_id: Option<&str>,
        exclude_region_id: Option<&str>,
        mode: LingeringCleanupMode,
    ) -> Result<(), RuntimeError> {
        let trigger = match mode {
            LingeringCleanupMode::StrictPreAttach => {
                signal_runtime::LingeringCleanupTrigger::RecoveryPreAttach
            }
            LingeringCleanupMode::BestEffortPostStart => {
                signal_runtime::LingeringCleanupTrigger::PostStartReconciliation
            }
        };
        let _ = self.runtime.enqueue_lingering_cleanup_work(
            sandbox_id,
            mode,
            trigger,
            processing_epoch,
            exclude_lease_id,
            exclude_region_id,
        );
        while let Some(plan) = self
            .runtime
            .dequeue_lingering_cleanup_work_for_sandbox(sandbox_id, processing_epoch)
        {
            for session in plan.candidates {
                if let Err(error) =
                    self.cleanup_orphan_lingering_transport(&session, plan.processing_epoch)
                {
                    self.runtime.record_lingering_cleanup_failure(
                        session.sandbox_id.as_str(),
                        session.lease_id.as_str(),
                        session.region_id.as_str(),
                        plan.mode,
                        plan.processing_epoch,
                        error.message.as_str(),
                    );
                    if plan.mode == LingeringCleanupMode::StrictPreAttach {
                        return Err(error);
                    }
                }
            }
        }
        Ok(())
    }

    fn cleanup_orphan_lingering_transport(
        &mut self,
        session: &signal_runtime::ActiveTransportConcurrencySession,
        processing_epoch: u64,
    ) -> Result<(), RuntimeError> {
        let Some(backing_path) = session.backing_path.clone() else {
            let error = RuntimeError::new(
                signal_runtime::RuntimeErrorKind::ResourceUnavailable,
                "orphan lingering transport is missing backing_path metadata",
            );
            self.runtime.record_broker_failure(
                session.sandbox_id.as_str(),
                Some(session.lease_id.clone()),
                Some(processing_epoch),
                None,
                BrokerFailureStage::TransportTeardown,
                error.message.clone(),
            );
            self.runtime.record_plugin_sandbox_transport(
                session.sandbox_id.as_str(),
                session.lease_id.as_str(),
                session.region_id.as_str(),
                PluginSandboxTransportStage::DetachFault,
                Some(processing_epoch),
                Some(error.message.clone()),
            );
            return Err(error);
        };
        let Some(total_bytes) = session.total_bytes else {
            let error = RuntimeError::new(
                signal_runtime::RuntimeErrorKind::ResourceUnavailable,
                "orphan lingering transport is missing total_bytes metadata",
            );
            self.runtime.record_broker_failure(
                session.sandbox_id.as_str(),
                Some(session.lease_id.clone()),
                Some(processing_epoch),
                None,
                BrokerFailureStage::TransportTeardown,
                error.message.clone(),
            );
            self.runtime.record_plugin_sandbox_transport(
                session.sandbox_id.as_str(),
                session.lease_id.as_str(),
                session.region_id.as_str(),
                PluginSandboxTransportStage::DetachFault,
                Some(processing_epoch),
                Some(error.message.clone()),
            );
            return Err(error);
        };

        let transport = SharedMemoryTransportPayload {
            region_id: session.region_id.clone(),
            transport_kind: signal_ipc::SharedMemoryTransportKind::MappedFile,
            backing_path,
            total_bytes,
        };

        self.runtime.record_plugin_sandbox_transport(
            session.sandbox_id.as_str(),
            session.lease_id.as_str(),
            session.region_id.as_str(),
            PluginSandboxTransportStage::DetachRequested,
            Some(processing_epoch),
            Some("orphan lingering cleanup".into()),
        );

        if let Err(error) = self.broker.destroy_region(&transport) {
            self.runtime.record_broker_failure(
                session.sandbox_id.as_str(),
                Some(session.lease_id.clone()),
                Some(processing_epoch),
                None,
                BrokerFailureStage::TransportDestroy,
                error.to_string(),
            );
            self.runtime.record_plugin_sandbox_transport(
                session.sandbox_id.as_str(),
                session.lease_id.as_str(),
                session.region_id.as_str(),
                PluginSandboxTransportStage::DetachFault,
                Some(processing_epoch),
                Some(error.to_string()),
            );
            return Err(runtime_error_from_io(error));
        }

        self.runtime.record_plugin_sandbox_transport(
            session.sandbox_id.as_str(),
            session.lease_id.as_str(),
            session.region_id.as_str(),
            PluginSandboxTransportStage::Detached,
            Some(processing_epoch),
            Some("orphan lingering cleanup".into()),
        );
        self.runtime.complete_lingering_cleanup_success(
            session.sandbox_id.as_str(),
            session.lease_id.as_str(),
            session.region_id.as_str(),
        );
        self.runtime.record_plugin_sandbox_lifecycle(
            session.sandbox_id.as_str(),
            PluginSandboxLifecycleStage::TransportTornDown,
            Some(processing_epoch),
        );
        Ok(())
    }

    fn cleanup_lingering_origin_transport(
        &mut self,
        sandbox_id: &str,
        lifecycle: &mut ClapSandboxLifecycleHarness,
        run: &LifecycleRunSummary,
        failure: Option<RecoveryFailureInjection>,
    ) -> Result<(), RuntimeError> {
        let Some(current_transport) = run.transport.as_ref() else {
            return Ok(());
        };

        self.runtime.record_plugin_sandbox_transport(
            sandbox_id,
            run.shared_memory_lease_id.as_str(),
            current_transport.region_id.as_str(),
            PluginSandboxTransportStage::DetachRequested,
            Some(run.processing_epoch),
            Some("lingering cleanup retry".into()),
        );

        if matches!(
            failure,
            Some(RecoveryFailureInjection::LingeringCleanupTeardown)
        ) {
            let error = std::io::Error::other("injected lingering cleanup retry failure");
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
            return Err(runtime_error_from_io(error));
        }

        self.runtime.record_plugin_sandbox_transport(
            sandbox_id,
            run.shared_memory_lease_id.as_str(),
            current_transport.region_id.as_str(),
            PluginSandboxTransportStage::Detached,
            Some(run.processing_epoch),
            Some("lingering cleanup retry".into()),
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
        Ok(())
    }

    fn reconcile_late_lingering_sessions_after_start(
        &mut self,
        sandbox_id: &str,
        active_run: &LifecycleRunSummary,
    ) {
        let _ = self.cleanup_orphan_lingering_sessions_for_sandbox(
            sandbox_id,
            active_run.processing_epoch,
            Some(active_run.shared_memory_lease_id.as_str()),
            active_run
                .transport
                .as_ref()
                .map(|transport| transport.region_id.as_str()),
            LingeringCleanupMode::BestEffortPostStart,
        );
    }
}
