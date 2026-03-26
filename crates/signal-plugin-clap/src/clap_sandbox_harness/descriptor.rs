use signal_ipc::PluginDescriptorPayload;
use signal_plugin::{
    PluginDescriptor, PluginFeature, PluginFormat, PluginIoLayout, PluginLifecycleContract,
    PluginParameterDescriptor, PluginParameterDomain, PluginParameterFlags,
    PluginProcessingContract, PluginStateContract, PluginTypeId,
};

use crate::ClapDiscoveredPluginType;

fn clap_default_io_layout(plugin_type_id: &str) -> Option<PluginIoLayout> {
    match plugin_type_id {
        "plugin:clap:default" | "plugin:clap:test" => Some(PluginIoLayout {
            audio_inputs: 2,
            audio_outputs: 2,
            midi_inputs: 1,
            midi_outputs: 1,
        }),
        "plugin:clap:server" => Some(PluginIoLayout {
            audio_inputs: 2,
            audio_outputs: 2,
            midi_inputs: 1,
            midi_outputs: 0,
        }),
        "plugin:clap:sandbox" => Some(PluginIoLayout {
            audio_inputs: 2,
            audio_outputs: 2,
            midi_inputs: 1,
            midi_outputs: 1,
        }),
        plugin_type_id if plugin_type_id.starts_with("plugin:clap:") => Some(PluginIoLayout {
            audio_inputs: 2,
            audio_outputs: 2,
            midi_inputs: 1,
            midi_outputs: 0,
        }),
        _ => None,
    }
}

fn clap_descriptor_name(plugin_type_id: &str) -> &'static str {
    match plugin_type_id {
        "plugin:clap:default" => "Signal Default CLAP Plugin",
        "plugin:clap:server" => "Signal Server CLAP Plugin",
        "plugin:clap:sandbox" => "Signal Sandbox CLAP Plugin",
        "plugin:clap:test" => "Signal Test CLAP Plugin",
        _ => "Signal Generic CLAP Plugin",
    }
}

fn clap_descriptor_features(plugin_type_id: &str) -> Vec<PluginFeature> {
    match plugin_type_id {
        "plugin:clap:sandbox" => vec![PluginFeature::AudioEffect, PluginFeature::Utility],
        _ => vec![PluginFeature::AudioEffect],
    }
}

pub(crate) fn clap_fixture_descriptor(
    plugin_type_id: &str,
    io_layout: PluginIoLayout,
) -> PluginDescriptor {
    let mut descriptor = PluginDescriptor::new(
        plugin_type_id.to_string(),
        "Signal",
        clap_descriptor_name(plugin_type_id),
        PluginFormat::Clap,
    )
    .with_version("0.1.0")
    .with_audio_buses(io_layout.main_audio_buses())
    .with_parameters(vec![
        PluginParameterDescriptor {
            parameter_id: 4_096,
            name: "Gain Automation".into(),
            unit: Some("normalized".into()),
            domain: PluginParameterDomain::GenericNormalized,
            default_normalized: 0.5,
            min_plain: 0.0,
            max_plain: 1.0,
            flags: PluginParameterFlags::automatable(),
        },
        PluginParameterDescriptor {
            parameter_id: 0,
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
        sample_accurate_automation: true,
        accepts_midi: io_layout.midi_inputs > 0,
        accepts_note_events: true,
        supports_note_expression: true,
        produces_midi: io_layout.midi_outputs > 0,
        silence_aware: true,
    })
    .with_lifecycle_contract(PluginLifecycleContract {
        requires_main_thread_for_state: false,
        supports_prepare: true,
        supports_activate: true,
        supports_reset_while_active: true,
    });
    for feature in clap_descriptor_features(plugin_type_id) {
        descriptor = descriptor.with_feature(feature);
    }
    descriptor
}

pub(crate) fn clap_discovered_plugin_type(
    plugin_type_id: &str,
) -> Option<ClapDiscoveredPluginType> {
    let default_io_layout = clap_default_io_layout(plugin_type_id)?;
    Some(ClapDiscoveredPluginType {
        plugin_type_id: PluginTypeId(plugin_type_id.to_string()),
        descriptor: clap_fixture_descriptor(plugin_type_id, default_io_layout),
        default_io_layout,
    })
}

pub(crate) fn descriptor_payload(descriptor: &PluginDescriptor) -> PluginDescriptorPayload {
    PluginDescriptorPayload {
        plugin_id: descriptor.plugin_id.clone(),
        vendor: descriptor.vendor.clone(),
        name: descriptor.name.clone(),
        format: match descriptor.format {
            PluginFormat::Clap => "clap",
            PluginFormat::Vst3 => "vst3",
            PluginFormat::Au => "au",
            PluginFormat::Lv2 => "lv2",
            PluginFormat::Native => "native",
        }
        .into(),
    }
}
