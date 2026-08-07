//! Audio-thread CLAP process session.

use std::{
    ptr,
    sync::atomic::{AtomicI64, Ordering},
};

use clap_sys::{
    audio_buffer::clap_audio_buffer,
    events::{
        clap_event_header, clap_event_midi, clap_event_note, clap_event_note_expression,
        clap_event_param_value, clap_event_transport, clap_input_events, clap_output_events,
        CLAP_CORE_EVENT_SPACE_ID, CLAP_EVENT_MIDI, CLAP_EVENT_NOTE_EXPRESSION, CLAP_EVENT_NOTE_OFF,
        CLAP_EVENT_NOTE_ON, CLAP_EVENT_PARAM_VALUE, CLAP_EVENT_TRANSPORT,
        CLAP_NOTE_EXPRESSION_BRIGHTNESS, CLAP_NOTE_EXPRESSION_PRESSURE,
        CLAP_NOTE_EXPRESSION_TUNING, CLAP_TRANSPORT_HAS_BEATS_TIMELINE,
        CLAP_TRANSPORT_HAS_SECONDS_TIMELINE, CLAP_TRANSPORT_HAS_TEMPO,
        CLAP_TRANSPORT_HAS_TIME_SIGNATURE,
    },
    fixedpoint::{CLAP_BEATTIME_FACTOR, CLAP_SECTIME_FACTOR},
    plugin::clap_plugin,
    process::{clap_process, CLAP_PROCESS_ERROR},
};
use signal_plugin::{
    NoteEventKind, NoteExpressionKind, PluginAudioBusDirection, PluginEvent, PluginParamChange,
    PluginParamChangeQueue, PLUGIN_PARAM_CHANGE_CAPACITY,
};
use std::sync::Arc;

use crate::discovery::PluginAudioBusDescriptorList;

use super::entry::ClapHostingError;

pub(crate) struct ParamOutCapture {
    pub(crate) queue: Arc<PluginParamChangeQueue>,
    /// The `clap_output_events` handed to the plugin; `ctx` points back at
    /// this boxed struct.
    pub(crate) list: clap_output_events,
}

pub(crate) unsafe extern "C" fn param_out_events_try_push(
    list: *const clap_output_events,
    event: *const clap_event_header,
) -> bool {
    if list.is_null() || (*list).ctx.is_null() || event.is_null() {
        return false;
    }
    if (*event).space_id == CLAP_CORE_EVENT_SPACE_ID
        && (*event).type_ == CLAP_EVENT_PARAM_VALUE
        && (*event).size as usize >= std::mem::size_of::<clap_event_param_value>()
    {
        let capture = &*(*list).ctx.cast::<ParamOutCapture>();
        let value_event = &*event.cast::<clap_event_param_value>();
        // A full ring still reports the push accepted: the ring coalesces
        // last-write-wins per drain, and refusing would make plugins spin.
        let _ = capture.queue.push(value_event.param_id, value_event.value);
    }
    true
}

pub(crate) unsafe extern "C" fn empty_in_events_size(_list: *const clap_input_events) -> u32 {
    0
}

pub(crate) unsafe extern "C" fn empty_in_events_get(
    _list: *const clap_input_events,
    _index: u32,
) -> *const clap_event_header {
    ptr::null()
}

/// Empty input event list, served when no param change is pending
/// (g12.023: pending changes ride a session-owned event list instead).
static EMPTY_IN_EVENTS: clap_input_events = clap_input_events {
    ctx: ptr::null_mut(),
    size: Some(empty_in_events_size),
    get: Some(empty_in_events_get),
};

/// Per-block cap on note/MIDI in-events forwarded to the plugin (matches
/// the render plane's per-block event capacity; overflow drops, earliest
/// wins — never an allocation on the audio thread).
const IN_EVENT_CAPACITY: usize = 1024;

