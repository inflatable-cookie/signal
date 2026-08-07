//! VST3 hosting wire: stream.

use std::ffi::{c_char, c_void};
use std::ptr;
use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use std::sync::Arc;

#[cfg(not(target_os = "macos"))]
use libloading::Library;

#[cfg(not(target_os = "macos"))]
use crate::vst3_host_adapter::introspection::resolve_module_binary_path;

use super::com::*;

// ── Host-side IBStream for opaque component/controller state ───────────────

#[repr(C)]
pub(crate) struct MemoryStreamVTable {
    pub(crate) query_interface:
        unsafe extern "C" fn(*mut c_void, *const Tuid, *mut *mut c_void) -> Tresult,
    pub(crate) add_ref: unsafe extern "C" fn(*mut c_void) -> u32,
    pub(crate) release: unsafe extern "C" fn(*mut c_void) -> u32,
    pub(crate) read: unsafe extern "C" fn(*mut c_void, *mut c_void, i32, *mut i32) -> Tresult,
    pub(crate) write: unsafe extern "C" fn(*mut c_void, *const c_void, i32, *mut i32) -> Tresult,
    pub(crate) seek: unsafe extern "C" fn(*mut c_void, i64, i32, *mut i64) -> Tresult,
    pub(crate) tell: unsafe extern "C" fn(*mut c_void, *mut i64) -> Tresult,
}

#[repr(C)]
pub(crate) struct MemoryStream {
    pub(crate) vtable: *const MemoryStreamVTable,
    pub(crate) bytes: Vec<u8>,
    pub(crate) position: usize,
    pub(crate) writable: bool,
}

impl MemoryStream {
    pub(crate) fn writer() -> Self {
        Self {
            vtable: &MEMORY_STREAM_VTABLE,
            bytes: Vec::new(),
            position: 0,
            writable: true,
        }
    }

    pub(crate) fn reader(bytes: &[u8]) -> Self {
        Self {
            vtable: &MEMORY_STREAM_VTABLE,
            bytes: bytes.to_vec(),
            position: 0,
            writable: false,
        }
    }

    pub(crate) fn as_raw(&mut self) -> *mut c_void {
        (self as *mut Self).cast()
    }
}

unsafe extern "C" fn stream_query_interface(
    this: *mut c_void,
    iid: *const Tuid,
    out: *mut *mut c_void,
) -> Tresult {
    if out.is_null() {
        return K_RESULT_FALSE;
    }
    *out = ptr::null_mut();
    if !iid.is_null() && (*iid == FUNKNOWN_IID || *iid == IBSTREAM_IID) {
        *out = this;
        return K_RESULT_OK;
    }
    K_NO_INTERFACE
}

unsafe extern "C" fn stream_add_ref(_this: *mut c_void) -> u32 {
    1
}

unsafe extern "C" fn stream_release(_this: *mut c_void) -> u32 {
    1
}

pub(crate) unsafe extern "C" fn stream_read(
    this: *mut c_void,
    buffer: *mut c_void,
    requested: i32,
    read: *mut i32,
) -> Tresult {
    if this.is_null() || buffer.is_null() || requested < 0 {
        return K_RESULT_FALSE;
    }
    let stream = &mut *(this as *mut MemoryStream);
    let available = stream.bytes.len().saturating_sub(stream.position);
    let count = available.min(requested as usize);
    ptr::copy_nonoverlapping(
        stream.bytes.as_ptr().add(stream.position),
        buffer.cast::<u8>(),
        count,
    );
    stream.position += count;
    if !read.is_null() {
        *read = count as i32;
    }
    K_RESULT_OK
}

