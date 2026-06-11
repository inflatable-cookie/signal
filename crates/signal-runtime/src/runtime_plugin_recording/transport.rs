use super::*;

impl SignalRuntime {
    /// Records a plugin sandbox fault, invalidates its render state, and emits the fault event.
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
        self.emit(RuntimeEvent::PluginSandboxFault {
            sandbox_id,
            kind,
            detail,
            processing_epoch,
        });
    }

    /// Records a plugin sandbox recovery cycle, invalidates its render state, and emits the event.
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
        self.emit(RuntimeEvent::RecoveryCycle {
            sandbox_id,
            intent,
            stop_reason,
            processing_epoch,
        });
    }

    /// Records a plugin sandbox lifecycle stage transition and emits the corresponding event.
    pub fn record_plugin_sandbox_lifecycle(
        &mut self,
        sandbox_id: impl Into<String>,
        stage: PluginSandboxLifecycleStage,
        processing_epoch: Option<u64>,
    ) {
        let sandbox_id = sandbox_id.into();
        self.plugin_lifecycle
            .record_lifecycle(sandbox_id.as_str(), stage, processing_epoch);
        self.emit(RuntimeEvent::PluginSandboxLifecycle {
            sandbox_id,
            stage,
            processing_epoch,
        });
    }

    /// Records a plugin sandbox transport stage transition and emits the corresponding event.
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
