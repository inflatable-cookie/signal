use super::*;

/// Target platform for VST3 discovery and session planning.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Vst3HostPlatform {
    /// macOS (`Contents/MacOS`).
    MacOs,
    /// Linux (`Contents/x86_64-linux` or `Contents/aarch64-linux`).
    Linux,
    /// Windows (`Contents/x86_64-win` or `Contents/arm64-win`).
    Windows,
}

/// Classification of a VST3 scan root (user or system bundle directory).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Vst3ScanRootKind {
    /// Per-user VST3 bundle root.
    UserBundleRoot,
    /// System-wide VST3 bundle root.
    SystemBundleRoot,
}

/// A filesystem root to scan for VST3 module bundles.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Vst3ScanRoot {
    /// Filesystem path of the scan root.
    pub root: String,
    /// Platform this root applies to.
    pub platform: Vst3HostPlatform,
    /// Classification of this scan root.
    pub kind: Vst3ScanRootKind,
}

/// A VST3 plugin type discovered during a scan, including its class ID, optional controller class, and default I/O layout.
#[derive(Clone, Debug, PartialEq)]
pub struct Vst3DiscoveredPluginType {
    /// Unique identifier for this plugin type.
    pub plugin_type_id: PluginTypeId,
    /// VST3 component class GUID string.
    pub class_id: String,
    /// VST3 controller class GUID string, if present.
    pub controller_class_id: Option<String>,
    /// VST3 sub-category string (e.g. `Instrument`, `Fx`).
    pub category: String,
    /// Path to the `.vst3` bundle directory.
    pub module_root: String,
    /// Full plugin descriptor including capabilities and contracts.
    pub descriptor: PluginDescriptor,
    /// Default I/O layout reported at scan time.
    pub default_io_layout: PluginIoLayout,
}
