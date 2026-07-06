//! VST3 plugin format adapter for Signal: bundle discovery, handwritten COM
//! hosting FFI ([`Vst3HostedInstance`] / [`Vst3ProcessSession`]), and the
//! compile-on-demand test fixture.

#![warn(missing_docs)]

#[doc(hidden)]
pub mod fixture;
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
    fn vst3_adapter_skips_bundles_without_metadata_or_binary() {
        let adapter = Vst3HostAdapter::default();
        let root = temp_plugin_root("metadata-required");
        let bundle = root.join("Signal Missing Moduleinfo.vst3");
        fs::create_dir_all(bundle.join("Contents"))
            .expect("vst3 bundle contents should be created");
        let metadata = vst3_scaffold_module_metadata_contents("plugin:vst3:utility")
            .expect("utility scaffold metadata should exist");
        fs::write(
            bundle.join("Contents").join("Info.plist"),
            vst3_info_plist_contents(&metadata, &bundle),
        )
        .expect("vst3 info plist should be written");

        let discovered = adapter
            .discover_plugins_for_roots(Vst3HostPlatform::Linux, &[root.display().to_string()]);

        assert!(discovered.is_empty());
        let _ = fs::remove_dir_all(root);
    }
}
