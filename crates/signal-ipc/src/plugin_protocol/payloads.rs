#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PluginMessageName {
    SandboxHandshake,
    SandboxLoadPluginType,
    SandboxCreateInstance,
    SandboxPrepareInstance,
    SandboxActivateInstance,
    SandboxHeartbeat,
    SandboxDeactivateInstance,
    SandboxResetInstance,
    SandboxDestroyInstance,
    SandboxFailure,
}

impl PluginMessageName {
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

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PluginDescriptorPayload {
    pub plugin_id: String,
    pub vendor: String,
    pub name: String,
    pub format: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PluginProcessConfigurationPayload {
    pub sample_rate_hz: u32,
    pub max_block_frames: u32,
    pub io_layout: PluginIoLayoutPayload,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PluginFaultPayload {
    pub kind: String,
    pub severity: String,
    pub message: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PluginInstanceStatePayload {
    pub plugin_type_id: String,
    pub instance_id: String,
    pub lifecycle_state: String,
    pub readiness_state: String,
    pub degraded_reasons: Vec<String>,
    pub active: bool,
    pub processing: Option<PluginProcessConfigurationPayload>,
    pub last_fault: Option<PluginFaultPayload>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PluginIoLayoutPayload {
    pub audio_inputs: u16,
    pub audio_outputs: u16,
    pub midi_inputs: u16,
    pub midi_outputs: u16,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SharedMemoryRegionPayload {
    pub offset_bytes: u32,
    pub size_bytes: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SharedMemoryTransportKind {
    MappedFile,
}

impl SharedMemoryTransportKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::MappedFile => "mappedFile",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SharedMemoryTransportPayload {
    pub region_id: String,
    pub transport_kind: SharedMemoryTransportKind,
    pub backing_path: String,
    pub total_bytes: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SharedMemoryLayoutPayload {
    pub audio_input: SharedMemoryRegionPayload,
    pub audio_output: SharedMemoryRegionPayload,
    pub event_input: SharedMemoryRegionPayload,
    pub event_output: SharedMemoryRegionPayload,
    pub render_context: SharedMemoryRegionPayload,
    pub completion: SharedMemoryRegionPayload,
}

impl SharedMemoryLayoutPayload {
    pub fn total_bytes(self) -> u32 {
        self.completion.offset_bytes + self.completion.size_bytes
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PluginMessagePayload {
    SandboxHandshakeRequest {
        sandbox_id: String,
        format: String,
    },
    SandboxHandshakeResponse {
        sandbox_id: String,
        protocol_version: u32,
        supports_state: bool,
        supports_midi: bool,
        max_block_frames: u32,
    },
    LoadPluginTypeRequest {
        sandbox_id: String,
        plugin_type_id: String,
        descriptor: PluginDescriptorPayload,
    },
    LoadPluginTypeResponse {
        plugin_type_id: String,
        descriptor: PluginDescriptorPayload,
    },
    CreateInstanceRequest {
        sandbox_id: String,
        plugin_type_id: String,
        instance_id: String,
    },
    CreateInstanceResponse {
        instance_id: String,
        instance_state: PluginInstanceStatePayload,
    },
    PrepareInstanceRequest {
        sandbox_id: String,
        instance_id: String,
        processing_epoch: u64,
        shared_memory_lease_id: String,
        shared_memory_transport: SharedMemoryTransportPayload,
        sample_rate_hz: u32,
        max_block_frames: u32,
        io_layout: PluginIoLayoutPayload,
        shared_memory: SharedMemoryLayoutPayload,
    },
    PrepareInstanceResponse {
        instance_id: String,
        processing_epoch: u64,
        shared_memory_lease_id: String,
        shared_memory_transport: SharedMemoryTransportPayload,
        shared_memory_bytes: u32,
        instance_state: PluginInstanceStatePayload,
    },
    ActivateInstanceRequest {
        sandbox_id: String,
        instance_id: String,
        processing_epoch: u64,
    },
    ActivateInstanceResponse {
        instance_id: String,
        processing_epoch: u64,
        instance_state: PluginInstanceStatePayload,
    },
    HeartbeatRequest {
        sandbox_id: String,
        instance_id: Option<String>,
        processing_epoch: Option<u64>,
    },
    HeartbeatResponse {
        sandbox_id: String,
        instance_id: Option<String>,
        processing_epoch: Option<u64>,
        active: bool,
        instance_state: Option<PluginInstanceStatePayload>,
    },
    DeactivateInstanceRequest {
        sandbox_id: String,
        instance_id: String,
    },
    DeactivateInstanceResponse {
        instance_id: String,
        instance_state: PluginInstanceStatePayload,
    },
    ResetInstanceRequest {
        sandbox_id: String,
        instance_id: String,
        processing_epoch: u64,
    },
    ResetInstanceResponse {
        instance_id: String,
        processing_epoch: u64,
        instance_state: PluginInstanceStatePayload,
    },
    DestroyInstanceRequest {
        sandbox_id: String,
        instance_id: String,
    },
    DestroyInstanceResponse {
        instance_id: String,
        instance_state: PluginInstanceStatePayload,
    },
    SandboxFailure {
        sandbox_id: String,
        instance_id: Option<String>,
        stage: String,
        error_kind: String,
        detail: String,
        fault: PluginFaultPayload,
        instance_state: Option<PluginInstanceStatePayload>,
        processing_epoch: Option<u64>,
        shared_memory_lease_id: Option<String>,
    },
}
