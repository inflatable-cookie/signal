use signal_plugin::{
    BlockDispatch, BlockProcessResult, BlockProcessingHeader, PluginDescriptor, PluginFormat,
    PluginInstanceId, PluginIoLayout, PluginLifecycleContract, PluginProcessingContract,
    PluginSandboxCapabilities, PluginTypeId, SandboxTransport, SharedMemoryLayout,
    SharedMemoryLease,
};

use crate::clap_sandbox_harness;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ClapHostExtension {
    AudioPorts,
    NotePorts,
    Params,
    State,
    Latency,
    Tail,
}

impl ClapHostExtension {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::AudioPorts => "audio-ports",
            Self::NotePorts => "note-ports",
            Self::Params => "params",
            Self::State => "state",
            Self::Latency => "latency",
            Self::Tail => "tail",
        }
    }
}

const MINIMUM_CLAP_EXTENSIONS: [ClapHostExtension; 4] = [
    ClapHostExtension::AudioPorts,
    ClapHostExtension::NotePorts,
    ClapHostExtension::Params,
    ClapHostExtension::State,
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ClapPluginHostAdapter {
    strict_sandbox_default: bool,
}

impl Default for ClapPluginHostAdapter {
    fn default() -> Self {
        Self {
            strict_sandbox_default: true,
        }
    }
}

impl ClapPluginHostAdapter {
    pub fn strict_sandbox_default(&self) -> bool {
        self.strict_sandbox_default
    }

    pub fn supports_format(&self, format: PluginFormat) -> bool {
        matches!(format, PluginFormat::Clap)
    }

    pub fn minimum_extension_set(&self) -> &'static [ClapHostExtension] {
        &MINIMUM_CLAP_EXTENSIONS
    }

    pub fn advertised_capabilities(&self, max_block_frames: u32) -> PluginSandboxCapabilities {
        PluginSandboxCapabilities {
            transport: SandboxTransport::SharedMemory,
            supports_state: true,
            supports_midi: true,
            max_block_frames,
        }
    }

    pub fn discover_plugin_type(&self, plugin_type_id: &str) -> Option<ClapDiscoveredPluginType> {
        clap_sandbox_harness::clap_discovered_plugin_type(plugin_type_id)
    }

    pub fn instantiate_plugin(
        &self,
        discovered: &ClapDiscoveredPluginType,
        instance_id: &str,
    ) -> ClapInstanceControlSurface {
        ClapInstanceControlSurface {
            plugin_type_id: discovered.plugin_type_id.clone(),
            instance_id: PluginInstanceId(instance_id.to_string()),
            descriptor: discovered.descriptor.clone(),
            lifecycle_contract: discovered.descriptor.lifecycle_contract,
            processing_contract: discovered.descriptor.processing_contract,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ClapDiscoveredPluginType {
    pub plugin_type_id: PluginTypeId,
    pub descriptor: PluginDescriptor,
    pub default_io_layout: PluginIoLayout,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ClapInstanceControlSurface {
    pub plugin_type_id: PluginTypeId,
    pub instance_id: PluginInstanceId,
    pub descriptor: PluginDescriptor,
    pub lifecycle_contract: PluginLifecycleContract,
    pub processing_contract: PluginProcessingContract,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ClapSharedMemoryHeader {
    pub plugin_type_id: PluginTypeId,
    pub instance_id: PluginInstanceId,
    pub block: BlockProcessingHeader,
    pub layout: SharedMemoryLayout,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ClapPreparePlan {
    pub plugin_type_id: PluginTypeId,
    pub instance_id: PluginInstanceId,
    pub sample_rate_hz: u32,
    pub max_block_frames: u32,
    pub io_layout: PluginIoLayout,
    pub lease: SharedMemoryLease,
}

#[derive(Clone, Debug, PartialEq)]
pub struct BrokeredBlockOutcome {
    pub dispatch: BlockDispatch,
    pub input: signal_plugin::BlockPayload,
    pub output: signal_plugin::BlockPayload,
    pub result: BlockProcessResult,
}
