use super::*;

impl SignalRuntime {
    /// Increments the xrun counter by one.
    pub fn increment_xruns(&mut self) {
        self.diagnostics.xruns = self.diagnostics.xruns.saturating_add(1);
    }

    /// Records a plugin sandbox instance state snapshot and emits the corresponding event.
    pub fn record_plugin_sandbox_instance_state(
        &mut self,
        state: PluginSandboxInstanceStateRecord,
    ) {
        self.plugin_lifecycle.record_instance_state(state.clone());
        self.emit(RuntimeEvent::PluginSandboxInstanceState { state });
    }

    /// Records an LV2 prepared negotiation result for the given sandbox.
    pub fn record_plugin_sandbox_lv2_prepared_negotiation(
        &mut self,
        sandbox_id: &str,
        negotiation: RuntimeLv2PreparedNegotiationRecord,
    ) {
        self.plugin_lifecycle
            .record_lv2_prepared_negotiation(sandbox_id, negotiation);
    }

    /// Records a heartbeat cycle stage transition and emits the corresponding event.
    pub fn record_heartbeat_cycle(
        &mut self,
        sandbox_id: impl Into<String>,
        stage: HeartbeatCycleStage,
        processing_epoch: Option<u64>,
        block_sequence: Option<u64>,
    ) {
        self.emit(RuntimeEvent::HeartbeatCycle {
            sandbox_id: sandbox_id.into(),
            stage,
            processing_epoch,
            block_sequence,
        });
    }

    /// Records a block dispatch record and emits the corresponding event.
    pub fn record_block_dispatch(&mut self, record: BlockDispatchRecord) {
        self.emit(RuntimeEvent::BlockDispatch {
            sandbox_id: record.sandbox_id,
            lease_id: record.lease_id,
            processing_epoch: record.processing_epoch,
            block_sequence: record.block_sequence,
            frame_count: record.frame_count,
            stage: record.stage,
            completion_state: record.completion_state,
        });
    }

    /// Records a broker invalidation event and emits the corresponding event.
    pub fn record_broker_invalidation(
        &mut self,
        sandbox_id: impl Into<String>,
        lease_id: impl Into<String>,
        processing_epoch: u64,
        block_sequence: Option<u64>,
        stage: BrokerInvalidationStage,
        reason: impl Into<String>,
    ) {
        self.emit(RuntimeEvent::BrokerInvalidation {
            sandbox_id: sandbox_id.into(),
            lease_id: lease_id.into(),
            processing_epoch,
            block_sequence,
            stage,
            reason: reason.into(),
        });
    }

    /// Records a completion slot stage transition and emits the corresponding event.
    pub fn record_completion_slot_transition(
        &mut self,
        sandbox_id: impl Into<String>,
        lease_id: impl Into<String>,
        processing_epoch: u64,
        block_sequence: u64,
        stage: CompletionSlotStage,
    ) {
        self.emit(RuntimeEvent::CompletionSlotTransition {
            sandbox_id: sandbox_id.into(),
            lease_id: lease_id.into(),
            processing_epoch,
            block_sequence,
            stage,
        });
    }

    /// Records a broker failure event and emits the corresponding event.
    pub fn record_broker_failure(
        &mut self,
        sandbox_id: impl Into<String>,
        lease_id: Option<String>,
        processing_epoch: Option<u64>,
        block_sequence: Option<u64>,
        stage: BrokerFailureStage,
        detail: impl Into<String>,
    ) {
        self.emit(RuntimeEvent::BrokerFailure {
            sandbox_id: sandbox_id.into(),
            lease_id,
            processing_epoch,
            block_sequence,
            stage,
            detail: detail.into(),
        });
    }

    /// Records a sandbox operation failure and emits the corresponding event.
    pub fn record_sandbox_operation_failure(&mut self, record: SandboxOperationFailureRecord) {
        self.emit(RuntimeEvent::SandboxOperationFailure {
            sandbox_id: record.sandbox_id,
            lease_id: record.lease_id,
            processing_epoch: record.processing_epoch,
            operation: record.operation,
            error_kind: record.error_kind,
            stage: record.stage,
            detail: record.detail,
        });
    }
}
