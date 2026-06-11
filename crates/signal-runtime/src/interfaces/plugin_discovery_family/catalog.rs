use super::*;

/// Opaque handle for an event sink subscription.  Returned by `subscribe()`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SubscriptionHandle(pub u64);

/// Request to scan one or more directory roots for plugins of specific formats.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PluginScanRequest {
    /// Directory roots to search for plugin bundles.
    pub roots: Vec<String>,
    /// Plugin formats to include in the scan.
    pub formats: Vec<PluginFormat>,
}

/// Opaque handle returned by `start_plugin_scan()`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ScanHandle(pub u64);

/// Result of a completed plugin scan.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimePluginScanReceipt {
    /// Handle identifying this scan.
    pub scan_handle: ScanHandle,
    /// Directory roots that were scanned.
    pub roots: Vec<String>,
    /// Plugin formats that were targeted.
    pub formats: Vec<PluginFormat>,
    /// Number of distinct plugin formats targeted.
    pub targeted_format_count: usize,
    /// Total number of plugin types discovered.
    pub discovered_type_count: usize,
    /// Number of distinct plugin formats with at least one discovered type.
    pub discovered_format_count: usize,
    /// Number of diagnostic issues encountered during the scan.
    pub discovery_diagnostic_count: usize,
    /// Individual diagnostic records for each scan issue.
    pub discovery_diagnostics: Vec<RuntimePluginScanDiagnosticRecord>,
    /// Per-format type and capability coverage records.
    pub format_coverage: Vec<RuntimePluginFormatCoverageRecord>,
    /// Per-format platform parity and sandbox lifecycle records.
    pub parity_coverage: Vec<RuntimePluginFormatParityRecord>,
    /// Aggregate capability coverage across all discovered types.
    pub capability_coverage: RuntimePluginCapabilityCoverageSummary,
}

/// Category of diagnostic issue found during a plugin scan.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimePluginScanDiagnosticKind {
    /// Plugin manifest is malformed or cannot be parsed.
    MalformedManifest,
    /// Plugin requires a feature that is not supported by the host.
    UnsupportedRequiredFeature,
}

/// A single diagnostic record from a plugin scan.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimePluginScanDiagnosticRecord {
    /// Plugin format associated with this diagnostic.
    pub format: PluginFormat,
    /// Scan root directory where the issue was found.
    pub root: String,
    /// Bundle root path containing the problematic plugin.
    pub bundle_root: String,
    /// Path to the manifest file, if available.
    pub manifest_path: Option<String>,
    /// Plugin type URI, if resolvable.
    pub plugin_type_id: Option<String>,
    /// Category of issue.
    pub kind: RuntimePluginScanDiagnosticKind,
}

/// Full descriptor for one discovered plugin type.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimePluginDiscoveredTypeRecord {
    /// Stable unique identifier for this plugin type (typically the plugin URI).
    pub plugin_type_id: String,
    /// Shorter plugin identifier (name-derived or format-specific).
    pub plugin_id: String,
    /// Vendor/manufacturer name.
    pub vendor: String,
    /// Display name of the plugin.
    pub name: String,
    /// Plugin format (LV2, CLAP, etc.).
    pub format: PluginFormat,
    /// Plugin version string, if declared.
    pub version: Option<String>,
    /// Declared plugin feature tags.
    pub features: Vec<PluginFeature>,
    /// Default I/O layout declared by the plugin.
    pub default_io_layout: PluginIoLayout,
    /// Default multichannel I/O summary derived from the default layout.
    pub default_multichannel_io: RuntimeMultichannelIoSummary,
    /// Complex I/O topology summary.
    pub complex_io_summary: RuntimePluginComplexIoSummary,
    /// Number of audio buses declared.
    pub audio_bus_count: usize,
    /// Number of parameters declared.
    pub parameter_count: usize,
    /// Plugin state contract (snapshot support, etc.).
    pub state_contract: PluginStateContract,
    /// Plugin processing contract (realtime guarantees, etc.).
    pub processing_contract: PluginProcessingContract,
    /// Plugin lifecycle contract (prepare/activate support, etc.).
    pub lifecycle_contract: PluginLifecycleContract,
    /// LV2-specific extension capability summary, if applicable.
    pub lv2_extension_capabilities: Option<RuntimeLv2ExtensionCapabilitySummary>,
}

