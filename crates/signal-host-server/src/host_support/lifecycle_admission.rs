use signal_ipc::SharedMemoryTransportPayload;
use signal_plugin_clap::{ClapBlockProtocol, ClapSandboxLifecycleHarness};
use signal_runtime::{
    BrokerFailureStage, PluginSandboxLifecycleStage, PluginSandboxTransportStage,
};

use super::super::ServerRuntimeHost;
use super::{lifecycle_stage_for_request, record_runtime_fault};

pub(crate) struct LifecycleAdmissionRollback<'a> {
    pub(crate) sandbox_id: &'a str,
    pub(crate) lifecycle: &'a mut ClapSandboxLifecycleHarness,
    pub(crate) processing_epoch: u64,
    pub(crate) lease_id: &'a str,
    pub(crate) transport: &'a SharedMemoryTransportPayload,
    pub(crate) detail: &'a str,
}

impl ServerRuntimeHost {
    pub(crate) fn rollback_unadmitted_lifecycle_setup(
        &mut self,
        protocol: &ClapBlockProtocol,
        rollback: LifecycleAdmissionRollback<'_>,
    ) {
        let LifecycleAdmissionRollback {
            sandbox_id,
            lifecycle,
            processing_epoch,
            lease_id,
            transport,
            detail,
        } = rollback;
        for request in protocol.teardown_sequence(sandbox_id, processing_epoch) {
            match lifecycle.handle(request.clone()) {
                Ok(_) => {
                    if let Some(stage) = lifecycle_stage_for_request(&request.payload) {
                        self.runtime.record_plugin_sandbox_lifecycle(
                            sandbox_id,
                            stage,
                            Some(processing_epoch),
                        );
                    }
                }
                Err(failure) => record_runtime_fault(&mut self.runtime, &failure),
            }
        }

        self.runtime.record_plugin_sandbox_transport(
            sandbox_id,
            lease_id,
            transport.region_id.as_str(),
            PluginSandboxTransportStage::DetachRequested,
            Some(processing_epoch),
            Some(detail.into()),
        );

        let destroy_error = self.broker.destroy_region(transport).err();
        if let Some(error) = destroy_error.as_ref() {
            self.runtime.record_broker_failure(
                sandbox_id,
                Some(lease_id.to_string()),
                Some(processing_epoch),
                None,
                BrokerFailureStage::TransportDestroy,
                error.to_string(),
            );
            self.runtime.record_plugin_sandbox_transport(
                sandbox_id,
                lease_id,
                transport.region_id.as_str(),
                PluginSandboxTransportStage::DetachFault,
                Some(processing_epoch),
                Some(error.to_string()),
            );
        }

        let teardown_error = lifecycle.teardown_active_transport().err();
        if let Some(error) = teardown_error.as_ref() {
            self.runtime.record_broker_failure(
                sandbox_id,
                Some(lease_id.to_string()),
                Some(processing_epoch),
                None,
                BrokerFailureStage::TransportTeardown,
                error.to_string(),
            );
            self.runtime.record_plugin_sandbox_transport(
                sandbox_id,
                lease_id,
                transport.region_id.as_str(),
                PluginSandboxTransportStage::DetachFault,
                Some(processing_epoch),
                Some(error.to_string()),
            );
        }

        if destroy_error.is_none() && teardown_error.is_none() {
            self.runtime.record_plugin_sandbox_transport(
                sandbox_id,
                lease_id,
                transport.region_id.as_str(),
                PluginSandboxTransportStage::Detached,
                Some(processing_epoch),
                Some(detail.into()),
            );
            self.runtime.record_plugin_sandbox_lifecycle(
                sandbox_id,
                PluginSandboxLifecycleStage::TransportTornDown,
                Some(processing_epoch),
            );
        }
    }
}
