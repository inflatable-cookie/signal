//! Real compiled LV2 fixture for tests: bundle writer + rustc harness (the
//! LV2 mirror of the CLAP and VST3 fixture generators, merged).
//!
//! The fixture is an actual `.lv2` bundle: a real `manifest.ttl`
//! (`a lv2:Plugin`, `lv2:binary`, `rdfs:seeAlso`), a plugin data TTL with
//! the full port model (stereo audio in/out plus Gain and Bypass control
//! ports with TTL defaults), and a rustc-compiled cdylib exporting
//! `lv2_descriptor`, so discovery and hosting tests exercise the genuine
//! TTL-parse/dlopen/descriptor-walk path. Its `run()` is real: output =
//! input × the Gain control port's value. The Gain port ships a NON-UNITY
//! `lv2:default` ([`LV2_FIXTURE_GAIN`] = 0.5), so processing is provable
//! from defaults alone — phase 1 has no param-set wire (wet = dry × 0.5).
//! The fixture's `instantiate` also REQUIRES a working `urid:map` feature
//! (declared `lv2:requiredFeature` in the TTL) and returns NULL unless the
//! host's map callback interns a URI to a nonzero URID, proving the
//! features array really arrives.
//!
//! Bundle directory names are derived space-free (v1 wire limitation —
//! library paths ride a whitespace-split stdio protocol).
//!
//! Shared across crates (the sandbox broker's integration tests compile
//! the same fixture), hence public but hidden from the documented API.

use std::{
    path::{Path, PathBuf},
    process::Command,
};

/// Fixed linear gain the fixture's Gain control port defaults to (and thus
/// the gain `run()` applies until a param-set wire exists).
pub const LV2_FIXTURE_GAIN: f32 = 0.5;

/// `lv2:index` of the fixture's Gain control port (its parameter_id).
pub const LV2_FIXTURE_GAIN_PORT_INDEX: u32 = 4;

/// `lv2:index` of the fixture's Bypass control port (its parameter_id).
pub const LV2_FIXTURE_BYPASS_PORT_INDEX: u32 = 5;

/// Returns `true` when a `rustc` binary is invocable (fixture tests skip
/// gracefully when it is not).
pub fn rustc_available() -> bool {
    Command::new("rustc")
        .arg("--version")
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

/// Compile the stereo-effect fixture bundle into `directory`, returning
/// the `.lv2` bundle root. `plugin_uri` is the LV2 identity (the hosting
/// load key); `plugin_name` derives the space-free bundle/module name.
pub fn compile_lv2_fixture(
    directory: &Path,
    plugin_uri: &str,
    plugin_name: &str,
) -> Result<PathBuf, String> {
    write_lv2_fixture_bundle(directory, plugin_uri, plugin_name, false)
}

/// Compile the atom-input variant: identical to the stereo fixture plus a
/// REQUIRED atom input port (an instrument-shaped plugin), which the
/// phase-1 stereo gate must reject with `layout_unsupported`.
pub fn compile_lv2_atom_fixture(
    directory: &Path,
    plugin_uri: &str,
    plugin_name: &str,
) -> Result<PathBuf, String> {
    write_lv2_fixture_bundle(directory, plugin_uri, plugin_name, true)
}

fn module_extension() -> &'static str {
    if cfg!(target_os = "macos") {
        "dylib"
    } else if cfg!(target_os = "windows") {
        "dll"
    } else {
        "so"
    }
}

fn write_lv2_fixture_bundle(
    directory: &Path,
    plugin_uri: &str,
    plugin_name: &str,
    with_atom_input: bool,
) -> Result<PathBuf, String> {
    let module_name = plugin_name.to_lowercase().replace(' ', "-");
    let bundle_root = directory.join(format!("{module_name}.lv2"));
    std::fs::create_dir_all(&bundle_root)
        .map_err(|error| format!("fixture bundle dir create failed: {error}"))?;

    let binary_name = format!("{module_name}.{}", module_extension());
    std::fs::write(
        bundle_root.join("manifest.ttl"),
        fixture_manifest_ttl(plugin_uri, &module_name, &binary_name),
    )
    .map_err(|error| format!("fixture manifest.ttl write failed: {error}"))?;
    std::fs::write(
        bundle_root.join(format!("{module_name}.ttl")),
        fixture_plugin_ttl(plugin_uri, plugin_name, with_atom_input),
    )
    .map_err(|error| format!("fixture plugin ttl write failed: {error}"))?;

    let source_path = directory.join(format!("{module_name}-lv2-fixture.rs"));
    std::fs::write(&source_path, lv2_fixture_source(plugin_uri))
        .map_err(|error| format!("fixture source write failed: {error}"))?;
    let module_path = bundle_root.join(&binary_name);
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
            "lv2 fixture compilation failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    Ok(bundle_root)
}

