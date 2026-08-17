use signal_plugin::PluginFormat;
use signal_plugin_au::AuDiscoveredPluginType;
use signal_plugin_clap::ClapDiscoveredPluginType;
use signal_plugin_lv2::Lv2DiscoveredPluginType;
use signal_plugin_vst3::Vst3DiscoveredPluginType;
use signal_runtime::{
    RuntimeLv2ExtensionCapabilitySummary, RuntimeMultichannelIoSummary,
    RuntimePluginComplexIoSummary, RuntimePluginDiscoveredTypeRecord,
    RuntimePluginFormatPlatformCoverageRecord, RuntimePluginHostPlatform,
    RuntimePluginIsolationOutcome, RuntimePluginParityBand,
};

pub(crate) fn runtime_plugin_discovered_type_record(
    discovered: ClapDiscoveredPluginType,
) -> RuntimePluginDiscoveredTypeRecord {
    let descriptor = discovered.descriptor;
    runtime_plugin_discovered_type_record_from_descriptor(
        discovered.plugin_type_id.0,
        discovered.default_io_layout,
        descriptor,
        None,
    )
}

fn runtime_plugin_discovered_type_record_from_descriptor(
    plugin_type_id: String,
    default_io_layout: signal_plugin::PluginIoLayout,
    descriptor: signal_plugin::PluginDescriptor,
    lv2_extension_capabilities: Option<signal_runtime::RuntimeLv2ExtensionCapabilitySummary>,
) -> RuntimePluginDiscoveredTypeRecord {
    RuntimePluginDiscoveredTypeRecord {
        plugin_type_id,
        plugin_id: descriptor.plugin_id.clone(),
        vendor: descriptor.vendor.clone(),
        name: descriptor.name.clone(),
        format: descriptor.format,
        version: descriptor.version.clone(),
        features: descriptor.features.clone(),
        default_io_layout,
        default_multichannel_io: RuntimeMultichannelIoSummary::for_plugin_io(default_io_layout),
        complex_io_summary: RuntimePluginComplexIoSummary::from_plugin_features_and_layout(
            &descriptor.features,
            default_io_layout,
        ),
        audio_bus_count: descriptor.audio_buses.len(),
        parameter_count: descriptor.parameters.len(),
        state_contract: descriptor.state_contract,
        processing_contract: descriptor.processing_contract,
        lifecycle_contract: descriptor.lifecycle_contract,
        lv2_extension_capabilities,
    }
}

pub(crate) fn runtime_vst3_discovered_type_record(
    discovered: Vst3DiscoveredPluginType,
) -> RuntimePluginDiscoveredTypeRecord {
    let descriptor = discovered.descriptor;
    runtime_plugin_discovered_type_record_from_descriptor(
        discovered.plugin_type_id.0,
        discovered.default_io_layout,
        descriptor,
        None,
    )
}

pub(crate) fn runtime_au_discovered_type_record(
    discovered: AuDiscoveredPluginType,
) -> RuntimePluginDiscoveredTypeRecord {
    let descriptor = discovered.descriptor;
    runtime_plugin_discovered_type_record_from_descriptor(
        discovered.plugin_type_id.0,
        discovered.default_io_layout,
        descriptor,
        None,
    )
}

pub(crate) fn runtime_lv2_discovered_type_record(
    discovered: Lv2DiscoveredPluginType,
) -> RuntimePluginDiscoveredTypeRecord {
    let descriptor = discovered.descriptor;
    runtime_plugin_discovered_type_record_from_descriptor(
        discovered.plugin_type_id.0,
        discovered.default_io_layout,
        descriptor,
        Some(RuntimeLv2ExtensionCapabilitySummary::from_lv2_feature_uris(
            &discovered.required_features,
            &discovered.optional_features,
        )),
    )
}

pub(crate) fn runtime_plugin_format_platform_coverage(
) -> Vec<RuntimePluginFormatPlatformCoverageRecord> {
    vec![
        RuntimePluginFormatPlatformCoverageRecord {
            format: PluginFormat::Clap,
            supported_platforms: vec![
                RuntimePluginHostPlatform::MacOs,
                RuntimePluginHostPlatform::Linux,
                RuntimePluginHostPlatform::Windows,
            ],
            unsupported_platforms: Vec::new(),
            linux_parity_band: RuntimePluginParityBand::Portable,
            linux_preferred_sandbox_outcome: Some(RuntimePluginIsolationOutcome::IsolatedSandbox),
            linux_strict_sandbox_default: true,
        },
        RuntimePluginFormatPlatformCoverageRecord {
            format: PluginFormat::Vst3,
            supported_platforms: vec![
                RuntimePluginHostPlatform::MacOs,
                RuntimePluginHostPlatform::Linux,
                RuntimePluginHostPlatform::Windows,
            ],
            unsupported_platforms: Vec::new(),
            linux_parity_band: RuntimePluginParityBand::Portable,
            linux_preferred_sandbox_outcome: Some(RuntimePluginIsolationOutcome::IsolatedSandbox),
            linux_strict_sandbox_default: true,
        },
        RuntimePluginFormatPlatformCoverageRecord {
            format: PluginFormat::Au,
            supported_platforms: vec![RuntimePluginHostPlatform::MacOs],
            unsupported_platforms: vec![
                RuntimePluginHostPlatform::Linux,
                RuntimePluginHostPlatform::Windows,
            ],
            linux_parity_band: RuntimePluginParityBand::Unsupported,
            linux_preferred_sandbox_outcome: None,
            linux_strict_sandbox_default: false,
        },
        RuntimePluginFormatPlatformCoverageRecord {
            format: PluginFormat::Lv2,
            supported_platforms: vec![
                RuntimePluginHostPlatform::MacOs,
                RuntimePluginHostPlatform::Linux,
            ],
            unsupported_platforms: vec![RuntimePluginHostPlatform::Windows],
            linux_parity_band: RuntimePluginParityBand::Portable,
            linux_preferred_sandbox_outcome: Some(RuntimePluginIsolationOutcome::IsolatedSandbox),
            linux_strict_sandbox_default: true,
        },
    ]
}
