use crate::*;
use signal_ipc::{PluginIoLayoutPayload, SharedMemoryLayoutPayload, SharedMemoryRegionPayload};
use signal_plugin::{
    EventPacket, MidiEvent, NoteEvent, NoteEventKind, NoteExpressionEvent, NoteExpressionKind,
    ParameterGestureEvent, ParameterGesturePhase, ParameterModulationEvent, ParameterValueEvent,
    PluginEvent, PluginIoLayout, SharedMemoryLayout, SharedMemoryRegion,
};

pub fn translate_input_events(packet: &EventPacket) -> ClapEventPacket {
    let events = packet
        .events
        .iter()
        .map(|event| match event {
            PluginEvent::ParameterValue(event) => ClapEvent::ParamValue(ClapParamValueEvent {
                offset_frames: event.offset_frames,
                clap_param_id: event.parameter_id,
                normalized_value: event.normalized_value as f64,
            }),
            PluginEvent::ParameterModulation(event) => {
                ClapEvent::ParamModulation(ClapParamModEvent {
                    offset_frames: event.offset_frames,
                    clap_param_id: event.parameter_id,
                    amount: event.amount as f64,
                })
            }
            PluginEvent::ParameterGesture(event) => {
                ClapEvent::ParamGesture(ClapParamGestureEvent {
                    offset_frames: event.offset_frames,
                    clap_param_id: event.parameter_id,
                    phase: match event.phase {
                        ParameterGesturePhase::Begin => ClapParamGesturePhase::Begin,
                        ParameterGesturePhase::End => ClapParamGesturePhase::End,
                    },
                })
            }
            PluginEvent::Note(event) => ClapEvent::Note(ClapNoteEvent {
                offset_frames: event.offset_frames,
                note_id: event.note_id,
                port_index: event.port_index,
                channel: event.channel,
                key: event.key,
                velocity: event.velocity as f64,
                kind: match event.kind {
                    NoteEventKind::NoteOn => ClapNoteEventKind::NoteOn,
                    NoteEventKind::NoteOff => ClapNoteEventKind::NoteOff,
                },
            }),
            PluginEvent::NoteExpression(event) => {
                ClapEvent::NoteExpression(ClapNoteExpressionEvent {
                    offset_frames: event.offset_frames,
                    note_id: event.note_id,
                    port_index: event.port_index,
                    channel: event.channel,
                    key: event.key,
                    expression: clap_note_expression_kind(event.expression),
                    value: event.value as f64,
                })
            }
            PluginEvent::Midi(event) => {
                if let Some(note_event) = midi_to_clap_note(*event) {
                    ClapEvent::Note(note_event)
                } else if let Some(note_expression) = midi_to_clap_note_expression(*event) {
                    ClapEvent::NoteExpression(note_expression)
                } else {
                    ClapEvent::Midi(ClapMidiEvent {
                        offset_frames: event.offset_frames,
                        port_index: 0,
                        data: [event.status, event.data1, event.data2],
                    })
                }
            }
        })
        .collect();
    ClapEventPacket { events }
}

pub fn translate_output_events(packet: &ClapEventPacket) -> EventPacket {
    let events = packet
        .events
        .iter()
        .map(|event| match event {
            ClapEvent::ParamValue(event) => PluginEvent::ParameterValue(ParameterValueEvent {
                offset_frames: event.offset_frames,
                parameter_id: event.clap_param_id,
                normalized_value: event.normalized_value as f32,
            }),
            ClapEvent::ParamModulation(event) => {
                PluginEvent::ParameterModulation(ParameterModulationEvent {
                    offset_frames: event.offset_frames,
                    parameter_id: event.clap_param_id,
                    amount: event.amount as f32,
                })
            }
            ClapEvent::ParamGesture(event) => {
                PluginEvent::ParameterGesture(ParameterGestureEvent {
                    offset_frames: event.offset_frames,
                    parameter_id: event.clap_param_id,
                    phase: match event.phase {
                        ClapParamGesturePhase::Begin => ParameterGesturePhase::Begin,
                        ClapParamGesturePhase::End => ParameterGesturePhase::End,
                    },
                })
            }
            ClapEvent::Note(event) => PluginEvent::Note(NoteEvent {
                offset_frames: event.offset_frames,
                note_id: event.note_id,
                port_index: event.port_index,
                channel: event.channel,
                key: event.key,
                velocity: event.velocity as f32,
                kind: match event.kind {
                    ClapNoteEventKind::NoteOn => NoteEventKind::NoteOn,
                    ClapNoteEventKind::NoteOff => NoteEventKind::NoteOff,
                },
            }),
            ClapEvent::NoteExpression(event) => PluginEvent::NoteExpression(NoteExpressionEvent {
                offset_frames: event.offset_frames,
                note_id: event.note_id,
                port_index: event.port_index,
                channel: event.channel,
                key: event.key,
                expression: plugin_note_expression_kind(event.expression),
                value: event.value as f32,
            }),
            ClapEvent::Midi(event) => PluginEvent::Midi(MidiEvent {
                offset_frames: event.offset_frames,
                status: event.data[0],
                data1: event.data[1],
                data2: event.data[2],
            }),
        })
        .collect();
    EventPacket::new(events)
}