/// `manifest.ttl`: identity, binary, and the seeAlso split — the port
/// model deliberately lives in the separate plugin TTL so tests exercise
/// the rdfs:seeAlso chase.
fn fixture_manifest_ttl(plugin_uri: &str, module_name: &str, binary_name: &str) -> String {
    format!(
        "@prefix lv2:  <http://lv2plug.in/ns/lv2core#> .\n\
         @prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .\n\
         \n\
         <{plugin_uri}>\n\
         \ta lv2:Plugin ;\n\
         \tlv2:binary <{binary_name}> ;\n\
         \trdfs:seeAlso <{module_name}.ttl> .\n",
    )
}

fn fixture_plugin_ttl(plugin_uri: &str, plugin_name: &str, with_atom_input: bool) -> String {
    let atom_port = if with_atom_input {
        "\t, [\n\
         \t\ta atom:AtomPort , lv2:InputPort ;\n\
         \t\tlv2:index 6 ;\n\
         \t\tlv2:symbol \"events_in\" ;\n\
         \t\tlv2:name \"Events In\" ;\n\
         \t]\n"
    } else {
        ""
    };
    format!(
        "@prefix lv2:  <http://lv2plug.in/ns/lv2core#> .\n\
         @prefix doap: <http://usefulinc.com/ns/doap#> .\n\
         @prefix atom: <http://lv2plug.in/ns/ext/atom#> .\n\
         @prefix units: <http://lv2plug.in/ns/extensions/units#> .\n\
         \n\
         <{plugin_uri}>\n\
         \ta lv2:Plugin ;\n\
         \tdoap:name \"{plugin_name}\" ;\n\
         \tlv2:requiredFeature <http://lv2plug.in/ns/ext/urid#map> ;\n\
         \tlv2:optionalFeature lv2:hardRTCapable ;\n\
         \tlv2:port [\n\
         \t\ta lv2:AudioPort , lv2:InputPort ;\n\
         \t\tlv2:index 0 ;\n\
         \t\tlv2:symbol \"in_l\" ;\n\
         \t\tlv2:name \"In Left\" ;\n\
         \t] , [\n\
         \t\ta lv2:AudioPort , lv2:InputPort ;\n\
         \t\tlv2:index 1 ;\n\
         \t\tlv2:symbol \"in_r\" ;\n\
         \t\tlv2:name \"In Right\" ;\n\
         \t] , [\n\
         \t\ta lv2:AudioPort , lv2:OutputPort ;\n\
         \t\tlv2:index 2 ;\n\
         \t\tlv2:symbol \"out_l\" ;\n\
         \t\tlv2:name \"Out Left\" ;\n\
         \t] , [\n\
         \t\ta lv2:AudioPort , lv2:OutputPort ;\n\
         \t\tlv2:index 3 ;\n\
         \t\tlv2:symbol \"out_r\" ;\n\
         \t\tlv2:name \"Out Right\" ;\n\
         \t] , [\n\
         \t\ta lv2:ControlPort , lv2:InputPort ;\n\
         \t\tlv2:index {gain_index} ;\n\
         \t\tlv2:symbol \"gain\" ;\n\
         \t\tlv2:name \"Gain\" ;\n\
         \t\tlv2:default {gain} ;\n\
         \t\tlv2:minimum 0.0 ;\n\
         \t\tlv2:maximum 1.0 ;\n\
         \t\tunits:unit units:coef ;\n\
         \t] , [\n\
         \t\ta lv2:ControlPort , lv2:InputPort ;\n\
         \t\tlv2:index {bypass_index} ;\n\
         \t\tlv2:symbol \"bypass\" ;\n\
         \t\tlv2:name \"Bypass\" ;\n\
         \t\tlv2:default 0.0 ;\n\
         \t\tlv2:minimum 0.0 ;\n\
         \t\tlv2:maximum 1.0 ;\n\
         \t\tlv2:portProperty lv2:toggled ;\n\
         \t\tlv2:designation lv2:enabled ;\n\
         \t]\n\
         {atom_port}\
         \t.\n",
        gain = LV2_FIXTURE_GAIN,
        gain_index = LV2_FIXTURE_GAIN_PORT_INDEX,
        bypass_index = LV2_FIXTURE_BYPASS_PORT_INDEX,
    )
}

