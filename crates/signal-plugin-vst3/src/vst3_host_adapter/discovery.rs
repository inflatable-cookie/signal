use super::introspection::{metadata_descriptor, metadata_io_layout, read_vst3_bundle_snapshot};
use super::*;
use std::{
    env, fs,
    path::{Path, PathBuf},
};

impl Vst3HostAdapter {
    /// Returns the default VST3 scan roots for the given platform.
    pub fn default_scan_roots(&self, platform: Vst3HostPlatform) -> Vec<Vst3ScanRoot> {
        match platform {
            Vst3HostPlatform::MacOs => vec![
                Vst3ScanRoot {
                    root: "~/Library/Audio/Plug-Ins/VST3".into(),
                    platform,
                    kind: Vst3ScanRootKind::UserBundleRoot,
                },
                Vst3ScanRoot {
                    root: "/Library/Audio/Plug-Ins/VST3".into(),
                    platform,
                    kind: Vst3ScanRootKind::SystemBundleRoot,
                },
            ],
            Vst3HostPlatform::Linux => vec![
                Vst3ScanRoot {
                    root: "~/.vst3".into(),
                    platform,
                    kind: Vst3ScanRootKind::UserBundleRoot,
                },
                Vst3ScanRoot {
                    root: "/usr/lib/vst3".into(),
                    platform,
                    kind: Vst3ScanRootKind::SystemBundleRoot,
                },
                Vst3ScanRoot {
                    root: "/usr/local/lib/vst3".into(),
                    platform,
                    kind: Vst3ScanRootKind::SystemBundleRoot,
                },
            ],
            Vst3HostPlatform::Windows => vec![
                Vst3ScanRoot {
                    root: "%LOCALAPPDATA%/Programs/Common/VST3".into(),
                    platform,
                    kind: Vst3ScanRootKind::UserBundleRoot,
                },
                Vst3ScanRoot {
                    root: "%COMMONPROGRAMFILES%/VST3".into(),
                    platform,
                    kind: Vst3ScanRootKind::SystemBundleRoot,
                },
            ],
        }
    }

    /// Scans the given filesystem roots for VST3 bundle directories and
    /// returns all discovered plugin types. An empty root list scans
    /// nothing — system plugin directories are never scanned implicitly;
    /// pass [`Vst3HostAdapter::default_scan_roots`] explicitly to opt in.
    pub fn discover_plugins_for_roots(
        &self,
        platform: Vst3HostPlatform,
        roots: &[String],
    ) -> Vec<Vst3DiscoveredPluginType> {
        let mut discovered = Vec::new();
        for root in roots {
            let expanded_root = expand_scan_root(root);
            if expanded_root
                .extension()
                .and_then(|extension| extension.to_str())
                .is_some_and(|extension| extension.eq_ignore_ascii_case("vst3"))
            {
                push_vst3_bundle_if_present(&mut discovered, &expanded_root, platform);
                continue;
            }
            let Ok(entries) = fs::read_dir(&expanded_root) else {
                continue;
            };
            for entry in entries.flatten() {
                let path = entry.path();
                let file_name = entry.file_name();
                let Some(file_name) = file_name.to_str() else {
                    continue;
                };
                if !file_name.ends_with(".vst3") {
                    continue;
                }
                push_vst3_bundle_if_present(&mut discovered, &path, platform);
            }
        }
        discovered
    }
}

fn push_vst3_bundle_if_present(
    discovered: &mut Vec<Vst3DiscoveredPluginType>,
    path: &Path,
    platform: Vst3HostPlatform,
) {
    let Ok(snapshot) = read_vst3_bundle_snapshot(path, platform) else {
        return;
    };
    for metadata in snapshot.plugins {
        let plugin = Vst3DiscoveredPluginType {
            plugin_type_id: PluginTypeId(metadata.plugin_type_id.clone()),
            class_id: metadata.class_id.clone(),
            controller_class_id: metadata.controller_class_id.clone(),
            category: metadata.category.clone(),
            module_root: path.to_string_lossy().into_owned(),
            descriptor: metadata_descriptor(&metadata),
            default_io_layout: metadata_io_layout(&metadata),
        };
        if !discovered
            .iter()
            .any(|existing: &Vst3DiscoveredPluginType| {
                existing.plugin_type_id == plugin.plugin_type_id
            })
        {
            discovered.push(plugin);
        }
    }
}

fn expand_scan_root(root: &str) -> PathBuf {
    if let Some(stripped) = root.strip_prefix("~/") {
        if let Some(home) = env::var_os("HOME") {
            return PathBuf::from(home).join(stripped);
        }
    }
    if root.contains('%') {
        let mut expanded = root.to_string();
        for (key, value) in env::vars() {
            let pattern = format!("%{key}%");
            if expanded.contains(&pattern) {
                expanded = expanded.replace(&pattern, &value);
            }
        }
        return PathBuf::from(expanded);
    }
    PathBuf::from(root)
}
