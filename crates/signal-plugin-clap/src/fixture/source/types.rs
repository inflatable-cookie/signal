use super::super::{CLAP_FIXTURE_GAIN, CLAP_FIXTURE_GUI_INITIAL_SIZE};

pub(crate) fn types_fragment(instrument: bool) -> String {
    format!(
        r#"use std::ffi::{{c_char, c_void, CStr}};
use std::ptr;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct clap_version {{
    pub major: u32,
    pub minor: u32,
    pub revision: u32,
}}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct clap_plugin_entry {{
    pub clap_version: clap_version,
    pub init: Option<unsafe extern "C" fn(*const c_char) -> bool>,
    pub deinit: Option<unsafe extern "C" fn()>,
    pub get_factory: Option<unsafe extern "C" fn(*const c_char) -> *const c_void>,
}}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct clap_plugin_descriptor {{
    pub clap_version: clap_version,
    pub id: *const c_char,
    pub name: *const c_char,
    pub vendor: *const c_char,
    pub url: *const c_char,
    pub manual_url: *const c_char,
    pub support_url: *const c_char,
    pub version: *const c_char,
    pub description: *const c_char,
    pub features: *const *const c_char,
}}

unsafe impl Sync for clap_plugin_descriptor {{}}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct clap_audio_buffer {{
    pub data32: *mut *mut f32,
    pub data64: *mut *mut f64,
    pub channel_count: u32,
    pub latency: u32,
    pub constant_mask: u64,
}}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct clap_process {{
    pub steady_time: i64,
    pub frames_count: u32,
    pub transport: *const c_void,
    pub audio_inputs: *const clap_audio_buffer,
    pub audio_outputs: *mut clap_audio_buffer,
    pub audio_inputs_count: u32,
    pub audio_outputs_count: u32,
    pub in_events: *const c_void,
    pub out_events: *const c_void,
}}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct clap_plugin {{
    pub desc: *const clap_plugin_descriptor,
    pub plugin_data: *mut c_void,
    pub init: Option<unsafe extern "C" fn(*const clap_plugin) -> bool>,
    pub destroy: Option<unsafe extern "C" fn(*const clap_plugin)>,
    pub activate: Option<unsafe extern "C" fn(*const clap_plugin, f64, u32, u32) -> bool>,
    pub deactivate: Option<unsafe extern "C" fn(*const clap_plugin)>,
    pub start_processing: Option<unsafe extern "C" fn(*const clap_plugin) -> bool>,
    pub stop_processing: Option<unsafe extern "C" fn(*const clap_plugin)>,
    pub reset: Option<unsafe extern "C" fn(*const clap_plugin)>,
    pub process: Option<unsafe extern "C" fn(*const clap_plugin, *const clap_process) -> i32>,
    pub get_extension: Option<unsafe extern "C" fn(*const clap_plugin, *const c_char) -> *const c_void>,
    pub on_main_thread: Option<unsafe extern "C" fn(*const clap_plugin)>,
}}

unsafe impl Sync for clap_plugin {{}}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct clap_plugin_factory {{
    pub get_plugin_count: Option<unsafe extern "C" fn(*const clap_plugin_factory) -> u32>,
    pub get_plugin_descriptor: Option<unsafe extern "C" fn(*const clap_plugin_factory, u32) -> *const clap_plugin_descriptor>,
    pub create_plugin: Option<unsafe extern "C" fn(*const clap_plugin_factory, *const c_void, *const c_char) -> *const clap_plugin>,
}}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct clap_audio_port_info {{
    pub id: u32,
    pub name: [c_char; 256],
    pub flags: u32,
    pub channel_count: u32,
    pub port_type: *const c_char,
    pub in_place_pair: u32,
}}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct clap_plugin_audio_ports {{
    pub count: Option<unsafe extern "C" fn(*const clap_plugin, bool) -> u32>,
    pub get: Option<unsafe extern "C" fn(*const clap_plugin, u32, bool, *mut clap_audio_port_info) -> bool>,
}}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct clap_plugin_latency {{
    pub get: Option<unsafe extern "C" fn(*const clap_plugin) -> u32>,
}}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct clap_note_port_info {{
    pub id: u32,
    pub supported_dialects: u32,
    pub preferred_dialect: u32,
    pub name: [c_char; 256],
}}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct clap_plugin_note_ports {{
    pub count: Option<unsafe extern "C" fn(*const clap_plugin, bool) -> u32>,
    pub get: Option<unsafe extern "C" fn(*const clap_plugin, u32, bool, *mut clap_note_port_info) -> bool>,
}}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct clap_param_info {{
    pub id: u32,
    pub flags: u32,
    pub cookie: *mut c_void,
    pub name: [c_char; 256],
    pub module: [c_char; 1024],
    pub min_value: f64,
    pub max_value: f64,
    pub default_value: f64,
}}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct clap_plugin_params {{
    pub count: Option<unsafe extern "C" fn(*const clap_plugin) -> u32>,
    pub get_info: Option<unsafe extern "C" fn(*const clap_plugin, u32, *mut clap_param_info) -> bool>,
    pub get_value: Option<unsafe extern "C" fn(*const clap_plugin, u32, *mut f64) -> bool>,
    pub value_to_text: Option<unsafe extern "C" fn(*const clap_plugin, u32, f64, *mut c_char, u32) -> bool>,
    pub text_to_value: Option<unsafe extern "C" fn(*const clap_plugin, u32, *const c_char, *mut f64) -> bool>,
    pub flush: Option<unsafe extern "C" fn(*const clap_plugin, *const c_void, *const c_void)>,
}}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct clap_host {{
    pub clap_version: clap_version,
    pub host_data: *mut c_void,
    pub name: *const c_char,
    pub vendor: *const c_char,
    pub url: *const c_char,
    pub version: *const c_char,
    pub get_extension: Option<unsafe extern "C" fn(*const clap_host, *const c_char) -> *const c_void>,
    pub request_restart: Option<unsafe extern "C" fn(*const clap_host)>,
    pub request_process: Option<unsafe extern "C" fn(*const clap_host)>,
    pub request_callback: Option<unsafe extern "C" fn(*const clap_host)>,
}}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct clap_window {{
    pub api: *const c_char,
    pub specific: *mut c_void,
}}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct clap_gui_resize_hints {{
    pub can_resize_horizontally: bool,
    pub can_resize_vertically: bool,
    pub preserve_aspect_ratio: bool,
    pub aspect_ratio_width: u32,
    pub aspect_ratio_height: u32,
}}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct clap_plugin_gui {{
    pub is_api_supported: Option<unsafe extern "C" fn(*const clap_plugin, *const c_char, bool) -> bool>,
    pub get_preferred_api: Option<unsafe extern "C" fn(*const clap_plugin, *mut *const c_char, *mut bool) -> bool>,
    pub create: Option<unsafe extern "C" fn(*const clap_plugin, *const c_char, bool) -> bool>,
    pub destroy: Option<unsafe extern "C" fn(*const clap_plugin)>,
    pub set_scale: Option<unsafe extern "C" fn(*const clap_plugin, f64) -> bool>,
    pub get_size: Option<unsafe extern "C" fn(*const clap_plugin, *mut u32, *mut u32) -> bool>,
    pub can_resize: Option<unsafe extern "C" fn(*const clap_plugin) -> bool>,
    pub get_resize_hints: Option<unsafe extern "C" fn(*const clap_plugin, *mut clap_gui_resize_hints) -> bool>,
    pub adjust_size: Option<unsafe extern "C" fn(*const clap_plugin, *mut u32, *mut u32) -> bool>,
    pub set_size: Option<unsafe extern "C" fn(*const clap_plugin, u32, u32) -> bool>,
    pub set_parent: Option<unsafe extern "C" fn(*const clap_plugin, *const clap_window) -> bool>,
    pub set_transient: Option<unsafe extern "C" fn(*const clap_plugin, *const clap_window) -> bool>,
    pub suggest_title: Option<unsafe extern "C" fn(*const clap_plugin, *const c_char)>,
    pub show: Option<unsafe extern "C" fn(*const clap_plugin) -> bool>,
    pub hide: Option<unsafe extern "C" fn(*const clap_plugin) -> bool>,
}}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct clap_host_params {{
    pub rescan: Option<unsafe extern "C" fn(*const clap_host, u32)>,
    pub clear: Option<unsafe extern "C" fn(*const clap_host, u32, u32)>,
    pub request_flush: Option<unsafe extern "C" fn(*const clap_host)>,
}}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct clap_host_gui {{
    pub resize_hints_changed: Option<unsafe extern "C" fn(*const clap_host)>,
    pub request_resize: Option<unsafe extern "C" fn(*const clap_host, u32, u32) -> bool>,
    pub request_show: Option<unsafe extern "C" fn(*const clap_host) -> bool>,
    pub request_hide: Option<unsafe extern "C" fn(*const clap_host) -> bool>,
    pub closed: Option<unsafe extern "C" fn(*const clap_host, bool)>,
}}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct clap_host_state {{
    pub mark_dirty: Option<unsafe extern "C" fn(*const clap_host)>,
}}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct clap_event_header {{
    pub size: u32,
    pub time: u32,
    pub space_id: u16,
    pub type_: u16,
    pub flags: u32,
}}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct clap_event_param_value {{
    pub header: clap_event_header,
    pub param_id: u32,
    pub cookie: *mut c_void,
    pub note_id: i32,
    pub port_index: i16,
    pub channel: i16,
    pub key: i16,
    pub value: f64,
}}

#[repr(C)]
pub struct clap_input_events {{
    pub ctx: *mut c_void,
    pub size: Option<unsafe extern "C" fn(*const clap_input_events) -> u32>,
    pub get: Option<unsafe extern "C" fn(*const clap_input_events, u32) -> *const clap_event_header>,
}}

#[repr(C)]
pub struct clap_output_events {{
    pub ctx: *mut c_void,
    pub try_push: Option<unsafe extern "C" fn(*const clap_output_events, *const clap_event_header) -> bool>,
}}

#[repr(C)]
pub struct clap_istream {{
    pub ctx: *mut c_void,
    pub read: Option<unsafe extern "C" fn(*const clap_istream, *mut c_void, u64) -> i64>,
}}

#[repr(C)]
pub struct clap_ostream {{
    pub ctx: *mut c_void,
    pub write: Option<unsafe extern "C" fn(*const clap_ostream, *const c_void, u64) -> i64>,
}}

#[repr(C)]
pub struct clap_plugin_state {{
    pub save: Option<unsafe extern "C" fn(*const clap_plugin, *const clap_ostream) -> bool>,
    pub load: Option<unsafe extern "C" fn(*const clap_plugin, *const clap_istream) -> bool>,
}}

const CLAP_CORE_EVENT_SPACE_ID: u16 = 0;
const CLAP_EVENT_NOTE_ON_TYPE: u16 = 0;
const CLAP_EVENT_NOTE_OFF_TYPE: u16 = 1;
const CLAP_EVENT_NOTE_EXPRESSION_TYPE: u16 = 4;
const CLAP_EVENT_PARAM_VALUE_TYPE: u16 = 5;
const CLAP_EVENT_MIDI_TYPE: u16 = 10;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct clap_event_note {{
    pub header: clap_event_header,
    pub note_id: i32,
    pub port_index: i16,
    pub channel: i16,
    pub key: i16,
    pub velocity: f64,
}}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct clap_event_note_expression {{
    pub header: clap_event_header,
    pub expression_id: i32,
    pub note_id: i32,
    pub port_index: i16,
    pub channel: i16,
    pub key: i16,
    pub value: f64,
}}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct clap_event_midi {{
    pub header: clap_event_header,
    pub port_index: u16,
    pub data: [u8; 3],
}}

const CLAP_AUDIO_PORT_IS_MAIN: u32 = 1;
const CLAP_PARAM_IS_STEPPED: u32 = 1 << 0;
const CLAP_PARAM_IS_BYPASS: u32 = 1 << 4;
const CLAP_PARAM_IS_AUTOMATABLE: u32 = 1 << 5;
const CLAP_PARAM_IS_MODULATABLE: u32 = 1 << 10;
const CLAP_NOTE_DIALECT_MIDI: u32 = 1 << 1;

/// Default gain the fixture's process() applies (kept in sync with the
/// host crate's `CLAP_FIXTURE_GAIN`). Live value in `GAIN_BITS`.
const FIXTURE_GAIN: f32 = {gain:?};

/// Live Gain param value (f32 bits): defaults to FIXTURE_GAIN and follows
/// CLAP_EVENT_PARAM_VALUE in-events for param id 4096 (g12.023).
static GAIN_BITS: std::sync::atomic::AtomicU32 =
    std::sync::atomic::AtomicU32::new(f32::to_bits(FIXTURE_GAIN));
static NOTE_LEVEL_BITS: std::sync::atomic::AtomicU32 =
    std::sync::atomic::AtomicU32::new(f32::to_bits(0.0));

struct FeaturePtrs([*const c_char; 3]);
unsafe impl Sync for FeaturePtrs {{}}

/// Trivial offscreen gui state (g12.022 fixture): pure bookkeeping, no
/// window system touched, so automated tests can run headless.
static GUI_CREATED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
static GUI_VISIBLE: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
static GUI_PARENTED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
static GUI_WIDTH: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new({gui_initial_width});
static GUI_HEIGHT: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new({gui_initial_height});
static HOST: std::sync::atomic::AtomicPtr<clap_host> =
    std::sync::atomic::AtomicPtr::new(ptr::null_mut());
/// Set by gui_show: the next processed block pushes a Gain PARAM_VALUE
/// out-event (the "user tweaked the editor" stand-in, g12.024).
static PENDING_PARAM_OUT: std::sync::atomic::AtomicBool =
    std::sync::atomic::AtomicBool::new(false);

static GUI: clap_plugin_gui = clap_plugin_gui {{
    is_api_supported: Some(gui_is_api_supported),
    get_preferred_api: None,
    create: Some(gui_create),
    destroy: Some(gui_destroy),
    set_scale: Some(gui_set_scale),
    get_size: Some(gui_get_size),
    can_resize: Some(gui_can_resize),
    get_resize_hints: None,
    adjust_size: Some(gui_adjust_size),
    set_size: Some(gui_set_size),
    set_parent: Some(gui_set_parent),
    set_transient: None,
    suggest_title: None,
    show: Some(gui_show),
    hide: Some(gui_hide),
}};"#,
        gain = if instrument { 1.0 } else { CLAP_FIXTURE_GAIN },
        gui_initial_width = CLAP_FIXTURE_GUI_INITIAL_SIZE.0,
        gui_initial_height = CLAP_FIXTURE_GUI_INITIAL_SIZE.1,
    )
}
