#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TransportFaultSource {
    HostBroker,
    SandboxOperation,
    RuntimeDispatch,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TransportFaultStage {
    PreparePlanCreate,
    PayloadWrite,
    PayloadRead,
    TransportDestroy,
    TransportTeardown,
    TransportDetachRequested,
    TransportDetached,
    PrepareAttach,
    ProcessAttach,
    ProcessFlush,
    ProcessProtocolViolation,
    ControlProtocolViolation,
    TransportDetachFault,
    CompletionRegionInvalidated,
    LeaseEpochInvalidated,
    CompletionSlotTimedOut,
    CompletionSlotInvalidated,
    FallbackApplied,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TransportFaultPhase {
    Prepare,
    Dispatch,
    Teardown,
    Control,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TransportFaultResource {
    PreparePlan,
    SharedMemoryPayload,
    SharedMemoryLease,
    CompletionSlot,
    ProcessProtocol,
    ControlProtocol,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TransportFaultRecord {
    pub sandbox_id: String,
    pub lease_id: Option<String>,
    pub processing_epoch: Option<u64>,
    pub block_sequence: Option<u64>,
    pub source: TransportFaultSource,
    pub stage: TransportFaultStage,
    pub phase: TransportFaultPhase,
    pub resource: TransportFaultResource,
    pub operation: String,
    pub error_kind: Option<String>,
    pub detail: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TransportFaultBoundaryMode {
    FaultAdjacentOnly,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TransportFaultSummary {
    pub boundary_mode: TransportFaultBoundaryMode,
    pub total_events: usize,
    pub host_broker_events: usize,
    pub sandbox_operation_events: usize,
    pub runtime_dispatch_events: usize,
    pub prepare_events: usize,
    pub dispatch_events: usize,
    pub teardown_events: usize,
    pub control_events: usize,
    pub first_processing_epoch: Option<u64>,
    pub last_processing_epoch: Option<u64>,
    pub first_block_sequence: Option<u64>,
    pub last_block_sequence: Option<u64>,
}

impl TransportFaultSummary {
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
