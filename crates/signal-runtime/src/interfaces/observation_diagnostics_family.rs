use super::*;

mod observation_diagnostics_render;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RuntimeEvent {
    ReadinessChanged(RuntimeReadiness),
    EffectiveConfigChanged(EffectiveRuntimeConfig),
    SupervisionChanged(RuntimeSupervisionSnapshot),
    PluginSandboxChanged {
        active_sandboxes: u32,
    },
    PluginSandboxFault {
        sandbox_id: String,
        kind: PluginFaultKind,
        detail: String,
        processing_epoch: Option<u64>,
    },
    RecoveryCycle {
        sandbox_id: String,
        intent: RecoveryRestartIntent,
        stop_reason: StopReason,
        processing_epoch: Option<u64>,
    },
    PluginSandboxLifecycle {
        sandbox_id: String,
        stage: PluginSandboxLifecycleStage,
        processing_epoch: Option<u64>,
    },
    PluginSandboxInstanceState {
        state: PluginSandboxInstanceStateRecord,
    },
    PluginSandboxTransport {
        sandbox_id: String,
        lease_id: String,
        region_id: String,
        stage: PluginSandboxTransportStage,
        processing_epoch: Option<u64>,
        detail: Option<String>,
    },
    HeartbeatCycle {
        sandbox_id: String,
        stage: HeartbeatCycleStage,
        processing_epoch: Option<u64>,
        block_sequence: Option<u64>,
    },
    BlockDispatch {
        sandbox_id: String,
        lease_id: String,
        processing_epoch: u64,
        block_sequence: u64,
        frame_count: u32,
        stage: BlockDispatchStage,
        completion_state: Option<CompletionState>,
    },
    LeaseRollover {
        sandbox_id: String,
        previous_lease_id: String,
        lease_id: String,
        processing_epoch: u64,
        first_block_sequence: u64,
    },
    BrokerInvalidation {
        sandbox_id: String,
        lease_id: String,
        processing_epoch: u64,
        block_sequence: Option<u64>,
        stage: BrokerInvalidationStage,
        reason: String,
    },
    CompletionSlotTransition {
        sandbox_id: String,
        lease_id: String,
        processing_epoch: u64,
        block_sequence: u64,
        stage: CompletionSlotStage,
    },
    BrokerFailure {
        sandbox_id: String,
        lease_id: Option<String>,
        processing_epoch: Option<u64>,
        block_sequence: Option<u64>,
        stage: BrokerFailureStage,
        detail: String,
    },
    SandboxOperationFailure {
        sandbox_id: String,
        lease_id: Option<String>,
        processing_epoch: Option<u64>,
        operation: String,
        error_kind: String,
        stage: SandboxOperationFailureStage,
        detail: String,
    },
    HardwareDeviceChanged {
        device_id: Option<String>,
    },
}

pub trait RuntimeEventSink: Send {
    fn push(&mut self, event: RuntimeEvent);
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PluginFaultRecord {
    pub sandbox_id: String,
    pub kind: PluginFaultKind,
    pub detail: String,
    pub processing_epoch: Option<u64>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RuntimeObservationDiagnostics {
    pub total_events: usize,
    pub supervision_updates: Vec<RuntimeSupervisionSnapshot>,
    pub plugin_faults: Vec<PluginFaultRecord>,
    pub plugin_instance_states: Vec<PluginSandboxInstanceStateRecord>,
    pub recovery_events: Vec<RecoveryRecord>,
    pub lifecycle_events: Vec<PluginSandboxLifecycleRecord>,
    pub transport_events: Vec<PluginSandboxTransportRecord>,
    pub heartbeat_events: Vec<HeartbeatCycleRecord>,
    pub block_dispatch_events: Vec<BlockDispatchRecord>,
    pub lease_rollover_events: Vec<LeaseRolloverRecord>,
    pub invalidation_events: Vec<BrokerInvalidationRecord>,
    pub completion_slot_events: Vec<CompletionSlotRecord>,
    pub transport_fault_events: Vec<TransportFaultRecord>,
    pub broker_failure_events: Vec<BrokerFailureRecord>,
    pub sandbox_operation_failure_events: Vec<SandboxOperationFailureRecord>,
}

impl RuntimeObservationDiagnostics {
    pub fn supervision_update_count(&self) -> usize {
        self.supervision_updates.len()
    }

    pub fn plugin_fault_count(&self) -> usize {
        self.plugin_faults.len()
    }

    pub fn plugin_instance_state_event_count(&self) -> usize {
        self.plugin_instance_states.len()
    }

    pub fn recovery_event_count(&self) -> usize {
        self.recovery_events.len()
    }

    pub fn lifecycle_event_count(&self) -> usize {
        self.lifecycle_events.len()
    }

    pub fn transport_event_count(&self) -> usize {
        self.transport_events.len()
    }

    pub fn heartbeat_event_count(&self) -> usize {
        self.heartbeat_events.len()
    }

    pub fn block_dispatch_event_count(&self) -> usize {
        self.block_dispatch_events.len()
    }

    pub fn lease_rollover_event_count(&self) -> usize {
        self.lease_rollover_events.len()
    }

    pub fn invalidation_event_count(&self) -> usize {
        self.invalidation_events.len()
    }

    pub fn completion_slot_event_count(&self) -> usize {
        self.completion_slot_events.len()
    }

    pub fn transport_fault_event_count(&self) -> usize {
        self.transport_fault_events.len()
    }

    pub fn broker_failure_event_count(&self) -> usize {
        self.broker_failure_events.len()
    }

    pub fn sandbox_operation_failure_event_count(&self) -> usize {
        self.sandbox_operation_failure_events.len()
    }

    pub fn fault_detail_count_containing(&self, needle: &str) -> usize {
        self.plugin_faults
            .iter()
            .filter(|fault| fault.detail.contains(needle))
            .count()
    }

    pub fn last_supervision_update(&self) -> Option<&RuntimeSupervisionSnapshot> {
        self.supervision_updates.last()
    }

    pub fn last_recovery_event(&self) -> Option<&RecoveryRecord> {
        self.recovery_events.last()
    }

    pub fn last_plugin_instance_state(&self) -> Option<&PluginSandboxInstanceStateRecord> {
        self.plugin_instance_states.last()
    }

    pub fn last_lifecycle_event(&self) -> Option<&PluginSandboxLifecycleRecord> {
        self.lifecycle_events.last()
    }

    pub fn last_transport_event(&self) -> Option<&PluginSandboxTransportRecord> {
        self.transport_events.last()
    }

    pub fn last_heartbeat_event(&self) -> Option<&HeartbeatCycleRecord> {
        self.heartbeat_events.last()
    }

    pub fn last_block_dispatch_event(&self) -> Option<&BlockDispatchRecord> {
        self.block_dispatch_events.last()
    }

    pub fn last_lease_rollover_event(&self) -> Option<&LeaseRolloverRecord> {
        self.lease_rollover_events.last()
    }

    pub fn last_invalidation_event(&self) -> Option<&BrokerInvalidationRecord> {
        self.invalidation_events.last()
    }

    pub fn last_completion_slot_event(&self) -> Option<&CompletionSlotRecord> {
        self.completion_slot_events.last()
    }

    pub fn last_transport_fault_event(&self) -> Option<&TransportFaultRecord> {
        self.transport_fault_events.last()
    }

    pub fn last_broker_failure_event(&self) -> Option<&BrokerFailureRecord> {
        self.broker_failure_events.last()
    }

    pub fn last_sandbox_operation_failure_event(&self) -> Option<&SandboxOperationFailureRecord> {
        self.sandbox_operation_failure_events.last()
    }
}
