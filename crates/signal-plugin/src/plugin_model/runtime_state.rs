use super::{PluginLifecycleState, PluginProcessConfiguration, PluginTypeId};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PluginInstanceId(pub String);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PluginFaultKind {
    InvalidRequest,
    UnsupportedCapability,
    InvalidState,
    ResourceUnavailable,
    ProcessingFailure,
    ProtocolViolation,
    Timeout,
    Crash,
    Fatal,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PluginFaultSeverity {
    Warning,
    Recoverable,
    Critical,
    Fatal,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PluginFault {
    pub kind: PluginFaultKind,
    pub severity: PluginFaultSeverity,
    pub message: String,
}

impl PluginFault {
    pub fn new(
        kind: PluginFaultKind,
        severity: PluginFaultSeverity,
        message: impl Into<String>,
    ) -> Self {
        Self {
            kind,
            severity,
            message: message.into(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PluginDegradedReason(pub &'static str);

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PluginReadiness {
    Starting,
    Ready,
    Degraded { reasons: Vec<PluginDegradedReason> },
    Stopped,
    Failed { fatal: PluginFault },
}

impl PluginReadiness {
    pub fn from_fault(fault: PluginFault) -> Self {
        match fault.severity {
            PluginFaultSeverity::Warning | PluginFaultSeverity::Recoverable => Self::Degraded {
                reasons: vec![PluginDegradedReason("plugin-fault")],
            },
            PluginFaultSeverity::Critical | PluginFaultSeverity::Fatal => {
                Self::Failed { fatal: fault }
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct PluginInstanceSnapshot {
    pub plugin_type_id: PluginTypeId,
    pub instance_id: PluginInstanceId,
    pub lifecycle_state: PluginLifecycleState,
    pub readiness: PluginReadiness,
    pub processing: Option<PluginProcessConfiguration>,
}
