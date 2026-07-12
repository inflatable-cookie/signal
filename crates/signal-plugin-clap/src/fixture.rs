//! Real compiled CLAP fixture for tests: source generator + rustc harness.
//!
//! The fixture is an actual CLAP cdylib (entry, factory, descriptor, audio
//! ports, note ports, params) compiled with `rustc` at test time, so
//! discovery and hosting tests exercise the genuine dlopen/FFI path. Its
//! `process()` is real too: a fixed-gain effect (output = input ×
//! [`CLAP_FIXTURE_GAIN`]), which gives hosting round-trip tests an audible,
//! exactly-checkable transform.
//!
//! Shared across crates (the sandbox broker's integration tests compile the
//! same fixture), hence public but hidden from the crate's documented API.

use std::{
    path::{Path, PathBuf},
    process::Command,
};

/// Linear gain the fixture's `process()` applies until a param write lands
/// (the Gain param's default; g12.023 makes the param live via
/// `CLAP_EVENT_PARAM_VALUE` in-events).
pub const CLAP_FIXTURE_GAIN: f32 = 0.5;

/// Param id of the fixture's live Gain parameter (plain range 0..1).
pub const CLAP_FIXTURE_GAIN_PARAM_ID: u32 = 4096;

/// Initial `clap.gui` content size the fixture reports from `get_size`.
pub const CLAP_FIXTURE_GUI_INITIAL_SIZE: (u32, u32) = (400, 300);

/// The resize the fixture's gui requests from the host on `show` (exercises
/// the host-callback path without any real window system).
pub const CLAP_FIXTURE_GUI_REQUESTED_SIZE: (u32, u32) = (500, 320);

/// The PLAIN Gain value the fixture's gui "tweaks" on `show`: pushed as a
/// `CLAP_EVENT_PARAM_VALUE` OUT-event at the top of the next processed
/// block (g12.024 plugin→host param sync proof; the Gain range is 0..1 so
/// plain == normalized).
pub const CLAP_FIXTURE_GUI_PARAM_OUT_VALUE: f64 = 0.75;

