use signal_ipc::CorrelationId;

use crate::PluginFormat;

/// A request from the host to spawn a new plugin sandbox process.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PluginSandboxRequest {
    /// Unique identifier for the sandbox to be created.
    pub sandbox_id: String,
    /// Plugin format the sandbox will host.
    pub format: PluginFormat,
    /// Isolation level for the sandbox process.
    pub policy: crate::SandboxPolicy,
    /// Optional correlation token for matching this request to a response.
    pub correlation_id: Option<CorrelationId>,
}

impl PluginSandboxRequest {
    /// Creates a new sandbox request with no correlation ID.
    pub fn new(
        sandbox_id: impl Into<String>,
        format: PluginFormat,
        policy: crate::SandboxPolicy,
    ) -> Self {
        Self {
            sandbox_id: sandbox_id.into(),
            format,
            policy,
            correlation_id: None,
        }
    }
}

/// The IPC mechanism used to exchange audio blocks between host and sandbox.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SandboxTransport {
    /// Audio and event data are exchanged through a shared-memory region.
    SharedMemory,
}

/// Capabilities reported by a sandbox process during handshake.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PluginSandboxCapabilities {
    /// Transport mechanism the sandbox supports.
    pub transport: SandboxTransport,
    /// Sandbox supports saving and restoring plugin state.
    pub supports_state: bool,
    /// Sandbox supports raw MIDI event delivery.
    pub supports_midi: bool,
    /// Maximum block size the sandbox can process.
    pub max_block_frames: u32,
}
