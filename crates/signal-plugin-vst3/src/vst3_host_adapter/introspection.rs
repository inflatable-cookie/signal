use super::Vst3HostPlatform;
use signal_plugin::{
    PluginDescriptor, PluginFeature, PluginFormat, PluginIoLayout, PluginLifecycleContract,
    PluginProcessingContract, PluginStateContract,
};
use std::{
    ffi::{c_char, c_void},
    fs, io,
    path::{Path, PathBuf},
};

pub(crate) const VST3_MODULEINFO_FILE: &str = "moduleinfo.json";

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Vst3ModuleMetadata {
    pub(crate) plugin_type_id: String,
    pub(crate) class_id: String,
    pub(crate) controller_class_id: Option<String>,
    pub(crate) category: String,
    pub(crate) vendor: String,
    pub(crate) name: String,
    pub(crate) version: String,
    pub(crate) audio_inputs: u16,
    pub(crate) audio_outputs: u16,
    pub(crate) midi_inputs: u16,
    pub(crate) midi_outputs: u16,
    pub(crate) features: Vec<PluginFeature>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Vst3FactoryClass {
    pub(crate) role: Vst3FactoryClassRole,
    pub(crate) class_id: String,
    pub(crate) category: String,
    pub(crate) name: String,
    pub(crate) vendor: Option<String>,
    pub(crate) version: Option<String>,
    pub(crate) subcategories: Vec<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Vst3FactoryClassRole {
    Component,
    Controller,
    Other,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Vst3BundleSnapshot {
    pub(crate) plugins: Vec<Vst3ModuleMetadata>,
    pub(crate) factory_classes: Vec<Vst3FactoryClass>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct Vst3BundleInfo {
    bundle_identifier: Option<String>,
    bundle_name: Option<String>,
    executable_name: Option<String>,
    version: Option<String>,
    signal_plugin_type_id: Option<String>,
    signal_audio_inputs: Option<u16>,
    signal_audio_outputs: Option<u16>,
    signal_midi_inputs: Option<u16>,
    signal_midi_outputs: Option<u16>,
    signal_features: Option<Vec<PluginFeature>>,
}

#[derive(serde::Deserialize)]
struct ModuleInfoDocument {
    #[serde(rename = "Factory Info")]
    factory_info: Option<ModuleFactoryInfo>,
    #[serde(rename = "Classes", default)]
    classes: Vec<ModuleInfoClass>,
}

#[derive(serde::Deserialize)]
struct ModuleFactoryInfo {
    #[serde(rename = "Vendor")]
    vendor: Option<String>,
}

#[derive(serde::Deserialize)]
struct ModuleInfoClass {
    #[serde(rename = "CID")]
    cid: String,
    #[serde(rename = "Category")]
    category: String,
    #[serde(rename = "Name")]
    name: String,
    #[serde(rename = "Vendor")]
    vendor: Option<String>,
    #[serde(rename = "Version")]
    version: Option<String>,
    #[serde(rename = "Sub Categories", default)]
    subcategories: Vec<String>,
}

#[repr(C)]
struct RawPluginFactory {
    vtable: *const PluginFactoryVTable,
}

#[repr(C)]
struct PluginFactoryVTable {
    query_interface: unsafe extern "C" fn(*mut c_void, *const c_void, *mut *mut c_void) -> i32,
    add_ref: unsafe extern "C" fn(*mut c_void) -> u32,
    release: unsafe extern "C" fn(*mut c_void) -> u32,
    get_factory_info: unsafe extern "C" fn(*mut c_void, *mut PFactoryInfo) -> i32,
    count_classes: unsafe extern "C" fn(*mut c_void) -> i32,
    get_class_info: unsafe extern "C" fn(*mut c_void, i32, *mut PClassInfo) -> i32,
    create_instance:
        unsafe extern "C" fn(*mut c_void, *const u8, *const c_void, *mut *mut c_void) -> i32,
}

#[repr(C)]
struct PFactoryInfo {
    vendor: [c_char; 64],
    url: [c_char; 256],
    email: [c_char; 128],
    flags: i32,
}

#[repr(C)]
struct PClassInfo {
    cid: [u8; 16],
    cardinality: i32,
    category: [c_char; 32],
    name: [c_char; 64],
}

type EntryProc = unsafe extern "C" fn(*mut c_void) -> bool;
type ExitProc = unsafe extern "C" fn();
type GetPluginFactoryProc = unsafe extern "C" fn() -> *mut c_void;

pub(crate) fn read_vst3_bundle_snapshot(
    bundle_root: &Path,
    platform: Vst3HostPlatform,
) -> io::Result<Vst3BundleSnapshot> {
    let bundle = read_vst3_bundle_info(bundle_root)?;
    let (factory_vendor, factory_classes) = read_vst3_factory_snapshot(bundle_root, platform)?;
    let component_classes = factory_classes
        .iter()
        .filter(|class| class.role == Vst3FactoryClassRole::Component)
        .cloned()
        .collect::<Vec<_>>();
    if component_classes.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "missing VST3 component classes",
        ));
    }

    let component_count = component_classes.len();
    let plugins = component_classes
        .iter()
        .map(|component| {
            let controller = match_vst3_controller(component, &factory_classes, component_count);
            let is_instrument = class_is_instrument(component);
            let plugin_type_id = if component_count == 1 {
                bundle
                    .signal_plugin_type_id
                    .clone()
                    .unwrap_or_else(|| derive_plugin_type_id(&bundle, component))
            } else {
                derive_plugin_type_id(&bundle, component)
            };
            let io_layout = derive_io_layout(&bundle, is_instrument);
            let features = bundle
                .signal_features
                .clone()
                .unwrap_or_else(|| default_features(is_instrument));
            Ok(Vst3ModuleMetadata {
                plugin_type_id,
                class_id: component.class_id.clone(),
                controller_class_id: controller.as_ref().map(|class| class.class_id.clone()),
                category: component.category.clone(),
                vendor: component
                    .vendor()
                    .unwrap_or_else(|| fallback_vendor(&bundle, factory_vendor.as_deref())),
                name: component.name.clone(),
                version: component.version().unwrap_or_else(|| {
                    bundle
                        .version
                        .clone()
                        .unwrap_or_else(|| "0.1.0".to_string())
                }),
                audio_inputs: io_layout.audio_inputs,
                audio_outputs: io_layout.audio_outputs,
                midi_inputs: io_layout.midi_inputs,
                midi_outputs: io_layout.midi_outputs,
                features,
            })
        })
        .collect::<io::Result<Vec<_>>>()?;

    Ok(Vst3BundleSnapshot {
        plugins,
        factory_classes,
    })
}

pub(crate) fn metadata_io_layout(metadata: &Vst3ModuleMetadata) -> PluginIoLayout {
    PluginIoLayout {
        audio_inputs: metadata.audio_inputs,
        audio_outputs: metadata.audio_outputs,
        midi_inputs: metadata.midi_inputs,
        midi_outputs: metadata.midi_outputs,
    }
}

pub(crate) fn metadata_descriptor(metadata: &Vst3ModuleMetadata) -> PluginDescriptor {
    let io_layout = metadata_io_layout(metadata);
    let mut descriptor = PluginDescriptor::new(
        metadata.plugin_type_id.clone(),
        metadata.vendor.clone(),
        metadata.name.clone(),
        PluginFormat::Vst3,
    )
    .with_version(metadata.version.as_str())
    .with_audio_buses(io_layout.main_audio_buses())
    // Scan-time parameter inventory is intentionally EMPTY: real inventories
    // arrive at load time via IEditController (g11.031, mirrors CLAP).
    .with_parameters(Vec::new())
    .with_state_contract(PluginStateContract {
        supports_snapshot: true,
        supports_reset: true,
        supports_bypass: true,
        exposes_latency: true,
        exposes_tail: true,
    })
    .with_lifecycle_contract(PluginLifecycleContract {
        requires_main_thread_for_state: false,
        supports_prepare: true,
        supports_activate: true,
        supports_reset_while_active: true,
    })
    .with_processing_contract(PluginProcessingContract {
        max_block_frames: 2048,
        sample_accurate_automation: true,
        accepts_midi: metadata.midi_inputs > 0,
        accepts_note_events: metadata.midi_inputs > 0,
        supports_note_expression: metadata.midi_inputs > 0,
        produces_midi: metadata.midi_outputs > 0,
        silence_aware: true,
    });
    descriptor.features = metadata.features.clone();
    descriptor
}

fn read_vst3_bundle_info(bundle_root: &Path) -> io::Result<Vst3BundleInfo> {
    let mut bundle = Vst3BundleInfo {
        bundle_name: bundle_root
            .file_stem()
            .and_then(|stem| stem.to_str())
            .map(str::to_string),
        ..Vst3BundleInfo::default()
    };
    let Some(info_plist_path) = candidate_info_plist_paths(bundle_root)
        .into_iter()
        .find(|path| path.is_file())
    else {
        return Ok(bundle);
    };
    let value = plist::Value::from_file(&info_plist_path).map_err(plist_to_io_error)?;
    let Some(dict) = value.into_dictionary() else {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "VST3 Info.plist should be a dictionary",
        ));
    };

    bundle.bundle_identifier = plist_string(&dict, "CFBundleIdentifier");
    bundle.bundle_name = plist_string(&dict, "CFBundleName")
        .or_else(|| plist_string(&dict, "CFBundleDisplayName"))
        .or(bundle.bundle_name);
    bundle.executable_name = plist_string(&dict, "CFBundleExecutable");
    bundle.version = plist_string(&dict, "CFBundleShortVersionString")
        .or_else(|| plist_string(&dict, "CFBundleVersion"));
    bundle.signal_plugin_type_id = plist_string(&dict, "SignalPluginTypeId");
    bundle.signal_audio_inputs = plist_u16(&dict, "SignalAudioInputs");
    bundle.signal_audio_outputs = plist_u16(&dict, "SignalAudioOutputs");
    bundle.signal_midi_inputs = plist_u16(&dict, "SignalMidiInputs");
    bundle.signal_midi_outputs = plist_u16(&dict, "SignalMidiOutputs");
    bundle.signal_features = plist_string_array(&dict, "SignalFeatures")
        .map(|features| parse_feature_list(&features.join(",")))
        .transpose()?;

    Ok(bundle)
}

