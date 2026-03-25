use super::*;

pub(super) fn runtime_plugin_host_platform_sort_key(platform: RuntimePluginHostPlatform) -> u8 {
    match platform {
        RuntimePluginHostPlatform::MacOs => 0,
        RuntimePluginHostPlatform::Linux => 1,
        RuntimePluginHostPlatform::Windows => 2,
    }
}

pub(super) fn runtime_plugin_parity_band(
    coverage: Option<&RuntimePluginFormatPlatformCoverageRecord>,
) -> RuntimePluginParityBand {
    match coverage {
        Some(coverage) if coverage.supported_platforms.is_empty() => {
            RuntimePluginParityBand::Unsupported
        }
        Some(coverage) if coverage.unsupported_platforms.is_empty() => {
            RuntimePluginParityBand::Portable
        }
        Some(_) => RuntimePluginParityBand::Guarded,
        None => RuntimePluginParityBand::Guarded,
    }
}

pub(super) fn runtime_plugin_platform_scope_summary(
    coverage: Option<&RuntimePluginFormatPlatformCoverageRecord>,
) -> String {
    if let Some(coverage) = coverage {
        return coverage.summary.clone();
    }
    "platforms=unknown unsupported=unknown".into()
}

pub(crate) fn plugin_format_sort_key(format: PluginFormat) -> u8 {
    match format {
        PluginFormat::Clap => 0,
        PluginFormat::Vst3 => 1,
        PluginFormat::Au => 2,
        PluginFormat::Lv2 => 3,
        PluginFormat::Native => 4,
    }
}
