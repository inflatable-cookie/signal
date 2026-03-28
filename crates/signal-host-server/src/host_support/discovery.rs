use signal_plugin::PluginFormat;
use signal_plugin_au::{AuHostAdapter, AuHostPlatform};
use signal_plugin_clap::ClapPluginHostAdapter;
use signal_plugin_lv2::{Lv2HostAdapter, Lv2HostPlatform};
use signal_plugin_vst3::{Vst3HostAdapter, Vst3HostPlatform};
use signal_runtime::{PluginScanRequest, RuntimePluginDiscoveredTypeRecord};

use super::metadata::{
    runtime_au_discovered_type_record, runtime_lv2_discovered_type_record,
    runtime_plugin_discovered_type_record, runtime_vst3_discovered_type_record,
};

pub(crate) fn discovered_plugins_for_scan(
    au: &AuHostAdapter,
    lv2: &Lv2HostAdapter,
    vst3: &Vst3HostAdapter,
    request: &PluginScanRequest,
) -> Vec<RuntimePluginDiscoveredTypeRecord> {
    let mut discovered = Vec::new();
    let include_clap = request.formats.is_empty() || request.formats.contains(&PluginFormat::Clap);
    if include_clap {
        let clap = ClapPluginHostAdapter::default();
        discovered.extend(
            ["plugin:clap:server", "plugin:clap:sandbox"]
                .into_iter()
                .filter_map(|plugin_type_id| clap.discover_plugin_type(plugin_type_id))
                .map(runtime_plugin_discovered_type_record),
        );
    }

    let include_vst3 = request.formats.is_empty() || request.formats.contains(&PluginFormat::Vst3);
    if include_vst3 {
        discovered.extend(
            vst3.discover_plugins_for_roots(Vst3HostPlatform::Linux, &request.roots)
                .into_iter()
                .map(runtime_vst3_discovered_type_record),
        );
    }

    let include_lv2 = request.formats.is_empty() || request.formats.contains(&PluginFormat::Lv2);
    if include_lv2 {
        discovered.extend(
            lv2.discover_plugins_for_roots(Lv2HostPlatform::Linux, &request.roots)
                .into_iter()
                .map(runtime_lv2_discovered_type_record),
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