/// Which backing array an in-event order entry points into.
#[derive(Clone, Copy)]
pub(crate) enum InEventSlot {
    Param(u32),
    Note(u32),
    NoteExpression(u32),
    Midi(u32),
}

/// The session-owned in-event list served to the plugin through
/// `clap_input_events` (g12.023, widened for note/CC delivery). Boxed by
/// the session so the `ctx` pointer inside the embedded
/// `clap_input_events` stays stable while the session moves between
/// threads. Rebuilt at the top of every processed block — param writes
/// from the shared change queue land at time offset 0 (block-boundary
/// posture), note/MIDI events keep their intra-block sample offsets. All
/// vecs are preallocated; the audio thread never allocates.
///
/// This is the MIDI 1.0 downconversion boundary for CLAP CC delivery:
/// [`PluginEvent::ControlChange`] values (normalized f32) become 3-byte
/// `clap_event_midi` messages here (`round(value * 127)`); note events use
/// CLAP's native `clap_event_note` and keep full float velocity.
pub(crate) struct ParamEventList {
    pub(crate) params: Vec<clap_event_param_value>,
    pub(crate) notes: Vec<clap_event_note>,
    pub(crate) note_expressions: Vec<clap_event_note_expression>,
    pub(crate) midi: Vec<clap_event_midi>,
    /// Delivery order (nondecreasing header time, params first at 0).
    pub(crate) order: Vec<InEventSlot>,
    /// The `clap_input_events` handed to the plugin; `ctx` points back at
    /// this boxed struct.
    pub(crate) list: clap_input_events,
}

pub(crate) unsafe extern "C" fn param_in_events_size(list: *const clap_input_events) -> u32 {
    if list.is_null() || (*list).ctx.is_null() {
        return 0;
    }
    (*(*list).ctx.cast::<ParamEventList>()).order.len() as u32
}

pub(crate) unsafe extern "C" fn param_in_events_get(
    list: *const clap_input_events,
    index: u32,
) -> *const clap_event_header {
    if list.is_null() || (*list).ctx.is_null() {
        return ptr::null();
    }
    let events = &(*(*list).ctx.cast::<ParamEventList>());
    match events.order.get(index as usize) {
        Some(InEventSlot::Param(slot)) => events
            .params
            .get(*slot as usize)
            .map(|event| (&event.header as *const clap_event_header).cast())
            .unwrap_or(ptr::null()),
        Some(InEventSlot::Note(slot)) => events
            .notes
            .get(*slot as usize)
            .map(|event| (&event.header as *const clap_event_header).cast())
            .unwrap_or(ptr::null()),
        Some(InEventSlot::NoteExpression(slot)) => events
            .note_expressions
            .get(*slot as usize)
            .map(|event| (&event.header as *const clap_event_header).cast())
            .unwrap_or(ptr::null()),
        Some(InEventSlot::Midi(slot)) => events
            .midi
            .get(*slot as usize)
            .map(|event| (&event.header as *const clap_event_header).cast())
            .unwrap_or(ptr::null()),
        None => ptr::null(),
    }
}

struct ClapAudioBusBuffers {
    samples: Vec<Vec<Vec<f32>>>,
    _channel_pointers: Vec<Vec<*mut f32>>,
    descriptors: Vec<clap_audio_buffer>,
}

