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

/// Linear gain the fixture's `process()` applies until a param write lands
/// (the Gain param's default; g12.023 makes the param live via the block's
/// input `IParameterChanges`).
pub const VST3_FIXTURE_GAIN: f32 = 0.5;

/// Param id of the fixture's live Gain parameter (normalized == plain).
pub const VST3_FIXTURE_GAIN_PARAM_ID: u32 = 4096;

/// MIDI controller number the fixture's `IMidiMapping` assigns to the Gain
/// parameter (bus 0, channel 0) — the CC → param delivery proof.
pub const VST3_FIXTURE_GAIN_CC: u8 = 7;

/// Initial editor content size the fixture's `IPlugView::getSize` reports.
pub const VST3_FIXTURE_VIEW_INITIAL_SIZE: (u32, u32) = (400, 300);

/// The resize the fixture's view requests from the host `IPlugFrame` on
/// `attached` (exercises the resizeView callback path without any real
/// window system).
pub const VST3_FIXTURE_VIEW_REQUESTED_SIZE: (u32, u32) = (500, 320);

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
const IPLUG_VIEW_IID: Tuid = tuid_from_uid(0x5BC32507, 0xD06049EA, 0xA6151B52, 0x2B755B29);
const IMIDI_MAPPING_IID: Tuid = tuid_from_uid(0xDF695DF2, 0x8B4B47EB, 0xAB3EF8FB, 0x2D1F6BB2);

/// Component class CID; canonical hex "51F1C7A15E0C4B3D9A2F41D67B3C55E2"
/// (kept in sync with the host crate's `VST3_FIXTURE_CLASS_ID_HEX`).
const FIXTURE_CID: Tuid = tuid_from_uid(0x51F1C7A1, 0x5E0C4B3D, 0x9A2F41D6, 0x7B3C55E2);

/// Default gain applied by process() (kept in sync with the host crate's
/// `VST3_FIXTURE_GAIN`). Live value in `GAIN_BITS`, updated ONLY from the
/// process-data input `IParameterChanges` — the g12.023 processor-side
/// param wire (controller setParamNormalized stays bookkeeping).
const FIXTURE_GAIN: f32 = {gain};

static GAIN_BITS: std::sync::atomic::AtomicU32 =
    std::sync::atomic::AtomicU32::new(f32::to_bits(FIXTURE_GAIN));

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
    midi_mapping_vtable: *const MidiMappingVTable,
}}

unsafe impl Sync for FixtureObject {{}}

/// IMidiMapping (FUnknown + getMidiControllerAssignment).
#[repr(C)]
struct MidiMappingVTable {{
    query_interface: unsafe extern "C" fn(*mut c_void, *const Tuid, *mut *mut c_void) -> Tresult,
    add_ref: unsafe extern "C" fn(*mut c_void) -> u32,
    release: unsafe extern "C" fn(*mut c_void) -> u32,
    get_midi_controller_assignment:
        unsafe extern "C" fn(*mut c_void, i32, i16, i16, *mut u32) -> Tresult,
}}

static MIDI_MAPPING_VTABLE: MidiMappingVTable = MidiMappingVTable {{
    query_interface: midi_mapping_query_interface,
    add_ref: no_op_add_ref,
    release: no_op_release,
    get_midi_controller_assignment: midi_mapping_get_assignment,
}};

unsafe extern "C" fn midi_mapping_query_interface(
    _this: *mut c_void,
    iid: *const Tuid,
    out: *mut *mut c_void,
) -> Tresult {{
    shared_query_interface(iid, out)
}}

/// CC 7 plus VST3 pitch-bend (128) and aftertouch (129) on bus 0 / channel
/// 0 map to the Gain param (id 4096); everything else is unassigned.
unsafe extern "C" fn midi_mapping_get_assignment(
    _this: *mut c_void,
    bus_index: i32,
    channel: i16,
    controller_number: i16,
    parameter_id: *mut u32,
) -> Tresult {{
    if parameter_id.is_null() {{
        return K_RESULT_FALSE;
    }}
    if bus_index == 0 && channel == 0 && matches!(controller_number, 7 | 128 | 129) {{
        *parameter_id = 4096;
        K_RESULT_OK
    }} else {{
        K_RESULT_FALSE
    }}
}}

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
    midi_mapping_vtable: &MIDI_MAPPING_VTABLE,
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

