use super::*;

/// Target platform for AU discovery and session planning.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AuHostPlatform {
    /// macOS (the only supported AU platform).
    MacOs,
}

/// Classification of an AU scan root (user, system, or built-in component directory).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AuScanRootKind {
    /// Per-user `~/Library/Audio/Plug-Ins/Components`.
    UserComponentRoot,
    /// System-wide `/Library/Audio/Plug-Ins/Components`.
    SystemComponentRoot,
    /// Apple built-in `/System/Library/Components`.
    BuiltInComponentRoot,
}

/// A filesystem root to scan for AU component bundles.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AuScanRoot {
    /// Filesystem path of the scan root.
    pub root: String,
    /// Platform this root applies to.
    pub platform: AuHostPlatform,
    /// Classification of this scan root.
    pub kind: AuScanRootKind,
}

/// An AU plugin type discovered during a scan, including its AudioComponent identity, descriptor, and default I/O layout.
#[derive(Clone, Debug, PartialEq)]
pub struct AuDiscoveredPluginType {
    /// Unique identifier for this plugin type.
    pub plugin_type_id: PluginTypeId,
    /// AudioComponent type OSType (e.g. `aufx`).
    pub component_type: String,
    /// AudioComponent subtype OSType (e.g. `Comp`).
    pub component_subtype: String,
    /// AudioComponent manufacturer OSType.
    pub manufacturer_code: String,
    /// Path to the `.component` bundle on disk.
    pub bundle_root: String,
    /// Full plugin descriptor including capabilities and contracts.
    pub descriptor: PluginDescriptor,
    /// Default I/O layout reported at scan time.
    pub default_io_layout: PluginIoLayout,
}

impl AuDiscoveredPluginType {
    /// Format-native load key handed to the hosting backends: the
    /// colon-separated fourcc triple `{type}:{subtype}:{manufacturer}`
    /// (colon-safe on the whitespace-separated broker wire).
    pub fn load_key(&self) -> String {
        format!(
            "{}:{}:{}",
            self.component_type, self.component_subtype, self.manufacturer_code
        )
    }
}
