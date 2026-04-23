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
    /// Optional failure contract for test error injection.
    pub failure_contract: AuFailureContract,
}

/// Live control surface for an AU plugin instance. Binds the discovered component identity to a running instance ID and its contracts.
#[derive(Clone, Debug, PartialEq)]
pub struct AuInstanceControlSurface {
    /// Plugin type this instance was created from.
    pub plugin_type_id: PluginTypeId,
    /// Unique identifier for this running instance.
    pub instance_id: PluginInstanceId,
    /// AudioComponent type OSType.
    pub component_type: String,
    /// AudioComponent subtype OSType.
    pub component_subtype: String,
    /// AudioComponent manufacturer OSType.
    pub manufacturer_code: String,
    /// Path to the `.component` bundle.
    pub bundle_root: String,
    /// Default I/O layout for this instance.
    pub default_io_layout: PluginIoLayout,
    /// Plugin descriptor including capabilities and contracts.
    pub descriptor: PluginDescriptor,
    /// Lifecycle contract for this instance.
    pub lifecycle_contract: PluginLifecycleContract,
    /// Processing contract for this instance.
    pub processing_contract: PluginProcessingContract,
    /// Failure contract for test error injection.
    pub failure_contract: AuFailureContract,
}

/// Processing session plan for an AU instance, capturing the agreed sample rate, block size, and I/O layout.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AuProcessSessionPlan {
    /// Plugin type being prepared.
    pub plugin_type_id: PluginTypeId,
    /// Instance being prepared.
    pub instance_id: PluginInstanceId,
    /// AudioComponent type OSType.
    pub component_type: String,
    /// AudioComponent subtype OSType.
    pub component_subtype: String,
    /// AudioComponent manufacturer OSType.
    pub manufacturer_code: String,
    /// Agreed sample rate in Hz.
    pub sample_rate_hz: u32,
    /// Maximum block size in frames.
    pub max_block_frames: u32,
    /// I/O layout for this session.
    pub io_layout: PluginIoLayout,
    /// Sandbox transport type for this session.
    pub transport: SandboxTransport,
    /// Path to the `.component` bundle.
    pub bundle_root: String,
    /// Human-readable session summary string.
    pub summary: String,
}

/// A captured state snapshot for an AU plugin instance, including raw bytes and a content digest.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AuStateSnapshot {
    /// Plugin type this snapshot was captured from.
    pub plugin_type_id: PluginTypeId,
    /// Instance this snapshot was captured from.
    pub instance_id: PluginInstanceId,
    /// AudioComponent type OSType.
    pub component_type: String,
    /// AudioComponent subtype OSType.
    pub component_subtype: String,
    /// AudioComponent manufacturer OSType.
    pub manufacturer_code: String,
    /// Raw serialised state bytes.
    pub bytes: Vec<u8>,
    /// Content digest of the state bytes.
    pub digest: String,
    /// Human-readable snapshot summary.
    pub summary: String,
}

/// Record produced when an AU instance is activated, confirming the sample rate and block size in use.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AuActivationRecord {
    /// Plugin type that was activated.
    pub plugin_type_id: PluginTypeId,
    /// Instance that was activated.
    pub instance_id: PluginInstanceId,
    /// AudioComponent type OSType.
    pub component_type: String,
    /// AudioComponent subtype OSType.
    pub component_subtype: String,
    /// AudioComponent manufacturer OSType.
    pub manufacturer_code: String,
    /// Sample rate in use after activation.
    pub sample_rate_hz: u32,
    /// Maximum block size in frames after activation.
    pub max_block_frames: u32,
    /// Human-readable activation summary.
    pub summary: String,
}

/// Record produced when an AU instance is torn down, reporting how many state bytes were flushed.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AuTeardownRecord {
    /// Plugin type that was torn down.
    pub plugin_type_id: PluginTypeId,
    /// Instance that was torn down.
    pub instance_id: PluginInstanceId,
    /// AudioComponent type OSType.
    pub component_type: String,
    /// AudioComponent subtype OSType.
    pub component_subtype: String,
    /// AudioComponent manufacturer OSType.
    pub manufacturer_code: String,
    /// Number of state bytes flushed during teardown.
    pub flushed_state_bytes: usize,
    /// Human-readable teardown summary.
    pub summary: String,
}

/// Optional fault modes declared in an AU bundle's metadata, used to exercise bounded error paths during testing.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct AuFailureContract {
    /// If set, causes `instantiate_plugin` to return this error string.
    pub init_failure: Option<String>,
    /// If set, causes `prepare_session` to return this error string.
    pub bus_layout_failure: Option<String>,
    /// If set, causes `activate_instance` to return this error string.
    pub render_context_failure: Option<String>,
}
