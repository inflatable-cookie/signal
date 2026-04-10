#![allow(dead_code)]

use std::{
    fs,
    path::PathBuf,
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};

pub(crate) struct TempPluginScanRoot {
    path: PathBuf,
}

impl TempPluginScanRoot {
    pub(crate) fn root(&self) -> String {
        self.path.display().to_string()
    }
}

impl Drop for TempPluginScanRoot {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

pub(crate) fn temp_public_server_vst3_scan_root() -> TempPluginScanRoot {
    let root = temp_scan_root("vst3");
    write_vst3_bundle(&root, "Signal Linux Synth.vst3", "plugin:vst3:linux-synth");
    write_vst3_bundle(
        &root,
        "Signal Multi Output Instrument.vst3",
        "plugin:vst3:multiout-instrument",
    );
    write_vst3_bundle(&root, "Signal Utility.vst3", "plugin:vst3:utility");
    write_vst3_bundle(&root, "Signal Bus FX.vst3", "plugin:vst3:bus-fx");
    TempPluginScanRoot { path: root }
}

pub(crate) fn temp_public_server_clap_scan_root() -> TempPluginScanRoot {
    let root = temp_scan_root("clap");
    write_clap_fixture_library(
        &root,
        "signal-server-main.clap",
        "plugin:clap:server",
        "Signal Server CLAP Plugin",
        0,
    );
    write_clap_fixture_library(
        &root,
        "signal-server-sandbox.clap",
        "plugin:clap:sandbox",
        "Signal Sandbox CLAP Plugin",
        1,
    );
    TempPluginScanRoot { path: root }
}

pub(crate) fn temp_public_server_au_scan_root() -> TempPluginScanRoot {
    let root = temp_scan_root("au");
    write_au_bundle(&root, "Signal Instrument.component", "plugin:au:instrument");
    write_au_bundle(
        &root,
        "Signal Multi Output Instrument.component",
        "plugin:au:multiout-instrument",
    );
    write_au_bundle(&root, "Signal Utility.component", "plugin:au:utility");
    write_au_bundle(&root, "Signal Bus FX.component", "plugin:au:bus-fx");
    TempPluginScanRoot { path: root }
}

pub(crate) fn temp_public_server_lv2_scan_root() -> TempPluginScanRoot {
    let root = temp_scan_root("lv2");
    write_lv2_bundle(&root, "Signal Linux Synth.lv2", "plugin:lv2:linux-synth");
    write_lv2_bundle(
        &root,
        "Signal Multi Output Instrument.lv2",
        "plugin:lv2:multiout-instrument",
    );
    write_lv2_bundle(&root, "Signal Utility.lv2", "plugin:lv2:utility");
    write_lv2_bundle(&root, "Signal Bus FX.lv2", "plugin:lv2:bus-fx");
    write_manifest_bundle(
        &root,
        "Broken Manifest.lv2",
        "@prefix signal: <https://signal.dev/ns/lv2#> .\nsignal:plugin_type_id \"plugin:lv2:broken-public\"\n",
    );
    write_manifest_bundle(
        &root,
        "Unsupported Feature.lv2",
        "@prefix lv2: <http://lv2plug.in/ns/lv2core#> .\n@prefix signal: <https://signal.dev/ns/lv2#> .\nsignal:plugin_type_id \"plugin:lv2:unsupported-public\" .\nsignal:plugin_uri \"https://signal.dev/plugins/lv2/unsupported-public\" .\nsignal:vendor \"Signal\" .\nsignal:name \"Unsupported Public LV2 Plugin\" .\nsignal:version \"0.1.0\" .\nsignal:audio_inputs \"2\" .\nsignal:audio_outputs \"2\" .\nsignal:midi_inputs \"0\" .\nsignal:midi_outputs \"0\" .\nsignal:required_feature \"http://lv2plug.in/ns/ext/atom#sequence\" .\nsignal:feature \"AudioEffect\" .\n",
    );
    write_manifest_bundle(
        &root,
        "Worker Unavailable.lv2",
        "@prefix lv2: <http://lv2plug.in/ns/lv2core#> .\n@prefix signal: <https://signal.dev/ns/lv2#> .\nsignal:plugin_type_id \"plugin:lv2:worker-unavailable-public\" .\nsignal:plugin_uri \"https://signal.dev/plugins/lv2/worker-unavailable-public\" .\nsignal:vendor \"Signal\" .\nsignal:name \"Worker Unavailable Public LV2 Plugin\" .\nsignal:version \"0.1.0\" .\nsignal:audio_inputs \"2\" .\nsignal:audio_outputs \"2\" .\nsignal:midi_inputs \"1\" .\nsignal:midi_outputs \"0\" .\nsignal:required_feature \"http://lv2plug.in/ns/ext/urid#map\" .\nsignal:required_feature \"http://lv2plug.in/ns/ext/worker#schedule\" .\nsignal:supported_extension \"http://lv2plug.in/ns/ext/patch#Message\" .\nsignal:prepare_fault \"worker_unavailable\" .\nsignal:feature \"AudioEffect\" .\n",
    );
    TempPluginScanRoot { path: root }
}

fn temp_scan_root(label: &str) -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock should be after epoch")
        .as_nanos();
    let root =
        std::env::temp_dir().join(format!("signal-public-host-server-{label}-scan-{unique}"));
    fs::create_dir_all(&root).expect("public server scan root should be created");
    root
}

