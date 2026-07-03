use signal_plugin::{
    PluginDescriptor, PluginFeature, PluginFormat, PluginIoLayout, PluginLifecycleContract,
    PluginProcessingContract, PluginStateContract,
};
use std::{
    io,
    path::{Path, PathBuf},
};

pub(crate) const AU_COMPONENT_INFO_PLIST: &str = "Info.plist";

/// Split the AudioComponent registry naming convention `"Vendor: Name"`
/// into its vendor and display-name halves. Names without a colon report
/// no vendor. Shared by the plist scan and the registry discovery path.
pub(crate) fn split_component_name(component_name: &str) -> (Option<String>, String) {
    match component_name.split_once(':') {
        Some((vendor, name)) => (Some(vendor.trim().to_string()), name.trim().to_string()),
        None => (None, component_name.trim().to_string()),
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct AuComponentMetadata {
    pub(crate) plugin_type_id: String,
    pub(crate) component_type: String,
    pub(crate) component_subtype: String,
    pub(crate) manufacturer_code: String,
    pub(crate) vendor: String,
    pub(crate) name: String,
    pub(crate) version: String,
    pub(crate) audio_inputs: u16,
    pub(crate) audio_outputs: u16,
    pub(crate) midi_inputs: u16,
    pub(crate) midi_outputs: u16,
    pub(crate) features: Vec<PluginFeature>,
}

/// Read every AudioComponent entry declared by a `.component` bundle's
/// Info.plist (a bundle may register several components).
pub(crate) fn read_au_component_metadata_list(
    bundle_root: &Path,
) -> io::Result<Vec<AuComponentMetadata>> {
    let metadata_path = candidate_metadata_paths(bundle_root)
        .into_iter()
        .find(|path| path.is_file())
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "missing AU Info.plist"))?;
    parse_au_component_metadata(&metadata_path)
}

fn candidate_metadata_paths(bundle_root: &Path) -> Vec<PathBuf> {
    vec![
        bundle_root.join("Contents").join(AU_COMPONENT_INFO_PLIST),
        bundle_root.join(AU_COMPONENT_INFO_PLIST),
    ]
}

fn parse_au_component_metadata(metadata_path: &Path) -> io::Result<Vec<AuComponentMetadata>> {
    let plist = plist::Value::from_file(metadata_path).map_err(|error| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("invalid AU Info.plist: {error}"),
        )
    })?;
    let root = plist.as_dictionary().ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            "AU Info.plist root is not a dictionary",
        )
    })?;
    let components = root
        .get("AudioComponents")
        .and_then(plist::Value::as_array)
        .ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                "missing AudioComponents entry in AU Info.plist",
            )
        })?;
    let component_dicts: Vec<&plist::Dictionary> = components
        .iter()
        .filter_map(plist::Value::as_dictionary)
        .collect();
    if component_dicts.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "missing AudioComponents dictionary in AU Info.plist",
        ));
    }

    // The root-level Signal* overrides describe THE component; they only
    // apply when the bundle declares exactly one (multi-component bundles
    // derive everything per entry — the historical first-entry-only bug).
    let single_component = component_dicts.len() == 1;
    let mut parsed = Vec::with_capacity(component_dicts.len());
    for component in component_dicts {
        let component_type = required_plist_string(component, "type")?;
        let component_subtype = required_plist_string(component, "subtype")?;
        let manufacturer_code = required_plist_string(component, "manufacturer")?;
        let component_name = required_plist_string(component, "name")?;
        let bundle_identifier = optional_plist_string(root, "CFBundleIdentifier");
        let (split_vendor, split_name) = split_component_name(&component_name);
        let vendor = optional_plist_string(root, "SignalVendor")
            .filter(|_| single_component)
            .or(split_vendor)
            .unwrap_or_else(|| "Unknown".into());
        let name = optional_plist_string(root, "SignalDisplayName")
            .filter(|_| single_component)
            .unwrap_or(split_name);
        let version = optional_plist_string(root, "CFBundleShortVersionString")
            .or_else(|| optional_plist_string(root, "CFBundleVersion"))
            .unwrap_or_else(|| "0.1.0".into());
        let plugin_type_id = optional_plist_string(root, "SignalPluginTypeId")
            .filter(|_| single_component)
            .unwrap_or_else(|| {
                derive_plugin_type_id(
                    bundle_identifier.as_deref(),
                    &component_type,
                    &component_subtype,
                    &manufacturer_code,
                )
            });
        let audio_inputs = optional_plist_u16(root, "SignalAudioInputs")
            .filter(|_| single_component)
            .unwrap_or_else(|| default_audio_inputs(&component_type));
        let audio_outputs = optional_plist_u16(root, "SignalAudioOutputs")
            .filter(|_| single_component)
            .unwrap_or_else(|| default_audio_outputs(&component_type));
        let midi_inputs = optional_plist_u16(root, "SignalMidiInputs")
            .filter(|_| single_component)
            .unwrap_or_else(|| default_midi_inputs(&component_type));
        let midi_outputs = optional_plist_u16(root, "SignalMidiOutputs")
            .filter(|_| single_component)
            .unwrap_or(0);
        let features = optional_feature_list(root, "SignalFeatures")
            .filter(|_| single_component)
            .unwrap_or_else(|| default_features(&component_type));
        parsed.push(AuComponentMetadata {
            plugin_type_id,
            component_type,
            component_subtype,
            manufacturer_code,
            vendor,
            name,
            version,
            audio_inputs,
            audio_outputs,
            midi_inputs,
            midi_outputs,
            features,
        });
    }
    Ok(parsed)
}