pub fn clap_note_expression_kind(kind: NoteExpressionKind) -> ClapNoteExpressionKind {
    match kind {
        NoteExpressionKind::Pressure => ClapNoteExpressionKind::Pressure,
        NoteExpressionKind::Timbre => ClapNoteExpressionKind::Timbre,
        NoteExpressionKind::Tuning => ClapNoteExpressionKind::Tuning,
    }
}

pub fn plugin_note_expression_kind(kind: ClapNoteExpressionKind) -> NoteExpressionKind {
    match kind {
        ClapNoteExpressionKind::Pressure => NoteExpressionKind::Pressure,
        ClapNoteExpressionKind::Timbre => NoteExpressionKind::Timbre,
        ClapNoteExpressionKind::Tuning => NoteExpressionKind::Tuning,
    }
}

pub fn midi_to_clap_note(event: MidiEvent) -> Option<ClapNoteEvent> {
    let status = event.status & 0xF0;
    let channel = event.status & 0x0F;
    match status {
        0x90 if event.data2 > 0 => Some(ClapNoteEvent {
            offset_frames: event.offset_frames,
            note_id: -1,
            port_index: 0,
            channel,
            key: event.data1,
            velocity: f64::from(event.data2) / 127.0,
            kind: ClapNoteEventKind::NoteOn,
        }),
        0x80 | 0x90 => Some(ClapNoteEvent {
            offset_frames: event.offset_frames,
            note_id: -1,
            port_index: 0,
            channel,
            key: event.data1,
            velocity: f64::from(event.data2) / 127.0,
            kind: ClapNoteEventKind::NoteOff,
        }),
        _ => None,
    }
}

pub fn midi_to_clap_note_expression(event: MidiEvent) -> Option<ClapNoteExpressionEvent> {
    let status = event.status & 0xF0;
    let channel = event.status & 0x0F;
    match status {
        0xA0 => Some(ClapNoteExpressionEvent {
            offset_frames: event.offset_frames,
            note_id: -1,
            port_index: 0,
            channel,
            key: event.data1,
            expression: ClapNoteExpressionKind::Pressure,
            value: f64::from(event.data2) / 127.0,
        }),
        _ => None,
    }
}

pub fn io_layout_payload(io_layout: PluginIoLayout) -> PluginIoLayoutPayload {
    PluginIoLayoutPayload {
        audio_inputs: io_layout.audio_inputs,
        audio_outputs: io_layout.audio_outputs,
        midi_inputs: io_layout.midi_inputs,
        midi_outputs: io_layout.midi_outputs,
    }
}

pub fn io_layout_from_payload(payload: PluginIoLayoutPayload) -> PluginIoLayout {
    PluginIoLayout {
        audio_inputs: payload.audio_inputs,
        audio_outputs: payload.audio_outputs,
        midi_inputs: payload.midi_inputs,
        midi_outputs: payload.midi_outputs,
    }
}

pub fn shared_memory_layout_payload(layout: SharedMemoryLayout) -> SharedMemoryLayoutPayload {
    SharedMemoryLayoutPayload {
        audio_input: shared_region_payload(layout.audio_input),
        audio_output: shared_region_payload(layout.audio_output),
        event_input: shared_region_payload(layout.event_input),
        event_output: shared_region_payload(layout.event_output),
        render_context: shared_region_payload(layout.render_context),
        completion: shared_region_payload(layout.completion),
    }
}

pub fn shared_memory_layout(payload: SharedMemoryLayoutPayload) -> SharedMemoryLayout {
    SharedMemoryLayout {
        audio_input: shared_region(payload.audio_input),
        audio_output: shared_region(payload.audio_output),
        event_input: shared_region(payload.event_input),
        event_output: shared_region(payload.event_output),
        render_context: shared_region(payload.render_context),
        completion: shared_region(payload.completion),
    }
}

pub fn shared_region_payload(region: SharedMemoryRegion) -> SharedMemoryRegionPayload {
    SharedMemoryRegionPayload {
        offset_bytes: region.offset_bytes,
        size_bytes: region.size_bytes,
    }
}

pub fn shared_region(payload: SharedMemoryRegionPayload) -> SharedMemoryRegion {
    SharedMemoryRegion {
        offset_bytes: payload.offset_bytes,
        size_bytes: payload.size_bytes,
    }
}