fn read_vst3_factory_snapshot(
    bundle_root: &Path,
    platform: Vst3HostPlatform,
) -> io::Result<(Option<String>, Vec<Vst3FactoryClass>)> {
    if let Some(moduleinfo_path) = candidate_moduleinfo_paths(bundle_root)
        .into_iter()
        .find(|path| path.is_file())
    {
        let document = json5::from_str::<ModuleInfoDocument>(&fs::read_to_string(moduleinfo_path)?)
            .map_err(|error| {
                io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("invalid VST3 moduleinfo.json: {error}"),
                )
            })?;
        let vendor = document
            .factory_info
            .and_then(|factory| factory.vendor)
            .or_else(|| {
                document
                    .classes
                    .iter()
                    .find_map(|class| class.vendor.clone())
            });
        let classes = document
            .classes
            .into_iter()
            .map(|class| Vst3FactoryClass {
                role: role_from_category(&class.category),
                class_id: class.cid,
                category: class.category,
                name: class.name,
                vendor: class.vendor,
                version: class.version,
                subcategories: class.subcategories,
            })
            .collect::<Vec<_>>();
        if classes.is_empty() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "missing VST3 classes in moduleinfo.json",
            ));
        }
        return Ok((vendor, classes));
    }
    load_vst3_factory_classes_from_module(bundle_root, platform)
}

