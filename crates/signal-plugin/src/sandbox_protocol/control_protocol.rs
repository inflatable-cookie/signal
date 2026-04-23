use signal_ipc::CorrelationId;

use crate::{
    PluginDescriptor, PluginFormat, PluginInstanceId, PluginIoLayout, PluginTypeId,
    SharedMemoryLayout,
};

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

/// A command sent from the host to the sandbox over the control channel.
#[derive(Clone, Debug, PartialEq)]
pub enum SandboxControlCommand {
    /// Initiates the protocol handshake; must be the first message sent.
    Handshake,
    /// Registers a plugin type and its descriptor with the sandbox.
    LoadPluginType {
        /// Identifies the plugin type to register.
        plugin_type_id: PluginTypeId,
        /// Full static descriptor for the plugin type.
        descriptor: PluginDescriptor,
    },
    /// Asks the sandbox to create a new plugin instance.
    CreateInstance {
        /// Type to instantiate.
        plugin_type_id: PluginTypeId,
        /// Identity to assign to the new instance.
        instance_id: PluginInstanceId,
    },
    /// Asks the sandbox to destroy an existing plugin instance.
    DestroyInstance {
        /// Instance to destroy.
        instance_id: PluginInstanceId,
    },
    /// Prepares a plugin instance for audio processing at the given configuration.
    PrepareInstance {
        /// Instance to prepare.
        instance_id: PluginInstanceId,
        /// Sample rate in Hz.
        sample_rate_hz: u32,
        /// Maximum frames per block.
        max_block_frames: u32,
        /// Audio and MIDI bus layout.
        io_layout: PluginIoLayout,
        /// Shared-memory region layout to use for block transport.
        shared_memory: SharedMemoryLayout,
    },
    /// Activates a prepared instance and starts the given processing epoch.
    ActivateInstance {
        /// Instance to activate.
        instance_id: PluginInstanceId,
        /// Monotonically increasing epoch counter; used to detect stale blocks.
        processing_epoch: u64,
    },
    /// Deactivates an active instance without destroying it.
    DeactivateInstance {
        /// Instance to deactivate.
        instance_id: PluginInstanceId,
    },
    /// Resets an instance's state and starts a new processing epoch.
    ResetInstance {
        /// Instance to reset.
        instance_id: PluginInstanceId,
        /// New processing epoch to use after the reset.
        processing_epoch: u64,
    },
    /// Requests an orderly sandbox shutdown.
    Shutdown,
}

/// A control-channel message from the host to a sandbox, carrying a command and routing metadata.
#[derive(Clone, Debug, PartialEq)]
pub struct SandboxControlRequest {
    /// Identifies the target sandbox.
    pub sandbox_id: String,
    /// Plugin format the sandbox hosts.
    pub format: PluginFormat,
    /// Optional correlation token for matching this request to a response.
    pub correlation_id: Option<CorrelationId>,
    /// The command to execute.
    pub command: SandboxControlCommand,
}

impl SandboxControlRequest {
    /// Creates a [`SandboxControlCommand::Handshake`] request with no correlation ID.
    pub fn handshake(sandbox_id: impl Into<String>, format: PluginFormat) -> Self {
        Self {
            sandbox_id: sandbox_id.into(),
            format,
            correlation_id: None,
            command: SandboxControlCommand::Handshake,
        }
    }
}

/// A response from the sandbox to a host control request.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SandboxControlResponse {
    /// Handshake accepted; carries the negotiated protocol version and capabilities.
    HandshakeAccepted {
        /// Negotiated protocol version.
        protocol_version: u32,
        /// Capabilities the sandbox supports.
        capabilities: PluginSandboxCapabilities,
    },
    /// Confirms that a plugin type was loaded successfully.
    PluginTypeLoaded {
        /// The type that was loaded.
        plugin_type_id: PluginTypeId,
    },
    /// Confirms that a plugin instance was created.
    InstanceCreated {
        /// The instance that was created.
        instance_id: PluginInstanceId,
    },
    /// Confirms that an instance was prepared; includes the active processing epoch.
    InstancePrepared {
        /// The instance that was prepared.
        instance_id: PluginInstanceId,
        /// Processing epoch now active.
        processing_epoch: u64,
    },
    /// Confirms that an instance was activated.
    InstanceActivated {
        /// The instance that was activated.
        instance_id: PluginInstanceId,
        /// Processing epoch now active.
        processing_epoch: u64,
    },
    /// Confirms that an instance was deactivated.
    InstanceDeactivated {
        /// The instance that was deactivated.
        instance_id: PluginInstanceId,
    },
    /// Confirms that an instance was reset; includes the new processing epoch.
    InstanceReset {
        /// The instance that was reset.
        instance_id: PluginInstanceId,
        /// New processing epoch now active.
        processing_epoch: u64,
    },
    /// Confirms that an instance was destroyed.
    InstanceDestroyed {
        /// The instance that was destroyed.
        instance_id: PluginInstanceId,
    },
    /// Confirms that the sandbox accepted the shutdown request.
    ShutdownAccepted,
}
