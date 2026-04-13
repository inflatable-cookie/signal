use signal_ipc::{
    SharedMemoryRegionLifecycleError, SharedMemoryRegionLifecycleErrorKind,
    SharedMemoryTransportPayload,
};
use signal_plugin_clap::ClapSandboxLifecycleHarness;
use signal_runtime::{
    complete_broker_transport_detach, record_broker_transport_detach_failure,
    record_broker_transport_detach_requested, RuntimeError,
};

use super::super::{LocalRuntimeHost, RecoveryFailureInjection};
use super::{runtime_error_from_io, LifecycleRunSummary};

impl LocalRuntimeHost {
    pub(crate) fn cleanup_orphan_lingering_transport(
        &mut self,
        session: &signal_runtime::ActiveTransportConcurrencySession,
        processing_epoch: u64,
    ) -> Result<(), RuntimeError> {
        let Some(backing_path) = session.backing_path.clone() else {
            let error = RuntimeError::new(
                signal_runtime::RuntimeErrorKind::ResourceUnavailable,
                "orphan lingering transport is missing backing_path metadata",
            );
            record_broker_transport_detach_failure(
                &mut self.runtime,
                session.sandbox_id.as_str(),
                session.lease_id.as_str(),
                session.region_id.as_str(),
                processing_epoch,
                None,
                signal_runtime::BrokerFailureStage::TransportTeardown,
                error.message.clone(),
            );
            return Err(error);
        };
        let Some(total_bytes) = session.total_bytes else {
            let error = RuntimeError::new(
                signal_runtime::RuntimeErrorKind::ResourceUnavailable,
                "orphan lingering transport is missing total_bytes metadata",
            );
            record_broker_transport_detach_failure(
                &mut self.runtime,
                session.sandbox_id.as_str(),
                session.lease_id.as_str(),
                session.region_id.as_str(),
                processing_epoch,
                None,
                signal_runtime::BrokerFailureStage::TransportTeardown,
                error.message.clone(),
            );
            return Err(error);
        };

        let transport = SharedMemoryTransportPayload {
            region_id: session.region_id.clone(),
            transport_kind: signal_ipc::SharedMemoryTransportKind::MappedFile,
            backing_path,
            total_bytes,
        };

        record_broker_transport_detach_requested(
            &mut self.runtime,
            session.sandbox_id.as_str(),
            session.lease_id.as_str(),
            session.region_id.as_str(),
            processing_epoch,
            "orphan lingering cleanup",
        );

        if let Err(error) = self.broker.destroy_region(&transport) {
            record_broker_transport_detach_failure(
                &mut self.runtime,
                session.sandbox_id.as_str(),
                session.lease_id.as_str(),
                session.region_id.as_str(),
                processing_epoch,
                None,
                signal_runtime::BrokerFailureStage::TransportDestroy,
                error.to_string(),
            );
            return Err(runtime_error_from_io(error.into()));
        }

        complete_broker_transport_detach(
            &mut self.runtime,
            session.sandbox_id.as_str(),
            session.lease_id.as_str(),
            session.region_id.as_str(),
            processing_epoch,
            "orphan lingering cleanup",
            false,
        );
        self.runtime.complete_lingering_cleanup_success(
            session.sandbox_id.as_str(),
            session.lease_id.as_str(),
            session.region_id.as_str(),
        );
        Ok(())
    }

    pub(crate) fn cleanup_lingering_origin_transport(
        &mut self,
        sandbox_id: &str,
        lifecycle: &mut ClapSandboxLifecycleHarness,
        run: &LifecycleRunSummary,
        failure: Option<RecoveryFailureInjection>,
    ) -> Result<(), RuntimeError> {
        let Some(current_transport) = run.transport.as_ref() else {
            return Ok(());
        };

        record_broker_transport_detach_requested(
            &mut self.runtime,
            sandbox_id,
            run.shared_memory_lease_id.as_str(),
            current_transport.region_id.as_str(),
            run.processing_epoch,
            "lingering cleanup retry",
        );

        if matches!(
            failure,
            Some(RecoveryFailureInjection::LingeringCleanupTeardown)
        ) {
            let error = std::io::Error::other("injected lingering cleanup retry failure");
            record_broker_transport_detach_failure(
                &mut self.runtime,
                sandbox_id,
                run.shared_memory_lease_id.as_str(),
                current_transport.region_id.as_str(),
                run.processing_epoch,
                Some(run.last_block_sequence),
                signal_runtime::BrokerFailureStage::TransportTeardown,
                error.to_string(),
            );
            return Err(runtime_error_from_io(error.into()));
        }

        if let Err(error) = self.broker.destroy_region(current_transport) {
            if error.kind() != SharedMemoryRegionLifecycleErrorKind::MissingMetadata {
                record_broker_transport_detach_failure(
                    &mut self.runtime,
                    sandbox_id,
                    run.shared_memory_lease_id.as_str(),
                    current_transport.region_id.as_str(),
                    run.processing_epoch,
                    Some(run.last_block_sequence),
                    signal_runtime::BrokerFailureStage::TransportDestroy,
                    error.to_string(),
                );
                return Err(runtime_error_from_io(error.into()));
            }
        }

        if let Err(error) = lifecycle.teardown_active_transport() {
            let missing_metadata = error
                .get_ref()
                .and_then(|source| source.downcast_ref::<SharedMemoryRegionLifecycleError>())
                .is_some_and(|error| {
                    error.kind() == SharedMemoryRegionLifecycleErrorKind::MissingMetadata
                });
            if !missing_metadata {
                record_broker_transport_detach_failure(
                    &mut self.runtime,
                    sandbox_id,
                    run.shared_memory_lease_id.as_str(),
                    current_transport.region_id.as_str(),
                    run.processing_epoch,
                    Some(run.last_block_sequence),
                    signal_runtime::BrokerFailureStage::TransportTeardown,
                    error.to_string(),
                );
                return Err(runtime_error_from_io(error));
            }
        }

        complete_broker_transport_detach(
            &mut self.runtime,
            sandbox_id,
            run.shared_memory_lease_id.as_str(),
            current_transport.region_id.as_str(),
            run.processing_epoch,
            "lingering cleanup retry",
            false,
        );
        Ok(())
    }
}
