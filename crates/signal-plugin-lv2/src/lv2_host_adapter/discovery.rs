use super::*;
use crate::lv2_host_adapter::introspection::{
    discovered_plugin_from_manifest, parse_lv2_manifest, unsupported_required_features,
};
use std::{env, fs, path::PathBuf};

impl Lv2HostAdapter {
    /// Returns the default LV2 scan roots for the given platform.
    pub fn default_scan_roots(&self, platform: Lv2HostPlatform) -> Vec<Lv2ScanRoot> {
        match platform {
            Lv2HostPlatform::Linux => vec![
                Lv2ScanRoot {
                    root: "~/.lv2".into(),
                    platform,
                    kind: Lv2ScanRootKind::UserBundleRoot,
                },
                Lv2ScanRoot {
                    root: "/usr/lib/lv2".into(),
                    platform,
                    kind: Lv2ScanRootKind::SystemBundleRoot,
                },
                Lv2ScanRoot {
                    root: "/usr/local/lib/lv2".into(),
                    platform,
                    kind: Lv2ScanRootKind::SystemBundleRoot,
                },
            ],
        }
    }

    /// Scans the given filesystem roots for LV2 bundles and returns all successfully discovered plugin types.
    pub fn discover_plugins_for_roots(
        &self,
        platform: Lv2HostPlatform,
        roots: &[String],
    ) -> Vec<Lv2DiscoveredPluginType> {
        self.discover_plugins_for_roots_with_diagnostics(platform, roots)
            .discovered
    }

    /// Scans the given roots and returns a [`Lv2DiscoveryBatch`] containing both discovered plugins and any per-bundle diagnostics. An empty root list scans nothing — system plugin directories are never scanned implicitly.
    pub fn discover_plugins_for_roots_with_diagnostics(
        &self,
        platform: Lv2HostPlatform,
        roots: &[String],
    ) -> Lv2DiscoveryBatch {
        let _ = platform;
        let mut batch = Lv2DiscoveryBatch::default();
        for root in roots {
            let expanded_root = expand_scan_root(root);
            let Ok(entries) = fs::read_dir(&expanded_root) else {
                continue;
            };
            for entry in entries.flatten() {
                let path = entry.path();
                let Some(file_name) = path.file_name().and_then(|name| name.to_str()) else {
                    continue;
                };
                if !file_name.ends_with(".lv2") || !path.is_dir() {
                    continue;
                }
                let metadata = match parse_lv2_manifest(&path) {
                    Ok(metadata) => metadata,
                    Err(detail) => {
                        batch
                            .diagnostics
                            .push(malformed_manifest_diagnostic(root, &path, detail));
                        continue;
                    }
                };
                let unsupported_required =
                    unsupported_required_features(&metadata.required_features);
                if !unsupported_required.is_empty() {
                    batch
                        .diagnostics
                        .push(unsupported_required_feature_diagnostic(
                            root,
                            &path,
                            &metadata.plugin_type_id,
                            unsupported_required,
                        ));
                    continue;
                }
                let plugin = discovered_plugin_from_manifest(&path, metadata);
                if !batch
                    .discovered
                    .iter()
                    .any(|existing: &Lv2DiscoveredPluginType| {
                        existing.plugin_type_id == plugin.plugin_type_id
                    })
                {
                    batch.discovered.push(plugin);
                }
            }
        }
        batch
    }
}

fn expand_scan_root(root: &str) -> PathBuf {
    if let Some(stripped) = root.strip_prefix("~/") {
        if let Some(home) = env::var_os("HOME") {
            return PathBuf::from(home).join(stripped);
        }
    }
    PathBuf::from(root)
}

fn malformed_manifest_diagnostic(
    root: &str,
    bundle_root: &std::path::Path,
    detail: String,
) -> Lv2DiscoveryDiagnostic {
    let manifest_path = bundle_root.join("manifest.ttl");
    let summary = format!(
        "format=Lv2 kind=MalformedManifest root={} bundle={} detail={}",
        root,
        bundle_root.display(),
        detail,
    );
    Lv2DiscoveryDiagnostic {
        root: root.into(),
        bundle_root: bundle_root.to_string_lossy().into_owned(),
        manifest_path: Some(manifest_path.to_string_lossy().into_owned()),
        plugin_type_id: None,
        kind: Lv2DiscoveryDiagnosticKind::MalformedManifest,
        detail,
        summary,
    }
}

fn unsupported_required_feature_diagnostic(
    root: &str,
    bundle_root: &std::path::Path,
    plugin_type_id: &str,
    unsupported_required: Vec<String>,
) -> Lv2DiscoveryDiagnostic {
    let manifest_path = bundle_root.join("manifest.ttl");
    let detail = format!(
        "unsupported required features: {}",
        unsupported_required.join(","),
    );
    let summary = format!(
        "format=Lv2 kind=UnsupportedRequiredFeature root={} bundle={} plugin_type={} detail={}",
        root,
        bundle_root.display(),
        plugin_type_id,
        detail,
    );
    Lv2DiscoveryDiagnostic {
        root: root.into(),
        bundle_root: bundle_root.to_string_lossy().into_owned(),
        manifest_path: Some(manifest_path.to_string_lossy().into_owned()),
        plugin_type_id: Some(plugin_type_id.into()),
        kind: Lv2DiscoveryDiagnosticKind::UnsupportedRequiredFeature,
        detail,
        summary,
    }
}
