use super::{Lv2DiscoveryDiagnosticKind, Lv2HostAdapter, Lv2HostPlatform};
use signal_plugin::{PluginFeature, PluginFormat};
use std::{
    fs,
    time::{SystemTime, UNIX_EPOCH},
};

fn temp_plugin_root(label: &str) -> std::path::PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock should be after epoch")
        .as_nanos();
    let root = std::env::temp_dir().join(format!("signal-lv2-{label}-{unique}"));
    fs::create_dir_all(&root).expect("temp lv2 root should be created");
    root
}

fn write_bundle(root: &std::path::Path, bundle: &str, files: &[(&str, &str)]) {
    let bundle_root = root.join(bundle);
    fs::create_dir_all(&bundle_root).expect("lv2 bundle should be created");
    for (name, contents) in files {
        fs::write(bundle_root.join(name), contents).expect("bundle file should be written");
    }
}

fn scan(adapter: &Lv2HostAdapter, root: &std::path::Path) -> super::Lv2DiscoveryBatch {
    adapter.discover_plugins_for_roots_with_diagnostics(
        super::current_lv2_platform(),
        &[root.display().to_string()],
    )
}

mod adapter;
mod discovery;
mod instruments;
mod ports;
mod rejection;
mod see_also;
