use signal_plugin::{
    PluginDescriptor, PluginFeature, PluginFormat, PluginIoLayout, PluginLifecycleContract,
    PluginParameterDescriptor, PluginParameterDomain, PluginParameterFlags,
    PluginProcessingContract, PluginStateContract, PluginTypeId,
};

use crate::Lv2DiscoveredPluginType;

pub(crate) fn lv2_fixture_bundle_name(plugin_type_id: &str) -> &'static str {
    match plugin_type_id {
        "plugin:lv2:linux-synth" => "Signal Linux Synth.lv2",
        "plugin:lv2:multiout-instrument" => "Signal Multi Output Instrument.lv2",
        "plugin:lv2:utility" => "Signal Utility.lv2",
        "plugin:lv2:bus-fx" => "Signal Bus FX.lv2",
        _ => "Signal Unknown.lv2",
    }
}

fn lv2_default_io_layout(plugin_type_id: &str) -> PluginIoLayout {
    match plugin_type_id {
        "plugin:lv2:multiout-instrument" => PluginIoLayout {
            audio_inputs: 0,
            audio_outputs: 6,
            midi_inputs: 1,
            midi_outputs: 0,
        },
        "plugin:lv2:linux-synth" => PluginIoLayout {
            audio_inputs: 0,
            audio_outputs: 2,
            midi_inputs: 1,
            midi_outputs: 0,
        },
        "plugin:lv2:bus-fx" => PluginIoLayout {
            audio_inputs: 4,
            audio_outputs: 4,
            midi_inputs: 0,
            midi_outputs: 0,
        },
        "plugin:lv2:utility" => PluginIoLayout {
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

fn lv2_fixture_name(plugin_type_id: &str) -> &'static str {
    match plugin_type_id {
        "plugin:lv2:linux-synth" => "Signal Linux Synth LV2 Plugin",
        "plugin:lv2:multiout-instrument" => "Signal Multi Output Instrument LV2 Plugin",
        "plugin:lv2:utility" => "Signal Utility LV2 Plugin",
        "plugin:lv2:bus-fx" => "Signal Bus FX LV2 Plugin",
        _ => "Signal Generic LV2 Plugin",
    }
}

fn lv2_fixture_uri(plugin_type_id: &str) -> &'static str {
    match plugin_type_id {
        "plugin:lv2:linux-synth" => "https://signal.dev/plugins/lv2/linux-synth",
        "plugin:lv2:multiout-instrument" => "https://signal.dev/plugins/lv2/multiout-instrument",
        "plugin:lv2:utility" => "https://signal.dev/plugins/lv2/utility",
        "plugin:lv2:bus-fx" => "https://signal.dev/plugins/lv2/bus-fx",
        _ => "https://signal.dev/plugins/lv2/unknown",
    }
}

fn lv2_fixture_required_features(plugin_type_id: &str) -> Vec<String> {
    match plugin_type_id {
        "plugin:lv2:linux-synth" | "plugin:lv2:multiout-instrument" => vec![
            "http://lv2plug.in/ns/ext/urid#map".into(),
            "http://lv2plug.in/ns/ext/worker#schedule".into(),
        ],
        "plugin:lv2:bus-fx" => vec!["http://lv2plug.in/ns/ext/urid#map".into()],
        "plugin:lv2:utility" => vec!["http://lv2plug.in/ns/ext/options#options".into()],
        _ => Vec::new(),
    }
}

fn lv2_fixture_supported_extensions(plugin_type_id: &str) -> Vec<String> {
    match plugin_type_id {
        "plugin:lv2:linux-synth" | "plugin:lv2:multiout-instrument" => vec![
            "http://lv2plug.in/ns/ext/patch#Message".into(),
            "http://lv2plug.in/ns/ext/state#state".into(),
        ],
        "plugin:lv2:bus-fx" => vec!["http://lv2plug.in/ns/ext/patch#Message".into()],
        "plugin:lv2:utility" => vec!["http://lv2plug.in/ns/ext/options#options".into()],
        _ => Vec::new(),
    }
}

fn lv2_fixture_features(plugin_type_id: &str) -> Vec<PluginFeature> {
    match plugin_type_id {
        "plugin:lv2:linux-synth" | "plugin:lv2:multiout-instrument" => {
            vec![PluginFeature::Instrument, PluginFeature::Analyzer]
        }
        "plugin:lv2:bus-fx" => vec![PluginFeature::AudioEffect, PluginFeature::Utility],
        "plugin:lv2:utility" => vec![PluginFeature::AudioEffect, PluginFeature::Utility],
        _ => vec![PluginFeature::Utility],
    }
}

fn lv2_fixture_descriptor(plugin_type_id: &str, io_layout: PluginIoLayout) -> PluginDescriptor {
    let mut descriptor = PluginDescriptor::new(
        plugin_type_id.to_string(),
        "Signal",
        lv2_fixture_name(plugin_type_id),
        PluginFormat::Lv2,
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
    for feature in lv2_fixture_features(plugin_type_id) {
        descriptor = descriptor.with_feature(feature);
    }
    descriptor
}

pub(crate) fn lv2_discovered_plugin_type(plugin_type_id: &str) -> Option<Lv2DiscoveredPluginType> {
    match plugin_type_id {
        "plugin:lv2:linux-synth"
        | "plugin:lv2:multiout-instrument"
        | "plugin:lv2:utility"
        | "plugin:lv2:bus-fx" => {
            let default_io_layout = lv2_default_io_layout(plugin_type_id);
            Some(Lv2DiscoveredPluginType {
                plugin_type_id: PluginTypeId(plugin_type_id.to_string()),
                plugin_uri: lv2_fixture_uri(plugin_type_id).into(),
                bundle_root: format!("fixture://{}", lv2_fixture_bundle_name(plugin_type_id)),
                manifest_path: format!(
                    "fixture://{}/manifest.ttl",
                    lv2_fixture_bundle_name(plugin_type_id)
                ),
                required_features: lv2_fixture_required_features(plugin_type_id),
                supported_extensions: lv2_fixture_supported_extensions(plugin_type_id),
                descriptor: lv2_fixture_descriptor(plugin_type_id, default_io_layout),
                default_io_layout,
            })
        }
        _ => None,
    }
}
