use super::introspection::parse_lv2_bundle;
use super::*;
use std::{env, fs, path::PathBuf};

impl Lv2HostAdapter {
    /// Returns the default LV2 scan roots for the given platform.
    pub fn default_scan_roots(&self, platform: Lv2HostPlatform) -> Vec<Lv2ScanRoot> {
        match platform {
            Lv2HostPlatform::MacOs => vec![
                Lv2ScanRoot {
                    root: "~/Library/Audio/Plug-Ins/LV2".into(),
                    platform,
                    kind: Lv2ScanRootKind::UserBundleRoot,
                },
                Lv2ScanRoot {
                    root: "/Library/Audio/Plug-Ins/LV2".into(),
                    platform,
                    kind: Lv2ScanRootKind::SystemBundleRoot,
                },
            ],
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

    /// Scans the given filesystem roots for `.lv2` bundles and returns all
    /// successfully discovered plugin types.
    pub fn discover_plugins_for_roots(
        &self,
        platform: Lv2HostPlatform,
        roots: &[String],
    ) -> Vec<Lv2DiscoveredPluginType> {
        self.discover_plugins_for_roots_with_diagnostics(platform, roots)
            .discovered
    }

    /// Scans the given roots and returns a [`Lv2DiscoveryBatch`] containing
    /// both discovered plugins and per-bundle/per-plugin diagnostics. An
    /// empty root list scans nothing — system plugin directories are never
    /// scanned implicitly.
    ///
    /// Discovery is pure file parsing (real Turtle manifests, rdfs:seeAlso
    /// chased within each bundle) — no plugin binary is ever opened at scan
    /// time. Plugins whose `lv2:requiredFeature` set exceeds the phase-1
    /// allowlist (`urid:map` only) are pre-filtered with a typed
    /// `UnsupportedRequiredFeature` diagnostic.
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
                let bundle = match parse_lv2_bundle(&path) {
                    Ok(bundle) => bundle,
                    Err(detail) => {
                        batch
                            .diagnostics
                            .push(malformed_diagnostic(root, &path, None, detail));
                        continue;
                    }
                };
                for (plugin_uri, detail) in bundle.plugin_faults {
                    batch.diagnostics.push(malformed_diagnostic(
                        root,
                        &path,
                        plugin_uri.as_deref(),
                        detail,
                    ));
                }
                for model in bundle.plugins {
                    let unsupported = model.unsupported_required_features();
                    if !unsupported.is_empty() {
                        batch
                            .diagnostics
                            .push(unsupported_required_feature_diagnostic(
                                root,
                                &path,
                                &model.plugin_uri,
                                unsupported,
                            ));
                        continue;
                    }
                    let plugin = super::introspection::discovered_plugin_from_model(&path, &model);
                    if !batch
                        .discovered
                        .iter()
                        .any(|existing| existing.plugin_type_id == plugin.plugin_type_id)
                    {
                        batch.discovered.push(plugin);
                    }
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

fn malformed_diagnostic(
    root: &str,
    bundle_root: &std::path::Path,
    plugin_uri: Option<&str>,
    detail: String,
) -> Lv2DiscoveryDiagnostic {
    let manifest_path = bundle_root.join("manifest.ttl");
    let plugin_type_id = plugin_uri.map(|uri| format!("plugin:lv2:{uri}"));
    let summary = format!(
        "format=Lv2 kind=MalformedManifest root={} bundle={} plugin_type={} detail={}",
        root,
        bundle_root.display(),
        plugin_type_id.as_deref().unwrap_or("-"),
        detail,
    );
    Lv2DiscoveryDiagnostic {
        root: root.into(),
        bundle_root: bundle_root.to_string_lossy().into_owned(),
        manifest_path: Some(manifest_path.to_string_lossy().into_owned()),
        plugin_type_id,
        kind: Lv2DiscoveryDiagnosticKind::MalformedManifest,
        detail,
        summary,
    }
}

fn unsupported_required_feature_diagnostic(
    root: &str,
    bundle_root: &std::path::Path,
    plugin_uri: &str,
    unsupported_required: Vec<String>,
) -> Lv2DiscoveryDiagnostic {
    let manifest_path = bundle_root.join("manifest.ttl");
    let plugin_type_id = format!("plugin:lv2:{plugin_uri}");
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
        plugin_type_id: Some(plugin_type_id),
        kind: Lv2DiscoveryDiagnosticKind::UnsupportedRequiredFeature,
        detail,
        summary,
    }
}
