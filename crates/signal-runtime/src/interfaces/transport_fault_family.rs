/// Origin subsystem that reported a transport fault.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TransportFaultSource {
    /// Fault originated in the host broker subsystem.
    HostBroker,
    /// Fault originated in a sandbox operation.
    SandboxOperation,
    /// Fault originated in the runtime dispatch layer.
    RuntimeDispatch,
}

/// Fine-grained lifecycle stage at which a transport fault occurred.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TransportFaultStage {
    /// Fault occurred while creating the prepare plan.
    PreparePlanCreate,
    /// Fault occurred while writing the shared memory payload.
    PayloadWrite,
    /// Fault occurred while reading the shared memory payload.
    PayloadRead,
    /// Fault occurred while destroying the transport.
    TransportDestroy,
    /// Fault occurred during transport teardown.
    TransportTeardown,
    /// Fault occurred while requesting a transport detach.
    TransportDetachRequested,
    /// Fault occurred after transport was detached.
    TransportDetached,
    /// Fault occurred while preparing an attach operation.
    PrepareAttach,
    /// Fault occurred while processing an attach operation.
    ProcessAttach,
    /// Fault occurred while flushing the process pipeline.
    ProcessFlush,
    /// Fault caused by a process protocol violation.
    ProcessProtocolViolation,
    /// Fault caused by a control protocol violation.
    ControlProtocolViolation,
    /// Fault occurred during a transport detach fault.
    TransportDetachFault,
    /// Fault caused by a completion region invalidation.
    CompletionRegionInvalidated,
    /// Fault caused by a lease epoch invalidation.
    LeaseEpochInvalidated,
    /// Fault caused by a completion slot timeout.
    CompletionSlotTimedOut,
    /// Fault caused by a completion slot invalidation.
    CompletionSlotInvalidated,
    /// Fault caused by a fallback being applied.
    FallbackApplied,
}

/// High-level phase of the transport session during which a fault was recorded.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TransportFaultPhase {
    /// Fault occurred during the prepare phase.
    Prepare,
    /// Fault occurred during the dispatch phase.
    Dispatch,
    /// Fault occurred during the teardown phase.
    Teardown,
    /// Fault occurred during the control phase.
    Control,
}

/// Resource involved in a transport fault.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TransportFaultResource {
    /// The prepare plan resource was involved.
    PreparePlan,
    /// The shared memory payload resource was involved.
    SharedMemoryPayload,
    /// The shared memory lease resource was involved.
    SharedMemoryLease,
    /// The completion slot resource was involved.
    CompletionSlot,
    /// The process protocol resource was involved.
    ProcessProtocol,
    /// The control protocol resource was involved.
    ControlProtocol,
}

/// Full fault record for one transport session failure event.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TransportFaultRecord {
    /// Sandbox ID where the fault occurred.
    pub sandbox_id: String,
    /// Lease ID associated with the faulted session.
    pub lease_id: Option<String>,
    /// Processing epoch at the time of the fault.
    pub processing_epoch: Option<u64>,
    /// Block sequence number at the time of the fault.
    pub block_sequence: Option<u64>,
    /// Subsystem that originated the fault.
    pub source: TransportFaultSource,
    /// Lifecycle stage at which the fault occurred.
    pub stage: TransportFaultStage,
    /// High-level phase of the transport session during the fault.
    pub phase: TransportFaultPhase,
    /// Resource involved in the fault.
    pub resource: TransportFaultResource,
    /// Operation name that failed.
    pub operation: String,
    /// Error kind string from the underlying failure.
    pub error_kind: Option<String>,
}

/// Scoping policy for transport fault aggregation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TransportFaultBoundaryMode {
    /// Aggregate only faults adjacent to the current fault boundary.
    FaultAdjacentOnly,
}

/// Aggregate transport fault counts and epoch bounds derived from a set of fault records.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TransportFaultSummary {
    /// Scoping policy used to aggregate these records.
    pub boundary_mode: TransportFaultBoundaryMode,
    /// Total number of fault records.
    pub total_events: usize,
    /// Number of faults originating from the host broker.
    pub host_broker_events: usize,
    /// Number of faults originating from sandbox operations.
    pub sandbox_operation_events: usize,
    /// Number of faults originating from runtime dispatch.
    pub runtime_dispatch_events: usize,
    /// Number of faults in the prepare phase.
    pub prepare_events: usize,
    /// Number of faults in the dispatch phase.
    pub dispatch_events: usize,
    /// Number of faults in the teardown phase.
    pub teardown_events: usize,
    /// Number of faults in the control phase.
    pub control_events: usize,
    /// Earliest processing epoch seen across all records.
    pub first_processing_epoch: Option<u64>,
    /// Latest processing epoch seen across all records.
    pub last_processing_epoch: Option<u64>,
    /// Earliest block sequence seen across all records.
    pub first_block_sequence: Option<u64>,
    /// Latest block sequence seen across all records.
    pub last_block_sequence: Option<u64>,
}

impl TransportFaultSummary {
    /// Builds an aggregate fault summary by scanning all fault records and tallying source, phase, and epoch bounds.
    pub fn from_records(records: &[TransportFaultRecord]) -> Self {
        let mut summary = Self {
            boundary_mode: TransportFaultBoundaryMode::FaultAdjacentOnly,
            total_events: records.len(),
            host_broker_events: 0,
            sandbox_operation_events: 0,
            runtime_dispatch_events: 0,
            prepare_events: 0,
            dispatch_events: 0,
            teardown_events: 0,
            control_events: 0,
            first_processing_epoch: None,
            last_processing_epoch: None,
            first_block_sequence: None,
            last_block_sequence: None,
        };

        for record in records {
            match record.source {
                TransportFaultSource::HostBroker => {
                    summary.host_broker_events = summary.host_broker_events.saturating_add(1)
                }
                TransportFaultSource::SandboxOperation => {
                    summary.sandbox_operation_events =
                        summary.sandbox_operation_events.saturating_add(1)
                }
                TransportFaultSource::RuntimeDispatch => {
                    summary.runtime_dispatch_events =
                        summary.runtime_dispatch_events.saturating_add(1)
                }
            }
            match record.phase {
                TransportFaultPhase::Prepare => {
                    summary.prepare_events = summary.prepare_events.saturating_add(1)
                }
                TransportFaultPhase::Dispatch => {
                    summary.dispatch_events = summary.dispatch_events.saturating_add(1)
                }
                TransportFaultPhase::Teardown => {
                    summary.teardown_events = summary.teardown_events.saturating_add(1)
                }
                TransportFaultPhase::Control => {
                    summary.control_events = summary.control_events.saturating_add(1)
                }
            }

            if let Some(epoch) = record.processing_epoch {
                summary.first_processing_epoch = Some(
                    summary
                        .first_processing_epoch
                        .map_or(epoch, |current| current.min(epoch)),
                );
                summary.last_processing_epoch = Some(
                    summary
                        .last_processing_epoch
                        .map_or(epoch, |current| current.max(epoch)),
                );
            }
            if let Some(block_sequence) = record.block_sequence {
                summary.first_block_sequence = Some(
                    summary
                        .first_block_sequence
                        .map_or(block_sequence, |current| current.min(block_sequence)),
                );
                summary.last_block_sequence = Some(
                    summary
                        .last_block_sequence
                        .map_or(block_sequence, |current| current.max(block_sequence)),
                );
            }
        }

        summary
    }
}
