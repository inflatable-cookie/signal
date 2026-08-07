//! VST3 hosting wire: events.

use std::ffi::c_void;
use std::ptr;
use std::sync::Arc;

#[cfg(not(target_os = "macos"))]
use libloading::Library;

#[cfg(not(target_os = "macos"))]
use crate::vst3_host_adapter::introspection::resolve_module_binary_path;

use super::com::*;
use super::parameters::*;
use super::stream::*;

// ── Host-side input IEventList + IMidiMapping (note/CC delivery) ────────────
//
// Note events ride VST3's native event list (float velocity preserved).
// INPUT CC has no event type in VST3: it maps through the controller's
// IMidiMapping to a parameter, and the mapped parameter change rides
// `IParameterChanges` with the CC event's intra-block sample offset. That
// mapping query IS the VST3 downconversion boundary; plugins exposing no
// IMidiMapping simply receive no CC (honest fallback, see
// [`Vst3HostedInstance::midi_cc_mapping_available`]). Pitch bend and channel
// pressure use the VST3 extended controller numbers 128 and 129 through the
// same mapping interface.

pub(crate) const VST3_PITCH_BEND_CONTROLLER: usize = 128;
pub(crate) const VST3_AFTERTOUCH_CONTROLLER: usize = 129;
pub(crate) const VST3_MIDI_CONTROLLER_COUNT: usize = 130;

/// `Steinberg::Vst::NoteOnEvent`.
#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) struct NoteOnEventPayload {
    pub(crate) channel: i16,
    pub(crate) pitch: i16,
    pub(crate) tuning: f32,
    pub(crate) velocity: f32,
    pub(crate) length: i32,
    pub(crate) note_id: i32,
}

/// `Steinberg::Vst::NoteOffEvent`.
#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) struct NoteOffEventPayload {
    pub(crate) channel: i16,
    pub(crate) pitch: i16,
    pub(crate) velocity: f32,
    pub(crate) note_id: i32,
    pub(crate) tuning: f32,
}

/// The `Event` union payload: sized/aligned to the widest published member
/// (pointer-bearing members give the C union 8-byte alignment; 24 bytes
/// covers `NoteExpressionTextEvent`).
#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) union EventPayload {
    note_on: NoteOnEventPayload,
    note_off: NoteOffEventPayload,
    _size: [u64; 3],
}

/// `Steinberg::Vst::Event::EventTypes`.
pub(crate) const K_NOTE_ON_EVENT: u16 = 0;
pub(crate) const K_NOTE_OFF_EVENT: u16 = 1;

/// `Steinberg::Vst::Event`.
#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) struct Vst3Event {
    pub(crate) bus_index: i32,
    pub(crate) sample_offset: i32,
    pub(crate) ppq_position: f64,
    pub(crate) flags: u16,
    pub(crate) type_: u16,
    pub(crate) payload: EventPayload,
}

impl Vst3Event {
    pub(crate) fn zeroed() -> Self {
        Self {
            bus_index: 0,
            sample_offset: 0,
            ppq_position: 0.0,
            flags: 0,
            type_: 0,
            payload: EventPayload { _size: [0; 3] },
        }
    }
}

/// `IEventList` vtable (FUnknown + list methods, declaration order).
#[repr(C)]
pub(crate) struct EventListVTable {
    pub(crate) query_interface:
        unsafe extern "C" fn(*mut c_void, *const Tuid, *mut *mut c_void) -> Tresult,
    pub(crate) add_ref: unsafe extern "C" fn(*mut c_void) -> u32,
    pub(crate) release: unsafe extern "C" fn(*mut c_void) -> u32,
    pub(crate) get_event_count: unsafe extern "C" fn(*mut c_void) -> i32,
    pub(crate) get_event: unsafe extern "C" fn(*mut c_void, i32, *mut Vst3Event) -> Tresult,
    pub(crate) add_event: unsafe extern "C" fn(*mut c_void, *mut Vst3Event) -> Tresult,
}

/// Per-block note in-event capacity (matches the render plane's cap).
pub(crate) const EVENT_LIST_CAPACITY: usize = 1024;

/// The block's input event list: a fixed pool plus the active count. Boxed
/// by the session so the pointer handed to the plugin stays stable.
#[repr(C)]
pub(crate) struct HostEventList {
    pub(crate) vtable: *const EventListVTable,
    pub(crate) events: Box<[Vst3Event]>,
    pub(crate) active: usize,
}

pub(crate) static EVENT_LIST_VTABLE: EventListVTable = EventListVTable {
    query_interface: event_list_query_interface,
    add_ref: param_com_add_ref,
    release: param_com_release,
    get_event_count: event_list_get_event_count,
    get_event: event_list_get_event,
    add_event: event_list_add_event,
};

