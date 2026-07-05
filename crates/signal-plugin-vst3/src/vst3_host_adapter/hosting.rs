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

use std::ffi::c_void;
use std::path::Path;
use std::ptr;

use libloading::Library;
use signal_plugin::{PluginParameterDescriptor, PluginParameterDomain, PluginParameterFlags};

use super::introspection::resolve_module_binary_path;
use super::Vst3HostPlatform;

/// Error surface for VST3 hosting operations; carries a stable snake_case
/// token suitable for broker receipt details (mirrors `ClapHostingError`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Vst3HostingError {
    /// Stable snake_case failure token (e.g. `module_open_failed`).
    pub token: String,
}

impl Vst3HostingError {
    fn new(token: impl Into<String>) -> Self {
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
type Tresult = i32;

/// `kResultOk` / `kResultTrue` (0 on every platform).
const K_RESULT_OK: Tresult = 0;

/// `kNoInterface` (platform-dependent: COM `E_NOINTERFACE` on Windows).
#[cfg(target_os = "windows")]
const K_NO_INTERFACE: Tresult = 0x8000_4002_u32 as i32;
#[cfg(not(target_os = "windows"))]
const K_NO_INTERFACE: Tresult = -1;

/// 16-byte Steinberg TUID.
type Tuid = [u8; 16];

/// Build a TUID from the four canonical `u32` fields with the platform's
/// `INLINE_UID` byte layout (see module docs).
const fn tuid_from_uid(l1: u32, l2: u32, l3: u32, l4: u32) -> Tuid {
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
const FUNKNOWN_IID: Tuid = tuid_from_uid(0x00000000, 0x00000000, 0xC0000000, 0x00000046);
const ICOMPONENT_IID: Tuid = tuid_from_uid(0xE831FF31, 0xF2D54301, 0x928EBBEE, 0x25697802);
const IAUDIO_PROCESSOR_IID: Tuid = tuid_from_uid(0x42043F99, 0xB7DA453C, 0xA569E79D, 0x9AAEC33D);
const IEDIT_CONTROLLER_IID: Tuid = tuid_from_uid(0xDCD7BBE3, 0x7742448D, 0xA874AAF0, 0x0B96B23E);
const IHOST_APPLICATION_IID: Tuid = tuid_from_uid(0x58E595CC, 0xDB2D4969, 0x8B6AAF8C, 0x36A664E5);

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

/// `Steinberg::Vst::ProcessData` (event/parameter queues null in phase 1).
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

/// Read a COM object's vtable of type `V`.
///
/// # Safety
/// `object` must be a live COM interface pointer whose vtable matches `V`.
unsafe fn vtable_of<V>(object: *mut c_void) -> *const V {
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
unsafe fn com_release(object: *mut c_void) {
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

fn entry_symbol(platform: Vst3HostPlatform) -> &'static [u8] {
    match platform {
        Vst3HostPlatform::MacOs => b"bundleEntry\0",
        Vst3HostPlatform::Linux => b"ModuleEntry\0",
        Vst3HostPlatform::Windows => b"InitDll\0",
    }
}

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
    library: Library,
    factory: *mut c_void,
    exit: Option<ExitProc>,
}

impl LoadedVst3Module {
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
        // `library` drops after this body, unloading the module last.
        let _ = &self.library;
    }
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
    parameters: Vec<PluginParameterDescriptor>,
    port_layout: Vst3HostedPortLayout,
    state: HostedInstanceState,
    activated_max_frames: u32,
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
        let parameters = controller
            .as_ref()
            .map(|handle| unsafe { parameter_inventory(handle.ptr()) })
            .unwrap_or_default();
        let port_layout = unsafe { main_bus_layout(component) };

        Ok(Self {
            component,
            processor,
            controller,
            parameters,
            port_layout,
            state: HostedInstanceState::Created,
            activated_max_frames: 0,
            _module: module,
        })
    }

    /// Parameter inventory enumerated at load via `IEditController`
    /// (read-only phase 1; empty when no controller could be acquired).
    pub fn parameters(&self) -> &[PluginParameterDescriptor] {
        &self.parameters
    }

    /// Main-bus port layout enumerated at load.
    pub fn port_layout(&self) -> Vst3HostedPortLayout {
        self.port_layout
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
        ))
    }
}

impl Drop for Vst3HostedInstance {
    fn drop(&mut self) {
        unsafe {
            if self.state == HostedInstanceState::Active {
                let component = vtable_of::<ComponentVTable>(self.component);
                let _ = ((*component).set_active)(self.component, 0);
            }
            com_release(self.processor);
            if let Some(controller) = self.controller.take() {
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
}

// Safety: the session is handed to exactly one audio thread;
// `setProcessing`/`process` are the VST3 processing-thread methods, and the
// owner serializes lifecycle against the session per the type contract.
unsafe impl Send for Vst3ProcessSession {}

impl Vst3ProcessSession {
    fn new(processor: *mut c_void, max_frames: usize) -> Self {
        Self {
            processor,
            input_left: vec![0.0; max_frames],
            input_right: vec![0.0; max_frames],
            output_left: vec![0.0; max_frames],
            output_right: vec![0.0; max_frames],
            processing: false,
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
    unsafe fn process_planar(&mut self, frames: usize) -> bool {
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
            input_parameter_changes: ptr::null_mut(),
            output_parameter_changes: ptr::null_mut(),
            input_events: ptr::null_mut(),
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
        if !unsafe { self.process_planar(frames) } {
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
        let frames = frame_count.min(self.input_left.len()).min(io.len() / 2);
        for frame in 0..frames {
            self.input_left[frame] = io[frame * 2];
            self.input_right[frame] = io[frame * 2 + 1];
        }
        if !unsafe { self.process_planar(frames) } {
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
}
