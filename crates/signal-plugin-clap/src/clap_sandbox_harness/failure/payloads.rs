use signal_ipc::PluginFaultPayload;
use signal_plugin::{PluginFault, PluginFaultKind, PluginFaultSeverity, PluginLifecycleState};

pub(crate) fn lifecycle_state_string(state: PluginLifecycleState) -> &'static str {
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

pub(crate) fn plugin_fault_payload(fault: &PluginFault) -> PluginFaultPayload {
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

pub(super) fn plugin_fault_for_error_kind(error_kind: &str, detail: &str) -> PluginFault {
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
