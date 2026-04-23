/// Names of messages exchanged over the plugin sandbox protocol.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PluginMessageName {
    /// Initial handshake to establish sandbox identity and protocol version.
    SandboxHandshake,
    /// Load a plugin type into the sandbox.
    SandboxLoadPluginType,
    /// Create a plugin instance.
    SandboxCreateInstance,
    /// Prepare an instance for audio processing (allocates shared memory).
    SandboxPrepareInstance,
    /// Activate an instance so it can process audio.
    SandboxActivateInstance,
    /// Periodic liveness check.
    SandboxHeartbeat,
    /// Deactivate an active instance.
    SandboxDeactivateInstance,
    /// Reset an instance to its initial state.
    SandboxResetInstance,
    /// Destroy an instance and release its resources.
    SandboxDestroyInstance,
    /// Report an unrecoverable sandbox failure.
    SandboxFailure,
}

impl PluginMessageName {
    /// Returns the wire-format string for this message name.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::SandboxHandshake => "sandbox.handshake",
            Self::SandboxLoadPluginType => "sandbox.loadPluginType",
            Self::SandboxCreateInstance => "sandbox.createInstance",
            Self::SandboxPrepareInstance => "sandbox.prepareInstance",
            Self::SandboxActivateInstance => "sandbox.activateInstance",
            Self::SandboxHeartbeat => "sandbox.heartbeat",
            Self::SandboxDeactivateInstance => "sandbox.deactivateInstance",
            Self::SandboxResetInstance => "sandbox.resetInstance",
            Self::SandboxDestroyInstance => "sandbox.destroyInstance",
            Self::SandboxFailure => "sandbox.failure",
        }
    }
}

/// Identifies a plugin type: its ID, vendor, display name, and format string.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PluginDescriptorPayload {
    /// Stable opaque identifier for the plugin type.
    pub plugin_id: String,
    /// Plugin vendor / manufacturer name.
    pub vendor: String,
    /// Human-readable plugin display name.
    pub name: String,
    /// Plugin format identifier (e.g. `"CLAP"`, `"VST3"`).
    pub format: String,
}

/// Processing configuration for an active plugin instance.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PluginProcessConfigurationPayload {
    /// Sample rate in Hz.
    pub sample_rate_hz: u32,
    /// Maximum number of frames per audio callback block.
    pub max_block_frames: u32,
    /// Audio and MIDI I/O channel counts.
    pub io_layout: PluginIoLayoutPayload,
}

/// Describes a fault reported by a plugin instance.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PluginFaultPayload {
    /// Machine-readable fault kind identifier.
    pub kind: String,
    /// Severity of the fault (e.g. `"warning"`, `"fatal"`).
    pub severity: String,
    /// Human-readable fault message.
    pub message: String,
}

/// Snapshot of the state of a plugin instance within the sandbox.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PluginInstanceStatePayload {
    /// Plugin type identifier.
    pub plugin_type_id: String,
    /// Unique instance identifier.
    pub instance_id: String,
    /// Current lifecycle state (e.g. `"active"`, `"deactivated"`).
    pub lifecycle_state: String,
    /// Readiness state (e.g. `"ready"`, `"degraded"`).
    pub readiness_state: String,
    /// Reasons the instance is degraded, if any.
    pub degraded_reasons: Vec<String>,
    /// Whether the instance is currently active and processing.
    pub active: bool,
    /// Processing configuration if the instance is prepared, otherwise `None`.
    pub processing: Option<PluginProcessConfigurationPayload>,
    /// Most recent fault, if any.
    pub last_fault: Option<PluginFaultPayload>,
}

/// Audio and MIDI port counts for a plugin instance.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PluginIoLayoutPayload {
    /// Number of audio input buses.
    pub audio_inputs: u16,
    /// Number of audio output buses.
    pub audio_outputs: u16,
    /// Number of MIDI input ports.
    pub midi_inputs: u16,
    /// Number of MIDI output ports.
    pub midi_outputs: u16,
}

/// Byte range within a shared-memory region used for a single logical buffer.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SharedMemoryRegionPayload {
    /// Byte offset of this buffer from the start of the shared-memory region.
    pub offset_bytes: u32,
    /// Size of this buffer in bytes.
    pub size_bytes: u32,
}

/// Transport mechanism used for a shared-memory region.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SharedMemoryTransportKind {
    /// Memory-mapped file on the local filesystem.
    MappedFile,
}

