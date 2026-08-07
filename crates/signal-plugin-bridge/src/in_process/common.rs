//! Shared in-process event conversion and GUI event vocabulary.

use signal_plugin::{
    ControlChangeEvent, MidiEvent, NoteEvent, NoteEventKind, NoteExpressionEvent,
    NoteExpressionKind, PluginEvent,
};
use signal_render_plane::{
    RenderBlockPluginEvent, RenderNoteExpressionKind, RenderPluginEventKind,
    RenderPluginEventSupport,
};

/// Per-block plugin event capacity for the in-process backends' conversion
/// scratch (mirrors the render plane's per-block cap).
pub(crate) const EVENT_SCRATCH_CAPACITY: usize = 1024;

pub(crate) const CLAP_EVENT_SUPPORT: RenderPluginEventSupport = RenderPluginEventSupport {
    notes: true,
    control_change: true,
    pitch_bend: true,
    channel_pressure: true,
    note_expression: true,
};

pub(crate) const AU_EVENT_SUPPORT: RenderPluginEventSupport = RenderPluginEventSupport {
    notes: true,
    control_change: true,
    pitch_bend: true,
    channel_pressure: true,
    note_expression: false,
};

/// Convert the render plane's block events into the plugin event vocabulary
/// handed to the per-format process sessions. Values stay normalized f32
/// here; each session downconverts (or maps) at its own format boundary.
/// Alloc-free within the scratch's preallocated capacity (overflow drops,
/// earliest wins). Returns `None` for event kinds no plugin format can
/// represent (the voice-bank family — those target native processors).
pub(crate) fn convert_block_event(event: &RenderBlockPluginEvent) -> Option<PluginEvent> {
    Some(match event.kind {
        RenderPluginEventKind::NoteOn { key, velocity } => PluginEvent::Note(NoteEvent {
            offset_frames: event.offset_frames,
            note_id: -1,
            port_index: 0,
            channel: event.channel,
            key,
            velocity,
            kind: NoteEventKind::NoteOn,
        }),
        RenderPluginEventKind::NoteOff { key } => PluginEvent::Note(NoteEvent {
            offset_frames: event.offset_frames,
            note_id: -1,
            port_index: 0,
            channel: event.channel,
            key,
            velocity: 0.0,
            kind: NoteEventKind::NoteOff,
        }),
        RenderPluginEventKind::ControlChange { controller, value } => {
            PluginEvent::ControlChange(ControlChangeEvent {
                offset_frames: event.offset_frames,
                port_index: 0,
                channel: event.channel,
                controller,
                value,
            })
        }
        RenderPluginEventKind::PitchBend { value } => {
            let bend = (((value.clamp(-1.0, 1.0) + 1.0) * 0.5) * 16_383.0).round() as u16;
            PluginEvent::Midi(MidiEvent {
                offset_frames: event.offset_frames,
                status: 0xE0 | (event.channel & 0x0F),
                data1: (bend & 0x7F) as u8,
                data2: ((bend >> 7) & 0x7F) as u8,
            })
        }
        RenderPluginEventKind::ChannelPressure { value } => PluginEvent::Midi(MidiEvent {
            offset_frames: event.offset_frames,
            status: 0xD0 | (event.channel & 0x0F),
            data1: (value.clamp(0.0, 1.0) * 127.0).round() as u8,
            data2: 0,
        }),
        RenderPluginEventKind::NoteExpression {
            key,
            expression,
            value,
        } => PluginEvent::NoteExpression(NoteExpressionEvent {
            offset_frames: event.offset_frames,
            note_id: -1,
            port_index: 0,
            channel: event.channel,
            key,
            expression: match expression {
                RenderNoteExpressionKind::Pressure => NoteExpressionKind::Pressure,
                RenderNoteExpressionKind::Timbre => NoteExpressionKind::Timbre,
                RenderNoteExpressionKind::Tuning => NoteExpressionKind::Tuning,
            },
            value,
        }),
        RenderPluginEventKind::VoiceStart { .. }
        | RenderPluginEventKind::VoiceStop { .. }
        | RenderPluginEventKind::VoiceParam { .. } => return None,
    })
}

pub(crate) fn convert_block_events(
    events: &[RenderBlockPluginEvent],
    scratch: &mut Vec<PluginEvent>,
) {
    scratch.clear();
    for event in events.iter().take(EVENT_SCRATCH_CAPACITY) {
        if let Some(converted) = convert_block_event(event) {
            scratch.push(converted);
        }
    }
}

/// One format-neutral plugin GUI callback drained from an in-process
/// backend (g12.024): the union of the CLAP `clap.gui` host callbacks and
/// the VST3 `IPlugFrame` resize requests, so the embedding host pumps one
/// event shape across formats (AU Cocoa views manage themselves — no
/// events).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PluginGuiEvent {
    /// The plugin's resize constraints changed (CLAP `resize_hints_changed`).
    ResizeHintsChanged,
    /// The plugin asks the host to resize its window (CLAP
    /// `request_resize` / VST3 `IPlugFrame::resizeView`).
    RequestResize {
        /// Requested content width (logical units on macOS).
        width: u32,
        /// Requested content height (logical units on macOS).
        height: u32,
    },
    /// The plugin asks the host to show its window (CLAP `request_show`).
    RequestShow,
    /// The plugin asks the host to hide its window (CLAP `request_hide`).
    RequestHide,
    /// The editor closed itself (CLAP `closed`).
    Closed {
        /// `true` when the plugin also destroyed the gui.
        was_destroyed: bool,
    },
}

impl From<signal_plugin_clap::ClapGuiEvent> for PluginGuiEvent {
    fn from(event: signal_plugin_clap::ClapGuiEvent) -> Self {
        match event {
            signal_plugin_clap::ClapGuiEvent::ResizeHintsChanged => Self::ResizeHintsChanged,
            signal_plugin_clap::ClapGuiEvent::RequestResize { width, height } => {
                Self::RequestResize { width, height }
            }
            signal_plugin_clap::ClapGuiEvent::RequestShow => Self::RequestShow,
            signal_plugin_clap::ClapGuiEvent::RequestHide => Self::RequestHide,
            signal_plugin_clap::ClapGuiEvent::Closed { was_destroyed } => {
                Self::Closed { was_destroyed }
            }
        }
    }
}

impl From<signal_plugin_vst3::Vst3GuiEvent> for PluginGuiEvent {
    fn from(event: signal_plugin_vst3::Vst3GuiEvent) -> Self {
        match event {
            signal_plugin_vst3::Vst3GuiEvent::RequestResize { width, height } => {
                Self::RequestResize { width, height }
            }
        }
    }
}
