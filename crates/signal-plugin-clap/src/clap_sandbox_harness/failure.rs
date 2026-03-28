use signal_ipc::{
    CorrelationId, PluginFaultPayload, PluginInstanceStatePayload, PluginMessageEnvelope,
    PluginMessageName, PluginMessagePayload,
};
use signal_plugin::{PluginFault, PluginFaultKind, PluginFaultSeverity, PluginLifecycleState};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ClapSandboxFailureInput {
    pub sandbox_id: String,
    pub instance_id: Option<String>,
    pub stage: String,
    pub error_kind: String,
    pub detail: String,
    pub processing_epoch: Option<u64>,
    pub shared_memory_lease_id: Option<String>,
    pub correlation_id: Option<CorrelationId>,
    pub instance_state: Option<PluginInstanceStatePayload>,
}

pub fn sandbox_failure_event(input: ClapSandboxFailureInput) -> PluginMessageEnvelope {
    failure_event(input)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ClapSandboxFailureStage {
    PrepareAttach,
    ProcessAttach,
    ProcessFlush,
    ProcessProtocolViolation,
    ControlProtocolViolation,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ClapSandboxFailureClassification {
    pub sandbox_id: String,
    pub lease_id: Option<String>,
    pub processing_epoch: Option<u64>,
    pub operation: String,
    pub error_kind: String,
    pub stage: ClapSandboxFailureStage,
    pub detail: String,
}

pub fn classify_sandbox_failure(
    envelope: &PluginMessageEnvelope,
) -> Option<ClapSandboxFailureClassification> {
    let PluginMessagePayload::SandboxFailure {
        sandbox_id,
        stage,
        error_kind,
        detail,
        processing_epoch,
        shared_memory_lease_id,
        ..
    } = &envelope.payload
    else {
        return None;
    };

    let stage = match (stage.as_str(), error_kind.as_str()) {
        ("prepareInstance", "resourceUnavailable")
            if detail.contains("attach shared-memory region") =>
        {
            ClapSandboxFailureStage::PrepareAttach
        }
        ("processBlock", "resourceUnavailable")
            if detail.contains("attach shared-memory region") =>
        {
            ClapSandboxFailureStage::ProcessAttach
        }
        ("processBlock", "resourceUnavailable")
            if detail.contains("flush shared-memory region") =>
        {
            ClapSandboxFailureStage::ProcessFlush
        }
        ("processBlock", "protocolViolation") => ClapSandboxFailureStage::ProcessProtocolViolation,
        (_, "protocolViolation") => ClapSandboxFailureStage::ControlProtocolViolation,
        _ => return None,
    };

    Some(ClapSandboxFailureClassification {
        sandbox_id: sandbox_id.clone(),
        lease_id: shared_memory_lease_id.clone(),
        processing_epoch: *processing_epoch,
        operation: stage_string(stage, &envelope.payload),
        error_kind: error_kind.clone(),
        stage,
        detail: detail.clone(),
    })
}

pub(super) fn lifecycle_state_string(state: PluginLifecycleState) -> &'static str {
    match state {
        PluginLifecycleState::Discovered => "Discovered",
        PluginLifecycleState::TypeLoaded => "TypeLoaded",
        PluginLifecycleState::InstanceCreated => "InstanceCreated",
        PluginLifecycleState::Prepared => "Prepared",
        PluginLifecycleState::Active => "Active",
        PluginLifecycleState::Inactive => "Inactive",
        PluginLifecycleState::Released => "Released",
        PluginLifecycleState::Faulted => "Faulted",
    }
}

pub(super) fn plugin_fault_payload(fault: &PluginFault) -> PluginFaultPayload {
    PluginFaultPayload {
        kind: match fault.kind {
            PluginFaultKind::InvalidRequest => "invalidRequest",
            PluginFaultKind::UnsupportedCapability => "unsupportedCapability",
            PluginFaultKind::InvalidState => "invalidState",
            PluginFaultKind::ResourceUnavailable => "resourceUnavailable",
            PluginFaultKind::ProcessingFailure => "processingFailure",
            PluginFaultKind::ProtocolViolation => "protocolViolation",
            PluginFaultKind::Timeout => "timeout",
            PluginFaultKind::Crash => "crash",
            PluginFaultKind::Fatal => "fatal",
        }
        .into(),
        severity: match fault.severity {
            PluginFaultSeverity::Warning => "warning",
            PluginFaultSeverity::Recoverable => "recoverable",
            PluginFaultSeverity::Critical => "critical",
            PluginFaultSeverity::Fatal => "fatal",
        }
        .into(),
        message: fault.message.clone(),
    }
}

fn plugin_fault_for_error_kind(error_kind: &str, detail: &str) -> PluginFault {
    let kind = match error_kind {
        "invalidRequest" => PluginFaultKind::InvalidRequest,
        "invalidState" => PluginFaultKind::InvalidState,
        "unsupported" => PluginFaultKind::UnsupportedCapability,
        "resourceUnavailable" => PluginFaultKind::ResourceUnavailable,
        "protocolViolation" => PluginFaultKind::ProtocolViolation,
        "timeout" => PluginFaultKind::Timeout,
        "processingFailure" => PluginFaultKind::ProcessingFailure,
        "crashed" => PluginFaultKind::Crash,
        _ => PluginFaultKind::Fatal,
    };
    let severity = match kind {
        PluginFaultKind::InvalidRequest
        | PluginFaultKind::InvalidState
        | PluginFaultKind::UnsupportedCapability => PluginFaultSeverity::Warning,
        PluginFaultKind::ResourceUnavailable
        | PluginFaultKind::ProcessingFailure
        | PluginFaultKind::Timeout => PluginFaultSeverity::Recoverable,
        PluginFaultKind::ProtocolViolation => PluginFaultSeverity::Critical,
        PluginFaultKind::Crash | PluginFaultKind::Fatal => PluginFaultSeverity::Fatal,
    };
    PluginFault::new(kind, severity, detail)
}

fn stage_string(stage: ClapSandboxFailureStage, payload: &PluginMessagePayload) -> String {
    match payload {
        PluginMessagePayload::SandboxFailure { stage, .. } => stage.clone(),
        _ => format!("{stage:?}"),
    }
}

pub(super) fn failure_event(input: ClapSandboxFailureInput) -> PluginMessageEnvelope {
    let ClapSandboxFailureInput {
        sandbox_id,
        instance_id,
        stage,
        error_kind,
        detail,
        processing_epoch,
        shared_memory_lease_id,
        correlation_id,
        instance_state,
    } = input;
    let fault = plugin_fault_for_error_kind(&error_kind, &detail);
    PluginMessageEnvelope::event(
        PluginMessageName::SandboxFailure,
        correlation_id,
        PluginMessagePayload::SandboxFailure {
            sandbox_id,
            instance_id,
            stage,
            error_kind,
            detail,
            fault: plugin_fault_payload(&fault),
            instance_state,
            processing_epoch,
            shared_memory_lease_id,
        },
    )
}
