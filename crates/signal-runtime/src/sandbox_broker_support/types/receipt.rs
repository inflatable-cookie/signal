//! Broker receipt state tokens and parsed receipt lines.

/// Record of a successfully attached sandbox broker session.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SandboxBrokerAttachedSession {
    /// Stable identifier for the attached sandbox.
    pub sandbox_id: String,
    /// Instance identifier assigned by the broker.
    pub instance_id: String,
    /// Processing epoch at which the session was attached.
    pub processing_epoch: u64,
    /// Shared-memory lease identifier for this session.
    pub lease_id: String,
    /// Shared-memory region identifier for this session.
    pub region_id: String,
    /// Human-readable detail from the broker attach receipt.
    pub detail: String,
}

/// Broker receipt `state=` token, parsed into a typed value.
///
/// The wire format is unchanged: states travel as plain tokens
/// (`starting`, `ready`, `attached`, `running`, `crashed`, `timed_out`,
/// `teardown_complete`). Unknown tokens are preserved in [`Self::Other`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SandboxBrokerReceiptState {
    /// Broker process is starting up.
    Starting,
    /// Broker process is ready to accept commands.
    Ready,
    /// A sandbox session is attached.
    Attached,
    /// An execution stream block was processed.
    Running,
    /// The brokered sandbox crashed.
    Crashed,
    /// A bounded deadline-miss (timeout) path was exercised.
    TimedOut,
    /// A teardown sequence completed.
    TeardownComplete,
    /// A CLAP plugin was loaded and its inventory enumerated (g11.012).
    PluginLoaded,
    /// The loaded plugin activated and leased its audio block region.
    PluginActivated,
    /// The plugin's main-port layout is outside the supported v1 shape
    /// (stereo in + stereo out); the chain compiles passthrough.
    LayoutUnsupported,
    /// The child's audio thread is live and serving process requests.
    ProcessingStarted,
    /// The child's audio thread stopped.
    ProcessingStopped,
    /// The plugin deactivated and its audio block region was destroyed.
    PluginDeactivated,
    /// The plugin instance was destroyed and its library closed.
    PluginUnloaded,
    /// A parameter write batch was applied to the loaded instance
    /// (g12.023).
    ParamSet,
    /// A child-owned editor window opened for the loaded instance
    /// (g13.027).
    EditorOpened,
    /// A child-owned editor window closed — as a command receipt
    /// (`reason=host_requested` / `reason=not_open`) or as a spontaneous
    /// child→parent notification (`reason=user_closed`).
    EditorClosed,
    /// Any state token this client does not recognise.
    Other(String),
}

impl SandboxBrokerReceiptState {
    pub(crate) fn parse(token: &str) -> Self {
        match token {
            "starting" => Self::Starting,
            "ready" => Self::Ready,
            "attached" => Self::Attached,
            "running" => Self::Running,
            "crashed" => Self::Crashed,
            "timed_out" => Self::TimedOut,
            "teardown_complete" => Self::TeardownComplete,
            "plugin_loaded" => Self::PluginLoaded,
            "plugin_activated" => Self::PluginActivated,
            "layout_unsupported" => Self::LayoutUnsupported,
            "processing_started" => Self::ProcessingStarted,
            "processing_stopped" => Self::ProcessingStopped,
            "plugin_deactivated" => Self::PluginDeactivated,
            "plugin_unloaded" => Self::PluginUnloaded,
            "param_set" => Self::ParamSet,
            "editor_opened" => Self::EditorOpened,
            "editor_closed" => Self::EditorClosed,
            other => Self::Other(other.to_string()),
        }
    }
}

impl std::fmt::Display for SandboxBrokerReceiptState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let token = match self {
            Self::Starting => "starting",
            Self::Ready => "ready",
            Self::Attached => "attached",
            Self::Running => "running",
            Self::Crashed => "crashed",
            Self::TimedOut => "timed_out",
            Self::TeardownComplete => "teardown_complete",
            Self::PluginLoaded => "plugin_loaded",
            Self::PluginActivated => "plugin_activated",
            Self::LayoutUnsupported => "layout_unsupported",
            Self::ProcessingStarted => "processing_started",
            Self::ProcessingStopped => "processing_stopped",
            Self::PluginDeactivated => "plugin_deactivated",
            Self::PluginUnloaded => "plugin_unloaded",
            Self::ParamSet => "param_set",
            Self::EditorOpened => "editor_opened",
            Self::EditorClosed => "editor_closed",
            Self::Other(other) => other.as_str(),
        };
        f.write_str(token)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct SandboxBrokerReceiptLine {
    pub(crate) state: SandboxBrokerReceiptState,
    pub(crate) sandbox_id: String,
    pub(crate) instance_id: Option<String>,
    pub(crate) processing_epoch: Option<u64>,
    pub(crate) lease_id: Option<String>,
    pub(crate) region_id: Option<String>,
    /// Extra `key=value` tokens (plugin inventory, shm coordinates);
    /// values remain wire-encoded until interpreted.
    pub(crate) extra: Vec<(String, String)>,
    pub(crate) detail: String,
}

impl SandboxBrokerReceiptLine {
    pub(crate) fn extra_value(&self, key: &str) -> Option<&str> {
        self.extra
            .iter()
            .find(|(extra_key, _)| extra_key == key)
            .map(|(_, value)| value.as_str())
    }
}

/// Receipt returned by a broker teardown request.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SandboxBrokerTeardownReceipt {
    /// Teardown receipt state reported by the broker.
    pub state: SandboxBrokerReceiptState,
    /// Instance identifier from the receipt, if present.
    pub instance_id: Option<String>,
    /// Processing epoch from the receipt, if present.
    pub processing_epoch: Option<u64>,
    /// Shared-memory lease identifier from the receipt, if present.
    pub lease_id: Option<String>,
    /// Shared-memory region identifier from the receipt, if present.
    pub region_id: Option<String>,
    /// Human-readable detail from the receipt.
    pub detail: String,
}
