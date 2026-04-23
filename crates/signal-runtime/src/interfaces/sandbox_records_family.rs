use super::*;

/// Flat record extracted from a `RecoveryCycle` event.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RecoveryRecord {
    /// Sandbox ID that was recovered.
    pub sandbox_id: String,
    /// Recovery restart intent.
    pub intent: RecoveryRestartIntent,
    /// Reason the sandbox was stopped before recovery.
    pub stop_reason: StopReason,
    /// Processing epoch at the time of the recovery cycle.
    pub processing_epoch: Option<u64>,
}

/// Stage in a plugin sandbox lifecycle sequence.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PluginSandboxLifecycleStage {
    /// Sandbox process has been ensured (started or found running).
    SandboxEnsured,
    /// Sandbox has completed the handshake sequence.
    SandboxHandshaken,
    /// Plugin type was loaded into the sandbox.
    PluginTypeLoaded,
    /// Plugin instance was created.
    InstanceCreated,
    /// Plugin instance was prepared for processing.
    InstancePrepared,
    /// A transport session was attached to the instance.
    TransportAttached,
    /// Plugin instance was activated for audio processing.
    InstanceActivated,
    /// Plugin instance was deactivated.
    InstanceDeactivated,
    /// Plugin instance state was reset.
    InstanceReset,
    /// Plugin instance was destroyed.
    InstanceDestroyed,
    /// Sandbox is being torn down.
    SandboxTeardown,
    /// All transport sessions were torn down.
    TransportTornDown,
    /// Sandbox was restarted after a fault or recovery cycle.
    SandboxRestarted,
}

/// Flat record extracted from a `PluginSandboxLifecycle` event.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PluginSandboxLifecycleRecord {
    /// Sandbox ID that transitioned.
    pub sandbox_id: String,
    /// Lifecycle stage reached.
    pub stage: PluginSandboxLifecycleStage,
    /// Processing epoch at the time of the transition.
    pub processing_epoch: Option<u64>,
}

/// Stage in a plugin sandbox transport session lifecycle.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PluginSandboxTransportStage {
    /// Transport session was attached.
    Attached,
    /// A detach was requested for the transport session.
    DetachRequested,
    /// Transport session was detached successfully.
    Detached,
    /// Transport session detach encountered a fault.
    DetachFault,
}

/// Flat record extracted from a `PluginSandboxTransport` event.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PluginSandboxTransportRecord {
    /// Sandbox ID for this transport event.
    pub sandbox_id: String,
    /// Lease ID for this transport session.
    pub lease_id: String,
    /// Region ID for this transport session.
    pub region_id: String,
    /// Transport stage reached.
    pub stage: PluginSandboxTransportStage,
    /// Processing epoch at the time of the event.
    pub processing_epoch: Option<u64>,
    /// Optional detail message (e.g. fault description).
    pub detail: Option<String>,
}

/// A single fault entry reported by a plugin sandbox instance.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PluginSandboxInstanceFaultRecord {
    /// Fault kind identifier.
    pub kind: String,
    /// Fault severity level.
    pub severity: String,
    /// Human-readable fault message.
    pub message: String,
}

/// Full per-instance state snapshot received in a heartbeat response.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PluginSandboxInstanceStateRecord {
    /// Sandbox ID.
    pub sandbox_id: String,
    /// Plugin type identifier for this instance.
    pub plugin_type_id: String,
    /// Plugin instance identifier.
    pub instance_id: String,
    /// Lifecycle state string reported by the sandbox.
    pub lifecycle_state: String,
    /// Readiness state string reported by the sandbox.
    pub readiness_state: String,
    /// Reasons the instance is currently degraded.
    pub degraded_reasons: Vec<String>,
    /// Whether the instance is currently active.
    pub active: bool,
    /// Processing epoch at the time of the heartbeat.
    pub processing_epoch: Option<u64>,
    /// Sample rate in Hz the instance is processing at.
    pub processing_sample_rate_hz: Option<u32>,
    /// Maximum block size in frames the instance supports.
    pub processing_max_block_frames: Option<u32>,
    /// Number of audio input channels.
    pub audio_inputs: Option<u16>,
    /// Number of audio output channels.
    pub audio_outputs: Option<u16>,
    /// Number of MIDI input ports.
    pub midi_inputs: Option<u16>,
    /// Number of MIDI output ports.
    pub midi_outputs: Option<u16>,
    /// Most recent fault record from this instance, if any.
    pub last_fault: Option<PluginSandboxInstanceFaultRecord>,
}

/// Phase of a sandbox heartbeat exchange.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HeartbeatCycleStage {
    /// Heartbeat request was sent to the sandbox.
    Requested,
    /// Sandbox responded to the heartbeat within the deadline.
    Responded,
    /// Sandbox did not respond to the heartbeat before the deadline.
    Missed,
}

/// Flat record extracted from a `HeartbeatCycle` event.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HeartbeatCycleRecord {
    /// Sandbox ID for this heartbeat.
    pub sandbox_id: String,
    /// Heartbeat exchange stage.
    pub stage: HeartbeatCycleStage,
    /// Processing epoch at the time of the heartbeat.
    pub processing_epoch: Option<u64>,
    /// Block sequence number associated with this heartbeat.
    pub block_sequence: Option<u64>,
}