fn write_vst3_bundle(root: &PathBuf, bundle: &str, plugin_type_id: &str) {
    let bundle_root = root.join(bundle);
    fs::create_dir_all(bundle_root.join("Contents").join("Resources"))
        .expect("public server vst3 resources should be created");
    fs::write(
        bundle_root.join("Contents").join("Info.plist"),
        vst3_info_plist_contents(vst3_metadata_contents(plugin_type_id), &bundle_root),
    )
    .expect("public server vst3 info plist should be written");
    fs::write(
        bundle_root
            .join("Contents")
            .join("Resources")
            .join("moduleinfo.json"),
        vst3_moduleinfo_contents(vst3_metadata_contents(plugin_type_id)),
    )
    .expect("public server vst3 moduleinfo should be written");
}

fn write_clap_fixture_library(
    root: &PathBuf,
    file_name: &str,
    plugin_type_id: &str,
    plugin_name: &str,
    midi_outputs: u16,
) {
    let source_path = root.join(format!("{file_name}.rs"));
    let library_path = root.join(file_name);
    fs::write(
        &source_path,
        clap_fixture_source(plugin_type_id, plugin_name, midi_outputs),
    )
    .expect("public server clap fixture source should be written");
    let output = Command::new("rustc")
        .arg("--crate-type=cdylib")
        .arg("--edition=2021")
        .arg(&source_path)
        .arg("-o")
        .arg(&library_path)
        .output()
        .expect("rustc should build public server clap fixture");
    assert!(
        output.status.success(),
        "public server clap fixture compilation should succeed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

fn clap_fixture_source(plugin_type_id: &str, plugin_name: &str, midi_outputs: u16) -> String {
    format!(
        r#"
use std::ffi::{{c_char, c_void, CStr}};
use std::ptr;
#[repr(C)] #[derive(Copy, Clone)] pub struct clap_version {{ pub major: u32, pub minor: u32, pub revision: u32 }}
#[repr(C)] #[derive(Copy, Clone)] pub struct clap_plugin_entry {{ pub clap_version: clap_version, pub init: Option<unsafe extern "C" fn(*const c_char) -> bool>, pub deinit: Option<unsafe extern "C" fn()>, pub get_factory: Option<unsafe extern "C" fn(*const c_char) -> *const c_void> }}
#[repr(C)] #[derive(Copy, Clone)] pub struct clap_plugin_descriptor {{ pub clap_version: clap_version, pub id: *const c_char, pub name: *const c_char, pub vendor: *const c_char, pub url: *const c_char, pub manual_url: *const c_char, pub support_url: *const c_char, pub version: *const c_char, pub description: *const c_char, pub features: *const *const c_char }}
unsafe impl Sync for clap_plugin_descriptor {{}}
#[repr(C)] #[derive(Copy, Clone)] pub struct clap_plugin {{ pub desc: *const clap_plugin_descriptor, pub plugin_data: *mut c_void, pub init: Option<unsafe extern "C" fn(*const clap_plugin) -> bool>, pub destroy: Option<unsafe extern "C" fn(*const clap_plugin)>, pub activate: Option<unsafe extern "C" fn(*const clap_plugin, f64, u32, u32) -> bool>, pub deactivate: Option<unsafe extern "C" fn(*const clap_plugin)>, pub start_processing: Option<unsafe extern "C" fn(*const clap_plugin) -> bool>, pub stop_processing: Option<unsafe extern "C" fn(*const clap_plugin)>, pub reset: Option<unsafe extern "C" fn(*const clap_plugin)>, pub process: Option<unsafe extern "C" fn(*const clap_plugin, *const c_void) -> i32>, pub get_extension: Option<unsafe extern "C" fn(*const clap_plugin, *const c_char) -> *const c_void>, pub on_main_thread: Option<unsafe extern "C" fn(*const clap_plugin)> }}
unsafe impl Sync for clap_plugin {{}}
#[repr(C)] #[derive(Copy, Clone)] pub struct clap_plugin_factory {{ pub get_plugin_count: Option<unsafe extern "C" fn(*const clap_plugin_factory) -> u32>, pub get_plugin_descriptor: Option<unsafe extern "C" fn(*const clap_plugin_factory, u32) -> *const clap_plugin_descriptor>, pub create_plugin: Option<unsafe extern "C" fn(*const clap_plugin_factory, *const c_void, *const c_char) -> *const clap_plugin> }}
#[repr(C)] #[derive(Copy, Clone)] pub struct clap_audio_port_info {{ pub id: u32, pub name: [c_char; 256], pub flags: u32, pub channel_count: u32, pub port_type: *const c_char, pub in_place_pair: u32 }}
#[repr(C)] #[derive(Copy, Clone)] pub struct clap_plugin_audio_ports {{ pub count: Option<unsafe extern "C" fn(*const clap_plugin, bool) -> u32>, pub get: Option<unsafe extern "C" fn(*const clap_plugin, u32, bool, *mut clap_audio_port_info) -> bool> }}
#[repr(C)] #[derive(Copy, Clone)] pub struct clap_note_port_info {{ pub id: u32, pub supported_dialects: u32, pub preferred_dialect: u32, pub name: [c_char; 256] }}
#[repr(C)] #[derive(Copy, Clone)] pub struct clap_plugin_note_ports {{ pub count: Option<unsafe extern "C" fn(*const clap_plugin, bool) -> u32>, pub get: Option<unsafe extern "C" fn(*const clap_plugin, u32, bool, *mut clap_note_port_info) -> bool> }}
#[repr(C)] #[derive(Copy, Clone)] pub struct clap_param_info {{ pub id: u32, pub flags: u32, pub cookie: *mut c_void, pub name: [c_char; 256], pub module: [c_char; 1024], pub min_value: f64, pub max_value: f64, pub default_value: f64 }}
#[repr(C)] #[derive(Copy, Clone)] pub struct clap_plugin_params {{ pub count: Option<unsafe extern "C" fn(*const clap_plugin) -> u32>, pub get_info: Option<unsafe extern "C" fn(*const clap_plugin, u32, *mut clap_param_info) -> bool>, pub get_value: Option<unsafe extern "C" fn(*const clap_plugin, u32, *mut f64) -> bool>, pub value_to_text: Option<unsafe extern "C" fn(*const clap_plugin, u32, f64, *mut c_char, u32) -> bool>, pub text_to_value: Option<unsafe extern "C" fn(*const clap_plugin, u32, *const c_char, *mut f64) -> bool>, pub flush: Option<unsafe extern "C" fn(*const clap_plugin, *const c_void, *const c_void)> }}
const CLAP_AUDIO_PORT_IS_MAIN: u32 = 1; const CLAP_PARAM_IS_STEPPED: u32 = 1 << 0; const CLAP_PARAM_IS_BYPASS: u32 = 1 << 4; const CLAP_PARAM_IS_AUTOMATABLE: u32 = 1 << 5; const CLAP_PARAM_IS_MODULATABLE: u32 = 1 << 10; const CLAP_NOTE_DIALECT_MIDI: u32 = 1 << 1;
struct FeaturePtrs([*const c_char; 3]); unsafe impl Sync for FeaturePtrs {{}}
static FACTORY_ID: &[u8] = b"clap.plugin-factory\0"; static AUDIO_PORTS_ID: &[u8] = b"clap.audio-ports\0"; static NOTE_PORTS_ID: &[u8] = b"clap.note-ports\0"; static PARAMS_ID: &[u8] = b"clap.params\0"; static STATE_ID: &[u8] = b"clap.state\0"; static LATENCY_ID: &[u8] = b"clap.latency\0"; static TAIL_ID: &[u8] = b"clap.tail\0"; static FEATURE_AUDIO_EFFECT: &[u8] = b"audio-effect\0"; static FEATURE_UTILITY: &[u8] = b"utility\0"; static FEATURES: FeaturePtrs = FeaturePtrs([FEATURE_AUDIO_EFFECT.as_ptr() as *const c_char, FEATURE_UTILITY.as_ptr() as *const c_char, ptr::null()]);
static PLUGIN_ID: &[u8] = concat!("{plugin_type_id}", "\0").as_bytes(); static PLUGIN_NAME: &[u8] = concat!("{plugin_name}", "\0").as_bytes(); static VENDOR: &[u8] = b"Signal\0"; static URL: &[u8] = b"https://signal.dev\0"; static VERSION: &[u8] = b"0.1.0\0"; static DESCRIPTION: &[u8] = b"Signal CLAP Fixture\0";
static DESCRIPTOR: clap_plugin_descriptor = clap_plugin_descriptor {{ clap_version: clap_version {{ major: 1, minor: 0, revision: 0 }}, id: PLUGIN_ID.as_ptr() as *const c_char, name: PLUGIN_NAME.as_ptr() as *const c_char, vendor: VENDOR.as_ptr() as *const c_char, url: URL.as_ptr() as *const c_char, manual_url: URL.as_ptr() as *const c_char, support_url: URL.as_ptr() as *const c_char, version: VERSION.as_ptr() as *const c_char, description: DESCRIPTION.as_ptr() as *const c_char, features: FEATURES.0.as_ptr() }};
static AUDIO_PORTS: clap_plugin_audio_ports = clap_plugin_audio_ports {{ count: Some(audio_port_count), get: Some(audio_port_get) }}; static NOTE_PORTS: clap_plugin_note_ports = clap_plugin_note_ports {{ count: Some(note_port_count), get: Some(note_port_get) }}; static PARAMS: clap_plugin_params = clap_plugin_params {{ count: Some(param_count), get_info: Some(param_get_info), get_value: Some(param_get_value), value_to_text: None, text_to_value: None, flush: None }};
static PLUGIN: clap_plugin = clap_plugin {{ desc: &DESCRIPTOR, plugin_data: ptr::null_mut(), init: Some(plugin_init), destroy: Some(plugin_destroy), activate: Some(plugin_activate), deactivate: Some(plugin_deactivate), start_processing: Some(plugin_start_processing), stop_processing: Some(plugin_stop_processing), reset: Some(plugin_reset), process: None, get_extension: Some(plugin_get_extension), on_main_thread: None }};
static FACTORY: clap_plugin_factory = clap_plugin_factory {{ get_plugin_count: Some(factory_get_plugin_count), get_plugin_descriptor: Some(factory_get_plugin_descriptor), create_plugin: Some(factory_create_plugin) }};
#[unsafe(no_mangle)] pub static clap_entry: clap_plugin_entry = clap_plugin_entry {{ clap_version: clap_version {{ major: 1, minor: 0, revision: 0 }}, init: Some(entry_init), deinit: Some(entry_deinit), get_factory: Some(entry_get_factory) }};
unsafe extern "C" fn entry_init(_plugin_path: *const c_char) -> bool {{ true }} unsafe extern "C" fn entry_deinit() {{}}
unsafe extern "C" fn entry_get_factory(factory_id: *const c_char) -> *const c_void {{ if CStr::from_ptr(factory_id).to_bytes_with_nul() == FACTORY_ID {{ (&FACTORY as *const clap_plugin_factory).cast() }} else {{ ptr::null() }} }}
unsafe extern "C" fn factory_get_plugin_count(_factory: *const clap_plugin_factory) -> u32 {{ 1 }} unsafe extern "C" fn factory_get_plugin_descriptor(_factory: *const clap_plugin_factory, index: u32) -> *const clap_plugin_descriptor {{ if index == 0 {{ &DESCRIPTOR }} else {{ ptr::null() }} }} unsafe extern "C" fn factory_create_plugin(_factory: *const clap_plugin_factory, _host: *const c_void, _plugin_id: *const c_char) -> *const clap_plugin {{ &PLUGIN }}
unsafe extern "C" fn plugin_init(_plugin: *const clap_plugin) -> bool {{ true }} unsafe extern "C" fn plugin_destroy(_plugin: *const clap_plugin) {{}} unsafe extern "C" fn plugin_activate(_plugin: *const clap_plugin, _sample_rate: f64, _min: u32, _max: u32) -> bool {{ true }} unsafe extern "C" fn plugin_deactivate(_plugin: *const clap_plugin) {{}} unsafe extern "C" fn plugin_start_processing(_plugin: *const clap_plugin) -> bool {{ true }} unsafe extern "C" fn plugin_stop_processing(_plugin: *const clap_plugin) {{}} unsafe extern "C" fn plugin_reset(_plugin: *const clap_plugin) {{}}
unsafe extern "C" fn plugin_get_extension(_plugin: *const clap_plugin, extension_id: *const c_char) -> *const c_void {{ let requested = CStr::from_ptr(extension_id).to_bytes_with_nul(); if requested == AUDIO_PORTS_ID {{ (&AUDIO_PORTS as *const clap_plugin_audio_ports).cast() }} else if requested == NOTE_PORTS_ID {{ (&NOTE_PORTS as *const clap_plugin_note_ports).cast() }} else if requested == PARAMS_ID {{ (&PARAMS as *const clap_plugin_params).cast() }} else if requested == STATE_ID || requested == LATENCY_ID || requested == TAIL_ID {{ 1usize as *const c_void }} else {{ ptr::null() }} }}
unsafe extern "C" fn audio_port_count(_plugin: *const clap_plugin, _is_input: bool) -> u32 {{ 1 }}
unsafe extern "C" fn audio_port_get(_plugin: *const clap_plugin, index: u32, is_input: bool, info: *mut clap_audio_port_info) -> bool {{ if index != 0 {{ return false; }} let name: &[u8] = if is_input {{ b\"Main Input\\0\".as_slice() }} else {{ b\"Main Output\\0\".as_slice() }}; let mut port = clap_audio_port_info {{ id: if is_input {{ 1 }} else {{ 2 }}, name: [0; 256], flags: CLAP_AUDIO_PORT_IS_MAIN, channel_count: 2, port_type: ptr::null(), in_place_pair: u32::MAX }}; for (slot, value) in port.name.iter_mut().zip(name.iter().copied()) {{ *slot = value as c_char; }} *info = port; true }}
unsafe extern "C" fn note_port_count(_plugin: *const clap_plugin, is_input: bool) -> u32 {{ if is_input {{ 1 }} else {{ {midi_outputs} }} }}
unsafe extern "C" fn note_port_get(_plugin: *const clap_plugin, index: u32, is_input: bool, info: *mut clap_note_port_info) -> bool {{ if index != 0 {{ return false; }} let name: &[u8] = if is_input {{ b\"MIDI In\\0\".as_slice() }} else {{ b\"MIDI Out\\0\".as_slice() }}; let mut port = clap_note_port_info {{ id: if is_input {{ 11 }} else {{ 12 }}, supported_dialects: CLAP_NOTE_DIALECT_MIDI, preferred_dialect: CLAP_NOTE_DIALECT_MIDI, name: [0; 256] }}; for (slot, value) in port.name.iter_mut().zip(name.iter().copied()) {{ *slot = value as c_char; }} *info = port; true }}
unsafe extern "C" fn param_count(_plugin: *const clap_plugin) -> u32 {{ 2 }}
unsafe extern "C" fn param_get_info(_plugin: *const clap_plugin, index: u32, info: *mut clap_param_info) -> bool {{ let (id, name, flags, default_value) = match index {{ 0 => (4096u32, b\"Gain\\0\".as_slice(), CLAP_PARAM_IS_AUTOMATABLE | CLAP_PARAM_IS_MODULATABLE, 0.5f64), 1 => (0u32, b\"Bypass\\0\".as_slice(), CLAP_PARAM_IS_BYPASS | CLAP_PARAM_IS_AUTOMATABLE | CLAP_PARAM_IS_STEPPED, 0.0f64), _ => return false }}; let mut param = clap_param_info {{ id, flags, cookie: ptr::null_mut(), name: [0; 256], module: [0; 1024], min_value: 0.0, max_value: 1.0, default_value }}; for (slot, value) in param.name.iter_mut().zip(name.iter().copied()) {{ *slot = value as c_char; }} *info = param; true }}
unsafe extern "C" fn param_get_value(_plugin: *const clap_plugin, param_id: u32, out_value: *mut f64) -> bool {{ if out_value.is_null() {{ return false; }} *out_value = if param_id == 4096 {{ 0.5 }} else {{ 0.0 }}; true }}
"#
    )
}

fn write_au_bundle(root: &PathBuf, bundle: &str, plugin_type_id: &str) {
    let bundle_root = root.join(bundle);
    fs::create_dir_all(bundle_root.join("Contents")).expect("public server au contents should be created");
    fs::write(
        bundle_root.join("Contents").join("Info.plist"),
        au_info_plist_contents(au_metadata_contents(plugin_type_id)),
    )
    .expect("public server au info plist should be written");
}

fn write_lv2_bundle(root: &PathBuf, bundle: &str, plugin_type_id: &str) {
    let bundle_root = root.join(bundle);
    fs::create_dir_all(&bundle_root).expect("public server lv2 bundle should be created");
    fs::write(
        bundle_root.join("manifest.ttl"),
        lv2_manifest_contents(plugin_type_id),
    )
    .expect("public server lv2 manifest should be written");
}

fn write_manifest_bundle(root: &PathBuf, bundle: &str, manifest: &str) {
    let bundle_root = root.join(bundle);
    fs::create_dir_all(&bundle_root).expect("public server lv2 bundle should be created");
    fs::write(bundle_root.join("manifest.ttl"), manifest)
        .expect("public server lv2 manifest should be written");
}

fn vst3_metadata_contents(plugin_type_id: &str) -> &'static str {
    match plugin_type_id {
        "plugin:vst3:linux-synth" => {
            "plugin_type_id=plugin:vst3:linux-synth\nclass_id=7E1D8F8A4D874D56A2C44DE250100101\ncontroller_class_id=7E1D8F8A4D874D56A2C44DE250100102\ncategory=Instrument\nvendor=Signal\nname=Signal Linux Synth VST3 Plugin\nversion=0.1.0\naudio_inputs=0\naudio_outputs=2\nmidi_inputs=1\nmidi_outputs=0\nfeatures=Instrument,Analyzer\n"
        }
        "plugin:vst3:multiout-instrument" => {
            "plugin_type_id=plugin:vst3:multiout-instrument\nclass_id=7E1D8F8A4D874D56A2C44DE250100011\ncontroller_class_id=7E1D8F8A4D874D56A2C44DE250100012\ncategory=Instrument\nvendor=Signal\nname=Signal Multi Output Instrument VST3 Plugin\nversion=0.1.0\naudio_inputs=0\naudio_outputs=6\nmidi_inputs=1\nmidi_outputs=0\nfeatures=Instrument,Analyzer\n"
        }
        "plugin:vst3:utility" => {
            "plugin_type_id=plugin:vst3:utility\nclass_id=7E1D8F8A4D874D56A2C44DE250100201\ncontroller_class_id=7E1D8F8A4D874D56A2C44DE250100202\ncategory=Fx\nvendor=Signal\nname=Signal Utility VST3 Plugin\nversion=0.1.0\naudio_inputs=2\naudio_outputs=2\nmidi_inputs=0\nmidi_outputs=0\nfeatures=AudioEffect,Utility\n"
        }
        "plugin:vst3:bus-fx" => {
            "plugin_type_id=plugin:vst3:bus-fx\nclass_id=7E1D8F8A4D874D56A2C44DE250100211\ncontroller_class_id=7E1D8F8A4D874D56A2C44DE250100212\ncategory=Fx\nvendor=Signal\nname=Signal Bus FX VST3 Plugin\nversion=0.1.0\naudio_inputs=4\naudio_outputs=4\nmidi_inputs=0\nmidi_outputs=0\nfeatures=AudioEffect,Utility\n"
        }
        other => panic!("unknown server public VST3 plugin type: {other}"),
    }
}

fn vst3_info_plist_contents(metadata: &str, bundle_root: &PathBuf) -> String {
    let mut plugin_type_id = "";
    let mut name = "Signal VST3 Plugin";
    let mut version = "0.1.0";
    let mut audio_inputs = "2";
    let mut audio_outputs = "2";
    let mut midi_inputs = "0";
    let mut midi_outputs = "0";
    let mut features = "";

    for line in metadata.lines().filter(|line| !line.trim().is_empty()) {
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        match key.trim() {
            "plugin_type_id" => plugin_type_id = value.trim(),
            "name" => name = value.trim(),
            "version" => version = value.trim(),
            "audio_inputs" => audio_inputs = value.trim(),
            "audio_outputs" => audio_outputs = value.trim(),
            "midi_inputs" => midi_inputs = value.trim(),
            "midi_outputs" => midi_outputs = value.trim(),
            "features" => features = value.trim(),
            _ => {}
        }
    }

    let executable_name = bundle_root
        .file_stem()
        .and_then(|stem| stem.to_str())
        .unwrap_or(name);
    let feature_array = features
        .split(',')
        .map(str::trim)
        .filter(|feature| !feature.is_empty())
        .map(|feature| format!("    <string>{feature}</string>"))
        .collect::<Vec<_>>()
        .join("\n");
    format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\
<!DOCTYPE plist PUBLIC \"-//Apple//DTD PLIST 1.0//EN\" \"http://www.apple.com/DTDs/PropertyList-1.0.dtd\">\n\
<plist version=\"1.0\">\n\
<dict>\n\
  <key>CFBundleExecutable</key>\n\
  <string>{executable_name}</string>\n\
  <key>CFBundleIdentifier</key>\n\
  <string>{plugin_type_id}</string>\n\
  <key>CFBundleName</key>\n\
  <string>{name}</string>\n\
  <key>CFBundlePackageType</key>\n\
  <string>BNDL</string>\n\
  <key>CFBundleShortVersionString</key>\n\
  <string>{version}</string>\n\
  <key>SignalPluginTypeId</key>\n\
  <string>{plugin_type_id}</string>\n\
  <key>SignalAudioInputs</key>\n\
  <integer>{audio_inputs}</integer>\n\
  <key>SignalAudioOutputs</key>\n\
  <integer>{audio_outputs}</integer>\n\
  <key>SignalMidiInputs</key>\n\
  <integer>{midi_inputs}</integer>\n\
  <key>SignalMidiOutputs</key>\n\
  <integer>{midi_outputs}</integer>\n\
  <key>SignalFeatures</key>\n\
  <array>\n\
{feature_array}\n\
  </array>\n\
</dict>\n\
</plist>\n"
    )
}

fn vst3_moduleinfo_contents(metadata: &str) -> String {
    let mut class_id = "";
    let mut controller_class_id = "";
    let mut category = "Fx";
    let mut vendor = "Signal";
    let mut name = "Signal VST3 Plugin";
    let mut version = "0.1.0";

    for line in metadata.lines().filter(|line| !line.trim().is_empty()) {
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        match key.trim() {
            "class_id" => class_id = value.trim(),
            "controller_class_id" => controller_class_id = value.trim(),
            "category" => category = value.trim(),
            "vendor" => vendor = value.trim(),
            "name" => name = value.trim(),
            "version" => version = value.trim(),
            _ => {}
        }
    }

    let subcategory = if category.eq_ignore_ascii_case("Instrument") {
        "Instrument"
    } else {
        "Fx"
    };
    let controller_class = if controller_class_id.is_empty()
        || controller_class_id.eq_ignore_ascii_case("none")
    {
        String::new()
    } else {
        format!(
            ",\n    {{\n      \"CID\": \"{controller_class_id}\",\n      \"Category\": \"Component Controller Class\",\n      \"Name\": \"{name}\",\n      \"Vendor\": \"{vendor}\",\n      \"Version\": \"{version}\",\n      \"Sub Categories\": [\"{subcategory}\"]\n    }}"
        )
    };

    format!(
        "{{\n  \"Name\": \"{name}\",\n  \"Version\": \"{version}\",\n  \"Factory Info\": {{\n    \"Vendor\": \"{vendor}\",\n    \"URL\": \"https://signal.dev\",\n    \"E-Mail\": \"\"\n  }},\n  \"Classes\": [\n    {{\n      \"CID\": \"{class_id}\",\n      \"Category\": \"Audio Module Class\",\n      \"Name\": \"{name}\",\n      \"Vendor\": \"{vendor}\",\n      \"Version\": \"{version}\",\n      \"Sub Categories\": [\"{subcategory}\"]\n    }}{controller_class}\n  ]\n}}\n"
    )
}

fn au_metadata_contents(plugin_type_id: &str) -> &'static str {
    match plugin_type_id {
        "plugin:au:instrument" => {
            "plugin_type_id=plugin:au:instrument\ncomponent_type=aumu\ncomponent_subtype=sigi\nmanufacturer_code=sigl\nvendor=Signal\nname=Signal Instrument AU Plugin\nversion=0.1.0\naudio_inputs=0\naudio_outputs=2\nmidi_inputs=1\nmidi_outputs=0\nfeatures=Instrument,Analyzer\n"
        }
        "plugin:au:multiout-instrument" => {
            "plugin_type_id=plugin:au:multiout-instrument\ncomponent_type=aumu\ncomponent_subtype=sigm\nmanufacturer_code=sigl\nvendor=Signal\nname=Signal Multi Output Instrument AU Plugin\nversion=0.1.0\naudio_inputs=0\naudio_outputs=6\nmidi_inputs=1\nmidi_outputs=0\nfeatures=Instrument,Analyzer\n"
        }
        "plugin:au:utility" => {
            "plugin_type_id=plugin:au:utility\ncomponent_type=aufx\ncomponent_subtype=sigu\nmanufacturer_code=sigl\nvendor=Signal\nname=Signal Utility AU Plugin\nversion=0.1.0\naudio_inputs=2\naudio_outputs=2\nmidi_inputs=0\nmidi_outputs=0\nfeatures=AudioEffect,Utility\n"
        }
        "plugin:au:bus-fx" => {
            "plugin_type_id=plugin:au:bus-fx\ncomponent_type=aufx\ncomponent_subtype=sigb\nmanufacturer_code=sigl\nvendor=Signal\nname=Signal Bus FX AU Plugin\nversion=0.1.0\naudio_inputs=4\naudio_outputs=4\nmidi_inputs=0\nmidi_outputs=0\nfeatures=AudioEffect,Utility\n"
        }
        other => panic!("unknown server public AU plugin type: {other}"),
    }
}

fn au_info_plist_contents(metadata: &str) -> String {
    let mut plugin_type_id = "";
    let mut component_type = "";
    let mut component_subtype = "";
    let mut manufacturer_code = "";
    let mut vendor = "Signal";
    let mut name = "Signal AU Plugin";
    let mut version = "0.1.0";
    let mut audio_inputs = "2";
    let mut audio_outputs = "2";
    let mut midi_inputs = "0";
    let mut midi_outputs = "0";
    let mut features = "";

    for line in metadata.lines().filter(|line| !line.trim().is_empty()) {
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        match key.trim() {
            "plugin_type_id" => plugin_type_id = value.trim(),
            "component_type" => component_type = value.trim(),
            "component_subtype" => component_subtype = value.trim(),
            "manufacturer_code" => manufacturer_code = value.trim(),
            "vendor" => vendor = value.trim(),
            "name" => name = value.trim(),
            "version" => version = value.trim(),
            "audio_inputs" => audio_inputs = value.trim(),
            "audio_outputs" => audio_outputs = value.trim(),
            "midi_inputs" => midi_inputs = value.trim(),
            "midi_outputs" => midi_outputs = value.trim(),
            "features" => features = value.trim(),
            _ => {}
        }
    }

    let feature_array = features
        .split(',')
        .map(str::trim)
        .filter(|feature| !feature.is_empty())
        .map(|feature| format!("    <string>{feature}</string>"))
        .collect::<Vec<_>>()
        .join("\n");
    format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\
<!DOCTYPE plist PUBLIC \"-//Apple//DTD PLIST 1.0//EN\" \"http://www.apple.com/DTDs/PropertyList-1.0.dtd\">\n\
<plist version=\"1.0\">\n\
<dict>\n\
  <key>AudioComponents</key>\n\
  <array>\n\
    <dict>\n\
      <key>manufacturer</key>\n\
      <string>{manufacturer_code}</string>\n\
      <key>name</key>\n\
      <string>{vendor}: {name}</string>\n\
      <key>sandboxSafe</key>\n\
      <false/>\n\
      <key>subtype</key>\n\
      <string>{component_subtype}</string>\n\
      <key>type</key>\n\
      <string>{component_type}</string>\n\
      <key>version</key>\n\
      <integer>1</integer>\n\
    </dict>\n\
  </array>\n\
  <key>CFBundleExecutable</key>\n\
  <string>{name}</string>\n\
  <key>CFBundleIdentifier</key>\n\
  <string>{plugin_type_id}</string>\n\
  <key>CFBundleName</key>\n\
  <string>{name}</string>\n\
  <key>CFBundlePackageType</key>\n\
  <string>BNDL</string>\n\
  <key>CFBundleShortVersionString</key>\n\
  <string>{version}</string>\n\
  <key>SignalPluginTypeId</key>\n\
  <string>{plugin_type_id}</string>\n\
  <key>SignalVendor</key>\n\
  <string>{vendor}</string>\n\
  <key>SignalDisplayName</key>\n\
  <string>{name}</string>\n\
  <key>SignalAudioInputs</key>\n\
  <integer>{audio_inputs}</integer>\n\
  <key>SignalAudioOutputs</key>\n\
  <integer>{audio_outputs}</integer>\n\
  <key>SignalMidiInputs</key>\n\
  <integer>{midi_inputs}</integer>\n\
  <key>SignalMidiOutputs</key>\n\
  <integer>{midi_outputs}</integer>\n\
  <key>SignalFeatures</key>\n\
  <array>\n\
{feature_array}\n\
  </array>\n\
</dict>\n\
</plist>\n"
    )
}

fn lv2_manifest_contents(plugin_type_id: &str) -> &'static str {
    match plugin_type_id {
        "plugin:lv2:linux-synth" => {
            "@prefix lv2: <http://lv2plug.in/ns/lv2core#> .\n@prefix signal: <https://signal.dev/ns/lv2#> .\nsignal:plugin_type_id \"plugin:lv2:linux-synth\" .\nsignal:plugin_uri \"https://signal.dev/plugins/lv2/linux-synth\" .\nsignal:vendor \"Signal\" .\nsignal:name \"Signal Linux Synth LV2 Plugin\" .\nsignal:version \"0.1.0\" .\nsignal:audio_inputs \"0\" .\nsignal:audio_outputs \"2\" .\nsignal:midi_inputs \"1\" .\nsignal:midi_outputs \"0\" .\nsignal:required_feature \"http://lv2plug.in/ns/ext/urid#map\" .\nsignal:required_feature \"http://lv2plug.in/ns/ext/worker#schedule\" .\nsignal:supported_extension \"http://lv2plug.in/ns/ext/patch#Message\" .\nsignal:supported_extension \"http://lv2plug.in/ns/ext/state#state\" .\nsignal:feature \"Instrument\" .\nsignal:feature \"Analyzer\" .\n"
        }
        "plugin:lv2:multiout-instrument" => {
            "@prefix lv2: <http://lv2plug.in/ns/lv2core#> .\n@prefix signal: <https://signal.dev/ns/lv2#> .\nsignal:plugin_type_id \"plugin:lv2:multiout-instrument\" .\nsignal:plugin_uri \"https://signal.dev/plugins/lv2/multiout-instrument\" .\nsignal:vendor \"Signal\" .\nsignal:name \"Signal Multi Output Instrument LV2 Plugin\" .\nsignal:version \"0.1.0\" .\nsignal:audio_inputs \"0\" .\nsignal:audio_outputs \"6\" .\nsignal:midi_inputs \"1\" .\nsignal:midi_outputs \"0\" .\nsignal:required_feature \"http://lv2plug.in/ns/ext/urid#map\" .\nsignal:required_feature \"http://lv2plug.in/ns/ext/worker#schedule\" .\nsignal:supported_extension \"http://lv2plug.in/ns/ext/patch#Message\" .\nsignal:supported_extension \"http://lv2plug.in/ns/ext/state#state\" .\nsignal:feature \"Instrument\" .\nsignal:feature \"Analyzer\" .\n"
        }
        "plugin:lv2:utility" => {
            "@prefix lv2: <http://lv2plug.in/ns/lv2core#> .\n@prefix signal: <https://signal.dev/ns/lv2#> .\nsignal:plugin_type_id \"plugin:lv2:utility\" .\nsignal:plugin_uri \"https://signal.dev/plugins/lv2/utility\" .\nsignal:vendor \"Signal\" .\nsignal:name \"Signal Utility LV2 Plugin\" .\nsignal:version \"0.1.0\" .\nsignal:audio_inputs \"2\" .\nsignal:audio_outputs \"2\" .\nsignal:midi_inputs \"0\" .\nsignal:midi_outputs \"0\" .\nsignal:required_feature \"http://lv2plug.in/ns/ext/options#options\" .\nsignal:supported_extension \"http://lv2plug.in/ns/ext/options#options\" .\nsignal:feature \"AudioEffect\" .\nsignal:feature \"Utility\" .\n"
        }
        "plugin:lv2:bus-fx" => {
            "@prefix lv2: <http://lv2plug.in/ns/lv2core#> .\n@prefix signal: <https://signal.dev/ns/lv2#> .\nsignal:plugin_type_id \"plugin:lv2:bus-fx\" .\nsignal:plugin_uri \"https://signal.dev/plugins/lv2/bus-fx\" .\nsignal:vendor \"Signal\" .\nsignal:name \"Signal Bus FX LV2 Plugin\" .\nsignal:version \"0.1.0\" .\nsignal:audio_inputs \"4\" .\nsignal:audio_outputs \"4\" .\nsignal:midi_inputs \"0\" .\nsignal:midi_outputs \"0\" .\nsignal:required_feature \"http://lv2plug.in/ns/ext/urid#map\" .\nsignal:supported_extension \"http://lv2plug.in/ns/ext/patch#Message\" .\nsignal:feature \"AudioEffect\" .\nsignal:feature \"Utility\" .\n"
        }
        other => panic!("unknown server public LV2 plugin type: {other}"),
    }
}