fn load_vst3_factory_classes_from_module(
    bundle_root: &Path,
    platform: Vst3HostPlatform,
) -> io::Result<(Option<String>, Vec<Vst3FactoryClass>)> {
    let module_path = resolve_module_binary_path(bundle_root, platform)?;
    let library = unsafe { libloading::Library::new(&module_path) }.map_err(libloading_to_io)?;

    unsafe {
        match platform {
            Vst3HostPlatform::MacOs => {
                if let Ok(entry) = library.get::<EntryProc>(b"bundleEntry\0") {
                    if !entry(std::ptr::null_mut()) {
                        return Err(io::Error::new(
                            io::ErrorKind::InvalidData,
                            "VST3 bundleEntry returned false",
                        ));
                    }
                }
            }
            Vst3HostPlatform::Linux => {
                if let Ok(entry) = library.get::<EntryProc>(b"ModuleEntry\0") {
                    if !entry(std::ptr::null_mut()) {
                        return Err(io::Error::new(
                            io::ErrorKind::InvalidData,
                            "VST3 ModuleEntry returned false",
                        ));
                    }
                }
            }
            Vst3HostPlatform::Windows => {
                if let Ok(entry) = library.get::<EntryProc>(b"InitDll\0") {
                    if !entry(std::ptr::null_mut()) {
                        return Err(io::Error::new(
                            io::ErrorKind::InvalidData,
                            "VST3 InitDll returned false",
                        ));
                    }
                }
            }
        }

        let get_plugin_factory = library
            .get::<GetPluginFactoryProc>(b"GetPluginFactory\0")
            .map_err(libloading_to_io)?;
        let factory_ptr = get_plugin_factory();
        if factory_ptr.is_null() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "VST3 GetPluginFactory returned null",
            ));
        }

        let factory = factory_ptr as *mut RawPluginFactory;
        let vtable = (*factory).vtable;
        if vtable.is_null() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "VST3 factory vtable was null",
            ));
        }

        let mut factory_info = PFactoryInfo {
            vendor: [0; 64],
            url: [0; 256],
            email: [0; 128],
            flags: 0,
        };
        let vendor = if ((*vtable).get_factory_info)(factory_ptr, &mut factory_info) == 0 {
            Some(c_char_array_to_string(&factory_info.vendor))
        } else {
            None
        };

        let class_count = ((*vtable).count_classes)(factory_ptr);
        if class_count <= 0 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "VST3 factory exposed no classes",
            ));
        }
        let mut classes = Vec::new();
        for index in 0..class_count {
            let mut class_info = PClassInfo {
                cid: [0; 16],
                cardinality: 0,
                category: [0; 32],
                name: [0; 64],
            };
            if ((*vtable).get_class_info)(factory_ptr, index, &mut class_info) != 0 {
                continue;
            }
            let category = c_char_array_to_string(&class_info.category);
            classes.push(Vst3FactoryClass {
                role: role_from_category(&category),
                class_id: bytes_to_upper_hex(&class_info.cid),
                category,
                name: c_char_array_to_string(&class_info.name),
                vendor: None,
                version: None,
                subcategories: Vec::new(),
            });
        }

        match platform {
            Vst3HostPlatform::MacOs => {
                if let Ok(exit) = library.get::<ExitProc>(b"bundleExit\0") {
                    exit();
                }
            }
            Vst3HostPlatform::Linux => {
                if let Ok(exit) = library.get::<ExitProc>(b"ModuleExit\0") {
                    exit();
                }
            }
            Vst3HostPlatform::Windows => {
                if let Ok(exit) = library.get::<ExitProc>(b"ExitDll\0") {
                    exit();
                }
            }
        }

        if classes.is_empty() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "VST3 factory exposed no readable classes",
            ));
        }
        Ok((vendor.filter(|value| !value.is_empty()), classes))
    }
}

