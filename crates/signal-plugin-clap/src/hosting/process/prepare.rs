use std::ptr;

use clap_sys::{
    events::{
        clap_event_header, clap_event_midi, clap_event_note, clap_event_note_expression,
        clap_event_param_value, clap_event_transport, clap_input_events, CLAP_CORE_EVENT_SPACE_ID,
        CLAP_EVENT_MIDI, CLAP_EVENT_NOTE_EXPRESSION, CLAP_EVENT_NOTE_OFF, CLAP_EVENT_NOTE_ON,
        CLAP_EVENT_PARAM_VALUE, CLAP_EVENT_TRANSPORT, CLAP_NOTE_EXPRESSION_BRIGHTNESS,
        CLAP_NOTE_EXPRESSION_PRESSURE, CLAP_NOTE_EXPRESSION_TUNING,
        CLAP_TRANSPORT_HAS_BEATS_TIMELINE, CLAP_TRANSPORT_HAS_SECONDS_TIMELINE,
        CLAP_TRANSPORT_HAS_TEMPO, CLAP_TRANSPORT_HAS_TIME_SIGNATURE,
    },
    fixedpoint::{CLAP_BEATTIME_FACTOR, CLAP_SECTIME_FACTOR},
};
use signal_plugin::{NoteEventKind, NoteExpressionKind, PluginEvent};

use super::events::{InEventSlot, EMPTY_IN_EVENTS, IN_EVENT_CAPACITY};
use super::session::ClapProcessSession;

impl ClapProcessSession {
    /// Build a valid stopped transport snapshot for the current block.
    ///
    /// CLAP permits a null `process.transport`, but a number of otherwise
    /// conforming plugins assume the pointer is always present. Supplying a
    /// conservative stopped timeline is harmless to plugins that honour the
    /// optional contract and avoids crashing those that do not.
    pub(crate) fn transport(&self, steady_time: i64) -> clap_event_transport {
        let seconds = steady_time as f64 / self.sample_rate_hz;
        let beats = seconds * (120.0 / 60.0);
        let beats_fixed = (beats * CLAP_BEATTIME_FACTOR as f64) as i64;
        let seconds_fixed = (seconds * CLAP_SECTIME_FACTOR as f64) as i64;
        let beats_per_bar = 4_i64 * CLAP_BEATTIME_FACTOR;
        let bar_number = beats_fixed.div_euclid(beats_per_bar) as i32;

        clap_event_transport {
            header: clap_event_header {
                size: std::mem::size_of::<clap_event_transport>() as u32,
                time: 0,
                space_id: CLAP_CORE_EVENT_SPACE_ID,
                type_: CLAP_EVENT_TRANSPORT,
                flags: 0,
            },
            flags: CLAP_TRANSPORT_HAS_TEMPO
                | CLAP_TRANSPORT_HAS_BEATS_TIMELINE
                | CLAP_TRANSPORT_HAS_SECONDS_TIMELINE
                | CLAP_TRANSPORT_HAS_TIME_SIGNATURE,
            song_pos_beats: beats_fixed,
            song_pos_seconds: seconds_fixed,
            tempo: 120.0,
            tempo_inc: 0.0,
            loop_start_beats: 0,
            loop_end_beats: 0,
            loop_start_seconds: 0,
            loop_end_seconds: 0,
            bar_start: i64::from(bar_number) * beats_per_bar,
            bar_number,
            tsig_num: 4,
            tsig_denom: 4,
        }
    }