pub(crate) unsafe extern "C" fn stream_write(
    this: *mut c_void,
    buffer: *const c_void,
    requested: i32,
    written: *mut i32,
) -> Tresult {
    if this.is_null() || buffer.is_null() || requested < 0 {
        return K_RESULT_FALSE;
    }
    let stream = &mut *(this as *mut MemoryStream);
    if !stream.writable {
        return K_RESULT_FALSE;
    }
    let count = requested as usize;
    let end = match stream.position.checked_add(count) {
        Some(end) => end,
        None => return K_RESULT_FALSE,
    };
    if end > stream.bytes.len() {
        stream.bytes.resize(end, 0);
    }
    ptr::copy_nonoverlapping(
        buffer.cast::<u8>(),
        stream.bytes.as_mut_ptr().add(stream.position),
        count,
    );
    stream.position = end;
    if !written.is_null() {
        *written = requested;
    }
    K_RESULT_OK
}

pub(crate) unsafe extern "C" fn stream_seek(
    this: *mut c_void,
    offset: i64,
    mode: i32,
    result: *mut i64,
) -> Tresult {
    if this.is_null() {
        return K_RESULT_FALSE;
    }
    let stream = &mut *(this as *mut MemoryStream);
    let base = match mode {
        0 => 0i64,
        1 => stream.position as i64,
        2 => stream.bytes.len() as i64,
        _ => return K_RESULT_FALSE,
    };
    let Some(position) = base.checked_add(offset) else {
        return K_RESULT_FALSE;
    };
    if position < 0 {
        return K_RESULT_FALSE;
    }
    stream.position = position as usize;
    if !result.is_null() {
        *result = position;
    }
    K_RESULT_OK
}

pub(crate) unsafe extern "C" fn stream_tell(this: *mut c_void, position: *mut i64) -> Tresult {
    if this.is_null() || position.is_null() {
        return K_RESULT_FALSE;
    }
    *position = (*(this as *mut MemoryStream)).position as i64;
    K_RESULT_OK
}

pub(crate) static MEMORY_STREAM_VTABLE: MemoryStreamVTable = MemoryStreamVTable {
    query_interface: stream_query_interface,
    add_ref: stream_add_ref,
    release: stream_release,
    read: stream_read,
    write: stream_write,
    seek: stream_seek,
    tell: stream_tell,
};

pub(crate) const STATE_ENVELOPE_MAGIC: &[u8; 8] = b"SCV3ST\0\x01";

pub(crate) fn encode_state_envelope(component: &[u8], controller: &[u8]) -> Vec<u8> {
    let mut result = Vec::with_capacity(24 + component.len() + controller.len());
    result.extend_from_slice(STATE_ENVELOPE_MAGIC);
    result.extend_from_slice(&(component.len() as u64).to_le_bytes());
    result.extend_from_slice(&(controller.len() as u64).to_le_bytes());
    result.extend_from_slice(component);
    result.extend_from_slice(controller);
    result
}

pub(crate) fn decode_state_envelope(bytes: &[u8]) -> Option<(&[u8], &[u8])> {
    if bytes.len() < 24 || &bytes[..8] != STATE_ENVELOPE_MAGIC {
        return None;
    }
    let component_len = u64::from_le_bytes(bytes[8..16].try_into().ok()?) as usize;
    let controller_len = u64::from_le_bytes(bytes[16..24].try_into().ok()?) as usize;
    let component_end = 24usize.checked_add(component_len)?;
    let controller_end = component_end.checked_add(controller_len)?;
    if controller_end != bytes.len() {
        return None;
    }
    Some((
        &bytes[24..component_end],
        &bytes[component_end..controller_end],
    ))
}

// Bus/processing constants.
pub(crate) const K_AUDIO: i32 = 0;
pub(crate) const K_INPUT: i32 = 0;
pub(crate) const K_OUTPUT: i32 = 1;
pub(crate) const K_MAIN: i32 = 0;
pub(crate) const K_REALTIME: i32 = 0;
pub(crate) const K_SAMPLE32: i32 = 0;
pub(crate) const K_PROJECT_TIME_MUSIC_VALID: u32 = 1 << 9;
pub(crate) const K_TEMPO_VALID: u32 = 1 << 10;
pub(crate) const K_BAR_POSITION_VALID: u32 = 1 << 11;
pub(crate) const K_TIME_SIG_VALID: u32 = 1 << 13;
pub(crate) const K_CONT_TIME_VALID: u32 = 1 << 17;
/// `kSpeakerL | kSpeakerR`.
pub(crate) const STEREO_ARRANGEMENT: u64 = 0x3;