fn candidate_info_plist_paths(bundle_root: &Path) -> Vec<PathBuf> {
    vec![bundle_root.join("Contents").join("Info.plist")]
}

fn candidate_moduleinfo_paths(bundle_root: &Path) -> Vec<PathBuf> {
    vec![
        bundle_root
            .join("Contents")
            .join("Resources")
            .join(VST3_MODULEINFO_FILE),
        bundle_root.join("Contents").join(VST3_MODULEINFO_FILE),
    ]
}

pub(crate) fn resolve_module_binary_path(
    bundle_root: &Path,
    platform: Vst3HostPlatform,
) -> io::Result<PathBuf> {
    let bundle = read_vst3_bundle_info(bundle_root)?;
    let bundle_stem = bundle_root
        .file_stem()
        .and_then(|stem| stem.to_str())
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "invalid VST3 bundle name"))?;
    let executable_name = bundle.executable_name.as_deref().unwrap_or(bundle_stem);
    let direct_candidates = match platform {
        Vst3HostPlatform::MacOs => vec![bundle_root
            .join("Contents")
            .join("MacOS")
            .join(executable_name)],
        Vst3HostPlatform::Linux => vec![
            bundle_root
                .join("Contents")
                .join("x86_64-linux")
                .join(format!("{executable_name}.so")),
            bundle_root
                .join("Contents")
                .join("aarch64-linux")
                .join(format!("{executable_name}.so")),
        ],
        Vst3HostPlatform::Windows => vec![
            bundle_root
                .join("Contents")
                .join("x86_64-win")
                .join(format!("{executable_name}.vst3")),
            bundle_root
                .join("Contents")
                .join("arm64-win")
                .join(format!("{executable_name}.vst3")),
        ],
    };
    if let Some(path) = direct_candidates.into_iter().find(|path| path.is_file()) {
        return Ok(path);
    }
    let search_root = bundle_root.join("Contents");
    let entries = fs::read_dir(&search_root).map_err(|error| {
        io::Error::new(
            error.kind(),
            format!(
                "failed to read VST3 bundle contents for module resolution: {}",
                search_root.display()
            ),
        )
    })?;
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            let Ok(children) = fs::read_dir(&path) else {
                continue;
            };
            for child in children.flatten() {
                let child_path = child.path();
                if child_path.is_file() {
                    return Ok(child_path);
                }
            }
        }
    }
    Err(io::Error::new(
        io::ErrorKind::NotFound,
        "unable to resolve VST3 module binary path",
    ))
}

