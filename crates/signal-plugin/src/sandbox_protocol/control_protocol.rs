use signal_ipc::CorrelationId;

use crate::{
    PluginDescriptor, PluginFormat, PluginInstanceId, PluginIoLayout, PluginTypeId, SharedMemoryLayout,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PluginSandboxRequest {
    pub sandbox_id: String,
    pub format: PluginFormat,
    pub policy: crate::SandboxPolicy,
    pub correlation_id: Option<CorrelationId>,
}

impl PluginSandboxRequest {
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SandboxTransport {
    SharedMemory,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PluginSandboxCapabilities {
    pub transport: SandboxTransport,
    pub supports_state: bool,
    pub supports_midi: bool,
    pub max_block_frames: u32,
}

#[derive(Clone, Debug, PartialEq)]
pub enum SandboxControlCommand {
    Handshake,
    LoadPluginType {
        plugin_type_id: PluginTypeId,
        descriptor: PluginDescriptor,
    },
    CreateInstance {
        plugin_type_id: PluginTypeId,
        instance_id: PluginInstanceId,
    },
    DestroyInstance {
        instance_id: PluginInstanceId,
    },
    PrepareInstance {
        instance_id: PluginInstanceId,
        sample_rate_hz: u32,
        max_block_frames: u32,
        io_layout: PluginIoLayout,
        shared_memory: SharedMemoryLayout,
    },
    ActivateInstance {
        instance_id: PluginInstanceId,
        processing_epoch: u64,
    },
    DeactivateInstance {
        instance_id: PluginInstanceId,
    },
    ResetInstance {
        instance_id: PluginInstanceId,
        processing_epoch: u64,
    },
    Shutdown,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SandboxControlRequest {
    pub sandbox_id: String,
    pub format: PluginFormat,
    pub correlation_id: Option<CorrelationId>,
    pub command: SandboxControlCommand,
}

impl SandboxControlRequest {
    pub fn handshake(sandbox_id: impl Into<String>, format: PluginFormat) -> Self {
        Self {
            sandbox_id: sandbox_id.into(),
            format,
            correlation_id: None,
            command: SandboxControlCommand::Handshake,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SandboxControlResponse {
    HandshakeAccepted {
        protocol_version: u32,
        capabilities: PluginSandboxCapabilities,
    },
    PluginTypeLoaded {
        plugin_type_id: PluginTypeId,
    },
    InstanceCreated {
        instance_id: PluginInstanceId,
    },
    InstancePrepared {
        instance_id: PluginInstanceId,
        processing_epoch: u64,
    },
    InstanceActivated {
        instance_id: PluginInstanceId,
        processing_epoch: u64,
    },
    InstanceDeactivated {
        instance_id: PluginInstanceId,
    },
    InstanceReset {
        instance_id: PluginInstanceId,
        processing_epoch: u64,
    },
    InstanceDestroyed {
        instance_id: PluginInstanceId,
    },
    ShutdownAccepted,
}
