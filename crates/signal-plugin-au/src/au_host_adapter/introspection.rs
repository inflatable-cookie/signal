use signal_plugin::{
    PluginDescriptor, PluginFeature, PluginFormat, PluginIoLayout, PluginLifecycleContract,
    PluginParameterDescriptor, PluginParameterDomain, PluginParameterFlags,
    PluginProcessingContract, PluginStateContract,
};
use std::{
    io,
    path::{Path, PathBuf},
};

pub(crate) const AU_COMPONENT_INFO_PLIST: &str = "Info.plist";

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

pub(crate) fn read_au_component_metadata(bundle_root: &Path) -> io::Result<AuComponentMetadata> {
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

fn parse_au_component_metadata(metadata_path: &Path) -> io::Result<AuComponentMetadata> {
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
    let component = components
        .first()
        .and_then(plist::Value::as_dictionary)
        .ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                "missing first AudioComponents dictionary in AU Info.plist",
            )
        })?;

    let component_type = required_plist_string(component, "type")?;
    let component_subtype = required_plist_string(component, "subtype")?;
    let manufacturer_code = required_plist_string(component, "manufacturer")?;
    let component_name = required_plist_string(component, "name")?;
    let bundle_identifier = optional_plist_string(root, "CFBundleIdentifier");
    let vendor = optional_plist_string(root, "SignalVendor")
        .or_else(|| {
            component_name
                .split_once(':')
                .map(|(vendor, _)| vendor.trim().to_string())
        })
        .unwrap_or_else(|| "Unknown".into());
    let name = optional_plist_string(root, "SignalDisplayName").unwrap_or_else(|| {
        component_name
            .split_once(':')
            .map(|(_, name)| name.trim().to_string())
            .unwrap_or(component_name.clone())
    });
    let version = optional_plist_string(root, "CFBundleShortVersionString")
        .or_else(|| optional_plist_string(root, "CFBundleVersion"))
        .unwrap_or_else(|| "0.1.0".into());
    let plugin_type_id = optional_plist_string(root, "SignalPluginTypeId").unwrap_or_else(|| {
        derive_plugin_type_id(
            bundle_identifier.as_deref(),
            &component_type,
            &component_subtype,
            &manufacturer_code,
        )
    });
    let audio_inputs = optional_plist_u16(root, "SignalAudioInputs")
        .unwrap_or_else(|| default_audio_inputs(&component_type));
    let audio_outputs = optional_plist_u16(root, "SignalAudioOutputs")
        .unwrap_or_else(|| default_audio_outputs(&component_type));
    let midi_inputs = optional_plist_u16(root, "SignalMidiInputs")
        .unwrap_or_else(|| default_midi_inputs(&component_type));
    let midi_outputs = optional_plist_u16(root, "SignalMidiOutputs").unwrap_or(0);
    let features = optional_feature_list(root, "SignalFeatures")
        .unwrap_or_else(|| default_features(&component_type));
    Ok(AuComponentMetadata {
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
    })
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
    .with_parameters(vec![
        PluginParameterDescriptor {
            parameter_id: 1,
            name: "Output Trim".into(),
            unit: Some("dB".into()),
            domain: PluginParameterDomain::Decibels,
            default_normalized: 0.5,
            min_plain: -24.0,
            max_plain: 24.0,
            flags: PluginParameterFlags::automatable(),
        },
        PluginParameterDescriptor {
            parameter_id: 2,
            name: "Bypass".into(),
            unit: None,
            domain: PluginParameterDomain::Bypass,
            default_normalized: 0.0,
            min_plain: 0.0,
            max_plain: 1.0,
            flags: PluginParameterFlags::bypass(),
        },
    ])
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