fn match_vst3_controller(
    component: &Vst3FactoryClass,
    classes: &[Vst3FactoryClass],
    component_count: usize,
) -> Option<Vst3FactoryClass> {
    let controllers = classes
        .iter()
        .filter(|class| class.role == Vst3FactoryClassRole::Controller)
        .cloned()
        .collect::<Vec<_>>();
    controllers
        .iter()
        .find(|controller| controller.name == component.name)
        .cloned()
        .or_else(|| {
            if component_count == 1 && controllers.len() == 1 {
                controllers.into_iter().next()
            } else {
                None
            }
        })
}

fn derive_io_layout(bundle: &Vst3BundleInfo, is_instrument: bool) -> PluginIoLayout {
    PluginIoLayout {
        audio_inputs: bundle
            .signal_audio_inputs
            .unwrap_or(if is_instrument { 0 } else { 2 }),
        audio_outputs: bundle.signal_audio_outputs.unwrap_or(2),
        midi_inputs: bundle
            .signal_midi_inputs
            .unwrap_or(if is_instrument { 1 } else { 0 }),
        midi_outputs: bundle.signal_midi_outputs.unwrap_or(0),
    }
}

fn default_features(is_instrument: bool) -> Vec<PluginFeature> {
    if is_instrument {
        vec![PluginFeature::Instrument, PluginFeature::Analyzer]
    } else {
        vec![PluginFeature::AudioEffect, PluginFeature::Utility]
    }
}

fn class_is_instrument(class: &Vst3FactoryClass) -> bool {
    class.subcategories.iter().any(|subcategory| {
        subcategory.eq_ignore_ascii_case("instrument") || subcategory.eq_ignore_ascii_case("synth")
    }) || class.category.eq_ignore_ascii_case("Instrument")
}

