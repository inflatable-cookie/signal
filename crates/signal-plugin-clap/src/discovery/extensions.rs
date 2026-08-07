use std::mem::MaybeUninit;

use clap_sys::ext::{
    audio_ports::{clap_audio_port_info, clap_plugin_audio_ports, CLAP_AUDIO_PORT_IS_MAIN},
    note_ports::{clap_note_port_info, clap_plugin_note_ports},
    params::{
        clap_param_info, clap_plugin_params, CLAP_PARAM_IS_AUTOMATABLE, CLAP_PARAM_IS_BYPASS,
        CLAP_PARAM_IS_HIDDEN, CLAP_PARAM_IS_MODULATABLE, CLAP_PARAM_IS_READONLY,
        CLAP_PARAM_IS_STEPPED,
    },
};
use signal_plugin::{
    PluginAudioBusDescriptor, PluginAudioBusDirection, PluginParameterDescriptor,
    PluginParameterDomain, PluginParameterFlags,
};

use super::util::clap_char_buffer_to_string;

/// Audio bus list alias shared with the hosting module.
pub(crate) type PluginAudioBusDescriptorList = Vec<PluginAudioBusDescriptor>;

pub(crate) unsafe fn audio_buses_from_extension(
    plugin: *const clap_sys::plugin::clap_plugin,
    extension: *const clap_plugin_audio_ports,
) -> PluginAudioBusDescriptorList {
    let Some(count) = (*extension).count else {
        return Vec::new();
    };
    let Some(get) = (*extension).get else {
        return Vec::new();
    };

    let mut buses = Vec::new();
    for is_input in [true, false] {
        for index in 0..count(plugin, is_input) {
            let mut info = MaybeUninit::<clap_audio_port_info>::zeroed();
            if get(plugin, index, is_input, info.as_mut_ptr()) {
                let info = info.assume_init();
                buses.push(PluginAudioBusDescriptor {
                    bus_id: format!("clap:{}:{}", if is_input { "in" } else { "out" }, info.id),
                    name: clap_char_buffer_to_string(&info.name),
                    direction: if is_input {
                        PluginAudioBusDirection::Input
                    } else {
                        PluginAudioBusDirection::Output
                    },
                    channels: info.channel_count as u16,
                    is_main: info.flags & CLAP_AUDIO_PORT_IS_MAIN != 0,
                    active: true,
                });
            }
        }
    }
    buses
}

pub(crate) unsafe fn parameter_descriptors_from_extension(
    plugin: *const clap_sys::plugin::clap_plugin,
    extension: *const clap_plugin_params,
) -> Vec<PluginParameterDescriptor> {
    let Some(count) = (*extension).count else {
        return Vec::new();
    };
    let Some(get_info) = (*extension).get_info else {
        return Vec::new();
    };

    let mut parameters = Vec::new();
    for index in 0..count(plugin) {
        let mut info = MaybeUninit::<clap_param_info>::zeroed();
        if get_info(plugin, index, info.as_mut_ptr()) {
            let info = info.assume_init();
            let flags = info.flags;
            // CLAP param info reports min/max/default all in the PLAIN
            // range; normalize the default into the descriptor's 0..=1
            // vocabulary (degenerate ranges normalize to 0).
            let range = info.max_value - info.min_value;
            let default_normalized = if range > f64::EPSILON {
                (((info.default_value - info.min_value) / range) as f32).clamp(0.0, 1.0)
            } else {
                0.0
            };
            // CLAP stepped parameters take integer plain values: the step
            // count is the integer span of the range (g12.013). CLAP param
            // info carries no unit string.
            let step_count = if flags & CLAP_PARAM_IS_STEPPED != 0 {
                Some((range.round() as u32).max(1))
            } else {
                None
            };
            parameters.push(PluginParameterDescriptor {
                parameter_id: info.id,
                name: clap_char_buffer_to_string(&info.name),
                unit: None,
                domain: if flags & CLAP_PARAM_IS_BYPASS != 0 {
                    PluginParameterDomain::Bypass
                } else {
                    PluginParameterDomain::GenericNormalized
                },
                default_normalized,
                min_plain: info.min_value as f32,
                max_plain: info.max_value as f32,
                step_count,
                flags: PluginParameterFlags {
                    automatable: flags & CLAP_PARAM_IS_AUTOMATABLE != 0,
                    modulatable: flags & CLAP_PARAM_IS_MODULATABLE != 0,
                    supports_gesture: flags & CLAP_PARAM_IS_AUTOMATABLE != 0,
                    stepped: flags & CLAP_PARAM_IS_STEPPED != 0,
                    hidden: flags & CLAP_PARAM_IS_HIDDEN != 0,
                    read_only: flags & CLAP_PARAM_IS_READONLY != 0,
                },
            });
        }
    }
    parameters
}

pub(super) unsafe fn note_port_count(
    plugin: *const clap_sys::plugin::clap_plugin,
    extension: *const clap_plugin_note_ports,
    is_input: bool,
) -> u16 {
    let Some(count) = (*extension).count else {
        return 0;
    };
    let Some(get) = (*extension).get else {
        return 0;
    };

    let mut discovered = 0;
    for index in 0..count(plugin, is_input) {
        let mut info = MaybeUninit::<clap_note_port_info>::zeroed();
        if get(plugin, index, is_input, info.as_mut_ptr()) {
            discovered += 1;
        }
    }
    discovered
}
