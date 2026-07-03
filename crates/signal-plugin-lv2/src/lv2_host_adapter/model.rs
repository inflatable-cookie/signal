use super::*;

/// Target platform for LV2 discovery and session planning.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Lv2HostPlatform {
    /// macOS (`~/Library/Audio/Plug-Ins/LV2`, `/Library/Audio/Plug-Ins/LV2`).
    MacOs,
    /// Linux (the primary LV2 platform).
    Linux,
}

/// The build-target LV2 platform. LV2 hosting is plain dlopen and needs no
/// further cfg gating — only the default bundle roots differ per platform.
pub const fn current_lv2_platform() -> Lv2HostPlatform {
    if cfg!(target_os = "macos") {
        Lv2HostPlatform::MacOs
    } else {
        Lv2HostPlatform::Linux
    }
}

/// Classification of an LV2 scan root (user or system bundle directory).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Lv2ScanRootKind {
    /// Per-user bundle root (`~/.lv2`, `~/Library/Audio/Plug-Ins/LV2`).
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

/// Class of an LV2 port as declared in the bundle TTL.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Lv2PortClasses {
    /// `a lv2:AudioPort`.
    pub audio: bool,
    /// `a lv2:ControlPort`.
    pub control: bool,
    /// `a atom:AtomPort` (LV2 Atom extension).
    pub atom: bool,
    /// `a ev:EventPort` (deprecated Event extension).
    pub event: bool,
    /// `a lv2:InputPort`.
    pub input: bool,
    /// `a lv2:OutputPort`.
    pub output: bool,
}

/// One LV2 port parsed from the bundle TTL, ordered by `lv2:index`.
#[derive(Clone, Debug, PartialEq)]
pub struct Lv2Port {
    /// `lv2:index` — the argument `connect_port` receives.
    pub index: u32,
    /// `lv2:symbol` (stable machine identifier).
    pub symbol: Option<String>,
    /// `lv2:name` (human-readable).
    pub name: Option<String>,
    /// Port classes from the port's `rdf:type`s.
    pub classes: Lv2PortClasses,
    /// `lv2:default` (control ports).
    pub default: Option<f32>,
    /// `lv2:minimum` (control ports).
    pub minimum: Option<f32>,
    /// `lv2:maximum` (control ports).
    pub maximum: Option<f32>,
    /// `lv2:portProperty lv2:connectionOptional` — the host may connect
    /// this port to NULL.
    pub connection_optional: bool,
}

impl Lv2Port {
    /// The effective default for a control port, documented rule:
    /// `lv2:default` when declared; otherwise the midpoint of
    /// `lv2:minimum`/`lv2:maximum` when both are declared; otherwise 0.0
    /// (unbounded ports).
    pub fn effective_default(&self) -> f32 {
        if let Some(default) = self.default {
            return default;
        }
        match (self.minimum, self.maximum) {
            (Some(minimum), Some(maximum)) => (minimum + maximum) / 2.0,
            _ => 0.0,
        }
    }
}

/// An LV2 plugin type discovered during a scan: identity, bundle paths, the
/// TTL port model, and feature requirements.
#[derive(Clone, Debug, PartialEq)]
pub struct Lv2DiscoveredPluginType {
    /// Unique identifier for this plugin type (`plugin:lv2:{uri}`).
    pub plugin_type_id: PluginTypeId,
    /// The canonical LV2 plugin URI (the hosting load key).
    pub plugin_uri: String,
    /// Path to the `.lv2` bundle directory (the hosting library path).
    pub bundle_root: String,
    /// Path to the bundle's `manifest.ttl`.
    pub manifest_path: String,
    /// Resolved path to the plugin's `lv2:binary` shared library.
    pub binary_path: String,
    /// LV2 required features declared in the TTL.
    pub required_features: Vec<String>,
    /// LV2 optional features declared in the TTL.
    pub optional_features: Vec<String>,
    /// The TTL port model, sorted by `lv2:index`.
    pub ports: Vec<Lv2Port>,
    /// A preparation fault mode for an unavailable required extension, if
    /// any. Phase 1 hosts `urid:map` only and pre-filters every other
    /// required feature at scan, so real scans report `None`; the field is
    /// the seam for later extension support.
    pub prepare_fault: Option<Lv2PreparationFaultMode>,
    /// Full plugin descriptor including capabilities and contracts.
    pub descriptor: PluginDescriptor,
    /// Default I/O layout derived from the TTL port model.
    pub default_io_layout: PluginIoLayout,
}

/// A preparation fault discovered for an LV2 plugin due to an unavailable
/// required extension.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Lv2PreparationFaultMode {
    /// The worker extension is required but unavailable.
    WorkerUnavailable,
    /// The URID extension is required but unavailable.
    UridUnavailable,
    /// The patch extension is required but unavailable.
    PatchUnavailable,
}

/// Category of an LV2 discovery diagnostic: malformed manifest or
/// unsupported required feature.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Lv2DiscoveryDiagnosticKind {
    /// The bundle's TTL could not be parsed against the supported Turtle
    /// subset, or its plugin/port model is incomplete.
    MalformedManifest,
    /// The plugin declares a required feature the host does not provide
    /// (phase 1 provides `urid:map` only).
    UnsupportedRequiredFeature,
}

/// A diagnostic emitted during LV2 discovery for a bundle (or one plugin
/// within a multi-plugin bundle) that could not be fully loaded.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Lv2DiscoveryDiagnostic {
    /// Scan root under which this bundle was found.
    pub root: String,
    /// Path to the `.lv2` bundle directory.
    pub bundle_root: String,
    /// Path to the bundle's `manifest.ttl`, if known.
    pub manifest_path: Option<String>,
    /// Plugin type ID (`plugin:lv2:{uri}`), when the plugin URI was
    /// readable before the failure.
    pub plugin_type_id: Option<String>,
    /// Kind of issue that prevented loading.
    pub kind: Lv2DiscoveryDiagnosticKind,
    /// Human-readable detail about the issue.
    pub detail: String,
    /// Human-readable diagnostic summary.
    pub summary: String,
}

/// The combined result of an LV2 scan: successfully discovered plugins and
/// any diagnostics for failed bundles.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Lv2DiscoveryBatch {
    /// Plugins that were successfully discovered.
    pub discovered: Vec<Lv2DiscoveredPluginType>,
    /// Diagnostics for bundles that could not be fully loaded.
    pub diagnostics: Vec<Lv2DiscoveryDiagnostic>,
}