/// Per-format type and capability counts from a scan.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimePluginFormatCoverageRecord {
    /// Plugin format this record covers.
    pub format: PluginFormat,
    /// Total plugin types discovered for this format.
    pub discovered_type_count: usize,
    /// Number of types with complex I/O topology.
    pub complex_io_type_count: usize,
    /// Number of instruments with multiple output buses.
    pub multi_output_instrument_count: usize,
    /// Number of FX types with bus routing capability.
    pub bus_capable_fx_count: usize,
    /// Number of FX types with sidechain input capability.
    pub sidechain_capable_fx_count: usize,
    /// Number of instrument types.
    pub instrument_count: usize,
    /// Number of audio effect types.
    pub audio_effect_count: usize,
    /// Number of analyzer types.
    pub analyzer_count: usize,
    /// Number of utility types.
    pub utility_count: usize,
    /// Number of note effect types.
    pub note_effect_count: usize,
    /// Number of types supporting snapshot state.
    pub supports_snapshot_count: usize,
    /// Number of types supporting the prepare lifecycle step.
    pub supports_prepare_count: usize,
    /// Number of types supporting the activate lifecycle step.
    pub supports_activate_count: usize,
    /// Number of types that accept MIDI input.
    pub accepts_midi_count: usize,
    /// Number of types that support note expression.
    pub supports_note_expression_count: usize,
    /// Number of types that produce MIDI output.
    pub produces_midi_count: usize,
    /// Maximum complex I/O port group count across all types in this format.
    pub max_complex_io_port_group_count: usize,
    /// Maximum audio bus count across all types in this format.
    pub max_audio_bus_count: usize,
    /// Maximum parameter count across all types in this format.
    pub max_parameter_count: usize,
}

/// Host platform supported by a plugin format.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum RuntimePluginHostPlatform {
    /// macOS host platform.
    MacOs,
    /// Linux host platform.
    Linux,
    /// Windows host platform.
    Windows,
}

/// Platform parity classification for a plugin format.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimePluginParityBand {
    /// Format runs natively on all supported platforms without workarounds.
    Portable,
    /// Format is available but requires platform-specific guards or fallbacks.
    Guarded,
    /// Format is not supported on this platform.
    Unsupported,
}

/// Per-format platform support and Linux parity detail.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimePluginFormatPlatformCoverageRecord {
    /// Plugin format this record covers.
    pub format: PluginFormat,
    /// Platforms on which this format is supported.
    pub supported_platforms: Vec<RuntimePluginHostPlatform>,
    /// Platforms on which this format is not supported.
    pub unsupported_platforms: Vec<RuntimePluginHostPlatform>,
    /// Linux-specific parity classification.
    pub linux_parity_band: RuntimePluginParityBand,
    /// Preferred sandbox isolation outcome on Linux, if applicable.
    pub linux_preferred_sandbox_outcome: Option<RuntimePluginIsolationOutcome>,
    /// Whether strict sandbox mode is the default on Linux.
    pub linux_strict_sandbox_default: bool,
}

/// Combined parity + sandbox lifecycle counts for one plugin format.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimePluginFormatParityRecord {
    /// Plugin format this record covers.
    pub format: PluginFormat,
    /// Overall cross-platform parity classification.
    pub parity_band: RuntimePluginParityBand,
    /// Linux-specific parity classification.
    pub linux_parity_band: RuntimePluginParityBand,
    /// Platforms on which this format is supported.
    pub supported_platforms: Vec<RuntimePluginHostPlatform>,
    /// Platforms on which this format is not supported.
    pub unsupported_platforms: Vec<RuntimePluginHostPlatform>,
    /// Whether this format is supported on Linux.
    pub linux_supported: bool,
    /// Preferred sandbox isolation outcome on Linux, if applicable.
    pub linux_preferred_sandbox_outcome: Option<RuntimePluginIsolationOutcome>,
    /// Whether strict sandbox mode is the default on Linux.
    pub linux_strict_sandbox_default: bool,
    /// Total plugin types discovered for this format.
    pub discovered_type_count: usize,
    /// Number of types that support the prepare lifecycle step.
    pub prepare_capable_type_count: usize,
    /// Number of types that support the activate lifecycle step.
    pub activate_capable_type_count: usize,
    /// Total sandbox instances across all types in this format.
    pub sandbox_count: usize,
    /// Number of sandboxes running in-process.
    pub in_process_sandbox_count: usize,
    /// Number of sandboxes in shared-process isolation.
    pub shared_sandbox_count: usize,
    /// Number of sandboxes in isolated-process mode.
    pub isolated_sandbox_count: usize,
    /// Number of sandboxes in the ready state.
    pub ready_sandbox_count: usize,
    /// Number of sandboxes currently restarting.
    pub restarting_sandbox_count: usize,
    /// Number of sandboxes eligible for rebinding.
    pub rebindable_sandbox_count: usize,
    /// Number of sandboxes in a degraded state.
    pub degraded_sandbox_count: usize,
    /// Number of sandboxes in a faulted state.
    pub faulted_sandbox_count: usize,
    /// Number of sandboxes in a quarantined state.
    pub quarantined_sandbox_count: usize,
    /// Number of sandboxes in a terminal (unrecoverable) state.
    pub terminal_sandbox_count: usize,
    /// Number of active transport sessions across all sandboxes.
    pub active_transport_count: usize,
    /// Number of explicit placement rules configured for this format.
    pub explicit_placement_rule_count: usize,
}