// ParameterInfo flags.
pub(crate) const PARAM_CAN_AUTOMATE: i32 = 1;
pub(crate) const PARAM_IS_READ_ONLY: i32 = 1 << 1;
pub(crate) const PARAM_IS_HIDDEN: i32 = 1 << 4;
pub(crate) const PARAM_IS_BYPASS: i32 = 1 << 16;
/// `RestartFlags::kLatencyChanged` from `ivsteditcontroller.h`.
pub const VST3_RESTART_LATENCY_CHANGED: u32 = 1 << 3;
/// `RestartFlags::kIoChanged` from `ivsteditcontroller.h`.
pub const VST3_RESTART_IO_CHANGED: u32 = 1 << 1;
pub(crate) const RESTART_PROCESSING_MASK: u32 =
    VST3_RESTART_IO_CHANGED | VST3_RESTART_LATENCY_CHANGED;

/// `kNotImplemented` (platform-dependent: COM `E_NOTIMPL` on Windows).
#[cfg(target_os = "windows")]
pub(crate) const K_NOT_IMPLEMENTED: Tresult = 0x8000_4001_u32 as i32;
#[cfg(not(target_os = "windows"))]
pub(crate) const K_NOT_IMPLEMENTED: Tresult = 3;

/// `FUnknown` method prefix shared by every vtable below.
#[repr(C)]
pub(crate) struct FUnknownVTable {
    pub(crate) query_interface:
        unsafe extern "C" fn(*mut c_void, *const Tuid, *mut *mut c_void) -> Tresult,
    pub(crate) add_ref: unsafe extern "C" fn(*mut c_void) -> u32,
    pub(crate) release: unsafe extern "C" fn(*mut c_void) -> u32,
}

/// `IPluginFactory` (mirrors the introspection module's layout, plus typed
/// `createInstance` arguments for hosting).
#[repr(C)]
pub(crate) struct PluginFactoryVTable {
    pub(crate) query_interface:
        unsafe extern "C" fn(*mut c_void, *const Tuid, *mut *mut c_void) -> Tresult,
    pub(crate) add_ref: unsafe extern "C" fn(*mut c_void) -> u32,
    pub(crate) release: unsafe extern "C" fn(*mut c_void) -> u32,
    pub(crate) get_factory_info: unsafe extern "C" fn(*mut c_void, *mut c_void) -> Tresult,
    pub(crate) count_classes: unsafe extern "C" fn(*mut c_void) -> i32,
    pub(crate) get_class_info: unsafe extern "C" fn(*mut c_void, i32, *mut c_void) -> Tresult,
    pub(crate) create_instance:
        unsafe extern "C" fn(*mut c_void, *const u8, *const u8, *mut *mut c_void) -> Tresult,
}

/// `PClassInfo` prefix used to distinguish component classes when a vendor's
/// `moduleinfo.json` advertises a stale class ID.
#[repr(C)]
pub(crate) struct FactoryClassInfo {
    pub(crate) cid: Tuid,
    pub(crate) cardinality: i32,
    pub(crate) category: [c_char; 32],
    pub(crate) name: [c_char; 64],
}

/// `IPluginFactory2` prefix needed to reach the `IPluginFactory3` extension.
#[repr(C)]
pub(crate) struct PluginFactory2VTable {
    pub(crate) base: PluginFactoryVTable,
    pub(crate) get_class_info_2: unsafe extern "C" fn(*mut c_void, i32, *mut c_void) -> Tresult,
}

