//! VST3 plugin format adapter for Signal.

#![warn(missing_docs)]

mod vst3_host_adapter;

pub use vst3_host_adapter::*;

#[cfg(test)]
mod tests {
    use super::{Vst3HostAdapter, Vst3HostPlatform};
    use crate::vst3_host_adapter::vst3_scaffold_module_metadata_contents;
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
        let root = std::env::temp_dir().join(format!("signal-vst3-{label}-{unique}"));
        fs::create_dir_all(&root).expect("temp vst3 root should be created");
        root
    }

    fn write_vst3_metadata(bundle_root: &std::path::Path, plugin_type_id: &str) {
        let metadata = vst3_scaffold_module_metadata_contents(plugin_type_id)
            .unwrap_or_else(|| panic!("unknown VST3 scaffold metadata request: {plugin_type_id}"));
        fs::create_dir_all(bundle_root.join("Contents").join("Resources"))
            .expect("vst3 metadata resources should be created");
        fs::write(
            bundle_root.join("Contents").join("Info.plist"),
            vst3_info_plist_contents(&metadata, bundle_root),
        )
        .expect("vst3 info plist should be written");
        fs::write(
            bundle_root
                .join("Contents")
                .join("Resources")
                .join("moduleinfo.json"),
            vst3_moduleinfo_contents(&metadata),
        )
        .expect("vst3 moduleinfo should be written");
    }

    fn vst3_info_plist_contents(metadata: &str, bundle_root: &std::path::Path) -> String {
        let mut plugin_type_id = "";
        let mut name = "Signal VST3 Plugin";
        let mut version = "0.1.0";
        let mut audio_inputs = "2";
        let mut audio_outputs = "2";
        let mut midi_inputs = "0";
        let mut midi_outputs = "0";
        let mut features = "";

        for line in metadata.lines().filter(|line| !line.trim().is_empty()) {
            let Some((key, value)) = line.split_once('=') else {
                continue;
            };
            match key.trim() {
                "plugin_type_id" => plugin_type_id = value.trim(),
                "name" => name = value.trim(),
                "version" => version = value.trim(),
                "audio_inputs" => audio_inputs = value.trim(),
                "audio_outputs" => audio_outputs = value.trim(),
                "midi_inputs" => midi_inputs = value.trim(),
                "midi_outputs" => midi_outputs = value.trim(),
                "features" => features = value.trim(),
                _ => {}
            }
        }

        let executable_name = bundle_root
            .file_stem()
            .and_then(|stem| stem.to_str())
            .unwrap_or(name);
        let feature_array = features
            .split(',')
            .map(str::trim)
            .filter(|feature| !feature.is_empty())
            .map(|feature| format!("    <string>{feature}</string>"))
            .collect::<Vec<_>>()
            .join("\n");
        format!(
            "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\
<!DOCTYPE plist PUBLIC \"-//Apple//DTD PLIST 1.0//EN\" \"http://www.apple.com/DTDs/PropertyList-1.0.dtd\">\n\
<plist version=\"1.0\">\n\
<dict>\n\
  <key>CFBundleExecutable</key>\n\
  <string>{executable_name}</string>\n\
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
</dict>\n\
</plist>\n"
        )
    }

    fn vst3_moduleinfo_contents(metadata: &str) -> String {
        let mut class_id = "";
        let mut controller_class_id = "";
        let mut category = "Fx";
        let mut vendor = "Signal";
        let mut name = "Signal VST3 Plugin";
        let mut version = "0.1.0";

        for line in metadata.lines().filter(|line| !line.trim().is_empty()) {
            let Some((key, value)) = line.split_once('=') else {
                continue;
            };
            match key.trim() {
                "class_id" => class_id = value.trim(),
                "controller_class_id" => controller_class_id = value.trim(),
                "category" => category = value.trim(),
                "vendor" => vendor = value.trim(),
                "name" => name = value.trim(),
                "version" => version = value.trim(),
                _ => {}
            }
        }

        let subcategory = if category.eq_ignore_ascii_case("Instrument") {
            "Instrument"
        } else {
            "Fx"
        };
        let controller_class = if controller_class_id.is_empty()
            || controller_class_id.eq_ignore_ascii_case("none")
        {
            String::new()
        } else {
            format!(
                ",\n    {{\n      \"CID\": \"{controller_class_id}\",\n      \"Category\": \"Component Controller Class\",\n      \"Name\": \"{name}\",\n      \"Vendor\": \"{vendor}\",\n      \"Version\": \"{version}\",\n      \"Sub Categories\": [\"{subcategory}\"]\n    }}"
            )
        };

        format!(
            "{{\n  \"Name\": \"{name}\",\n  \"Version\": \"{version}\",\n  \"Factory Info\": {{\n    \"Vendor\": \"{vendor}\",\n    \"URL\": \"https://signal.dev\",\n    \"E-Mail\": \"\"\n  }},\n  \"Classes\": [\n    {{\n      \"CID\": \"{class_id}\",\n      \"Category\": \"Audio Module Class\",\n      \"Name\": \"{name}\",\n      \"Vendor\": \"{vendor}\",\n      \"Version\": \"{version}\",\n      \"Sub Categories\": [\"{subcategory}\"]\n    }}{controller_class}\n  ]\n}}\n"
        )
    }

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

        let root = temp_plugin_root("discovery");
        let linux_synth = root.join("Signal Linux Synth.vst3");
        fs::create_dir_all(&linux_synth).expect("linux synth bundle should be created");
        write_vst3_metadata(&linux_synth, "plugin:vst3:linux-synth");
        let multiout = root.join("Signal Multi Output Instrument.vst3");
        fs::create_dir_all(&multiout).expect("multiout bundle should be created");
        write_vst3_metadata(&multiout, "plugin:vst3:multiout-instrument");
        let utility = root.join("Signal Utility.vst3");
        fs::create_dir_all(&utility).expect("utility bundle should be created");
        write_vst3_metadata(&utility, "plugin:vst3:utility");
        let bus_fx = root.join("Signal Bus FX.vst3");
        fs::create_dir_all(&bus_fx).expect("bus fx bundle should be created");
        write_vst3_metadata(&bus_fx, "plugin:vst3:bus-fx");
        let discovered = adapter
            .discover_plugins_for_roots(Vst3HostPlatform::Linux, &[root.display().to_string()]);
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
            .all(|plugin| plugin.module_root.starts_with(&root.display().to_string())));
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn vst3_session_plan_preserves_controller_pairing_and_transport() {
        let adapter = Vst3HostAdapter::default();
        let root = temp_plugin_root("session");
        let instrument = root.join("Signal Instrument.vst3");
        fs::create_dir_all(&instrument).expect("instrument bundle should be created");
        write_vst3_metadata(&instrument, "plugin:vst3:instrument");
        let discovered = adapter
            .discover_plugins_for_roots(Vst3HostPlatform::Linux, &[root.display().to_string()])
            .into_iter()
            .find(|plugin| plugin.plugin_type_id.0 == "plugin:vst3:instrument")
            .expect("discovered vst3 instrument");
        let instance = adapter
            .instantiate_plugin(&discovered, "instance:vst3:test")
            .expect("instantiated vst3 instrument");
        let stored_state = adapter
            .store_state_snapshot(&instance)
            .expect("stored vst3 state");
        let loaded_state = adapter
            .load_state_snapshot(&instance, stored_state.bytes.as_slice())
            .expect("loaded vst3 state");
        let activation = adapter
            .activate_instance(&instance, 48_000, 512, Some(&loaded_state))
            .expect("activated vst3 instance");
        let session = adapter.prepare_session(&instance, 48_000, 512);
        let block = adapter
            .execute_block(&instance, &session, Some(&loaded_state), 7, 256, 3, 1)
            .expect("executed vst3 block");
        let teardown = adapter
            .teardown_instance(&instance, Some(&loaded_state))
            .expect("tore down vst3 instance");

        assert_eq!(session.plugin_type_id.0, "plugin:vst3:instrument");
        assert_eq!(
            session.controller_class_id.as_deref(),
            Some("7E1D8F8A4D874D56A2C44DE250100002")
        );
        assert_eq!(stored_state.instance_id.0, "instance:vst3:test");
        assert!(!stored_state.bytes.is_empty());
        assert_eq!(loaded_state.digest, stored_state.digest);
        assert_eq!(activation.state_bytes, loaded_state.bytes.len());
        assert_eq!(teardown.flushed_state_bytes, loaded_state.bytes.len());
        assert_eq!(session.component_name, "Signal Instrument VST3 Plugin");
        assert_eq!(
            session.controller_name.as_deref(),
            Some("Signal Instrument VST3 Plugin")
        );
        assert_eq!(session.factory_export_count, 2);
        assert_eq!(session.io_layout.audio_outputs, 2);
        assert_eq!(
            session.transport,
            signal_plugin::SandboxTransport::SharedMemory
        );
        assert_eq!(block.block_sequence, 7);
        assert_eq!(block.block_frames, 256);
        assert_eq!(block.audio_output_channels, 2);
        assert_eq!(block.parameter_event_count, 3);
        assert_eq!(block.midi_event_count, 1);
        assert!(!block.parameter_signature.is_empty());
        assert_eq!(
            block.parameter_application_order,
            "block7:parameters(3)->midi(1)"
        );
        assert_eq!(
            block.event_packet_order,
            "block7[param_packets=3,midi_packets=1,apply=parameters_then_midi]"
        );
        assert!(block.state_transition.contains("state_transition=applied"));
        assert!(!block.next_state_digest.is_empty());
        assert_eq!(
            block.state_digest.as_deref(),
            Some(stored_state.digest.as_str())
        );
        assert!(session.summary.contains("plugin:vst3:instrument"));
        assert!(session.summary.contains("factory_exports=2"));
        assert!(block.summary.contains("block_sequence=7"));
        assert!(block.summary.contains("parameter_events=3"));
        assert!(block.summary.contains("parameter_signature="));
        assert!(block
            .summary
            .contains("parameter_application_order=block7:parameters(3)->midi(1)"));
        assert!(block.summary.contains(
            "event_packet_order=block7[param_packets=3,midi_packets=1,apply=parameters_then_midi]"
        ));
        assert!(block.summary.contains("next_state_digest="));
        assert!(block.summary.contains("state_digest="));
        assert!(activation.summary.contains("state_bytes="));
        assert!(teardown.summary.contains("flushed_state_bytes="));
        let _ = fs::remove_dir_all(root);
    }
}
