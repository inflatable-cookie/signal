use super::*;

pub(crate) fn map_broker_failure_stage(stage: BrokerFailureStage) -> TransportFaultStage {
    match stage {
        BrokerFailureStage::PreparePlanCreate => TransportFaultStage::PreparePlanCreate,
        BrokerFailureStage::PayloadWrite => TransportFaultStage::PayloadWrite,
        BrokerFailureStage::PayloadRead => TransportFaultStage::PayloadRead,
        BrokerFailureStage::TransportDestroy => TransportFaultStage::TransportDestroy,
        BrokerFailureStage::TransportTeardown => TransportFaultStage::TransportTeardown,
    }
}

pub(crate) fn map_broker_failure_phase(stage: BrokerFailureStage) -> TransportFaultPhase {
    match stage {
        BrokerFailureStage::PreparePlanCreate => TransportFaultPhase::Prepare,
        BrokerFailureStage::PayloadWrite | BrokerFailureStage::PayloadRead => {
            TransportFaultPhase::Dispatch
        }
        BrokerFailureStage::TransportDestroy | BrokerFailureStage::TransportTeardown => {
            TransportFaultPhase::Teardown
        }
    }
}

pub(crate) fn map_broker_failure_resource(stage: BrokerFailureStage) -> TransportFaultResource {
    match stage {
        BrokerFailureStage::PreparePlanCreate => TransportFaultResource::PreparePlan,
        BrokerFailureStage::PayloadWrite | BrokerFailureStage::PayloadRead => {
            TransportFaultResource::SharedMemoryPayload
        }
        BrokerFailureStage::TransportDestroy | BrokerFailureStage::TransportTeardown => {
            TransportFaultResource::SharedMemoryLease
        }
    }
}

pub(crate) fn broker_failure_operation(stage: BrokerFailureStage) -> &'static str {
    match stage {
        BrokerFailureStage::PreparePlanCreate => "prepare_plan.create",
        BrokerFailureStage::PayloadWrite => "block_payload.write",
        BrokerFailureStage::PayloadRead => "block_payload.read",
        BrokerFailureStage::TransportDestroy => "lease.destroy_region",
        BrokerFailureStage::TransportTeardown => "lease.teardown_transport",
    }
}

pub(crate) fn map_broker_invalidation_stage(stage: BrokerInvalidationStage) -> TransportFaultStage {
    match stage {
        BrokerInvalidationStage::CompletionRegionInvalidated => {
            TransportFaultStage::CompletionRegionInvalidated
        }
        BrokerInvalidationStage::LeaseEpochInvalidated => {
            TransportFaultStage::LeaseEpochInvalidated
        }
    }
}

pub(crate) fn map_broker_invalidation_resource(
    stage: BrokerInvalidationStage,
) -> TransportFaultResource {
    match stage {
        BrokerInvalidationStage::CompletionRegionInvalidated => {
            TransportFaultResource::SharedMemoryPayload
        }
        BrokerInvalidationStage::LeaseEpochInvalidated => TransportFaultResource::SharedMemoryLease,
    }
}

pub(crate) fn broker_invalidation_operation(stage: BrokerInvalidationStage) -> &'static str {
    match stage {
        BrokerInvalidationStage::CompletionRegionInvalidated => "completion_region.invalidate",
        BrokerInvalidationStage::LeaseEpochInvalidated => "lease_epoch.invalidate",
    }
}

pub(crate) fn map_sandbox_operation_failure_stage(
    stage: SandboxOperationFailureStage,
) -> TransportFaultStage {
    match stage {
        SandboxOperationFailureStage::PrepareAttach => TransportFaultStage::PrepareAttach,
        SandboxOperationFailureStage::ProcessAttach => TransportFaultStage::ProcessAttach,
        SandboxOperationFailureStage::ProcessFlush => TransportFaultStage::ProcessFlush,
        SandboxOperationFailureStage::ProcessProtocolViolation => {
            TransportFaultStage::ProcessProtocolViolation
        }
        SandboxOperationFailureStage::ControlProtocolViolation => {
            TransportFaultStage::ControlProtocolViolation
        }
    }
}

pub(crate) fn map_sandbox_operation_failure_phase(
    stage: SandboxOperationFailureStage,
) -> TransportFaultPhase {
    match stage {
        SandboxOperationFailureStage::PrepareAttach => TransportFaultPhase::Prepare,
        SandboxOperationFailureStage::ProcessAttach
        | SandboxOperationFailureStage::ProcessFlush
        | SandboxOperationFailureStage::ProcessProtocolViolation => TransportFaultPhase::Dispatch,
        SandboxOperationFailureStage::ControlProtocolViolation => TransportFaultPhase::Control,
    }
}

pub(crate) fn map_sandbox_operation_failure_resource(
    stage: SandboxOperationFailureStage,
) -> TransportFaultResource {
    match stage {
        SandboxOperationFailureStage::PrepareAttach
        | SandboxOperationFailureStage::ProcessAttach => TransportFaultResource::SharedMemoryLease,
        SandboxOperationFailureStage::ProcessFlush => TransportFaultResource::SharedMemoryPayload,
        SandboxOperationFailureStage::ProcessProtocolViolation => {
            TransportFaultResource::ProcessProtocol
        }
        SandboxOperationFailureStage::ControlProtocolViolation => {
            TransportFaultResource::ControlProtocol
        }
    }
}