pub(crate) fn metadata_io_layout(metadata: &AuComponentMetadata) -> PluginIoLayout {
    PluginIoLayout {
        audio_inputs: metadata.audio_inputs,
        audio_outputs: metadata.audio_outputs,
        midi_inputs: metadata.midi_inputs,
        midi_outputs: metadata.midi_outputs,
    }
}

pub(crate) fn metadata_descriptor(metadata: &AuComponentMetadata) -> PluginDescriptor {
    let io_layout = metadata_io_layout(metadata);
    let mut descriptor = PluginDescriptor::new(
        metadata.plugin_type_id.clone(),
        metadata.vendor.clone(),
        metadata.name.clone(),
        PluginFormat::Au,
    )
    .with_version(metadata.version.as_str())
    .with_audio_buses(io_layout.main_audio_buses())
    // Scan-time parameter inventory is intentionally EMPTY (decision-4
    // parity with VST3/g11.031): the real inventory is enumerated at load
    // through the hosting FFI, never faked at discovery time.
    .with_state_contract(PluginStateContract {
        supports_snapshot: true,
        supports_reset: true,
        supports_bypass: true,
        exposes_latency: false,
        exposes_tail: true,
    })
    .with_processing_contract(PluginProcessingContract {
        max_block_frames: 4_096,
        sample_accurate_automation: false,
        accepts_midi: io_layout.midi_inputs > 0,
        accepts_note_events: io_layout.midi_inputs > 0,
        supports_note_expression: io_layout.midi_inputs > 0,
        produces_midi: false,
        silence_aware: false,
    })
    .with_lifecycle_contract(PluginLifecycleContract {
        requires_main_thread_for_state: true,
        supports_prepare: true,
        supports_activate: true,
        supports_reset_while_active: false,
    });
    for feature in &metadata.features {
        descriptor = descriptor.with_feature(*feature);
    }
    descriptor
}

fn parse_feature_list(value: &str) -> io::Result<Vec<PluginFeature>> {
    value
        .split(',')
        .map(str::trim)
        .filter(|item| !item.is_empty())
        .map(|item| match item {
            "Instrument" => Ok(PluginFeature::Instrument),
            "Analyzer" => Ok(PluginFeature::Analyzer),
            "AudioEffect" => Ok(PluginFeature::AudioEffect),
            "Utility" => Ok(PluginFeature::Utility),
            other => Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("unknown AU feature `{other}`"),
            )),
        })
        .collect()
}

fn required_plist_string(dict: &plist::Dictionary, key: &str) -> io::Result<String> {
    optional_plist_string(dict, key).ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("missing `{key}` in AU Info.plist"),
        )
    })
}

fn optional_plist_string(dict: &plist::Dictionary, key: &str) -> Option<String> {
    dict.get(key).and_then(|value| match value {
        plist::Value::String(string) => Some(string.clone()),
        plist::Value::Integer(integer) => Some(integer.to_string()),
        _ => None,
    })
}

fn optional_plist_u16(dict: &plist::Dictionary, key: &str) -> Option<u16> {
    dict.get(key).and_then(|value| match value {
        plist::Value::Integer(integer) => integer.as_unsigned()?.try_into().ok(),
        plist::Value::String(string) => string.parse::<u16>().ok(),
        _ => None,
    })
}

