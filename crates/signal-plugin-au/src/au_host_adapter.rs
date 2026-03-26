use signal_plugin::{
    PluginDescriptor, PluginFormat, PluginInstanceId, PluginIoLayout, PluginLifecycleContract,
    PluginProcessingContract, PluginSandboxCapabilities, PluginTypeId, SandboxTransport,
};

use crate::fixtures::{au_discovered_plugin_type, au_fixture_bundle_name};

mod discovery;
mod model;
mod session;

pub use model::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AuHostAdapter {
    strict_sandbox_default: bool,
}

impl Default for AuHostAdapter {
    fn default() -> Self {
        Self {
            strict_sandbox_default: true,
        }
    }
}

impl AuHostAdapter {
    pub fn strict_sandbox_default(&self) -> bool {
        self.strict_sandbox_default
    }

    pub fn supports_format(&self, format: PluginFormat) -> bool {
        matches!(format, PluginFormat::Au)
    }

    pub fn advertised_capabilities(&self, max_block_frames: u32) -> PluginSandboxCapabilities {
        PluginSandboxCapabilities {
            transport: SandboxTransport::SharedMemory,
            supports_state: true,
            supports_midi: true,
            max_block_frames,
        }
    }
}
