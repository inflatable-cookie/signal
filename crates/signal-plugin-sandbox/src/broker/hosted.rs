//! Format-specific hosted plugin wrappers used by the sandbox broker.

use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use std::thread::JoinHandle;

use crate::child_gui::ChildEditorSpec;
use signal_ipc::{MappedSharedMemoryRegion, PluginAudioBlockLayout};
use signal_plugin::{PluginEvent, PluginParameterDescriptor};
use signal_plugin_au::{AuHostedInstance, AuProcessSession};
use signal_plugin_clap::{ClapHostedInstance, ClapProcessSession};
use signal_plugin_lv2::{Lv2HostedInstance, Lv2ProcessSession};
use signal_plugin_vst3::{Vst3HostedInstance, Vst3ProcessSession};

pub(crate) struct AttachedRegion {
    pub(crate) region: MappedSharedMemoryRegion,
    pub(crate) lease_id: String,
    pub(crate) processed_blocks: u64,
}

/// The child audio thread: owns the process session and the shm view while
/// processing runs.
pub(crate) struct AudioThread {
    pub(crate) stop: Arc<AtomicBool>,
    pub(crate) join: JoinHandle<()>,
}

/// Shared-memory audio bridge for an activated plugin instance.
pub(crate) struct ActivatedAudio {
    pub(crate) region: MappedSharedMemoryRegion,
    pub(crate) layout: PluginAudioBlockLayout,
    pub(crate) thread: Option<AudioThread>,
}

/// Format-selected hosted instance (g11.031, AU per g11.032, LV2 per
/// g11.033): the broker infers the plugin format from the library path
/// extension (`.clap` / `.vst3` / `.component` / `.lv2`), keeping the
/// stdio wire format unchanged. Directory extensions are lexical — `.lv2`
/// bundle directories ride the same check VST3 bundle directories do.
/// `load-plugin`'s second argument is the format-native load key: the raw
/// plugin id for CLAP, the component class CID hex for VST3, the fourcc
/// triple `{type}:{subtype}:{manufacturer}` for AU, the bare plugin URI
/// for LV2 (the child re-parses the bundle TTL to rebuild the port model,
/// paralleling AU's rebuild-from-load-key). AU registry entries carry the
/// sentinel path `au-registry.component` — never opened; the child
/// rebuilds the `AudioComponentDescription` from the load key and
/// resolves it through the system registry.
pub(crate) enum HostedPluginInstance {
    Clap(ClapHostedInstance),
    Vst3(Vst3HostedInstance),
    Au(AuHostedInstance),
    /// Boxed: the LV2 instance embeds its TTL port model and feature set,
    /// making it much larger than the pointer-sized COM/FFI handles.
    Lv2(Box<Lv2HostedInstance>),
}

/// Format-selected raw process session for the child audio thread.
pub(crate) enum HostedProcessSession {
    Clap(ClapProcessSession),
    Vst3(Vst3ProcessSession),
    Au(AuProcessSession),
    Lv2(Lv2ProcessSession),
}

impl HostedPluginInstance {
    /// Load `load_key` from `library_path`, selecting the format by path
    /// extension. Unknown extensions fail with a stable token.
    pub(crate) fn load(library_path: &str, load_key: &str) -> Result<Self, String> {
        let path = std::path::Path::new(library_path);
        match path
            .extension()
            .and_then(|extension| extension.to_str())
            .map(str::to_ascii_lowercase)
            .as_deref()
        {
            Some("clap") => ClapHostedInstance::load(path, load_key)
                .map(Self::Clap)
                .map_err(|error| error.token),
            Some("vst3") => Vst3HostedInstance::load(path, load_key)
                .map(Self::Vst3)
                .map_err(|error| error.token),
            Some("component") => AuHostedInstance::load(path, load_key)
                .map(Self::Au)
                .map_err(|error| error.token),
            Some("lv2") => Lv2HostedInstance::load(path, load_key)
                .map(|instance| Self::Lv2(Box::new(instance)))
                .map_err(|error| error.token),
            _ => Err("unsupported_library_extension".to_string()),
        }
    }

    pub(crate) fn parameters(&self) -> Vec<PluginParameterDescriptor> {
        match self {
            Self::Clap(instance) => instance.parameters().to_vec(),
            Self::Vst3(instance) => instance.parameters().to_vec(),
            Self::Au(instance) => instance.parameters().to_vec(),
            Self::Lv2(instance) => instance.parameters().to_vec(),
        }
    }

    /// Main-bus channel counts (input, output).
    pub(crate) fn main_ports(&self) -> (u16, u16) {
        match self {
            Self::Clap(instance) => {
                let layout = instance.port_layout();
                (layout.main_input_channels, layout.main_output_channels)
            }
            Self::Vst3(instance) => {
                let layout = instance.port_layout();
                (layout.main_input_channels, layout.main_output_channels)
            }
            Self::Au(instance) => {
                let layout = instance.port_layout();
                (layout.main_input_channels, layout.main_output_channels)
            }
            Self::Lv2(instance) => {
                let layout = instance.port_layout();
                (layout.main_input_channels, layout.main_output_channels)
            }
        }
    }

    pub(crate) fn is_supported_stereo_processor(&self) -> bool {
        let (inputs, outputs) = self.main_ports();
        matches!((inputs, outputs), (0 | 2, 2))
    }