/// `IPluginFactory3` adds Unicode class metadata and a factory-level host
/// context supplied before class enumeration or instance creation.
#[repr(C)]
pub(crate) struct PluginFactory3VTable {
    pub(crate) base: PluginFactory2VTable,
    pub(crate) get_class_info_unicode:
        unsafe extern "C" fn(*mut c_void, i32, *mut c_void) -> Tresult,
    pub(crate) set_host_context: unsafe extern "C" fn(*mut c_void, *mut c_void) -> Tresult,
}

/// `Steinberg::Vst::BusInfo`.
#[repr(C)]
pub(crate) struct BusInfo {
    pub(crate) media_type: i32,
    pub(crate) direction: i32,
    pub(crate) channel_count: i32,
    pub(crate) name: [i16; 128],
    pub(crate) bus_type: i32,
    pub(crate) flags: u32,
}

impl BusInfo {
    pub(crate) fn zeroed() -> Self {
        Self {
            media_type: 0,
            direction: 0,
            channel_count: 0,
            name: [0; 128],
            bus_type: 0,
            flags: 0,
        }
    }
}

/// `IComponent` (FUnknown + IPluginBase + IComponent methods, in order).
#[repr(C)]
pub(crate) struct ComponentVTable {
    pub(crate) query_interface:
        unsafe extern "C" fn(*mut c_void, *const Tuid, *mut *mut c_void) -> Tresult,
    pub(crate) add_ref: unsafe extern "C" fn(*mut c_void) -> u32,
    pub(crate) release: unsafe extern "C" fn(*mut c_void) -> u32,
    pub(crate) initialize: unsafe extern "C" fn(*mut c_void, *mut c_void) -> Tresult,
    pub(crate) terminate: unsafe extern "C" fn(*mut c_void) -> Tresult,
    pub(crate) get_controller_class_id: unsafe extern "C" fn(*mut c_void, *mut Tuid) -> Tresult,
    pub(crate) set_io_mode: unsafe extern "C" fn(*mut c_void, i32) -> Tresult,
    pub(crate) get_bus_count: unsafe extern "C" fn(*mut c_void, i32, i32) -> i32,
    pub(crate) get_bus_info:
        unsafe extern "C" fn(*mut c_void, i32, i32, i32, *mut BusInfo) -> Tresult,
    pub(crate) get_routing_info:
        unsafe extern "C" fn(*mut c_void, *mut c_void, *mut c_void) -> Tresult,
    pub(crate) activate_bus: unsafe extern "C" fn(*mut c_void, i32, i32, i32, u8) -> Tresult,
    pub(crate) set_active: unsafe extern "C" fn(*mut c_void, u8) -> Tresult,
    pub(crate) set_state: unsafe extern "C" fn(*mut c_void, *mut c_void) -> Tresult,
    pub(crate) get_state: unsafe extern "C" fn(*mut c_void, *mut c_void) -> Tresult,
}

/// `Steinberg::Vst::ProcessSetup`.
#[repr(C)]
pub(crate) struct ProcessSetup {
    pub(crate) process_mode: i32,
    pub(crate) symbolic_sample_size: i32,
    pub(crate) max_samples_per_block: i32,
    pub(crate) sample_rate: f64,
}

/// `Steinberg::Vst::AudioBusBuffers` (32-bit float member of the union).
#[repr(C)]
pub(crate) struct AudioBusBuffers {
    pub(crate) num_channels: i32,
    pub(crate) silence_flags: u64,
    pub(crate) channel_buffers32: *mut *mut f32,
}

/// `Steinberg::Vst::ProcessData` (input parameter changes live per
/// g12.023; event queues still null).
#[repr(C)]
pub(crate) struct ProcessData {
    pub(crate) process_mode: i32,
    pub(crate) symbolic_sample_size: i32,
    pub(crate) num_samples: i32,
    pub(crate) num_inputs: i32,
    pub(crate) num_outputs: i32,
    pub(crate) inputs: *mut AudioBusBuffers,
    pub(crate) outputs: *mut AudioBusBuffers,
    pub(crate) input_parameter_changes: *mut c_void,
    pub(crate) output_parameter_changes: *mut c_void,
    pub(crate) input_events: *mut c_void,
    pub(crate) output_events: *mut c_void,
    pub(crate) process_context: *mut c_void,
}

