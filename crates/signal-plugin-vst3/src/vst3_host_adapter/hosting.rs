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
//! (and as `moduleinfo.json` carries them on non-Windows), so
//! [`tuid_from_class_id_hex`] is a straight hex decode on macOS/Linux and
//! applies the COM swap only on Windows.

#![allow(unsafe_op_in_unsafe_fn)]
// The COM vtables mirror Steinberg's interface layouts, which are wider
// than clippy's default argument budget for a few methods.
#![allow(clippy::too_many_arguments)]

use std::ffi::{c_char, c_void, CString};
use std::path::Path;
use std::ptr;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use signal_plugin::{
    PluginEvent, PluginParamChange, PluginParamChangeQueue, PluginParameterDescriptor,
    PluginParameterDomain, PluginParameterFlags, PLUGIN_PARAM_CHANGE_CAPACITY,
};

#[cfg(not(target_os = "macos"))]
use libloading::Library;

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
const ICOMPONENT_HANDLER_IID: Tuid = tuid_from_uid(0x93A0BEA3, 0x0BD045DB, 0x8E890B0C, 0xC1E46AC6);
const IHOST_APPLICATION_IID: Tuid = tuid_from_uid(0x58E595CC, 0xDB2D4969, 0x8B6AAF8C, 0x36A664E5);
// ivstparameterchanges.h (published interface definitions).
const IPARAMETER_CHANGES_IID: Tuid = tuid_from_uid(0xA4779663, 0x0BB64A56, 0xB44384A8, 0x466FEB9D);
const IPARAM_VALUE_QUEUE_IID: Tuid = tuid_from_uid(0x01263A18, 0xED074F6F, 0x98C9D356, 0x4686F9BA);
// ivstevents.h / ivstmidicontrollers.h (published interface definitions).
const IEVENT_LIST_IID: Tuid = tuid_from_uid(0x3A2C4214, 0x346349FE, 0xB2C4F397, 0xB9695A44);
const IMIDI_MAPPING_IID: Tuid = tuid_from_uid(0xDF695DF2, 0x8B4B47EB, 0xAB3EF8FB, 0x2D1F6BB2);
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
/// `kSpeakerL | kSpeakerR`.
const STEREO_ARRANGEMENT: u64 = 0x3;

// ParameterInfo flags.
const PARAM_CAN_AUTOMATE: i32 = 1;
const PARAM_IS_READ_ONLY: i32 = 1 << 1;
const PARAM_IS_HIDDEN: i32 = 1 << 4;
const PARAM_IS_BYPASS: i32 = 1 << 16;
/// `RestartFlags::kLatencyChanged` from `ivsteditcontroller.h`.
const RESTART_LATENCY_CHANGED: i32 = 1 << 3;

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
    if !this.is_null() && flags & RESTART_LATENCY_CHANGED != 0 {
        (*(this.cast::<ComponentHandler>()))
            .latency_changes
            .fetch_add(1, Ordering::Relaxed);
    }
    K_RESULT_OK
}

#[cfg(test)]
mod component_handler_tests {
    use super::*;

