use signal_plugin::{
    PluginDescriptor, PluginFormat, PluginInstanceId, PluginIoLayout, PluginLifecycleContract,
    PluginProcessingContract, PluginSandboxCapabilities, PluginTypeId, SandboxTransport,
};

mod discovery;
mod introspection;
mod model;
#[cfg(test)]
mod scaffold;
mod session;

pub use model::*;
#[cfg(test)]
pub(crate) use scaffold::vst3_scaffold_module_metadata_contents;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Vst3HostAdapter {
    strict_sandbox_default: bool,
}

impl Default for Vst3HostAdapter {
    fn default() -> Self {
        Self {
            strict_sandbox_default: true,
        }
    }
}

impl Vst3HostAdapter {
    pub fn strict_sandbox_default(&self) -> bool {
        self.strict_sandbox_default
    }

    pub fn supports_format(&self, format: PluginFormat) -> bool {
        matches!(format, PluginFormat::Vst3)
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
