// Tests for signal-plugin-clap
#[allow(clippy::module_inception)]
mod tests {
    use crate::{ClapHostExtension, ClapPluginHostAdapter};
    use signal_plugin::PluginFormat;
    use std::{
        fs,
        path::PathBuf,
        process,
        process::Command,
        time::{SystemTime, UNIX_EPOCH},
    };

    fn test_broker_root(name: &str) -> PathBuf {
        // Nanosecond timestamps can collide across concurrently-starting
        // tests (clock granularity); the counter keeps roots unique so
        // fixture writes never interleave.
        static SEQUENCE: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
        let sequence = SEQUENCE.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        std::env::temp_dir().join(format!(
            "signal-plugin-clap-tests-{}-{name}-{timestamp}-{sequence}",
            process::id()
        ))
    }

    pub(super) struct TempClapScanRoot {
        path: PathBuf,
    }

    impl TempClapScanRoot {
        pub(super) fn root(&self) -> String {
            self.path.display().to_string()
        }
    }

    impl Drop for TempClapScanRoot {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.path);
        }
    }

    pub(super) fn temp_real_clap_scan_root(
        plugin_type_id: &str,
        plugin_name: &str,
        midi_outputs: u16,
    ) -> TempClapScanRoot {
        let root = test_broker_root("clap-real-scan");
        fs::create_dir_all(&root).expect("real clap scan root should be created");
        let source_path = root.join("fixture.rs");
        let library_path = root.join(format!(
            "{}.clap",
            plugin_name.to_lowercase().replace(' ', "-")
        ));
        let source = clap_fixture_source(plugin_type_id, plugin_name, midi_outputs);
        fs::write(&source_path, source).expect("clap fixture source should be written");
        let output = Command::new("rustc")
            .arg("--crate-type=cdylib")
            .arg("--edition=2021")
            .arg(&source_path)
            .arg("-o")
            .arg(&library_path)
            .output()
            .expect("rustc should build the clap fixture");
        assert!(
            output.status.success(),
            "clap fixture compilation should succeed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        TempClapScanRoot { path: root }
    }

    fn clap_fixture_source(plugin_type_id: &str, plugin_name: &str, midi_outputs: u16) -> String {
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
    pub process: Option<unsafe extern "C" fn(*const clap_plugin, *const c_void) -> i32>,
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

const CLAP_AUDIO_PORT_IS_MAIN: u32 = 1;
const CLAP_PARAM_IS_STEPPED: u32 = 1 << 0;
const CLAP_PARAM_IS_BYPASS: u32 = 1 << 4;
const CLAP_PARAM_IS_AUTOMATABLE: u32 = 1 << 5;
const CLAP_PARAM_IS_MODULATABLE: u32 = 1 << 10;
const CLAP_NOTE_DIALECT_MIDI: u32 = 1 << 1;

struct FeaturePtrs([*const c_char; 3]);
unsafe impl Sync for FeaturePtrs {{}}

static FACTORY_ID: &[u8] = b"clap.plugin-factory\0";
static AUDIO_PORTS_ID: &[u8] = b"clap.audio-ports\0";
static NOTE_PORTS_ID: &[u8] = b"clap.note-ports\0";
static PARAMS_ID: &[u8] = b"clap.params\0";
static STATE_ID: &[u8] = b"clap.state\0";
static LATENCY_ID: &[u8] = b"clap.latency\0";
static TAIL_ID: &[u8] = b"clap.tail\0";
static FEATURE_AUDIO_EFFECT: &[u8] = b"audio-effect\0";
static FEATURE_UTILITY: &[u8] = b"utility\0";
static FEATURES: FeaturePtrs = FeaturePtrs([
    FEATURE_AUDIO_EFFECT.as_ptr() as *const c_char,
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
    process: None,
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
    _host: *const c_void,
    _plugin_id: *const c_char,
) -> *const clap_plugin {{
    &PLUGIN
}}

unsafe extern "C" fn plugin_init(_plugin: *const clap_plugin) -> bool {{ true }}
unsafe extern "C" fn plugin_destroy(_plugin: *const clap_plugin) {{}}
unsafe extern "C" fn plugin_activate(_plugin: *const clap_plugin, _sample_rate: f64, _min: u32, _max: u32) -> bool {{ true }}
unsafe extern "C" fn plugin_deactivate(_plugin: *const clap_plugin) {{}}
unsafe extern "C" fn plugin_start_processing(_plugin: *const clap_plugin) -> bool {{ true }}
unsafe extern "C" fn plugin_stop_processing(_plugin: *const clap_plugin) {{}}
unsafe extern "C" fn plugin_reset(_plugin: *const clap_plugin) {{}}

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
    }} else if requested == STATE_ID || requested == LATENCY_ID || requested == TAIL_ID {{
        1usize as *const c_void
    }} else {{
        ptr::null()
    }}
}}

unsafe extern "C" fn audio_port_count(_plugin: *const clap_plugin, _is_input: bool) -> u32 {{ 1 }}
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
        0 => (4096u32, b"Gain\0".as_slice(), CLAP_PARAM_IS_AUTOMATABLE | CLAP_PARAM_IS_MODULATABLE, 0.5f64),
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
    *out_value = if param_id == 4096 {{ 0.5 }} else {{ 0.0 }};
    true
}}
"#
        )
    }

    mod adapter;
}
