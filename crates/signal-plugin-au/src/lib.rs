//! Audio Unit plugin adapter surfaces for Signal.

mod au_host_adapter;

pub use au_host_adapter::*;

#[cfg(test)]
mod tests {
    use super::{AuHostAdapter, AuHostPlatform};
    use crate::au_host_adapter::au_scaffold_component_metadata_contents;
    use signal_plugin::PluginFormat;
    use std::{
        fs,
        time::{SystemTime, UNIX_EPOCH},
    };

    fn temp_plugin_root(label: &str) -> std::path::PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock should be after epoch")
            .as_nanos();
        let root = std::env::temp_dir().join(format!("signal-au-{label}-{unique}"));
        fs::create_dir_all(&root).expect("temp au root should be created");
        root
    }

    fn write_au_bundle(bundle_root: &std::path::Path, plugin_type_id: &str) {
        let metadata = au_scaffold_component_metadata_contents(plugin_type_id)
            .unwrap_or_else(|| panic!("unknown AU scaffold metadata request: {plugin_type_id}"));
        fs::create_dir_all(bundle_root.join("Contents").join("Resources"))
            .expect("au metadata resources should be created");
        fs::write(
            bundle_root
                .join("Contents")
                .join("Resources")
                .join("signal-au-component.txt"),
            metadata,
        )
        .expect("au component metadata should be written");
    }

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

        let root = temp_plugin_root("discovery");
        write_au_bundle(
            &root.join("Signal Instrument.component"),
            "plugin:au:instrument",
        );
        write_au_bundle(
            &root.join("Signal Multi Output Instrument.component"),
            "plugin:au:multiout-instrument",
        );
        write_au_bundle(&root.join("Signal Bus FX.component"), "plugin:au:bus-fx");
        let discovered = adapter
            .discover_plugins_for_roots(AuHostPlatform::MacOs, &[root.display().to_string()]);
        assert_eq!(discovered.len(), 3);
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
        assert!(discovered
            .iter()
            .all(|plugin| plugin.bundle_root.starts_with(&root.display().to_string())));
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn au_session_plan_preserves_component_identity_and_transport() {
        let adapter = AuHostAdapter::default();
        let root = temp_plugin_root("session");
        write_au_bundle(
            &root.join("Signal Instrument.component"),
            "plugin:au:instrument",
        );
        let discovered = adapter
            .discover_plugins_for_roots(AuHostPlatform::MacOs, &[root.display().to_string()])
            .into_iter()
            .find(|plugin| plugin.plugin_type_id.0 == "plugin:au:instrument")
            .expect("discovered au instrument");
        let instance = adapter
            .instantiate_plugin(&discovered, "instance:au:test")
            .expect("au instantiate should succeed");
        let session = adapter
            .prepare_session(&instance, 48_000, 512)
            .expect("au prepare should succeed");

        assert_eq!(session.plugin_type_id.0, "plugin:au:instrument");
        assert_eq!(session.component_type, "aumu");
        assert_eq!(session.component_subtype, "sigi");
        assert_eq!(session.io_layout.audio_outputs, 2);
        assert_eq!(
            session.transport,
            signal_plugin::SandboxTransport::SharedMemory
        );
        assert!(session.summary.contains("plugin:au:instrument"));
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn au_adapter_exposes_bounded_state_activation_and_teardown_records() {
        let adapter = AuHostAdapter::default();
        let root = temp_plugin_root("lifecycle");
        write_au_bundle(
            &root.join("Signal Instrument.component"),
            "plugin:au:instrument",
        );
        let discovered = adapter
            .discover_plugins_for_roots(AuHostPlatform::MacOs, &[root.display().to_string()])
            .into_iter()
            .find(|plugin| plugin.plugin_type_id.0 == "plugin:au:instrument")
            .expect("discovered au instrument");
        let instance = adapter
            .instantiate_plugin(&discovered, "instance:au:lifecycle")
            .expect("au instantiate should succeed");
        let state = adapter.store_state_snapshot(&instance);
        let activation = adapter
            .activate_instance(&instance, 48_000, 512, Some(&state))
            .expect("au activation should succeed");
        let teardown = adapter.teardown_instance(&instance, Some(&state));

        assert_eq!(state.plugin_type_id.0, "plugin:au:instrument");
        assert!(!state.bytes.is_empty());
        assert!(state.digest.starts_with("au-state:plugin:au:instrument"));
        assert!(state.summary.contains("state_stored=1"));

        assert_eq!(activation.sample_rate_hz, 48_000);
        assert_eq!(activation.max_block_frames, 512);
        assert!(activation.summary.contains("activation=ready"));
        assert!(activation.summary.contains("state_digest="));

        assert_eq!(teardown.flushed_state_bytes, state.bytes.len());
        assert!(teardown.summary.contains("flushed_state_bytes="));
        assert!(teardown.summary.contains("suspended=1"));
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn au_adapter_exposes_explicit_fault_boundaries_from_metadata() {
        let adapter = AuHostAdapter::default();
        let root = temp_plugin_root("faults");
        fs::create_dir_all(
            root.join("Signal Fault.component")
                .join("Contents")
                .join("Resources"),
        )
        .expect("fault au metadata resources should be created");
        fs::write(
            root.join("Signal Fault.component")
                .join("Contents")
                .join("Resources")
                .join("signal-au-component.txt"),
            concat!(
                "plugin_type_id=plugin:au:faulty\n",
                "component_type=aumu\n",
                "component_subtype=sigf\n",
                "manufacturer_code=sigl\n",
                "vendor=Signal\n",
                "name=Signal Fault AU Plugin\n",
                "version=0.1.0\n",
                "audio_inputs=0\n",
                "audio_outputs=2\n",
                "midi_inputs=1\n",
                "midi_outputs=0\n",
                "features=Instrument,Analyzer\n",
                "render_context_failure=unsupported_sample_rate\n"
            ),
        )
        .expect("fault au metadata should be written");
        let discovered = adapter
            .discover_plugins_for_roots(AuHostPlatform::MacOs, &[root.display().to_string()])
            .into_iter()
            .find(|plugin| plugin.plugin_type_id.0 == "plugin:au:faulty")
            .expect("discovered faulty au");
        let instance = adapter
            .instantiate_plugin(&discovered, "instance:au:fault")
            .expect("faulty au instantiate should still succeed");
        let prepare = adapter
            .prepare_session(&instance, 48_000, 512)
            .expect("faulty au prepare should still succeed");
        let state = adapter.store_state_snapshot(&instance);
        let activation = adapter.activate_instance(&instance, 48_000, 512, Some(&state));

        assert_eq!(prepare.plugin_type_id.0, "plugin:au:faulty");
        assert!(activation
            .expect_err("faulty au activation should fail")
            .contains("unsupported_sample_rate"));
        let _ = fs::remove_dir_all(root);
    }
}
