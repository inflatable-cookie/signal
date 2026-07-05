//! Real compiled VST3 fixture for tests: source generator + rustc harness
//! (the VST3 mirror of `signal_plugin_clap::fixture`).
//!
//! The fixture is an actual VST3 bundle: a rustc-compiled cdylib laid out at
//! the platform module path (`Contents/MacOS/<name>` on macOS,
//! `Contents/<arch>-linux/<name>.so` on Linux) plus `Contents/Info.plist`
//! and `Contents/Resources/moduleinfo.json`, so discovery and hosting tests
//! exercise the genuine bundle-resolution/dlopen/COM path. The module
//! exports `bundleEntry`/`ModuleEntry`/`InitDll` (+ exits) and
//! `GetPluginFactory`; the single class is a single-component effect
//! implementing `IComponent`, `IAudioProcessor`, and `IEditController`
//! facets on one static object. Its `process()` is real: a fixed-gain
//! effect (output = input × [`VST3_FIXTURE_GAIN`]) with two controller
//! parameters (Gain id 4096 default 0.5, Bypass id 0) matching the CLAP
//! fixture's inventory shape.
//!
//! Shared across crates (the sandbox broker's integration tests compile the
//! same fixture), hence public but hidden from the crate's documented API.

use std::{
    path::{Path, PathBuf},
    process::Command,
};

/// Fixed linear gain the fixture's `process()` applies to every sample.
pub const VST3_FIXTURE_GAIN: f32 = 0.5;

/// Canonical component-class ID hex of the fixture (the catalog load key on
/// non-Windows platforms; hosting's hex decoder applies the COM swap on
/// Windows). Must stay in sync with the four UID fields in the generated
/// source below.
pub const VST3_FIXTURE_CLASS_ID_HEX: &str = "51F1C7A15E0C4B3D9A2F41D67B3C55E2";

