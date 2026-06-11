use super::*;

/// Push event emitted by the runtime to all registered `RuntimeEventSink`s.
///
/// Variants cover state transitions (readiness, config), sandbox lifecycle
/// stages, transport attach/detach, block dispatch, broker faults, and hardware
/// changes.  Consumers typically feed these into a [`RuntimeEventRecorder`] and
/// inspect them via the typed helper methods.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RuntimeEvent {
    /// Runtime readiness changed (e.g. started, degraded, stopped).
    ReadinessChanged(RuntimeReadiness),
    /// Effective config changed after a reconfigure or restart.
    EffectiveConfigChanged(EffectiveRuntimeConfig),
    /// Supervision state changed (watchdog counts, safe mode, xrun overload).
    SupervisionChanged(RuntimeSupervisionSnapshot),
    /// The number of active plugin sandboxes changed.
    PluginSandboxChanged {
        /// New count of active plugin sandboxes.
        active_sandboxes: u32,
    },
    /// A plugin sandbox reported a fault (timeout, crash, or protocol
    /// violation).
    PluginSandboxFault {
        /// ID of the faulting sandbox.
        sandbox_id: String,
        /// Category of the fault.
        kind: PluginFaultKind,
        /// Human-readable description of the fault.
        detail: String,
        /// Processing epoch at the time of the fault, if known.
        processing_epoch: Option<u64>,
    },
    /// A recovery cycle began for a sandbox.
    RecoveryCycle {
        /// ID of the sandbox being recovered.
        sandbox_id: String,
        /// Intent describing the type of recovery required.
        intent: RecoveryRestartIntent,
        /// Reason the runtime was stopped to perform recovery.
        stop_reason: StopReason,
        /// Processing epoch at the time recovery was triggered, if known.
        processing_epoch: Option<u64>,
    },
    /// A sandbox advanced through a lifecycle stage.
    PluginSandboxLifecycle {
        /// ID of the sandbox that advanced.
        sandbox_id: String,
        /// Lifecycle stage reached.
        stage: PluginSandboxLifecycleStage,
        /// Processing epoch at the time of the lifecycle transition, if known.
        processing_epoch: Option<u64>,
    },
    /// A sandbox reported its current instance state (from a heartbeat
    /// response).
    PluginSandboxInstanceState {
        /// The reported instance state record.
        state: PluginSandboxInstanceStateRecord,
    },
    /// A sandbox transport session transitioned (attached, detach-requested,
    /// detached, or detach-faulted).
    PluginSandboxTransport {
        /// ID of the sandbox whose transport session transitioned.
        sandbox_id: String,
        /// ID of the shared-memory lease associated with this session.
        lease_id: String,
        /// ID of the shared-memory region associated with this session.
        region_id: String,
        /// Transport stage reached.
        stage: PluginSandboxTransportStage,
        /// Processing epoch at the time of the transition, if known.
        processing_epoch: Option<u64>,
        /// Additional detail about the transition, if any.
        detail: Option<String>,
    },
    /// A heartbeat exchange was requested, responded, or missed.
    HeartbeatCycle {
        /// ID of the sandbox involved in the heartbeat.
        sandbox_id: String,
        /// Stage of the heartbeat cycle.
        stage: HeartbeatCycleStage,
        /// Processing epoch associated with this heartbeat, if known.
        processing_epoch: Option<u64>,
        /// Block sequence number associated with this heartbeat, if known.
        block_sequence: Option<u64>,
    },
    /// A block was dispatched to (or returned from) a sandbox.
    BlockDispatch {
        /// ID of the sandbox receiving the block.
        sandbox_id: String,
        /// ID of the shared-memory lease for this dispatch.
        lease_id: String,
        /// Processing epoch for this block.
        processing_epoch: u64,
        /// Sequence number of the dispatched block.
        block_sequence: u64,
        /// Number of audio frames in this block.
        frame_count: u32,
        /// Dispatch stage (send or receive).
        stage: BlockDispatchStage,
        /// Completion state returned by the sandbox, if available.
        completion_state: Option<CompletionState>,
    },
    /// The shared-memory lease rolled over to a new region.
    LeaseRollover {
        /// ID of the sandbox whose lease rolled over.
        sandbox_id: String,
        /// ID of the previous lease being retired.
        previous_lease_id: String,
        /// ID of the new lease.
        lease_id: String,
        /// Processing epoch at the start of the new lease.
        processing_epoch: u64,
        /// Block sequence number of the first block in the new lease.
        first_block_sequence: u64,
    },
    /// The broker invalidated a completion region or lease epoch.
    BrokerInvalidation {
        /// ID of the sandbox whose broker was invalidated.
        sandbox_id: String,
        /// ID of the lease that was invalidated.
        lease_id: String,
        /// Processing epoch at the time of invalidation.
        processing_epoch: u64,
        /// Block sequence number at the time of invalidation, if known.
        block_sequence: Option<u64>,
        /// Stage of the invalidation event.
        stage: BrokerInvalidationStage,
        /// Human-readable reason for the invalidation.
        reason: String,
    },
    /// A completion slot transitioned through its state machine.
    CompletionSlotTransition {
        /// ID of the sandbox whose completion slot transitioned.
        sandbox_id: String,
        /// ID of the lease associated with this slot.
        lease_id: String,
        /// Processing epoch for this transition.
        processing_epoch: u64,
        /// Block sequence number associated with this transition.
        block_sequence: u64,
        /// Completion slot stage reached.
        stage: CompletionSlotStage,
    },
    /// The broker reported a hard failure (payload I/O, transport destroy,
    /// etc.).
    BrokerFailure {
        /// ID of the sandbox whose broker failed.
        sandbox_id: String,
        /// ID of the associated lease, if known.
        lease_id: Option<String>,
        /// Processing epoch at the time of failure, if known.
        processing_epoch: Option<u64>,
        /// Block sequence number at the time of failure, if known.
        block_sequence: Option<u64>,
        /// Stage at which the broker failure occurred.
        stage: BrokerFailureStage,
        /// Human-readable description of the failure.
        detail: String,
    },
    /// A sandbox operation failed (prepare, flush, protocol violation).
    SandboxOperationFailure {
        /// ID of the sandbox where the operation failed.
        sandbox_id: String,
        /// ID of the associated lease, if known.
        lease_id: Option<String>,
        /// Processing epoch at the time of failure, if known.
        processing_epoch: Option<u64>,
        /// Name of the operation that failed.
        operation: String,
        /// Error kind string describing the failure category.
        error_kind: String,
        /// Stage at which the operation failure occurred.
        stage: SandboxOperationFailureStage,
        /// Human-readable description of the failure.
        detail: String,
    },
    /// The active hardware device changed (device loss or reconfiguration).
    HardwareDeviceChanged {
        /// ID of the new active device, or `None` if no device is active.
        device_id: Option<String>,
    },
}