/// Returns `true` when a `rustc` binary is invocable (fixture tests skip
/// gracefully when it is not).
pub fn rustc_available() -> bool {
    Command::new("rustc")
        .arg("--version")
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

/// Compile the fixture cdylib into `directory`, returning the library path.
/// The library file is named after `plugin_name` with a `.clap` extension so
/// directory scans pick it up. Errors carry the rustc failure detail.
pub fn compile_clap_fixture(
    directory: &Path,
    plugin_type_id: &str,
    plugin_name: &str,
    midi_outputs: u16,
) -> Result<PathBuf, String> {
    std::fs::create_dir_all(directory)
        .map_err(|error| format!("fixture directory create failed: {error}"))?;
    let source_path = directory.join("fixture.rs");
    let library_path = directory.join(format!(
        "{}.clap",
        plugin_name.to_lowercase().replace(' ', "-")
    ));
    let source = clap_fixture_source(plugin_type_id, plugin_name, midi_outputs);
    std::fs::write(&source_path, source)
        .map_err(|error| format!("fixture source write failed: {error}"))?;
    let output = Command::new("rustc")
        .arg("--crate-type=cdylib")
        .arg("--edition=2021")
        .arg(&source_path)
        .arg("-o")
        .arg(&library_path)
        .output()
        .map_err(|error| format!("rustc invocation failed: {error}"))?;
    if !output.status.success() {
        return Err(format!(
            "clap fixture compilation failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    Ok(library_path)
}

/// Compile the same real CLAP fixture as a MIDI-input, stereo-output
/// instrument with no audio input bus. Note velocity drives its generated
/// constant signal; note-off returns it to silence.
pub fn compile_clap_instrument_fixture(
    directory: &Path,
    plugin_type_id: &str,
    plugin_name: &str,
) -> Result<PathBuf, String> {
    std::fs::create_dir_all(directory)
        .map_err(|error| format!("fixture directory create failed: {error}"))?;
    let source_path = directory.join("instrument-fixture.rs");
    let library_path = directory.join(format!(
        "{}.clap",
        plugin_name.to_lowercase().replace(' ', "-")
    ));
    std::fs::write(
        &source_path,
        clap_fixture_source_for_layout(plugin_type_id, plugin_name, 0, true),
    )
    .map_err(|error| format!("fixture source write failed: {error}"))?;
    let output = Command::new("rustc")
        .arg("--crate-type=cdylib")
        .arg("--edition=2021")
        .arg(&source_path)
        .arg("-o")
        .arg(&library_path)
        .output()
        .map_err(|error| format!("rustc invocation failed: {error}"))?;
    if !output.status.success() {
        return Err(format!(
            "clap fixture compilation failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    Ok(library_path)
}

/// Full Rust source of the fixture cdylib.
pub fn clap_fixture_source(plugin_type_id: &str, plugin_name: &str, midi_outputs: u16) -> String {
    clap_fixture_source_for_layout(plugin_type_id, plugin_name, midi_outputs, false)
}

fn clap_fixture_source_for_layout(
    plugin_type_id: &str,
    plugin_name: &str,
    midi_outputs: u16,
    instrument: bool,
) -> String {
    format!(
        r#"
use std::ffi::{{c_char, c_void, CStr}};
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
}};

static FACTORY_ID: &[u8] = b"clap.plugin-factory\0";
static AUDIO_PORTS_ID: &[u8] = b"clap.audio-ports\0";
static NOTE_PORTS_ID: &[u8] = b"clap.note-ports\0";
static PARAMS_ID: &[u8] = b"clap.params\0";
static GUI_ID: &[u8] = b"clap.gui\0";
static STATE_ID: &[u8] = b"clap.state\0";
static LATENCY_ID: &[u8] = b"clap.latency\0";
static TAIL_ID: &[u8] = b"clap.tail\0";
static FEATURE_AUDIO_EFFECT: &[u8] = b"audio-effect\0";
static FEATURE_INSTRUMENT: &[u8] = b"instrument\0";
static FEATURE_UTILITY: &[u8] = b"utility\0";
static FEATURES: FeaturePtrs = FeaturePtrs([
    {primary_feature_symbol}.as_ptr() as *const c_char,
    FEATURE_UTILITY.as_ptr() as *const c_char,
    ptr::null(),
]);
static PLUGIN_ID: &[u8] = concat!("{plugin_type_id}", "\0").as_bytes();
static PLUGIN_NAME: &[u8] = concat!("{plugin_name}", "\0").as_bytes();
static VENDOR: &[u8] = b"Signal\0";
static URL: &[u8] = b"https://signal.dev\0";
static VERSION: &[u8] = b"0.1.0\0";
static DESCRIPTION: &[u8] = b"Signal CLAP Fixture\0";

static DESCRIPTOR: clap_plugin_descriptor = clap_plugin_descriptor {{
    clap_version: clap_version {{ major: 1, minor: 0, revision: 0 }},
    id: PLUGIN_ID.as_ptr() as *const c_char,
    name: PLUGIN_NAME.as_ptr() as *const c_char,
    vendor: VENDOR.as_ptr() as *const c_char,
    url: URL.as_ptr() as *const c_char,
    manual_url: URL.as_ptr() as *const c_char,
    support_url: URL.as_ptr() as *const c_char,
    version: VERSION.as_ptr() as *const c_char,
    description: DESCRIPTION.as_ptr() as *const c_char,
    features: FEATURES.0.as_ptr(),
}};

static AUDIO_PORTS: clap_plugin_audio_ports = clap_plugin_audio_ports {{
    count: Some(audio_port_count),
    get: Some(audio_port_get),
}};
static LATENCY: clap_plugin_latency = clap_plugin_latency {{
    get: Some(latency_get),
}};

static NOTE_PORTS: clap_plugin_note_ports = clap_plugin_note_ports {{
    count: Some(note_port_count),
    get: Some(note_port_get),
}};

static PARAMS: clap_plugin_params = clap_plugin_params {{
    count: Some(param_count),
    get_info: Some(param_get_info),
    get_value: Some(param_get_value),
    value_to_text: None,
    text_to_value: None,
    flush: None,
}};

static STATE: clap_plugin_state = clap_plugin_state {{
    save: Some(state_save),
    load: Some(state_load),
}};

static PLUGIN: clap_plugin = clap_plugin {{
    desc: &DESCRIPTOR,
    plugin_data: ptr::null_mut(),
    init: Some(plugin_init),
    destroy: Some(plugin_destroy),
    activate: Some(plugin_activate),
    deactivate: Some(plugin_deactivate),
    start_processing: Some(plugin_start_processing),
    stop_processing: Some(plugin_stop_processing),
    reset: Some(plugin_reset),
    process: Some(plugin_process),
    get_extension: Some(plugin_get_extension),
    on_main_thread: None,
}};

static FACTORY: clap_plugin_factory = clap_plugin_factory {{
    get_plugin_count: Some(factory_get_plugin_count),
    get_plugin_descriptor: Some(factory_get_plugin_descriptor),
    create_plugin: Some(factory_create_plugin),
}};

#[unsafe(no_mangle)]
pub static clap_entry: clap_plugin_entry = clap_plugin_entry {{
    clap_version: clap_version {{ major: 1, minor: 0, revision: 0 }},
    init: Some(entry_init),
    deinit: Some(entry_deinit),
    get_factory: Some(entry_get_factory),
}};

unsafe extern "C" fn entry_init(_plugin_path: *const c_char) -> bool {{ true }}
unsafe extern "C" fn entry_deinit() {{}}

unsafe extern "C" fn entry_get_factory(factory_id: *const c_char) -> *const c_void {{
    let requested = CStr::from_ptr(factory_id).to_bytes_with_nul();
    if requested == FACTORY_ID {{
        (&FACTORY as *const clap_plugin_factory).cast()
    }} else {{
        ptr::null()
    }}
}}

unsafe extern "C" fn factory_get_plugin_count(_factory: *const clap_plugin_factory) -> u32 {{ 1 }}
unsafe extern "C" fn factory_get_plugin_descriptor(
    _factory: *const clap_plugin_factory,
    index: u32,
) -> *const clap_plugin_descriptor {{
    if index == 0 {{ &DESCRIPTOR }} else {{ ptr::null() }}
}}

unsafe extern "C" fn factory_create_plugin(
    _factory: *const clap_plugin_factory,
    host: *const c_void,
    _plugin_id: *const c_char,
) -> *const clap_plugin {{
    HOST.store(host as *mut clap_host, std::sync::atomic::Ordering::SeqCst);
    &PLUGIN
}}

unsafe extern "C" fn plugin_init(_plugin: *const clap_plugin) -> bool {{ true }}
unsafe extern "C" fn plugin_destroy(_plugin: *const clap_plugin) {{}}
unsafe extern "C" fn plugin_activate(_plugin: *const clap_plugin, _sample_rate: f64, _min: u32, _max: u32) -> bool {{ true }}
unsafe extern "C" fn plugin_deactivate(_plugin: *const clap_plugin) {{}}
unsafe extern "C" fn plugin_start_processing(_plugin: *const clap_plugin) -> bool {{ true }}
unsafe extern "C" fn plugin_stop_processing(_plugin: *const clap_plugin) {{}}
unsafe extern "C" fn plugin_reset(_plugin: *const clap_plugin) {{}}

/// Cap on per-block gain steps gathered from note/MIDI in-events (the
/// note/CC delivery proof; more than enough for the tests).
const GAIN_STEP_CAPACITY: usize = 64;

/// Apply pending in-events before the block renders. PARAM_VALUE events
/// for the Gain param (id 4096) keep their block-boundary semantics
/// (g12.023: stored immediately). Note and MIDI CC7 events become
/// SAMPLE-OFFSET voice-level steps for instruments (gain steps for effects)
/// so hosts can assert both the decoded bytes and
/// the intra-block offsets from the audio output alone:
///   NOTE_ON  → gain = velocity from the event's time offset
///   NOTE_OFF → gain = 0.0 from the event's time offset
///   MIDI 0xB0 cc=7 → gain = data2 / 127 from the event's time offset
/// Returns `(time, value, voice_level)` steps in delivery order.
unsafe fn apply_param_events(
    in_events: *const c_void,
    steps: &mut [(u32, f32, bool); GAIN_STEP_CAPACITY],
) -> usize {{
    if in_events.is_null() {{
        return 0;
    }}
    let list = &*(in_events as *const clap_input_events);
    let (Some(size), Some(get)) = (list.size, list.get) else {{
        return 0;
    }};
    let mut step_count = 0usize;
    let count = size(list as *const clap_input_events);
    for index in 0..count {{
        let header = get(list as *const clap_input_events, index);
        if header.is_null() {{
            continue;
        }}
        if (*header).space_id != CLAP_CORE_EVENT_SPACE_ID {{
            continue;
        }}
        match (*header).type_ {{
            CLAP_EVENT_PARAM_VALUE_TYPE => {{
                let event = &*(header as *const clap_event_param_value);
                if event.param_id == 4096 {{
                    GAIN_BITS.store(
                        (event.value as f32).to_bits(),
                        std::sync::atomic::Ordering::SeqCst,
                    );
                }}
            }}
            CLAP_EVENT_NOTE_ON_TYPE | CLAP_EVENT_NOTE_OFF_TYPE => {{
                let event = &*(header as *const clap_event_note);
                if step_count < GAIN_STEP_CAPACITY {{
                    let gain = if (*header).type_ == CLAP_EVENT_NOTE_ON_TYPE {{
                        event.velocity as f32
                    }} else {{
                        0.0
                    }};
                    steps[step_count] = ((*header).time, gain, {instrument});
                    step_count += 1;
                }}
            }}
            CLAP_EVENT_NOTE_EXPRESSION_TYPE => {{
                let event = &*(header as *const clap_event_note_expression);
                if step_count < GAIN_STEP_CAPACITY {{
                    steps[step_count] = ((*header).time, event.value as f32, {instrument});
                    step_count += 1;
                }}
            }}
            CLAP_EVENT_MIDI_TYPE => {{
                let event = &*(header as *const clap_event_midi);
                if event.data[0] & 0xF0 == 0xB0
                    && event.data[1] == 7
                    && step_count < GAIN_STEP_CAPACITY
                {{
                    steps[step_count] = ((*header).time, f32::from(event.data[2]) / 127.0, {instrument});
                    step_count += 1;
                }}
            }}
            _ => {{}}
        }}
    }}
    step_count
}}

/// Real audio processing: output = input × the LIVE Gain param on every
/// channel of the main port pair (in-events applied first, block-boundary).
/// Returns CLAP_PROCESS_CONTINUE (1) on success.
unsafe extern "C" fn plugin_process(
    _plugin: *const clap_plugin,
    process: *const clap_process,
) -> i32 {{
    if process.is_null() {{
        return 0;
    }}
    let process = &*process;
    let mut gain_steps = [(0u32, 0f32, false); GAIN_STEP_CAPACITY];
    let step_count = apply_param_events(process.in_events, &mut gain_steps);
    if PENDING_PARAM_OUT.swap(false, std::sync::atomic::Ordering::SeqCst)
        && !process.out_events.is_null()
    {{
        let out_events = &*(process.out_events as *const clap_output_events);
        if let Some(try_push) = out_events.try_push {{
            let event = clap_event_param_value {{
                header: clap_event_header {{
                    size: std::mem::size_of::<clap_event_param_value>() as u32,
                    time: 0,
                    space_id: CLAP_CORE_EVENT_SPACE_ID,
                    type_: CLAP_EVENT_PARAM_VALUE_TYPE,
                    flags: 0,
                }},
                param_id: 4096,
                cookie: ptr::null_mut(),
                note_id: -1,
                port_index: -1,
                channel: -1,
                key: -1,
                value: {gui_param_out_value}f64,
            }};
            GAIN_BITS.store(
                ({gui_param_out_value}f32).to_bits(),
                std::sync::atomic::Ordering::SeqCst,
            );
            let _ = try_push(
                out_events as *const clap_output_events,
                &event.header as *const clap_event_header,
            );
        }}
    }}
    if process.audio_outputs_count < 1 || process.audio_outputs.is_null() {{
        return 0;
    }}
    let output = &*process.audio_outputs;
    if output.data32.is_null() {{
        return 0;
    }}
    let input = if {instrument} {{
        None
    }} else {{
        if process.audio_inputs_count < 1 || process.audio_inputs.is_null() {{ return 0; }}
        let input = &*process.audio_inputs;
        if input.data32.is_null() {{ return 0; }}
        Some(input)
    }};
    let frames = process.frames_count as usize;
    let channels = input
        .map(|input| input.channel_count.min(output.channel_count))
        .unwrap_or(output.channel_count) as usize;
    for channel in 0..channels {{
        let source = input.map(|input| *input.data32.add(channel));
        let dest = *output.data32.add(channel);
        if source.is_some_and(|source| source.is_null()) || dest.is_null() {{
            return 0;
        }}
        // Gain and instrument voice level are independent: parameter writes
        // scale held notes instead of being overwritten by note events.
        let mut gain = f32::from_bits(GAIN_BITS.load(std::sync::atomic::Ordering::SeqCst));
        let mut note_level = f32::from_bits(
            NOTE_LEVEL_BITS.load(std::sync::atomic::Ordering::SeqCst),
        );
        let mut next_step = 0usize;
        for frame in 0..frames {{
            while next_step < step_count && gain_steps[next_step].0 as usize <= frame {{
                if gain_steps[next_step].2 {{
                    note_level = gain_steps[next_step].1;
                }} else {{
                    gain = gain_steps[next_step].1;
                }}
                next_step += 1;
            }}
            *dest.add(frame) = match source {{
                Some(source) => *source.add(frame) * gain,
                None => note_level * gain,
            }};
        }}
    }}
    for step in &gain_steps[..step_count] {{
        if step.2 {{
            NOTE_LEVEL_BITS.store(step.1.to_bits(), std::sync::atomic::Ordering::SeqCst);
        }} else {{
            GAIN_BITS.store(step.1.to_bits(), std::sync::atomic::Ordering::SeqCst);
        }}
    }}
    1
}}

unsafe extern "C" fn plugin_get_extension(
    _plugin: *const clap_plugin,
    extension_id: *const c_char,
) -> *const c_void {{
    let requested = CStr::from_ptr(extension_id).to_bytes_with_nul();
    if requested == AUDIO_PORTS_ID {{
        (&AUDIO_PORTS as *const clap_plugin_audio_ports).cast()
    }} else if requested == NOTE_PORTS_ID {{
        (&NOTE_PORTS as *const clap_plugin_note_ports).cast()
    }} else if requested == PARAMS_ID {{
        (&PARAMS as *const clap_plugin_params).cast()
    }} else if requested == GUI_ID {{
        (&GUI as *const clap_plugin_gui).cast()
    }} else if requested == LATENCY_ID {{
        (&LATENCY as *const clap_plugin_latency).cast()
    }} else if requested == STATE_ID {{
        (&STATE as *const clap_plugin_state).cast()
    }} else if requested == TAIL_ID {{
        1usize as *const c_void
    }} else {{
        ptr::null()
    }}
}}

// ── clap.state ─────────────────────────────────────────────────────────────

unsafe extern "C" fn state_save(
    _plugin: *const clap_plugin,
    stream: *const clap_ostream,
) -> bool {{
    if stream.is_null() {{
        return false;
    }}
    let Some(write) = (*stream).write else {{ return false }};
    let mut state = [0u8; 8];
    state[..4].copy_from_slice(&GAIN_BITS.load(std::sync::atomic::Ordering::SeqCst).to_le_bytes());
    state[4..].copy_from_slice(&NOTE_LEVEL_BITS.load(std::sync::atomic::Ordering::SeqCst).to_le_bytes());
    let mut offset = 0usize;
    while offset < state.len() {{
        let written = write(
            stream,
            state.as_ptr().add(offset).cast(),
            (state.len() - offset) as u64,
        );
        if written <= 0 || written as usize > state.len() - offset {{
            return false;
        }}
        offset += written as usize;
    }}
    true
}}

unsafe extern "C" fn state_load(
    _plugin: *const clap_plugin,
    stream: *const clap_istream,
) -> bool {{
    if stream.is_null() {{
        return false;
    }}
    let Some(read) = (*stream).read else {{ return false }};
    let mut state = [0u8; 8];
    let mut offset = 0usize;
    while offset < state.len() {{
        let count = read(
            stream,
            state.as_mut_ptr().add(offset).cast(),
            (state.len() - offset) as u64,
        );
        if count <= 0 || count as usize > state.len() - offset {{
            return false;
        }}
        offset += count as usize;
    }}
    GAIN_BITS.store(
        u32::from_le_bytes(state[..4].try_into().unwrap()),
        std::sync::atomic::Ordering::SeqCst,
    );
    NOTE_LEVEL_BITS.store(
        u32::from_le_bytes(state[4..].try_into().unwrap()),
        std::sync::atomic::Ordering::SeqCst,
    );
    true
}}

// ── clap.gui (offscreen bookkeeping only) ──────────────────────────────────

unsafe extern "C" fn gui_is_api_supported(
    _plugin: *const clap_plugin,
    _api: *const c_char,
    is_floating: bool,
) -> bool {{
    // Embedded on every window API (nothing is dereferenced), floating
    // unsupported: matches the phase-1 host path.
    !is_floating
}}

unsafe extern "C" fn gui_create(
    _plugin: *const clap_plugin,
    _api: *const c_char,
    is_floating: bool,
) -> bool {{
    if is_floating {{
        return false;
    }}
    GUI_WIDTH.store({gui_initial_width}, std::sync::atomic::Ordering::SeqCst);
    GUI_HEIGHT.store({gui_initial_height}, std::sync::atomic::Ordering::SeqCst);
    GUI_CREATED.store(true, std::sync::atomic::Ordering::SeqCst);
    true
}}

unsafe extern "C" fn gui_destroy(_plugin: *const clap_plugin) {{
    GUI_CREATED.store(false, std::sync::atomic::Ordering::SeqCst);
    GUI_VISIBLE.store(false, std::sync::atomic::Ordering::SeqCst);
    GUI_PARENTED.store(false, std::sync::atomic::Ordering::SeqCst);
}}

unsafe extern "C" fn gui_set_scale(_plugin: *const clap_plugin, _scale: f64) -> bool {{
    true
}}

unsafe extern "C" fn gui_get_size(
    _plugin: *const clap_plugin,
    width: *mut u32,
    height: *mut u32,
) -> bool {{
    if !GUI_CREATED.load(std::sync::atomic::Ordering::SeqCst) || width.is_null() || height.is_null()
    {{
        return false;
    }}
    *width = GUI_WIDTH.load(std::sync::atomic::Ordering::SeqCst);
    *height = GUI_HEIGHT.load(std::sync::atomic::Ordering::SeqCst);
    true
}}

unsafe extern "C" fn gui_can_resize(_plugin: *const clap_plugin) -> bool {{
    true
}}

unsafe extern "C" fn gui_adjust_size(
    _plugin: *const clap_plugin,
    width: *mut u32,
    height: *mut u32,
) -> bool {{
    !width.is_null() && !height.is_null()
}}

unsafe extern "C" fn gui_set_size(_plugin: *const clap_plugin, width: u32, height: u32) -> bool {{
    if !GUI_CREATED.load(std::sync::atomic::Ordering::SeqCst) {{
        return false;
    }}
    GUI_WIDTH.store(width, std::sync::atomic::Ordering::SeqCst);
    GUI_HEIGHT.store(height, std::sync::atomic::Ordering::SeqCst);
    true
}}

unsafe extern "C" fn gui_set_parent(
    _plugin: *const clap_plugin,
    window: *const clap_window,
) -> bool {{
    // The parent handle is recorded, never dereferenced (offscreen test
    // plugin): any non-null handle parents successfully.
    if window.is_null() || (*window).specific.is_null() {{
        return false;
    }}
    GUI_PARENTED.store(true, std::sync::atomic::Ordering::SeqCst);
    true
}}

unsafe extern "C" fn gui_show(_plugin: *const clap_plugin) -> bool {{
    if !GUI_CREATED.load(std::sync::atomic::Ordering::SeqCst) {{
        return false;
    }}
    GUI_VISIBLE.store(true, std::sync::atomic::Ordering::SeqCst);
    // Stand-in editor tweak: the next processed block pushes a Gain
    // PARAM_VALUE out-event for the plugin→host sync proof (g12.024).
    PENDING_PARAM_OUT.store(true, std::sync::atomic::Ordering::SeqCst);
    let host = HOST.load(std::sync::atomic::Ordering::SeqCst);
    if !host.is_null() {{
        if let Some(get_extension) = (*host).get_extension {{
            let extension = get_extension(host, STATE_ID.as_ptr().cast());
            if !extension.is_null() {{
                if let Some(mark_dirty) = (*(extension as *const clap_host_state)).mark_dirty {{
                    mark_dirty(host);
                }}
            }}
        }}
    }}
    // Exercise the host-callback path: ask the host for a resize.
    let host = HOST.load(std::sync::atomic::Ordering::SeqCst);
    if !host.is_null() {{
        if let Some(get_extension) = (*host).get_extension {{
            let extension = get_extension(host, GUI_ID.as_ptr() as *const c_char);
            if !extension.is_null() {{
                let host_gui = extension as *const clap_host_gui;
                if let Some(request_resize) = (*host_gui).request_resize {{
                    let _ = request_resize(host, {gui_request_width}, {gui_request_height});
                }}
            }}
            // Exercise the host clap.params wiring too (g12.024): an
            // editor tweak conventionally asks the host for a flush.
            let params_extension = get_extension(host, PARAMS_ID.as_ptr() as *const c_char);
            if !params_extension.is_null() {{
                let host_params = params_extension as *const clap_host_params;
                if let Some(request_flush) = (*host_params).request_flush {{
                    request_flush(host);
                }}
            }}
        }}
    }}
    true
}}

unsafe extern "C" fn gui_hide(_plugin: *const clap_plugin) -> bool {{
    GUI_VISIBLE.store(false, std::sync::atomic::Ordering::SeqCst);
    true
}}

unsafe extern "C" fn audio_port_count(_plugin: *const clap_plugin, is_input: bool) -> u32 {{
    if is_input {{ {audio_input_count} }} else {{ 1 }}
}}
unsafe extern "C" fn latency_get(_plugin: *const clap_plugin) -> u32 {{ 0 }}
unsafe extern "C" fn audio_port_get(
    _plugin: *const clap_plugin,
    index: u32,
    is_input: bool,
    info: *mut clap_audio_port_info,
) -> bool {{
    if index != 0 {{ return false; }}
    let name: &[u8] = if is_input {{ b"Main Input\0".as_slice() }} else {{ b"Main Output\0".as_slice() }};
    let channel_count = 2;
    let mut port = clap_audio_port_info {{
        id: if is_input {{ 1 }} else {{ 2 }},
        name: [0; 256],
        flags: CLAP_AUDIO_PORT_IS_MAIN,
        channel_count,
        port_type: ptr::null(),
        in_place_pair: u32::MAX,
    }};
    for (slot, value) in port.name.iter_mut().zip(name.iter().copied()) {{
        *slot = value as c_char;
    }}
    *info = port;
    true
}}

unsafe extern "C" fn note_port_count(_plugin: *const clap_plugin, is_input: bool) -> u32 {{
    if is_input {{ 1 }} else {{ {midi_outputs} }}
}}

unsafe extern "C" fn note_port_get(
    _plugin: *const clap_plugin,
    index: u32,
    is_input: bool,
    info: *mut clap_note_port_info,
) -> bool {{
    if index != 0 {{ return false; }}
    let name: &[u8] = if is_input {{ b"MIDI In\0".as_slice() }} else {{ b"MIDI Out\0".as_slice() }};
    let mut port = clap_note_port_info {{
        id: if is_input {{ 11 }} else {{ 12 }},
        supported_dialects: CLAP_NOTE_DIALECT_MIDI,
        preferred_dialect: CLAP_NOTE_DIALECT_MIDI,
        name: [0; 256],
    }};
    for (slot, value) in port.name.iter_mut().zip(name.iter().copied()) {{
        *slot = value as c_char;
    }}
    *info = port;
    true
}}

unsafe extern "C" fn param_count(_plugin: *const clap_plugin) -> u32 {{ 2 }}
unsafe extern "C" fn param_get_info(
    _plugin: *const clap_plugin,
    index: u32,
    info: *mut clap_param_info,
) -> bool {{
    let (id, name, flags, default_value) = match index {{
        0 => (4096u32, b"Gain\0".as_slice(), CLAP_PARAM_IS_AUTOMATABLE | CLAP_PARAM_IS_MODULATABLE, FIXTURE_GAIN as f64),
        1 => (0u32, b"Bypass\0".as_slice(), CLAP_PARAM_IS_BYPASS | CLAP_PARAM_IS_AUTOMATABLE | CLAP_PARAM_IS_STEPPED, 0.0f64),
        _ => return false,
    }};
    let mut param = clap_param_info {{
        id,
        flags,
        cookie: ptr::null_mut(),
        name: [0; 256],
        module: [0; 1024],
        min_value: 0.0,
        max_value: 1.0,
        default_value,
    }};
    for (slot, value) in param.name.iter_mut().zip(name.iter().copied()) {{
        *slot = value as c_char;
    }}
    *info = param;
    true
}}

unsafe extern "C" fn param_get_value(
    _plugin: *const clap_plugin,
    param_id: u32,
    out_value: *mut f64,
) -> bool {{
    if out_value.is_null() {{ return false; }}
    *out_value = if param_id == 4096 {{
        f32::from_bits(GAIN_BITS.load(std::sync::atomic::Ordering::SeqCst)) as f64
    }} else {{
        0.0
    }};
    true
}}
"#,
        gain = if instrument { 1.0 } else { CLAP_FIXTURE_GAIN },
        plugin_type_id = plugin_type_id,
        plugin_name = plugin_name,
        midi_outputs = midi_outputs,
        instrument = instrument,
        audio_input_count = if instrument { 0 } else { 1 },
        primary_feature_symbol = if instrument {
            "FEATURE_INSTRUMENT"
        } else {
            "FEATURE_AUDIO_EFFECT"
        },
        gui_initial_width = CLAP_FIXTURE_GUI_INITIAL_SIZE.0,
        gui_initial_height = CLAP_FIXTURE_GUI_INITIAL_SIZE.1,
        gui_request_width = CLAP_FIXTURE_GUI_REQUESTED_SIZE.0,
        gui_request_height = CLAP_FIXTURE_GUI_REQUESTED_SIZE.1,
        gui_param_out_value = CLAP_FIXTURE_GUI_PARAM_OUT_VALUE,
    )
}
