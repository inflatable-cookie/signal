use super::introspection::{metadata_descriptor, metadata_io_layout, read_au_component_metadata};
use super::*;
use std::{env, fs, path::PathBuf};

impl AuHostAdapter {
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

    pub fn discover_plugins_for_roots(
        &self,
        platform: AuHostPlatform,
        roots: &[String],
    ) -> Vec<AuDiscoveredPluginType> {
        let roots = if roots.is_empty() {
            self.default_scan_roots(platform)
                .into_iter()
                .map(|root| root.root)
                .collect::<Vec<_>>()
        } else {
            roots.to_vec()
        };
        let mut discovered = Vec::new();
        for root in roots {
            let expanded_root = expand_scan_root(&root);
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
                let Ok(metadata) = read_au_component_metadata(&path) else {
                    continue;
                };
                let plugin = AuDiscoveredPluginType {
                    plugin_type_id: PluginTypeId(metadata.plugin_type_id.clone()),
                    component_type: metadata.component_type.clone(),
                    component_subtype: metadata.component_subtype.clone(),
                    manufacturer_code: metadata.manufacturer_code.clone(),
                    bundle_root: path.to_string_lossy().into_owned(),
                    descriptor: metadata_descriptor(&metadata),
                    default_io_layout: metadata_io_layout(&metadata),
                    failure_contract: AuFailureContract {
                        init_failure: metadata.init_failure.clone(),
                        bus_layout_failure: metadata.bus_layout_failure.clone(),
                        render_context_failure: metadata.render_context_failure.clone(),
                    },
                };
                if !discovered.iter().any(|existing: &AuDiscoveredPluginType| {
                    existing.plugin_type_id == plugin.plugin_type_id
                }) {
                    discovered.push(plugin);
                }
            }
        }
        discovered
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
