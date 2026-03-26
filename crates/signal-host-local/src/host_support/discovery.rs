use signal_plugin::PluginFormat;
use signal_plugin_au::AuHostPlatform;
use signal_plugin_vst3::Vst3HostPlatform;
use signal_runtime::{PluginScanRequest, RuntimePluginDiscoveredTypeRecord};

use super::{
    runtime_au_discovered_type_record, runtime_plugin_discovered_type_record,
    runtime_vst3_discovered_type_record,
};

pub(crate) fn discovered_plugins_for_scan(
    clap: &signal_plugin_clap::ClapPluginHostAdapter,
    au: &signal_plugin_au::AuHostAdapter,
    vst3: &signal_plugin_vst3::Vst3HostAdapter,
    request: &PluginScanRequest,
) -> Vec<RuntimePluginDiscoveredTypeRecord> {
    let mut discovered = Vec::new();
    let include_clap = request.formats.is_empty() || request.formats.contains(&PluginFormat::Clap);
    if include_clap {
        discovered.extend(
            ["plugin:clap:default", "plugin:clap:sandbox"]
                .into_iter()
                .filter_map(|plugin_type_id| clap.discover_plugin_type(plugin_type_id))
                .map(runtime_plugin_discovered_type_record),
        );
    }

    let include_vst3 = request.formats.is_empty() || request.formats.contains(&PluginFormat::Vst3);
    if include_vst3 {
        discovered.extend(
            vst3.discover_plugins_for_roots(Vst3HostPlatform::MacOs, &request.roots)
                .into_iter()
                .map(runtime_vst3_discovered_type_record),
        );
    }

    let include_au = request.formats.is_empty() || request.formats.contains(&PluginFormat::Au);
    if include_au {
        discovered.extend(
            au.discover_plugins_for_roots(AuHostPlatform::MacOs, &request.roots)
                .into_iter()
                .map(runtime_au_discovered_type_record),
        );
    }

    discovered
}
