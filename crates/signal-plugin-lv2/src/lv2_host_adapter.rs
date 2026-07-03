use signal_plugin::{
    PluginDescriptor, PluginFormat, PluginIoLayout, PluginSandboxCapabilities, PluginTypeId,
    SandboxTransport,
};

mod discovery;
pub mod hosting;
mod introspection;
mod model;
pub mod turtle;

pub use hosting::{Lv2HostedInstance, Lv2HostedPortLayout, Lv2HostingError, Lv2ProcessSession};
pub use introspection::{parse_lv2_bundle, Lv2BundleModel, Lv2PluginModel};
pub use model::*;

/// Host-side adapter for LV2 plugins: `.lv2` bundle scanning over real
/// Turtle manifests (discovery) plus the in-child hosting surfaces
/// ([`Lv2HostedInstance`] / [`Lv2ProcessSession`]).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Lv2HostAdapter {
    strict_sandbox_default: bool,
}

impl Default for Lv2HostAdapter {
    fn default() -> Self {
        Self {
            strict_sandbox_default: true,
        }
    }
}

impl Lv2HostAdapter {
    /// Returns whether strict sandboxing is the default policy for this adapter.
    pub fn strict_sandbox_default(&self) -> bool {
        self.strict_sandbox_default
    }

    /// Returns `true` if the given plugin format is LV2.
    pub fn supports_format(&self, format: PluginFormat) -> bool {
        matches!(format, PluginFormat::Lv2)
    }

    /// Builds the sandbox capabilities advertised to LV2 plugins for the given maximum block size.
    pub fn advertised_capabilities(&self, max_block_frames: u32) -> PluginSandboxCapabilities {
        PluginSandboxCapabilities {
            transport: SandboxTransport::SharedMemory,
            supports_state: true,
            supports_midi: true,
            max_block_frames,
        }
    }
}