fn optional_feature_list(dict: &plist::Dictionary, key: &str) -> Option<Vec<PluginFeature>> {
    dict.get(key).and_then(|value| match value {
        plist::Value::Array(items) => {
            let joined = items
                .iter()
                .filter_map(plist::Value::as_string)
                .collect::<Vec<_>>()
                .join(",");
            parse_feature_list(&joined).ok()
        }
        plist::Value::String(string) => parse_feature_list(string).ok(),
        _ => None,
    })
}

fn derive_plugin_type_id(
    bundle_identifier: Option<&str>,
    component_type: &str,
    component_subtype: &str,
    manufacturer_code: &str,
) -> String {
    match bundle_identifier {
        Some(identifier) if !identifier.is_empty() => {
            format!(
                "plugin:au:{}:{}:{}:{}",
                identifier, component_type, component_subtype, manufacturer_code
            )
        }
        _ => format!(
            "plugin:au:{}:{}:{}",
            component_type, component_subtype, manufacturer_code
        ),
    }
}

fn default_audio_inputs(component_type: &str) -> u16 {
    if component_type.eq_ignore_ascii_case("aumu") {
        0
    } else {
        2
    }
}

fn default_audio_outputs(_component_type: &str) -> u16 {
    2
}

fn default_midi_inputs(component_type: &str) -> u16 {
    if component_type.eq_ignore_ascii_case("aumu") {
        1
    } else {
        0
    }
}

fn default_features(component_type: &str) -> Vec<PluginFeature> {
    if component_type.eq_ignore_ascii_case("aumu") {
        vec![PluginFeature::Instrument, PluginFeature::Analyzer]
    } else {
        vec![PluginFeature::AudioEffect, PluginFeature::Utility]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    const MULTI_COMPONENT_PLIST: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>AudioComponents</key>
  <array>
    <dict>
      <key>manufacturer</key>
      <string>sigl</string>
      <key>name</key>
      <string>Signal: Multi FX A</string>
      <key>subtype</key>
      <string>mfxa</string>
      <key>type</key>
      <string>aufx</string>
      <key>version</key>
      <integer>1</integer>
    </dict>
    <dict>
      <key>manufacturer</key>
      <string>sigl</string>
      <key>name</key>
      <string>Signal: Multi Synth B</string>
      <key>subtype</key>
      <string>msyb</string>
      <key>type</key>
      <string>aumu</string>
      <key>version</key>
      <integer>1</integer>
    </dict>
  </array>
  <key>CFBundleIdentifier</key>
  <string>com.signal.multi</string>
  <key>CFBundleShortVersionString</key>
  <string>2.3.4</string>
</dict>
</plist>
"#;

    #[test]
    fn multi_component_plists_yield_one_metadata_per_component() {
        let root = std::env::temp_dir().join(format!(
            "signal-au-multi-plist-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos(),
        ));
        let bundle = root.join("Signal Multi.component");
        fs::create_dir_all(bundle.join("Contents")).expect("bundle dirs");
        fs::write(
            bundle.join("Contents").join("Info.plist"),
            MULTI_COMPONENT_PLIST,
        )
        .expect("plist written");

        let parsed = read_au_component_metadata_list(&bundle).expect("plist parses");
        assert_eq!(parsed.len(), 2, "one metadata per AudioComponents entry");

        let effect = &parsed[0];
        assert_eq!(effect.component_type, "aufx");
        assert_eq!(effect.component_subtype, "mfxa");
        assert_eq!(effect.vendor, "Signal");
        assert_eq!(effect.name, "Multi FX A");
        assert_eq!(effect.version, "2.3.4");
        assert_eq!(
            effect.plugin_type_id,
            "plugin:au:com.signal.multi:aufx:mfxa:sigl"
        );
        assert_eq!(effect.audio_inputs, 2);

        let synth = &parsed[1];
        assert_eq!(synth.component_type, "aumu");
        assert_eq!(synth.component_subtype, "msyb");
        assert_eq!(synth.name, "Multi Synth B");
        assert_eq!(synth.audio_inputs, 0);
        assert_eq!(synth.midi_inputs, 1);
        assert_ne!(effect.plugin_type_id, synth.plugin_type_id);

        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn component_name_splitter_handles_missing_vendor() {
        assert_eq!(
            split_component_name("Apple: AUDelay"),
            (Some("Apple".to_string()), "AUDelay".to_string()),
        );
        assert_eq!(
            split_component_name("Bare Name"),
            (None, "Bare Name".to_string())
        );
    }
}
