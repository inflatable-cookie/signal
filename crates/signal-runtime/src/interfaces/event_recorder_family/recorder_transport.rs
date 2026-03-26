use super::*;

impl RuntimeEventRecorder {
    pub fn transport_events(&self) -> Vec<PluginSandboxTransportRecord> {
        self.snapshot()
            .into_iter()
            .filter_map(|event| match event {
                RuntimeEvent::PluginSandboxTransport {
                    sandbox_id,
                    lease_id,
                    region_id,
                    stage,
                    processing_epoch,
                    detail,
                } => Some(PluginSandboxTransportRecord {
                    sandbox_id,
                    lease_id,
                    region_id,
                    stage,
                    processing_epoch,
                    detail,
                }),
                _ => None,
            })
            .collect()
    }

    pub fn heartbeat_events(&self) -> Vec<HeartbeatCycleRecord> {
        self.snapshot()
            .into_iter()
            .filter_map(|event| match event {
                RuntimeEvent::HeartbeatCycle {
                    sandbox_id,
                    stage,
                    processing_epoch,
                    block_sequence,
                } => Some(HeartbeatCycleRecord {
                    sandbox_id,
                    stage,
                    processing_epoch,
                    block_sequence,
                }),
                _ => None,
            })
            .collect()
    }

    pub fn block_dispatch_events(&self) -> Vec<BlockDispatchRecord> {
        self.snapshot()
            .into_iter()
            .filter_map(|event| match event {
                RuntimeEvent::BlockDispatch {
                    sandbox_id,
                    lease_id,
                    processing_epoch,
                    block_sequence,
                    frame_count,
                    stage,
                    completion_state,
                } => Some(BlockDispatchRecord {
                    sandbox_id,
                    lease_id,
                    processing_epoch,
                    block_sequence,
                    frame_count,
                    stage,
                    completion_state,
                }),
                _ => None,
            })
            .collect()
    }

    pub fn lease_rollover_events(&self) -> Vec<LeaseRolloverRecord> {
        self.snapshot()
            .into_iter()
            .filter_map(|event| match event {
                RuntimeEvent::LeaseRollover {
                    sandbox_id,
                    previous_lease_id,
                    lease_id,
                    processing_epoch,
                    first_block_sequence,
                } => Some(LeaseRolloverRecord {
                    sandbox_id,
                    previous_lease_id,
                    lease_id,
                    processing_epoch,
                    first_block_sequence,
                }),
                _ => None,
            })
            .collect()
    }

    pub fn invalidation_events(&self) -> Vec<BrokerInvalidationRecord> {
        self.snapshot()
            .into_iter()
            .filter_map(|event| match event {
                RuntimeEvent::BrokerInvalidation {
                    sandbox_id,
                    lease_id,
                    processing_epoch,
                    block_sequence,
                    stage,
                    reason,
                } => Some(BrokerInvalidationRecord {
                    sandbox_id,
                    lease_id,
                    processing_epoch,
                    block_sequence,
                    stage,
                    reason,
                }),
                _ => None,
            })
            .collect()
    }

    pub fn completion_slot_events(&self) -> Vec<CompletionSlotRecord> {
        self.snapshot()
            .into_iter()
            .filter_map(|event| match event {
                RuntimeEvent::CompletionSlotTransition {
                    sandbox_id,
                    lease_id,
                    processing_epoch,
                    block_sequence,
                    stage,
                } => Some(CompletionSlotRecord {
                    sandbox_id,
                    lease_id,
                    processing_epoch,
                    block_sequence,
                    stage,
                }),
                _ => None,
            })
            .collect()
    }

    pub fn broker_failure_events(&self) -> Vec<BrokerFailureRecord> {
        self.snapshot()
            .into_iter()
            .filter_map(|event| match event {
                RuntimeEvent::BrokerFailure {
                    sandbox_id,
                    lease_id,
                    processing_epoch,
                    block_sequence,
                    stage,
                    detail,
                } => Some(BrokerFailureRecord {
                    sandbox_id,
                    lease_id,
                    processing_epoch,
                    block_sequence,
                    stage,
                    detail,
                }),
                _ => None,
            })
            .collect()
    }

    pub fn transport_fault_events(&self) -> Vec<TransportFaultRecord> {
        self.snapshot()
            .into_iter()
            .filter_map(transport_fault_record_from_event)
            .collect()
    }
}
