use super::*;

impl SignalRuntime {
    pub fn record_plugin_sandbox_fault(
        &mut self,
        sandbox_id: impl Into<String>,
        kind: PluginFaultKind,
        detail: impl Into<String>,
        processing_epoch: Option<u64>,
    ) {
        let sandbox_id = sandbox_id.into();
        let detail = detail.into();
        self.plugin_lifecycle.record_fault(
            sandbox_id.as_str(),
            kind,
            detail.clone(),
            processing_epoch,
        );
        self.invalidate_plugin_render_state_for_sandbox(sandbox_id.as_str());
        self.emit(RuntimeEvent::PluginSandboxFault {
            sandbox_id,
            kind,
            detail,
            processing_epoch,
        });
    }

    pub fn record_recovery_cycle(
        &mut self,
        sandbox_id: impl Into<String>,
        intent: RecoveryRestartIntent,
        stop_reason: StopReason,
        processing_epoch: Option<u64>,
    ) {
        let sandbox_id = sandbox_id.into();
        self.plugin_lifecycle.record_recovery_cycle(
            sandbox_id.as_str(),
            intent,
            stop_reason,
            processing_epoch,
        );
        self.invalidate_plugin_render_state_for_sandbox(sandbox_id.as_str());
        self.emit(RuntimeEvent::RecoveryCycle {
            sandbox_id,
            intent,
            stop_reason,
            processing_epoch,
        });
    }

    pub fn record_plugin_sandbox_lifecycle(
        &mut self,
        sandbox_id: impl Into<String>,
        stage: PluginSandboxLifecycleStage,
        processing_epoch: Option<u64>,
    ) {
        let sandbox_id = sandbox_id.into();
        self.plugin_lifecycle
            .record_lifecycle(sandbox_id.as_str(), stage, processing_epoch);
        if matches!(
            stage,
            PluginSandboxLifecycleStage::InstanceDeactivated
                | PluginSandboxLifecycleStage::InstanceReset
                | PluginSandboxLifecycleStage::InstanceDestroyed
                | PluginSandboxLifecycleStage::SandboxTeardown
                | PluginSandboxLifecycleStage::TransportTornDown
                | PluginSandboxLifecycleStage::SandboxRestarted
        ) {
            self.invalidate_plugin_render_state_for_sandbox(sandbox_id.as_str());
        }
        self.emit(RuntimeEvent::PluginSandboxLifecycle {
            sandbox_id,
            stage,
            processing_epoch,
        });
    }

    pub fn record_plugin_sandbox_transport(
        &mut self,
        sandbox_id: impl Into<String>,
        lease_id: impl Into<String>,
        region_id: impl Into<String>,
        stage: PluginSandboxTransportStage,
        processing_epoch: Option<u64>,
        detail: Option<String>,
    ) {
        let sandbox_id = sandbox_id.into();
        let lease_id = lease_id.into();
        let region_id = region_id.into();
        self.plugin_lifecycle.record_transport(
            sandbox_id.as_str(),
            lease_id.as_str(),
            region_id.as_str(),
            stage,
            processing_epoch,
            detail.clone(),
        );
        if matches!(
            stage,
            PluginSandboxTransportStage::DetachRequested
                | PluginSandboxTransportStage::Detached
                | PluginSandboxTransportStage::DetachFault
        ) {
            self.invalidate_plugin_render_state_for_sandbox(sandbox_id.as_str());
        }
        match stage {
            PluginSandboxTransportStage::Attached => {
                self.transport_concurrency.mark_session_state(
                    sandbox_id.as_str(),
                    lease_id.as_str(),
                    region_id.as_str(),
                    TransportSessionState::AttachActive,
                );
            }
            PluginSandboxTransportStage::DetachRequested => {
                self.transport_concurrency.mark_session_state(
                    sandbox_id.as_str(),
                    lease_id.as_str(),
                    region_id.as_str(),
                    TransportSessionState::DetachRequested,
                );
            }
            PluginSandboxTransportStage::DetachFault => {
                self.transport_concurrency.mark_session_state(
                    sandbox_id.as_str(),
                    lease_id.as_str(),
                    region_id.as_str(),
                    TransportSessionState::DetachFaulted,
                );
            }
            PluginSandboxTransportStage::Detached => {
                self.transport_concurrency.mark_session_state(
                    sandbox_id.as_str(),
                    lease_id.as_str(),
                    region_id.as_str(),
                    TransportSessionState::Detached,
                );
            }
        }
        self.refresh_prework_service_policy_and_state(processing_epoch);
        self.emit(RuntimeEvent::PluginSandboxTransport {
            sandbox_id,
            lease_id,
            region_id,
            stage,
            processing_epoch,
            detail,
        });
    }
}
