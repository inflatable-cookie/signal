use signal_plugin::{
    PluginDescriptor, PluginFeature, PluginFormat, PluginIoLayout, PluginLifecycleContract,
    PluginParameterDescriptor, PluginParameterDomain, PluginParameterFlags,
    PluginProcessingContract, PluginStateContract, PluginTypeId,
};

use crate::Vst3DiscoveredPluginType;

pub(crate) fn vst3_fixture_bundle_name(plugin_type_id: &str) -> &'static str {
    match plugin_type_id {
        "plugin:vst3:instrument" => "Signal Instrument.vst3",
        "plugin:vst3:multiout-instrument" => "Signal Multi Output Instrument.vst3",
        "plugin:vst3:linux-synth" => "Signal Linux Synth.vst3",
        "plugin:vst3:utility" => "Signal Utility.vst3",
        "plugin:vst3:bus-fx" => "Signal Bus FX.vst3",
        _ => "Signal Unknown.vst3",
    }
}

fn vst3_default_io_layout(plugin_type_id: &str) -> PluginIoLayout {
    match plugin_type_id {
        "plugin:vst3:multiout-instrument" => PluginIoLayout {
            audio_inputs: 0,
            audio_outputs: 6,
            midi_inputs: 1,
            midi_outputs: 0,
        },
        "plugin:vst3:instrument" | "plugin:vst3:linux-synth" => PluginIoLayout {
            audio_inputs: 0,
            audio_outputs: 2,
            midi_inputs: 1,
            midi_outputs: 0,
        },
        "plugin:vst3:bus-fx" => PluginIoLayout {
            audio_inputs: 4,
            audio_outputs: 4,
            midi_inputs: 0,
            midi_outputs: 0,
        },
        "plugin:vst3:utility" => PluginIoLayout {
            audio_inputs: 2,
            audio_outputs: 2,
            midi_inputs: 0,
            midi_outputs: 0,
        },
        _ => PluginIoLayout {
            audio_inputs: 2,
            audio_outputs: 2,
            midi_inputs: 0,
            midi_outputs: 0,
        },
    }
}

fn vst3_fixture_name(plugin_type_id: &str) -> &'static str {
    match plugin_type_id {
        "plugin:vst3:instrument" => "Signal Instrument VST3 Plugin",
        "plugin:vst3:multiout-instrument" => "Signal Multi Output Instrument VST3 Plugin",
        "plugin:vst3:linux-synth" => "Signal Linux Synth VST3 Plugin",
        "plugin:vst3:utility" => "Signal Utility VST3 Plugin",
        "plugin:vst3:bus-fx" => "Signal Bus FX VST3 Plugin",
        _ => "Signal Generic VST3 Plugin",
    }
}

fn vst3_fixture_features(plugin_type_id: &str) -> Vec<PluginFeature> {
    match plugin_type_id {
        "plugin:vst3:instrument"
        | "plugin:vst3:multiout-instrument"
        | "plugin:vst3:linux-synth" => {
            vec![PluginFeature::Instrument, PluginFeature::Analyzer]
        }
        "plugin:vst3:bus-fx" => vec![PluginFeature::AudioEffect, PluginFeature::Utility],
        "plugin:vst3:utility" => vec![PluginFeature::AudioEffect, PluginFeature::Utility],
        _ => vec![PluginFeature::Utility],
    }
}

fn vst3_fixture_descriptor(plugin_type_id: &str, io_layout: PluginIoLayout) -> PluginDescriptor {
    let mut descriptor = PluginDescriptor::new(
        plugin_type_id.to_string(),
        "Signal",
        vst3_fixture_name(plugin_type_id),
        PluginFormat::Vst3,
    )
    .with_version("0.1.0")
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
    for feature in vst3_fixture_features(plugin_type_id) {
        descriptor = descriptor.with_feature(feature);
    }
    descriptor
}

pub(crate) fn vst3_discovered_plugin_type(
    plugin_type_id: &str,
) -> Option<Vst3DiscoveredPluginType> {
    let (class_id, controller_class_id, category) = match plugin_type_id {
        "plugin:vst3:instrument" => (
            "7E1D8F8A4D874D56A2C44DE250100001",
            Some("7E1D8F8A4D874D56A2C44DE250100002"),
            "Instrument",
        ),
        "plugin:vst3:multiout-instrument" => (
            "7E1D8F8A4D874D56A2C44DE250100011",
            Some("7E1D8F8A4D874D56A2C44DE250100012"),
            "Instrument",
        ),
        "plugin:vst3:linux-synth" => (
            "7E1D8F8A4D874D56A2C44DE250100101",
            Some("7E1D8F8A4D874D56A2C44DE250100102"),
            "Instrument",
        ),
        "plugin:vst3:utility" => (
            "7E1D8F8A4D874D56A2C44DE250100201",
            Some("7E1D8F8A4D874D56A2C44DE250100202"),
            "Fx",
        ),
        "plugin:vst3:bus-fx" => (
            "7E1D8F8A4D874D56A2C44DE250100211",
            Some("7E1D8F8A4D874D56A2C44DE250100212"),
            "Fx",
        ),
        _ => return None,
    };
    let default_io_layout = vst3_default_io_layout(plugin_type_id);
    Some(Vst3DiscoveredPluginType {
        plugin_type_id: PluginTypeId(plugin_type_id.to_string()),
        class_id: class_id.into(),
        controller_class_id: controller_class_id.map(str::to_string),
        category: category.into(),
        module_root: format!("fixture://{}", vst3_fixture_bundle_name(plugin_type_id)),
        descriptor: vst3_fixture_descriptor(plugin_type_id, default_io_layout),
        default_io_layout,
    })
}