unsafe extern "C" fn event_list_query_interface(
    this: *mut c_void,
    iid: *const Tuid,
    out: *mut *mut c_void,
) -> Tresult {
    if out.is_null() {
        return K_NO_INTERFACE;
    }
    if !iid.is_null() && (*iid == FUNKNOWN_IID || *iid == IEVENT_LIST_IID) {
        *out = this;
        return K_RESULT_OK;
    }
    *out = ptr::null_mut();
    K_NO_INTERFACE
}

unsafe extern "C" fn event_list_get_event_count(this: *mut c_void) -> i32 {
    (*this.cast::<HostEventList>()).active as i32
}

unsafe extern "C" fn event_list_get_event(
    this: *mut c_void,
    index: i32,
    event: *mut Vst3Event,
) -> Tresult {
    if event.is_null() {
        return K_NO_INTERFACE;
    }
    let list = &*this.cast::<HostEventList>();
    if index < 0 || index as usize >= list.active {
        return K_NO_INTERFACE;
    }
    *event = list.events[index as usize];
    K_RESULT_OK
}

unsafe extern "C" fn event_list_add_event(this: *mut c_void, event: *mut Vst3Event) -> Tresult {
    if event.is_null() {
        return K_NO_INTERFACE;
    }
    let list = &mut *this.cast::<HostEventList>();
    if list.active == list.events.len() {
        return K_RESULT_FALSE;
    }
    list.events[list.active] = *event;
    list.active += 1;
    K_RESULT_OK
}

impl HostEventList {
    pub(crate) fn new() -> Box<Self> {
        Box::new(Self {
            vtable: &EVENT_LIST_VTABLE,
            events: vec![Vst3Event::zeroed(); EVENT_LIST_CAPACITY].into_boxed_slice(),
            active: 0,
        })
    }

    pub(crate) fn clear(&mut self) {
        self.active = 0;
    }

    /// Append one note event; silently drops on capacity overflow.
    pub(crate) fn push_note(&mut self, note: &signal_plugin::NoteEvent) {
        if self.active == self.events.len() {
            return;
        }
        let event = &mut self.events[self.active];
        event.bus_index = 0;
        event.sample_offset = note.offset_frames.min(i32::MAX as u32) as i32;
        event.ppq_position = 0.0;
        event.flags = 0;
        match note.kind {
            signal_plugin::NoteEventKind::NoteOn => {
                event.type_ = K_NOTE_ON_EVENT;
                event.payload = EventPayload {
                    note_on: NoteOnEventPayload {
                        channel: i16::from(note.channel),
                        pitch: i16::from(note.key),
                        tuning: 0.0,
                        velocity: note.velocity.clamp(0.0, 1.0),
                        length: 0,
                        note_id: note.note_id,
                    },
                };
            }
            signal_plugin::NoteEventKind::NoteOff => {
                event.type_ = K_NOTE_OFF_EVENT;
                event.payload = EventPayload {
                    note_off: NoteOffEventPayload {
                        channel: i16::from(note.channel),
                        pitch: i16::from(note.key),
                        velocity: note.velocity.clamp(0.0, 1.0),
                        note_id: note.note_id,
                        tuning: 0.0,
                    },
                };
            }
        }
        self.active += 1;
    }
}

/// `IMidiMapping` vtable (FUnknown + the one mapping method).
#[repr(C)]
pub(crate) struct MidiMappingVTable {
    pub(crate) query_interface:
        unsafe extern "C" fn(*mut c_void, *const Tuid, *mut *mut c_void) -> Tresult,
    pub(crate) add_ref: unsafe extern "C" fn(*mut c_void) -> u32,
    pub(crate) release: unsafe extern "C" fn(*mut c_void) -> u32,
    pub(crate) get_midi_controller_assignment:
        unsafe extern "C" fn(*mut c_void, i32, i16, i16, *mut u32) -> Tresult,
}

/// Query the controller's `IMidiMapping` for bus 0 / channel 0 CC → param
/// assignments (controllers 0..=127). `None` when the controller exposes no
/// mapping — the honest no-CC fallback. Runs at load on the lifecycle
/// thread; the resulting table is immutable and shared into sessions.
pub(crate) unsafe fn midi_cc_parameter_map(
    controller: *mut c_void,
) -> Option<Arc<[Option<u32>; VST3_MIDI_CONTROLLER_COUNT]>> {
    let mapping = com_query_interface(controller, &IMIDI_MAPPING_IID)?;
    let vtable = vtable_of::<MidiMappingVTable>(mapping);
    let mut map = [None; VST3_MIDI_CONTROLLER_COUNT];
    for controller_number in 0..VST3_MIDI_CONTROLLER_COUNT as i16 {
        let mut parameter_id = 0u32;
        if ((*vtable).get_midi_controller_assignment)(
            mapping,
            0,
            0,
            controller_number,
            &mut parameter_id,
        ) == K_RESULT_OK
        {
            map[controller_number as usize] = Some(parameter_id);
        }
    }
    com_release(mapping);
    Some(Arc::new(map))
}
