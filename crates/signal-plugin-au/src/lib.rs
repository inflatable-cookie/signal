//! Audio Units plugin format adapter for Signal. macOS only.

#![warn(missing_docs)]

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
        fs::create_dir_all(bundle_root.join("Contents")).expect("au bundle contents should exist");
        fs::write(
            bundle_root.join("Contents").join("Info.plist"),
            au_info_plist_contents(&metadata),
        )
        .expect("au component info plist should be written");
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
        fs::create_dir_all(root.join("Signal Fault.component").join("Contents"))
            .expect("fault au bundle contents should be created");
        fs::write(
            root.join("Signal Fault.component")
                .join("Contents")
                .join("Info.plist"),
            au_info_plist_contents(concat!(
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
            )),
        )
        .expect("fault au info plist should be written");
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

    fn au_info_plist_contents(metadata: &str) -> String {
        let mut plugin_type_id = "";
        let mut component_type = "";
        let mut component_subtype = "";
        let mut manufacturer_code = "";
        let mut vendor = "Signal";
        let mut name = "Signal AU Plugin";
        let mut version = "0.1.0";
        let mut audio_inputs = "2";
        let mut audio_outputs = "2";
        let mut midi_inputs = "0";
        let mut midi_outputs = "0";
        let mut features = "";
        let mut init_failure = None;
        let mut bus_layout_failure = None;
        let mut render_context_failure = None;

        for line in metadata.lines().filter(|line| !line.trim().is_empty()) {
            let Some((key, value)) = line.split_once('=') else {
                continue;
            };
            match key.trim() {
                "plugin_type_id" => plugin_type_id = value.trim(),
                "component_type" => component_type = value.trim(),
                "component_subtype" => component_subtype = value.trim(),
                "manufacturer_code" => manufacturer_code = value.trim(),
                "vendor" => vendor = value.trim(),
                "name" => name = value.trim(),
                "version" => version = value.trim(),
                "audio_inputs" => audio_inputs = value.trim(),
                "audio_outputs" => audio_outputs = value.trim(),
                "midi_inputs" => midi_inputs = value.trim(),
                "midi_outputs" => midi_outputs = value.trim(),
                "features" => features = value.trim(),
                "init_failure" => init_failure = Some(value.trim()),
                "bus_layout_failure" => bus_layout_failure = Some(value.trim()),
                "render_context_failure" => render_context_failure = Some(value.trim()),
                _ => {}
            }
        }

        let feature_array = features
            .split(',')
            .map(str::trim)
            .filter(|feature| !feature.is_empty())
            .map(|feature| format!("    <string>{feature}</string>"))
            .collect::<Vec<_>>()
            .join("\n");
        let mut extras = String::new();
        if let Some(value) = init_failure {
            extras.push_str(&format!(
                "  <key>SignalInitFailure</key>\n  <string>{value}</string>\n"
            ));
        }
        if let Some(value) = bus_layout_failure {
            extras.push_str(&format!(
                "  <key>SignalBusLayoutFailure</key>\n  <string>{value}</string>\n"
            ));
        }
        if let Some(value) = render_context_failure {
            extras.push_str(&format!(
                "  <key>SignalRenderContextFailure</key>\n  <string>{value}</string>\n"
            ));
        }

        format!(
            "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\
<!DOCTYPE plist PUBLIC \"-//Apple//DTD PLIST 1.0//EN\" \"http://www.apple.com/DTDs/PropertyList-1.0.dtd\">\n\
<plist version=\"1.0\">\n\
<dict>\n\
  <key>AudioComponents</key>\n\
  <array>\n\
    <dict>\n\
      <key>manufacturer</key>\n\
      <string>{manufacturer_code}</string>\n\
      <key>name</key>\n\
      <string>{vendor}: {name}</string>\n\
      <key>sandboxSafe</key>\n\
      <false/>\n\
      <key>subtype</key>\n\
      <string>{component_subtype}</string>\n\
      <key>type</key>\n\
      <string>{component_type}</string>\n\
      <key>version</key>\n\
      <integer>1</integer>\n\
    </dict>\n\
  </array>\n\
  <key>CFBundleExecutable</key>\n\
  <string>{name}</string>\n\
  <key>CFBundleIdentifier</key>\n\
  <string>{plugin_type_id}</string>\n\
  <key>CFBundleName</key>\n\
  <string>{name}</string>\n\
  <key>CFBundlePackageType</key>\n\
  <string>BNDL</string>\n\
  <key>CFBundleShortVersionString</key>\n\
  <string>{version}</string>\n\
  <key>SignalPluginTypeId</key>\n\
  <string>{plugin_type_id}</string>\n\
  <key>SignalVendor</key>\n\
  <string>{vendor}</string>\n\
  <key>SignalDisplayName</key>\n\
  <string>{name}</string>\n\
  <key>SignalAudioInputs</key>\n\
  <integer>{audio_inputs}</integer>\n\
  <key>SignalAudioOutputs</key>\n\
  <integer>{audio_outputs}</integer>\n\
  <key>SignalMidiInputs</key>\n\
  <integer>{midi_inputs}</integer>\n\
  <key>SignalMidiOutputs</key>\n\
  <integer>{midi_outputs}</integer>\n\
  <key>SignalFeatures</key>\n\
  <array>\n\
{feature_array}\n\
  </array>\n\
{extras}</dict>\n\
</plist>\n"
        )
    }
}