impl SharedMemoryTransportKind {
    /// Returns the wire-format string for this transport kind.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::MappedFile => "mappedFile",
        }
    }
}

/// Identifies a specific shared-memory region and describes how to map it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SharedMemoryTransportPayload {
    /// Unique identifier for this region.
    pub region_id: String,
    /// How the region is backed (always `MappedFile` today).
    pub transport_kind: SharedMemoryTransportKind,
    /// Filesystem path to the backing file.
    pub backing_path: String,
    /// Total size of the region in bytes.
    pub total_bytes: u32,
}

/// Byte-range layout of all logical buffers within a single shared-memory region.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SharedMemoryLayoutPayload {
    /// Audio input buffer region.
    pub audio_input: SharedMemoryRegionPayload,
    /// Audio output buffer region.
    pub audio_output: SharedMemoryRegionPayload,
    /// Event input buffer region (parameter and MIDI events from host to plugin).
    pub event_input: SharedMemoryRegionPayload,
    /// Event output buffer region (events from plugin back to host).
    pub event_output: SharedMemoryRegionPayload,
    /// Render context region (block metadata written by the host each callback).
    pub render_context: SharedMemoryRegionPayload,
    /// Completion signalling region written by the plugin when processing is done.
    pub completion: SharedMemoryRegionPayload,
}

impl SharedMemoryLayoutPayload {
    /// Computes the total byte count needed to hold all regions end-to-end.
    pub fn total_bytes(self) -> u32 {
        self.completion.offset_bytes + self.completion.size_bytes
    }
}