/// Returns `true` when a `rustc` binary is invocable (fixture tests skip
/// gracefully when it is not).
pub fn rustc_available() -> bool {
    Command::new("rustc")
        .arg("--version")
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

/// Compile the fixture bundle into `directory`, returning the bundle root
/// (`<plugin-name>.vst3`). The bundle carries `moduleinfo.json` (so scans
/// never execute the module) and an `Info.plist` with the Signal metadata
/// keys. Errors carry the rustc failure detail.
pub fn compile_vst3_fixture(
    directory: &Path,
    plugin_type_id: &str,
    plugin_name: &str,
) -> Result<PathBuf, String> {
    let module_name = plugin_name.to_lowercase().replace(' ', "-");
    let bundle_root = directory.join(format!("{module_name}.vst3"));
    let contents = bundle_root.join("Contents");
    let module_dir = if cfg!(target_os = "macos") {
        contents.join("MacOS")
    } else if cfg!(target_os = "windows") {
        contents.join(if cfg!(target_arch = "aarch64") {
            "arm64-win"
        } else {
            "x86_64-win"
        })
    } else {
        contents.join(if cfg!(target_arch = "aarch64") {
            "aarch64-linux"
        } else {
            "x86_64-linux"
        })
    };
    let module_path = if cfg!(target_os = "macos") {
        module_dir.join(&module_name)
    } else if cfg!(target_os = "windows") {
        module_dir.join(format!("{module_name}.vst3"))
    } else {
        module_dir.join(format!("{module_name}.so"))
    };
    std::fs::create_dir_all(&module_dir)
        .map_err(|error| format!("fixture module dir create failed: {error}"))?;
    std::fs::create_dir_all(contents.join("Resources"))
        .map_err(|error| format!("fixture resources dir create failed: {error}"))?;

    let source_path = directory.join(format!("{module_name}-fixture.rs"));
    std::fs::write(&source_path, vst3_fixture_source(plugin_name))
        .map_err(|error| format!("fixture source write failed: {error}"))?;
    let output = Command::new("rustc")
        .arg("--crate-type=cdylib")
        .arg("--edition=2021")
        .arg(&source_path)
        .arg("-o")
        .arg(&module_path)
        .output()
        .map_err(|error| format!("rustc invocation failed: {error}"))?;
    if !output.status.success() {
        return Err(format!(
            "vst3 fixture compilation failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    std::fs::write(
        contents.join("Info.plist"),
        fixture_info_plist(plugin_type_id, plugin_name, &module_name),
    )
    .map_err(|error| format!("fixture Info.plist write failed: {error}"))?;
    std::fs::write(
        contents.join("Resources").join("moduleinfo.json"),
        fixture_moduleinfo(plugin_name),
    )
    .map_err(|error| format!("fixture moduleinfo write failed: {error}"))?;
    Ok(bundle_root)
}

fn fixture_info_plist(plugin_type_id: &str, plugin_name: &str, module_name: &str) -> String {
    format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\
<!DOCTYPE plist PUBLIC \"-//Apple//DTD PLIST 1.0//EN\" \"http://www.apple.com/DTDs/PropertyList-1.0.dtd\">\n\
<plist version=\"1.0\">\n\
<dict>\n\
  <key>CFBundleExecutable</key>\n\
  <string>{module_name}</string>\n\
  <key>CFBundleIdentifier</key>\n\
  <string>dev.signal.vst3-fixture</string>\n\
  <key>CFBundleName</key>\n\
  <string>{plugin_name}</string>\n\
  <key>CFBundlePackageType</key>\n\
  <string>BNDL</string>\n\
  <key>CFBundleShortVersionString</key>\n\
  <string>0.1.0</string>\n\
  <key>SignalPluginTypeId</key>\n\
  <string>{plugin_type_id}</string>\n\
  <key>SignalAudioInputs</key>\n\
  <integer>2</integer>\n\
  <key>SignalAudioOutputs</key>\n\
  <integer>2</integer>\n\
  <key>SignalMidiInputs</key>\n\
  <integer>0</integer>\n\
  <key>SignalMidiOutputs</key>\n\
  <integer>0</integer>\n\
  <key>SignalFeatures</key>\n\
  <array>\n\
    <string>AudioEffect</string>\n\
    <string>Utility</string>\n\
  </array>\n\
</dict>\n\
</plist>\n"
    )
}

fn fixture_moduleinfo(plugin_name: &str) -> String {
    format!(
        "{{\n  \"Name\": \"{plugin_name}\",\n  \"Version\": \"0.1.0\",\n  \"Factory Info\": {{\n    \"Vendor\": \"Signal\",\n    \"URL\": \"https://signal.dev\",\n    \"E-Mail\": \"\"\n  }},\n  \"Classes\": [\n    {{\n      \"CID\": \"{VST3_FIXTURE_CLASS_ID_HEX}\",\n      \"Category\": \"Audio Module Class\",\n      \"Name\": \"{plugin_name}\",\n      \"Vendor\": \"Signal\",\n      \"Version\": \"0.1.0\",\n      \"Sub Categories\": [\"Fx\"]\n    }}\n  ]\n}}\n"
    )
}

/// Full Rust source of the fixture cdylib.
pub fn vst3_fixture_source(plugin_name: &str) -> String {
    format!(
        r#"
//! rustc-compiled VST3 fixture module: single-component stereo gain effect.
#![allow(non_snake_case)]

use std::ffi::{{c_char, c_void}};
use std::ptr;

type Tresult = i32;
const K_RESULT_OK: Tresult = 0;
const K_RESULT_FALSE: Tresult = 1;
#[cfg(target_os = "windows")]
const K_NO_INTERFACE: Tresult = 0x8000_4002_u32 as i32;
#[cfg(not(target_os = "windows"))]
const K_NO_INTERFACE: Tresult = -1;

type Tuid = [u8; 16];

const fn tuid_from_uid(l1: u32, l2: u32, l3: u32, l4: u32) -> Tuid {{
    if cfg!(target_os = "windows") {{
        [
            (l1 & 0xFF) as u8, ((l1 >> 8) & 0xFF) as u8,
            ((l1 >> 16) & 0xFF) as u8, ((l1 >> 24) & 0xFF) as u8,
            ((l2 >> 16) & 0xFF) as u8, ((l2 >> 24) & 0xFF) as u8,
            (l2 & 0xFF) as u8, ((l2 >> 8) & 0xFF) as u8,
            ((l3 >> 24) & 0xFF) as u8, ((l3 >> 16) & 0xFF) as u8,
            ((l3 >> 8) & 0xFF) as u8, (l3 & 0xFF) as u8,
            ((l4 >> 24) & 0xFF) as u8, ((l4 >> 16) & 0xFF) as u8,
            ((l4 >> 8) & 0xFF) as u8, (l4 & 0xFF) as u8,
        ]
    }} else {{
        [
            ((l1 >> 24) & 0xFF) as u8, ((l1 >> 16) & 0xFF) as u8,
            ((l1 >> 8) & 0xFF) as u8, (l1 & 0xFF) as u8,
            ((l2 >> 24) & 0xFF) as u8, ((l2 >> 16) & 0xFF) as u8,
            ((l2 >> 8) & 0xFF) as u8, (l2 & 0xFF) as u8,
            ((l3 >> 24) & 0xFF) as u8, ((l3 >> 16) & 0xFF) as u8,
            ((l3 >> 8) & 0xFF) as u8, (l3 & 0xFF) as u8,
            ((l4 >> 24) & 0xFF) as u8, ((l4 >> 16) & 0xFF) as u8,
            ((l4 >> 8) & 0xFF) as u8, (l4 & 0xFF) as u8,
        ]
    }}
}}

const FUNKNOWN_IID: Tuid = tuid_from_uid(0x00000000, 0x00000000, 0xC0000000, 0x00000046);
const IPLUGIN_BASE_IID: Tuid = tuid_from_uid(0x22888DDB, 0x156E45AE, 0x8358B348, 0x08190625);
const IPLUGIN_FACTORY_IID: Tuid = tuid_from_uid(0x7A4D811C, 0x52114A1F, 0xAEED8D2C, 0x4EEBC9CB);
const ICOMPONENT_IID: Tuid = tuid_from_uid(0xE831FF31, 0xF2D54301, 0x928EBBEE, 0x25697802);
const IAUDIO_PROCESSOR_IID: Tuid = tuid_from_uid(0x42043F99, 0xB7DA453C, 0xA569E79D, 0x9AAEC33D);
const IEDIT_CONTROLLER_IID: Tuid = tuid_from_uid(0xDCD7BBE3, 0x7742448D, 0xA874AAF0, 0x0B96B23E);

/// Component class CID; canonical hex "51F1C7A15E0C4B3D9A2F41D67B3C55E2"
/// (kept in sync with the host crate's `VST3_FIXTURE_CLASS_ID_HEX`).
const FIXTURE_CID: Tuid = tuid_from_uid(0x51F1C7A1, 0x5E0C4B3D, 0x9A2F41D6, 0x7B3C55E2);

/// Fixed gain applied by process() (kept in sync with the host crate's
/// `VST3_FIXTURE_GAIN`).
const FIXTURE_GAIN: f32 = {gain};

const PLUGIN_NAME: &str = "{plugin_name}";

const K_AUDIO: i32 = 0;
const K_INPUT: i32 = 0;
const K_OUTPUT: i32 = 1;
const STEREO: u64 = 0x3;
const PARAM_CAN_AUTOMATE: i32 = 1;
const PARAM_IS_BYPASS: i32 = 1 << 16;

fn write_utf16(dst: &mut [i16; 128], text: &str) {{
    let mut index = 0;
    for unit in text.encode_utf16().take(127) {{
        dst[index] = unit as i16;
        index += 1;
    }}
    dst[index] = 0;
}}

fn write_utf16_ptr(dst: *mut i16, text: &str) {{
    if dst.is_null() {{
        return;
    }}
    let mut index = 0;
    for unit in text.encode_utf16().take(127) {{
        unsafe {{ *dst.add(index) = unit as i16 }};
        index += 1;
    }}
    unsafe {{ *dst.add(index) = 0 }};
}}

// ── Interface structs ───────────────────────────────────────────────────────

#[repr(C)]
struct BusInfo {{
    media_type: i32,
    direction: i32,
    channel_count: i32,
    name: [i16; 128],
    bus_type: i32,
    flags: u32,
}}

#[repr(C)]
struct ProcessSetup {{
    process_mode: i32,
    symbolic_sample_size: i32,
    max_samples_per_block: i32,
    sample_rate: f64,
}}

#[repr(C)]
struct AudioBusBuffers {{
    num_channels: i32,
    silence_flags: u64,
    channel_buffers32: *mut *mut f32,
}}

#[repr(C)]
struct ProcessData {{
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
}}

#[repr(C)]
struct ParameterInfo {{
    id: u32,
    title: [i16; 128],
    short_title: [i16; 128],
    units: [i16; 128],
    step_count: i32,
    default_normalized_value: f64,
    unit_id: i32,
    flags: i32,
}}

#[repr(C)]
struct ComponentVTable {{
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
}}

#[repr(C)]
struct AudioProcessorVTable {{
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
}}

#[repr(C)]
struct EditControllerVTable {{
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
    get_param_string_by_value:
        unsafe extern "C" fn(*mut c_void, u32, f64, *mut i16) -> Tresult,
    get_param_value_by_string:
        unsafe extern "C" fn(*mut c_void, u32, *mut i16, *mut f64) -> Tresult,
    normalized_param_to_plain: unsafe extern "C" fn(*mut c_void, u32, f64) -> f64,
    plain_param_to_normalized: unsafe extern "C" fn(*mut c_void, u32, f64) -> f64,
    get_param_normalized: unsafe extern "C" fn(*mut c_void, u32) -> f64,
    set_param_normalized: unsafe extern "C" fn(*mut c_void, u32, f64) -> Tresult,
    set_component_handler: unsafe extern "C" fn(*mut c_void, *mut c_void) -> Tresult,
    create_view: unsafe extern "C" fn(*mut c_void, *const c_char) -> *mut c_void,
}}

#[repr(C)]
struct PFactoryInfo {{
    vendor: [c_char; 64],
    url: [c_char; 256],
    email: [c_char; 128],
    flags: i32,
}}

#[repr(C)]
struct PClassInfo {{
    cid: Tuid,
    cardinality: i32,
    category: [c_char; 32],
    name: [c_char; 64],
}}

#[repr(C)]
struct FactoryVTable {{
    query_interface: unsafe extern "C" fn(*mut c_void, *const Tuid, *mut *mut c_void) -> Tresult,
    add_ref: unsafe extern "C" fn(*mut c_void) -> u32,
    release: unsafe extern "C" fn(*mut c_void) -> u32,
    get_factory_info: unsafe extern "C" fn(*mut c_void, *mut PFactoryInfo) -> Tresult,
    count_classes: unsafe extern "C" fn(*mut c_void) -> i32,
    get_class_info: unsafe extern "C" fn(*mut c_void, i32, *mut PClassInfo) -> Tresult,
    create_instance:
        unsafe extern "C" fn(*mut c_void, *const u8, *const u8, *mut *mut c_void) -> Tresult,
}}

// ── The single-component plugin object (three facets, static) ──────────────

/// One static COM object: facet 0 = IComponent (also FUnknown/IPluginBase),
/// facet 1 = IAudioProcessor, facet 2 = IEditController. queryInterface
/// hands out facet addresses; refcounting is a no-op (static lifetime).
#[repr(C)]
struct FixtureObject {{
    component_vtable: *const ComponentVTable,
    processor_vtable: *const AudioProcessorVTable,
    controller_vtable: *const EditControllerVTable,
}}

unsafe impl Sync for FixtureObject {{}}

static COMPONENT_VTABLE: ComponentVTable = ComponentVTable {{
    query_interface: component_query_interface,
    add_ref: no_op_add_ref,
    release: no_op_release,
    initialize: base_initialize,
    terminate: base_terminate,
    get_controller_class_id: component_get_controller_class_id,
    set_io_mode: component_set_io_mode,
    get_bus_count: component_get_bus_count,
    get_bus_info: component_get_bus_info,
    get_routing_info: component_get_routing_info,
    activate_bus: component_activate_bus,
    set_active: component_set_active,
    set_state: state_noop,
    get_state: state_noop,
}};

static PROCESSOR_VTABLE: AudioProcessorVTable = AudioProcessorVTable {{
    query_interface: processor_query_interface,
    add_ref: no_op_add_ref,
    release: no_op_release,
    set_bus_arrangements: processor_set_bus_arrangements,
    get_bus_arrangement: processor_get_bus_arrangement,
    can_process_sample_size: processor_can_process_sample_size,
    get_latency_samples: processor_get_latency_samples,
    setup_processing: processor_setup_processing,
    set_processing: processor_set_processing,
    process: processor_process,
    get_tail_samples: processor_get_tail_samples,
}};

static CONTROLLER_VTABLE: EditControllerVTable = EditControllerVTable {{
    query_interface: controller_query_interface,
    add_ref: no_op_add_ref,
    release: no_op_release,
    initialize: base_initialize,
    terminate: base_terminate,
    set_component_state: state_noop,
    set_state: state_noop,
    get_state: state_noop,
    get_parameter_count: controller_get_parameter_count,
    get_parameter_info: controller_get_parameter_info,
    get_param_string_by_value: controller_get_param_string_by_value,
    get_param_value_by_string: controller_get_param_value_by_string,
    normalized_param_to_plain: controller_normalized_param_to_plain,
    plain_param_to_normalized: controller_plain_param_to_normalized,
    get_param_normalized: controller_get_param_normalized,
    set_param_normalized: controller_set_param_normalized,
    set_component_handler: controller_set_component_handler,
    create_view: controller_create_view,
}};

static FIXTURE_OBJECT: FixtureObject = FixtureObject {{
    component_vtable: &COMPONENT_VTABLE,
    processor_vtable: &PROCESSOR_VTABLE,
    controller_vtable: &CONTROLLER_VTABLE,
}};

fn object_base() -> *mut c_void {{
    &FIXTURE_OBJECT as *const FixtureObject as *mut c_void
}}

fn processor_facet() -> *mut c_void {{
    unsafe {{ &raw const FIXTURE_OBJECT.processor_vtable as *mut c_void }}
}}

fn controller_facet() -> *mut c_void {{
    unsafe {{ &raw const FIXTURE_OBJECT.controller_vtable as *mut c_void }}
}}

unsafe fn facet_for(iid: *const Tuid) -> Option<*mut c_void> {{
    if iid.is_null() {{
        return None;
    }}
    let iid = *iid;
    if iid == FUNKNOWN_IID || iid == IPLUGIN_BASE_IID || iid == ICOMPONENT_IID {{
        Some(object_base())
    }} else if iid == IAUDIO_PROCESSOR_IID {{
        Some(processor_facet())
    }} else if iid == IEDIT_CONTROLLER_IID {{
        Some(controller_facet())
    }} else {{
        None
    }}
}}

unsafe extern "C" fn component_query_interface(
    _this: *mut c_void,
    iid: *const Tuid,
    out: *mut *mut c_void,
) -> Tresult {{
    shared_query_interface(iid, out)
}}

unsafe extern "C" fn processor_query_interface(
    _this: *mut c_void,
    iid: *const Tuid,
    out: *mut *mut c_void,
) -> Tresult {{
    shared_query_interface(iid, out)
}}

unsafe extern "C" fn controller_query_interface(
    _this: *mut c_void,
    iid: *const Tuid,
    out: *mut *mut c_void,
) -> Tresult {{
    shared_query_interface(iid, out)
}}

unsafe fn shared_query_interface(iid: *const Tuid, out: *mut *mut c_void) -> Tresult {{
    if out.is_null() {{
        return K_NO_INTERFACE;
    }}
    match facet_for(iid) {{
        Some(facet) => {{
            *out = facet;
            K_RESULT_OK
        }}
        None => {{
            *out = ptr::null_mut();
            K_NO_INTERFACE
        }}
    }}
}}

unsafe extern "C" fn no_op_add_ref(_this: *mut c_void) -> u32 {{ 1 }}
unsafe extern "C" fn no_op_release(_this: *mut c_void) -> u32 {{ 1 }}
unsafe extern "C" fn base_initialize(_this: *mut c_void, _context: *mut c_void) -> Tresult {{
    K_RESULT_OK
}}
unsafe extern "C" fn base_terminate(_this: *mut c_void) -> Tresult {{ K_RESULT_OK }}
unsafe extern "C" fn state_noop(_this: *mut c_void, _stream: *mut c_void) -> Tresult {{
    K_RESULT_OK
}}

unsafe extern "C" fn component_get_controller_class_id(
    _this: *mut c_void,
    _class_id: *mut Tuid,
) -> Tresult {{
    // Single-component plugin: the controller is a facet of this object.
    K_RESULT_FALSE
}}

unsafe extern "C" fn component_set_io_mode(_this: *mut c_void, _mode: i32) -> Tresult {{
    K_RESULT_OK
}}

unsafe extern "C" fn component_get_bus_count(
    _this: *mut c_void,
    media_type: i32,
    _direction: i32,
) -> i32 {{
    if media_type == K_AUDIO {{ 1 }} else {{ 0 }}
}}

unsafe extern "C" fn component_get_bus_info(
    _this: *mut c_void,
    media_type: i32,
    direction: i32,
    index: i32,
    info: *mut BusInfo,
) -> Tresult {{
    if media_type != K_AUDIO || index != 0 || info.is_null() {{
        return K_RESULT_FALSE;
    }}
    let info = &mut *info;
    info.media_type = K_AUDIO;
    info.direction = direction;
    info.channel_count = 2;
    info.bus_type = 0; // kMain
    info.flags = 1; // kDefaultActive
    let mut name = [0i16; 128];
    write_utf16(
        &mut name,
        if direction == K_INPUT {{ "Main Input" }} else {{ "Main Output" }},
    );
    info.name = name;
    K_RESULT_OK
}}

unsafe extern "C" fn component_get_routing_info(
    _this: *mut c_void,
    _input: *mut c_void,
    _output: *mut c_void,
) -> Tresult {{
    K_RESULT_FALSE
}}

unsafe extern "C" fn component_activate_bus(
    _this: *mut c_void,
    media_type: i32,
    _direction: i32,
    index: i32,
    _state: u8,
) -> Tresult {{
    if media_type == K_AUDIO && index == 0 {{ K_RESULT_OK }} else {{ K_RESULT_FALSE }}
}}

unsafe extern "C" fn component_set_active(_this: *mut c_void, _state: u8) -> Tresult {{
    K_RESULT_OK
}}

unsafe extern "C" fn processor_set_bus_arrangements(
    _this: *mut c_void,
    inputs: *mut u64,
    num_inputs: i32,
    outputs: *mut u64,
    num_outputs: i32,
) -> Tresult {{
    if num_inputs == 1
        && num_outputs == 1
        && !inputs.is_null()
        && !outputs.is_null()
        && *inputs == STEREO
        && *outputs == STEREO
    {{
        K_RESULT_OK
    }} else {{
        K_RESULT_FALSE
    }}
}}

unsafe extern "C" fn processor_get_bus_arrangement(
    _this: *mut c_void,
    _direction: i32,
    index: i32,
    arrangement: *mut u64,
) -> Tresult {{
    if index != 0 || arrangement.is_null() {{
        return K_RESULT_FALSE;
    }}
    *arrangement = STEREO;
    K_RESULT_OK
}}

unsafe extern "C" fn processor_can_process_sample_size(
    _this: *mut c_void,
    symbolic_sample_size: i32,
) -> Tresult {{
    if symbolic_sample_size == 0 {{ K_RESULT_OK }} else {{ K_RESULT_FALSE }}
}}

unsafe extern "C" fn processor_get_latency_samples(_this: *mut c_void) -> u32 {{ 0 }}

unsafe extern "C" fn processor_setup_processing(
    _this: *mut c_void,
    setup: *mut ProcessSetup,
) -> Tresult {{
    if setup.is_null() {{ K_RESULT_FALSE }} else {{ K_RESULT_OK }}
}}

unsafe extern "C" fn processor_set_processing(_this: *mut c_void, _state: u8) -> Tresult {{
    K_RESULT_OK
}}

/// Real audio processing: output = input × FIXTURE_GAIN on every channel of
/// the main bus pair.
unsafe extern "C" fn processor_process(_this: *mut c_void, data: *mut ProcessData) -> Tresult {{
    if data.is_null() {{
        return K_RESULT_FALSE;
    }}
    let data = &*data;
    if data.num_inputs < 1
        || data.num_outputs < 1
        || data.inputs.is_null()
        || data.outputs.is_null()
    {{
        return K_RESULT_FALSE;
    }}
    let input = &*data.inputs;
    let output = &*data.outputs;
    if input.channel_buffers32.is_null() || output.channel_buffers32.is_null() {{
        return K_RESULT_FALSE;
    }}
    let frames = data.num_samples.max(0) as usize;
    let channels = input.num_channels.min(output.num_channels).max(0) as usize;
    for channel in 0..channels {{
        let source = *input.channel_buffers32.add(channel);
        let dest = *output.channel_buffers32.add(channel);
        if source.is_null() || dest.is_null() {{
            return K_RESULT_FALSE;
        }}
        for frame in 0..frames {{
            *dest.add(frame) = *source.add(frame) * FIXTURE_GAIN;
        }}
    }}
    K_RESULT_OK
}}

unsafe extern "C" fn processor_get_tail_samples(_this: *mut c_void) -> u32 {{ 0 }}

unsafe extern "C" fn controller_get_parameter_count(_this: *mut c_void) -> i32 {{ 2 }}

unsafe extern "C" fn controller_get_parameter_info(
    _this: *mut c_void,
    index: i32,
    info: *mut ParameterInfo,
) -> Tresult {{
    if info.is_null() {{
        return K_RESULT_FALSE;
    }}
    let (id, title, unit, flags, step_count, default_value) = match index {{
        0 => (4096u32, "Gain", "dB", PARAM_CAN_AUTOMATE, 0, 0.5f64),
        1 => (0u32, "Bypass", "", PARAM_CAN_AUTOMATE | PARAM_IS_BYPASS, 1, 0.0f64),
        _ => return K_RESULT_FALSE,
    }};
    let info = &mut *info;
    info.id = id;
    let mut buffer = [0i16; 128];
    write_utf16(&mut buffer, title);
    info.title = buffer;
    info.short_title = buffer;
    let mut unit_buffer = [0i16; 128];
    write_utf16(&mut unit_buffer, unit);
    info.units = unit_buffer;
    info.step_count = step_count;
    info.default_normalized_value = default_value;
    info.unit_id = 0;
    info.flags = flags;
    K_RESULT_OK
}}

unsafe extern "C" fn controller_get_param_string_by_value(
    _this: *mut c_void,
    _id: u32,
    value: f64,
    string: *mut i16,
) -> Tresult {{
    write_utf16_ptr(string, &format!("{{value:.2}}"));
    K_RESULT_OK
}}

unsafe extern "C" fn controller_get_param_value_by_string(
    _this: *mut c_void,
    _id: u32,
    _string: *mut i16,
    _value: *mut f64,
) -> Tresult {{
    K_RESULT_FALSE
}}

unsafe extern "C" fn controller_normalized_param_to_plain(
    _this: *mut c_void,
    _id: u32,
    normalized: f64,
) -> f64 {{
    normalized
}}

unsafe extern "C" fn controller_plain_param_to_normalized(
    _this: *mut c_void,
    _id: u32,
    plain: f64,
) -> f64 {{
    plain
}}

unsafe extern "C" fn controller_get_param_normalized(_this: *mut c_void, id: u32) -> f64 {{
    if id == 4096 {{ 0.5 }} else {{ 0.0 }}
}}

unsafe extern "C" fn controller_set_param_normalized(
    _this: *mut c_void,
    _id: u32,
    _value: f64,
) -> Tresult {{
    K_RESULT_OK
}}

unsafe extern "C" fn controller_set_component_handler(
    _this: *mut c_void,
    _handler: *mut c_void,
) -> Tresult {{
    K_RESULT_OK
}}

unsafe extern "C" fn controller_create_view(
    _this: *mut c_void,
    _name: *const c_char,
) -> *mut c_void {{
    ptr::null_mut()
}}

// ── Factory ─────────────────────────────────────────────────────────────────

#[repr(C)]
struct FixtureFactory {{
    vtable: *const FactoryVTable,
}}

unsafe impl Sync for FixtureFactory {{}}

static FACTORY_VTABLE: FactoryVTable = FactoryVTable {{
    query_interface: factory_query_interface,
    add_ref: no_op_add_ref,
    release: no_op_release,
    get_factory_info: factory_get_factory_info,
    count_classes: factory_count_classes,
    get_class_info: factory_get_class_info,
    create_instance: factory_create_instance,
}};

static FACTORY: FixtureFactory = FixtureFactory {{
    vtable: &FACTORY_VTABLE,
}};

fn write_c_chars(dst: &mut [c_char], text: &str) {{
    for (slot, byte) in dst.iter_mut().zip(text.bytes().chain(std::iter::once(0))) {{
        *slot = byte as c_char;
    }}
}}

unsafe extern "C" fn factory_query_interface(
    this: *mut c_void,
    iid: *const Tuid,
    out: *mut *mut c_void,
) -> Tresult {{
    if out.is_null() {{
        return K_NO_INTERFACE;
    }}
    if !iid.is_null() && (*iid == FUNKNOWN_IID || *iid == IPLUGIN_FACTORY_IID) {{
        *out = this;
        return K_RESULT_OK;
    }}
    *out = ptr::null_mut();
    K_NO_INTERFACE
}}

unsafe extern "C" fn factory_get_factory_info(
    _this: *mut c_void,
    info: *mut PFactoryInfo,
) -> Tresult {{
    if info.is_null() {{
        return K_RESULT_FALSE;
    }}
    let info = &mut *info;
    info.vendor = [0; 64];
    info.url = [0; 256];
    info.email = [0; 128];
    info.flags = 0x10; // kUnicode
    write_c_chars(&mut info.vendor, "Signal");
    write_c_chars(&mut info.url, "https://signal.dev");
    K_RESULT_OK
}}

unsafe extern "C" fn factory_count_classes(_this: *mut c_void) -> i32 {{ 1 }}

unsafe extern "C" fn factory_get_class_info(
    _this: *mut c_void,
    index: i32,
    info: *mut PClassInfo,
) -> Tresult {{
    if index != 0 || info.is_null() {{
        return K_RESULT_FALSE;
    }}
    let info = &mut *info;
    info.cid = FIXTURE_CID;
    info.cardinality = 0x7FFFFFFF; // kManyInstances
    info.category = [0; 32];
    info.name = [0; 64];
    write_c_chars(&mut info.category, "Audio Module Class");
    write_c_chars(&mut info.name, PLUGIN_NAME);
    K_RESULT_OK
}}

unsafe extern "C" fn factory_create_instance(
    _this: *mut c_void,
    cid: *const u8,
    iid: *const u8,
    out: *mut *mut c_void,
) -> Tresult {{
    if out.is_null() {{
        return K_NO_INTERFACE;
    }}
    *out = ptr::null_mut();
    if cid.is_null() || iid.is_null() {{
        return K_NO_INTERFACE;
    }}
    let mut requested_cid: Tuid = [0; 16];
    ptr::copy_nonoverlapping(cid, requested_cid.as_mut_ptr(), 16);
    if requested_cid != FIXTURE_CID {{
        return K_NO_INTERFACE;
    }}
    shared_query_interface(iid as *const Tuid, out)
}}

// ── Module entry points ─────────────────────────────────────────────────────

#[no_mangle]
pub unsafe extern "C" fn GetPluginFactory() -> *mut c_void {{
    &FACTORY as *const FixtureFactory as *mut c_void
}}

#[no_mangle]
pub unsafe extern "C" fn bundleEntry(_bundle_ref: *mut c_void) -> bool {{ true }}

#[no_mangle]
pub unsafe extern "C" fn bundleExit() {{}}

#[no_mangle]
pub unsafe extern "C" fn ModuleEntry(_shared_library_handle: *mut c_void) -> bool {{ true }}

#[no_mangle]
pub unsafe extern "C" fn ModuleExit() {{}}

#[no_mangle]
pub unsafe extern "C" fn InitDll() -> bool {{ true }}

#[no_mangle]
pub unsafe extern "C" fn ExitDll() {{}}
"#,
        gain = VST3_FIXTURE_GAIN,
        plugin_name = plugin_name,
    )
}
