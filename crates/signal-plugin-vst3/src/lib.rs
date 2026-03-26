//! VST3 plugin adapter surfaces for Signal.

mod fixtures;
mod vst3_host_adapter;

pub use vst3_host_adapter::*;

#[cfg(test)]
mod tests {
    use super::{Vst3HostAdapter, Vst3HostPlatform};
    use signal_plugin::PluginFormat;

    #[test]
    fn vst3_adapter_reports_supported_format_and_capabilities() {
        let adapter = Vst3HostAdapter::default();
        assert!(adapter.supports_format(PluginFormat::Vst3));
        assert!(!adapter.supports_format(PluginFormat::Clap));
        assert!(adapter.strict_sandbox_default());
        assert!(adapter.advertised_capabilities(2048).supports_state);
    }

    #[test]
    fn vst3_adapter_discovers_linux_scan_roots_and_plugin_types() {
        let adapter = Vst3HostAdapter::default();
        let linux_roots = adapter
            .default_scan_roots(Vst3HostPlatform::Linux)
            .into_iter()
            .map(|root| root.root)
            .collect::<Vec<_>>();
        assert!(linux_roots.iter().any(|root| root == "~/.vst3"));
        assert!(linux_roots.iter().any(|root| root == "/usr/lib/vst3"));

        let discovered = adapter.discover_plugins_for_roots(
            Vst3HostPlatform::Linux,
            &[String::from("~/.vst3"), String::from("/usr/lib/vst3")],
        );
        assert_eq!(discovered.len(), 4);
        assert_eq!(discovered[0].descriptor.format, PluginFormat::Vst3);
        assert!(discovered
            .iter()
            .any(|plugin| plugin.plugin_type_id.0 == "plugin:vst3:linux-synth"));
        assert!(discovered
            .iter()
            .any(|plugin| plugin.plugin_type_id.0 == "plugin:vst3:multiout-instrument"));
        assert!(discovered
            .iter()
            .any(|plugin| plugin.plugin_type_id.0 == "plugin:vst3:bus-fx"));
        assert!(discovered
            .iter()
            .all(|plugin| plugin.module_root.starts_with("~/.vst3/")));
    }

    #[test]
    fn vst3_session_plan_preserves_controller_pairing_and_transport() {
        let adapter = Vst3HostAdapter::default();
        let discovered = adapter
            .discover_plugin_type("plugin:vst3:instrument")
            .expect("discovered vst3 instrument");
        let instance = adapter.instantiate_plugin(&discovered, "instance:vst3:test");
        let session = adapter.prepare_session(&instance, 48_000, 512);

        assert_eq!(session.plugin_type_id.0, "plugin:vst3:instrument");
        assert_eq!(
            session.controller_class_id.as_deref(),
            Some("7E1D8F8A4D874D56A2C44DE250100002")
        );
        assert_eq!(session.io_layout.audio_outputs, 2);
        assert_eq!(
            session.transport,
            signal_plugin::SandboxTransport::SharedMemory
        );
        assert!(session.summary.contains("plugin:vst3:instrument"));
    }
}
