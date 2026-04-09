use super::introspection::{
    metadata_descriptor, metadata_io_layout, read_vst3_factory_classes, read_vst3_module_metadata,
    Vst3FactoryClassRole,
};
use super::*;
use std::{env, fs, path::PathBuf};

impl Vst3HostAdapter {
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

    pub fn discover_plugins_for_roots(
        &self,
        platform: Vst3HostPlatform,
        roots: &[String],
    ) -> Vec<Vst3DiscoveredPluginType> {
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
                if !file_name.ends_with(".vst3") {
                    continue;
                }
                let Ok(metadata) = read_vst3_module_metadata(&path) else {
                    continue;
                };
                let Ok(factory_classes) = read_vst3_factory_classes(&path) else {
                    continue;
                };
                if !factory_classes.iter().any(|class| {
                    class.role == Vst3FactoryClassRole::Component
                        && class.class_id == metadata.class_id
                        && class.category == metadata.category
                        && class.name == metadata.name
                }) {
                    continue;
                }
                if let Some(controller_class_id) = metadata.controller_class_id.as_deref() {
                    if !factory_classes.iter().any(|class| {
                        class.role == Vst3FactoryClassRole::Controller
                            && class.class_id == controller_class_id
                            && class.name == metadata.name
                    }) {
                        continue;
                    }
                }
                if metadata.controller_class_id.is_none()
                    && factory_classes
                        .iter()
                        .any(|class| class.role == Vst3FactoryClassRole::Controller)
                {
                    continue;
                }
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
        discovered
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