/// Receiver for [`RuntimeEvent`]s pushed by the runtime.
///
/// Implement this on a type to receive real-time event notifications.
/// [`RuntimeEventRecorder`] is the standard implementation for in-process use.
pub trait RuntimeEventSink: Send {
    /// Called by the runtime for each event in the order it occurs.
    fn push(&mut self, event: RuntimeEvent);
}

/// Flat record extracted from a [`RuntimeEvent::PluginSandboxFault`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PluginFaultRecord {
    /// ID of the faulting sandbox.
    pub sandbox_id: String,
    /// Category of the fault.
    pub kind: PluginFaultKind,
    /// Human-readable description of the fault.
    pub detail: String,
    /// Processing epoch at the time of the fault, if known.
    pub processing_epoch: Option<u64>,
}

/// Categorised event counts and typed slices derived from a
/// [`RuntimeEventRecorder`] snapshot.
///
/// Built by calling `recorder.diagnostics()`.  Used inside
/// [`RuntimeObservationReport`] and [`RuntimeSupervisorReport`].
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RuntimeObservationDiagnostics {
    /// Total number of events observed in this window.
    pub total_events: usize,
    /// All supervision state update snapshots in order.
    pub supervision_updates: Vec<RuntimeSupervisionSnapshot>,
    /// All plugin fault records in order.
    pub plugin_faults: Vec<PluginFaultRecord>,
    /// All plugin instance state records in order.
    pub plugin_instance_states: Vec<PluginSandboxInstanceStateRecord>,
    /// All recovery event records in order.
    pub recovery_events: Vec<RecoveryRecord>,
    /// All sandbox lifecycle event records in order.
    pub lifecycle_events: Vec<PluginSandboxLifecycleRecord>,
    /// All sandbox transport session event records in order.
    pub transport_events: Vec<PluginSandboxTransportRecord>,
    /// All heartbeat cycle event records in order.
    pub heartbeat_events: Vec<HeartbeatCycleRecord>,
    /// All block dispatch event records in order.
    pub block_dispatch_events: Vec<BlockDispatchRecord>,
    /// All lease rollover event records in order.
    pub lease_rollover_events: Vec<LeaseRolloverRecord>,
    /// All broker invalidation event records in order.
    pub invalidation_events: Vec<BrokerInvalidationRecord>,
    /// All completion slot transition event records in order.
    pub completion_slot_events: Vec<CompletionSlotRecord>,
    /// All transport fault event records in order.
    pub transport_fault_events: Vec<TransportFaultRecord>,
    /// All broker failure event records in order.
    pub broker_failure_events: Vec<BrokerFailureRecord>,
    /// All sandbox operation failure event records in order.
    pub sandbox_operation_failure_events: Vec<SandboxOperationFailureRecord>,
}

