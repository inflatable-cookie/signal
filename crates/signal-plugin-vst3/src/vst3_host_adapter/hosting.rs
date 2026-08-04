//! In-child VST3 instance hosting: module/factory loading, instance
//! lifecycle (create/initialize/activate/setProcessing), parameter inventory
//! via `IEditController`, and a raw process session for the sandbox audio
//! thread — the VST3 mirror of `signal-plugin-clap`'s hosting module.
//!
//! # FFI design
//!
//! The COM surface is handwritten, extending the introspection module's
//! factory-enumeration FFI (no `vst3-sys`, no Steinberg SDK code). Each
//! interface is a `#[repr(C)]` vtable whose first three slots are the
//! `FUnknown` methods (`queryInterface`/`addRef`/`release`); base-interface
//! methods precede derived methods in declaration order. On macOS and Linux
//! VST3 uses the plain C calling convention (`extern "C"`); the historical
//! thiscall concern is Windows/x86-only and out of scope here.
//!
//! # TUID byte order
//!
//! Interface and class IDs are 16-byte TUIDs. Steinberg's `INLINE_UID`
//! stores the four canonical `u32` fields big-endian on non-Windows
//! platforms, but COM-compatible (first field and the two 16-bit halves of
//! the second byte-swapped little-endian) on Windows. [`tuid_from_uid`]
//! encodes that per-platform. Catalog load keys are the *raw in-memory*
//! TUID hex exactly as the introspection module reports `PClassInfo` CIDs
//! (and as conforming `moduleinfo.json` files carry them on non-Windows), so
//! [`tuid_from_class_id_hex`] is a straight hex decode on macOS/Linux and
//! applies the COM swap only on Windows.

#![allow(unsafe_op_in_unsafe_fn)]
// The COM vtables mirror Steinberg's interface layouts, which are wider
// than clippy's default argument budget for a few methods.
#![allow(clippy::too_many_arguments)]

use std::collections::HashMap;
use std::ffi::{c_char, c_void, CStr, CString};
use std::path::Path;
use std::ptr;
use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use signal_plugin::{
    PluginEvent, PluginParamChange, PluginParamChangeQueue, PluginParameterDescriptor,
    PluginParameterDomain, PluginParameterFlags, PLUGIN_PARAM_CHANGE_CAPACITY,
};

#[cfg(not(target_os = "macos"))]
use libloading::Library;

use super::ara::AraInspectionSession;
use super::gui::{Vst3GuiEvent, Vst3GuiSession, IPLUG_VIEW_IID, VIEW_TYPE_EDITOR};
use super::Vst3HostPlatform;

#[cfg(not(target_os = "macos"))]
use super::introspection::resolve_module_binary_path;

/// Error surface for VST3 hosting operations; carries a stable snake_case
/// token suitable for broker receipt details (mirrors `ClapHostingError`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Vst3HostingError {
    /// Stable snake_case failure token (e.g. `module_open_failed`).
    pub token: String,
}

impl Vst3HostingError {
    pub(crate) fn new(token: impl Into<String>) -> Self {
        Self {
            token: token.into(),
        }
    }
}

impl std::fmt::Display for Vst3HostingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.token)
    }
}

impl std::error::Error for Vst3HostingError {}

/// The build-target VST3 platform (module layout + entry symbol names).
pub const fn current_vst3_platform() -> Vst3HostPlatform {
    if cfg!(target_os = "macos") {
        Vst3HostPlatform::MacOs
    } else if cfg!(target_os = "windows") {
        Vst3HostPlatform::Windows
    } else {
        Vst3HostPlatform::Linux
    }
}

// ── COM primitives ──────────────────────────────────────────────────────────

/// Steinberg `tresult`.
pub(crate) type Tresult = i32;

/// `kResultOk` / `kResultTrue` (0 on every platform).
pub(crate) const K_RESULT_OK: Tresult = 0;
const K_RESULT_FALSE: Tresult = 1;

/// `kNoInterface` (platform-dependent: COM `E_NOINTERFACE` on Windows).
#[cfg(target_os = "windows")]
pub(crate) const K_NO_INTERFACE: Tresult = 0x8000_4002_u32 as i32;
#[cfg(not(target_os = "windows"))]
pub(crate) const K_NO_INTERFACE: Tresult = -1;

/// 16-byte Steinberg TUID.
pub(crate) type Tuid = [u8; 16];

/// Build a TUID from the four canonical `u32` fields with the platform's
/// `INLINE_UID` byte layout (see module docs).
pub(crate) const fn tuid_from_uid(l1: u32, l2: u32, l3: u32, l4: u32) -> Tuid {
    if cfg!(target_os = "windows") {
        [
            (l1 & 0xFF) as u8,
            ((l1 >> 8) & 0xFF) as u8,
            ((l1 >> 16) & 0xFF) as u8,
            ((l1 >> 24) & 0xFF) as u8,
            ((l2 >> 16) & 0xFF) as u8,
            ((l2 >> 24) & 0xFF) as u8,
            (l2 & 0xFF) as u8,
            ((l2 >> 8) & 0xFF) as u8,
            ((l3 >> 24) & 0xFF) as u8,
            ((l3 >> 16) & 0xFF) as u8,
            ((l3 >> 8) & 0xFF) as u8,
            (l3 & 0xFF) as u8,
            ((l4 >> 24) & 0xFF) as u8,
            ((l4 >> 16) & 0xFF) as u8,
            ((l4 >> 8) & 0xFF) as u8,
            (l4 & 0xFF) as u8,
        ]
    } else {
        [
            ((l1 >> 24) & 0xFF) as u8,
            ((l1 >> 16) & 0xFF) as u8,
            ((l1 >> 8) & 0xFF) as u8,
            (l1 & 0xFF) as u8,
            ((l2 >> 24) & 0xFF) as u8,
            ((l2 >> 16) & 0xFF) as u8,
            ((l2 >> 8) & 0xFF) as u8,
            (l2 & 0xFF) as u8,
            ((l3 >> 24) & 0xFF) as u8,
            ((l3 >> 16) & 0xFF) as u8,
            ((l3 >> 8) & 0xFF) as u8,
            (l3 & 0xFF) as u8,
            ((l4 >> 24) & 0xFF) as u8,
            ((l4 >> 16) & 0xFF) as u8,
            ((l4 >> 8) & 0xFF) as u8,
            (l4 & 0xFF) as u8,
        ]
    }
}

/// Decode a catalog load key (raw in-memory TUID hex on non-Windows, the
/// canonical class-ID hex everywhere) into the in-memory TUID.
fn tuid_from_class_id_hex(class_id_hex: &str) -> Option<Tuid> {
    let hex = class_id_hex.trim();
    if hex.len() != 32 || !hex.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return None;
    }
    let mut bytes = [0u8; 16];
    for (index, slot) in bytes.iter_mut().enumerate() {
        *slot = u8::from_str_radix(&hex[index * 2..index * 2 + 2], 16).ok()?;
    }
    if cfg!(target_os = "windows") {
        // Canonical hex → COM in-memory layout: swap the first 4-byte field
        // and the two following 2-byte fields.
        let mut swapped = bytes;
        swapped[0..4].reverse();
        swapped[4..6].reverse();
        swapped[6..8].reverse();
        Some(swapped)
    } else {
        Some(bytes)
    }
}

// Interface IIDs (canonical field values from the published VST3 interface
// definitions; encoded per-platform by `tuid_from_uid`).
pub(crate) const FUNKNOWN_IID: Tuid = tuid_from_uid(0x00000000, 0x00000000, 0xC0000000, 0x00000046);
const ICOMPONENT_IID: Tuid = tuid_from_uid(0xE831FF31, 0xF2D54301, 0x928EBBEE, 0x25697802);
const IAUDIO_PROCESSOR_IID: Tuid = tuid_from_uid(0x42043F99, 0xB7DA453C, 0xA569E79D, 0x9AAEC33D);
const IEDIT_CONTROLLER_IID: Tuid = tuid_from_uid(0xDCD7BBE3, 0x7742448D, 0xA874AACC, 0x979C759E);
const IPLUGIN_FACTORY_3_IID: Tuid = tuid_from_uid(0x4555A2AB, 0xC1234E57, 0x9B122910, 0x36878931);
const ICOMPONENT_HANDLER_IID: Tuid = tuid_from_uid(0x93A0BEA3, 0x0BD045DB, 0x8E890B0C, 0xC1E46AC6);
const IHOST_APPLICATION_IID: Tuid = tuid_from_uid(0x58E595CC, 0xDB2D4969, 0x8B6AAF8C, 0x36A664E5);
const IMESSAGE_IID: Tuid = tuid_from_uid(0x936F033B, 0xC6C047DB, 0xBB0882F8, 0x13C1E613);
const IATTRIBUTE_LIST_IID: Tuid = tuid_from_uid(0x1E5F0AEB, 0xCC7F4533, 0xA2544011, 0x38AD5EE4);
// ivstparameterchanges.h (published interface definitions).
const IPARAMETER_CHANGES_IID: Tuid = tuid_from_uid(0xA4779663, 0x0BB64A56, 0xB44384A8, 0x466FEB9D);
const IPARAM_VALUE_QUEUE_IID: Tuid = tuid_from_uid(0x01263A18, 0xED074F6F, 0x98C9D356, 0x4686F9BA);
// ivstevents.h / ivstmidicontrollers.h (published interface definitions).
const IEVENT_LIST_IID: Tuid = tuid_from_uid(0x3A2C4214, 0x346349FE, 0xB2C4F397, 0xB9695A44);
const IMIDI_MAPPING_IID: Tuid = tuid_from_uid(0xDF695DF2, 0x8B4B47EB, 0xAB3EF8FB, 0x2D1F6BB2);
const ICONNECTION_POINT_IID: Tuid = tuid_from_uid(0x70A4156F, 0x6E6E4026, 0x989148BF, 0xAA60D8D1);
const IBSTREAM_IID: Tuid = tuid_from_uid(0xC3BF6EA2, 0x30994752, 0x9B6BF990, 0x1EE33E9B);

// ── Host-side IBStream for opaque component/controller state ───────────────

#[repr(C)]
struct MemoryStreamVTable {
    query_interface: unsafe extern "C" fn(*mut c_void, *const Tuid, *mut *mut c_void) -> Tresult,
    add_ref: unsafe extern "C" fn(*mut c_void) -> u32,
    release: unsafe extern "C" fn(*mut c_void) -> u32,
    read: unsafe extern "C" fn(*mut c_void, *mut c_void, i32, *mut i32) -> Tresult,
    write: unsafe extern "C" fn(*mut c_void, *const c_void, i32, *mut i32) -> Tresult,
    seek: unsafe extern "C" fn(*mut c_void, i64, i32, *mut i64) -> Tresult,
    tell: unsafe extern "C" fn(*mut c_void, *mut i64) -> Tresult,
}

#[repr(C)]
struct MemoryStream {
    vtable: *const MemoryStreamVTable,
    bytes: Vec<u8>,
    position: usize,
    writable: bool,
}

impl MemoryStream {
    fn writer() -> Self {
        Self {
            vtable: &MEMORY_STREAM_VTABLE,
            bytes: Vec::new(),
            position: 0,
            writable: true,
        }
    }

    fn reader(bytes: &[u8]) -> Self {
        Self {
            vtable: &MEMORY_STREAM_VTABLE,
            bytes: bytes.to_vec(),
            position: 0,
            writable: false,
        }
    }

