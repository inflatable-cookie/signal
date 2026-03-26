//! Audio Unit plugin adapter surfaces for Signal.

mod au_host_adapter;
mod fixtures;

pub use au_host_adapter::*;

#[cfg(test)]
mod tests {
    use super::{AuHostAdapter, AuHostPlatform};
    use signal_plugin::PluginFormat;

    #[test]
    fn au_adapter_reports_supported_format_and_capabilities() {
        let adapter = AuHostAdapter::default();
        assert!(adapter.supports_format(PluginFormat::Au));
        assert!(!adapter.supports_format(PluginFormat::Clap));
        assert!(adapter.strict_sandbox_default());
        assert!(adapter.advertised_capabilities(2048).supports_state);
    }

    #[test]
    fn au_adapter_discovers_macos_scan_roots_and_plugin_types() {
        let adapter = AuHostAdapter::default();
        let mac_roots = adapter
            .default_scan_roots(AuHostPlatform::MacOs)
            .into_iter()
            .map(|root| root.root)
            .collect::<Vec<_>>();
        assert!(mac_roots
            .iter()
            .any(|root| root == "~/Library/Audio/Plug-Ins/Components"));
        assert!(mac_roots
            .iter()
            .any(|root| root == "/Library/Audio/Plug-Ins/Components"));

        let discovered = adapter.discover_plugins_for_roots(
            AuHostPlatform::MacOs,
            &[
                String::from("~/Library/Audio/Plug-Ins/Components"),
                String::from("/Library/Audio/Plug-Ins/Components"),
            ],
        );
        assert_eq!(discovered.len(), 4);
        assert_eq!(discovered[0].descriptor.format, PluginFormat::Au);
        assert!(discovered
            .iter()
            .any(|plugin| plugin.plugin_type_id.0 == "plugin:au:instrument"));
        assert!(discovered
            .iter()
            .any(|plugin| plugin.plugin_type_id.0 == "plugin:au:multiout-instrument"));
        assert!(discovered
            .iter()
            .any(|plugin| plugin.plugin_type_id.0 == "plugin:au:bus-fx"));
        assert!(discovered.iter().all(|plugin| plugin
            .bundle_root
            .starts_with("~/Library/Audio/Plug-Ins/Components/")));
    }

    #[test]
    fn au_session_plan_preserves_component_identity_and_transport() {
        let adapter = AuHostAdapter::default();
        let discovered = adapter
            .discover_plugin_type("plugin:au:instrument")
            .expect("discovered au instrument");
        let instance = adapter.instantiate_plugin(&discovered, "instance:au:test");
        let session = adapter.prepare_session(&instance, 48_000, 512);

        assert_eq!(session.plugin_type_id.0, "plugin:au:instrument");
        assert_eq!(session.component_type, "aumu");
        assert_eq!(session.component_subtype, "sigi");
        assert_eq!(session.io_layout.audio_outputs, 2);
        assert_eq!(
            session.transport,
            signal_plugin::SandboxTransport::SharedMemory
        );
        assert!(session.summary.contains("plugin:au:instrument"));
    }
}
