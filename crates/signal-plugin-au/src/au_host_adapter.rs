use signal_plugin::{
    PluginDescriptor, PluginFormat, PluginIoLayout, PluginSandboxCapabilities, PluginTypeId,
    SandboxTransport,
};

mod discovery;
mod introspection;
mod model;
#[cfg(test)]
mod scaffold;

#[cfg(test)]
pub(crate) use scaffold::au_scaffold_component_metadata_contents;

pub use model::*;

/// Host-side adapter for Audio Units plugins. Handles discovery, instantiation, session planning, and capability negotiation on macOS.
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
    /// Returns whether strict sandboxing is the default policy for this adapter.
    pub fn strict_sandbox_default(&self) -> bool {
        self.strict_sandbox_default
    }

    /// Returns `true` if the given plugin format is AU.
    pub fn supports_format(&self, format: PluginFormat) -> bool {
        matches!(format, PluginFormat::Au)
    }

    /// Builds the sandbox capabilities advertised to AU plugins for the given maximum block size.
    pub fn advertised_capabilities(&self, max_block_frames: u32) -> PluginSandboxCapabilities {
        PluginSandboxCapabilities {
            transport: SandboxTransport::SharedMemory,
            supports_state: true,
            supports_midi: true,
            max_block_frames,
        }
    }
}
