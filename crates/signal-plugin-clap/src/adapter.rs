use std::{
    collections::HashMap,
    env,
    sync::{Arc, Mutex},
};

use signal_plugin::{
    PluginDescriptor, PluginFormat, PluginIoLayout, PluginSandboxCapabilities, PluginTypeId,
    SandboxTransport,
};

use crate::discovery::discover_clap_plugins_for_roots;

/// Target platform for CLAP discovery and bundle resolution.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ClapHostPlatform {
    /// macOS (`Contents/MacOS`).
    MacOs,
    /// Linux (`Contents/x86_64-linux` or `Contents/aarch64-linux`).
    Linux,
}

/// The build-target CLAP platform.
pub const fn current_clap_platform() -> ClapHostPlatform {
    if cfg!(target_os = "macos") {
        ClapHostPlatform::MacOs
    } else {
        ClapHostPlatform::Linux
    }
}

/// Classification of a CLAP scan root.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ClapScanRootKind {
    /// Per-user CLAP bundle root.
    UserBundleRoot,
    /// System-wide CLAP bundle root.
    SystemBundleRoot,
    /// A root supplied through `CLAP_PATH`.
    EnvironmentPath,
}

/// A filesystem root to scan for CLAP files or bundles.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ClapScanRoot {
    /// Filesystem path of the scan root.
    pub root: String,
    /// Platform this root applies to.
    pub platform: ClapHostPlatform,
    /// Classification of this scan root.
    pub kind: ClapScanRootKind,
}

/// A CLAP host extension that the Signal host advertises to plugins.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ClapHostExtension {
    /// The `clap.audio-ports` extension for audio bus negotiation.
    AudioPorts,
    /// The `clap.note-ports` extension for note port negotiation.
    NotePorts,
    /// The `clap.params` extension for parameter access.
    Params,
    /// The `clap.state` extension for state save/restore.
    State,
    /// The `clap.latency` extension for latency reporting.
    Latency,
    /// The `clap.tail` extension for tail-length reporting.
    Tail,
}

