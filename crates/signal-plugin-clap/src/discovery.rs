use std::{
    ffi::{c_char, c_void, CStr, CString},
    mem::MaybeUninit,
    path::{Path, PathBuf},
    ptr,
};

use crate::hosting::LoadedClapEntry;

use clap_sys::{
    entry::clap_plugin_entry,
    ext::{
        audio_ports::{
            clap_audio_port_info, clap_plugin_audio_ports, CLAP_AUDIO_PORT_IS_MAIN,
            CLAP_EXT_AUDIO_PORTS,
        },
        latency::CLAP_EXT_LATENCY,
        note_ports::{clap_note_port_info, clap_plugin_note_ports, CLAP_EXT_NOTE_PORTS},
        params::{
            clap_param_info, clap_plugin_params, CLAP_EXT_PARAMS, CLAP_PARAM_IS_AUTOMATABLE,
            CLAP_PARAM_IS_BYPASS, CLAP_PARAM_IS_HIDDEN, CLAP_PARAM_IS_MODULATABLE,
            CLAP_PARAM_IS_READONLY, CLAP_PARAM_IS_STEPPED,
        },
        state::CLAP_EXT_STATE,
        tail::CLAP_EXT_TAIL,
    },
    factory::plugin_factory::{clap_plugin_factory, CLAP_PLUGIN_FACTORY_ID},
    host::clap_host,
    plugin::clap_plugin_descriptor,
    plugin_features::{
        CLAP_PLUGIN_FEATURE_ANALYZER, CLAP_PLUGIN_FEATURE_AUDIO_EFFECT,
        CLAP_PLUGIN_FEATURE_INSTRUMENT, CLAP_PLUGIN_FEATURE_NOTE_EFFECT,
        CLAP_PLUGIN_FEATURE_UTILITY,
    },
    version::clap_version,
};
use signal_plugin::{
    PluginAudioBusDescriptor, PluginAudioBusDirection, PluginDescriptor, PluginFeature,
    PluginFormat, PluginIoLayout, PluginLifecycleContract, PluginParameterDescriptor,
    PluginParameterDomain, PluginParameterFlags, PluginProcessingContract, PluginStateContract,
    PluginTypeId,
};

use crate::ClapDiscoveredPluginType;

#[derive(Clone, Debug)]
struct DiscoveredPluginIoSummary {
    default_io_layout: PluginIoLayout,
    audio_buses: Vec<PluginAudioBusDescriptor>,
    parameters: Vec<PluginParameterDescriptor>,
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

pub(crate) fn discover_clap_plugins_for_roots(
    roots: &[String],
    probe_capabilities: bool,
) -> Vec<ClapDiscoveredPluginType> {
    roots
        .iter()
        .flat_map(|root| scan_clap_root(root, probe_capabilities))
        .collect()
}

fn scan_clap_root(root: &str, probe_capabilities: bool) -> Vec<ClapDiscoveredPluginType> {
    let root = expand_home(root);
    let Ok(metadata) = std::fs::metadata(&root) else {
        return Vec::new();
    };

    if metadata.is_file() {
        return discover_from_clap_library(&root, probe_capabilities).unwrap_or_default();
    }

    collect_clap_candidates(&root)
        .into_iter()
        .flat_map(|candidate| {
            discover_from_clap_library(&candidate, probe_capabilities).unwrap_or_default()
        })
        .collect()
}

fn collect_clap_candidates(root: &Path) -> Vec<PathBuf> {
    let mut candidates = Vec::new();
    if let Ok(entries) = std::fs::read_dir(root) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                if path
                    .extension()
                    .and_then(|extension| extension.to_str())
                    .is_some_and(|extension| extension.eq_ignore_ascii_case("clap"))
                {
                    if let Some(bundle_binary) = clap_bundle_binary(&path) {
                        candidates.push(bundle_binary);
                    }
                } else {
                    candidates.extend(collect_clap_candidates(&path));
                }
            } else if path
                .extension()
                .and_then(|extension| extension.to_str())
                .is_some_and(|extension| extension.eq_ignore_ascii_case("clap"))
            {
                candidates.push(path);
            }
        }
    }
    candidates
}

fn clap_bundle_binary(bundle_root: &Path) -> Option<PathBuf> {
    let macos_root = bundle_root.join("Contents").join("MacOS");
    if let Ok(entries) = std::fs::read_dir(&macos_root) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                return Some(path);
            }
        }
    }
    None
}

