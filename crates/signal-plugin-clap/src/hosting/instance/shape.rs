use std::ptr;

use clap_sys::{
    ext::audio_ports::{clap_plugin_audio_ports, CLAP_EXT_AUDIO_PORTS},
    ext::gui::{clap_plugin_gui, CLAP_EXT_GUI},
    plugin::clap_plugin,
};
use signal_plugin::{PluginAudioBusDirection, PluginParameterDescriptor};

use crate::discovery::{
    audio_buses_from_extension, parameter_descriptors_from_extension, PluginAudioBusDescriptorList,
};

use super::layout::ClapHostedPortLayout;

/// Enumerate a live instance's parameters and main-bus port layout.
pub(crate) unsafe fn instance_shape(
    plugin: *const clap_plugin,
) -> (
    Vec<PluginParameterDescriptor>,
    ClapHostedPortLayout,
    PluginAudioBusDescriptorList,
) {
    let mut parameters = Vec::new();
    let mut buses = Vec::new();
    let mut layout = ClapHostedPortLayout {
        main_input_channels: 0,
        main_output_channels: 0,
    };
    let Some(get_extension) = (*plugin).get_extension else {
        return (parameters, layout, buses);
    };

    let params_extension = get_extension(plugin, clap_sys::ext::params::CLAP_EXT_PARAMS.as_ptr());
    if !params_extension.is_null() {
        parameters = parameter_descriptors_from_extension(
            plugin,
            params_extension.cast::<clap_sys::ext::params::clap_plugin_params>(),
        );
    }

    let audio_ports = get_extension(plugin, CLAP_EXT_AUDIO_PORTS.as_ptr());
    if !audio_ports.is_null() {
        buses = audio_buses_from_extension(plugin, audio_ports.cast::<clap_plugin_audio_ports>());
        for bus in &buses {
            if !bus.is_main {
                continue;
            }
            match bus.direction {
                PluginAudioBusDirection::Input => layout.main_input_channels = bus.channels,
                PluginAudioBusDirection::Output => layout.main_output_channels = bus.channels,
            }
        }
    }
    (parameters, layout, buses)
}

/// Query the plugin's `clap.gui` extension and whether it supports this
/// platform's embedded window API. Runs at load, on the lifecycle thread.
pub(crate) unsafe fn gui_shape(plugin: *const clap_plugin) -> (*const clap_plugin_gui, bool) {
    let Some(get_extension) = (*plugin).get_extension else {
        return (ptr::null(), false);
    };
    let extension = get_extension(plugin, CLAP_EXT_GUI.as_ptr());
    if extension.is_null() {
        return (ptr::null(), false);
    }
    let gui = extension.cast::<clap_plugin_gui>();
    let api_supported = (*gui)
        .is_api_supported
        .map(|is_api_supported| is_api_supported(plugin, crate::gui::WINDOW_API.as_ptr(), false))
        .unwrap_or(false);
    (gui, api_supported)
}