/// All message payloads used in the plugin sandbox protocol.
///
/// Each variant corresponds to a specific [`PluginMessageName`]. Request and
/// response variants are paired by name; the `SandboxFailure` variant is
/// unidirectional from sandbox to host.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PluginMessagePayload {
    /// Sandbox-to-host handshake request.
    SandboxHandshakeRequest {
        /// Unique ID of the sandbox process.
        sandbox_id: String,
        /// Plugin format this sandbox handles.
        format: String,
    },
    /// Host-to-sandbox handshake response.
    SandboxHandshakeResponse {
        /// Sandbox ID echoed from the request.
        sandbox_id: String,
        /// Protocol version agreed for this session.
        protocol_version: u32,
        /// Whether the sandbox supports preset state save/restore.
        supports_state: bool,
        /// Whether the sandbox supports MIDI processing.
        supports_midi: bool,
        /// Maximum frames per audio callback block.
        max_block_frames: u32,
    },
    /// Host-to-sandbox request to load a plugin type.
    LoadPluginTypeRequest {
        /// Sandbox ID.
        sandbox_id: String,
        /// Plugin type to load.
        plugin_type_id: String,
        /// Plugin descriptor.
        descriptor: PluginDescriptorPayload,
    },
    /// Sandbox-to-host confirmation that a plugin type was loaded.
    LoadPluginTypeResponse {
        /// Plugin type that was loaded.
        plugin_type_id: String,
        /// Plugin descriptor echoed from the request.
        descriptor: PluginDescriptorPayload,
    },
    /// Host-to-sandbox request to create an instance of a loaded plugin type.
    CreateInstanceRequest {
        /// Sandbox ID.
        sandbox_id: String,
        /// Plugin type to instantiate.
        plugin_type_id: String,
        /// Unique ID for the new instance.
        instance_id: String,
    },
    /// Sandbox-to-host confirmation that an instance was created.
    CreateInstanceResponse {
        /// Instance ID.
        instance_id: String,
        /// Initial state of the new instance.
        instance_state: PluginInstanceStatePayload,
    },
    /// Host-to-sandbox request to allocate shared memory and prepare an instance.
    PrepareInstanceRequest {
        /// Sandbox ID.
        sandbox_id: String,
        /// Instance to prepare.
        instance_id: String,
        /// Monotonically increasing epoch for this processing session.
        processing_epoch: u64,
        /// Lease ID used to name the shared-memory backing file.
        shared_memory_lease_id: String,
        /// Transport descriptor for the shared-memory region.
        shared_memory_transport: SharedMemoryTransportPayload,
        /// Sample rate in Hz.
        sample_rate_hz: u32,
        /// Maximum frames per callback block.
        max_block_frames: u32,
        /// Audio and MIDI I/O layout.
        io_layout: PluginIoLayoutPayload,
        /// Buffer layout within the shared-memory region.
        shared_memory: SharedMemoryLayoutPayload,
    },
    /// Sandbox-to-host confirmation that an instance is prepared.
    PrepareInstanceResponse {
        /// Instance ID.
        instance_id: String,
        /// Processing epoch echoed from the request.
        processing_epoch: u64,
        /// Shared-memory lease ID echoed from the request.
        shared_memory_lease_id: String,
        /// Transport descriptor echoed from the request.
        shared_memory_transport: SharedMemoryTransportPayload,
        /// Actual byte count of the shared-memory region.
        shared_memory_bytes: u32,
        /// Updated instance state after preparation.
        instance_state: PluginInstanceStatePayload,
    },
    /// Host-to-sandbox request to activate a prepared instance.
    ActivateInstanceRequest {
        /// Sandbox ID.
        sandbox_id: String,
        /// Instance to activate.
        instance_id: String,
        /// Processing epoch for this activation.
        processing_epoch: u64,
    },
    /// Sandbox-to-host confirmation that an instance was activated.
    ActivateInstanceResponse {
        /// Instance ID.
        instance_id: String,
        /// Processing epoch echoed from the request.
        processing_epoch: u64,
        /// Updated instance state after activation.
        instance_state: PluginInstanceStatePayload,
    },
    /// Host-to-sandbox liveness check, optionally scoped to an instance.
    HeartbeatRequest {
        /// Sandbox ID.
        sandbox_id: String,
        /// Instance ID if checking instance liveness, otherwise `None`.
        instance_id: Option<String>,
        /// Processing epoch for the instance, if applicable.
        processing_epoch: Option<u64>,
    },
    /// Sandbox-to-host heartbeat response.
    HeartbeatResponse {
        /// Sandbox ID.
        sandbox_id: String,
        /// Instance ID echoed from the request.
        instance_id: Option<String>,
        /// Processing epoch echoed from the request.
        processing_epoch: Option<u64>,
        /// Whether the sandbox considers the instance active.
        active: bool,
        /// Current instance state, if an instance ID was specified.
        instance_state: Option<PluginInstanceStatePayload>,
    },
    /// Host-to-sandbox request to deactivate an active instance.
    DeactivateInstanceRequest {
        /// Sandbox ID.
        sandbox_id: String,
        /// Instance to deactivate.
        instance_id: String,
    },
    /// Sandbox-to-host confirmation that an instance was deactivated.
    DeactivateInstanceResponse {
        /// Instance ID.
        instance_id: String,
        /// Updated instance state after deactivation.
        instance_state: PluginInstanceStatePayload,
    },
    /// Host-to-sandbox request to reset an instance to its initial state.
    ResetInstanceRequest {
        /// Sandbox ID.
        sandbox_id: String,
        /// Instance to reset.
        instance_id: String,
        /// Processing epoch at the time of the reset.
        processing_epoch: u64,
    },
    /// Sandbox-to-host confirmation that an instance was reset.
    ResetInstanceResponse {
        /// Instance ID.
        instance_id: String,
        /// Processing epoch echoed from the request.
        processing_epoch: u64,
        /// Updated instance state after reset.
        instance_state: PluginInstanceStatePayload,
    },
    /// Host-to-sandbox request to destroy an instance and release its resources.
    DestroyInstanceRequest {
        /// Sandbox ID.
        sandbox_id: String,
        /// Instance to destroy.
        instance_id: String,
    },
    /// Sandbox-to-host confirmation that an instance was destroyed.
    DestroyInstanceResponse {
        /// Instance ID.
        instance_id: String,
        /// Final instance state at the time of destruction.
        instance_state: PluginInstanceStatePayload,
    },
    /// Sandbox-to-host notification of an unrecoverable failure.
    SandboxFailure {
        /// Sandbox ID.
        sandbox_id: String,
        /// Instance ID involved in the failure, if applicable.
        instance_id: Option<String>,
        /// Lifecycle stage at which the failure occurred.
        stage: String,
        /// Machine-readable error kind.
        error_kind: String,
        /// Human-readable error detail.
        detail: String,
        /// Structured fault record.
        fault: PluginFaultPayload,
        /// Instance state at the time of failure, if available.
        instance_state: Option<PluginInstanceStatePayload>,
        /// Processing epoch at the time of failure, if applicable.
        processing_epoch: Option<u64>,
        /// Shared-memory lease ID if one was active at the time of failure.
        shared_memory_lease_id: Option<String>,
    },
}
