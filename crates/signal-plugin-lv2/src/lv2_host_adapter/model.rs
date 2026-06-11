use super::*;

/// Target platform for LV2 discovery and session planning.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Lv2HostPlatform {
    /// Linux (the primary LV2 platform).
    Linux,
}

/// Classification of an LV2 scan root (user or system bundle directory).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Lv2ScanRootKind {
    /// Per-user `~/.lv2` root.
    UserBundleRoot,
    /// System-wide LV2 bundle root.
    SystemBundleRoot,
}

/// A filesystem root to scan for LV2 bundles.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Lv2ScanRoot {
    /// Filesystem path of the scan root.
    pub root: String,
    /// Platform this root applies to.
    pub platform: Lv2HostPlatform,
    /// Classification of this scan root.
    pub kind: Lv2ScanRootKind,
}

/// An LV2 plugin type discovered during a scan, including its URI, manifest path, required features, and default I/O layout.
#[derive(Clone, Debug, PartialEq)]
pub struct Lv2DiscoveredPluginType {
    /// Unique identifier for this plugin type.
    pub plugin_type_id: PluginTypeId,
    /// LV2 plugin URI (e.g. `http://example.com/MyPlugin`).
    pub plugin_uri: String,
    /// Path to the `.lv2` bundle directory.
    pub bundle_root: String,
    /// Path to the bundle's `manifest.ttl`.
    pub manifest_path: String,
    /// LV2 required features declared in the manifest.
    pub required_features: Vec<String>,
    /// LV2 optional/supported extensions declared in the manifest.
    pub supported_extensions: Vec<String>,
    /// A preparation fault mode triggered by an unavailable required extension, if any.
    pub prepare_fault: Option<Lv2PreparationFaultMode>,
    /// Full plugin descriptor including capabilities and contracts.
    pub descriptor: PluginDescriptor,
    /// Default I/O layout reported at scan time.
    pub default_io_layout: PluginIoLayout,
}

/// A preparation fault discovered for an LV2 plugin due to an unavailable required extension.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Lv2PreparationFaultMode {
    /// The `lv2:WorkerInterface` extension is required but unavailable.
    WorkerUnavailable,
    /// The `lv2:URID` extension is required but unavailable.
    UridUnavailable,
    /// The `lv2:Patch` extension is required but unavailable.
    PatchUnavailable,
}

/// Category of an LV2 discovery diagnostic: malformed manifest or unsupported required feature.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Lv2DiscoveryDiagnosticKind {
    /// The bundle's `manifest.ttl` could not be parsed.
    MalformedManifest,
    /// The bundle declares a required feature that the host does not support.
    UnsupportedRequiredFeature,
}

/// A diagnostic emitted during LV2 discovery for a bundle that could not be fully loaded.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Lv2DiscoveryDiagnostic {
    /// Scan root under which this bundle was found.
    pub root: String,
    /// Path to the `.lv2` bundle directory.
    pub bundle_root: String,
    /// Path to the bundle's `manifest.ttl`, if known.
    pub manifest_path: Option<String>,
    /// Plugin type ID from the manifest, if it could be read.
    pub plugin_type_id: Option<String>,
    /// Kind of issue that prevented loading.
    pub kind: Lv2DiscoveryDiagnosticKind,
    /// Human-readable detail about the issue.
    pub detail: String,
    /// Human-readable diagnostic summary.
    pub summary: String,
}

/// The combined result of an LV2 scan: successfully discovered plugins and any diagnostics for failed bundles.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Lv2DiscoveryBatch {
    /// Plugins that were successfully discovered.
    pub discovered: Vec<Lv2DiscoveredPluginType>,
    /// Diagnostics for bundles that could not be fully loaded.
    pub diagnostics: Vec<Lv2DiscoveryDiagnostic>,
}