/// `Steinberg::Vst::Chord`.
#[repr(C)]
pub(crate) struct ProcessChord {
    pub(crate) key_note: u8,
    pub(crate) root_note: u8,
    pub(crate) chord_mask: i16,
}

/// `Steinberg::Vst::FrameRate`.
#[repr(C)]
pub(crate) struct ProcessFrameRate {
    pub(crate) frames_per_second: u32,
    pub(crate) flags: u32,
}

/// `Steinberg::Vst::ProcessContext`.
#[repr(C)]
pub(crate) struct ProcessContext {
    pub(crate) state: u32,
    pub(crate) sample_rate: f64,
    pub(crate) project_time_samples: i64,
    pub(crate) system_time: i64,
    pub(crate) continuous_time_samples: i64,
    pub(crate) project_time_music: f64,
    pub(crate) bar_position_music: f64,
    pub(crate) cycle_start_music: f64,
    pub(crate) cycle_end_music: f64,
    pub(crate) tempo: f64,
    pub(crate) time_sig_numerator: i32,
    pub(crate) time_sig_denominator: i32,
    pub(crate) chord: ProcessChord,
    pub(crate) smpte_offset_subframes: i32,
    pub(crate) frame_rate: ProcessFrameRate,
    pub(crate) samples_to_next_clock: i32,
}

/// `IAudioProcessor`.
#[repr(C)]
pub(crate) struct AudioProcessorVTable {
    pub(crate) query_interface:
        unsafe extern "C" fn(*mut c_void, *const Tuid, *mut *mut c_void) -> Tresult,
    pub(crate) add_ref: unsafe extern "C" fn(*mut c_void) -> u32,
    pub(crate) release: unsafe extern "C" fn(*mut c_void) -> u32,
    pub(crate) set_bus_arrangements:
        unsafe extern "C" fn(*mut c_void, *mut u64, i32, *mut u64, i32) -> Tresult,
    pub(crate) get_bus_arrangement:
        unsafe extern "C" fn(*mut c_void, i32, i32, *mut u64) -> Tresult,
    pub(crate) can_process_sample_size: unsafe extern "C" fn(*mut c_void, i32) -> Tresult,
    pub(crate) get_latency_samples: unsafe extern "C" fn(*mut c_void) -> u32,
    pub(crate) setup_processing: unsafe extern "C" fn(*mut c_void, *mut ProcessSetup) -> Tresult,
    pub(crate) set_processing: unsafe extern "C" fn(*mut c_void, u8) -> Tresult,
    pub(crate) process: unsafe extern "C" fn(*mut c_void, *mut ProcessData) -> Tresult,
    pub(crate) get_tail_samples: unsafe extern "C" fn(*mut c_void) -> u32,
}

/// `Steinberg::Vst::ParameterInfo`.
#[repr(C)]
pub(crate) struct ParameterInfo {
    pub(crate) id: u32,
    pub(crate) title: [i16; 128],
    pub(crate) short_title: [i16; 128],
    pub(crate) units: [i16; 128],
    pub(crate) step_count: i32,
    pub(crate) default_normalized_value: f64,
    pub(crate) unit_id: i32,
    pub(crate) flags: i32,
}

impl ParameterInfo {
    pub(crate) fn zeroed() -> Self {
        Self {
            id: 0,
            title: [0; 128],
            short_title: [0; 128],
            units: [0; 128],
            step_count: 0,
            default_normalized_value: 0.0,
            unit_id: 0,
            flags: 0,
        }
    }
}

