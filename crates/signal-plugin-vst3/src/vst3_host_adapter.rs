use signal_plugin::{
    PluginDescriptor, PluginFormat, PluginIoLayout, PluginSandboxCapabilities, PluginTypeId,
    SandboxTransport,
};
use std::process;

mod discovery;
mod gui;
mod hosting;
mod introspection;
mod model;
#[cfg(test)]
mod scaffold;

pub use discovery::{Vst3DiscoveryBatch, Vst3DiscoveryDiagnostic, Vst3DiscoveryDiagnosticKind};
pub use gui::{Vst3GuiEvent, Vst3GuiSession};
pub use hosting::{
    current_vst3_platform, Vst3HostedInstance, Vst3HostedPortLayout, Vst3HostingError,
    Vst3ProcessSession,
};
pub use model::*;
#[cfg(test)]
pub(crate) use scaffold::vst3_scaffold_module_metadata_contents;

const VST3_SCAN_HELPER_ARG: &str = "--signal-vst3-scan-helper";

/// Runs the private VST3 scan helper mode when the current process was
/// launched with Signal's helper argument.
///
/// Host applications should call this before initializing UI frameworks or
/// audio state. Normal launches return immediately. Helper launches print a
/// JSON factory snapshot to stdout and terminate the process.
pub fn run_vst3_scan_helper_from_args() {
    let mut args = std::env::args_os().skip(1);
    let Some(first) = args.next() else {
        return;
    };
    if first != VST3_SCAN_HELPER_ARG {
        return;
    }

    let code = introspection::run_vst3_scan_helper(args);
    process::exit(code);
}

/// Runs the VST3 scan helper command body for wrapper binaries.
pub fn vst3_scan_helper_main<I>(args: I) -> i32
where
    I: IntoIterator<Item = std::ffi::OsString>,
{
    introspection::run_vst3_scan_helper(args)
}

/// Host-side adapter for VST3 plugins. Handles metadata discovery,
/// instantiation, and capability negotiation.
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
    /// Returns whether strict sandboxing is the default policy for this adapter.
    pub fn strict_sandbox_default(&self) -> bool {
        self.strict_sandbox_default
    }

    /// Returns `true` if the given plugin format is VST3.
    pub fn supports_format(&self, format: PluginFormat) -> bool {
        matches!(format, PluginFormat::Vst3)
    }

    /// Builds the sandbox capabilities advertised to VST3 plugins for the given maximum block size.
    pub fn advertised_capabilities(&self, max_block_frames: u32) -> PluginSandboxCapabilities {
        PluginSandboxCapabilities {
            transport: SandboxTransport::SharedMemory,
            supports_state: true,
            supports_midi: true,
            max_block_frames,
        }
    }
}
