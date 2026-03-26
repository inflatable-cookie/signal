use super::*;

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

    pub fn discover_plugin_type(&self, plugin_type_id: &str) -> Option<Vst3DiscoveredPluginType> {
        vst3_discovered_plugin_type(plugin_type_id)
    }

    pub fn discover_plugins_for_roots(
        &self,
        platform: Vst3HostPlatform,
        roots: &[String],
    ) -> Vec<Vst3DiscoveredPluginType> {
        let known_roots = self
            .default_scan_roots(platform)
            .into_iter()
            .map(|root| root.root)
            .collect::<Vec<_>>();
        let matched_roots = if roots.is_empty() {
            known_roots
        } else {
            roots
                .iter()
                .filter(|root| known_root_matches(&known_roots, root))
                .cloned()
                .collect::<Vec<_>>()
        };
        if matched_roots.is_empty() {
            return Vec::new();
        }

        let fixture_ids = match platform {
            Vst3HostPlatform::MacOs => vec![
                "plugin:vst3:instrument",
                "plugin:vst3:multiout-instrument",
                "plugin:vst3:utility",
                "plugin:vst3:bus-fx",
            ],
            Vst3HostPlatform::Linux => vec![
                "plugin:vst3:linux-synth",
                "plugin:vst3:multiout-instrument",
                "plugin:vst3:utility",
                "plugin:vst3:bus-fx",
            ],
            Vst3HostPlatform::Windows => vec![
                "plugin:vst3:instrument",
                "plugin:vst3:multiout-instrument",
                "plugin:vst3:utility",
                "plugin:vst3:bus-fx",
            ],
        };

        fixture_ids
            .into_iter()
            .filter_map(|plugin_type_id| {
                let mut discovered = self.discover_plugin_type(plugin_type_id)?;
                discovered.module_root = format!(
                    "{}/{}",
                    matched_roots[0],
                    vst3_fixture_bundle_name(plugin_type_id)
                );
                Some(discovered)
            })
            .collect()
    }
}

fn known_root_matches(known_roots: &[String], root: &str) -> bool {
    known_roots.iter().any(|known| known == root)
}