    #[test]
    fn only_latency_restart_flags_advance_the_revision() {
        let mut handler = Box::new(ComponentHandler {
            vtable: &COMPONENT_HANDLER_VTABLE,
            latency_changes: AtomicU64::new(0),
        });
        let ptr = (&mut *handler as *mut ComponentHandler).cast();

        unsafe {
            component_handler_restart_component(ptr, 1 << 1);
            component_handler_restart_component(ptr, RESTART_LATENCY_CHANGED);
        }

        assert_eq!(handler.latency_changes.load(Ordering::Relaxed), 1);
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
unsafe fn com_query_interface(object: *mut c_void, iid: &Tuid) -> Option<*mut c_void> {
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
    _cid: *mut u8,
    _iid: *mut u8,
    out: *mut *mut c_void,
) -> Tresult {
    if !out.is_null() {
        *out = ptr::null_mut();
    }
    K_NO_INTERFACE
}

fn host_context() -> *mut c_void {
    &HOST_APPLICATION as *const StaticHostApplication as *mut c_void
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
            let exit = bundle.exit();
            Ok(Self {
                bundle,
                factory,
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
            let exit = library
                .get::<ExitProc>(exit_symbol(platform))
                .ok()
                .map(|symbol| *symbol);
            Ok(Self {
                library,
                factory,
                exit,
            })
        }
    }

    /// Create an instance of `cid` exposing `iid` through the factory.
    unsafe fn create_instance(&self, cid: &Tuid, iid: &Tuid) -> Option<*mut c_void> {
        let vtable = vtable_of::<PluginFactoryVTable>(self.factory);
        let mut out: *mut c_void = ptr::null_mut();
        let result =
            ((*vtable).create_instance)(self.factory, cid.as_ptr(), iid.as_ptr(), &mut out);
        (result == K_RESULT_OK && !out.is_null()).then_some(out)
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
    _this: *mut c_void,
    _id: *const u32,
    _index: *mut i32,
) -> *mut c_void {
    // Input list: the host writes it, the plugin only reads.
    ptr::null_mut()
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

unsafe extern "C" fn event_list_add_event(_this: *mut c_void, _event: *mut Vst3Event) -> Tresult {
    // Input list: the host writes it, the plugin only reads.
    K_NO_INTERFACE
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

impl Vst3HostedPortLayout {
    /// Phase 1 supports exactly a stereo main in + stereo main out effect.
    pub fn is_stereo_effect(&self) -> bool {
        self.main_input_channels == 2 && self.main_output_channels == 2
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
    /// Stable host callback object installed on the edit controller.
    component_handler: Option<Box<ComponentHandler>>,
    parameters: Vec<PluginParameterDescriptor>,
    port_layout: Vst3HostedPortLayout,
    state: HostedInstanceState,
    activated_max_frames: u32,
    /// Whether the controller produced an editor view at the load-time
    /// probe (g12.024): `createView("editor")` returned non-null. Cached —
    /// `gui_supported` must stay a cheap read for the states poll.
    gui_view_supported: bool,
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
        let cid = tuid_from_class_id_hex(class_id_hex)
            .ok_or_else(|| Vst3HostingError::new("class_id_invalid"))?;
        let module = LoadedVst3Module::load(bundle_root)?;

        let component = unsafe { module.create_instance(&cid, &ICOMPONENT_IID) }
            .ok_or_else(|| Vst3HostingError::new("create_component_failed"))?;
        unsafe {
            let vtable = vtable_of::<ComponentVTable>(component);
            if ((*vtable).initialize)(component, host_context()) != K_RESULT_OK {
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

        let controller = unsafe { acquire_controller(component, &module) };
        let component_handler = controller.as_ref().and_then(|controller| unsafe {
            let mut handler = Box::new(ComponentHandler {
                vtable: &COMPONENT_HANDLER_VTABLE,
                latency_changes: AtomicU64::new(0),
            });
            let vtable = vtable_of::<EditControllerVTable>(controller.ptr());
            let ptr = (&mut *handler as *mut ComponentHandler).cast();
            (((*vtable).set_component_handler)(controller.ptr(), ptr) == K_RESULT_OK)
                .then_some(handler)
        });
        let parameters = controller
            .as_ref()
            .map(|handle| unsafe { parameter_inventory(handle.ptr()) })
            .unwrap_or_default();
        let port_layout = unsafe { main_bus_layout(component) };
        // Editor probe (g12.024): createView + immediate release — the
        // standard capability check. Mirrors the CLAP load-time
        // `is_api_supported` probe's threading posture.
        let gui_view_supported = controller
            .as_ref()
            .map(|handle| unsafe {
                let view = controller_create_view(handle.ptr());
                if view.is_null() {
                    false
                } else {
                    com_release(view);
                    true
                }
            })
            .unwrap_or(false);
        let midi_cc_params = controller
            .as_ref()
            .and_then(|handle| unsafe { midi_cc_parameter_map(handle.ptr()) });

        Ok(Self {
            component,
            processor,
            controller,
            component_handler,
            parameters,
            port_layout,
            state: HostedInstanceState::Created,
            activated_max_frames: 0,
            gui_view_supported,
            gui_session: None,
            param_changes: Arc::new(PluginParamChangeQueue::new()),
            midi_cc_params,
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

    /// Main-bus port layout enumerated at load.
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

    /// Activate for processing: stereo/stereo bus arrangement (verified via
    /// `getBusArrangement`), 32-bit samples, `setupProcessing`, main buses
    /// activated, `setActive(true)`. Non-stereo negotiation fails with the
    /// stable `layout_unsupported` token, same as the CLAP path.
    pub fn activate(
        &mut self,
        sample_rate_hz: f64,
        _min_frames: u32,
        max_frames: u32,
    ) -> Result<(), Vst3HostingError> {
        if self.state == HostedInstanceState::Active {
            return Err(Vst3HostingError::new("already_active"));
        }
        unsafe {
            let processor = vtable_of::<AudioProcessorVTable>(self.processor);

            // Negotiate stereo/stereo BEFORE setActive; a plugin may reject
            // the call yet still report stereo, so trust getBusArrangement.
            let mut input_arrangement = STEREO_ARRANGEMENT;
            let mut output_arrangement = STEREO_ARRANGEMENT;
            let _ = ((*processor).set_bus_arrangements)(
                self.processor,
                &mut input_arrangement,
                1,
                &mut output_arrangement,
                1,
            );
            let mut verified_input = 0u64;
            let mut verified_output = 0u64;
            if ((*processor).get_bus_arrangement)(self.processor, K_INPUT, 0, &mut verified_input)
                != K_RESULT_OK
                || ((*processor).get_bus_arrangement)(
                    self.processor,
                    K_OUTPUT,
                    0,
                    &mut verified_output,
                ) != K_RESULT_OK
                || verified_input != STEREO_ARRANGEMENT
                || verified_output != STEREO_ARRANGEMENT
            {
                return Err(Vst3HostingError::new("layout_unsupported"));
            }

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
            let _ = ((*component).activate_bus)(self.component, K_AUDIO, K_INPUT, 0, 1);
            let _ = ((*component).activate_bus)(self.component, K_AUDIO, K_OUTPUT, 0, 1);
            if ((*component).set_active)(self.component, 1) != K_RESULT_OK {
                return Err(Vst3HostingError::new("set_active_failed"));
            }
        }
        self.state = HostedInstanceState::Active;
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

    /// Whether the controller produced an editor view at the load-time
    /// probe. Cached at load.
    pub fn gui_supported(&self) -> bool {
        self.gui_view_supported
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
            self.activated_max_frames as usize,
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
        // `_module` drops after this body: exit proc, then dlclose.
    }
}

/// Acquire the edit controller: component facet first, else the separate
/// controller class through the factory. `None` = no parameter inventory.
unsafe fn acquire_controller(
    component: *mut c_void,
    module: &LoadedVst3Module,
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
    if ((*controller_vtable).initialize)(controller, host_context()) != K_RESULT_OK {
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

/// Read the main audio bus channel counts (bus 0 per direction, preferring
/// an explicit `kMain` bus when one exists).
unsafe fn main_bus_layout(component: *mut c_void) -> Vst3HostedPortLayout {
    let vtable = vtable_of::<ComponentVTable>(component);
    let mut layout = Vst3HostedPortLayout {
        main_input_channels: 0,
        main_output_channels: 0,
    };
    for (direction, slot) in [
        (K_INPUT, &mut layout.main_input_channels),
        (K_OUTPUT, &mut layout.main_output_channels),
    ] {
        let count = ((*vtable).get_bus_count)(component, K_AUDIO, direction).max(0);
        let mut fallback: Option<u16> = None;
        for index in 0..count {
            let mut info = BusInfo::zeroed();
            if ((*vtable).get_bus_info)(component, K_AUDIO, direction, index, &mut info)
                != K_RESULT_OK
            {
                continue;
            }
            let channels = info.channel_count.clamp(0, u16::MAX as i32) as u16;
            if info.bus_type == K_MAIN {
                *slot = channels;
                fallback = None;
                break;
            }
            if fallback.is_none() {
                fallback = Some(channels);
            }
        }
        if let Some(channels) = fallback {
            *slot = channels;
        }
    }
    layout
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

/// Raw, movable process handle for one activated VST3 instance: the
/// `IAudioProcessor` pointer plus planar stereo buffers preallocated at the
/// activated max block size. The sandbox moves this onto its audio thread;
/// the owning [`Vst3HostedInstance`] must outlive it and must not run
/// lifecycle transitions while the session is live. The per-block
/// `ProcessData`/`AudioBusBuffers` structs are stack-built from the
/// preallocated buffers, so processing never allocates.
pub struct Vst3ProcessSession {
    processor: *mut c_void,
    input_left: Vec<f32>,
    input_right: Vec<f32>,
    output_left: Vec<f32>,
    output_right: Vec<f32>,
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
        max_frames: usize,
        param_changes: Arc<PluginParamChangeQueue>,
        midi_cc_params: Option<Arc<[Option<u32>; VST3_MIDI_CONTROLLER_COUNT]>>,
    ) -> Self {
        Self {
            processor,
            input_left: vec![0.0; max_frames],
            input_right: vec![0.0; max_frames],
            output_left: vec![0.0; max_frames],
            output_right: vec![0.0; max_frames],
            processing: false,
            param_changes,
            param_scratch: Vec::with_capacity(PLUGIN_PARAM_CHANGE_CAPACITY),
            input_changes: HostParameterChanges::new(),
            input_events: HostEventList::new(),
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
        let input_parameter_changes: *mut c_void = if self.input_changes.active > 0 {
            (&mut *self.input_changes as *mut HostParameterChanges).cast()
        } else {
            ptr::null_mut()
        };
        let input_events: *mut c_void = if self.input_events.active > 0 {
            (&mut *self.input_events as *mut HostEventList).cast()
        } else {
            ptr::null_mut()
        };
        let mut input_channels = [self.input_left.as_mut_ptr(), self.input_right.as_mut_ptr()];
        let mut output_channels = [
            self.output_left.as_mut_ptr(),
            self.output_right.as_mut_ptr(),
        ];
        let mut input_bus = AudioBusBuffers {
            num_channels: 2,
            silence_flags: 0,
            channel_buffers32: input_channels.as_mut_ptr(),
        };
        let mut output_bus = AudioBusBuffers {
            num_channels: 2,
            silence_flags: 0,
            channel_buffers32: output_channels.as_mut_ptr(),
        };
        let mut data = ProcessData {
            process_mode: K_REALTIME,
            symbolic_sample_size: K_SAMPLE32,
            num_samples: frames as i32,
            num_inputs: 1,
            num_outputs: 1,
            inputs: &mut input_bus,
            outputs: &mut output_bus,
            input_parameter_changes,
            output_parameter_changes: ptr::null_mut(),
            input_events,
            output_events: ptr::null_mut(),
            process_context: ptr::null_mut(),
        };
        let vtable = vtable_of::<AudioProcessorVTable>(self.processor);
        ((*vtable).process)(self.processor, &mut data) == K_RESULT_OK
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
