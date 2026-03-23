use signal_plugin::{
    PluginDescriptor, PluginFeature, PluginFormat, PluginIoLayout, PluginLifecycleContract,
    PluginParameterDescriptor, PluginParameterDomain, PluginParameterFlags,
    PluginProcessingContract, PluginStateContract, PluginTypeId,
};

use crate::AuDiscoveredPluginType;

pub(crate) fn au_fixture_bundle_name(plugin_type_id: &str) -> &'static str {
    match plugin_type_id {
        "plugin:au:instrument" => "Signal Instrument.component",
        "plugin:au:multiout-instrument" => "Signal Multi Output Instrument.component",
        "plugin:au:utility" => "Signal Utility.component",
        "plugin:au:bus-fx" => "Signal Bus FX.component",
        _ => "Signal Unknown.component",
    }
}

fn au_default_io_layout(plugin_type_id: &str) -> PluginIoLayout {
    match plugin_type_id {
        "plugin:au:multiout-instrument" => PluginIoLayout {
            audio_inputs: 0,
            audio_outputs: 6,
            midi_inputs: 1,
            midi_outputs: 0,
        },
        "plugin:au:instrument" => PluginIoLayout {
            audio_inputs: 0,
            audio_outputs: 2,
            midi_inputs: 1,
            midi_outputs: 0,
        },
        "plugin:au:bus-fx" => PluginIoLayout {
            audio_inputs: 4,
            audio_outputs: 4,
            midi_inputs: 0,
            midi_outputs: 0,
        },
        "plugin:au:utility" => PluginIoLayout {
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

fn au_fixture_name(plugin_type_id: &str) -> &'static str {
    match plugin_type_id {
        "plugin:au:instrument" => "Signal Instrument AU Plugin",
        "plugin:au:multiout-instrument" => "Signal Multi Output Instrument AU Plugin",
        "plugin:au:utility" => "Signal Utility AU Plugin",
        "plugin:au:bus-fx" => "Signal Bus FX AU Plugin",
        _ => "Signal Generic AU Plugin",
    }
}

fn au_fixture_features(plugin_type_id: &str) -> Vec<PluginFeature> {
    match plugin_type_id {
        "plugin:au:instrument" | "plugin:au:multiout-instrument" => {
            vec![PluginFeature::Instrument, PluginFeature::Analyzer]
        }
        "plugin:au:bus-fx" => vec![PluginFeature::AudioEffect, PluginFeature::Utility],
        "plugin:au:utility" => vec![PluginFeature::AudioEffect, PluginFeature::Utility],
        _ => vec![PluginFeature::Utility],
    }
}

fn au_fixture_descriptor(plugin_type_id: &str, io_layout: PluginIoLayout) -> PluginDescriptor {
    let mut descriptor = PluginDescriptor::new(
        plugin_type_id.to_string(),
        "Signal",
        au_fixture_name(plugin_type_id),
        PluginFormat::Au,
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
    for feature in au_fixture_features(plugin_type_id) {
        descriptor = descriptor.with_feature(feature);
    }
    descriptor
}

pub(crate) fn au_discovered_plugin_type(plugin_type_id: &str) -> Option<AuDiscoveredPluginType> {
    let (component_type, component_subtype, manufacturer_code) = match plugin_type_id {
        "plugin:au:instrument" => ("aumu", "sigi", "sigl"),
        "plugin:au:multiout-instrument" => ("aumu", "sigm", "sigl"),
        "plugin:au:utility" => ("aufx", "sigu", "sigl"),
        "plugin:au:bus-fx" => ("aufx", "sigb", "sigl"),
        _ => return None,
    };
    let default_io_layout = au_default_io_layout(plugin_type_id);
    Some(AuDiscoveredPluginType {
        plugin_type_id: PluginTypeId(plugin_type_id.to_string()),
        component_type: component_type.into(),
        component_subtype: component_subtype.into(),
        manufacturer_code: manufacturer_code.into(),
        bundle_root: format!("fixture://{}", au_fixture_bundle_name(plugin_type_id)),
        descriptor: au_fixture_descriptor(plugin_type_id, default_io_layout),
        default_io_layout,
    })
}
