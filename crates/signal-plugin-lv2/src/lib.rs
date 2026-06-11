//! LV2 plugin format adapter for Signal.

#![warn(missing_docs)]

mod lv2_host_adapter;

pub use lv2_host_adapter::*;

#[cfg(test)]
mod tests {
    use super::{Lv2DiscoveryDiagnosticKind, Lv2HostAdapter, Lv2HostPlatform};
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
        let root = std::env::temp_dir().join(format!("signal-lv2-{label}-{unique}"));
        fs::create_dir_all(&root).expect("temp lv2 root should be created");
        root
    }

    fn write_lv2_bundle(root: &std::path::Path, bundle: &str, plugin_type_id: &str) {
        let bundle_root = root.join(bundle);
        fs::create_dir_all(&bundle_root).expect("lv2 bundle should be created");
        fs::write(
            bundle_root.join("manifest.ttl"),
            lv2_manifest_contents(plugin_type_id),
        )
        .expect("manifest should be written");
    }

    fn write_manifest_bundle(root: &std::path::Path, bundle: &str, manifest: &str) {
        let bundle_root = root.join(bundle);
        fs::create_dir_all(&bundle_root).expect("lv2 bundle should be created");
        fs::write(bundle_root.join("manifest.ttl"), manifest).expect("manifest should be written");
    }

    fn lv2_manifest_contents(plugin_type_id: &str) -> &'static str {
        match plugin_type_id {
            "plugin:lv2:linux-synth" => {
                "@prefix lv2: <http://lv2plug.in/ns/lv2core#> .\n@prefix signal: <https://signal.dev/ns/lv2#> .\nsignal:plugin_type_id \"plugin:lv2:linux-synth\" .\nsignal:plugin_uri \"https://signal.dev/plugins/lv2/linux-synth\" .\nsignal:vendor \"Signal\" .\nsignal:name \"Signal Linux Synth LV2 Plugin\" .\nsignal:version \"0.1.0\" .\nsignal:audio_inputs \"0\" .\nsignal:audio_outputs \"2\" .\nsignal:midi_inputs \"1\" .\nsignal:midi_outputs \"0\" .\nsignal:required_feature \"http://lv2plug.in/ns/ext/urid#map\" .\nsignal:required_feature \"http://lv2plug.in/ns/ext/worker#schedule\" .\nsignal:supported_extension \"http://lv2plug.in/ns/ext/patch#Message\" .\nsignal:supported_extension \"http://lv2plug.in/ns/ext/state#state\" .\nsignal:feature \"Instrument\" .\nsignal:feature \"Analyzer\" .\n"
            }
            "plugin:lv2:multiout-instrument" => {
                "@prefix lv2: <http://lv2plug.in/ns/lv2core#> .\n@prefix signal: <https://signal.dev/ns/lv2#> .\nsignal:plugin_type_id \"plugin:lv2:multiout-instrument\" .\nsignal:plugin_uri \"https://signal.dev/plugins/lv2/multiout-instrument\" .\nsignal:vendor \"Signal\" .\nsignal:name \"Signal Multi Output Instrument LV2 Plugin\" .\nsignal:version \"0.1.0\" .\nsignal:audio_inputs \"0\" .\nsignal:audio_outputs \"6\" .\nsignal:midi_inputs \"1\" .\nsignal:midi_outputs \"0\" .\nsignal:required_feature \"http://lv2plug.in/ns/ext/urid#map\" .\nsignal:required_feature \"http://lv2plug.in/ns/ext/worker#schedule\" .\nsignal:supported_extension \"http://lv2plug.in/ns/ext/patch#Message\" .\nsignal:supported_extension \"http://lv2plug.in/ns/ext/state#state\" .\nsignal:feature \"Instrument\" .\nsignal:feature \"Analyzer\" .\n"
            }
            "plugin:lv2:utility" => {
                "@prefix lv2: <http://lv2plug.in/ns/lv2core#> .\n@prefix signal: <https://signal.dev/ns/lv2#> .\nsignal:plugin_type_id \"plugin:lv2:utility\" .\nsignal:plugin_uri \"https://signal.dev/plugins/lv2/utility\" .\nsignal:vendor \"Signal\" .\nsignal:name \"Signal Utility LV2 Plugin\" .\nsignal:version \"0.1.0\" .\nsignal:audio_inputs \"2\" .\nsignal:audio_outputs \"2\" .\nsignal:midi_inputs \"0\" .\nsignal:midi_outputs \"0\" .\nsignal:required_feature \"http://lv2plug.in/ns/ext/options#options\" .\nsignal:supported_extension \"http://lv2plug.in/ns/ext/options#options\" .\nsignal:feature \"AudioEffect\" .\nsignal:feature \"Utility\" .\n"
            }
            "plugin:lv2:bus-fx" => {
                "@prefix lv2: <http://lv2plug.in/ns/lv2core#> .\n@prefix signal: <https://signal.dev/ns/lv2#> .\nsignal:plugin_type_id \"plugin:lv2:bus-fx\" .\nsignal:plugin_uri \"https://signal.dev/plugins/lv2/bus-fx\" .\nsignal:vendor \"Signal\" .\nsignal:name \"Signal Bus FX LV2 Plugin\" .\nsignal:version \"0.1.0\" .\nsignal:audio_inputs \"4\" .\nsignal:audio_outputs \"4\" .\nsignal:midi_inputs \"0\" .\nsignal:midi_outputs \"0\" .\nsignal:required_feature \"http://lv2plug.in/ns/ext/urid#map\" .\nsignal:supported_extension \"http://lv2plug.in/ns/ext/patch#Message\" .\nsignal:feature \"AudioEffect\" .\nsignal:feature \"Utility\" .\n"
            }
            other => panic!("unknown LV2 test plugin type: {other}"),
        }
    }

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

        let root = temp_plugin_root("discovery");
        write_lv2_bundle(&root, "Signal Linux Synth.lv2", "plugin:lv2:linux-synth");
        write_lv2_bundle(
            &root,
            "Signal Multi Output Instrument.lv2",
            "plugin:lv2:multiout-instrument",
        );
        write_lv2_bundle(&root, "Signal Bus FX.lv2", "plugin:lv2:bus-fx");
        let discovered = adapter
            .discover_plugins_for_roots(Lv2HostPlatform::Linux, &[root.display().to_string()]);
        assert_eq!(discovered.len(), 3);
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
            .all(|plugin| plugin.bundle_root.starts_with(&root.display().to_string())));
        assert!(discovered
            .iter()
            .all(|plugin| plugin.manifest_path.ends_with("/manifest.ttl")));
        assert!(discovered
            .iter()
            .all(|plugin| plugin.prepare_fault.is_none()));
        assert!(discovered.iter().any(|plugin| {
            plugin.plugin_type_id.0 == "plugin:lv2:linux-synth"
                && plugin.plugin_uri == "https://signal.dev/plugins/lv2/linux-synth"
        }));
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn lv2_adapter_reports_malformed_and_unsupported_required_feature_diagnostics() {
        let adapter = Lv2HostAdapter::default();
        let root = temp_plugin_root("diagnostics");
        write_lv2_bundle(&root, "Signal Linux Synth.lv2", "plugin:lv2:linux-synth");
        write_manifest_bundle(
            &root,
            "Broken Manifest.lv2",
            "@prefix signal: <https://signal.dev/ns/lv2#> .\nsignal:plugin_type_id \"plugin:lv2:broken\"\n",
        );
        write_manifest_bundle(
            &root,
            "Unsupported Feature.lv2",
            "@prefix lv2: <http://lv2plug.in/ns/lv2core#> .\n@prefix signal: <https://signal.dev/ns/lv2#> .\nsignal:plugin_type_id \"plugin:lv2:unsupported-feature\" .\nsignal:plugin_uri \"https://signal.dev/plugins/lv2/unsupported-feature\" .\nsignal:vendor \"Signal\" .\nsignal:name \"Unsupported Feature LV2 Plugin\" .\nsignal:version \"0.1.0\" .\nsignal:audio_inputs \"2\" .\nsignal:audio_outputs \"2\" .\nsignal:midi_inputs \"0\" .\nsignal:midi_outputs \"0\" .\nsignal:required_feature \"http://lv2plug.in/ns/ext/atom#sequence\" .\nsignal:feature \"AudioEffect\" .\n",
        );

        let batch = adapter.discover_plugins_for_roots_with_diagnostics(
            Lv2HostPlatform::Linux,
            &[root.display().to_string()],
        );

        assert_eq!(batch.discovered.len(), 1);
        assert_eq!(batch.diagnostics.len(), 2);
        assert!(batch.diagnostics.iter().any(|diagnostic| {
            diagnostic.kind == Lv2DiscoveryDiagnosticKind::MalformedManifest
                && diagnostic.bundle_root.ends_with("Broken Manifest.lv2")
        }));
        assert!(batch.diagnostics.iter().any(|diagnostic| {
            diagnostic.kind == Lv2DiscoveryDiagnosticKind::UnsupportedRequiredFeature
                && diagnostic.plugin_type_id.as_deref() == Some("plugin:lv2:unsupported-feature")
                && diagnostic
                    .detail
                    .contains("http://lv2plug.in/ns/ext/atom#sequence")
        }));
        let _ = fs::remove_dir_all(root);
    }
}
