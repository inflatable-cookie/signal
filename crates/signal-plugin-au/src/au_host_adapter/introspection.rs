use signal_plugin::{
    PluginDescriptor, PluginFeature, PluginFormat, PluginIoLayout, PluginLifecycleContract,
    PluginParameterDescriptor, PluginParameterDomain, PluginParameterFlags,
    PluginProcessingContract, PluginStateContract,
};
use std::{
    fs, io,
    path::{Path, PathBuf},
};

pub(crate) const AU_COMPONENT_METADATA_FILE: &str = "signal-au-component.txt";

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
    pub(crate) init_failure: Option<String>,
    pub(crate) bus_layout_failure: Option<String>,
    pub(crate) render_context_failure: Option<String>,
}

pub(crate) fn read_au_component_metadata(bundle_root: &Path) -> io::Result<AuComponentMetadata> {
    let metadata_path = candidate_metadata_paths(bundle_root)
        .into_iter()
        .find(|path| path.is_file())
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "missing AU component metadata"))?;
    parse_au_component_metadata(&fs::read_to_string(metadata_path)?)
}

fn candidate_metadata_paths(bundle_root: &Path) -> Vec<PathBuf> {
    vec![
        bundle_root
            .join("Contents")
            .join("Resources")
            .join(AU_COMPONENT_METADATA_FILE),
        bundle_root.join(AU_COMPONENT_METADATA_FILE),
    ]
}

fn parse_au_component_metadata(input: &str) -> io::Result<AuComponentMetadata> {
    let mut plugin_type_id = None;
    let mut component_type = None;
    let mut component_subtype = None;
    let mut manufacturer_code = None;
    let mut vendor = None;
    let mut name = None;
    let mut version = None;
    let mut audio_inputs = None;
    let mut audio_outputs = None;
    let mut midi_inputs = None;
    let mut midi_outputs = None;
    let mut features = Vec::new();
    let mut init_failure = None;
    let mut bus_layout_failure = None;
    let mut render_context_failure = None;

    for line in input.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let Some((key, value)) = line.split_once('=') else {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("invalid AU metadata line `{line}`"),
            ));
        };
        let value = value.trim();
        match key.trim() {
            "plugin_type_id" => plugin_type_id = Some(value.to_string()),
            "component_type" => component_type = Some(value.to_string()),
            "component_subtype" => component_subtype = Some(value.to_string()),
            "manufacturer_code" => manufacturer_code = Some(value.to_string()),
            "vendor" => vendor = Some(value.to_string()),
            "name" => name = Some(value.to_string()),
            "version" => version = Some(value.to_string()),
            "audio_inputs" => audio_inputs = Some(parse_u16_field("audio_inputs", value)?),
            "audio_outputs" => audio_outputs = Some(parse_u16_field("audio_outputs", value)?),
            "midi_inputs" => midi_inputs = Some(parse_u16_field("midi_inputs", value)?),
            "midi_outputs" => midi_outputs = Some(parse_u16_field("midi_outputs", value)?),
            "features" => features = parse_feature_list(value)?,
            "init_failure" => init_failure = Some(value.to_string()),
            "bus_layout_failure" => bus_layout_failure = Some(value.to_string()),
            "render_context_failure" => render_context_failure = Some(value.to_string()),
            _ => {}
        }
    }

    Ok(AuComponentMetadata {
        plugin_type_id: plugin_type_id.ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                "missing plugin_type_id metadata",
            )
        })?,
        component_type: component_type.ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                "missing component_type metadata",
            )
        })?,
        component_subtype: component_subtype.ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                "missing component_subtype metadata",
            )
        })?,
        manufacturer_code: manufacturer_code.ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                "missing manufacturer_code metadata",
            )
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
        init_failure,
        bus_layout_failure,
        render_context_failure,
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

fn parse_u16_field(field: &str, value: &str) -> io::Result<u16> {
    value.parse::<u16>().map_err(|_| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("invalid {field} value `{value}`"),
        )
    })
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