impl ClapHostExtension {
    /// Returns the CLAP extension identifier string for this extension.
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

/// Host-side discovery adapter for CLAP plugins.
///
/// Scans explicitly provided filesystem roots and keeps a catalog of the
/// plugin types found in the last scan. Default scans enumerate factory
/// descriptors only; in-process capability probing (which instantiates
/// plugins) must be requested explicitly via
/// [`ClapPluginHostAdapter::discover_plugins_for_roots_with_options`].
#[derive(Clone, Debug)]
pub struct ClapPluginHostAdapter {
    strict_sandbox_default: bool,
    discovery_catalog: Arc<Mutex<HashMap<String, ClapDiscoveredPluginType>>>,
}

impl Default for ClapPluginHostAdapter {
    fn default() -> Self {
        Self {
            strict_sandbox_default: true,
            discovery_catalog: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

impl ClapPluginHostAdapter {
    /// Returns the default CLAP scan roots for the given platform.
    ///
    /// Entries from `CLAP_PATH` are appended in their environment order after
    /// the platform defaults. Empty path entries are ignored.
    pub fn default_scan_roots(&self, platform: ClapHostPlatform) -> Vec<ClapScanRoot> {
        let clap_path = env::var("CLAP_PATH").ok();
        default_clap_scan_roots(platform, clap_path.as_deref())
    }

    /// Returns whether strict sandboxing is the default policy for this adapter.
    pub fn strict_sandbox_default(&self) -> bool {
        self.strict_sandbox_default
    }

    /// Returns `true` if the given plugin format is CLAP.
    pub fn supports_format(&self, format: PluginFormat) -> bool {
        matches!(format, PluginFormat::Clap)
    }

    /// Returns the minimum set of CLAP host extensions that must be supported.
    pub fn minimum_extension_set(&self) -> &'static [ClapHostExtension] {
        &MINIMUM_CLAP_EXTENSIONS
    }

    /// Builds the sandbox capabilities advertised to CLAP plugins for the given maximum block size.
    pub fn advertised_capabilities(&self, max_block_frames: u32) -> PluginSandboxCapabilities {
        PluginSandboxCapabilities {
            transport: SandboxTransport::SharedMemory,
            supports_state: true,
            supports_midi: true,
            max_block_frames,
        }
    }

    /// Looks up a plugin type discovered in the most recent scan by its type ID string.
    pub fn discover_plugin_type(&self, plugin_type_id: &str) -> Option<ClapDiscoveredPluginType> {
        self.discovery_catalog
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .get(plugin_type_id)
            .cloned()
    }

    /// Scans the given filesystem roots for CLAP plugins using factory
    /// descriptor enumeration only (no plugin instantiation) and updates the
    /// internal catalog. The current build-target platform selects bundle
    /// resolution. An empty root list scans nothing.
    pub fn discover_plugins_for_roots(&self, roots: &[String]) -> Vec<ClapDiscoveredPluginType> {
        self.discover_plugins_for_platform(current_clap_platform(), roots)
    }

    /// Scans explicit filesystem roots using the requested platform's bundle
    /// layout. This is useful to consumers that scan a target platform
    /// explicitly and to cross-platform discovery tests.
    pub fn discover_plugins_for_platform(
        &self,
        platform: ClapHostPlatform,
        roots: &[String],
    ) -> Vec<ClapDiscoveredPluginType> {
        self.discover_plugins_for_platform_with_options(platform, roots, false)
    }

    /// Scans the given filesystem roots for CLAP plugins and updates the
    /// internal catalog.
    ///
    /// When `probe_capabilities` is `true`, each discovered plugin is
    /// instantiated in-process to read its audio/note ports, parameters, and
    /// extension support. Only enable this for trusted fixtures or once a
    /// sandboxed scanner exists — instantiating arbitrary third-party plugins
    /// in the control process is unsafe.
    pub fn discover_plugins_for_roots_with_options(
        &self,
        roots: &[String],
        probe_capabilities: bool,
    ) -> Vec<ClapDiscoveredPluginType> {
        self.discover_plugins_for_platform_with_options(
            current_clap_platform(),
            roots,
            probe_capabilities,
        )
    }

    /// Scans explicit filesystem roots using the requested platform's bundle
    /// layout and optionally probes plugin capabilities in-process.
    pub fn discover_plugins_for_platform_with_options(
        &self,
        platform: ClapHostPlatform,
        roots: &[String],
        probe_capabilities: bool,
    ) -> Vec<ClapDiscoveredPluginType> {
        let discovered = discover_clap_plugins_for_roots(platform, roots, probe_capabilities);
        let mut catalog = self
            .discovery_catalog
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        catalog.clear();
        catalog.extend(
            discovered
                .iter()
                .cloned()
                .map(|plugin| (plugin.plugin_type_id.0.clone(), plugin)),
        );
        discovered
    }
}

fn default_clap_scan_roots(
    platform: ClapHostPlatform,
    clap_path: Option<&str>,
) -> Vec<ClapScanRoot> {
    let mut roots = match platform {
        ClapHostPlatform::MacOs => vec![
            ClapScanRoot {
                root: "~/Library/Audio/Plug-Ins/CLAP".into(),
                platform,
                kind: ClapScanRootKind::UserBundleRoot,
            },
            ClapScanRoot {
                root: "/Library/Audio/Plug-Ins/CLAP".into(),
                platform,
                kind: ClapScanRootKind::SystemBundleRoot,
            },
        ],
        ClapHostPlatform::Linux => vec![
            ClapScanRoot {
                root: "~/.clap".into(),
                platform,
                kind: ClapScanRootKind::UserBundleRoot,
            },
            ClapScanRoot {
                root: "/usr/lib/clap".into(),
                platform,
                kind: ClapScanRootKind::SystemBundleRoot,
            },
            ClapScanRoot {
                root: "/usr/local/lib/clap".into(),
                platform,
                kind: ClapScanRootKind::SystemBundleRoot,
            },
        ],
    };

    roots.extend(
        clap_path
            .into_iter()
            .flat_map(|paths| paths.split(':'))
            .filter(|path| !path.is_empty())
            .map(|root| ClapScanRoot {
                root: root.to_string(),
                platform,
                kind: ClapScanRootKind::EnvironmentPath,
            }),
    );
    roots
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clap_path_entries_append_after_platform_defaults() {
        let roots = default_clap_scan_roots(ClapHostPlatform::Linux, Some("/one:/two::"));

        assert_eq!(
            roots
                .iter()
                .map(|root| root.root.as_str())
                .collect::<Vec<_>>(),
            vec![
                "~/.clap",
                "/usr/lib/clap",
                "/usr/local/lib/clap",
                "/one",
                "/two"
            ]
        );
        assert!(roots[3..]
            .iter()
            .all(|root| root.kind == ClapScanRootKind::EnvironmentPath));
    }
}

/// A CLAP plugin type discovered during a scan, including its descriptor and default I/O layout.
#[derive(Clone, Debug, PartialEq)]
pub struct ClapDiscoveredPluginType {
    /// Unique identifier for this plugin type.
    pub plugin_type_id: PluginTypeId,
    /// Path to the CLAP shared library on disk.
    pub library_path: String,
    /// Full plugin descriptor including contracts and metadata.
    pub descriptor: PluginDescriptor,
    /// Default I/O layout reported by the plugin at scan time. All-zero
    /// unless capability probing was enabled for the scan.
    pub default_io_layout: PluginIoLayout,
}