/// `IEditController` (FUnknown + IPluginBase + IEditController, in order).
#[repr(C)]
pub(crate) struct EditControllerVTable {
    pub(crate) query_interface:
        unsafe extern "C" fn(*mut c_void, *const Tuid, *mut *mut c_void) -> Tresult,
    pub(crate) add_ref: unsafe extern "C" fn(*mut c_void) -> u32,
    pub(crate) release: unsafe extern "C" fn(*mut c_void) -> u32,
    pub(crate) initialize: unsafe extern "C" fn(*mut c_void, *mut c_void) -> Tresult,
    pub(crate) terminate: unsafe extern "C" fn(*mut c_void) -> Tresult,
    pub(crate) set_component_state: unsafe extern "C" fn(*mut c_void, *mut c_void) -> Tresult,
    pub(crate) set_state: unsafe extern "C" fn(*mut c_void, *mut c_void) -> Tresult,
    pub(crate) get_state: unsafe extern "C" fn(*mut c_void, *mut c_void) -> Tresult,
    pub(crate) get_parameter_count: unsafe extern "C" fn(*mut c_void) -> i32,
    pub(crate) get_parameter_info:
        unsafe extern "C" fn(*mut c_void, i32, *mut ParameterInfo) -> Tresult,
    pub(crate) get_param_string_by_value:
        unsafe extern "C" fn(*mut c_void, u32, f64, *mut i16) -> Tresult,
    pub(crate) get_param_value_by_string:
        unsafe extern "C" fn(*mut c_void, u32, *mut i16, *mut f64) -> Tresult,
    pub(crate) normalized_param_to_plain: unsafe extern "C" fn(*mut c_void, u32, f64) -> f64,
    pub(crate) plain_param_to_normalized: unsafe extern "C" fn(*mut c_void, u32, f64) -> f64,
    pub(crate) get_param_normalized: unsafe extern "C" fn(*mut c_void, u32) -> f64,
    pub(crate) set_param_normalized: unsafe extern "C" fn(*mut c_void, u32, f64) -> Tresult,
    pub(crate) set_component_handler: unsafe extern "C" fn(*mut c_void, *mut c_void) -> Tresult,
    pub(crate) create_view:
        unsafe extern "C" fn(*mut c_void, *const std::ffi::c_char) -> *mut c_void,
}

/// Minimal `IComponentHandler` receiving controller edit and restart calls.
#[repr(C)]
pub(crate) struct ComponentHandlerVTable {
    pub(crate) query_interface:
        unsafe extern "C" fn(*mut c_void, *const Tuid, *mut *mut c_void) -> Tresult,
    pub(crate) add_ref: unsafe extern "C" fn(*mut c_void) -> u32,
    pub(crate) release: unsafe extern "C" fn(*mut c_void) -> u32,
    pub(crate) begin_edit: unsafe extern "C" fn(*mut c_void, u32) -> Tresult,
    pub(crate) perform_edit: unsafe extern "C" fn(*mut c_void, u32, f64) -> Tresult,
    pub(crate) end_edit: unsafe extern "C" fn(*mut c_void, u32) -> Tresult,
    pub(crate) restart_component: unsafe extern "C" fn(*mut c_void, i32) -> Tresult,
}

#[repr(C)]
pub(crate) struct ComponentHandler {
    pub(crate) vtable: *const ComponentHandlerVTable,
    pub(crate) latency_changes: AtomicU64,
    pub(crate) pending_restart_flags: Arc<AtomicU32>,
}

unsafe impl Send for ComponentHandler {}
unsafe impl Sync for ComponentHandler {}

pub(crate) static COMPONENT_HANDLER_VTABLE: ComponentHandlerVTable = ComponentHandlerVTable {
    query_interface: component_handler_query_interface,
    add_ref: component_handler_add_ref,
    release: component_handler_release,
    begin_edit: component_handler_begin_edit,
    perform_edit: component_handler_perform_edit,
    end_edit: component_handler_end_edit,
    restart_component: component_handler_restart_component,
};