    /// Rebuild the block's in-events: param writes from the shared change
    /// queue (block-boundary application, time offset 0) followed by the
    /// block's note/CC events at their intra-block sample offsets (`events`
    /// must be sorted by `offset_frames`; the render plane's delivery
    /// contract). Alloc-free; returns the `clap_input_events` to hand to
    /// the plugin (the empty static list when nothing is pending).
    pub(crate) fn prepare_in_events(&mut self, events: &[PluginEvent]) -> *const clap_input_events {
        let list = &mut *self.param_events;
        list.params.clear();
        list.notes.clear();
        list.note_expressions.clear();
        list.midi.clear();
        list.order.clear();
        if !self.param_changes.is_empty() {
            self.param_changes.drain_coalesced(&mut self.param_scratch);
            for change in &self.param_scratch {
                list.params.push(clap_event_param_value {
                    header: clap_event_header {
                        size: std::mem::size_of::<clap_event_param_value>() as u32,
                        time: 0,
                        space_id: CLAP_CORE_EVENT_SPACE_ID,
                        type_: CLAP_EVENT_PARAM_VALUE,
                        flags: 0,
                    },
                    param_id: change.parameter_id,
                    cookie: ptr::null_mut(),
                    note_id: -1,
                    port_index: -1,
                    channel: -1,
                    key: -1,
                    value: change.value,
                });
                list.order
                    .push(InEventSlot::Param(list.params.len() as u32 - 1));
            }
        }
        for event in events {
            match event {
                PluginEvent::Note(note) => {
                    if list.notes.len() == IN_EVENT_CAPACITY {
                        continue;
                    }
                    list.notes.push(clap_event_note {
                        header: clap_event_header {
                            size: std::mem::size_of::<clap_event_note>() as u32,
                            time: note.offset_frames,
                            space_id: CLAP_CORE_EVENT_SPACE_ID,
                            type_: match note.kind {
                                NoteEventKind::NoteOn => CLAP_EVENT_NOTE_ON,
                                NoteEventKind::NoteOff => CLAP_EVENT_NOTE_OFF,
                            },
                            flags: 0,
                        },
                        note_id: note.note_id,
                        port_index: note.port_index as i16,
                        channel: i16::from(note.channel),
                        key: i16::from(note.key),
                        velocity: f64::from(note.velocity.clamp(0.0, 1.0)),
                    });
                    list.order
                        .push(InEventSlot::Note(list.notes.len() as u32 - 1));
                }
                PluginEvent::NoteExpression(expression) => {
                    if list.note_expressions.len() == IN_EVENT_CAPACITY {
                        continue;
                    }
                    let (expression_id, value) = match expression.expression {
                        NoteExpressionKind::Pressure => (
                            CLAP_NOTE_EXPRESSION_PRESSURE,
                            f64::from(expression.value.clamp(0.0, 1.0)),
                        ),
                        NoteExpressionKind::Timbre => (
                            CLAP_NOTE_EXPRESSION_BRIGHTNESS,
                            f64::from(expression.value.clamp(0.0, 1.0)),
                        ),
                        NoteExpressionKind::Tuning => (
                            CLAP_NOTE_EXPRESSION_TUNING,
                            f64::from(expression.value) / 100.0,
                        ),
                    };
                    list.note_expressions.push(clap_event_note_expression {
                        header: clap_event_header {
                            size: std::mem::size_of::<clap_event_note_expression>() as u32,
                            time: expression.offset_frames,
                            space_id: CLAP_CORE_EVENT_SPACE_ID,
                            type_: CLAP_EVENT_NOTE_EXPRESSION,
                            flags: 0,
                        },
                        expression_id,
                        note_id: expression.note_id,
                        port_index: expression.port_index as i16,
                        channel: i16::from(expression.channel),
                        key: i16::from(expression.key),
                        value,
                    });
                    list.order.push(InEventSlot::NoteExpression(
                        list.note_expressions.len() as u32 - 1,
                    ));
                }
                PluginEvent::ControlChange(change) => {
                    // The CLAP CC boundary: normalized f32 → 3-byte MIDI 1.0
                    // (CLAP has no float CC event).
                    if list.midi.len() == IN_EVENT_CAPACITY {
                        continue;
                    }
                    list.midi.push(clap_event_midi {
                        header: clap_event_header {
                            size: std::mem::size_of::<clap_event_midi>() as u32,
                            time: change.offset_frames,
                            space_id: CLAP_CORE_EVENT_SPACE_ID,
                            type_: CLAP_EVENT_MIDI,
                            flags: 0,
                        },
                        port_index: change.port_index,
                        data: [
                            0xB0 | (change.channel & 0x0F),
                            change.controller & 0x7F,
                            (change.value.clamp(0.0, 1.0) * 127.0).round() as u8,
                        ],
                    });
                    list.order
                        .push(InEventSlot::Midi(list.midi.len() as u32 - 1));
                }
                PluginEvent::Midi(midi) => {
                    if list.midi.len() == IN_EVENT_CAPACITY {
                        continue;
                    }
                    list.midi.push(clap_event_midi {
                        header: clap_event_header {
                            size: std::mem::size_of::<clap_event_midi>() as u32,
                            time: midi.offset_frames,
                            space_id: CLAP_CORE_EVENT_SPACE_ID,
                            type_: CLAP_EVENT_MIDI,
                            flags: 0,
                        },
                        port_index: 0,
                        data: [midi.status, midi.data1, midi.data2],
                    });
                    list.order
                        .push(InEventSlot::Midi(list.midi.len() as u32 - 1));
                }
                // Parameter events ride the wire queue; gestures have no
                // process input representation here.
                _ => {}
            }
        }
        if list.order.is_empty() {
            return &EMPTY_IN_EVENTS;
        }
        &list.list
    }
}