impl ClapAudioBusBuffers {
    fn new(channel_counts: &[usize], max_frames: usize) -> Self {
        let mut samples = channel_counts
            .iter()
            .map(|&channel_count| vec![vec![0.0; max_frames]; channel_count])
            .collect::<Vec<_>>();
        let mut channel_pointers = samples
            .iter_mut()
            .map(|channels| {
                channels
                    .iter_mut()
                    .map(|channel| channel.as_mut_ptr())
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();
        let descriptors = channel_pointers
            .iter_mut()
            .map(|channels| clap_audio_buffer {
                data32: if channels.is_empty() {
                    ptr::null_mut()
                } else {
                    channels.as_mut_ptr()
                },
                data64: ptr::null_mut(),
                channel_count: channels.len() as u32,
                latency: 0,
                constant_mask: 0,
            })
            .collect();
        Self {
            samples,
            _channel_pointers: channel_pointers,
            descriptors,
        }
    }

    fn clear(&mut self, frames: usize) {
        for bus in &mut self.samples {
            for channel in bus {
                channel[..frames].fill(0.0);
            }
        }
    }

    fn copy_interleaved_stereo_into(&mut self, bus_index: usize, input: &[f32], frames: usize) {
        let Some(bus) = self.samples.get_mut(bus_index) else {
            return;
        };
        let [left, right, ..] = bus.as_mut_slice() else {
            return;
        };
        for frame in 0..frames {
            left[frame] = input[frame * 2];
            right[frame] = input[frame * 2 + 1];
        }
    }

    fn copy_interleaved_stereo_from(&self, bus_index: usize, output: &mut [f32], frames: usize) {
        let Some(bus) = self.samples.get(bus_index) else {
            return;
        };
        let [left, right, ..] = bus.as_slice() else {
            return;
        };
        for frame in 0..frames {
            output[frame * 2] = left[frame];
            output[frame * 2 + 1] = right[frame];
        }
    }

    fn as_ptr(&self) -> *const clap_audio_buffer {
        if self.descriptors.is_empty() {
            ptr::null()
        } else {
            self.descriptors.as_ptr()
        }
    }

    fn as_mut_ptr(&mut self) -> *mut clap_audio_buffer {
        if self.descriptors.is_empty() {
            ptr::null_mut()
        } else {
            self.descriptors.as_mut_ptr()
        }
    }

    fn len(&self) -> u32 {
        self.descriptors.len() as u32
    }
}

/// Raw, movable process handle for one activated instance: the plugin
/// pointer plus preallocated planar audio-bus buffers. The sandbox moves this
/// onto its audio thread; the owning [`ClapHostedInstance`] must outlive it
/// and must not run lifecycle transitions while the session is live.
pub struct ClapProcessSession {
    plugin: *const clap_plugin,
    sample_rate_hz: f64,
    input_buses: ClapAudioBusBuffers,
    output_buses: ClapAudioBusBuffers,
    main_input_bus: Option<usize>,
    main_output_bus: usize,
    max_frames: usize,
    steady_time: AtomicI64,
    processing: bool,
    /// Pending param writes shared with the owning instance (g12.023).
    param_changes: Arc<PluginParamChangeQueue>,
    /// Drain scratch (preallocated; audio thread never allocates).
    param_scratch: Vec<PluginParamChange>,
    /// The in-event list served to the plugin, rebuilt per block.
    param_events: Box<ParamEventList>,
    /// The out-events capture served to the plugin (g12.024).
    param_out: Box<ParamOutCapture>,
}

// Safety: the session is handed to exactly one audio thread; CLAP's process
// and start/stop_processing are audio-thread functions, and the owner
// serializes lifecycle against the session per the type contract above.
unsafe impl Send for ClapProcessSession {}

impl ClapProcessSession {
    pub(crate) fn new(
        plugin: *const clap_plugin,
        sample_rate_hz: f64,
        max_frames: usize,
        audio_buses: &PluginAudioBusDescriptorList,
        param_changes: Arc<PluginParamChangeQueue>,
        param_out_queue: Arc<PluginParamChangeQueue>,
    ) -> Self {
        let input_buses = audio_buses
            .iter()
            .filter(|bus| bus.direction == PluginAudioBusDirection::Input)
            .collect::<Vec<_>>();
        let output_buses = audio_buses
            .iter()
            .filter(|bus| bus.direction == PluginAudioBusDirection::Output)
            .collect::<Vec<_>>();
        let main_input_bus = input_buses.iter().position(|bus| bus.is_main);
        let main_output_bus = output_buses
            .iter()
            .position(|bus| bus.is_main)
            .expect("supported CLAP layouts always have a main output bus");
        let input_channel_counts = input_buses
            .iter()
            .map(|bus| usize::from(bus.channels))
            .collect::<Vec<_>>();
        let output_channel_counts = output_buses
            .iter()
            .map(|bus| usize::from(bus.channels))
            .collect::<Vec<_>>();
        let mut param_events = Box::new(ParamEventList {
            params: Vec::with_capacity(PLUGIN_PARAM_CHANGE_CAPACITY),
            notes: Vec::with_capacity(IN_EVENT_CAPACITY),
            note_expressions: Vec::with_capacity(IN_EVENT_CAPACITY),
            midi: Vec::with_capacity(IN_EVENT_CAPACITY),
            order: Vec::with_capacity(PLUGIN_PARAM_CHANGE_CAPACITY + IN_EVENT_CAPACITY),
            list: clap_input_events {
                ctx: ptr::null_mut(),
                size: Some(param_in_events_size),
                get: Some(param_in_events_get),
            },
        });
        // Self-referential ctx: the list lives inside the box (stable
        // address) for the session's whole lifetime.
        param_events.list.ctx = (&mut *param_events as *mut ParamEventList).cast();
        let mut param_out = Box::new(ParamOutCapture {
            queue: param_out_queue,
            list: clap_output_events {
                ctx: ptr::null_mut(),
                try_push: Some(param_out_events_try_push),
            },
        });
        param_out.list.ctx = (&mut *param_out as *mut ParamOutCapture).cast();
        Self {
            plugin,
            sample_rate_hz,
            input_buses: ClapAudioBusBuffers::new(&input_channel_counts, max_frames),
            output_buses: ClapAudioBusBuffers::new(&output_channel_counts, max_frames),
            main_input_bus,
            main_output_bus,
            max_frames,
            steady_time: AtomicI64::new(0),
            processing: false,
            param_changes,
            param_scratch: Vec::with_capacity(PLUGIN_PARAM_CHANGE_CAPACITY),
            param_events,
            param_out,
        }
    }

    /// Build a valid stopped transport snapshot for the current block.
    ///
    /// CLAP permits a null `process.transport`, but a number of otherwise
    /// conforming plugins assume the pointer is always present. Supplying a
    /// conservative stopped timeline is harmless to plugins that honour the
    /// optional contract and avoids crashing those that do not.
    fn transport(&self, steady_time: i64) -> clap_event_transport {
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
    fn prepare_in_events(&mut self, events: &[PluginEvent]) -> *const clap_input_events {
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

    /// `start_processing` on the audio thread; must precede `process`.
    pub fn start(&mut self) -> Result<(), ClapHostingError> {
        if self.processing {
            return Ok(());
        }
        let ok = unsafe {
            (*self.plugin)
                .start_processing
                .map(|start| start(self.plugin))
                .unwrap_or(true)
        };
        if !ok {
            return Err(ClapHostingError::new("start_processing_failed"));
        }
        self.processing = true;
        Ok(())
    }

    /// `stop_processing` on the audio thread.
    pub fn stop(&mut self) {
        if !self.processing {
            return;
        }
        if let Some(stop) = unsafe { (*self.plugin).stop_processing } {
            unsafe { stop(self.plugin) };
        }
        self.processing = false;
    }

    /// Process one block: optional interleaved stereo in, stereo out.
    /// Alloc-free (buffers preallocated at activate). On plugin error the
    /// input passes through unchanged. Returns `false` on error.
    pub fn process_interleaved_stereo(
        &mut self,
        input: &[f32],
        output: &mut [f32],
        frame_count: usize,
    ) -> bool {
        let frames = frame_count
            .min(self.max_frames)
            .min(input.len() / 2)
            .min(output.len() / 2);
        let in_events = self.prepare_in_events(&[]);
        self.input_buses.clear(frames);
        self.output_buses.clear(frames);
        if let Some(main_input_bus) = self.main_input_bus {
            self.input_buses
                .copy_interleaved_stereo_into(main_input_bus, input, frames);
        }
        let steady_time = self.steady_time.load(Ordering::Relaxed);
        let audio_inputs = self.input_buses.as_ptr();
        let audio_inputs_count = self.input_buses.len();
        let audio_outputs = self.output_buses.as_mut_ptr();
        let audio_outputs_count = self.output_buses.len();
        let transport = self.transport(steady_time);
        let process = clap_process {
            steady_time,
            frames_count: frames as u32,
            transport: &transport,
            audio_inputs,
            audio_outputs,
            audio_inputs_count,
            audio_outputs_count,
            in_events,
            out_events: &self.param_out.list,
        };
        self.steady_time
            .store(steady_time + frames as i64, Ordering::Relaxed);

        let status = unsafe {
            (*self.plugin)
                .process
                .map(|process_fn| process_fn(self.plugin, &process))
                .unwrap_or(CLAP_PROCESS_ERROR)
        };
        if status == CLAP_PROCESS_ERROR {
            output[..frames * 2].copy_from_slice(&input[..frames * 2]);
            return false;
        }
        self.output_buses
            .copy_interleaved_stereo_from(self.main_output_bus, output, frames);
        true
    }

    /// In-place variant for the in-process isolation tier: processes the
    /// interleaved stereo buffer and writes the result back over it ONLY on
    /// success; on plugin error the buffer is left untouched (bypass
    /// semantics). Alloc-free. `true` = buffer transformed.
    pub fn process_in_place(&mut self, io: &mut [f32], frame_count: usize) -> bool {
        self.process_in_place_with_events(io, frame_count, &[])
    }

    /// [`Self::process_in_place`] with a per-block plugin event slice
    /// (sorted by `offset_frames`): note events map to CLAP note in-events
    /// (float velocity preserved), CC events downconvert to 3-byte MIDI at
    /// this boundary. Alloc-free. `true` = buffer transformed.
    pub fn process_in_place_with_events(
        &mut self,
        io: &mut [f32],
        frame_count: usize,
        events: &[PluginEvent],
    ) -> bool {
        let frames = frame_count.min(self.max_frames).min(io.len() / 2);
        let in_events = self.prepare_in_events(events);
        self.input_buses.clear(frames);
        self.output_buses.clear(frames);
        if let Some(main_input_bus) = self.main_input_bus {
            self.input_buses
                .copy_interleaved_stereo_into(main_input_bus, io, frames);
        }
        let steady_time = self.steady_time.load(Ordering::Relaxed);
        let audio_inputs = self.input_buses.as_ptr();
        let audio_inputs_count = self.input_buses.len();
        let audio_outputs = self.output_buses.as_mut_ptr();
        let audio_outputs_count = self.output_buses.len();
        let transport = self.transport(steady_time);
        let process = clap_process {
            steady_time,
            frames_count: frames as u32,
            transport: &transport,
            audio_inputs,
            audio_outputs,
            audio_inputs_count,
            audio_outputs_count,
            in_events,
            out_events: &self.param_out.list,
        };
        self.steady_time
            .store(steady_time + frames as i64, Ordering::Relaxed);

        let status = unsafe {
            (*self.plugin)
                .process
                .map(|process_fn| process_fn(self.plugin, &process))
                .unwrap_or(CLAP_PROCESS_ERROR)
        };
        if status == CLAP_PROCESS_ERROR {
            return false;
        }
        self.output_buses
            .copy_interleaved_stereo_from(self.main_output_bus, io, frames);
        true
    }

    /// Whether `start()` has succeeded and `stop()` has not yet run.
    pub fn is_processing(&self) -> bool {
        self.processing
    }
}
