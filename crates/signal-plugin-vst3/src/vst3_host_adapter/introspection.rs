use signal_plugin::{
    PluginDescriptor, PluginFeature, PluginFormat, PluginIoLayout, PluginLifecycleContract,
    PluginParameterDescriptor, PluginParameterDomain, PluginParameterFlags,
    PluginProcessingContract, PluginStateContract,
};
use std::{
    fs, io,
    path::{Path, PathBuf},
};

pub(crate) const VST3_MODULE_METADATA_FILE: &str = "signal-vst3-module.txt";
pub(crate) const VST3_FACTORY_METADATA_FILE: &str = "signal-vst3-factory.txt";

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Vst3ModuleMetadata {
    pub(crate) plugin_type_id: String,
    pub(crate) class_id: String,
    pub(crate) controller_class_id: Option<String>,
    pub(crate) category: String,
    pub(crate) vendor: String,
    pub(crate) name: String,
    pub(crate) version: String,
    pub(crate) audio_inputs: u16,
    pub(crate) audio_outputs: u16,
    pub(crate) midi_inputs: u16,
    pub(crate) midi_outputs: u16,
    pub(crate) features: Vec<PluginFeature>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Vst3FactoryClass {
    pub(crate) role: Vst3FactoryClassRole,
    pub(crate) class_id: String,
    pub(crate) category: String,
    pub(crate) name: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Vst3FactoryClassRole {
    Component,
    Controller,
}

pub(crate) fn read_vst3_module_metadata(bundle_root: &Path) -> io::Result<Vst3ModuleMetadata> {
    let metadata_path = candidate_metadata_paths(bundle_root)
        .into_iter()
        .find(|path| path.is_file())
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "missing VST3 module metadata"))?;
    parse_vst3_module_metadata(&fs::read_to_string(metadata_path)?)
}

pub(crate) fn read_vst3_factory_classes(bundle_root: &Path) -> io::Result<Vec<Vst3FactoryClass>> {
    let metadata_path = candidate_factory_paths(bundle_root)
        .into_iter()
        .find(|path| path.is_file())
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "missing VST3 factory metadata"))?;
    parse_vst3_factory_metadata(&fs::read_to_string(metadata_path)?)
}

fn candidate_metadata_paths(bundle_root: &Path) -> Vec<PathBuf> {
    vec![
        bundle_root
            .join("Contents")
            .join("Resources")
            .join(VST3_MODULE_METADATA_FILE),
        bundle_root.join(VST3_MODULE_METADATA_FILE),
    ]
}

fn candidate_factory_paths(bundle_root: &Path) -> Vec<PathBuf> {
    vec![
        bundle_root
            .join("Contents")
            .join("Resources")
            .join(VST3_FACTORY_METADATA_FILE),
        bundle_root.join(VST3_FACTORY_METADATA_FILE),
    ]
}

fn parse_vst3_module_metadata(input: &str) -> io::Result<Vst3ModuleMetadata> {
    let mut plugin_type_id = None;
    let mut class_id = None;
    let mut controller_class_id = None;
    let mut category = None;
    let mut vendor = None;
    let mut name = None;
    let mut version = None;
    let mut audio_inputs = None;
    let mut audio_outputs = None;
    let mut midi_inputs = None;
    let mut midi_outputs = None;
    let mut features = Vec::new();

    for line in input.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let Some((key, value)) = line.split_once('=') else {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("invalid VST3 metadata line `{line}`"),
            ));
        };
        let value = value.trim();
        match key.trim() {
            "plugin_type_id" => plugin_type_id = Some(value.to_string()),
            "class_id" => class_id = Some(value.to_string()),
            "controller_class_id" => {
                controller_class_id = if value.is_empty() || value.eq_ignore_ascii_case("none") {
                    None
                } else {
                    Some(value.to_string())
                };
            }
            "category" => category = Some(value.to_string()),
            "vendor" => vendor = Some(value.to_string()),
            "name" => name = Some(value.to_string()),
            "version" => version = Some(value.to_string()),
            "audio_inputs" => audio_inputs = Some(parse_u16_field("audio_inputs", value)?),
            "audio_outputs" => audio_outputs = Some(parse_u16_field("audio_outputs", value)?),
            "midi_inputs" => midi_inputs = Some(parse_u16_field("midi_inputs", value)?),
            "midi_outputs" => midi_outputs = Some(parse_u16_field("midi_outputs", value)?),
            "features" => features = parse_feature_list(value)?,
            _ => {}
        }
    }

    Ok(Vst3ModuleMetadata {
        plugin_type_id: plugin_type_id.ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                "missing plugin_type_id metadata",
            )
        })?,
        class_id: class_id.ok_or_else(|| {
            io::Error::new(io::ErrorKind::InvalidData, "missing class_id metadata")
        })?,
        controller_class_id,
        category: category.ok_or_else(|| {
            io::Error::new(io::ErrorKind::InvalidData, "missing category metadata")
        })?,
        vendor: vendor
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "missing vendor metadata"))?,
        name: name
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "missing name metadata"))?,
        version: version.ok_or_else(|| {
            io::Error::new(io::ErrorKind::InvalidData, "missing version metadata")
        })?,
        audio_inputs: audio_inputs.ok_or_else(|| {
            io::Error::new(io::ErrorKind::InvalidData, "missing audio_inputs metadata")
        })?,
        audio_outputs: audio_outputs.ok_or_else(|| {
            io::Error::new(io::ErrorKind::InvalidData, "missing audio_outputs metadata")
        })?,
        midi_inputs: midi_inputs.ok_or_else(|| {
            io::Error::new(io::ErrorKind::InvalidData, "missing midi_inputs metadata")
        })?,
        midi_outputs: midi_outputs.ok_or_else(|| {
            io::Error::new(io::ErrorKind::InvalidData, "missing midi_outputs metadata")
        })?,
        features,
    })
}

