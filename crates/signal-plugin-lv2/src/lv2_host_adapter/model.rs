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

/// Live control surface for an LV2 plugin instance. Binds the discovered type to a running instance ID and its extension posture.
#[derive(Clone, Debug, PartialEq)]
pub struct Lv2InstanceControlSurface {
    /// Plugin type this instance was created from.
    pub plugin_type_id: PluginTypeId,
    /// Unique identifier for this running instance.
    pub instance_id: PluginInstanceId,
    /// LV2 plugin URI.
    pub plugin_uri: String,
    /// Path to the `.lv2` bundle directory.
    pub bundle_root: String,
    /// Path to the bundle's `manifest.ttl`.
    pub manifest_path: String,
    /// LV2 required features declared by the plugin.
    pub required_features: Vec<String>,
    /// Preparation fault mode triggered by a missing required extension, if any.
    pub prepare_fault: Option<Lv2PreparationFaultMode>,
    /// Default I/O layout for this instance.
    pub default_io_layout: PluginIoLayout,
    /// Plugin descriptor including capabilities and contracts.
    pub descriptor: PluginDescriptor,
    /// Lifecycle contract for this instance.
    pub lifecycle_contract: PluginLifecycleContract,
    /// Processing contract for this instance.
    pub processing_contract: PluginProcessingContract,
}

/// Whether the LV2 worker extension is absent, available, or required and available.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Lv2WorkerNegotiationPosture {
    /// The worker extension is not present.
    WorkerAbsent,
    /// The worker extension is available but not required.
    WorkerAvailable,
    /// The worker extension is required and available.
    WorkerRequiredAvailable,
}

/// Whether the LV2 URID mapping extension was required and negotiated.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Lv2UridNegotiationPosture {
    /// The URID extension was not required.
    NotRequired,
    /// The URID extension was negotiated.
    Negotiated,
}

/// Whether the LV2 patch message extension is supported by the plugin.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Lv2PatchExchangePosture {
    /// The patch extension is not supported.
    Absent,
    /// The patch extension is supported.
    Supported,
}

/// Whether a specific LV2 extension was not required or has been successfully negotiated.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Lv2ExtensionNegotiationState {
    /// No extensions required negotiation.
    NotRequired,
    /// All required extensions were successfully negotiated.
    Negotiated,
}

/// The negotiated extension posture for an LV2 instance after preparation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Lv2ExtensionPreparationRecord {
    /// Worker extension negotiation posture.
    pub worker_posture: Lv2WorkerNegotiationPosture,
    /// URID mapping negotiation posture.
    pub urid_negotiation_posture: Lv2UridNegotiationPosture,
    /// Patch exchange support posture.
    pub patch_exchange_posture: Lv2PatchExchangePosture,
    /// Overall extension negotiation state.
    pub extension_negotiation_state: Lv2ExtensionNegotiationState,
    /// Human-readable summary of the negotiated posture.
    pub summary: String,
}

/// Processing session plan for an LV2 instance, capturing the agreed sample rate, block size, extension posture, and transport.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Lv2ProcessSessionPlan {
    /// Plugin type being prepared.
    pub plugin_type_id: PluginTypeId,
    /// Instance being prepared.
    pub instance_id: PluginInstanceId,
    /// LV2 plugin URI.
    pub plugin_uri: String,
    /// Agreed sample rate in Hz.
    pub sample_rate_hz: u32,
    /// Maximum block size in frames.
    pub max_block_frames: u32,
    /// I/O layout for this session.
    pub io_layout: PluginIoLayout,
    /// Sandbox transport type for this session.
    pub transport: SandboxTransport,
    /// Path to the `.lv2` bundle directory.
    pub bundle_root: String,
    /// Path to the bundle's `manifest.ttl`.
    pub manifest_path: String,
    /// Negotiated extension posture for this session.
    pub extension_preparation: Lv2ExtensionPreparationRecord,
    /// Human-readable session summary.
    pub summary: String,
}

/// Record produced when an LV2 instance is torn down.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Lv2TeardownRecord {
    /// Human-readable teardown summary.
    pub summary: String,
}

/// Record produced after executing an audio block through an LV2 instance.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Lv2BlockProcessingRecord {
    /// Block sequence number.
    pub block_sequence: u64,
    /// Number of audio frames in this block.
    pub block_frames: u32,
    /// Number of audio output channels written.
    pub audio_output_channels: u16,
    /// Number of MIDI events in this block.
    pub midi_event_count: u16,
    /// Number of patch messages in this block.
    pub patch_message_count: u16,
    /// Completion status string.
    pub completion_status: String,
    /// Human-readable block processing summary.
    pub summary: String,
}