/// Full Rust source of the fixture cdylib.
pub fn lv2_fixture_source(plugin_uri: &str) -> String {
    format!(
        r#"
use std::ffi::{{c_char, c_void, CStr}};
use std::ptr;

#[repr(C)]
pub struct Lv2Feature {{
    pub uri: *const c_char,
    pub data: *mut c_void,
}}

#[repr(C)]
pub struct Lv2UridMap {{
    pub handle: *mut c_void,
    pub map: unsafe extern "C" fn(*mut c_void, *const c_char) -> u32,
}}

#[repr(C)]
pub struct Lv2Descriptor {{
    pub uri: *const c_char,
    pub instantiate: Option<
        unsafe extern "C" fn(
            *const Lv2Descriptor,
            f64,
            *const c_char,
            *const *const Lv2Feature,
        ) -> *mut c_void,
    >,
    pub connect_port: Option<unsafe extern "C" fn(*mut c_void, u32, *mut c_void)>,
    pub activate: Option<unsafe extern "C" fn(*mut c_void)>,
    pub run: Option<unsafe extern "C" fn(*mut c_void, u32)>,
    pub deactivate: Option<unsafe extern "C" fn(*mut c_void)>,
    pub cleanup: Option<unsafe extern "C" fn(*mut c_void)>,
    pub extension_data: Option<unsafe extern "C" fn(*const c_char) -> *const c_void>,
}}

unsafe impl Sync for Lv2Descriptor {{}}

static PLUGIN_URI: &[u8] = concat!("{plugin_uri}", "\0").as_bytes();
static URID_MAP_URI: &[u8] = b"http://lv2plug.in/ns/ext/urid#map\0";
static PROOF_URI: &[u8] = b"https://signal.dev/fixtures/lv2#proof\0";

/// Port connection table: in_l, in_r, out_l, out_r, gain, bypass (+ the
/// optional atom port slot in the variant bundle, never processed).
struct Instance {{
    ports: [*mut c_void; 7],
}}

/// `instantiate` REQUIRES a working urid:map feature: it walks the
/// NULL-terminated array, calls the host's map callback, and refuses to
/// instantiate unless a URI interned to a nonzero URID — proving the
/// host's feature plumbing end to end.
unsafe extern "C" fn instantiate(
    _descriptor: *const Lv2Descriptor,
    _sample_rate: f64,
    _bundle_path: *const c_char,
    features: *const *const Lv2Feature,
) -> *mut c_void {{
    if features.is_null() {{
        return ptr::null_mut();
    }}
    let mut index = 0usize;
    loop {{
        let feature = *features.add(index);
        if feature.is_null() {{
            return ptr::null_mut();
        }}
        if !(*feature).uri.is_null()
            && CStr::from_ptr((*feature).uri).to_bytes_with_nul() == URID_MAP_URI
        {{
            let map = (*feature).data as *const Lv2UridMap;
            if map.is_null() {{
                return ptr::null_mut();
            }}
            let urid = ((*map).map)((*map).handle, PROOF_URI.as_ptr() as *const c_char);
            if urid == 0 {{
                return ptr::null_mut();
            }}
            break;
        }}
        index += 1;
    }}
    Box::into_raw(Box::new(Instance {{
        ports: [ptr::null_mut(); 7],
    }})) as *mut c_void
}}

unsafe extern "C" fn connect_port(handle: *mut c_void, port: u32, data: *mut c_void) {{
    if handle.is_null() {{
        return;
    }}
    let instance = &mut *(handle as *mut Instance);
    if (port as usize) < instance.ports.len() {{
        instance.ports[port as usize] = data;
    }}
}}

unsafe extern "C" fn activate(_handle: *mut c_void) {{}}

/// Real audio processing: out = in × the Gain control port's value on both
/// channels. The gain port defaults to a non-unity value in the TTL, so a
/// default-parameter run audibly (and byte-exactly) halves the input.
unsafe extern "C" fn run(handle: *mut c_void, sample_count: u32) {{
    if handle.is_null() {{
        return;
    }}
    let instance = &*(handle as *mut Instance);
    let [in_l, in_r, out_l, out_r, gain, _bypass, _atom] = instance.ports;
    if in_l.is_null() || in_r.is_null() || out_l.is_null() || out_r.is_null() {{
        return;
    }}
    let gain_value = if gain.is_null() {{
        1.0f32
    }} else {{
        *(gain as *const f32)
    }};
    for frame in 0..sample_count as usize {{
        *(out_l as *mut f32).add(frame) = *(in_l as *const f32).add(frame) * gain_value;
        *(out_r as *mut f32).add(frame) = *(in_r as *const f32).add(frame) * gain_value;
    }}
}}

unsafe extern "C" fn deactivate(_handle: *mut c_void) {{}}

unsafe extern "C" fn cleanup(handle: *mut c_void) {{
    if !handle.is_null() {{
        drop(Box::from_raw(handle as *mut Instance));
    }}
}}

unsafe extern "C" fn extension_data(_uri: *const c_char) -> *const c_void {{
    ptr::null()
}}

static DESCRIPTOR: Lv2Descriptor = Lv2Descriptor {{
    uri: PLUGIN_URI.as_ptr() as *const c_char,
    instantiate: Some(instantiate),
    connect_port: Some(connect_port),
    activate: Some(activate),
    run: Some(run),
    deactivate: Some(deactivate),
    cleanup: Some(cleanup),
    extension_data: Some(extension_data),
}};

#[unsafe(no_mangle)]
pub extern "C" fn lv2_descriptor(index: u32) -> *const Lv2Descriptor {{
    if index == 0 {{
        &DESCRIPTOR
    }} else {{
        ptr::null()
    }}
}}
"#,
        plugin_uri = plugin_uri,
    )
}