fn midi_mapping_facet() -> *mut c_void {{
    unsafe {{ &raw const FIXTURE_OBJECT.midi_mapping_vtable as *mut c_void }}
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
    }} else if iid == IMIDI_MAPPING_IID {{
        Some(midi_mapping_facet())
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

// ── Input IParameterChanges consumption (g12.023) ──────────────────────────

#[repr(C)]
struct ParamValueQueueVTable {{
    query_interface: unsafe extern "C" fn(*mut c_void, *const Tuid, *mut *mut c_void) -> Tresult,
    add_ref: unsafe extern "C" fn(*mut c_void) -> u32,
    release: unsafe extern "C" fn(*mut c_void) -> u32,
    get_parameter_id: unsafe extern "C" fn(*mut c_void) -> u32,
    get_point_count: unsafe extern "C" fn(*mut c_void) -> i32,
    get_point: unsafe extern "C" fn(*mut c_void, i32, *mut i32, *mut f64) -> Tresult,
    add_point: unsafe extern "C" fn(*mut c_void, i32, f64, *mut i32) -> Tresult,
}}

#[repr(C)]
struct ParameterChangesVTable {{
    query_interface: unsafe extern "C" fn(*mut c_void, *const Tuid, *mut *mut c_void) -> Tresult,
    add_ref: unsafe extern "C" fn(*mut c_void) -> u32,
    release: unsafe extern "C" fn(*mut c_void) -> u32,
    get_parameter_count: unsafe extern "C" fn(*mut c_void) -> i32,
    get_parameter_data: unsafe extern "C" fn(*mut c_void, i32) -> *mut c_void,
    add_parameter_data: unsafe extern "C" fn(*mut c_void, *const u32, *mut i32) -> *mut c_void,
}}

/// Per-block cap on gain steps gathered from param points and note events.
const GAIN_STEP_CAPACITY: usize = 64;

/// Gather every Gain (id 4096) point in the block's input parameter
/// changes as `(sample_offset, gain)` steps — the real host contract:
/// sample-offset points apply FROM their offset (wire writes arrive at
/// offset 0, IMidiMapping-routed CC at the CC event's offset).
unsafe fn gather_parameter_steps(
    changes: *mut c_void,
    steps: &mut [(i32, f32); GAIN_STEP_CAPACITY],
    step_count: &mut usize,
) {{
    if changes.is_null() {{
        return;
    }}
    let changes_vtable = *(changes as *mut *const ParameterChangesVTable);
    let count = ((*changes_vtable).get_parameter_count)(changes);
    for index in 0..count {{
        let queue = ((*changes_vtable).get_parameter_data)(changes, index);
        if queue.is_null() {{
            continue;
        }}
        let queue_vtable = *(queue as *mut *const ParamValueQueueVTable);
        if ((*queue_vtable).get_parameter_id)(queue) != 4096 {{
            continue;
        }}
        let points = ((*queue_vtable).get_point_count)(queue);
        for point in 0..points {{
            let mut sample_offset = 0i32;
            let mut value = 0f64;
            if ((*queue_vtable).get_point)(queue, point, &mut sample_offset, &mut value)
                == K_RESULT_OK
                && *step_count < GAIN_STEP_CAPACITY
            {{
                steps[*step_count] = (sample_offset, value as f32);
                *step_count += 1;
            }}
        }}
    }}
}}

// ── Input IEventList consumption (note delivery proof) ─────────────────────

#[repr(C)]
#[derive(Clone, Copy)]
struct NoteOnEventPayload {{
    channel: i16,
    pitch: i16,
    tuning: f32,
    velocity: f32,
    length: i32,
    note_id: i32,
}}

#[repr(C)]
#[derive(Clone, Copy)]
union EventPayload {{
    note_on: NoteOnEventPayload,
    _size: [u64; 3],
}}

#[repr(C)]
#[derive(Clone, Copy)]
struct Vst3Event {{
    bus_index: i32,
    sample_offset: i32,
    ppq_position: f64,
    flags: u16,
    type_: u16,
    payload: EventPayload,
}}

#[repr(C)]
struct EventListVTable {{
    query_interface: unsafe extern "C" fn(*mut c_void, *const Tuid, *mut *mut c_void) -> Tresult,
    add_ref: unsafe extern "C" fn(*mut c_void) -> u32,
    release: unsafe extern "C" fn(*mut c_void) -> u32,
    get_event_count: unsafe extern "C" fn(*mut c_void) -> i32,
    get_event: unsafe extern "C" fn(*mut c_void, i32, *mut Vst3Event) -> Tresult,
    add_event: unsafe extern "C" fn(*mut c_void, *mut Vst3Event) -> Tresult,
}}

/// Gather note events as gain steps: NOTE_ON (type 0) → gain = velocity at
/// its sample offset, NOTE_OFF (type 1) → gain = 0.0 at its sample offset —
/// making delivered notes AND their offsets audible in the output.
unsafe fn gather_note_steps(
    events: *mut c_void,
    steps: &mut [(i32, f32); GAIN_STEP_CAPACITY],
    step_count: &mut usize,
) {{
    if events.is_null() {{
        return;
    }}
    let list_vtable = *(events as *mut *const EventListVTable);
    let count = ((*list_vtable).get_event_count)(events);
    for index in 0..count {{
        let mut event = std::mem::MaybeUninit::<Vst3Event>::zeroed();
        if ((*list_vtable).get_event)(events, index, event.as_mut_ptr()) != K_RESULT_OK {{
            continue;
        }}
        let event = event.assume_init();
        if *step_count == GAIN_STEP_CAPACITY {{
            break;
        }}
        match event.type_ {{
            0 => {{
                steps[*step_count] = (event.sample_offset, event.payload.note_on.velocity);
                *step_count += 1;
            }}
            1 => {{
                steps[*step_count] = (event.sample_offset, 0.0);
                *step_count += 1;
            }}
            _ => {{}}
        }}
    }}
}}

/// Real audio processing: output = input × the LIVE Gain on every channel
/// of the main bus pair. The gain starts at the stored value and follows
/// the block's gathered `(offset, gain)` steps from their sample offsets
/// (param points, IMidiMapping CC points, and note events all land here);
/// the final step persists into later blocks.
unsafe extern "C" fn processor_process(_this: *mut c_void, data: *mut ProcessData) -> Tresult {{
    if data.is_null() {{
        return K_RESULT_FALSE;
    }}
    let data = &*data;
    let mut gain_steps = [(0i32, 0f32); GAIN_STEP_CAPACITY];
    let mut step_count = 0usize;
    gather_parameter_steps(data.input_parameter_changes, &mut gain_steps, &mut step_count);
    gather_note_steps(data.input_events, &mut gain_steps, &mut step_count);
    gain_steps[..step_count].sort_by_key(|step| step.0);
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
        let mut gain = f32::from_bits(GAIN_BITS.load(std::sync::atomic::Ordering::SeqCst));
        let mut next_step = 0usize;
        for frame in 0..frames {{
            while next_step < step_count && gain_steps[next_step].0 as usize <= frame {{
                gain = gain_steps[next_step].1;
                next_step += 1;
            }}
            *dest.add(frame) = *source.add(frame) * gain;
        }}
    }}
    if step_count > 0 {{
        GAIN_BITS.store(
            gain_steps[step_count - 1].1.to_bits(),
            std::sync::atomic::Ordering::SeqCst,
        );
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
    name: *const c_char,
) -> *mut c_void {{
    // Editor views only, per spec; the returned object is static so the
    // host's create/release probe and open/close cycles are all no-ops.
    if name.is_null() {{
        return ptr::null_mut();
    }}
    let mut len = 0usize;
    while *name.add(len) != 0 {{
        len += 1;
    }}
    let requested = std::slice::from_raw_parts(name as *const u8, len);
    if requested == b"editor" {{
        view_object()
    }} else {{
        ptr::null_mut()
    }}
}}

// ── Minimal IPlugView (offscreen bookkeeping, g12.024) ──────────────────────

#[repr(C)]
struct ViewRect {{
    left: i32,
    top: i32,
    right: i32,
    bottom: i32,
}}

#[repr(C)]
struct PlugViewVTable {{
    query_interface: unsafe extern "C" fn(*mut c_void, *const Tuid, *mut *mut c_void) -> Tresult,
    add_ref: unsafe extern "C" fn(*mut c_void) -> u32,
    release: unsafe extern "C" fn(*mut c_void) -> u32,
    is_platform_type_supported: unsafe extern "C" fn(*mut c_void, *const c_char) -> Tresult,
    attached: unsafe extern "C" fn(*mut c_void, *mut c_void, *const c_char) -> Tresult,
    removed: unsafe extern "C" fn(*mut c_void) -> Tresult,
    on_wheel: unsafe extern "C" fn(*mut c_void, f32) -> Tresult,
    on_key_down: unsafe extern "C" fn(*mut c_void, u16, i16, i16) -> Tresult,
    on_key_up: unsafe extern "C" fn(*mut c_void, u16, i16, i16) -> Tresult,
    get_size: unsafe extern "C" fn(*mut c_void, *mut ViewRect) -> Tresult,
    on_size: unsafe extern "C" fn(*mut c_void, *mut ViewRect) -> Tresult,
    on_focus: unsafe extern "C" fn(*mut c_void, u8) -> Tresult,
    set_frame: unsafe extern "C" fn(*mut c_void, *mut c_void) -> Tresult,
    can_resize: unsafe extern "C" fn(*mut c_void) -> Tresult,
    check_size_constraint: unsafe extern "C" fn(*mut c_void, *mut ViewRect) -> Tresult,
}}

#[repr(C)]
struct PlugFrameVTable {{
    query_interface: unsafe extern "C" fn(*mut c_void, *const Tuid, *mut *mut c_void) -> Tresult,
    add_ref: unsafe extern "C" fn(*mut c_void) -> u32,
    release: unsafe extern "C" fn(*mut c_void) -> u32,
    resize_view: unsafe extern "C" fn(*mut c_void, *mut c_void, *mut ViewRect) -> Tresult,
}}

#[repr(C)]
struct FixtureView {{
    vtable: *const PlugViewVTable,
}}

unsafe impl Sync for FixtureView {{}}

static VIEW_VTABLE: PlugViewVTable = PlugViewVTable {{
    query_interface: view_query_interface,
    add_ref: no_op_add_ref,
    release: no_op_release,
    is_platform_type_supported: view_is_platform_type_supported,
    attached: view_attached,
    removed: view_removed,
    on_wheel: view_on_wheel,
    on_key_down: view_on_key,
    on_key_up: view_on_key,
    get_size: view_get_size,
    on_size: view_on_size,
    on_focus: view_on_focus,
    set_frame: view_set_frame,
    can_resize: view_can_resize,
    check_size_constraint: view_check_size_constraint,
}};

static FIXTURE_VIEW: FixtureView = FixtureView {{
    vtable: &VIEW_VTABLE,
}};

/// Offscreen view bookkeeping: parent handle recorded, never dereferenced.
static VIEW_ATTACHED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
static VIEW_WIDTH: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new({view_initial_width});
static VIEW_HEIGHT: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new({view_initial_height});
static VIEW_FRAME: std::sync::atomic::AtomicPtr<c_void> =
    std::sync::atomic::AtomicPtr::new(ptr::null_mut());

fn view_object() -> *mut c_void {{
    &FIXTURE_VIEW as *const FixtureView as *mut c_void
}}

unsafe extern "C" fn view_query_interface(
    this: *mut c_void,
    iid: *const Tuid,
    out: *mut *mut c_void,
) -> Tresult {{
    if out.is_null() {{
        return K_NO_INTERFACE;
    }}
    if !iid.is_null() && (*iid == FUNKNOWN_IID || *iid == IPLUG_VIEW_IID) {{
        *out = this;
        return K_RESULT_OK;
    }}
    *out = ptr::null_mut();
    K_NO_INTERFACE
}}

unsafe extern "C" fn view_is_platform_type_supported(
    _this: *mut c_void,
    _platform_type: *const c_char,
) -> Tresult {{
    // Every platform type: the handle is bookkeeping, never dereferenced.
    K_RESULT_OK
}}

unsafe extern "C" fn view_attached(
    this: *mut c_void,
    parent: *mut c_void,
    _platform_type: *const c_char,
) -> Tresult {{
    if parent.is_null() {{
        return K_RESULT_FALSE;
    }}
    VIEW_WIDTH.store({view_initial_width}, std::sync::atomic::Ordering::SeqCst);
    VIEW_HEIGHT.store({view_initial_height}, std::sync::atomic::Ordering::SeqCst);
    VIEW_ATTACHED.store(true, std::sync::atomic::Ordering::SeqCst);
    // Exercise the host-callback path: ask the host frame for a resize.
    let frame = VIEW_FRAME.load(std::sync::atomic::Ordering::SeqCst);
    if !frame.is_null() {{
        let frame_vtable = *(frame as *mut *const PlugFrameVTable);
        let mut rect = ViewRect {{
            left: 0,
            top: 0,
            right: {view_request_width},
            bottom: {view_request_height},
        }};
        let _ = ((*frame_vtable).resize_view)(frame, this, &mut rect);
    }}
    K_RESULT_OK
}}

unsafe extern "C" fn view_removed(_this: *mut c_void) -> Tresult {{
    VIEW_ATTACHED.store(false, std::sync::atomic::Ordering::SeqCst);
    K_RESULT_OK
}}

unsafe extern "C" fn view_on_wheel(_this: *mut c_void, _distance: f32) -> Tresult {{
    K_RESULT_FALSE
}}

unsafe extern "C" fn view_on_key(
    _this: *mut c_void,
    _key: u16,
    _key_code: i16,
    _modifiers: i16,
) -> Tresult {{
    K_RESULT_FALSE
}}

unsafe extern "C" fn view_get_size(_this: *mut c_void, size: *mut ViewRect) -> Tresult {{
    if size.is_null() {{
        return K_RESULT_FALSE;
    }}
    let size = &mut *size;
    size.left = 0;
    size.top = 0;
    size.right = VIEW_WIDTH.load(std::sync::atomic::Ordering::SeqCst) as i32;
    size.bottom = VIEW_HEIGHT.load(std::sync::atomic::Ordering::SeqCst) as i32;
    K_RESULT_OK
}}

unsafe extern "C" fn view_on_size(_this: *mut c_void, new_size: *mut ViewRect) -> Tresult {{
    if new_size.is_null() {{
        return K_RESULT_FALSE;
    }}
    let new_size = &*new_size;
    VIEW_WIDTH.store(
        (new_size.right - new_size.left).max(0) as u32,
        std::sync::atomic::Ordering::SeqCst,
    );
    VIEW_HEIGHT.store(
        (new_size.bottom - new_size.top).max(0) as u32,
        std::sync::atomic::Ordering::SeqCst,
    );
    K_RESULT_OK
}}

unsafe extern "C" fn view_on_focus(_this: *mut c_void, _state: u8) -> Tresult {{
    K_RESULT_OK
}}

unsafe extern "C" fn view_set_frame(_this: *mut c_void, frame: *mut c_void) -> Tresult {{
    VIEW_FRAME.store(frame, std::sync::atomic::Ordering::SeqCst);
    K_RESULT_OK
}}

unsafe extern "C" fn view_can_resize(_this: *mut c_void) -> Tresult {{
    K_RESULT_OK
}}

unsafe extern "C" fn view_check_size_constraint(
    _this: *mut c_void,
    rect: *mut ViewRect,
) -> Tresult {{
    // No constraints: any proposed rect is accepted unchanged.
    if rect.is_null() {{ K_RESULT_FALSE }} else {{ K_RESULT_OK }}
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
#[cfg(target_os = "macos")]
pub unsafe extern "C" fn bundleEntry(bundle_ref: *mut c_void) -> bool {{ !bundle_ref.is_null() }}
#[cfg(not(target_os = "macos"))]
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
        view_initial_width = VST3_FIXTURE_VIEW_INITIAL_SIZE.0,
        view_initial_height = VST3_FIXTURE_VIEW_INITIAL_SIZE.1,
        view_request_width = VST3_FIXTURE_VIEW_REQUESTED_SIZE.0,
        view_request_height = VST3_FIXTURE_VIEW_REQUESTED_SIZE.1,
    )
}