    fn as_raw(&mut self) -> *mut c_void {
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

unsafe extern "C" fn stream_read(
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

unsafe extern "C" fn stream_write(
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

unsafe extern "C" fn stream_seek(
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

unsafe extern "C" fn stream_tell(this: *mut c_void, position: *mut i64) -> Tresult {
    if this.is_null() || position.is_null() {
        return K_RESULT_FALSE;
    }
    *position = (*(this as *mut MemoryStream)).position as i64;
    K_RESULT_OK
}

static MEMORY_STREAM_VTABLE: MemoryStreamVTable = MemoryStreamVTable {
    query_interface: stream_query_interface,
    add_ref: stream_add_ref,
    release: stream_release,
    read: stream_read,
    write: stream_write,
    seek: stream_seek,
    tell: stream_tell,
};

const STATE_ENVELOPE_MAGIC: &[u8; 8] = b"SCV3ST\0\x01";

fn encode_state_envelope(component: &[u8], controller: &[u8]) -> Vec<u8> {
    let mut result = Vec::with_capacity(24 + component.len() + controller.len());
    result.extend_from_slice(STATE_ENVELOPE_MAGIC);
    result.extend_from_slice(&(component.len() as u64).to_le_bytes());
    result.extend_from_slice(&(controller.len() as u64).to_le_bytes());
    result.extend_from_slice(component);
    result.extend_from_slice(controller);
    result
}

fn decode_state_envelope(bytes: &[u8]) -> Option<(&[u8], &[u8])> {
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
const K_AUDIO: i32 = 0;
const K_INPUT: i32 = 0;
const K_OUTPUT: i32 = 1;
const K_MAIN: i32 = 0;
const K_REALTIME: i32 = 0;
const K_SAMPLE32: i32 = 0;
const K_PROJECT_TIME_MUSIC_VALID: u32 = 1 << 9;
const K_TEMPO_VALID: u32 = 1 << 10;
const K_BAR_POSITION_VALID: u32 = 1 << 11;
const K_TIME_SIG_VALID: u32 = 1 << 13;
const K_CONT_TIME_VALID: u32 = 1 << 17;
/// `kSpeakerL | kSpeakerR`.
const STEREO_ARRANGEMENT: u64 = 0x3;

// ParameterInfo flags.
const PARAM_CAN_AUTOMATE: i32 = 1;
const PARAM_IS_READ_ONLY: i32 = 1 << 1;
const PARAM_IS_HIDDEN: i32 = 1 << 4;
const PARAM_IS_BYPASS: i32 = 1 << 16;
/// `RestartFlags::kLatencyChanged` from `ivsteditcontroller.h`.
pub const VST3_RESTART_LATENCY_CHANGED: u32 = 1 << 3;
/// `RestartFlags::kIoChanged` from `ivsteditcontroller.h`.
pub const VST3_RESTART_IO_CHANGED: u32 = 1 << 1;
const RESTART_PROCESSING_MASK: u32 = VST3_RESTART_IO_CHANGED | VST3_RESTART_LATENCY_CHANGED;

/// `kNotImplemented` (platform-dependent: COM `E_NOTIMPL` on Windows).
#[cfg(target_os = "windows")]
const K_NOT_IMPLEMENTED: Tresult = 0x8000_4001_u32 as i32;
#[cfg(not(target_os = "windows"))]
const K_NOT_IMPLEMENTED: Tresult = 3;

/// `FUnknown` method prefix shared by every vtable below.
#[repr(C)]
struct FUnknownVTable {
    query_interface: unsafe extern "C" fn(*mut c_void, *const Tuid, *mut *mut c_void) -> Tresult,
    add_ref: unsafe extern "C" fn(*mut c_void) -> u32,
    release: unsafe extern "C" fn(*mut c_void) -> u32,
}

/// `IPluginFactory` (mirrors the introspection module's layout, plus typed
/// `createInstance` arguments for hosting).
#[repr(C)]
struct PluginFactoryVTable {
    query_interface: unsafe extern "C" fn(*mut c_void, *const Tuid, *mut *mut c_void) -> Tresult,
    add_ref: unsafe extern "C" fn(*mut c_void) -> u32,
    release: unsafe extern "C" fn(*mut c_void) -> u32,
    get_factory_info: unsafe extern "C" fn(*mut c_void, *mut c_void) -> Tresult,
    count_classes: unsafe extern "C" fn(*mut c_void) -> i32,
    get_class_info: unsafe extern "C" fn(*mut c_void, i32, *mut c_void) -> Tresult,
    create_instance:
        unsafe extern "C" fn(*mut c_void, *const u8, *const u8, *mut *mut c_void) -> Tresult,
}

/// `PClassInfo` prefix used to distinguish component classes when a vendor's
/// `moduleinfo.json` advertises a stale class ID.
#[repr(C)]
struct FactoryClassInfo {
    cid: Tuid,
    cardinality: i32,
    category: [c_char; 32],
    name: [c_char; 64],
}

/// `IPluginFactory2` prefix needed to reach the `IPluginFactory3` extension.
#[repr(C)]
struct PluginFactory2VTable {
    base: PluginFactoryVTable,
    get_class_info_2: unsafe extern "C" fn(*mut c_void, i32, *mut c_void) -> Tresult,
}

/// `IPluginFactory3` adds Unicode class metadata and a factory-level host
/// context supplied before class enumeration or instance creation.
#[repr(C)]
struct PluginFactory3VTable {
    base: PluginFactory2VTable,
    get_class_info_unicode: unsafe extern "C" fn(*mut c_void, i32, *mut c_void) -> Tresult,
    set_host_context: unsafe extern "C" fn(*mut c_void, *mut c_void) -> Tresult,
}

/// `Steinberg::Vst::BusInfo`.
#[repr(C)]
struct BusInfo {
    media_type: i32,
    direction: i32,
    channel_count: i32,
    name: [i16; 128],
    bus_type: i32,
    flags: u32,
}

impl BusInfo {
    fn zeroed() -> Self {
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
struct ComponentVTable {
    query_interface: unsafe extern "C" fn(*mut c_void, *const Tuid, *mut *mut c_void) -> Tresult,
    add_ref: unsafe extern "C" fn(*mut c_void) -> u32,
    release: unsafe extern "C" fn(*mut c_void) -> u32,
    initialize: unsafe extern "C" fn(*mut c_void, *mut c_void) -> Tresult,
    terminate: unsafe extern "C" fn(*mut c_void) -> Tresult,
    get_controller_class_id: unsafe extern "C" fn(*mut c_void, *mut Tuid) -> Tresult,
    set_io_mode: unsafe extern "C" fn(*mut c_void, i32) -> Tresult,
    get_bus_count: unsafe extern "C" fn(*mut c_void, i32, i32) -> i32,
    get_bus_info: unsafe extern "C" fn(*mut c_void, i32, i32, i32, *mut BusInfo) -> Tresult,
    get_routing_info: unsafe extern "C" fn(*mut c_void, *mut c_void, *mut c_void) -> Tresult,
    activate_bus: unsafe extern "C" fn(*mut c_void, i32, i32, i32, u8) -> Tresult,
    set_active: unsafe extern "C" fn(*mut c_void, u8) -> Tresult,
    set_state: unsafe extern "C" fn(*mut c_void, *mut c_void) -> Tresult,
    get_state: unsafe extern "C" fn(*mut c_void, *mut c_void) -> Tresult,
}

/// `Steinberg::Vst::ProcessSetup`.
#[repr(C)]
struct ProcessSetup {
    process_mode: i32,
    symbolic_sample_size: i32,
    max_samples_per_block: i32,
    sample_rate: f64,
}

/// `Steinberg::Vst::AudioBusBuffers` (32-bit float member of the union).
#[repr(C)]
struct AudioBusBuffers {
    num_channels: i32,
    silence_flags: u64,
    channel_buffers32: *mut *mut f32,
}

/// `Steinberg::Vst::ProcessData` (input parameter changes live per
/// g12.023; event queues still null).
#[repr(C)]
struct ProcessData {
    process_mode: i32,
    symbolic_sample_size: i32,
    num_samples: i32,
    num_inputs: i32,
    num_outputs: i32,
    inputs: *mut AudioBusBuffers,
    outputs: *mut AudioBusBuffers,
    input_parameter_changes: *mut c_void,
    output_parameter_changes: *mut c_void,
    input_events: *mut c_void,
    output_events: *mut c_void,
    process_context: *mut c_void,
}

/// `Steinberg::Vst::Chord`.
#[repr(C)]
struct ProcessChord {
    key_note: u8,
    root_note: u8,
    chord_mask: i16,
}

/// `Steinberg::Vst::FrameRate`.
#[repr(C)]
struct ProcessFrameRate {
    frames_per_second: u32,
    flags: u32,
}

/// `Steinberg::Vst::ProcessContext`.
#[repr(C)]
struct ProcessContext {
    state: u32,
    sample_rate: f64,
    project_time_samples: i64,
    system_time: i64,
    continuous_time_samples: i64,
    project_time_music: f64,
    bar_position_music: f64,
    cycle_start_music: f64,
    cycle_end_music: f64,
    tempo: f64,
    time_sig_numerator: i32,
    time_sig_denominator: i32,
    chord: ProcessChord,
    smpte_offset_subframes: i32,
    frame_rate: ProcessFrameRate,
    samples_to_next_clock: i32,
}

/// `IAudioProcessor`.
#[repr(C)]
struct AudioProcessorVTable {
    query_interface: unsafe extern "C" fn(*mut c_void, *const Tuid, *mut *mut c_void) -> Tresult,
    add_ref: unsafe extern "C" fn(*mut c_void) -> u32,
    release: unsafe extern "C" fn(*mut c_void) -> u32,
    set_bus_arrangements:
        unsafe extern "C" fn(*mut c_void, *mut u64, i32, *mut u64, i32) -> Tresult,
    get_bus_arrangement: unsafe extern "C" fn(*mut c_void, i32, i32, *mut u64) -> Tresult,
    can_process_sample_size: unsafe extern "C" fn(*mut c_void, i32) -> Tresult,
    get_latency_samples: unsafe extern "C" fn(*mut c_void) -> u32,
    setup_processing: unsafe extern "C" fn(*mut c_void, *mut ProcessSetup) -> Tresult,
    set_processing: unsafe extern "C" fn(*mut c_void, u8) -> Tresult,
    process: unsafe extern "C" fn(*mut c_void, *mut ProcessData) -> Tresult,
    get_tail_samples: unsafe extern "C" fn(*mut c_void) -> u32,
}

/// `Steinberg::Vst::ParameterInfo`.
#[repr(C)]
struct ParameterInfo {
    id: u32,
    title: [i16; 128],
    short_title: [i16; 128],
    units: [i16; 128],
    step_count: i32,
    default_normalized_value: f64,
    unit_id: i32,
    flags: i32,
}

impl ParameterInfo {
    fn zeroed() -> Self {
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
struct EditControllerVTable {
    query_interface: unsafe extern "C" fn(*mut c_void, *const Tuid, *mut *mut c_void) -> Tresult,
    add_ref: unsafe extern "C" fn(*mut c_void) -> u32,
    release: unsafe extern "C" fn(*mut c_void) -> u32,
    initialize: unsafe extern "C" fn(*mut c_void, *mut c_void) -> Tresult,
    terminate: unsafe extern "C" fn(*mut c_void) -> Tresult,
    set_component_state: unsafe extern "C" fn(*mut c_void, *mut c_void) -> Tresult,
    set_state: unsafe extern "C" fn(*mut c_void, *mut c_void) -> Tresult,
    get_state: unsafe extern "C" fn(*mut c_void, *mut c_void) -> Tresult,
    get_parameter_count: unsafe extern "C" fn(*mut c_void) -> i32,
    get_parameter_info: unsafe extern "C" fn(*mut c_void, i32, *mut ParameterInfo) -> Tresult,
    get_param_string_by_value: unsafe extern "C" fn(*mut c_void, u32, f64, *mut i16) -> Tresult,
    get_param_value_by_string:
        unsafe extern "C" fn(*mut c_void, u32, *mut i16, *mut f64) -> Tresult,
    normalized_param_to_plain: unsafe extern "C" fn(*mut c_void, u32, f64) -> f64,
    plain_param_to_normalized: unsafe extern "C" fn(*mut c_void, u32, f64) -> f64,
    get_param_normalized: unsafe extern "C" fn(*mut c_void, u32) -> f64,
    set_param_normalized: unsafe extern "C" fn(*mut c_void, u32, f64) -> Tresult,
    set_component_handler: unsafe extern "C" fn(*mut c_void, *mut c_void) -> Tresult,
    create_view: unsafe extern "C" fn(*mut c_void, *const std::ffi::c_char) -> *mut c_void,
}

/// Minimal `IComponentHandler` receiving controller edit and restart calls.
#[repr(C)]
struct ComponentHandlerVTable {
    query_interface: unsafe extern "C" fn(*mut c_void, *const Tuid, *mut *mut c_void) -> Tresult,
    add_ref: unsafe extern "C" fn(*mut c_void) -> u32,
    release: unsafe extern "C" fn(*mut c_void) -> u32,
    begin_edit: unsafe extern "C" fn(*mut c_void, u32) -> Tresult,
    perform_edit: unsafe extern "C" fn(*mut c_void, u32, f64) -> Tresult,
    end_edit: unsafe extern "C" fn(*mut c_void, u32) -> Tresult,
    restart_component: unsafe extern "C" fn(*mut c_void, i32) -> Tresult,
}

#[repr(C)]
struct ComponentHandler {
    vtable: *const ComponentHandlerVTable,
    latency_changes: AtomicU64,
    pending_restart_flags: Arc<AtomicU32>,
}

unsafe impl Send for ComponentHandler {}
unsafe impl Sync for ComponentHandler {}

static COMPONENT_HANDLER_VTABLE: ComponentHandlerVTable = ComponentHandlerVTable {
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

#[cfg(test)]
mod component_handler_tests {
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

// ── Minimal host context (IHostApplication) ────────────────────────────────

enum HostAttribute {
    Integer(i64),
    Float(f64),
    String(Vec<i16>),
    Binary(Vec<u8>),
}

#[repr(C)]
struct HostAttributeListVTable {
    query_interface: unsafe extern "C" fn(*mut c_void, *const Tuid, *mut *mut c_void) -> Tresult,
    add_ref: unsafe extern "C" fn(*mut c_void) -> u32,
    release: unsafe extern "C" fn(*mut c_void) -> u32,
    set_int: unsafe extern "C" fn(*mut c_void, *const c_char, i64) -> Tresult,
    get_int: unsafe extern "C" fn(*mut c_void, *const c_char, *mut i64) -> Tresult,
    set_float: unsafe extern "C" fn(*mut c_void, *const c_char, f64) -> Tresult,
    get_float: unsafe extern "C" fn(*mut c_void, *const c_char, *mut f64) -> Tresult,
    set_string: unsafe extern "C" fn(*mut c_void, *const c_char, *const i16) -> Tresult,
    get_string: unsafe extern "C" fn(*mut c_void, *const c_char, *mut i16, u32) -> Tresult,
    set_binary: unsafe extern "C" fn(*mut c_void, *const c_char, *const c_void, u32) -> Tresult,
    get_binary:
        unsafe extern "C" fn(*mut c_void, *const c_char, *mut *const c_void, *mut u32) -> Tresult,
}

#[repr(C)]
struct HostAttributeList {
    vtable: *const HostAttributeListVTable,
    refs: AtomicU32,
    values: Mutex<HashMap<Vec<u8>, HostAttribute>>,
}

static HOST_ATTRIBUTE_LIST_VTABLE: HostAttributeListVTable = HostAttributeListVTable {
    query_interface: host_attribute_list_query_interface,
    add_ref: host_attribute_list_add_ref,
    release: host_attribute_list_release,
    set_int: host_attribute_list_set_int,
    get_int: host_attribute_list_get_int,
    set_float: host_attribute_list_set_float,
    get_float: host_attribute_list_get_float,
    set_string: host_attribute_list_set_string,
    get_string: host_attribute_list_get_string,
    set_binary: host_attribute_list_set_binary,
    get_binary: host_attribute_list_get_binary,
};

fn new_host_attribute_list() -> *mut c_void {
    Box::into_raw(Box::new(HostAttributeList {
        vtable: &HOST_ATTRIBUTE_LIST_VTABLE,
        refs: AtomicU32::new(1),
        values: Mutex::new(HashMap::new()),
    }))
    .cast()
}

unsafe fn attribute_key(id: *const c_char) -> Option<Vec<u8>> {
    (!id.is_null()).then(|| CStr::from_ptr(id).to_bytes().to_vec())
}

unsafe extern "C" fn host_attribute_list_query_interface(
    this: *mut c_void,
    iid: *const Tuid,
    out: *mut *mut c_void,
) -> Tresult {
    if out.is_null() {
        return K_NO_INTERFACE;
    }
    if !iid.is_null() && (*iid == FUNKNOWN_IID || *iid == IATTRIBUTE_LIST_IID) {
        *out = this;
        host_attribute_list_add_ref(this);
        return K_RESULT_OK;
    }
    *out = ptr::null_mut();
    K_NO_INTERFACE
}

unsafe extern "C" fn host_attribute_list_add_ref(this: *mut c_void) -> u32 {
    (*(this.cast::<HostAttributeList>()))
        .refs
        .fetch_add(1, Ordering::Relaxed)
        + 1
}

unsafe extern "C" fn host_attribute_list_release(this: *mut c_void) -> u32 {
    let remaining = (*(this.cast::<HostAttributeList>()))
        .refs
        .fetch_sub(1, Ordering::Release)
        - 1;
    if remaining == 0 {
        std::sync::atomic::fence(Ordering::Acquire);
        drop(Box::from_raw(this.cast::<HostAttributeList>()));
    }
    remaining
}

unsafe extern "C" fn host_attribute_list_set_int(
    this: *mut c_void,
    id: *const c_char,
    value: i64,
) -> Tresult {
    let Some(key) = attribute_key(id) else {
        return K_RESULT_FALSE;
    };
    (*(this.cast::<HostAttributeList>()))
        .values
        .lock()
        .expect("host attribute list poisoned")
        .insert(key, HostAttribute::Integer(value));
    K_RESULT_OK
}

unsafe extern "C" fn host_attribute_list_get_int(
    this: *mut c_void,
    id: *const c_char,
    value: *mut i64,
) -> Tresult {
    let Some(key) = attribute_key(id) else {
        return K_RESULT_FALSE;
    };
    if value.is_null() {
        return K_RESULT_FALSE;
    }
    match (*(this.cast::<HostAttributeList>()))
        .values
        .lock()
        .expect("host attribute list poisoned")
        .get(&key)
    {
        Some(HostAttribute::Integer(stored)) => {
            *value = *stored;
            K_RESULT_OK
        }
        _ => K_RESULT_FALSE,
    }
}

unsafe extern "C" fn host_attribute_list_set_float(
    this: *mut c_void,
    id: *const c_char,
    value: f64,
) -> Tresult {
    let Some(key) = attribute_key(id) else {
        return K_RESULT_FALSE;
    };
    (*(this.cast::<HostAttributeList>()))
        .values
        .lock()
        .expect("host attribute list poisoned")
        .insert(key, HostAttribute::Float(value));
    K_RESULT_OK
}

unsafe extern "C" fn host_attribute_list_get_float(
    this: *mut c_void,
    id: *const c_char,
    value: *mut f64,
) -> Tresult {
    let Some(key) = attribute_key(id) else {
        return K_RESULT_FALSE;
    };
    if value.is_null() {
        return K_RESULT_FALSE;
    }
    match (*(this.cast::<HostAttributeList>()))
        .values
        .lock()
        .expect("host attribute list poisoned")
        .get(&key)
    {
        Some(HostAttribute::Float(stored)) => {
            *value = *stored;
            K_RESULT_OK
        }
        _ => K_RESULT_FALSE,
    }
}

unsafe extern "C" fn host_attribute_list_set_string(
    this: *mut c_void,
    id: *const c_char,
    value: *const i16,
) -> Tresult {
    let Some(key) = attribute_key(id) else {
        return K_RESULT_FALSE;
    };
    if value.is_null() {
        return K_RESULT_FALSE;
    }
    let length = (0..).position(|index| *value.add(index) == 0).unwrap_or(0);
    let mut stored = std::slice::from_raw_parts(value, length).to_vec();
    stored.push(0);
    (*(this.cast::<HostAttributeList>()))
        .values
        .lock()
        .expect("host attribute list poisoned")
        .insert(key, HostAttribute::String(stored));
    K_RESULT_OK
}

unsafe extern "C" fn host_attribute_list_get_string(
    this: *mut c_void,
    id: *const c_char,
    value: *mut i16,
    size_in_bytes: u32,
) -> Tresult {
    let Some(key) = attribute_key(id) else {
        return K_RESULT_FALSE;
    };
    if value.is_null() {
        return K_RESULT_FALSE;
    }
    match (*(this.cast::<HostAttributeList>()))
        .values
        .lock()
        .expect("host attribute list poisoned")
        .get(&key)
    {
        Some(HostAttribute::String(stored)) => {
            let units = stored.len().min(size_in_bytes as usize / size_of::<i16>());
            ptr::copy_nonoverlapping(stored.as_ptr(), value, units);
            K_RESULT_OK
        }
        _ => K_RESULT_FALSE,
    }
}

unsafe extern "C" fn host_attribute_list_set_binary(
    this: *mut c_void,
    id: *const c_char,
    value: *const c_void,
    size_in_bytes: u32,
) -> Tresult {
    let Some(key) = attribute_key(id) else {
        return K_RESULT_FALSE;
    };
    if value.is_null() && size_in_bytes != 0 {
        return K_RESULT_FALSE;
    }
    let stored = if size_in_bytes == 0 {
        Vec::new()
    } else {
        std::slice::from_raw_parts(value.cast::<u8>(), size_in_bytes as usize).to_vec()
    };
    (*(this.cast::<HostAttributeList>()))
        .values
        .lock()
        .expect("host attribute list poisoned")
        .insert(key, HostAttribute::Binary(stored));
    K_RESULT_OK
}

unsafe extern "C" fn host_attribute_list_get_binary(
    this: *mut c_void,
    id: *const c_char,
    value: *mut *const c_void,
    size_in_bytes: *mut u32,
) -> Tresult {
    let Some(key) = attribute_key(id) else {
        return K_RESULT_FALSE;
    };
    if value.is_null() || size_in_bytes.is_null() {
        return K_RESULT_FALSE;
    }
    match (*(this.cast::<HostAttributeList>()))
        .values
        .lock()
        .expect("host attribute list poisoned")
        .get(&key)
    {
        Some(HostAttribute::Binary(stored)) => {
            *value = stored.as_ptr().cast();
            *size_in_bytes = stored.len() as u32;
            K_RESULT_OK
        }
        _ => {
            *value = ptr::null();
            *size_in_bytes = 0;
            K_RESULT_FALSE
        }
    }
}

#[repr(C)]
struct HostMessageVTable {
    query_interface: unsafe extern "C" fn(*mut c_void, *const Tuid, *mut *mut c_void) -> Tresult,
    add_ref: unsafe extern "C" fn(*mut c_void) -> u32,
    release: unsafe extern "C" fn(*mut c_void) -> u32,
    get_message_id: unsafe extern "C" fn(*mut c_void) -> *const c_char,
    set_message_id: unsafe extern "C" fn(*mut c_void, *const c_char),
    get_attributes: unsafe extern "C" fn(*mut c_void) -> *mut c_void,
}

#[repr(C)]
struct HostMessage {
    vtable: *const HostMessageVTable,
    refs: AtomicU32,
    message_id: Mutex<Option<CString>>,
    attributes: *mut c_void,
}

static HOST_MESSAGE_VTABLE: HostMessageVTable = HostMessageVTable {
    query_interface: host_message_query_interface,
    add_ref: host_message_add_ref,
    release: host_message_release,
    get_message_id: host_message_get_id,
    set_message_id: host_message_set_id,
    get_attributes: host_message_get_attributes,
};

fn new_host_message() -> *mut c_void {
    Box::into_raw(Box::new(HostMessage {
        vtable: &HOST_MESSAGE_VTABLE,
        refs: AtomicU32::new(1),
        message_id: Mutex::new(None),
        attributes: new_host_attribute_list(),
    }))
    .cast()
}

unsafe extern "C" fn host_message_query_interface(
    this: *mut c_void,
    iid: *const Tuid,
    out: *mut *mut c_void,
) -> Tresult {
    if out.is_null() {
        return K_NO_INTERFACE;
    }
    if !iid.is_null() && (*iid == FUNKNOWN_IID || *iid == IMESSAGE_IID) {
        *out = this;
        host_message_add_ref(this);
        return K_RESULT_OK;
    }
    *out = ptr::null_mut();
    K_NO_INTERFACE
}

unsafe extern "C" fn host_message_add_ref(this: *mut c_void) -> u32 {
    (*(this.cast::<HostMessage>()))
        .refs
        .fetch_add(1, Ordering::Relaxed)
        + 1
}

unsafe extern "C" fn host_message_release(this: *mut c_void) -> u32 {
    let message = this.cast::<HostMessage>();
    let remaining = (*message).refs.fetch_sub(1, Ordering::Release) - 1;
    if remaining == 0 {
        std::sync::atomic::fence(Ordering::Acquire);
        host_attribute_list_release((*message).attributes);
        drop(Box::from_raw(message));
    }
    remaining
}

unsafe extern "C" fn host_message_get_id(this: *mut c_void) -> *const c_char {
    (*(this.cast::<HostMessage>()))
        .message_id
        .lock()
        .expect("host message ID poisoned")
        .as_ref()
        .map_or(ptr::null(), |id| id.as_ptr())
}

unsafe extern "C" fn host_message_set_id(this: *mut c_void, id: *const c_char) {
    let value = (!id.is_null()).then(|| CStr::from_ptr(id).to_owned());
    *(*(this.cast::<HostMessage>()))
        .message_id
        .lock()
        .expect("host message ID poisoned") = value;
}

unsafe extern "C" fn host_message_get_attributes(this: *mut c_void) -> *mut c_void {
    (*(this.cast::<HostMessage>())).attributes
}

#[repr(C)]
struct HostApplicationVTable {
    query_interface: unsafe extern "C" fn(*mut c_void, *const Tuid, *mut *mut c_void) -> Tresult,
    add_ref: unsafe extern "C" fn(*mut c_void) -> u32,
    release: unsafe extern "C" fn(*mut c_void) -> u32,
    get_name: unsafe extern "C" fn(*mut c_void, *mut i16) -> Tresult,
    create_instance:
        unsafe extern "C" fn(*mut c_void, *mut u8, *mut u8, *mut *mut c_void) -> Tresult,
}

#[repr(C)]
struct StaticHostApplication {
    vtable: *const HostApplicationVTable,
}

// Safety: the static host object is immutable and its methods are
// stateless/thread-safe.
unsafe impl Sync for StaticHostApplication {}

static HOST_APPLICATION_VTABLE: HostApplicationVTable = HostApplicationVTable {
    query_interface: host_query_interface,
    add_ref: host_add_ref,
    release: host_release,
    get_name: host_get_name,
    create_instance: host_create_instance,
};

static HOST_APPLICATION: StaticHostApplication = StaticHostApplication {
    vtable: &HOST_APPLICATION_VTABLE,
};

unsafe extern "C" fn host_query_interface(
    this: *mut c_void,
    iid: *const Tuid,
    out: *mut *mut c_void,
) -> Tresult {
    if out.is_null() {
        return K_NO_INTERFACE;
    }
    if !iid.is_null() && (*iid == FUNKNOWN_IID || *iid == IHOST_APPLICATION_IID) {
        *out = this;
        return K_RESULT_OK;
    }
    *out = ptr::null_mut();
    K_NO_INTERFACE
}

unsafe extern "C" fn host_add_ref(_this: *mut c_void) -> u32 {
    1
}

unsafe extern "C" fn host_release(_this: *mut c_void) -> u32 {
    1
}

unsafe extern "C" fn host_get_name(_this: *mut c_void, name: *mut i16) -> Tresult {
    if name.is_null() {
        return K_NO_INTERFACE;
    }
    let label = "Signal Sandbox Host";
    for (index, unit) in label.encode_utf16().take(127).enumerate() {
        *name.add(index) = unit as i16;
    }
    *name.add(label.encode_utf16().take(127).count()) = 0;
    K_RESULT_OK
}

unsafe extern "C" fn host_create_instance(
    _this: *mut c_void,
    cid: *mut u8,
    iid: *mut u8,
    out: *mut *mut c_void,
) -> Tresult {
    if out.is_null() {
        return K_RESULT_FALSE;
    }
    *out = ptr::null_mut();
    if cid.is_null() || iid.is_null() {
        return K_RESULT_FALSE;
    }
    let cid = &*cid.cast::<Tuid>();
    let iid = &*iid.cast::<Tuid>();
    if *cid == IMESSAGE_IID && *iid == IMESSAGE_IID {
        *out = new_host_message();
        return K_RESULT_OK;
    }
    if *cid == IATTRIBUTE_LIST_IID && *iid == IATTRIBUTE_LIST_IID {
        *out = new_host_attribute_list();
        return K_RESULT_OK;
    }
    K_RESULT_FALSE
}

fn host_context() -> *mut c_void {
    &HOST_APPLICATION as *const StaticHostApplication as *mut c_void
}

/// Supply Signal's standard host context to factories implementing
/// `IPluginFactory3`. Older factories remain valid and require no action.
pub(super) unsafe fn set_factory_host_context(factory: *mut c_void) -> bool {
    configure_factory_host_context(factory, host_context())
}

/// Clear a factory context before retrying a legacy or application-private
/// factory that rejects ordinary VST3 creation after receiving host context.
pub(super) unsafe fn clear_factory_host_context(factory: *mut c_void) -> bool {
    configure_factory_host_context(factory, ptr::null_mut())
}

pub(super) fn should_set_factory_host_context(bundle_root: &Path) -> bool {
    !bundle_root
        .extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| extension.eq_ignore_ascii_case("bundle"))
}

unsafe fn configure_factory_host_context(factory: *mut c_void, context: *mut c_void) -> bool {
    if factory.is_null() {
        return false;
    }
    let vtable = vtable_of::<PluginFactoryVTable>(factory);
    let mut factory_3 = ptr::null_mut();
    if ((*vtable).query_interface)(factory, &IPLUGIN_FACTORY_3_IID, &mut factory_3) != K_RESULT_OK
        || factory_3.is_null()
    {
        return false;
    }
    let factory_3_vtable = vtable_of::<PluginFactory3VTable>(factory_3);
    ((*factory_3_vtable).set_host_context)(factory_3, context);
    com_release(factory_3);
    true
}

#[cfg(test)]
mod host_application_tests {
    use super::*;

    #[test]
    fn skips_factory_context_for_application_private_bundle_components() {
        assert!(!should_set_factory_host_context(Path::new(
            "/Applications/Cubase.app/Contents/Components/Modulation FX.bundle"
        )));
        assert!(should_set_factory_host_context(Path::new(
            "/Library/Audio/Plug-Ins/VST3/Example.vst3"
        )));
    }

    #[test]
    fn creates_messages_with_writable_attributes() {
        unsafe {
            let mut cid = IMESSAGE_IID;
            let mut iid = IMESSAGE_IID;
            let mut message = ptr::null_mut();
            assert_eq!(
                host_create_instance(
                    host_context(),
                    cid.as_mut_ptr(),
                    iid.as_mut_ptr(),
                    &mut message,
                ),
                K_RESULT_OK
            );
            assert!(!message.is_null());

            let message_vtable = vtable_of::<HostMessageVTable>(message);
            let message_id = c"slate-ui-message";
            ((*message_vtable).set_message_id)(message, message_id.as_ptr());
            assert_eq!(
                CStr::from_ptr(((*message_vtable).get_message_id)(message)),
                message_id
            );

            let attributes = ((*message_vtable).get_attributes)(message);
            assert!(!attributes.is_null());
            let attributes_vtable = vtable_of::<HostAttributeListVTable>(attributes);
            let key = c"parameter";
            assert_eq!(
                ((*attributes_vtable).set_int)(attributes, key.as_ptr(), 42),
                K_RESULT_OK
            );
            let mut value = 0;
            assert_eq!(
                ((*attributes_vtable).get_int)(attributes, key.as_ptr(), &mut value),
                K_RESULT_OK
            );
            assert_eq!(value, 42);

            assert_eq!(((*message_vtable).release)(message), 0);
        }
    }

    #[test]
    fn creates_standalone_attribute_lists() {
        unsafe {
            let mut cid = IATTRIBUTE_LIST_IID;
            let mut iid = IATTRIBUTE_LIST_IID;
            let mut attributes = ptr::null_mut();
            assert_eq!(
                host_create_instance(
                    host_context(),
                    cid.as_mut_ptr(),
                    iid.as_mut_ptr(),
                    &mut attributes,
                ),
                K_RESULT_OK
            );
            assert!(!attributes.is_null());
            assert_eq!(host_attribute_list_release(attributes), 0);
        }
    }
}

// ── Module loading ──────────────────────────────────────────────────────────

type EntryProc = unsafe extern "C" fn(*mut c_void) -> bool;
type ExitProc = unsafe extern "C" fn();
type GetPluginFactoryProc = unsafe extern "C" fn() -> *mut c_void;

#[cfg(target_os = "macos")]
mod macos_bundle {
    use super::*;

    type CFAllocatorRef = *const c_void;
    type CFStringRef = *const c_void;
    type CFURLRef = *const c_void;
    type CFBundleRef = *mut c_void;

    #[link(name = "CoreFoundation", kind = "framework")]
    extern "C" {
        static kCFAllocatorDefault: CFAllocatorRef;
        fn CFBundleCreate(allocator: CFAllocatorRef, bundleURL: CFURLRef) -> CFBundleRef;
        fn CFBundleGetFunctionPointerForName(
            bundle: CFBundleRef,
            functionName: CFStringRef,
        ) -> *mut c_void;
        fn CFBundleLoadExecutable(bundle: CFBundleRef) -> u8;
        fn CFRelease(cf: *const c_void);
        fn CFStringCreateWithCString(
            alloc: CFAllocatorRef,
            cStr: *const c_char,
            encoding: u32,
        ) -> CFStringRef;
        fn CFURLCreateWithFileSystemPath(
            allocator: CFAllocatorRef,
            filePath: CFStringRef,
            pathStyle: isize,
            isDirectory: u8,
        ) -> CFURLRef;
    }

    const K_CF_STRING_ENCODING_UTF8: u32 = 0x0800_0100;
    const K_CF_URL_POSIX_PATH_STYLE: isize = 0;

    pub(super) struct MacVst3Bundle {
        bundle: CFBundleRef,
    }

    impl MacVst3Bundle {
        pub(super) fn load(bundle_root: &Path) -> Result<Self, Vst3HostingError> {
            let path = bundle_root
                .to_str()
                .ok_or_else(|| Vst3HostingError::new("module_bundle_path_invalid"))?;
            let path_c = CString::new(path)
                .map_err(|_| Vst3HostingError::new("module_bundle_path_invalid"))?;
            unsafe {
                let path_string = CFStringCreateWithCString(
                    kCFAllocatorDefault,
                    path_c.as_ptr(),
                    K_CF_STRING_ENCODING_UTF8,
                );
                if path_string.is_null() {
                    return Err(Vst3HostingError::new("module_bundle_path_invalid"));
                }
                let bundle_url = CFURLCreateWithFileSystemPath(
                    kCFAllocatorDefault,
                    path_string,
                    K_CF_URL_POSIX_PATH_STYLE,
                    1,
                );
                CFRelease(path_string);
                if bundle_url.is_null() {
                    return Err(Vst3HostingError::new("module_bundle_url_invalid"));
                }
                let bundle = CFBundleCreate(kCFAllocatorDefault, bundle_url);
                CFRelease(bundle_url);
                if bundle.is_null() {
                    return Err(Vst3HostingError::new("module_bundle_open_failed"));
                }
                if CFBundleLoadExecutable(bundle) == 0 {
                    CFRelease(bundle);
                    return Err(Vst3HostingError::new("module_open_failed"));
                }
                Ok(Self { bundle })
            }
        }

        pub(super) fn bundle_ref(&self) -> *mut c_void {
            self.bundle.cast()
        }

        unsafe fn function_ptr(&self, name: &str) -> Option<*mut c_void> {
            let name_c = CString::new(name).ok()?;
            let name_string = CFStringCreateWithCString(
                kCFAllocatorDefault,
                name_c.as_ptr(),
                K_CF_STRING_ENCODING_UTF8,
            );
            if name_string.is_null() {
                return None;
            }
            let pointer = CFBundleGetFunctionPointerForName(self.bundle, name_string);
            CFRelease(name_string);
            (!pointer.is_null()).then_some(pointer)
        }

        pub(super) unsafe fn entry(&self) -> Option<EntryProc> {
            self.function_ptr("bundleEntry")
                .map(|pointer| std::mem::transmute(pointer))
        }

        pub(super) unsafe fn exit(&self) -> Option<ExitProc> {
            self.function_ptr("bundleExit")
                .map(|pointer| std::mem::transmute(pointer))
        }

        pub(super) unsafe fn factory(&self) -> Option<GetPluginFactoryProc> {
            self.function_ptr("GetPluginFactory")
                .map(|pointer| std::mem::transmute(pointer))
        }
    }

    impl Drop for MacVst3Bundle {
        fn drop(&mut self) {
            unsafe {
                // Do not CFBundleUnloadExecutable: Objective-C classes cannot
                // be unregistered safely once a plugin bundle has registered
                // them with the process runtime.
                CFRelease(self.bundle);
            }
        }
    }
}

#[cfg(not(target_os = "macos"))]
fn entry_symbol(platform: Vst3HostPlatform) -> &'static [u8] {
    match platform {
        Vst3HostPlatform::MacOs => b"bundleEntry\0",
        Vst3HostPlatform::Linux => b"ModuleEntry\0",
        Vst3HostPlatform::Windows => b"InitDll\0",
    }
}

#[cfg(not(target_os = "macos"))]
fn exit_symbol(platform: Vst3HostPlatform) -> &'static [u8] {
    match platform {
        Vst3HostPlatform::MacOs => b"bundleExit\0",
        Vst3HostPlatform::Linux => b"ModuleExit\0",
        Vst3HostPlatform::Windows => b"ExitDll\0",
    }
}

/// A dlopen'd VST3 module with its platform entry run and the factory
/// resolved. Runs the exit proc and closes the library on drop.
struct LoadedVst3Module {
    #[cfg(target_os = "macos")]
    bundle: macos_bundle::MacVst3Bundle,
    #[cfg(not(target_os = "macos"))]
    library: Library,
    factory: *mut c_void,
    factory_context_set: bool,
    exit: Option<ExitProc>,
}

impl LoadedVst3Module {
    #[cfg(target_os = "macos")]
    fn load(bundle_root: &Path) -> Result<Self, Vst3HostingError> {
        let bundle = macos_bundle::MacVst3Bundle::load(bundle_root)?;
        unsafe {
            if let Some(entry) = bundle.entry() {
                if !entry(bundle.bundle_ref()) {
                    return Err(Vst3HostingError::new("module_entry_failed"));
                }
            }
            let get_factory = bundle
                .factory()
                .ok_or_else(|| Vst3HostingError::new("get_plugin_factory_missing"))?;
            let factory = get_factory();
            if factory.is_null() {
                return Err(Vst3HostingError::new("plugin_factory_null"));
            }
            let factory_context_set =
                should_set_factory_host_context(bundle_root) && set_factory_host_context(factory);
            let exit = bundle.exit();
            Ok(Self {
                bundle,
                factory,
                factory_context_set,
                exit,
            })
        }
    }

    #[cfg(not(target_os = "macos"))]
    fn load(bundle_root: &Path) -> Result<Self, Vst3HostingError> {
        let platform = current_vst3_platform();
        let module_path = resolve_module_binary_path(bundle_root, platform)
            .map_err(|_| Vst3HostingError::new("module_binary_unresolved"))?;
        let library = unsafe { Library::new(&module_path) }
            .map_err(|_| Vst3HostingError::new("module_open_failed"))?;
        unsafe {
            if let Ok(entry) = library.get::<EntryProc>(entry_symbol(platform)) {
                if !entry(ptr::null_mut()) {
                    return Err(Vst3HostingError::new("module_entry_failed"));
                }
            }
            let get_factory = library
                .get::<GetPluginFactoryProc>(b"GetPluginFactory\0")
                .map_err(|_| Vst3HostingError::new("get_plugin_factory_missing"))?;
            let factory = get_factory();
            if factory.is_null() {
                return Err(Vst3HostingError::new("plugin_factory_null"));
            }
            let factory_context_set =
                should_set_factory_host_context(bundle_root) && set_factory_host_context(factory);
            let exit = library
                .get::<ExitProc>(exit_symbol(platform))
                .ok()
                .map(|symbol| *symbol);
            Ok(Self {
                library,
                factory,
                factory_context_set,
                exit,
            })
        }
    }

    /// Create an instance of `cid` exposing `iid` through the factory.
    unsafe fn create_instance(&self, cid: &Tuid, iid: &Tuid) -> Option<*mut c_void> {
        let vtable = vtable_of::<PluginFactoryVTable>(self.factory);
        let mut out: *mut c_void = ptr::null_mut();
        let mut result =
            ((*vtable).create_instance)(self.factory, cid.as_ptr(), iid.as_ptr(), &mut out);
        if self.factory_context_set && (result != K_RESULT_OK || out.is_null()) {
            clear_factory_host_context(self.factory);
            out = ptr::null_mut();
            result =
                ((*vtable).create_instance)(self.factory, cid.as_ptr(), iid.as_ptr(), &mut out);
        }
        (result == K_RESULT_OK && !out.is_null()).then_some(out)
    }

    /// Return the factory's sole audio-module class, if it has exactly one.
    unsafe fn unique_component_class_id(&self) -> Option<Tuid> {
        let vtable = vtable_of::<PluginFactoryVTable>(self.factory);
        let class_count = ((*vtable).count_classes)(self.factory);
        let mut component = None;
        for index in 0..class_count {
            let mut info = FactoryClassInfo {
                cid: [0; 16],
                cardinality: 0,
                category: [0; 32],
                name: [0; 64],
            };
            if ((*vtable).get_class_info)(
                self.factory,
                index,
                (&mut info as *mut FactoryClassInfo).cast(),
            ) != K_RESULT_OK
            {
                continue;
            }
            let category = CStr::from_ptr(info.category.as_ptr()).to_bytes();
            if category != b"Audio Module Class" {
                continue;
            }
            if component.is_some() {
                return None;
            }
            component = Some(info.cid);
        }
        component
    }
}

impl Drop for LoadedVst3Module {
    fn drop(&mut self) {
        if let Some(exit) = self.exit {
            unsafe { exit() };
        }
        #[cfg(not(target_os = "macos"))]
        // `library` drops after this body, unloading the module last.
        let _ = &self.library;
        #[cfg(target_os = "macos")]
        // `bundle` drops after this body, releasing the CFBundle object last.
        let _ = &self.bundle;
    }
}

// ── Host-side IParameterChanges (g12.023 param-set wire) ───────────────────
//
// The input parameter-change list handed to `IAudioProcessor::process`:
// one single-point queue per changed parameter, every point at sample
// offset 0 (block-boundary application, sample-accuracy posture v1).
// Everything is preallocated at the change-queue capacity and rebuilt in
// place per block — no allocation on the audio thread. Refcounting is a
// no-op: the session owns the objects and outlives every process call.

/// `IParamValueQueue` vtable (FUnknown + queue methods, declaration order).
#[repr(C)]
struct ParamValueQueueVTable {
    query_interface: unsafe extern "C" fn(*mut c_void, *const Tuid, *mut *mut c_void) -> Tresult,
    add_ref: unsafe extern "C" fn(*mut c_void) -> u32,
    release: unsafe extern "C" fn(*mut c_void) -> u32,
    get_parameter_id: unsafe extern "C" fn(*mut c_void) -> u32,
    get_point_count: unsafe extern "C" fn(*mut c_void) -> i32,
    get_point: unsafe extern "C" fn(*mut c_void, i32, *mut i32, *mut f64) -> Tresult,
    add_point: unsafe extern "C" fn(*mut c_void, i32, f64, *mut i32) -> Tresult,
}

/// `IParameterChanges` vtable (FUnknown + list methods, declaration order).
#[repr(C)]
struct ParameterChangesVTable {
    query_interface: unsafe extern "C" fn(*mut c_void, *const Tuid, *mut *mut c_void) -> Tresult,
    add_ref: unsafe extern "C" fn(*mut c_void) -> u32,
    release: unsafe extern "C" fn(*mut c_void) -> u32,
    get_parameter_count: unsafe extern "C" fn(*mut c_void) -> i32,
    get_parameter_data: unsafe extern "C" fn(*mut c_void, i32) -> *mut c_void,
    add_parameter_data: unsafe extern "C" fn(*mut c_void, *const u32, *mut i32) -> *mut c_void,
}

/// Sample-offset points one host param queue can carry per block (wire
/// writes use one point at offset 0; MIDI-mapped CC series use one point
/// per CC event at its intra-block offset).
const PARAM_QUEUE_POINT_CAPACITY: usize = 128;

/// One value queue: `(sample_offset, value)` points for one parameter,
/// ascending offsets. Preallocated; rebuilt in place per block.
#[repr(C)]
struct HostParamValueQueue {
    vtable: *const ParamValueQueueVTable,
    parameter_id: u32,
    points: Box<[(i32, f64)]>,
    point_count: usize,
}

/// The block's input parameter-change list: a fixed-length queue pool plus
/// the active count. Boxed by the session so every pointer handed to the
/// plugin stays stable.
#[repr(C)]
struct HostParameterChanges {
    vtable: *const ParameterChangesVTable,
    queues: Box<[HostParamValueQueue]>,
    active: usize,
}

static PARAM_VALUE_QUEUE_VTABLE: ParamValueQueueVTable = ParamValueQueueVTable {
    query_interface: param_queue_query_interface,
    add_ref: param_com_add_ref,
    release: param_com_release,
    get_parameter_id: param_queue_get_parameter_id,
    get_point_count: param_queue_get_point_count,
    get_point: param_queue_get_point,
    add_point: param_queue_add_point,
};

static PARAMETER_CHANGES_VTABLE: ParameterChangesVTable = ParameterChangesVTable {
    query_interface: param_changes_query_interface,
    add_ref: param_com_add_ref,
    release: param_com_release,
    get_parameter_count: param_changes_get_parameter_count,
    get_parameter_data: param_changes_get_parameter_data,
    add_parameter_data: param_changes_add_parameter_data,
};

unsafe extern "C" fn param_com_add_ref(_this: *mut c_void) -> u32 {
    1
}

unsafe extern "C" fn param_com_release(_this: *mut c_void) -> u32 {
    1
}

unsafe extern "C" fn param_queue_query_interface(
    this: *mut c_void,
    iid: *const Tuid,
    out: *mut *mut c_void,
) -> Tresult {
    if out.is_null() {
        return K_NO_INTERFACE;
    }
    if !iid.is_null() && (*iid == FUNKNOWN_IID || *iid == IPARAM_VALUE_QUEUE_IID) {
        *out = this;
        return K_RESULT_OK;
    }
    *out = ptr::null_mut();
    K_NO_INTERFACE
}

unsafe extern "C" fn param_queue_get_parameter_id(this: *mut c_void) -> u32 {
    (*this.cast::<HostParamValueQueue>()).parameter_id
}

unsafe extern "C" fn param_queue_get_point_count(this: *mut c_void) -> i32 {
    (*this.cast::<HostParamValueQueue>()).point_count as i32
}

unsafe extern "C" fn param_queue_get_point(
    this: *mut c_void,
    index: i32,
    sample_offset: *mut i32,
    value: *mut f64,
) -> Tresult {
    if sample_offset.is_null() || value.is_null() {
        return K_NO_INTERFACE;
    }
    let queue = &*this.cast::<HostParamValueQueue>();
    if index < 0 || index as usize >= queue.point_count {
        return K_NO_INTERFACE;
    }
    let (offset, point_value) = queue.points[index as usize];
    *sample_offset = offset;
    *value = point_value;
    K_RESULT_OK
}

unsafe extern "C" fn param_queue_add_point(
    _this: *mut c_void,
    _sample_offset: i32,
    _value: f64,
    _index: *mut i32,
) -> Tresult {
    // Input list: the host writes it, the plugin only reads.
    K_NO_INTERFACE
}

unsafe extern "C" fn param_changes_query_interface(
    this: *mut c_void,
    iid: *const Tuid,
    out: *mut *mut c_void,
) -> Tresult {
    if out.is_null() {
        return K_NO_INTERFACE;
    }
    if !iid.is_null() && (*iid == FUNKNOWN_IID || *iid == IPARAMETER_CHANGES_IID) {
        *out = this;
        return K_RESULT_OK;
    }
    *out = ptr::null_mut();
    K_NO_INTERFACE
}

unsafe extern "C" fn param_changes_get_parameter_count(this: *mut c_void) -> i32 {
    (*this.cast::<HostParameterChanges>()).active as i32
}

unsafe extern "C" fn param_changes_get_parameter_data(
    this: *mut c_void,
    index: i32,
) -> *mut c_void {
    let changes = &mut *this.cast::<HostParameterChanges>();
    if index < 0 || index as usize >= changes.active {
        return ptr::null_mut();
    }
    (&mut changes.queues[index as usize] as *mut HostParamValueQueue).cast()
}

unsafe extern "C" fn param_changes_add_parameter_data(
    this: *mut c_void,
    id: *const u32,
    index: *mut i32,
) -> *mut c_void {
    if id.is_null() || index.is_null() {
        return ptr::null_mut();
    }
    let changes = &mut *this.cast::<HostParameterChanges>();
    if let Some((queue_index, queue)) = changes.queues[..changes.active]
        .iter_mut()
        .enumerate()
        .find(|(_, queue)| queue.parameter_id == *id)
    {
        *index = queue_index as i32;
        return (queue as *mut HostParamValueQueue).cast();
    }
    if changes.active == changes.queues.len() {
        return ptr::null_mut();
    }
    let queue_index = changes.active;
    changes.active += 1;
    let queue = &mut changes.queues[queue_index];
    queue.parameter_id = *id;
    queue.point_count = 0;
    *index = queue_index as i32;
    (queue as *mut HostParamValueQueue).cast()
}

impl HostParameterChanges {
    fn new() -> Box<Self> {
        let queues = (0..PLUGIN_PARAM_CHANGE_CAPACITY)
            .map(|_| HostParamValueQueue {
                vtable: &PARAM_VALUE_QUEUE_VTABLE,
                parameter_id: 0,
                points: vec![(0i32, 0f64); PARAM_QUEUE_POINT_CAPACITY].into_boxed_slice(),
                point_count: 0,
            })
            .collect::<Vec<_>>()
            .into_boxed_slice();
        Box::new(Self {
            vtable: &PARAMETER_CHANGES_VTABLE,
            queues,
            active: 0,
        })
    }

    /// Reset to an empty list (start of block). Alloc-free.
    fn clear(&mut self) {
        self.active = 0;
    }

    /// Append one `(sample_offset, value)` point for `parameter_id`,
    /// reusing the parameter's queue when one is already active this block.
    /// Alloc-free; silently drops on pool/point capacity overflow.
    fn push_point(&mut self, parameter_id: u32, sample_offset: i32, value: f64) {
        let existing = self.queues[..self.active]
            .iter_mut()
            .find(|queue| queue.parameter_id == parameter_id);
        let queue = match existing {
            Some(queue) => queue,
            None => {
                if self.active == self.queues.len() {
                    return;
                }
                let queue = &mut self.queues[self.active];
                queue.parameter_id = parameter_id;
                queue.point_count = 0;
                self.active += 1;
                queue
            }
        };
        if queue.point_count == queue.points.len() {
            return;
        }
        queue.points[queue.point_count] = (sample_offset, value);
        queue.point_count += 1;
    }

    /// Rebuild the list in place from the drained wire changes (one point
    /// per parameter at offset 0 — block-boundary posture). Alloc-free.
    fn set_changes(&mut self, changes: &[PluginParamChange]) {
        self.clear();
        for change in changes {
            self.push_point(change.parameter_id, 0, change.value);
        }
    }
}

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

const VST3_PITCH_BEND_CONTROLLER: usize = 128;
const VST3_AFTERTOUCH_CONTROLLER: usize = 129;
const VST3_MIDI_CONTROLLER_COUNT: usize = 130;

/// `Steinberg::Vst::NoteOnEvent`.
#[repr(C)]
#[derive(Clone, Copy)]
struct NoteOnEventPayload {
    channel: i16,
    pitch: i16,
    tuning: f32,
    velocity: f32,
    length: i32,
    note_id: i32,
}

/// `Steinberg::Vst::NoteOffEvent`.
#[repr(C)]
#[derive(Clone, Copy)]
struct NoteOffEventPayload {
    channel: i16,
    pitch: i16,
    velocity: f32,
    note_id: i32,
    tuning: f32,
}

/// The `Event` union payload: sized/aligned to the widest published member
/// (pointer-bearing members give the C union 8-byte alignment; 24 bytes
/// covers `NoteExpressionTextEvent`).
#[repr(C)]
#[derive(Clone, Copy)]
union EventPayload {
    note_on: NoteOnEventPayload,
    note_off: NoteOffEventPayload,
    _size: [u64; 3],
}

/// `Steinberg::Vst::Event::EventTypes`.
const K_NOTE_ON_EVENT: u16 = 0;
const K_NOTE_OFF_EVENT: u16 = 1;

/// `Steinberg::Vst::Event`.
#[repr(C)]
#[derive(Clone, Copy)]
struct Vst3Event {
    bus_index: i32,
    sample_offset: i32,
    ppq_position: f64,
    flags: u16,
    type_: u16,
    payload: EventPayload,
}

impl Vst3Event {
    fn zeroed() -> Self {
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
struct EventListVTable {
    query_interface: unsafe extern "C" fn(*mut c_void, *const Tuid, *mut *mut c_void) -> Tresult,
    add_ref: unsafe extern "C" fn(*mut c_void) -> u32,
    release: unsafe extern "C" fn(*mut c_void) -> u32,
    get_event_count: unsafe extern "C" fn(*mut c_void) -> i32,
    get_event: unsafe extern "C" fn(*mut c_void, i32, *mut Vst3Event) -> Tresult,
    add_event: unsafe extern "C" fn(*mut c_void, *mut Vst3Event) -> Tresult,
}

/// Per-block note in-event capacity (matches the render plane's cap).
const EVENT_LIST_CAPACITY: usize = 1024;

/// The block's input event list: a fixed pool plus the active count. Boxed
/// by the session so the pointer handed to the plugin stays stable.
#[repr(C)]
struct HostEventList {
    vtable: *const EventListVTable,
    events: Box<[Vst3Event]>,
    active: usize,
}

static EVENT_LIST_VTABLE: EventListVTable = EventListVTable {
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
    fn new() -> Box<Self> {
        Box::new(Self {
            vtable: &EVENT_LIST_VTABLE,
            events: vec![Vst3Event::zeroed(); EVENT_LIST_CAPACITY].into_boxed_slice(),
            active: 0,
        })
    }

    fn clear(&mut self) {
        self.active = 0;
    }

    /// Append one note event; silently drops on capacity overflow.
    fn push_note(&mut self, note: &signal_plugin::NoteEvent) {
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
struct MidiMappingVTable {
    query_interface: unsafe extern "C" fn(*mut c_void, *const Tuid, *mut *mut c_void) -> Tresult,
    add_ref: unsafe extern "C" fn(*mut c_void) -> u32,
    release: unsafe extern "C" fn(*mut c_void) -> u32,
    get_midi_controller_assignment:
        unsafe extern "C" fn(*mut c_void, i32, i16, i16, *mut u32) -> Tresult,
}

/// Query the controller's `IMidiMapping` for bus 0 / channel 0 CC → param
/// assignments (controllers 0..=127). `None` when the controller exposes no
/// mapping — the honest no-CC fallback. Runs at load on the lifecycle
/// thread; the resulting table is immutable and shared into sessions.
unsafe fn midi_cc_parameter_map(
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

// ── Hosted instance ─────────────────────────────────────────────────────────

/// Main-bus stereo port layout summary for a hosted VST3 instance (mirrors
/// `ClapHostedPortLayout`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Vst3HostedPortLayout {
    /// Channel count of the main audio input bus (0 = none).
    pub main_input_channels: u16,
    /// Channel count of the main audio output bus (0 = none).
    pub main_output_channels: u16,
}

#[derive(Clone, Debug)]
struct Vst3AudioBusLayout {
    input_channels: Vec<u16>,
    output_channels: Vec<u16>,
    main_input: Option<usize>,
    main_output: Option<usize>,
}

impl Vst3AudioBusLayout {
    fn port_layout(&self) -> Vst3HostedPortLayout {
        Vst3HostedPortLayout {
            main_input_channels: self
                .main_input
                .map(|index| self.input_channels[index])
                .unwrap_or(0),
            main_output_channels: self
                .main_output
                .map(|index| self.output_channels[index])
                .unwrap_or(0),
        }
    }
}

impl Vst3HostedPortLayout {
    /// Phase 1 supports exactly a stereo main in + stereo main out effect.
    pub fn is_stereo_effect(&self) -> bool {
        self.main_input_channels == 2 && self.main_output_channels == 2
    }

    /// MIDI instrument layout supported by the current host: no main audio
    /// input and one stereo main output.
    pub fn is_stereo_instrument(&self) -> bool {
        self.main_input_channels == 0 && self.main_output_channels == 2
    }

    /// Whether the current stereo process session can host this layout.
    pub fn is_supported_stereo_processor(&self) -> bool {
        self.is_stereo_effect() || self.is_stereo_instrument()
    }
}

/// Lifecycle state of a hosted instance.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum HostedInstanceState {
    Created,
    Active,
}

/// How the edit controller was obtained (drop/teardown differs).
enum ControllerHandle {
    /// `queryInterface` facet of the component object itself: release only.
    ComponentFacet(*mut c_void),
    /// Separate class created through the factory: terminate then release.
    Separate(*mut c_void),
}

impl ControllerHandle {
    fn ptr(&self) -> *mut c_void {
        match self {
            Self::ComponentFacet(ptr) | Self::Separate(ptr) => *ptr,
        }
    }
}

/// Connected `IConnectionPoint` facets for a separate component/controller
/// pair. The connection is established in both directions and must be torn
/// down before either plugin object is terminated.
struct ControllerConnection {
    component: *mut c_void,
    controller: *mut c_void,
}

#[repr(C)]
struct ConnectionPointVTable {
    query_interface: unsafe extern "C" fn(*mut c_void, *const Tuid, *mut *mut c_void) -> Tresult,
    add_ref: unsafe extern "C" fn(*mut c_void) -> u32,
    release: unsafe extern "C" fn(*mut c_void) -> u32,
    connect: unsafe extern "C" fn(*mut c_void, *mut c_void) -> Tresult,
    disconnect: unsafe extern "C" fn(*mut c_void, *mut c_void) -> Tresult,
    notify: unsafe extern "C" fn(*mut c_void, *mut c_void) -> Tresult,
}

impl ControllerConnection {
    unsafe fn establish(component: *mut c_void, controller: *mut c_void) -> Option<Self> {
        let component_point = com_query_interface(component, &ICONNECTION_POINT_IID)?;
        let Some(controller_point) = com_query_interface(controller, &ICONNECTION_POINT_IID) else {
            com_release(component_point);
            return None;
        };
        let component_vtable = vtable_of::<ConnectionPointVTable>(component_point);
        if ((*component_vtable).connect)(component_point, controller_point) != K_RESULT_OK {
            com_release(controller_point);
            com_release(component_point);
            return None;
        }
        let controller_vtable = vtable_of::<ConnectionPointVTable>(controller_point);
        if ((*controller_vtable).connect)(controller_point, component_point) != K_RESULT_OK {
            let _ = ((*component_vtable).disconnect)(component_point, controller_point);
            com_release(controller_point);
            com_release(component_point);
            return None;
        }
        Some(Self {
            component: component_point,
            controller: controller_point,
        })
    }
}

impl Drop for ControllerConnection {
    fn drop(&mut self) {
        unsafe {
            let component_vtable = vtable_of::<ConnectionPointVTable>(self.component);
            let controller_vtable = vtable_of::<ConnectionPointVTable>(self.controller);
            let _ = ((*controller_vtable).disconnect)(self.controller, self.component);
            let _ = ((*component_vtable).disconnect)(self.component, self.controller);
            com_release(self.controller);
            com_release(self.component);
        }
    }
}

/// One live VST3 plugin instance hosted in this process: owns the loaded
/// module, the `IComponent`/`IAudioProcessor` pair, and the (optional)
/// `IEditController`.
///
/// Threading: create/activate/deactivate/destroy run on the owning (main)
/// thread; audio processing runs through [`Vst3ProcessSession`], which the
/// sandbox moves onto its audio thread. While a process session is live the
/// owner must not run lifecycle transitions until the session stops.
pub struct Vst3HostedInstance {
    component: *mut c_void,
    processor: *mut c_void,
    controller: Option<ControllerHandle>,
    /// Bidirectional component/controller messaging for controllers exposed
    /// as a separate VST3 class. Dropped before either endpoint terminates.
    controller_connection: Option<ControllerConnection>,
    /// Stable host callback object installed on the edit controller.
    component_handler: Option<Box<ComponentHandler>>,
    /// Restart requests accepted by the component handler and serviced by
    /// the owning host control thread.
    pending_restart_flags: Arc<AtomicU32>,
    parameters: Vec<PluginParameterDescriptor>,
    port_layout: Vst3HostedPortLayout,
    audio_bus_layout: Vst3AudioBusLayout,
    state: HostedInstanceState,
    activated_sample_rate_hz: f64,
    activated_max_frames: u32,
    /// The live editor view, when open. Torn down (removed + released)
    /// BEFORE the controller in `Drop` — the mandated release ordering.
    gui_session: Option<Vst3GuiSession>,
    /// Pending param writes bound for the audio thread's
    /// `IParameterChanges` (g12.023); shared with every process session
    /// built from this instance.
    param_changes: Arc<PluginParamChangeQueue>,
    /// Bus 0 / channel 0 CC → parameter assignments from the controller's
    /// `IMidiMapping`, queried once at load. `None` = no mapping exposed
    /// (CC events are dropped for this plugin — the honest fallback).
    midi_cc_params: Option<Arc<[Option<u32>; VST3_MIDI_CONTROLLER_COUNT]>>,
    /// Empty ARA document used only by the isolated inspector. Ordinary
    /// processing loads leave this unset.
    ara_inspection: Option<AraInspectionSession>,
    /// Keeps the module mapped for the instance lifetime; declared last so
    /// it drops after the COM pointers above are released in `drop`.
    _module: LoadedVst3Module,
}

impl Vst3HostedInstance {
    /// Load the module inside `bundle_root`, create the component class
    /// identified by `class_id_hex` (the catalog load key), initialize it
    /// against the minimal host context, and enumerate its parameter
    /// inventory and main-bus port layout.
    ///
    /// The edit controller is acquired by `queryInterface` on the component
    /// (single-component plugins) or, failing that, by
    /// `IComponent::getControllerClassId` + a second factory
    /// `createInstance`. If neither works the inventory degrades to empty.
    pub fn load(bundle_root: &Path, class_id_hex: &str) -> Result<Self, Vst3HostingError> {
        Self::load_internal(bundle_root, class_id_hex, false)
    }

    /// Load a component for isolated UI inspection, binding an empty ARA
    /// document before activation when the component exposes ARA entry
    /// points. This is not full ARA host support.
    pub fn load_for_inspection(
        bundle_root: &Path,
        class_id_hex: &str,
    ) -> Result<Self, Vst3HostingError> {
        Self::load_internal(bundle_root, class_id_hex, true)
    }

    fn load_internal(
        bundle_root: &Path,
        class_id_hex: &str,
        enable_ara_inspection: bool,
    ) -> Result<Self, Vst3HostingError> {
        let cid = tuid_from_class_id_hex(class_id_hex)
            .ok_or_else(|| Vst3HostingError::new("class_id_invalid"))?;
        let module = LoadedVst3Module::load(bundle_root)?;

        let component = unsafe { module.create_instance(&cid, &ICOMPONENT_IID) }
            .or_else(|| {
                if !super::introspection::moduleinfo_declares_component_class(
                    bundle_root,
                    class_id_hex,
                ) {
                    return None;
                }
                let factory_cid = unsafe { module.unique_component_class_id() }?;
                (factory_cid != cid)
                    .then(|| unsafe { module.create_instance(&factory_cid, &ICOMPONENT_IID) })
                    .flatten()
            })
            .ok_or_else(|| Vst3HostingError::new("create_component_failed"))?;
        let host = host_context();
        unsafe {
            let vtable = vtable_of::<ComponentVTable>(component);
            if ((*vtable).initialize)(component, host) != K_RESULT_OK {
                com_release(component);
                return Err(Vst3HostingError::new("component_initialize_failed"));
            }
        }

        // IAudioProcessor: usually the same object, sometimes separate.
        let Some(processor) = (unsafe { com_query_interface(component, &IAUDIO_PROCESSOR_IID) })
        else {
            unsafe {
                let vtable = vtable_of::<ComponentVTable>(component);
                ((*vtable).terminate)(component);
                com_release(component);
            }
            return Err(Vst3HostingError::new("audio_processor_missing"));
        };

        let ara_inspection = if enable_ara_inspection {
            match unsafe { AraInspectionSession::try_bind(component) } {
                Ok(session) => session,
                Err(error) => {
                    unsafe {
                        com_release(processor);
                        let vtable = vtable_of::<ComponentVTable>(component);
                        ((*vtable).terminate)(component);
                        com_release(component);
                    }
                    return Err(error);
                }
            }
        } else {
            None
        };

        let controller = unsafe { acquire_controller(component, &module, host) };
        let pending_restart_flags = Arc::new(AtomicU32::new(0));
        let component_handler = controller.as_ref().and_then(|controller| unsafe {
            let mut handler = Box::new(ComponentHandler {
                vtable: &COMPONENT_HANDLER_VTABLE,
                latency_changes: AtomicU64::new(0),
                pending_restart_flags: Arc::clone(&pending_restart_flags),
            });
            let vtable = vtable_of::<EditControllerVTable>(controller.ptr());
            let ptr = (&mut *handler as *mut ComponentHandler).cast();
            (((*vtable).set_component_handler)(controller.ptr(), ptr) == K_RESULT_OK)
                .then_some(handler)
        });
        let controller_connection = match controller.as_ref() {
            Some(ControllerHandle::Separate(controller)) => unsafe {
                ControllerConnection::establish(component, *controller)
            },
            _ => None,
        };
        if let Some(controller) = &controller {
            unsafe { synchronize_controller_from_component(component, controller.ptr()) };
        }
        let parameters = controller
            .as_ref()
            .map(|handle| unsafe { parameter_inventory(handle.ptr()) })
            .unwrap_or_default();
        let audio_bus_layout = unsafe { audio_bus_layout(component) };
        let port_layout = audio_bus_layout.port_layout();
        let midi_cc_params = controller
            .as_ref()
            .and_then(|handle| unsafe { midi_cc_parameter_map(handle.ptr()) });

        Ok(Self {
            component,
            processor,
            controller,
            controller_connection,
            component_handler,
            pending_restart_flags,
            parameters,
            port_layout,
            audio_bus_layout,
            state: HostedInstanceState::Created,
            activated_sample_rate_hz: 0.0,
            activated_max_frames: 0,
            gui_session: None,
            param_changes: Arc::new(PluginParamChangeQueue::new()),
            midi_cc_params,
            ara_inspection,
            _module: module,
        })
    }

    /// Whether the controller exposed an `IMidiMapping` at load: with one,
    /// CC events deliver as mapped parameter changes; without one they are
    /// dropped (VST3 has no input CC event type).
    pub fn midi_cc_mapping_available(&self) -> bool {
        self.midi_cc_params.is_some()
    }

    /// Whether `IMidiMapping` assigns this ordinary or extended controller
    /// number to a processor parameter.
    pub fn midi_controller_mapping_available(&self, controller: u16) -> bool {
        self.midi_cc_params
            .as_ref()
            .and_then(|map| map.get(usize::from(controller)))
            .copied()
            .flatten()
            .is_some()
    }

    /// Parameter inventory enumerated at load via `IEditController`
    /// (empty when no controller could be acquired).
    pub fn parameters(&self) -> &[PluginParameterDescriptor] {
        &self.parameters
    }

    /// Capture component and optional controller state into a small
    /// host-owned envelope. The payload remains opaque to Signal.
    pub fn save_state(&self) -> Result<Vec<u8>, Vst3HostingError> {
        unsafe {
            let component_vtable = vtable_of::<ComponentVTable>(self.component);
            let mut component = MemoryStream::writer();
            if ((*component_vtable).get_state)(self.component, component.as_raw()) != K_RESULT_OK {
                return Err(Vst3HostingError::new("state_capture_failed"));
            }

            let mut controller_bytes = Vec::new();
            if let Some(controller) = &self.controller {
                let controller_vtable = vtable_of::<EditControllerVTable>(controller.ptr());
                let mut controller_stream = MemoryStream::writer();
                if ((*controller_vtable).get_state)(controller.ptr(), controller_stream.as_raw())
                    == K_RESULT_OK
                {
                    controller_bytes = controller_stream.bytes;
                }
            }
            Ok(encode_state_envelope(&component.bytes, &controller_bytes))
        }
    }

    /// Restore component and optional controller state captured by
    /// [`Self::save_state`].
    pub fn load_state(&mut self, bytes: &[u8]) -> Result<(), Vst3HostingError> {
        let (component_bytes, controller_bytes) = decode_state_envelope(bytes)
            .ok_or_else(|| Vst3HostingError::new("state_deserialize_failed"))?;
        unsafe {
            let component_vtable = vtable_of::<ComponentVTable>(self.component);
            let mut component_stream = MemoryStream::reader(component_bytes);
            if ((*component_vtable).set_state)(self.component, component_stream.as_raw())
                != K_RESULT_OK
            {
                return Err(Vst3HostingError::new("state_restore_failed"));
            }

            if let Some(controller) = &self.controller {
                let controller_vtable = vtable_of::<EditControllerVTable>(controller.ptr());
                let mut component_for_controller = MemoryStream::reader(component_bytes);
                let _ = ((*controller_vtable).set_component_state)(
                    controller.ptr(),
                    component_for_controller.as_raw(),
                );
                if !controller_bytes.is_empty() {
                    let mut controller_stream = MemoryStream::reader(controller_bytes);
                    if ((*controller_vtable).set_state)(
                        controller.ptr(),
                        controller_stream.as_raw(),
                    ) != K_RESULT_OK
                    {
                        return Err(Vst3HostingError::new("controller_state_restore_failed"));
                    }
                }
            }
        }
        Ok(())
    }

    /// Queue one parameter write (g12.023). VST3's set domain is the
    /// normalized 0..1 value itself: it lands in the processor through the
    /// next block's `IParameterChanges` (block-boundary posture v1), and
    /// `IEditController::setParamNormalized` runs here so the controller's
    /// state (GUIs, `getParamNormalized`) stays in sync — the documented
    /// host duty for host-driven changes.
    pub fn set_parameter_normalized(
        &self,
        parameter_id: u32,
        normalized: f32,
    ) -> Result<(), Vst3HostingError> {
        if !self
            .parameters
            .iter()
            .any(|parameter| parameter.parameter_id == parameter_id)
        {
            return Err(Vst3HostingError::new("unknown_parameter"));
        }
        let normalized = f64::from(normalized.clamp(0.0, 1.0));
        if let Some(controller) = &self.controller {
            unsafe {
                let vtable = vtable_of::<EditControllerVTable>(controller.ptr());
                let _ =
                    ((*vtable).set_param_normalized)(controller.ptr(), parameter_id, normalized);
            }
        }
        if !self.param_changes.push(parameter_id, normalized) {
            return Err(Vst3HostingError::new("param_queue_full"));
        }
        Ok(())
    }

    /// Current main-bus port layout, including successful activation-time
    /// negotiation.
    pub fn port_layout(&self) -> Vst3HostedPortLayout {
        self.port_layout
    }

    /// Current processor-reported latency in sample frames.
    pub fn latency_frames(&self) -> u32 {
        unsafe {
            let vtable = vtable_of::<AudioProcessorVTable>(self.processor);
            ((*vtable).get_latency_samples)(self.processor)
        }
    }

    /// Number of controller `kLatencyChanged` restart notifications.
    pub fn latency_change_count(&self) -> u64 {
        self.component_handler
            .as_ref()
            .map(|handler| handler.latency_changes.load(Ordering::Relaxed))
            .unwrap_or(0)
    }

    /// Shared restart flags accepted from `IComponentHandler`. Audio hosts
    /// use this to stop at a block boundary before the control thread
    /// services the requested lifecycle transition.
    pub fn pending_restart_flags(&self) -> Arc<AtomicU32> {
        Arc::clone(&self.pending_restart_flags)
    }

    /// Deactivate, refresh dynamic I/O when requested, reactivate, and build
    /// a replacement process session on the owning control thread.
    pub fn restart_processing(
        &mut self,
        flags: u32,
    ) -> Result<Vst3ProcessSession, Vst3HostingError> {
        let sample_rate_hz = self.activated_sample_rate_hz;
        let max_frames = self.activated_max_frames;
        self.deactivate()?;
        if flags & VST3_RESTART_IO_CHANGED != 0 {
            self.audio_bus_layout = unsafe { audio_bus_layout(self.component) };
            self.port_layout = self.audio_bus_layout.port_layout();
        }
        self.activate(sample_rate_hz, 1, max_frames)?;
        self.process_session()
    }

    /// Activate for processing by negotiating the available main buses to a
    /// stereo effect (2-in/2-out) or instrument (0-in/2-out), then selecting
    /// 32-bit samples, calling `setupProcessing`, activating the main buses,
    /// and calling `setActive(true)`. Unsupported negotiation fails with the
    /// stable `layout_unsupported` token, same as the CLAP path. Components
    /// without any audio output fail with `no_audio_buses`; their editors may
    /// still be hosted without creating a process session.
    pub fn activate(
        &mut self,
        sample_rate_hz: f64,
        _min_frames: u32,
        max_frames: u32,
    ) -> Result<(), Vst3HostingError> {
        if self.state == HostedInstanceState::Active {
            return Err(Vst3HostingError::new("already_active"));
        }
        if self.audio_bus_layout.main_output.is_none() {
            return Err(Vst3HostingError::new("no_audio_buses"));
        }
        unsafe {
            let processor = vtable_of::<AudioProcessorVTable>(self.processor);
            let has_audio_input = self.audio_bus_layout.main_input.is_some();

            // VST3 requires the arrangement array to cover every declared bus,
            // including inactive auxiliaries. Preserve each auxiliary layout
            // and negotiate only the main bus to stereo.
            let mut input_arrangements = bus_arrangements(
                self.processor,
                K_INPUT,
                &self.audio_bus_layout.input_channels,
            );
            let mut output_arrangements = bus_arrangements(
                self.processor,
                K_OUTPUT,
                &self.audio_bus_layout.output_channels,
            );
            if let Some(index) = self.audio_bus_layout.main_input {
                input_arrangements[index] = STEREO_ARRANGEMENT;
            }
            if let Some(index) = self.audio_bus_layout.main_output {
                output_arrangements[index] = STEREO_ARRANGEMENT;
            }
            let _ = ((*processor).set_bus_arrangements)(
                self.processor,
                pointer_or_null(&mut input_arrangements),
                input_arrangements.len() as i32,
                pointer_or_null(&mut output_arrangements),
                output_arrangements.len() as i32,
            );
            let mut verified_input = 0u64;
            let mut verified_output = 0u64;
            let input_verified = !has_audio_input
                || (((*processor).get_bus_arrangement)(
                    self.processor,
                    K_INPUT,
                    self.audio_bus_layout.main_input.unwrap_or(0) as i32,
                    &mut verified_input,
                ) == K_RESULT_OK
                    && verified_input == STEREO_ARRANGEMENT);
            let output_result = ((*processor).get_bus_arrangement)(
                self.processor,
                K_OUTPUT,
                self.audio_bus_layout.main_output.unwrap_or(0) as i32,
                &mut verified_output,
            );
            if !input_verified
                || output_result != K_RESULT_OK
                || verified_output != STEREO_ARRANGEMENT
            {
                return Err(Vst3HostingError::new("layout_unsupported"));
            }
            if let Some(index) = self.audio_bus_layout.main_input {
                self.audio_bus_layout.input_channels[index] = 2;
            }
            if let Some(index) = self.audio_bus_layout.main_output {
                self.audio_bus_layout.output_channels[index] = 2;
            }
            self.port_layout = self.audio_bus_layout.port_layout();

            if ((*processor).can_process_sample_size)(self.processor, K_SAMPLE32) != K_RESULT_OK {
                return Err(Vst3HostingError::new("sample_size_unsupported"));
            }

            let mut setup = ProcessSetup {
                process_mode: K_REALTIME,
                symbolic_sample_size: K_SAMPLE32,
                max_samples_per_block: max_frames as i32,
                sample_rate: sample_rate_hz,
            };
            if ((*processor).setup_processing)(self.processor, &mut setup) != K_RESULT_OK {
                return Err(Vst3HostingError::new("setup_processing_failed"));
            }

            let component = vtable_of::<ComponentVTable>(self.component);
            if let Some(index) = self.audio_bus_layout.main_input {
                let _ =
                    ((*component).activate_bus)(self.component, K_AUDIO, K_INPUT, index as i32, 1);
            }
            if let Some(index) = self.audio_bus_layout.main_output {
                let _ =
                    ((*component).activate_bus)(self.component, K_AUDIO, K_OUTPUT, index as i32, 1);
            }
            if ((*component).set_active)(self.component, 1) != K_RESULT_OK {
                return Err(Vst3HostingError::new("set_active_failed"));
            }
        }
        self.state = HostedInstanceState::Active;
        self.activated_sample_rate_hz = sample_rate_hz;
        self.activated_max_frames = max_frames;
        Ok(())
    }

    /// Deactivate an active instance (no-op tokened error when inactive).
    pub fn deactivate(&mut self) -> Result<(), Vst3HostingError> {
        if self.state != HostedInstanceState::Active {
            return Err(Vst3HostingError::new("not_active"));
        }
        unsafe {
            let component = vtable_of::<ComponentVTable>(self.component);
            let _ = ((*component).set_active)(self.component, 0);
        }
        self.state = HostedInstanceState::Created;
        Ok(())
    }

    // ── IPlugView hosting (g12.024, GUI phase 2) ───────────────────────
    //
    // MAIN-THREAD CONTRACT: every gui_* method below maps to a VST3
    // UI-thread function. The embedding host must dispatch these onto the
    // application main thread (Tauri `run_on_main_thread`); this type only
    // serializes access, it cannot pick the thread.

    /// Whether the plugin exposes an edit controller that may provide an
    /// editor. `createView("editor")` is deliberately deferred until the
    /// real GUI open: some plugins do not tolerate probe-and-discard or
    /// require their processor to be active first.
    pub fn gui_supported(&self) -> bool {
        self.controller.is_some()
    }

    /// Whether an editor view is currently attached.
    pub fn gui_is_open(&self) -> bool {
        self.gui_session.is_some()
    }

    /// Open the embedded editor parented into `parent` (an `NSView*` on
    /// macOS): `createView("editor")` → platform check → `setFrame` →
    /// `getSize` → `attached`. Returns the plugin's initial content size
    /// (logical units). Errors with stable tokens (`gui_unsupported`,
    /// `gui_already_open`, `gui_attached_failed`, …).
    pub fn gui_open_embedded(
        &mut self,
        parent: *mut c_void,
        _scale: Option<f64>,
    ) -> Result<(u32, u32), Vst3HostingError> {
        if self.gui_session.is_some() {
            return Err(Vst3HostingError::new("gui_already_open"));
        }
        let controller = self
            .controller
            .as_ref()
            .ok_or_else(|| Vst3HostingError::new("gui_unsupported"))?;
        let view = unsafe { controller_create_view(controller.ptr()) };
        let session = unsafe { Vst3GuiSession::open_embedded(view, parent) }?;
        let size = session.size();
        self.gui_session = Some(session);
        Ok(size)
    }

    /// The open editor view, for size/resize interaction.
    pub fn gui_session_mut(&mut self) -> Option<&mut Vst3GuiSession> {
        self.gui_session.as_mut()
    }

    /// The open editor view, read-only.
    pub fn gui_session(&self) -> Option<&Vst3GuiSession> {
        self.gui_session.as_ref()
    }

    /// Destroy the open editor view (idempotent; `removed` + release — the
    /// plugin instance stays live).
    pub fn gui_destroy(&mut self) {
        self.gui_session = None;
    }

    /// Drain host-side view callbacks queued since the last call
    /// (`resizeView` requests). Empty when no editor is open.
    pub fn take_gui_events(&self) -> Vec<Vst3GuiEvent> {
        self.gui_session
            .as_ref()
            .map(|session| session.take_events())
            .unwrap_or_default()
    }

    /// Build the raw process session for the sandbox audio thread. Only
    /// valid while active; the session preallocates its planar buffers at
    /// the activated max block size, so processing never allocates.
    pub fn process_session(&self) -> Result<Vst3ProcessSession, Vst3HostingError> {
        if self.state != HostedInstanceState::Active {
            return Err(Vst3HostingError::new("not_active"));
        }
        Ok(Vst3ProcessSession::new(
            self.processor,
            self.activated_sample_rate_hz,
            self.activated_max_frames as usize,
            self.audio_bus_layout.clone(),
            Arc::clone(&self.param_changes),
            self.midi_cc_params.clone(),
        ))
    }
}

impl Drop for Vst3HostedInstance {
    fn drop(&mut self) {
        // View teardown (removed + release) must precede controller
        // teardown. This is the fallback path (teardown with an editor
        // still open); the orderly path closes the editor on the main
        // thread first.
        self.gui_session = None;
        self.controller_connection = None;
        unsafe {
            if self.state == HostedInstanceState::Active {
                let component = vtable_of::<ComponentVTable>(self.component);
                let _ = ((*component).set_active)(self.component, 0);
            }
            com_release(self.processor);
            if let Some(controller) = self.controller.take() {
                let vtable = vtable_of::<EditControllerVTable>(controller.ptr());
                let _ = ((*vtable).set_component_handler)(controller.ptr(), ptr::null_mut());
                match controller {
                    ControllerHandle::ComponentFacet(ptr) => com_release(ptr),
                    ControllerHandle::Separate(ptr) => {
                        let vtable = vtable_of::<EditControllerVTable>(ptr);
                        let _ = ((*vtable).terminate)(ptr);
                        com_release(ptr);
                    }
                }
            }
            let component = vtable_of::<ComponentVTable>(self.component);
            let _ = ((*component).terminate)(self.component);
            com_release(self.component);
        }
        // Drop the ARA document after the bound component, but before the
        // module unloads. The ARA contract permits either component/document
        // destruction order.
        self.ara_inspection = None;
        // `_module` drops after this body: exit proc, then dlclose.
    }
}

/// Give the edit controller the component's initial state before querying
/// parameters or creating its editor. Separate-controller plugins commonly
/// build their UI from this state during `setComponentState`.
unsafe fn synchronize_controller_from_component(component: *mut c_void, controller: *mut c_void) {
    let component_vtable = vtable_of::<ComponentVTable>(component);
    let mut state = MemoryStream::writer();
    if ((*component_vtable).get_state)(component, state.as_raw()) != K_RESULT_OK {
        return;
    }
    state.position = 0;
    let controller_vtable = vtable_of::<EditControllerVTable>(controller);
    let _ = ((*controller_vtable).set_component_state)(controller, state.as_raw());
}

/// Acquire the edit controller: component facet first, else the separate
/// controller class through the factory. `None` = no parameter inventory.
unsafe fn acquire_controller(
    component: *mut c_void,
    module: &LoadedVst3Module,
    host: *mut c_void,
) -> Option<ControllerHandle> {
    if let Some(facet) = com_query_interface(component, &IEDIT_CONTROLLER_IID) {
        return Some(ControllerHandle::ComponentFacet(facet));
    }
    let component_vtable = vtable_of::<ComponentVTable>(component);
    let mut controller_cid: Tuid = [0; 16];
    if ((*component_vtable).get_controller_class_id)(component, &mut controller_cid) != K_RESULT_OK
        || controller_cid == [0; 16]
    {
        return None;
    }
    let controller = module.create_instance(&controller_cid, &IEDIT_CONTROLLER_IID)?;
    let controller_vtable = vtable_of::<EditControllerVTable>(controller);
    if ((*controller_vtable).initialize)(controller, host) != K_RESULT_OK {
        com_release(controller);
        return None;
    }
    Some(ControllerHandle::Separate(controller))
}

/// `IEditController::createView(ViewType::kEditor)`: the plugin's editor
/// view, owned by the caller (null when the plugin has no editor).
pub(crate) unsafe fn controller_create_view(controller: *mut c_void) -> *mut c_void {
    let vtable = vtable_of::<EditControllerVTable>(controller);
    let view = ((*vtable).create_view)(controller, VIEW_TYPE_EDITOR.as_ptr());
    // Some plugins return a view that fails the IPlugView identity check;
    // trust queryInterface over the raw pointer.
    if view.is_null() {
        return ptr::null_mut();
    }
    match com_query_interface(view, &IPLUG_VIEW_IID) {
        Some(typed) => {
            // createView's reference plus queryInterface's addRef: drop one.
            com_release(view);
            typed
        }
        None => {
            com_release(view);
            ptr::null_mut()
        }
    }
}

/// Enumerate the controller's parameter inventory into Signal descriptors.
unsafe fn parameter_inventory(controller: *mut c_void) -> Vec<PluginParameterDescriptor> {
    let vtable = vtable_of::<EditControllerVTable>(controller);
    let count = ((*vtable).get_parameter_count)(controller).max(0);
    let mut parameters = Vec::with_capacity(count as usize);
    for index in 0..count {
        let mut info = ParameterInfo::zeroed();
        if ((*vtable).get_parameter_info)(controller, index, &mut info) != K_RESULT_OK {
            continue;
        }
        let min_plain = ((*vtable).normalized_param_to_plain)(controller, info.id, 0.0) as f32;
        let max_plain = ((*vtable).normalized_param_to_plain)(controller, info.id, 1.0) as f32;
        let is_bypass = info.flags & PARAM_IS_BYPASS != 0;
        let unit = utf16_field_to_string(&info.units);
        parameters.push(PluginParameterDescriptor {
            parameter_id: info.id,
            name: utf16_field_to_string(&info.title).unwrap_or_else(|| format!("Param {index}")),
            unit,
            domain: if is_bypass {
                PluginParameterDomain::Bypass
            } else {
                PluginParameterDomain::GenericNormalized
            },
            default_normalized: info.default_normalized_value as f32,
            min_plain: min_plain.min(max_plain),
            max_plain: max_plain.max(min_plain),
            // VST3 reports the step count directly: 0 = continuous,
            // n = n discrete steps (n + 1 values, 1 = toggle).
            step_count: (info.step_count > 0).then_some(info.step_count as u32),
            flags: PluginParameterFlags {
                automatable: info.flags & PARAM_CAN_AUTOMATE != 0,
                modulatable: false,
                supports_gesture: false,
                stepped: info.step_count > 0,
                hidden: info.flags & PARAM_IS_HIDDEN != 0,
                read_only: info.flags & PARAM_IS_READ_ONLY != 0,
            },
        });
    }
    parameters
}

/// Read every declared audio bus while identifying the main bus in each
/// direction. ProcessData must retain this complete topology even when only
/// the main buses are active.
unsafe fn audio_bus_layout(component: *mut c_void) -> Vst3AudioBusLayout {
    let vtable = vtable_of::<ComponentVTable>(component);
    let mut layout = Vst3AudioBusLayout {
        input_channels: Vec::new(),
        output_channels: Vec::new(),
        main_input: None,
        main_output: None,
    };
    for (direction, channels, main) in [
        (K_INPUT, &mut layout.input_channels, &mut layout.main_input),
        (
            K_OUTPUT,
            &mut layout.output_channels,
            &mut layout.main_output,
        ),
    ] {
        let count = ((*vtable).get_bus_count)(component, K_AUDIO, direction).max(0);
        for index in 0..count {
            let mut info = BusInfo::zeroed();
            if ((*vtable).get_bus_info)(component, K_AUDIO, direction, index, &mut info)
                != K_RESULT_OK
            {
                channels.push(0);
                continue;
            }
            channels.push(info.channel_count.clamp(0, u16::MAX as i32) as u16);
            *main = select_main_bus(*main, info.bus_type, index as usize);
        }
        if main.is_none() && !channels.is_empty() {
            *main = Some(0);
        }
    }
    layout
}

fn select_main_bus(current: Option<usize>, bus_type: i32, index: usize) -> Option<usize> {
    current.or_else(|| (bus_type == K_MAIN).then_some(index))
}

unsafe fn bus_arrangements(
    processor: *mut c_void,
    direction: i32,
    channel_counts: &[u16],
) -> Vec<u64> {
    let vtable = vtable_of::<AudioProcessorVTable>(processor);
    channel_counts
        .iter()
        .enumerate()
        .map(|(index, channels)| {
            let mut arrangement = 0;
            if ((*vtable).get_bus_arrangement)(processor, direction, index as i32, &mut arrangement)
                == K_RESULT_OK
            {
                arrangement
            } else if *channels == 2 {
                STEREO_ARRANGEMENT
            } else {
                0
            }
        })
        .collect()
}

fn pointer_or_null(values: &mut [u64]) -> *mut u64 {
    if values.is_empty() {
        ptr::null_mut()
    } else {
        values.as_mut_ptr()
    }
}

fn utf16_field_to_string(field: &[i16]) -> Option<String> {
    let units: Vec<u16> = field
        .iter()
        .copied()
        .take_while(|unit| *unit != 0)
        .map(|unit| unit as u16)
        .collect();
    if units.is_empty() {
        return None;
    }
    let text = String::from_utf16_lossy(&units).trim().to_string();
    (!text.is_empty()).then_some(text)
}

// ── Raw process session (audio thread) ──────────────────────────────────────

struct Vst3AudioBusBuffers {
    _channel_samples: Vec<Vec<Box<[f32]>>>,
    _channel_pointers: Vec<Box<[*mut f32]>>,
    descriptors: Vec<AudioBusBuffers>,
    main_index: Option<usize>,
}

impl Vst3AudioBusBuffers {
    fn new(channel_counts: &[u16], main_index: Option<usize>, max_frames: usize) -> Self {
        // The SDK permits null sample addresses for inactive buses, but some
        // multi-output frameworks still render every declared bus. Back all
        // channels with discardable scratch so those plugins remain safe.
        let mut channel_samples = channel_counts
            .iter()
            .map(|channels| {
                (0..usize::from(*channels))
                    .map(|_| vec![0.0; max_frames].into_boxed_slice())
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();
        let flat_pointers = channel_samples
            .iter_mut()
            .flat_map(|channels| channels.iter_mut().map(|samples| samples.as_mut_ptr()))
            .collect::<Vec<_>>();
        let mut channel_offset = 0;
        let mut channel_pointers = channel_counts
            .iter()
            .map(|channels| {
                let own_start = channel_offset;
                let own_end = own_start + usize::from(*channels);
                channel_offset = own_end;
                flat_pointers[own_start..own_end]
                    .iter()
                    .chain(flat_pointers[..own_start].iter())
                    .chain(flat_pointers[own_end..].iter())
                    .copied()
                    .collect::<Vec<_>>()
                    .into_boxed_slice()
            })
            .collect::<Vec<_>>();
        let descriptors = channel_pointers
            .iter_mut()
            .zip(channel_counts)
            .map(|(channels, channel_count)| AudioBusBuffers {
                // `channel_pointers` deliberately carries fallback pointers
                // after this bus's own channels, but the VST3 descriptor must
                // still advertise this bus's declared channel count. Using
                // the backing slice length here makes every bus appear to
                // contain the total channels across the plugin, which breaks
                // multi-output instruments such as Kontakt.
                num_channels: i32::from(*channel_count),
                silence_flags: 0,
                channel_buffers32: channels.as_mut_ptr(),
            })
            .collect();
        Self {
            _channel_samples: channel_samples,
            _channel_pointers: channel_pointers,
            descriptors,
            main_index,
        }
    }

    fn copy_main_from(&mut self, left: &[f32], right: &[f32], frames: usize) {
        let Some(index) = self.main_index else {
            return;
        };
        let channels = &mut self._channel_samples[index];
        if channels.len() >= 2 {
            channels[0][..frames].copy_from_slice(&left[..frames]);
            channels[1][..frames].copy_from_slice(&right[..frames]);
        }
    }

    fn copy_main_to(&self, left: &mut [f32], right: &mut [f32], frames: usize) {
        let Some(index) = self.main_index else {
            return;
        };
        let channels = &self._channel_samples[index];
        if channels.len() >= 2 {
            left[..frames].copy_from_slice(&channels[0][..frames]);
            right[..frames].copy_from_slice(&channels[1][..frames]);
        }
    }

    fn clear(&mut self, frames: usize) {
        for bus in &mut self._channel_samples {
            for channel in bus {
                channel[..frames].fill(0.0);
            }
        }
    }

    fn as_mut_ptr(&mut self) -> *mut AudioBusBuffers {
        if self.descriptors.is_empty() {
            ptr::null_mut()
        } else {
            self.descriptors.as_mut_ptr()
        }
    }

    fn len(&self) -> i32 {
        self.descriptors.len() as i32
    }
}

/// Raw, movable process handle for one activated VST3 instance: the
/// `IAudioProcessor` pointer plus planar stereo buffers preallocated at the
/// activated max block size. The sandbox moves this onto its audio thread;
/// the owning [`Vst3HostedInstance`] must outlive it and must not run
/// lifecycle transitions while the session is live. The per-block
/// `ProcessData`/`AudioBusBuffers` structs are stack-built from the
/// preallocated buffers, so processing never allocates.
pub struct Vst3ProcessSession {
    processor: *mut c_void,
    sample_rate_hz: f64,
    project_time_samples: i64,
    input_left: Vec<f32>,
    input_right: Vec<f32>,
    output_left: Vec<f32>,
    output_right: Vec<f32>,
    input_buses: Vst3AudioBusBuffers,
    output_buses: Vst3AudioBusBuffers,
    processing: bool,
    /// Pending param writes shared with the owning instance (g12.023).
    param_changes: Arc<PluginParamChangeQueue>,
    /// Drain scratch (preallocated; audio thread never allocates).
    param_scratch: Vec<PluginParamChange>,
    /// The host-side `IParameterChanges` rebuilt per block; boxed so the
    /// pointers handed to the plugin stay stable.
    input_changes: Box<HostParameterChanges>,
    /// The host-side input `IEventList` rebuilt per block (note events).
    input_events: Box<HostEventList>,
    /// Writable sinks for plugin-originated parameter changes and events.
    /// They are cleared each block; inspection does not consume them yet.
    output_changes: Box<HostParameterChanges>,
    output_events: Box<HostEventList>,
    /// CC → parameter assignments (`IMidiMapping`, queried at load); `None`
    /// drops CC events.
    midi_cc_params: Option<Arc<[Option<u32>; VST3_MIDI_CONTROLLER_COUNT]>>,
}

// Safety: the session is handed to exactly one audio thread;
// `setProcessing`/`process` are the VST3 processing-thread methods, and the
// owner serializes lifecycle against the session per the type contract.
unsafe impl Send for Vst3ProcessSession {}

impl Vst3ProcessSession {
    fn new(
        processor: *mut c_void,
        sample_rate_hz: f64,
        max_frames: usize,
        audio_bus_layout: Vst3AudioBusLayout,
        param_changes: Arc<PluginParamChangeQueue>,
        midi_cc_params: Option<Arc<[Option<u32>; VST3_MIDI_CONTROLLER_COUNT]>>,
    ) -> Self {
        Self {
            processor,
            sample_rate_hz,
            project_time_samples: 0,
            input_left: vec![0.0; max_frames],
            input_right: vec![0.0; max_frames],
            output_left: vec![0.0; max_frames],
            output_right: vec![0.0; max_frames],
            input_buses: Vst3AudioBusBuffers::new(
                &audio_bus_layout.input_channels,
                audio_bus_layout.main_input,
                max_frames,
            ),
            output_buses: Vst3AudioBusBuffers::new(
                &audio_bus_layout.output_channels,
                audio_bus_layout.main_output,
                max_frames,
            ),
            processing: false,
            param_changes,
            param_scratch: Vec::with_capacity(PLUGIN_PARAM_CHANGE_CAPACITY),
            input_changes: HostParameterChanges::new(),
            input_events: HostEventList::new(),
            output_changes: HostParameterChanges::new(),
            output_events: HostEventList::new(),
            midi_cc_params,
        }
    }

    /// `setProcessing(true)` on the audio thread; must precede `process`.
    pub fn start(&mut self) -> Result<(), Vst3HostingError> {
        if self.processing {
            return Ok(());
        }
        let result = unsafe {
            let vtable = vtable_of::<AudioProcessorVTable>(self.processor);
            ((*vtable).set_processing)(self.processor, 1)
        };
        // Plugins may answer kNotImplemented; only hard failures block.
        let _ = result;
        self.processing = true;
        Ok(())
    }

    /// `setProcessing(false)` on the audio thread.
    pub fn stop(&mut self) {
        if !self.processing {
            return;
        }
        unsafe {
            let vtable = vtable_of::<AudioProcessorVTable>(self.processor);
            let _ = ((*vtable).set_processing)(self.processor, 0);
        }
        self.processing = false;
    }

    /// Whether `start()` has run and `stop()` has not yet.
    pub fn is_processing(&self) -> bool {
        self.processing
    }

    /// Run one block through `IAudioProcessor::process` using the
    /// preallocated planar buffers. Returns `false` on plugin error.
    ///
    /// # Safety
    /// `frames` must be within the preallocated buffer bounds (callers clamp).
    unsafe fn process_planar(&mut self, frames: usize, events: &[PluginEvent]) -> bool {
        // Drain pending param writes into the block's IParameterChanges
        // (block-boundary application, offset 0). Alloc-free.
        if self.param_changes.is_empty() {
            self.input_changes.clear();
        } else {
            self.param_changes.drain_coalesced(&mut self.param_scratch);
            self.input_changes.set_changes(&self.param_scratch);
        }
        // Note/CC delivery: notes ride the input IEventList; CC events map
        // through the load-time IMidiMapping table onto parameter-change
        // points carrying the event's intra-block sample offset (VST3's
        // input-CC contract). Unmapped or mapping-less CC drops silently —
        // there is nothing honest to send instead.
        self.input_events.clear();
        for event in events {
            match event {
                PluginEvent::Note(note) => self.input_events.push_note(note),
                PluginEvent::ControlChange(change) => {
                    if let Some(map) = &self.midi_cc_params {
                        if let Some(parameter_id) = map[usize::from(change.controller & 0x7F)] {
                            self.input_changes.push_point(
                                parameter_id,
                                change.offset_frames.min(i32::MAX as u32) as i32,
                                f64::from(change.value.clamp(0.0, 1.0)),
                            );
                        }
                    }
                }
                PluginEvent::Midi(midi) => {
                    let status = midi.status & 0xF0;
                    let (controller, value) = match status {
                        0xE0 => (
                            VST3_PITCH_BEND_CONTROLLER,
                            f64::from(
                                u16::from(midi.data1 & 0x7F) | (u16::from(midi.data2 & 0x7F) << 7),
                            ) / 16_383.0,
                        ),
                        0xD0 => (
                            VST3_AFTERTOUCH_CONTROLLER,
                            f64::from(midi.data1 & 0x7F) / 127.0,
                        ),
                        _ => continue,
                    };
                    if let Some(parameter_id) =
                        self.midi_cc_params.as_ref().and_then(|map| map[controller])
                    {
                        self.input_changes.push_point(
                            parameter_id,
                            midi.offset_frames.min(i32::MAX as u32) as i32,
                            value,
                        );
                    }
                }
                _ => {}
            }
        }
        self.output_changes.clear();
        self.output_events.clear();
        let input_parameter_changes =
            (&mut *self.input_changes as *mut HostParameterChanges).cast();
        let input_events = (&mut *self.input_events as *mut HostEventList).cast();
        let output_parameter_changes =
            (&mut *self.output_changes as *mut HostParameterChanges).cast();
        let output_events = (&mut *self.output_events as *mut HostEventList).cast();
        self.input_buses.clear(frames);
        self.input_buses
            .copy_main_from(&self.input_left, &self.input_right, frames);
        self.output_buses.clear(frames);
        let num_inputs = self.input_buses.len();
        let num_outputs = self.output_buses.len();
        let inputs = self.input_buses.as_mut_ptr();
        let outputs = self.output_buses.as_mut_ptr();
        let project_time_music = self.project_time_samples as f64 / self.sample_rate_hz * 2.0;
        let mut process_context = ProcessContext {
            state: K_PROJECT_TIME_MUSIC_VALID
                | K_TEMPO_VALID
                | K_BAR_POSITION_VALID
                | K_TIME_SIG_VALID
                | K_CONT_TIME_VALID,
            sample_rate: self.sample_rate_hz,
            project_time_samples: self.project_time_samples,
            system_time: 0,
            continuous_time_samples: self.project_time_samples,
            project_time_music,
            bar_position_music: (project_time_music / 4.0).floor() * 4.0,
            cycle_start_music: 0.0,
            cycle_end_music: 0.0,
            tempo: 120.0,
            time_sig_numerator: 4,
            time_sig_denominator: 4,
            chord: ProcessChord {
                key_note: 0,
                root_note: 0,
                chord_mask: 0,
            },
            smpte_offset_subframes: 0,
            frame_rate: ProcessFrameRate {
                frames_per_second: 0,
                flags: 0,
            },
            samples_to_next_clock: 0,
        };
        let mut data = ProcessData {
            process_mode: K_REALTIME,
            symbolic_sample_size: K_SAMPLE32,
            num_samples: frames as i32,
            num_inputs,
            num_outputs,
            inputs,
            outputs,
            input_parameter_changes,
            output_parameter_changes,
            input_events,
            output_events,
            process_context: (&mut process_context as *mut ProcessContext).cast(),
        };
        let vtable = vtable_of::<AudioProcessorVTable>(self.processor);
        let result = ((*vtable).process)(self.processor, &mut data) == K_RESULT_OK;
        self.output_buses
            .copy_main_to(&mut self.output_left, &mut self.output_right, frames);
        self.project_time_samples += frames as i64;
        result
    }

    /// Process one block: interleaved stereo in, interleaved stereo out.
    /// Alloc-free (buffers preallocated at activate). On plugin error the
    /// input passes through unchanged. Returns `false` on error.
    pub fn process_interleaved_stereo(
        &mut self,
        input: &[f32],
        output: &mut [f32],
        frame_count: usize,
    ) -> bool {
        let frames = frame_count
            .min(self.input_left.len())
            .min(input.len() / 2)
            .min(output.len() / 2);
        for frame in 0..frames {
            self.input_left[frame] = input[frame * 2];
            self.input_right[frame] = input[frame * 2 + 1];
        }
        if !unsafe { self.process_planar(frames, &[]) } {
            output[..frames * 2].copy_from_slice(&input[..frames * 2]);
            return false;
        }
        for frame in 0..frames {
            output[frame * 2] = self.output_left[frame];
            output[frame * 2 + 1] = self.output_right[frame];
        }
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
    /// (sorted by `offset_frames`): note events ride the input
    /// `IEventList`; CC events become `IMidiMapping`-mapped parameter
    /// changes at their sample offsets. Alloc-free. `true` = buffer
    /// transformed.
    pub fn process_in_place_with_events(
        &mut self,
        io: &mut [f32],
        frame_count: usize,
        events: &[PluginEvent],
    ) -> bool {
        let frames = frame_count.min(self.input_left.len()).min(io.len() / 2);
        for frame in 0..frames {
            self.input_left[frame] = io[frame * 2];
            self.input_right[frame] = io[frame * 2 + 1];
        }
        if !unsafe { self.process_planar(frames, events) } {
            return false;
        }
        for frame in 0..frames {
            io[frame * 2] = self.output_left[frame];
            io[frame * 2 + 1] = self.output_right[frame];
        }
        true
    }
}

#[cfg(test)]
mod tuid_tests {
    use super::*;

    #[test]
    fn first_declared_main_bus_wins_for_multi_output_instruments() {
        let first = select_main_bus(None, K_MAIN, 0);
        let second = select_main_bus(first, K_MAIN, 1);

        assert_eq!(second, Some(0));
    }

    #[test]
    fn multi_output_buffers_report_each_bus_declared_channel_count() {
        let buffers = Vst3AudioBusBuffers::new(&[2, 2, 1], Some(0), 64);

        assert_eq!(
            buffers
                .descriptors
                .iter()
                .map(|descriptor| descriptor.num_channels)
                .collect::<Vec<_>>(),
            vec![2, 2, 1]
        );
    }

    #[test]
    fn stereo_processor_layout_accepts_effects_and_instruments_only() {
        let effect = Vst3HostedPortLayout {
            main_input_channels: 2,
            main_output_channels: 2,
        };
        let instrument = Vst3HostedPortLayout {
            main_input_channels: 0,
            main_output_channels: 2,
        };
        let mono_output = Vst3HostedPortLayout {
            main_input_channels: 0,
            main_output_channels: 1,
        };
        let surround = Vst3HostedPortLayout {
            main_input_channels: 2,
            main_output_channels: 6,
        };

        assert!(effect.is_stereo_effect());
        assert!(!effect.is_stereo_instrument());
        assert!(effect.is_supported_stereo_processor());
        assert!(!instrument.is_stereo_effect());
        assert!(instrument.is_stereo_instrument());
        assert!(instrument.is_supported_stereo_processor());
        assert!(!mono_output.is_supported_stereo_processor());
        assert!(!surround.is_supported_stereo_processor());
    }

    #[test]
    fn tuid_layout_matches_platform_expectations() {
        let tuid = tuid_from_uid(0x11223344, 0x55667788, 0x99AABBCC, 0xDDEEFF00);
        if cfg!(target_os = "windows") {
            assert_eq!(
                tuid,
                [
                    0x44, 0x33, 0x22, 0x11, 0x66, 0x55, 0x88, 0x77, 0x99, 0xAA, 0xBB, 0xCC, 0xDD,
                    0xEE, 0xFF, 0x00
                ]
            );
        } else {
            assert_eq!(
                tuid,
                [
                    0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88, 0x99, 0xAA, 0xBB, 0xCC, 0xDD,
                    0xEE, 0xFF, 0x00
                ]
            );
        }
    }

    #[test]
    fn class_id_hex_round_trips_the_interface_encoding() {
        // The canonical hex of a component class must decode to the same
        // in-memory TUID that `tuid_from_uid` builds for those fields.
        let expected = tuid_from_uid(0x11223344, 0x55667788, 0x99AABBCC, 0xDDEEFF00);
        let decoded = tuid_from_class_id_hex("112233445566778899AABBCCDDEEFF00")
            .expect("canonical hex should decode");
        assert_eq!(decoded, expected);
        assert!(tuid_from_class_id_hex("nonsense").is_none());
        assert!(tuid_from_class_id_hex("1122").is_none());
    }

    #[test]
    fn state_envelope_round_trips_component_and_controller_state() {
        let encoded = encode_state_envelope(b"component-state", b"controller-state");
        let (component, controller) =
            decode_state_envelope(&encoded).expect("valid state envelope");

        assert_eq!(component, b"component-state");
        assert_eq!(controller, b"controller-state");
        assert!(decode_state_envelope(b"not-a-state-envelope").is_none());

        let mut trailing_bytes = encoded;
        trailing_bytes.push(0);
        assert!(decode_state_envelope(&trailing_bytes).is_none());
    }

    #[test]
    fn memory_stream_supports_plugin_write_seek_and_read_calls() {
        let mut stream = MemoryStream::writer();
        let source = b"plugin-state";
        let mut written = 0;
        let result = unsafe {
            stream_write(
                stream.as_raw(),
                source.as_ptr().cast(),
                source.len() as i32,
                &mut written,
            )
        };
        assert_eq!(result, K_RESULT_OK);
        assert_eq!(written, source.len() as i32);

        let mut position = -1;
        assert_eq!(
            unsafe { stream_seek(stream.as_raw(), 0, 0, &mut position) },
            K_RESULT_OK
        );
        assert_eq!(position, 0);

        let mut destination = [0u8; 12];
        let mut read = 0;
        assert_eq!(
            unsafe {
                stream_read(
                    stream.as_raw(),
                    destination.as_mut_ptr().cast(),
                    destination.len() as i32,
                    &mut read,
                )
            },
            K_RESULT_OK
        );
        assert_eq!(read, destination.len() as i32);
        assert_eq!(&destination, source);
    }
}
