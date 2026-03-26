use super::*;

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

    pub fn discover_plugin_type(&self, plugin_type_id: &str) -> Option<AuDiscoveredPluginType> {
        au_discovered_plugin_type(plugin_type_id)
    }

    pub fn discover_plugins_for_roots(
        &self,
        platform: AuHostPlatform,
        roots: &[String],
    ) -> Vec<AuDiscoveredPluginType> {
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
            "plugin:au:instrument",
            "plugin:au:multiout-instrument",
            "plugin:au:utility",
            "plugin:au:bus-fx",
        ]
        .into_iter()
        .filter_map(|plugin_type_id| {
            let mut discovered = self.discover_plugin_type(plugin_type_id)?;
            discovered.bundle_root = format!(
                "{}/{}",
                matched_roots[0],
                au_fixture_bundle_name(plugin_type_id)
            );
            Some(discovered)
        })
        .collect()
    }
}

fn known_root_matches(known_roots: &[String], root: &str) -> bool {
    known_roots.iter().any(|known| known == root)
}
