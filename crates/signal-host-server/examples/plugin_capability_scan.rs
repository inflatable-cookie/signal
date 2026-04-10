use serde::Serialize;
use signal_host_server::ServerRuntimeHost;
use signal_plugin::{PluginDescriptor, PluginFeature, PluginFormat};
use signal_plugin_au::{AuHostAdapter, AuHostPlatform};
use signal_plugin_clap::ClapPluginHostAdapter;
use signal_plugin_lv2::{Lv2HostAdapter, Lv2HostPlatform};
use signal_plugin_vst3::{Vst3HostAdapter, Vst3HostPlatform};
use signal_runtime::{PluginScanRequest, RuntimeConfig, RuntimeSupervisorApi, SignalRuntime};
use std::collections::BTreeSet;

#[derive(Serialize)]
struct HostScanInventory {
    host_surface: &'static str,
    platform: &'static str,
    scan_roots: Vec<String>,
    scan_formats: Vec<String>,
    discovered_plugin_count: usize,
    plugins: Vec<InventoryPluginRecord>,
}

#[derive(Serialize)]
struct InventoryPluginRecord {
    plugin_type_id: String,
    plugin_id: String,
    format: String,
    vendor: String,
    name: String,
    version: Option<String>,
    features: Vec<String>,
    parameter_count: usize,
    audio_bus_count: usize,
    summary: String,
    launch_root: String,
    interaction_posture: &'static str,
}

fn main() {
    let (roots, formats) = parse_args();
    let runtime = SignalRuntime::new(RuntimeConfig::server(48_000, 512));
    let mut host = ServerRuntimeHost::new(runtime);

    host.start_plugin_scan(PluginScanRequest {
        roots: roots.clone(),
        formats: formats.clone(),
    })
    .expect("server plugin capability scan should succeed");

    let report = host.supervisor_report();
    let discovered_type_ids = report
        .observation
        .plugin_discovery_snapshot
        .discovered_types
        .iter()
        .map(|plugin| plugin.plugin_type_id.clone())
        .collect::<BTreeSet<_>>();

    let mut plugins = Vec::new();
    if formats.is_empty() || formats.contains(&PluginFormat::Clap) {
        let adapter = ClapPluginHostAdapter::default();
        for plugin in adapter.discover_plugins_for_roots(&roots) {
            assert!(
                discovered_type_ids.contains(&plugin.plugin_type_id.0),
                "server host discovery snapshot should include scanned CLAP type {}",
                plugin.plugin_type_id.0
            );
            plugins.push(InventoryPluginRecord::from_clap(plugin));
        }
    }

    if formats.is_empty() || formats.contains(&PluginFormat::Vst3) {
        let adapter = Vst3HostAdapter::default();
        for plugin in adapter.discover_plugins_for_roots(Vst3HostPlatform::Linux, &roots) {
            assert!(
                discovered_type_ids.contains(&plugin.plugin_type_id.0),
                "server host discovery snapshot should include scanned VST3 type {}",
                plugin.plugin_type_id.0
            );
            plugins.push(InventoryPluginRecord::from_vst3(plugin));
        }
    }

    if formats.is_empty() || formats.contains(&PluginFormat::Au) {
        let adapter = AuHostAdapter::default();
        for plugin in adapter.discover_plugins_for_roots(AuHostPlatform::MacOs, &roots) {
            assert!(
                discovered_type_ids.contains(&plugin.plugin_type_id.0),
                "server host discovery snapshot should include scanned AU type {}",
                plugin.plugin_type_id.0
            );
            plugins.push(InventoryPluginRecord::from_au(plugin));
        }
    }

    if formats.is_empty() || formats.contains(&PluginFormat::Lv2) {
        let adapter = Lv2HostAdapter::default();
        let batch = adapter.discover_plugins_for_roots_with_diagnostics(Lv2HostPlatform::Linux, &roots);
        for plugin in batch.discovered {
            assert!(
                discovered_type_ids.contains(&plugin.plugin_type_id.0),
                "server host discovery snapshot should include scanned LV2 type {}",
                plugin.plugin_type_id.0
            );
            plugins.push(InventoryPluginRecord::from_lv2(plugin));
        }
    }

    plugins.sort_by(|left, right| {
        left.format
            .cmp(&right.format)
            .then(left.vendor.cmp(&right.vendor))
            .then(left.name.cmp(&right.name))
            .then(left.plugin_type_id.cmp(&right.plugin_type_id))
    });

    let inventory = HostScanInventory {
        host_surface: "server",
        platform: if cfg!(target_os = "macos") {
            "macos"
        } else if cfg!(target_os = "linux") {
            "linux"
        } else if cfg!(target_os = "windows") {
            "windows"
        } else {
            "unknown"
        },
        scan_roots: roots,
        scan_formats: rendered_formats(&formats),
        discovered_plugin_count: plugins.len(),
        plugins,
    };

    println!(
        "{}",
        serde_json::to_string_pretty(&inventory)
            .expect("server plugin capability inventory should serialize")
    );
}

