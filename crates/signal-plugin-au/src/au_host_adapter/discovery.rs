use super::introspection::{metadata_descriptor, metadata_io_layout, read_au_component_metadata};
use super::*;
use std::{env, fs, path::PathBuf};

impl AuHostAdapter {
    /// Returns the default AU scan roots for the given platform.
    pub fn default_scan_roots(&self, platform: AuHostPlatform) -> Vec<AuScanRoot> {
        match platform {
            AuHostPlatform::MacOs => vec![
                AuScanRoot {
                    root: "~/Library/Audio/Plug-Ins/Components".into(),
                    platform,
                    kind: AuScanRootKind::UserComponentRoot,
                },
                AuScanRoot {
                    root: "/Library/Audio/Plug-Ins/Components".into(),
                    platform,
                    kind: AuScanRootKind::SystemComponentRoot,
                },
                AuScanRoot {
                    root: "/System/Library/Components".into(),
                    platform,
                    kind: AuScanRootKind::BuiltInComponentRoot,
                },
            ],
        }
    }

    /// Scans the given filesystem roots for AU component bundles and returns
    /// all discovered plugin types. An empty root list scans nothing —
    /// system plugin directories are never scanned implicitly; pass
    /// [`AuHostAdapter::default_scan_roots`] explicitly to opt in.
    pub fn discover_plugins_for_roots(
        &self,
        _platform: AuHostPlatform,
        roots: &[String],
    ) -> Vec<AuDiscoveredPluginType> {
        let mut discovered = Vec::new();
        for root in roots {
            let expanded_root = expand_scan_root(&root);
            if expanded_root
                .extension()
                .and_then(|extension| extension.to_str())
                .is_some_and(|extension| extension.eq_ignore_ascii_case("component"))
            {
                push_au_component_if_present(&mut discovered, &expanded_root);
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
                if !file_name.ends_with(".component") {
                    continue;
                }
                push_au_component_if_present(&mut discovered, &path);
            }
        }
        discovered
    }
}

fn push_au_component_if_present(discovered: &mut Vec<AuDiscoveredPluginType>, path: &PathBuf) {
    let Ok(metadata) = read_au_component_metadata(path) else {
        return;
    };
    let plugin = AuDiscoveredPluginType {
        plugin_type_id: PluginTypeId(metadata.plugin_type_id.clone()),
        component_type: metadata.component_type.clone(),
        component_subtype: metadata.component_subtype.clone(),
        manufacturer_code: metadata.manufacturer_code.clone(),
        bundle_root: path.to_string_lossy().into_owned(),
        descriptor: metadata_descriptor(&metadata),
        default_io_layout: metadata_io_layout(&metadata),
    };
    if !discovered
        .iter()
        .any(|existing: &AuDiscoveredPluginType| existing.plugin_type_id == plugin.plugin_type_id)
    {
        discovered.push(plugin);
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
