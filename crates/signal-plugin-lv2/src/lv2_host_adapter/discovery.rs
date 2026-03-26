use super::*;

impl Lv2HostAdapter {
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

    pub fn discover_plugin_type(&self, plugin_type_id: &str) -> Option<Lv2DiscoveredPluginType> {
        lv2_discovered_plugin_type(plugin_type_id)
    }

    pub fn discover_plugins_for_roots(
        &self,
        platform: Lv2HostPlatform,
        roots: &[String],
    ) -> Vec<Lv2DiscoveredPluginType> {
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

        [
            "plugin:lv2:linux-synth",
            "plugin:lv2:multiout-instrument",
            "plugin:lv2:utility",
            "plugin:lv2:bus-fx",
        ]
        .into_iter()
        .filter_map(|plugin_type_id| {
            let mut discovered = self.discover_plugin_type(plugin_type_id)?;
            let bundle_root = format!(
                "{}/{}",
                matched_roots[0],
                lv2_fixture_bundle_name(plugin_type_id)
            );
            discovered.bundle_root = bundle_root.clone();
            discovered.manifest_path = format!("{bundle_root}/manifest.ttl");
            Some(discovered)
        })
        .collect()
    }
}

fn known_root_matches(known_roots: &[String], root: &str) -> bool {
    known_roots.iter().any(|known| known == root)
}