impl InventoryPluginRecord {
    fn from_clap(plugin: signal_plugin_clap::ClapDiscoveredPluginType) -> Self {
        Self::from_descriptor(
            plugin.plugin_type_id.0,
            PluginFormat::Clap,
            normalized_clap_launch_root(&plugin.library_path),
            plugin.descriptor,
        )
    }

    fn from_vst3(plugin: signal_plugin_vst3::Vst3DiscoveredPluginType) -> Self {
        Self::from_descriptor(
            plugin.plugin_type_id.0,
            PluginFormat::Vst3,
            plugin.module_root,
            plugin.descriptor,
        )
    }

    fn from_au(plugin: signal_plugin_au::AuDiscoveredPluginType) -> Self {
        Self::from_descriptor(
            plugin.plugin_type_id.0,
            PluginFormat::Au,
            plugin.bundle_root,
            plugin.descriptor,
        )
    }

    fn from_lv2(plugin: signal_plugin_lv2::Lv2DiscoveredPluginType) -> Self {
        Self::from_descriptor(
            plugin.plugin_type_id.0,
            PluginFormat::Lv2,
            plugin.bundle_root,
            plugin.descriptor,
        )
    }

    fn from_descriptor(
        plugin_type_id: String,
        format: PluginFormat,
        launch_root: String,
        descriptor: PluginDescriptor,
    ) -> Self {
        Self {
            plugin_id: descriptor.plugin_id.clone(),
            vendor: descriptor.vendor.clone(),
            name: descriptor.name.clone(),
            version: descriptor.version.clone(),
            features: descriptor.features.iter().map(render_feature).collect(),
            parameter_count: descriptor.parameters.len(),
            audio_bus_count: descriptor.audio_buses.len(),
            summary: format!(
                "{} {} {:?} params={} buses={} features={}",
                descriptor.vendor,
                descriptor.name,
                descriptor.format,
                descriptor.parameters.len(),
                descriptor.audio_buses.len(),
                descriptor.features.len()
            ),
            plugin_type_id,
            format: render_format(format),
            launch_root,
            interaction_posture: "bounded-host-bootstrap",
        }
    }
}

fn parse_args() -> (Vec<String>, Vec<PluginFormat>) {
    let mut roots = Vec::new();
    let mut formats = Vec::new();
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--root" => roots.push(args.next().expect("--root expects a value")),
            "--format" => formats.push(parse_format(&args.next().expect("--format expects a value"))),
            other => panic!("unsupported argument: {other}"),
        }
    }
    (roots, formats)
}

fn parse_format(value: &str) -> PluginFormat {
    match value {
        "clap" => PluginFormat::Clap,
        "vst3" => PluginFormat::Vst3,
        "au" => PluginFormat::Au,
        "lv2" => PluginFormat::Lv2,
        other => panic!("unsupported server scan format: {other}"),
    }
}

fn rendered_formats(formats: &[PluginFormat]) -> Vec<String> {
    formats.iter().copied().map(render_format).collect()
}

fn render_format(format: PluginFormat) -> String {
    match format {
        PluginFormat::Clap => "Clap",
        PluginFormat::Vst3 => "Vst3",
        PluginFormat::Au => "Au",
        PluginFormat::Lv2 => "Lv2",
        PluginFormat::Native => "Native",
    }
    .into()
}

fn render_feature(feature: &PluginFeature) -> String {
    match feature {
        PluginFeature::Instrument => "Instrument",
        PluginFeature::AudioEffect => "AudioEffect",
        PluginFeature::Analyzer => "Analyzer",
        PluginFeature::Utility => "Utility",
        PluginFeature::NoteEffect => "NoteEffect",
    }
    .into()
}

fn normalized_clap_launch_root(path: &str) -> String {
    let rendered = std::path::Path::new(path);
    for ancestor in rendered.ancestors() {
        if ancestor
            .extension()
            .and_then(|extension| extension.to_str())
            .is_some_and(|extension| extension.eq_ignore_ascii_case("clap"))
        {
            return ancestor.to_string_lossy().to_string();
        }
    }
    rendered.to_string_lossy().to_string()
}