impl RuntimeObservationDiagnostics {
    /// Returns the number of supervision update events.
    pub fn supervision_update_count(&self) -> usize {
        self.supervision_updates.len()
    }

    /// Returns the number of plugin fault events.
    pub fn plugin_fault_count(&self) -> usize {
        self.plugin_faults.len()
    }

    /// Returns the number of plugin instance state events.
    pub fn plugin_instance_state_event_count(&self) -> usize {
        self.plugin_instance_states.len()
    }

    /// Returns the number of recovery events.
    pub fn recovery_event_count(&self) -> usize {
        self.recovery_events.len()
    }

    /// Returns the number of sandbox lifecycle events.
    pub fn lifecycle_event_count(&self) -> usize {
        self.lifecycle_events.len()
    }

    /// Returns the number of sandbox transport session events.
    pub fn transport_event_count(&self) -> usize {
        self.transport_events.len()
    }

    /// Returns the number of heartbeat cycle events.
    pub fn heartbeat_event_count(&self) -> usize {
        self.heartbeat_events.len()
    }

    /// Returns the number of block dispatch events.
    pub fn block_dispatch_event_count(&self) -> usize {
        self.block_dispatch_events.len()
    }

    /// Returns the number of lease rollover events.
    pub fn lease_rollover_event_count(&self) -> usize {
        self.lease_rollover_events.len()
    }

    /// Returns the number of broker invalidation events.
    pub fn invalidation_event_count(&self) -> usize {
        self.invalidation_events.len()
    }

    /// Returns the number of completion slot transition events.
    pub fn completion_slot_event_count(&self) -> usize {
        self.completion_slot_events.len()
    }

    /// Returns the number of transport fault events.
    pub fn transport_fault_event_count(&self) -> usize {
        self.transport_fault_events.len()
    }

    /// Returns the number of broker failure events.
    pub fn broker_failure_event_count(&self) -> usize {
        self.broker_failure_events.len()
    }

    /// Returns the number of sandbox operation failure events.
    pub fn sandbox_operation_failure_event_count(&self) -> usize {
        self.sandbox_operation_failure_events.len()
    }

    /// Returns the count of plugin faults whose detail string contains `needle`.
    pub fn fault_detail_count_containing(&self, needle: &str) -> usize {
        self.plugin_faults
            .iter()
            .filter(|fault| fault.detail.contains(needle))
            .count()
    }

    /// Returns the most recent supervision update snapshot, if any.
    pub fn last_supervision_update(&self) -> Option<&RuntimeSupervisionSnapshot> {
        self.supervision_updates.last()
    }

    /// Returns the most recent recovery event record, if any.
    pub fn last_recovery_event(&self) -> Option<&RecoveryRecord> {
        self.recovery_events.last()
    }

    /// Returns the most recent plugin instance state record, if any.
    pub fn last_plugin_instance_state(&self) -> Option<&PluginSandboxInstanceStateRecord> {
        self.plugin_instance_states.last()
    }

    /// Returns the most recent sandbox lifecycle event record, if any.
    pub fn last_lifecycle_event(&self) -> Option<&PluginSandboxLifecycleRecord> {
        self.lifecycle_events.last()
    }

    /// Returns the most recent sandbox transport event record, if any.
    pub fn last_transport_event(&self) -> Option<&PluginSandboxTransportRecord> {
        self.transport_events.last()
    }

    /// Returns the most recent heartbeat cycle event record, if any.
    pub fn last_heartbeat_event(&self) -> Option<&HeartbeatCycleRecord> {
        self.heartbeat_events.last()
    }

    /// Returns the most recent block dispatch event record, if any.
    pub fn last_block_dispatch_event(&self) -> Option<&BlockDispatchRecord> {
        self.block_dispatch_events.last()
    }

    /// Returns the most recent lease rollover event record, if any.
    pub fn last_lease_rollover_event(&self) -> Option<&LeaseRolloverRecord> {
        self.lease_rollover_events.last()
    }

    /// Returns the most recent broker invalidation event record, if any.
    pub fn last_invalidation_event(&self) -> Option<&BrokerInvalidationRecord> {
        self.invalidation_events.last()
    }

    /// Returns the most recent completion slot transition event record, if any.
    pub fn last_completion_slot_event(&self) -> Option<&CompletionSlotRecord> {
        self.completion_slot_events.last()
    }

    /// Returns the most recent transport fault event record, if any.
    pub fn last_transport_fault_event(&self) -> Option<&TransportFaultRecord> {
        self.transport_fault_events.last()
    }

    /// Returns the most recent broker failure event record, if any.
    pub fn last_broker_failure_event(&self) -> Option<&BrokerFailureRecord> {
        self.broker_failure_events.last()
    }

    /// Returns the most recent sandbox operation failure event record, if any.
    pub fn last_sandbox_operation_failure_event(&self) -> Option<&SandboxOperationFailureRecord> {
        self.sandbox_operation_failure_events.last()
    }
}
