use signal_plugin::PluginFormat;
use signal_plugin_au::{AuDiscoveredPluginType, AuHostAdapter, AuHostPlatform};
use signal_plugin_clap::ClapPluginHostAdapter;
use signal_plugin_lv2::{Lv2DiscoveredPluginType, Lv2HostAdapter, Lv2HostPlatform};
use signal_plugin_vst3::{Vst3DiscoveredPluginType, Vst3HostAdapter, Vst3HostPlatform};
use signal_runtime::{
    PluginScanRequest, RuntimePluginDiscoveredTypeRecord, RuntimePluginScanDiagnosticKind,
    RuntimePluginScanDiagnosticRecord,
};

use super::metadata::{
    runtime_au_discovered_type_record, runtime_lv2_discovered_type_record,
    runtime_plugin_discovered_type_record, runtime_vst3_discovered_type_record,
};

pub(crate) struct ServerScanDiscoveries {
    pub(crate) runtime_records: Vec<RuntimePluginDiscoveredTypeRecord>,
    pub(crate) runtime_diagnostics: Vec<RuntimePluginScanDiagnosticRecord>,
    pub(crate) clap: Vec<signal_plugin_clap::ClapDiscoveredPluginType>,
    pub(crate) au: Vec<AuDiscoveredPluginType>,
    pub(crate) lv2: Vec<Lv2DiscoveredPluginType>,
    pub(crate) vst3: Vec<Vst3DiscoveredPluginType>,
}

pub(crate) fn discovered_plugins_for_scan(
    clap: &ClapPluginHostAdapter,
    au: &AuHostAdapter,
    lv2: &Lv2HostAdapter,
    vst3: &Vst3HostAdapter,
    request: &PluginScanRequest,
) -> ServerScanDiscoveries {
    let mut runtime_records = Vec::new();
    let mut runtime_diagnostics = Vec::new();
    let mut clap_discoveries = Vec::new();
    let mut au_discoveries = Vec::new();
    let mut lv2_discoveries = Vec::new();
    let mut vst3_discoveries = Vec::new();
    let include_clap = request.formats.is_empty() || request.formats.contains(&PluginFormat::Clap);
    if include_clap {
        clap_discoveries = clap.discover_plugins_for_roots(&request.roots);
        runtime_records.extend(
            clap_discoveries
                .iter()
                .cloned()
                .map(runtime_plugin_discovered_type_record),
        );
    }

    let include_vst3 = request.formats.is_empty() || request.formats.contains(&PluginFormat::Vst3);
    if include_vst3 {
        vst3_discoveries = vst3.discover_plugins_for_roots(Vst3HostPlatform::Linux, &request.roots);
        runtime_records.extend(
            vst3_discoveries
                .iter()
                .cloned()
                .map(runtime_vst3_discovered_type_record),
        );
    }

    let include_lv2 = request.formats.is_empty() || request.formats.contains(&PluginFormat::Lv2);
    if include_lv2 {
        let lv2_batch =
            lv2.discover_plugins_for_roots_with_diagnostics(Lv2HostPlatform::Linux, &request.roots);
        runtime_diagnostics.extend(lv2_batch.diagnostics.into_iter().map(|diagnostic| {
            RuntimePluginScanDiagnosticRecord {
                format: PluginFormat::Lv2,
                root: diagnostic.root,
                bundle_root: diagnostic.bundle_root,
                manifest_path: diagnostic.manifest_path,
                plugin_type_id: diagnostic.plugin_type_id,
                kind: match diagnostic.kind {
                    signal_plugin_lv2::Lv2DiscoveryDiagnosticKind::MalformedManifest => {
                        RuntimePluginScanDiagnosticKind::MalformedManifest
                    }
                    signal_plugin_lv2::Lv2DiscoveryDiagnosticKind::UnsupportedRequiredFeature => {
                        RuntimePluginScanDiagnosticKind::UnsupportedRequiredFeature
                    }
                },
                detail: diagnostic.detail,
                summary: diagnostic.summary,
            }
        }));
        lv2_discoveries = lv2_batch.discovered;
        runtime_records.extend(
            lv2_discoveries
                .iter()
                .cloned()
                .map(runtime_lv2_discovered_type_record),
        );
    }

    let include_au = request.formats.is_empty() || request.formats.contains(&PluginFormat::Au);
    if include_au {
        au_discoveries = au.discover_plugins_for_roots(AuHostPlatform::MacOs, &request.roots);
        runtime_records.extend(
            au_discoveries
                .iter()
                .cloned()
                .map(runtime_au_discovered_type_record),
        );
    }

    ServerScanDiscoveries {
        runtime_records,
        runtime_diagnostics,
        clap: clap_discoveries,
        au: au_discoveries,
        lv2: lv2_discoveries,
        vst3: vst3_discoveries,
    }
}
