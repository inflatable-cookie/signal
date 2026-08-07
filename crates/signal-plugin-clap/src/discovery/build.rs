use std::{
    ffi::{c_char, CStr},
    path::Path,
};

use clap_sys::{
    ext::{latency::CLAP_EXT_LATENCY, state::CLAP_EXT_STATE, tail::CLAP_EXT_TAIL},
    factory::plugin_factory::clap_plugin_factory,
    plugin::clap_plugin_descriptor,
    plugin_features::{
        CLAP_PLUGIN_FEATURE_ANALYZER, CLAP_PLUGIN_FEATURE_AUDIO_EFFECT,
        CLAP_PLUGIN_FEATURE_INSTRUMENT, CLAP_PLUGIN_FEATURE_NOTE_EFFECT,
        CLAP_PLUGIN_FEATURE_UTILITY,
    },
};
use signal_plugin::{
    PluginDescriptor, PluginFeature, PluginFormat, PluginLifecycleContract, PluginParameterDomain,
    PluginProcessingContract, PluginStateContract, PluginTypeId,
};

use crate::ClapDiscoveredPluginType;

use super::{
    probe::{plugin_supports_extension, probe_plugin_shape, DiscoveredPluginIoSummary},
    util::cstr_ptr_to_string,
};

pub(super) unsafe fn build_discovered_plugin(
    factory: *const clap_plugin_factory,
    descriptor_ptr: *const clap_plugin_descriptor,
    library_path: &Path,
    probe_capabilities: bool,
) -> Option<ClapDiscoveredPluginType> {
    let descriptor = &*descriptor_ptr;
    let plugin_id = cstr_ptr_to_string(descriptor.id)?;
    let vendor = cstr_ptr_to_string(descriptor.vendor).unwrap_or_else(|| "Unknown".into());
    let name = cstr_ptr_to_string(descriptor.name).unwrap_or_else(|| plugin_id.clone());
    let version = cstr_ptr_to_string(descriptor.version);
    let features = clap_features(descriptor.features);
    let io_and_params = if probe_capabilities {
        probe_plugin_shape(factory, &plugin_id)
    } else {
        DiscoveredPluginIoSummary::default()
    };

    let mut plugin_descriptor =
        PluginDescriptor::new(plugin_id.clone(), vendor, name, PluginFormat::Clap);
    if let Some(version) = version {
        plugin_descriptor = plugin_descriptor.with_version(version);
    }
    for feature in &features {
        plugin_descriptor = plugin_descriptor.with_feature(*feature);
    }
    let state_contract = if probe_capabilities {
        PluginStateContract {
            supports_snapshot: plugin_supports_extension(
                factory,
                &plugin_id,
                CLAP_EXT_STATE.as_ptr(),
            ),
            supports_reset: true,
            supports_bypass: io_and_params
                .parameters
                .iter()
                .any(|parameter| parameter.domain == PluginParameterDomain::Bypass),
            exposes_latency: plugin_supports_extension(
                factory,
                &plugin_id,
                CLAP_EXT_LATENCY.as_ptr(),
            ),
            exposes_tail: plugin_supports_extension(factory, &plugin_id, CLAP_EXT_TAIL.as_ptr()),
        }
    } else {
        // Without instantiating the plugin nothing about its state surface
        // is known; report the conservative minimum.
        PluginStateContract {
            supports_snapshot: false,
            supports_reset: false,
            supports_bypass: false,
            exposes_latency: false,
            exposes_tail: false,
        }
    };
    plugin_descriptor = plugin_descriptor
        .with_audio_buses(io_and_params.audio_buses.clone())
        .with_parameters(io_and_params.parameters.clone())
        .with_state_contract(state_contract)
        .with_processing_contract(PluginProcessingContract {
            max_block_frames: 4096,
            sample_accurate_automation: !io_and_params.parameters.is_empty(),
            accepts_midi: io_and_params.default_io_layout.midi_inputs > 0,
            accepts_note_events: io_and_params.default_io_layout.midi_inputs > 0,
            supports_note_expression: io_and_params.default_io_layout.midi_inputs > 0,
            produces_midi: io_and_params.default_io_layout.midi_outputs > 0,
            silence_aware: true,
        })
        .with_lifecycle_contract(PluginLifecycleContract {
            requires_main_thread_for_state: false,
            supports_prepare: true,
            supports_activate: true,
            supports_reset_while_active: true,
        });

    Some(ClapDiscoveredPluginType {
        plugin_type_id: PluginTypeId(plugin_id),
        library_path: library_path.to_string_lossy().to_string(),
        descriptor: plugin_descriptor,
        default_io_layout: io_and_params.default_io_layout,
    })
}

fn clap_features(features_ptr: *const *const c_char) -> Vec<PluginFeature> {
    if features_ptr.is_null() {
        return Vec::new();
    }
    let mut features = Vec::new();
    let mut offset = 0usize;
    loop {
        let feature_ptr = unsafe { *features_ptr.add(offset) };
        if feature_ptr.is_null() {
            break;
        }
        let feature = unsafe { CStr::from_ptr(feature_ptr) };
        let mapped = if feature == CLAP_PLUGIN_FEATURE_AUDIO_EFFECT {
            Some(PluginFeature::AudioEffect)
        } else if feature == CLAP_PLUGIN_FEATURE_INSTRUMENT {
            Some(PluginFeature::Instrument)
        } else if feature == CLAP_PLUGIN_FEATURE_ANALYZER {
            Some(PluginFeature::Analyzer)
        } else if feature == CLAP_PLUGIN_FEATURE_UTILITY {
            Some(PluginFeature::Utility)
        } else if feature == CLAP_PLUGIN_FEATURE_NOTE_EFFECT {
            Some(PluginFeature::NoteEffect)
        } else {
            None
        };
        if let Some(mapped) = mapped {
            if !features.contains(&mapped) {
                features.push(mapped);
            }
        }
        offset += 1;
    }
    features
}