fn parse_vst3_factory_metadata(input: &str) -> io::Result<Vec<Vst3FactoryClass>> {
    let mut classes = Vec::new();
    for line in input.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let Some((key, value)) = line.split_once('=') else {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("invalid VST3 factory line `{line}`"),
            ));
        };
        let role = match key.trim() {
            "component" => Vst3FactoryClassRole::Component,
            "controller" => Vst3FactoryClassRole::Controller,
            _ => continue,
        };
        let mut fields = value.split('|').map(str::trim);
        let class_id = fields
            .next()
            .filter(|value| !value.is_empty())
            .ok_or_else(|| {
                io::Error::new(io::ErrorKind::InvalidData, "missing VST3 factory class id")
            })?;
        let category = fields
            .next()
            .filter(|value| !value.is_empty())
            .ok_or_else(|| {
                io::Error::new(io::ErrorKind::InvalidData, "missing VST3 factory category")
            })?;
        let name = fields
            .next()
            .filter(|value| !value.is_empty())
            .ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::InvalidData,
                    "missing VST3 factory class name",
                )
            })?;
        classes.push(Vst3FactoryClass {
            role,
            class_id: class_id.to_string(),
            category: category.to_string(),
            name: name.to_string(),
        });
    }
    if classes.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "missing VST3 factory classes",
        ));
    }
    Ok(classes)
}

pub(crate) fn metadata_io_layout(metadata: &Vst3ModuleMetadata) -> PluginIoLayout {
    PluginIoLayout {
        audio_inputs: metadata.audio_inputs,
        audio_outputs: metadata.audio_outputs,
        midi_inputs: metadata.midi_inputs,
        midi_outputs: metadata.midi_outputs,
    }
}

pub(crate) fn metadata_descriptor(metadata: &Vst3ModuleMetadata) -> PluginDescriptor {
    let io_layout = metadata_io_layout(metadata);
    let mut descriptor = PluginDescriptor::new(
        metadata.plugin_type_id.clone(),
        metadata.vendor.clone(),
        metadata.name.clone(),
        PluginFormat::Vst3,
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
        descriptor = descriptor.with_feature(feature.clone());
    }
    descriptor
}

fn parse_u16_field(field: &str, value: &str) -> io::Result<u16> {
    value.parse::<u16>().map_err(|_| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("invalid {field} metadata `{value}`"),
        )
    })
}

fn parse_feature_list(value: &str) -> io::Result<Vec<PluginFeature>> {
    let mut features = Vec::new();
    for raw in value.split(',') {
        let feature = match raw.trim() {
            "" => continue,
            "Instrument" => PluginFeature::Instrument,
            "Analyzer" => PluginFeature::Analyzer,
            "AudioEffect" => PluginFeature::AudioEffect,
            "Utility" => PluginFeature::Utility,
            other => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("unsupported VST3 feature metadata `{other}`"),
                ));
            }
        };
        if !features.contains(&feature) {
            features.push(feature);
        }
    }
    Ok(features)
}
