use std::ffi::{c_char, CString};

use clap_sys::{
    ext::{
        audio_ports::{clap_plugin_audio_ports, CLAP_EXT_AUDIO_PORTS},
        note_ports::{clap_plugin_note_ports, CLAP_EXT_NOTE_PORTS},
        params::{clap_plugin_params, CLAP_EXT_PARAMS},
    },
    factory::plugin_factory::clap_plugin_factory,
    plugin::clap_plugin,
};
use signal_plugin::PluginIoLayout;

use super::{
    extensions::{
        audio_buses_from_extension, note_port_count, parameter_descriptors_from_extension,
    },
    host::discovery_host,
};

#[derive(Clone, Debug)]
pub(super) struct DiscoveredPluginIoSummary {
    pub(super) default_io_layout: PluginIoLayout,
    pub(super) audio_buses: Vec<signal_plugin::PluginAudioBusDescriptor>,
    pub(super) parameters: Vec<signal_plugin::PluginParameterDescriptor>,
}

impl Default for DiscoveredPluginIoSummary {
    fn default() -> Self {
        Self {
            default_io_layout: PluginIoLayout {
                audio_inputs: 0,
                audio_outputs: 0,
                midi_inputs: 0,
                midi_outputs: 0,
            },
            audio_buses: Vec::new(),
            parameters: Vec::new(),
        }
    }
}

pub(super) unsafe fn probe_plugin_shape(
    factory: *const clap_plugin_factory,
    plugin_id: &str,
) -> DiscoveredPluginIoSummary {
    let host = discovery_host();
    let plugin_id = match CString::new(plugin_id) {
        Ok(value) => value,
        Err(_) => return DiscoveredPluginIoSummary::default(),
    };
    let Some(create_plugin) = (*factory).create_plugin else {
        return DiscoveredPluginIoSummary::default();
    };
    let plugin = create_plugin(factory, &host, plugin_id.as_ptr());
    if plugin.is_null() {
        return DiscoveredPluginIoSummary::default();
    }

    let init_ok = (*plugin).init.map(|init| init(plugin)).unwrap_or(true);
    if !init_ok {
        if let Some(destroy) = (*plugin).destroy {
            destroy(plugin);
        }
        return DiscoveredPluginIoSummary::default();
    }

    let summary = plugin_io_and_parameter_summary(plugin);
    if let Some(destroy) = (*plugin).destroy {
        destroy(plugin);
    }
    summary
}

pub(super) unsafe fn plugin_io_and_parameter_summary(
    plugin: *const clap_plugin,
) -> DiscoveredPluginIoSummary {
    use signal_plugin::PluginAudioBusDirection;

    let mut summary = DiscoveredPluginIoSummary::default();

    let get_extension = (*plugin).get_extension;
    let audio_ports = get_extension.and_then(|get_extension| {
        let extension = get_extension(plugin, CLAP_EXT_AUDIO_PORTS.as_ptr());
        (!extension.is_null()).then_some(extension.cast::<clap_plugin_audio_ports>())
    });
    if let Some(audio_ports) = audio_ports {
        summary.audio_buses = audio_buses_from_extension(plugin, audio_ports);
        summary.default_io_layout.audio_inputs = summary
            .audio_buses
            .iter()
            .filter(|bus| bus.direction == PluginAudioBusDirection::Input)
            .map(|bus| bus.channels)
            .sum();
        summary.default_io_layout.audio_outputs = summary
            .audio_buses
            .iter()
            .filter(|bus| bus.direction == PluginAudioBusDirection::Output)
            .map(|bus| bus.channels)
            .sum();
    }

    let note_ports = get_extension.and_then(|get_extension| {
        let extension = get_extension(plugin, CLAP_EXT_NOTE_PORTS.as_ptr());
        (!extension.is_null()).then_some(extension.cast::<clap_plugin_note_ports>())
    });
    if let Some(note_ports) = note_ports {
        summary.default_io_layout.midi_inputs = note_port_count(plugin, note_ports, true);
        summary.default_io_layout.midi_outputs = note_port_count(plugin, note_ports, false);
    }

    let params = get_extension.and_then(|get_extension| {
        let extension = get_extension(plugin, CLAP_EXT_PARAMS.as_ptr());
        (!extension.is_null()).then_some(extension.cast::<clap_plugin_params>())
    });
    if let Some(params) = params {
        summary.parameters = parameter_descriptors_from_extension(plugin, params);
    }

    summary
}

pub(super) unsafe fn plugin_supports_extension(
    factory: *const clap_plugin_factory,
    plugin_id: &str,
    extension_id: *const c_char,
) -> bool {
    let host = discovery_host();
    let plugin_id = match CString::new(plugin_id) {
        Ok(value) => value,
        Err(_) => return false,
    };
    let Some(create_plugin) = (*factory).create_plugin else {
        return false;
    };
    let plugin = create_plugin(factory, &host, plugin_id.as_ptr());
    if plugin.is_null() {
        return false;
    }

    let init_ok = (*plugin).init.map(|init| init(plugin)).unwrap_or(true);
    let supported = init_ok
        && (*plugin)
            .get_extension
            .map(|get_extension| !get_extension(plugin, extension_id).is_null())
            .unwrap_or(false);
    if let Some(destroy) = (*plugin).destroy {
        destroy(plugin);
    }
    supported
}
