use super::{PluginLifecycleState, PluginProcessConfiguration, PluginTypeId};

/// Runtime identity of a specific plugin instance, unique within the session.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PluginInstanceId(pub String);

/// Classification of the error that caused a plugin fault.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PluginFaultKind {
    /// The host sent a request that was structurally or semantically invalid.
    InvalidRequest,
    /// The plugin does not support the requested capability.
    UnsupportedCapability,
    /// The operation was not valid in the plugin's current lifecycle state.
    InvalidState,
    /// A required resource (memory, file, IPC handle) was not available.
    ResourceUnavailable,
    /// An error occurred during audio processing.
    ProcessingFailure,
    /// The sandbox or plugin violated the expected communication protocol.
    ProtocolViolation,
    /// The plugin did not respond within the allowed time.
    Timeout,
    /// The plugin process crashed.
    Crash,
    /// An unrecoverable fatal error; the plugin cannot continue.
    Fatal,
}

/// How severe a plugin fault is and whether recovery is possible.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PluginFaultSeverity {
    /// Non-critical; the plugin can continue operating.
    Warning,
    /// The current operation failed but the plugin can be retried or reset.
    Recoverable,
    /// The plugin is in a bad state; it should be restarted.
    Critical,
    /// The plugin cannot recover; it must be released.
    Fatal,
}

/// A structured error from a plugin or its sandbox.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PluginFault {
    /// Classification of the fault.
    pub kind: PluginFaultKind,
    /// Severity of the fault.
    pub severity: PluginFaultSeverity,
    /// Human-readable description of what went wrong.
    pub message: String,
}

impl PluginFault {
    /// Creates a new `PluginFault` with the given kind, severity, and message.
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

/// A static description of why a plugin is operating in a degraded state.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PluginDegradedReason(pub &'static str);

/// Operational readiness of a plugin instance as seen by the host.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PluginReadiness {
    /// The plugin is initialising and not yet ready to process.
    Starting,
    /// The plugin is fully operational.
    Ready,
    /// The plugin is running but with one or more known limitations.
    Degraded {
        /// The known limitations currently affecting this plugin.
        reasons: Vec<PluginDegradedReason>,
    },
    /// The plugin has been stopped cleanly.
    Stopped,
    /// The plugin has failed with a fatal fault and cannot continue.
    Failed {
        /// The fatal fault that caused the plugin to stop.
        fatal: PluginFault,
    },
}

impl PluginReadiness {
    /// Converts a [`PluginFault`] into the appropriate readiness state.
    ///
    /// Warning or recoverable faults yield `Degraded`; critical or fatal faults yield `Failed`.
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

/// A point-in-time snapshot of a plugin instance's observable state.
#[derive(Clone, Debug, PartialEq)]
pub struct PluginInstanceSnapshot {
    /// The plugin type this instance was created from.
    pub plugin_type_id: PluginTypeId,
    /// The unique identifier for this instance.
    pub instance_id: PluginInstanceId,
    /// Current lifecycle phase.
    pub lifecycle_state: PluginLifecycleState,
    /// Current operational readiness.
    pub readiness: PluginReadiness,
    /// Active processing configuration, present only while the instance is prepared or active.
    pub processing: Option<PluginProcessConfiguration>,
}
