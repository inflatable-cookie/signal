use super::*;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RecoveryRecord {
    pub sandbox_id: String,
    pub intent: RecoveryRestartIntent,
    pub stop_reason: StopReason,
    pub processing_epoch: Option<u64>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PluginSandboxLifecycleStage {
    SandboxEnsured,
    SandboxHandshaken,
    PluginTypeLoaded,
    InstanceCreated,
    InstancePrepared,
    TransportAttached,
    InstanceActivated,
    InstanceDeactivated,
    InstanceReset,
    InstanceDestroyed,
    SandboxTeardown,
    TransportTornDown,
    SandboxRestarted,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PluginSandboxLifecycleRecord {
    pub sandbox_id: String,
    pub stage: PluginSandboxLifecycleStage,
    pub processing_epoch: Option<u64>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PluginSandboxTransportStage {
    Attached,
    DetachRequested,
    Detached,
    DetachFault,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PluginSandboxTransportRecord {
    pub sandbox_id: String,
    pub lease_id: String,
    pub region_id: String,
    pub stage: PluginSandboxTransportStage,
    pub processing_epoch: Option<u64>,
    pub detail: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PluginSandboxInstanceFaultRecord {
    pub kind: String,
    pub severity: String,
    pub message: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PluginSandboxInstanceStateRecord {
    pub sandbox_id: String,
    pub plugin_type_id: String,
    pub instance_id: String,
    pub lifecycle_state: String,
    pub readiness_state: String,
    pub degraded_reasons: Vec<String>,
    pub active: bool,
    pub processing_epoch: Option<u64>,
    pub processing_sample_rate_hz: Option<u32>,
    pub processing_max_block_frames: Option<u32>,
    pub audio_inputs: Option<u16>,
    pub audio_outputs: Option<u16>,
    pub midi_inputs: Option<u16>,
    pub midi_outputs: Option<u16>,
    pub last_fault: Option<PluginSandboxInstanceFaultRecord>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HeartbeatCycleStage {
    Requested,
    Responded,
    Missed,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HeartbeatCycleRecord {
    pub sandbox_id: String,
    pub stage: HeartbeatCycleStage,
    pub processing_epoch: Option<u64>,
    pub block_sequence: Option<u64>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BlockDispatchStage {
    Requested,
    Completed,
    TimedOut,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BlockDispatchRecord {
    pub sandbox_id: String,
    pub lease_id: String,
    pub processing_epoch: u64,
    pub block_sequence: u64,
    pub frame_count: u32,
    pub stage: BlockDispatchStage,
    pub completion_state: Option<CompletionState>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LeaseRolloverRecord {
    pub sandbox_id: String,
    pub previous_lease_id: String,
    pub lease_id: String,
    pub processing_epoch: u64,
    pub first_block_sequence: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BrokerInvalidationStage {
    CompletionRegionInvalidated,
    LeaseEpochInvalidated,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BrokerInvalidationRecord {
    pub sandbox_id: String,
    pub lease_id: String,
    pub processing_epoch: u64,
    pub block_sequence: Option<u64>,
    pub stage: BrokerInvalidationStage,
    pub reason: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CompletionSlotStage {
    ReadyForProcessing,
    Processing,
    Completed,
    TimedOut,
    Invalidated,
    FallbackApplied,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CompletionSlotRecord {
    pub sandbox_id: String,
    pub lease_id: String,
    pub processing_epoch: u64,
    pub block_sequence: u64,
    pub stage: CompletionSlotStage,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BrokerFailureStage {
    PreparePlanCreate,
    PayloadWrite,
    PayloadRead,
    TransportDestroy,
    TransportTeardown,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BrokerFailureRecord {
    pub sandbox_id: String,
    pub lease_id: Option<String>,
    pub processing_epoch: Option<u64>,
    pub block_sequence: Option<u64>,
    pub stage: BrokerFailureStage,
    pub detail: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SandboxOperationFailureStage {
    PrepareAttach,
    ProcessAttach,
    ProcessFlush,
    ProcessProtocolViolation,
    ControlProtocolViolation,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SandboxOperationFailureRecord {
    pub sandbox_id: String,
    pub lease_id: Option<String>,
    pub processing_epoch: Option<u64>,
    pub operation: String,
    pub error_kind: String,
    pub stage: SandboxOperationFailureStage,
    pub detail: String,
}
