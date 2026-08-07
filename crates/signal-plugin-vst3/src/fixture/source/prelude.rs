use super::super::VST3_FIXTURE_GAIN;

pub(crate) fn prelude_fragment(plugin_name: &str) -> String {
    format!(
        r#"//! rustc-compiled VST3 fixture module: single-component stereo gain effect.
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
const IEDIT_CONTROLLER_IID: Tuid = tuid_from_uid(0xDCD7BBE3, 0x7742448D, 0xA874AACC, 0x979C759E);
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
}}"#,
        gain = VST3_FIXTURE_GAIN,
        plugin_name = plugin_name,
    )
}
