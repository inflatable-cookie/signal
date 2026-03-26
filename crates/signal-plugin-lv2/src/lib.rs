//! LV2 plugin adapter surfaces for Signal.

mod fixtures;
mod lv2_host_adapter;

pub use lv2_host_adapter::*;

#[cfg(test)]
mod tests {
    use super::{Lv2HostAdapter, Lv2HostPlatform};
    use signal_plugin::PluginFormat;

    #[test]
    fn lv2_adapter_reports_supported_format_and_capabilities() {
        let adapter = Lv2HostAdapter::default();
        assert!(adapter.supports_format(PluginFormat::Lv2));
        assert!(!adapter.supports_format(PluginFormat::Clap));
        assert!(adapter.strict_sandbox_default());
        assert!(adapter.advertised_capabilities(2048).supports_state);
    }

    #[test]
    fn lv2_adapter_discovers_linux_scan_roots_and_plugin_types() {
        let adapter = Lv2HostAdapter::default();
        let linux_roots = adapter
            .default_scan_roots(Lv2HostPlatform::Linux)
            .into_iter()
            .map(|root| root.root)
            .collect::<Vec<_>>();
        assert!(linux_roots.iter().any(|root| root == "~/.lv2"));
        assert!(linux_roots.iter().any(|root| root == "/usr/lib/lv2"));

        let discovered = adapter.discover_plugins_for_roots(
            Lv2HostPlatform::Linux,
            &[String::from("~/.lv2"), String::from("/usr/lib/lv2")],
        );
        assert_eq!(discovered.len(), 4);
        assert_eq!(discovered[0].descriptor.format, PluginFormat::Lv2);
        assert!(discovered
            .iter()
            .any(|plugin| plugin.plugin_type_id.0 == "plugin:lv2:linux-synth"));
        assert!(discovered
            .iter()
            .any(|plugin| plugin.plugin_type_id.0 == "plugin:lv2:multiout-instrument"));
        assert!(discovered
            .iter()
            .any(|plugin| plugin.plugin_type_id.0 == "plugin:lv2:bus-fx"));
        assert!(discovered
            .iter()
            .all(|plugin| plugin.bundle_root.starts_with("~/.lv2/")));
        assert!(discovered
            .iter()
            .all(|plugin| plugin.manifest_path.ends_with("/manifest.ttl")));
    }

    #[test]
    fn lv2_session_plan_preserves_uri_manifest_and_transport() {
        let adapter = Lv2HostAdapter::default();
        let discovered = adapter
            .discover_plugin_type("plugin:lv2:linux-synth")
            .expect("discovered lv2 synth");
        let instance = adapter.instantiate_plugin(&discovered, "instance:lv2:test");
        let session = adapter.prepare_session(&instance, 48_000, 512);

        assert_eq!(session.plugin_type_id.0, "plugin:lv2:linux-synth");
        assert_eq!(
            session.plugin_uri,
            "https://signal.dev/plugins/lv2/linux-synth"
        );
        assert_eq!(session.io_layout.audio_outputs, 2);
        assert_eq!(
            session.transport,
            signal_plugin::SandboxTransport::SharedMemory
        );
        assert!(session.summary.contains("manifest.ttl"));
    }
}
