use super::*;
use signal_plugin::{
    PluginDescriptor, PluginFeature, PluginFormat, PluginIoLayout, PluginLifecycleContract,
    PluginParameterDescriptor, PluginParameterDomain, PluginParameterFlags,
    PluginProcessingContract, PluginStateContract, PluginTypeId,
};
use std::{collections::BTreeMap, fs, path::Path};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct Lv2ManifestMetadata {
    pub plugin_type_id: String,
    pub plugin_uri: String,
    pub vendor: String,
    pub name: String,
    pub version: String,
    pub audio_inputs: u32,
    pub audio_outputs: u32,
    pub midi_inputs: u32,
    pub midi_outputs: u32,
    pub required_features: Vec<String>,
    pub supported_extensions: Vec<String>,
    pub prepare_fault: Option<Lv2PreparationFaultMode>,
    pub features: Vec<PluginFeature>,
}

pub(super) fn parse_lv2_manifest(bundle_root: &Path) -> Result<Lv2ManifestMetadata, String> {
    let manifest_path = bundle_root.join("manifest.ttl");
    let content = fs::read_to_string(&manifest_path)
        .map_err(|error| format!("failed to read {}: {error}", manifest_path.display()))?;
    parse_lv2_manifest_contents(&content)
}

pub(super) fn discovered_plugin_from_manifest(
    bundle_root: &Path,
    metadata: Lv2ManifestMetadata,
) -> Lv2DiscoveredPluginType {
    let default_io_layout = PluginIoLayout {
        audio_inputs: metadata.audio_inputs as u16,
        audio_outputs: metadata.audio_outputs as u16,
        midi_inputs: metadata.midi_inputs as u16,
        midi_outputs: metadata.midi_outputs as u16,
    };
    let descriptor = lv2_descriptor_from_manifest(&metadata, default_io_layout);
    Lv2DiscoveredPluginType {
        plugin_type_id: PluginTypeId(metadata.plugin_type_id),
        plugin_uri: metadata.plugin_uri,
        bundle_root: bundle_root.to_string_lossy().into_owned(),
        manifest_path: bundle_root
            .join("manifest.ttl")
            .to_string_lossy()
            .into_owned(),
        required_features: metadata.required_features,
        supported_extensions: metadata.supported_extensions,
        prepare_fault: metadata.prepare_fault,
        descriptor,
        default_io_layout,
    }
}

fn parse_lv2_manifest_contents(contents: &str) -> Result<Lv2ManifestMetadata, String> {
    let mut values: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for line in contents.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('@') || line.starts_with('#') {
            continue;
        }
        let Some((key, value)) = line.split_once(char::is_whitespace) else {
            continue;
        };
        let key = key
            .strip_prefix("signal:")
            .unwrap_or(key)
            .trim_end_matches(':')
            .to_string();
        let value = value
            .trim()
            .trim_end_matches(';')
            .trim_end_matches('.')
            .trim()
            .trim_matches('"')
            .to_string();
        if value.is_empty() {
            continue;
        }
        values.entry(key).or_default().push(value);
    }

    let required = |key: &str| -> Result<String, String> {
        values
            .get(key)
            .and_then(|entries| entries.first())
            .cloned()
            .ok_or_else(|| format!("manifest missing signal:{key}"))
    };
    let required_u32 = |key: &str| -> Result<u32, String> {
        required(key)?
            .parse::<u32>()
            .map_err(|error| format!("invalid signal:{key}: {error}"))
    };

    Ok(Lv2ManifestMetadata {
        plugin_type_id: required("plugin_type_id")?,
        plugin_uri: required("plugin_uri")?,
        vendor: required("vendor")?,
        name: required("name")?,
        version: required("version")?,
        audio_inputs: required_u32("audio_inputs")?,
        audio_outputs: required_u32("audio_outputs")?,
        midi_inputs: required_u32("midi_inputs")?,
        midi_outputs: required_u32("midi_outputs")?,
        required_features: values.remove("required_feature").unwrap_or_default(),
        supported_extensions: values.remove("supported_extension").unwrap_or_default(),
        prepare_fault: values
            .remove("prepare_fault")
            .and_then(|mut values| values.drain(..).next())
            .map(parse_prepare_fault_mode)
            .transpose()?,
        features: values
            .remove("feature")
            .unwrap_or_default()
            .into_iter()
            .map(parse_plugin_feature)
            .collect::<Result<Vec<_>, _>>()?,
    })
}

fn parse_plugin_feature(feature: String) -> Result<PluginFeature, String> {
    match feature.as_str() {
        "Instrument" => Ok(PluginFeature::Instrument),
        "Analyzer" => Ok(PluginFeature::Analyzer),
        "AudioEffect" => Ok(PluginFeature::AudioEffect),
        "Utility" => Ok(PluginFeature::Utility),
        other => Err(format!("unsupported signal:feature {other}")),
    }
}

fn parse_prepare_fault_mode(mode: String) -> Result<Lv2PreparationFaultMode, String> {
    match mode.as_str() {
        "worker_unavailable" => Ok(Lv2PreparationFaultMode::WorkerUnavailable),
        "urid_unavailable" => Ok(Lv2PreparationFaultMode::UridUnavailable),
        "patch_unavailable" => Ok(Lv2PreparationFaultMode::PatchUnavailable),
        other => Err(format!("unsupported signal:prepare_fault {other}")),
    }
}

pub(super) fn unsupported_required_features(required_features: &[String]) -> Vec<String> {
    required_features
        .iter()
        .filter(|feature| !is_supported_required_feature(feature))
        .cloned()
        .collect()
}

fn is_supported_required_feature(feature: &str) -> bool {
    matches!(
        feature,
        "http://lv2plug.in/ns/ext/urid#map"
            | "http://lv2plug.in/ns/ext/worker#schedule"
            | "http://lv2plug.in/ns/ext/options#options"
    )
}

fn lv2_descriptor_from_manifest(
    metadata: &Lv2ManifestMetadata,
    io_layout: PluginIoLayout,
) -> PluginDescriptor {
    let mut descriptor = PluginDescriptor::new(
        metadata.plugin_type_id.clone(),
        &metadata.vendor,
        &metadata.name,
        PluginFormat::Lv2,
    )
    .with_version(&metadata.version)
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
        exposes_latency: true,
        exposes_tail: true,
    })
    .with_processing_contract(PluginProcessingContract {
        max_block_frames: 4_096,
        sample_accurate_automation: false,
        accepts_midi: io_layout.midi_inputs > 0,
        accepts_note_events: io_layout.midi_inputs > 0,
        supports_note_expression: false,
        produces_midi: false,
        silence_aware: true,
    })
    .with_lifecycle_contract(PluginLifecycleContract {
        requires_main_thread_for_state: false,
        supports_prepare: true,
        supports_activate: true,
        supports_reset_while_active: false,
    });
    for feature in &metadata.features {
        descriptor = descriptor.with_feature(*feature);
    }
    descriptor
}
