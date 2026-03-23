use super::*;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SubscriptionHandle(pub u64);

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PluginScanRequest {
    pub roots: Vec<String>,
    pub formats: Vec<PluginFormat>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ScanHandle(pub u64);

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimePluginScanReceipt {
    pub scan_handle: ScanHandle,
    pub roots: Vec<String>,
    pub formats: Vec<PluginFormat>,
    pub targeted_format_count: usize,
    pub discovered_type_count: usize,
    pub discovered_format_count: usize,
    pub format_coverage: Vec<RuntimePluginFormatCoverageRecord>,
    pub parity_coverage: Vec<RuntimePluginFormatParityRecord>,
    pub capability_coverage: RuntimePluginCapabilityCoverageSummary,
    pub summary: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimePluginDiscoveredTypeRecord {
    pub plugin_type_id: String,
    pub plugin_id: String,
    pub vendor: String,
    pub name: String,
    pub format: PluginFormat,
    pub version: Option<String>,
    pub features: Vec<PluginFeature>,
    pub default_io_layout: PluginIoLayout,
    pub default_multichannel_io: RuntimeMultichannelIoSummary,
    pub complex_io_summary: RuntimePluginComplexIoSummary,
    pub audio_bus_count: usize,
    pub parameter_count: usize,
    pub state_contract: PluginStateContract,
    pub processing_contract: PluginProcessingContract,
    pub lifecycle_contract: PluginLifecycleContract,
    pub lv2_extension_capabilities: Option<RuntimeLv2ExtensionCapabilitySummary>,
    pub summary: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimePluginFormatCoverageRecord {
    pub format: PluginFormat,
    pub discovered_type_count: usize,
    pub complex_io_type_count: usize,
    pub multi_output_instrument_count: usize,
    pub bus_capable_fx_count: usize,
    pub sidechain_capable_fx_count: usize,
    pub instrument_count: usize,
    pub audio_effect_count: usize,
    pub analyzer_count: usize,
    pub utility_count: usize,
    pub note_effect_count: usize,
    pub supports_snapshot_count: usize,
    pub supports_prepare_count: usize,
    pub supports_activate_count: usize,
    pub accepts_midi_count: usize,
    pub supports_note_expression_count: usize,
    pub produces_midi_count: usize,
    pub max_complex_io_port_group_count: usize,
    pub max_audio_bus_count: usize,
    pub max_parameter_count: usize,
    pub summary: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum RuntimePluginHostPlatform {
    MacOs,
    Linux,
    Windows,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimePluginParityBand {
    Portable,
    Guarded,
    Unsupported,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimePluginFormatPlatformCoverageRecord {
    pub format: PluginFormat,
    pub supported_platforms: Vec<RuntimePluginHostPlatform>,
    pub unsupported_platforms: Vec<RuntimePluginHostPlatform>,
    pub linux_parity_band: RuntimePluginParityBand,
    pub linux_preferred_sandbox_outcome: Option<RuntimePluginIsolationOutcome>,
    pub linux_strict_sandbox_default: bool,
    pub summary: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimePluginFormatParityRecord {
    pub format: PluginFormat,
    pub parity_band: RuntimePluginParityBand,
    pub linux_parity_band: RuntimePluginParityBand,
    pub supported_platforms: Vec<RuntimePluginHostPlatform>,
    pub unsupported_platforms: Vec<RuntimePluginHostPlatform>,
    pub linux_supported: bool,
    pub linux_preferred_sandbox_outcome: Option<RuntimePluginIsolationOutcome>,
    pub linux_strict_sandbox_default: bool,
    pub discovered_type_count: usize,
    pub prepare_capable_type_count: usize,
    pub activate_capable_type_count: usize,
    pub sandbox_count: usize,
    pub in_process_sandbox_count: usize,
    pub shared_sandbox_count: usize,
    pub isolated_sandbox_count: usize,
    pub ready_sandbox_count: usize,
    pub restarting_sandbox_count: usize,
    pub rebindable_sandbox_count: usize,
    pub degraded_sandbox_count: usize,
    pub faulted_sandbox_count: usize,
    pub quarantined_sandbox_count: usize,
    pub terminal_sandbox_count: usize,
    pub active_transport_count: usize,
    pub explicit_placement_rule_count: usize,
    pub summary: String,
}