    pub(crate) fn activate(
        &mut self,
        sample_rate_hz: f64,
        min_frames: u32,
        max_frames: u32,
    ) -> Result<(), String> {
        match self {
            Self::Clap(instance) => instance
                .activate(sample_rate_hz, min_frames, max_frames)
                .map_err(|error| error.token),
            Self::Vst3(instance) => instance
                .activate(sample_rate_hz, min_frames, max_frames)
                .map_err(|error| error.token),
            Self::Au(instance) => instance
                .activate(sample_rate_hz, min_frames, max_frames)
                .map_err(|error| error.token),
            Self::Lv2(instance) => instance
                .activate(sample_rate_hz, min_frames, max_frames)
                .map_err(|error| error.token),
        }
    }

    pub(crate) fn deactivate(&mut self) -> Result<(), String> {
        match self {
            Self::Clap(instance) => instance.deactivate().map_err(|error| error.token),
            Self::Vst3(instance) => instance.deactivate().map_err(|error| error.token),
            Self::Au(instance) => instance.deactivate().map_err(|error| error.token),
            Self::Lv2(instance) => instance.deactivate().map_err(|error| error.token),
        }
    }

    /// Queue one normalized 0..1 parameter write on the format's set path
    /// (g12.023): CLAP process in-events, VST3 IParameterChanges +
    /// controller sync, AU AudioUnitSetParameter, LV2 control-slot write
    /// before `run()`. Delivery is block-boundary on the child's audio
    /// thread.
    pub(crate) fn set_parameter_normalized(
        &mut self,
        parameter_id: u32,
        normalized: f32,
    ) -> Result<(), String> {
        match self {
            Self::Clap(instance) => instance
                .set_parameter_normalized(parameter_id, normalized)
                .map_err(|error| error.token),
            Self::Vst3(instance) => instance
                .set_parameter_normalized(parameter_id, normalized)
                .map_err(|error| error.token),
            Self::Au(instance) => instance
                .set_parameter_normalized(parameter_id, normalized)
                .map_err(|error| error.token),
            Self::Lv2(instance) => instance
                .set_parameter_normalized(parameter_id, normalized)
                .map_err(|error| error.token),
        }
    }

    /// Editor open spec for the child-owned window path (g13.027 Batch 1).
    /// CLAP is first-class; VST3/AU child editors are recorded follow-up
    /// state (their adapters still assume in-process host services), LV2
    /// native GUIs are excluded by packet posture. Errors are stable
    /// tokens.
    pub(crate) fn child_editor_spec(&self) -> Result<ChildEditorSpec, String> {
        match self {
            Self::Clap(instance) => instance
                .gui_raw_parts()
                .map(ChildEditorSpec::Clap)
                .ok_or_else(|| "gui_unsupported".to_string()),
            Self::Vst3(_) => Err("editor_format_unported:vst3".to_string()),
            Self::Au(_) => Err("editor_format_unported:au".to_string()),
            Self::Lv2(_) => Err("editor_format_excluded:lv2".to_string()),
        }
    }

    pub(crate) fn process_session(&self) -> Result<HostedProcessSession, String> {
        match self {
            Self::Clap(instance) => instance
                .process_session()
                .map(HostedProcessSession::Clap)
                .map_err(|error| error.token),
            Self::Vst3(instance) => instance
                .process_session()
                .map(HostedProcessSession::Vst3)
                .map_err(|error| error.token),
            Self::Au(instance) => instance
                .process_session()
                .map(HostedProcessSession::Au)
                .map_err(|error| error.token),
            Self::Lv2(instance) => instance
                .process_session()
                .map(HostedProcessSession::Lv2)
                .map_err(|error| error.token),
        }
    }
}

impl HostedProcessSession {
    pub(crate) fn start(&mut self) -> Result<(), String> {
        match self {
            Self::Clap(session) => session.start().map_err(|error| error.token),
            Self::Vst3(session) => session.start().map_err(|error| error.token),
            Self::Au(session) => session.start().map_err(|error| error.token),
            Self::Lv2(session) => session.start().map_err(|error| error.token),
        }
    }

    pub(crate) fn stop(&mut self) {
        match self {
            Self::Clap(session) => session.stop(),
            Self::Vst3(session) => session.stop(),
            Self::Au(session) => session.stop(),
            Self::Lv2(session) => session.stop(),
        }
    }

    pub(crate) fn process_interleaved_stereo_with_events(
        &mut self,
        input: &[f32],
        output: &mut [f32],
        frame_count: usize,
        events: &[PluginEvent],
    ) -> bool {
        let samples = (frame_count * 2).min(input.len()).min(output.len());
        output[..samples].copy_from_slice(&input[..samples]);
        match self {
            Self::Clap(session) => {
                session.process_in_place_with_events(output, frame_count, events)
            }
            Self::Vst3(session) => {
                session.process_in_place_with_events(output, frame_count, events)
            }
            Self::Au(session) => session.process_in_place_with_events(output, frame_count, events),
            Self::Lv2(session) => session.process_interleaved_stereo(input, output, frame_count),
        }
    }
}

/// A loaded (and possibly activated) hosted plugin instance.
pub(crate) struct LoadedPlugin {
    pub(crate) instance: HostedPluginInstance,
    pub(crate) plugin_id: String,
    pub(crate) audio: Option<ActivatedAudio>,
}