/// Phase of a block dispatch exchange with a sandbox.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BlockDispatchStage {
    /// Block render was dispatched to the sandbox.
    Requested,
    /// Sandbox completed the block render before the deadline.
    Completed,
    /// Sandbox did not complete the block render before the deadline.
    TimedOut,
}

/// Flat record extracted from a `BlockDispatch` event.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BlockDispatchRecord {
    /// Sandbox ID for this dispatch.
    pub sandbox_id: String,
    /// Lease ID for this dispatch session.
    pub lease_id: String,
    /// Processing epoch for this block.
    pub processing_epoch: u64,
    /// Block sequence number for this dispatch.
    pub block_sequence: u64,
    /// Number of audio frames in this block.
    pub frame_count: u32,
    /// Dispatch stage reached.
    pub stage: BlockDispatchStage,
    /// Completion state, if the dispatch completed.
    pub completion_state: Option<CompletionState>,
}

/// Flat record extracted from a `LeaseRollover` event.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LeaseRolloverRecord {
    /// Sandbox ID for this rollover.
    pub sandbox_id: String,
    /// Lease ID that was replaced.
    pub previous_lease_id: String,
    /// New lease ID after rollover.
    pub lease_id: String,
    /// Processing epoch at the time of the rollover.
    pub processing_epoch: u64,
    /// First block sequence number under the new lease.
    pub first_block_sequence: u64,
}

/// Stage of a broker invalidation cycle.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BrokerInvalidationStage {
    /// The shared-memory completion region was explicitly invalidated.
    CompletionRegionInvalidated,
    /// The lease epoch was rolled over, invalidating any in-flight completions.
    LeaseEpochInvalidated,
}

/// Flat record extracted from a `BrokerInvalidation` event.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BrokerInvalidationRecord {
    /// Sandbox ID for this invalidation.
    pub sandbox_id: String,
    /// Lease ID associated with the invalidation.
    pub lease_id: String,
    /// Processing epoch at the time of the invalidation.
    pub processing_epoch: u64,
    /// Block sequence number at the time of invalidation, if applicable.
    pub block_sequence: Option<u64>,
    /// Invalidation stage.
    pub stage: BrokerInvalidationStage,
    /// Human-readable reason for the invalidation.
    pub reason: String,
}

/// Stage of a completion-slot state machine transition.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CompletionSlotStage {
    /// Slot is armed and waiting for the sandbox to begin processing.
    ReadyForProcessing,
    /// Sandbox is actively processing the block associated with this slot.
    Processing,
    /// Sandbox signalled completion before the deadline.
    Completed,
    /// Sandbox did not signal completion before the deadline.
    TimedOut,
    /// Slot was explicitly invalidated before completion.
    Invalidated,
    /// Runtime applied a fallback result because the slot could not complete normally.
    FallbackApplied,
}

/// Flat record extracted from a `CompletionSlotTransition` event.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CompletionSlotRecord {
    /// Sandbox ID for this completion slot.
    pub sandbox_id: String,
    /// Lease ID for this completion slot.
    pub lease_id: String,
    /// Processing epoch for this slot.
    pub processing_epoch: u64,
    /// Block sequence number for this slot.
    pub block_sequence: u64,
    /// Completion slot stage reached.
    pub stage: CompletionSlotStage,
}

/// Stage at which the broker reported a hard failure.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BrokerFailureStage {
    /// Failure occurred while creating the prepare plan.
    PreparePlanCreate,
    /// Failure occurred writing a block payload into shared memory.
    PayloadWrite,
    /// Failure occurred reading a block payload from shared memory.
    PayloadRead,
    /// Failure occurred destroying the broker transport.
    TransportDestroy,
    /// Failure occurred during transport teardown.
    TransportTeardown,
}

/// Flat record extracted from a `BrokerFailure` event.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BrokerFailureRecord {
    /// Sandbox ID where the broker failure occurred.
    pub sandbox_id: String,
    /// Lease ID associated with the failure, if applicable.
    pub lease_id: Option<String>,
    /// Processing epoch at the time of the failure.
    pub processing_epoch: Option<u64>,
    /// Block sequence number at the time of the failure.
    pub block_sequence: Option<u64>,
    /// Stage at which the broker failure occurred.
    pub stage: BrokerFailureStage,
    /// Detailed failure description.
    pub detail: String,
}

/// Stage at which a sandbox operation failed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SandboxOperationFailureStage {
    /// Failure during the prepare-attach phase of sandbox startup.
    PrepareAttach,
    /// Failure during the process-attach phase.
    ProcessAttach,
    /// Failure during a flush operation on the sandbox process.
    ProcessFlush,
    /// Sandbox violated the block-processing protocol.
    ProcessProtocolViolation,
    /// Sandbox violated the control messaging protocol.
    ControlProtocolViolation,
}

/// Flat record extracted from a `SandboxOperationFailure` event.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SandboxOperationFailureRecord {
    /// Sandbox ID where the operation failure occurred.
    pub sandbox_id: String,
    /// Lease ID associated with the failure, if applicable.
    pub lease_id: Option<String>,
    /// Processing epoch at the time of the failure.
    pub processing_epoch: Option<u64>,
    /// Name of the operation that failed.
    pub operation: String,
    /// Error kind string from the underlying failure.
    pub error_kind: String,
    /// Stage at which the operation failure occurred.
    pub stage: SandboxOperationFailureStage,
    /// Detailed failure description.
    pub detail: String,
}