unsafe extern "C" fn component_handler_query_interface(
    this: *mut c_void,
    iid: *const Tuid,
    out: *mut *mut c_void,
) -> Tresult {
    if out.is_null() {
        return K_NO_INTERFACE;
    }
    if !iid.is_null() && (*iid == FUNKNOWN_IID || *iid == ICOMPONENT_HANDLER_IID) {
        *out = this;
        return K_RESULT_OK;
    }
    *out = ptr::null_mut();
    K_NO_INTERFACE
}

unsafe extern "C" fn component_handler_add_ref(_this: *mut c_void) -> u32 {
    1
}

unsafe extern "C" fn component_handler_release(_this: *mut c_void) -> u32 {
    1
}

unsafe extern "C" fn component_handler_begin_edit(_this: *mut c_void, _id: u32) -> Tresult {
    K_RESULT_OK
}

unsafe extern "C" fn component_handler_perform_edit(
    _this: *mut c_void,
    _id: u32,
    _value: f64,
) -> Tresult {
    K_RESULT_OK
}

unsafe extern "C" fn component_handler_end_edit(_this: *mut c_void, _id: u32) -> Tresult {
    K_RESULT_OK
}

unsafe extern "C" fn component_handler_restart_component(this: *mut c_void, flags: i32) -> Tresult {
    if this.is_null() {
        return K_NOT_IMPLEMENTED;
    }
    let supported = (flags as u32) & RESTART_PROCESSING_MASK;
    if supported == 0 || supported != flags as u32 {
        return K_NOT_IMPLEMENTED;
    }
    let handler = &*(this.cast::<ComponentHandler>());
    if supported & VST3_RESTART_LATENCY_CHANGED != 0 {
        handler.latency_changes.fetch_add(1, Ordering::Relaxed);
    }
    handler
        .pending_restart_flags
        .fetch_or(supported, Ordering::Release);
    K_RESULT_OK
}

/// Read a COM object's vtable of type `V`.
///
/// # Safety
/// `object` must be a live COM interface pointer whose vtable matches `V`.
pub(crate) unsafe fn vtable_of<V>(object: *mut c_void) -> *const V {
    *(object as *mut *const V)
}

/// `FUnknown::queryInterface` returning an owned (addRef'd) pointer.
pub(crate) unsafe fn com_query_interface(object: *mut c_void, iid: &Tuid) -> Option<*mut c_void> {
    let vtable = vtable_of::<FUnknownVTable>(object);
    let mut out: *mut c_void = ptr::null_mut();
    let result = ((*vtable).query_interface)(object, iid, &mut out);
    (result == K_RESULT_OK && !out.is_null()).then_some(out)
}

/// `FUnknown::release`.
pub(crate) unsafe fn com_release(object: *mut c_void) {
    let vtable = vtable_of::<FUnknownVTable>(object);
    ((*vtable).release)(object);
}

#[cfg(test)]
pub(crate) mod component_handler_tests {
    use super::*;

    #[test]
    fn only_supported_processing_restart_flags_are_accepted_and_queued() {
        let pending_restart_flags = Arc::new(AtomicU32::new(0));
        let mut handler = Box::new(ComponentHandler {
            vtable: &COMPONENT_HANDLER_VTABLE,
            latency_changes: AtomicU64::new(0),
            pending_restart_flags: Arc::clone(&pending_restart_flags),
        });
        let ptr = (&mut *handler as *mut ComponentHandler).cast();

        let io =
            unsafe { component_handler_restart_component(ptr, VST3_RESTART_IO_CHANGED as i32) };
        let latency = unsafe {
            component_handler_restart_component(ptr, VST3_RESTART_LATENCY_CHANGED as i32)
        };
        let reload = unsafe { component_handler_restart_component(ptr, 1) };

        assert_eq!(io, K_RESULT_OK);
        assert_eq!(latency, K_RESULT_OK);
        assert_eq!(reload, K_NOT_IMPLEMENTED);
        assert_eq!(handler.latency_changes.load(Ordering::Relaxed), 1);
        assert_eq!(
            pending_restart_flags.load(Ordering::Acquire),
            RESTART_PROCESSING_MASK
        );
    }
}
