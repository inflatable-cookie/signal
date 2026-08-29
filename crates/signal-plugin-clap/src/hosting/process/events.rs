use std::{ptr, sync::Arc};

use clap_sys::events::{
    clap_event_header, clap_event_midi, clap_event_note, clap_event_note_expression,
    clap_event_param_value, clap_input_events, clap_output_events, CLAP_CORE_EVENT_SPACE_ID,
    CLAP_EVENT_PARAM_VALUE,
};
use signal_plugin::PluginParamChangeQueue;

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
pub(crate) static EMPTY_IN_EVENTS: clap_input_events = clap_input_events {
    ctx: ptr::null_mut(),
    size: Some(empty_in_events_size),
    get: Some(empty_in_events_get),
};

/// Per-block cap on note/MIDI in-events forwarded to the plugin (matches
/// the render plane's per-block event capacity; overflow drops, earliest
/// wins — never an allocation on the audio thread).
pub(crate) const IN_EVENT_CAPACITY: usize = 1024;

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
/// `PluginEvent::ControlChange` values (normalized f32) become 3-byte
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