fn discover_from_clap_library(
    library_path: &Path,
    probe_capabilities: bool,
) -> Option<Vec<ClapDiscoveredPluginType>> {
    // Entry loading is shared with hosting (`hosting::LoadedClapEntry`):
    // discovery and the sandbox child dlopen and initialize CLAP entries
    // through the same path. The entry deinitializes on drop.
    let entry = LoadedClapEntry::load(library_path).ok()?;
    Some(unsafe { discover_from_entry(entry.entry(), library_path, probe_capabilities) })
}

unsafe fn discover_from_entry(
    entry: clap_plugin_entry,
    library_path: &Path,
    probe_capabilities: bool,
) -> Vec<ClapDiscoveredPluginType> {
    let Some(get_factory) = entry.get_factory else {
        return Vec::new();
    };
    let factory_ptr = get_factory(CLAP_PLUGIN_FACTORY_ID.as_ptr());
    if factory_ptr.is_null() {
        return Vec::new();
    }
    let factory = factory_ptr.cast::<clap_plugin_factory>();
    let Some(get_plugin_count) = (*factory).get_plugin_count else {
        return Vec::new();
    };
    let Some(get_plugin_descriptor) = (*factory).get_plugin_descriptor else {
        return Vec::new();
    };

    let mut discovered = Vec::new();
    for index in 0..get_plugin_count(factory) {
        let descriptor_ptr = get_plugin_descriptor(factory, index);
        if descriptor_ptr.is_null() {
            continue;
        }
        if let Some(plugin) =
            build_discovered_plugin(factory, descriptor_ptr, library_path, probe_capabilities)
        {
            discovered.push(plugin);
        }
    }
    discovered
}

unsafe fn build_discovered_plugin(
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

unsafe fn probe_plugin_shape(
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

unsafe fn plugin_io_and_parameter_summary(
    plugin: *const clap_sys::plugin::clap_plugin,
) -> DiscoveredPluginIoSummary {
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

unsafe fn note_port_count(
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
            parameters.push(PluginParameterDescriptor {
                parameter_id: info.id,
                name: clap_char_buffer_to_string(&info.name),
                unit: None,
                domain: if flags & CLAP_PARAM_IS_BYPASS != 0 {
                    PluginParameterDomain::Bypass
                } else {
                    PluginParameterDomain::GenericNormalized
                },
                default_normalized: info.default_value as f32,
                min_plain: info.min_value as f32,
                max_plain: info.max_value as f32,
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

unsafe fn plugin_supports_extension(
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

fn discovery_host() -> clap_host {
    clap_host {
        clap_version: clap_version {
            major: 1,
            minor: 0,
            revision: 0,
        },
        host_data: ptr::null_mut(),
        name: c"Signal Discovery Host".as_ptr(),
        vendor: c"Signal".as_ptr(),
        url: c"https://signal.dev".as_ptr(),
        version: c"0.1.0".as_ptr(),
        get_extension: Some(discovery_host_get_extension),
        request_restart: Some(discovery_host_request_restart),
        request_process: Some(discovery_host_request_process),
        request_callback: Some(discovery_host_request_callback),
    }
}

unsafe extern "C" fn discovery_host_get_extension(
    _host: *const clap_host,
    _extension_id: *const c_char,
) -> *const c_void {
    ptr::null()
}

unsafe extern "C" fn discovery_host_request_restart(_host: *const clap_host) {}
unsafe extern "C" fn discovery_host_request_process(_host: *const clap_host) {}
unsafe extern "C" fn discovery_host_request_callback(_host: *const clap_host) {}

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

fn clap_char_buffer_to_string(buffer: &[c_char]) -> String {
    let len = buffer
        .iter()
        .position(|value| *value == 0)
        .unwrap_or(buffer.len());
    let bytes = buffer[..len]
        .iter()
        .map(|value| *value as u8)
        .collect::<Vec<_>>();
    String::from_utf8_lossy(&bytes).trim().to_string()
}

fn cstr_ptr_to_string(ptr: *const c_char) -> Option<String> {
    if ptr.is_null() {
        None
    } else {
        Some(
            unsafe { CStr::from_ptr(ptr) }
                .to_string_lossy()
                .into_owned(),
        )
    }
}

fn expand_home(root: &str) -> PathBuf {
    if let Some(stripped) = root.strip_prefix("~/") {
        if let Some(home) = std::env::var_os("HOME") {
            return PathBuf::from(home).join(stripped);
        }
    }
    PathBuf::from(root)
}