fn derive_plugin_type_id(bundle: &Vst3BundleInfo, class: &Vst3FactoryClass) -> String {
    let base = bundle
        .bundle_identifier
        .clone()
        .or_else(|| bundle.bundle_name.clone())
        .unwrap_or_else(|| "vst3-plugin".to_string());
    let bundle_key = sanitize_plugin_id_segment(&base);
    let class_key = class.class_id.to_ascii_lowercase();
    format!("plugin:vst3:{bundle_key}:{class_key}")
}

fn fallback_vendor(bundle: &Vst3BundleInfo, factory_vendor: Option<&str>) -> String {
    factory_vendor
        .map(str::to_string)
        .or_else(|| {
            bundle
                .bundle_identifier
                .as_deref()
                .and_then(|bundle_id| bundle_id.split('.').next_back().map(str::to_string))
        })
        .unwrap_or_else(|| "Unknown".to_string())
}

fn role_from_category(category: &str) -> Vst3FactoryClassRole {
    if category.eq_ignore_ascii_case("Component Controller Class")
        || category.eq_ignore_ascii_case("Controller")
    {
        Vst3FactoryClassRole::Controller
    } else if category.eq_ignore_ascii_case("Audio Module Class")
        || category.eq_ignore_ascii_case("Instrument")
        || category.eq_ignore_ascii_case("Fx")
    {
        Vst3FactoryClassRole::Component
    } else {
        Vst3FactoryClassRole::Other
    }
}

fn sanitize_plugin_id_segment(value: &str) -> String {
    let mut sanitized = String::with_capacity(value.len());
    for ch in value.chars() {
        let lower = ch.to_ascii_lowercase();
        if lower.is_ascii_alphanumeric() {
            sanitized.push(lower);
        } else if !sanitized.ends_with('-') {
            sanitized.push('-');
        }
    }
    sanitized.trim_matches('-').to_string()
}

fn plist_string(dict: &plist::Dictionary, key: &str) -> Option<String> {
    dict.get(key)
        .and_then(plist::Value::as_string)
        .map(str::to_string)
}

fn plist_u16(dict: &plist::Dictionary, key: &str) -> Option<u16> {
    dict.get(key)
        .and_then(plist::Value::as_signed_integer)
        .and_then(|value| u16::try_from(value).ok())
}

fn plist_string_array(dict: &plist::Dictionary, key: &str) -> Option<Vec<String>> {
    dict.get(key)
        .and_then(plist::Value::as_array)
        .map(|values| {
            values
                .iter()
                .filter_map(plist::Value::as_string)
                .map(str::to_string)
                .collect::<Vec<_>>()
        })
}

fn parse_feature_list(raw: &str) -> io::Result<Vec<PluginFeature>> {
    raw.split(',')
        .map(str::trim)
        .filter(|feature| !feature.is_empty())
        .map(|feature| match feature {
            "Instrument" => Ok(PluginFeature::Instrument),
            "Analyzer" => Ok(PluginFeature::Analyzer),
            "AudioEffect" => Ok(PluginFeature::AudioEffect),
            "Utility" => Ok(PluginFeature::Utility),
            other => Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("unsupported VST3 feature `{other}`"),
            )),
        })
        .collect::<io::Result<Vec<_>>>()
}

fn bytes_to_upper_hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02X}")).collect()
}

fn c_char_array_to_string<const N: usize>(value: &[c_char; N]) -> String {
    let bytes = value
        .iter()
        .copied()
        .take_while(|byte| *byte != 0)
        .map(|byte| byte as u8)
        .collect::<Vec<_>>();
    String::from_utf8_lossy(&bytes).trim().to_string()
}

fn plist_to_io_error(error: plist::Error) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, error.to_string())
}

fn libloading_to_io(error: libloading::Error) -> io::Error {
    io::Error::other(error.to_string())
}

impl Vst3FactoryClass {
    fn vendor(&self) -> Option<String> {
        self.vendor.clone()
    }

    fn version(&self) -> Option<String> {
        self.version.clone()
    }
}
