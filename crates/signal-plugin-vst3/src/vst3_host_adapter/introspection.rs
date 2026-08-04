use super::Vst3HostPlatform;
use signal_plugin::{
    PluginDescriptor, PluginFeature, PluginFormat, PluginIoLayout, PluginLifecycleContract,
    PluginProcessingContract, PluginStateContract,
};
use std::{
    ffi::{c_char, c_void, CString, OsString},
    fs, io,
    io::Read,
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    thread,
    time::{Duration, Instant},
};

#[cfg(not(target_os = "macos"))]
use libloading::Library;

use super::hosting::{
    clear_factory_host_context, set_factory_host_context, should_set_factory_host_context,
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

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct Vst3FactoryClass {
    pub(crate) role: Vst3FactoryClassRole,
    pub(crate) class_id: String,
    pub(crate) category: String,
    pub(crate) name: String,
    pub(crate) vendor: Option<String>,
    pub(crate) version: Option<String>,
    pub(crate) subcategories: Vec<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) enum Vst3FactoryClassRole {
    Component,
    Controller,
    Other,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct Vst3FactorySnapshotWire {
    vendor: Option<String>,
    classes: Vec<Vst3FactoryClass>,
}

const VST3_SCAN_HELPER_ENV: &str = "SIGNAL_VST3_SCAN_HELPER";
const VST3_SCAN_HELPER_TIMEOUT_MS_ENV: &str = "SIGNAL_VST3_SCAN_HELPER_TIMEOUT_MS";
const VST3_SCAN_HELPER_DEFAULT_TIMEOUT: Duration = Duration::from_secs(10);
const VST3_SCAN_HELPER_BINARY: &str = "signal-vst3-scan-helper";

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
struct PluginFactory2VTable {
    base: PluginFactoryVTable,
    get_class_info_2: unsafe extern "C" fn(*mut c_void, i32, *mut PClassInfo2) -> i32,
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

#[repr(C)]
struct PClassInfo2 {
    cid: [u8; 16],
    cardinality: i32,
    category: [c_char; 32],
    name: [c_char; 64],
    class_flags: u32,
    subcategories: [c_char; 128],
    vendor: [c_char; 64],
    version: [c_char; 64],
    sdk_version: [c_char; 64],
}

const fn vst3_tuid(l1: u32, l2: u32, l3: u32, l4: u32) -> [u8; 16] {
    if cfg!(target_os = "windows") {
        [
            l1 as u8,
            (l1 >> 8) as u8,
            (l1 >> 16) as u8,
            (l1 >> 24) as u8,
            (l2 >> 16) as u8,
            (l2 >> 24) as u8,
            l2 as u8,
            (l2 >> 8) as u8,
            (l3 >> 24) as u8,
            (l3 >> 16) as u8,
            (l3 >> 8) as u8,
            l3 as u8,
            (l4 >> 24) as u8,
            (l4 >> 16) as u8,
            (l4 >> 8) as u8,
            l4 as u8,
        ]
    } else {
        [
            (l1 >> 24) as u8,
            (l1 >> 16) as u8,
            (l1 >> 8) as u8,
            l1 as u8,
            (l2 >> 24) as u8,
            (l2 >> 16) as u8,
            (l2 >> 8) as u8,
            l2 as u8,
            (l3 >> 24) as u8,
            (l3 >> 16) as u8,
            (l3 >> 8) as u8,
            l3 as u8,
            (l4 >> 24) as u8,
            (l4 >> 16) as u8,
            (l4 >> 8) as u8,
            l4 as u8,
        ]
    }
}

const IPLUGIN_FACTORY_2_IID: [u8; 16] = vst3_tuid(0x0007B650, 0xF24B4C0B, 0xA464EDB9, 0xF00B2ABB);

type EntryProc = unsafe extern "C" fn(*mut c_void) -> bool;
type ExitProc = unsafe extern "C" fn();
type GetPluginFactoryProc = unsafe extern "C" fn() -> *mut c_void;

#[cfg(target_os = "macos")]
mod macos_bundle {
    use super::*;

    type CFAllocatorRef = *const c_void;
    type CFStringRef = *const c_void;
    type CFURLRef = *const c_void;
    type CFBundleRef = *mut c_void;

    #[link(name = "CoreFoundation", kind = "framework")]
    extern "C" {
        static kCFAllocatorDefault: CFAllocatorRef;
        fn CFBundleCreate(allocator: CFAllocatorRef, bundleURL: CFURLRef) -> CFBundleRef;
        fn CFBundleGetFunctionPointerForName(
            bundle: CFBundleRef,
            functionName: CFStringRef,
        ) -> *mut c_void;
        fn CFBundleLoadExecutable(bundle: CFBundleRef) -> u8;
        fn CFRelease(cf: *const c_void);
        fn CFStringCreateWithCString(
            alloc: CFAllocatorRef,
            cStr: *const c_char,
            encoding: u32,
        ) -> CFStringRef;
        fn CFURLCreateWithFileSystemPath(
            allocator: CFAllocatorRef,
            filePath: CFStringRef,
            pathStyle: isize,
            isDirectory: u8,
        ) -> CFURLRef;
    }

    const K_CF_STRING_ENCODING_UTF8: u32 = 0x0800_0100;
    const K_CF_URL_POSIX_PATH_STYLE: isize = 0;

    pub(super) struct MacVst3Bundle {
        bundle: CFBundleRef,
    }

    impl MacVst3Bundle {
        pub(super) fn load(bundle_root: &Path) -> io::Result<Self> {
            let path = bundle_root
                .to_str()
                .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "invalid VST3 path"))?;
            let path_c = CString::new(path)
                .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "invalid VST3 path"))?;
            unsafe {
                let path_string = CFStringCreateWithCString(
                    kCFAllocatorDefault,
                    path_c.as_ptr(),
                    K_CF_STRING_ENCODING_UTF8,
                );
                if path_string.is_null() {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        "invalid VST3 path",
                    ));
                }
                let bundle_url = CFURLCreateWithFileSystemPath(
                    kCFAllocatorDefault,
                    path_string,
                    K_CF_URL_POSIX_PATH_STYLE,
                    1,
                );
                CFRelease(path_string);
                if bundle_url.is_null() {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        "invalid VST3 bundle URL",
                    ));
                }
                let bundle = CFBundleCreate(kCFAllocatorDefault, bundle_url);
                CFRelease(bundle_url);
                if bundle.is_null() {
                    return Err(io::Error::new(
                        io::ErrorKind::NotFound,
                        "failed to open VST3 bundle",
                    ));
                }
                if CFBundleLoadExecutable(bundle) == 0 {
                    CFRelease(bundle);
                    return Err(io::Error::other("failed to load VST3 bundle executable"));
                }
                Ok(Self { bundle })
            }
        }

        pub(super) fn bundle_ref(&self) -> *mut c_void {
            self.bundle.cast()
        }

        fn function_ptr(&self, name: &str) -> Option<*mut c_void> {
            let name_c = CString::new(name).ok()?;
            unsafe {
                let name_string = CFStringCreateWithCString(
                    kCFAllocatorDefault,
                    name_c.as_ptr(),
                    K_CF_STRING_ENCODING_UTF8,
                );
                if name_string.is_null() {
                    return None;
                }
                let pointer = CFBundleGetFunctionPointerForName(self.bundle, name_string);
                CFRelease(name_string);
                (!pointer.is_null()).then_some(pointer)
            }
        }

        pub(super) fn entry(&self) -> Option<EntryProc> {
            self.function_ptr("bundleEntry")
                .map(|pointer| unsafe { std::mem::transmute(pointer) })
        }

        pub(super) fn exit(&self) -> Option<ExitProc> {
            self.function_ptr("bundleExit")
                .map(|pointer| unsafe { std::mem::transmute(pointer) })
        }

        pub(super) fn factory(&self) -> Option<GetPluginFactoryProc> {
            self.function_ptr("GetPluginFactory")
                .map(|pointer| unsafe { std::mem::transmute(pointer) })
        }
    }

    impl Drop for MacVst3Bundle {
        fn drop(&mut self) {
            unsafe {
                // Objective-C classes registered by a plugin bundle cannot be
                // unregistered safely, so discovery releases the bundle object
                // without unloading executable code.
                CFRelease(self.bundle);
            }
        }
    }
}

pub(crate) fn read_vst3_bundle_snapshot(
    bundle_root: &Path,
    platform: Vst3HostPlatform,
) -> io::Result<Vst3BundleSnapshot> {
    let bundle = read_vst3_bundle_info(bundle_root)?;
    preflight_vendor_scan_access(&bundle)?;
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

fn preflight_vendor_scan_access(bundle: &Vst3BundleInfo) -> io::Result<()> {
    if !bundle
        .bundle_identifier
        .as_deref()
        .is_some_and(is_native_instruments_bundle)
    {
        return Ok(());
    }
    let Some(home) = std::env::var_os("HOME") else {
        return Ok(());
    };
    let documents = PathBuf::from(home)
        .join("Documents")
        .join("Native Instruments");
    let denied = match fs::read_dir(&documents) {
        Err(error) => error.kind() == io::ErrorKind::PermissionDenied,
        Ok(mut entries) => entries.next().is_some_and(|entry| {
            entry.is_err_and(|error| error.kind() == io::ErrorKind::PermissionDenied)
        }),
    };
    if denied {
        Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            format!(
                "Native Instruments VST3 inspection requires macOS Documents folder access ({})",
                documents.display()
            ),
        ))
    } else {
        Ok(())
    }
}

fn is_native_instruments_bundle(identifier: &str) -> bool {
    identifier
        .to_ascii_lowercase()
        .starts_with("com.native-instruments.")
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
    let mut moduleinfo_error = None;
    if let Some(moduleinfo_path) = candidate_moduleinfo_paths(bundle_root)
        .into_iter()
        .find(|path| path.is_file())
    {
        match json5::from_str::<ModuleInfoDocument>(&fs::read_to_string(moduleinfo_path)?) {
            Ok(document) => {
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
                if !classes.is_empty() {
                    return Ok((vendor, classes));
                }
                moduleinfo_error = Some("missing VST3 classes in moduleinfo.json".to_string());
            }
            Err(error) => {
                moduleinfo_error = Some(format!("invalid VST3 moduleinfo.json: {error}"));
            }
        }
    }
    match load_vst3_factory_classes_with_helper(bundle_root, platform) {
        Ok(snapshot) => Ok(snapshot),
        Err(error) if moduleinfo_error.is_some() => Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!(
                "{}; factory fallback failed: {error}",
                moduleinfo_error.expect("checked moduleinfo error")
            ),
        )),
        Err(error) => Err(error),
    }
}

/// Whether `moduleinfo.json` explicitly advertises `class_id` as a component.
/// Hosting uses this to safely recover from vendors that ship stale generated
/// class IDs while their binary exposes one unambiguous component class.
pub(super) fn moduleinfo_declares_component_class(bundle_root: &Path, class_id: &str) -> bool {
    candidate_moduleinfo_paths(bundle_root)
        .into_iter()
        .find(|path| path.is_file())
        .and_then(|path| fs::read_to_string(path).ok())
        .and_then(|contents| json5::from_str::<ModuleInfoDocument>(&contents).ok())
        .is_some_and(|document| {
            document.classes.into_iter().any(|class| {
                role_from_category(&class.category) == Vst3FactoryClassRole::Component
                    && class.cid.eq_ignore_ascii_case(class_id)
            })
        })
}

pub(super) fn run_vst3_scan_helper<I>(args: I) -> i32
where
    I: IntoIterator<Item = OsString>,
{
    let mut args = args.into_iter();
    let Some(first) = args.next() else {
        eprintln!("missing VST3 scan helper platform");
        return 64;
    };
    let platform_arg = if first == super::VST3_SCAN_HELPER_ARG {
        let Some(platform) = args.next() else {
            eprintln!("missing VST3 scan helper platform");
            return 64;
        };
        platform
    } else {
        first
    };
    let Some(bundle_root) = args.next() else {
        eprintln!("missing VST3 scan helper bundle path");
        return 64;
    };
    let Some(platform) = parse_platform_arg(&platform_arg) else {
        eprintln!("unsupported VST3 scan helper platform");
        return 64;
    };
    let bundle_root = PathBuf::from(bundle_root);
    if let Err(error) =
        read_vst3_bundle_info(&bundle_root).and_then(|bundle| preflight_vendor_scan_access(&bundle))
    {
        eprintln!("{error}");
        return 65;
    }
    match load_vst3_factory_classes_from_module(&bundle_root, platform) {
        Ok((vendor, classes)) => {
            let payload = Vst3FactorySnapshotWire { vendor, classes };
            match serde_json::to_string(&payload) {
                Ok(json) => {
                    println!("{json}");
                    0
                }
                Err(error) => {
                    eprintln!("failed to encode VST3 scan helper result: {error}");
                    70
                }
            }
        }
        Err(error) => {
            eprintln!("{error}");
            65
        }
    }
}

fn load_vst3_factory_classes_with_helper(
    bundle_root: &Path,
    platform: Vst3HostPlatform,
) -> io::Result<(Option<String>, Vec<Vst3FactoryClass>)> {
    let mut command = scan_helper_command()?;
    command
        .arg(super::VST3_SCAN_HELPER_ARG)
        .arg(platform_arg(platform))
        .arg(bundle_root)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let child = command.spawn().map_err(|error| {
        io::Error::new(
            error.kind(),
            format!("failed to start VST3 scan helper: {error}"),
        )
    })?;
    read_vst3_scan_helper_child(child, scan_helper_timeout())
}

fn read_vst3_scan_helper_child(
    mut child: Child,
    timeout: Duration,
) -> io::Result<(Option<String>, Vec<Vst3FactoryClass>)> {
    let deadline = Instant::now() + timeout;
    loop {
        if let Some(status) = child.try_wait()? {
            let mut stdout = Vec::new();
            if let Some(mut output) = child.stdout.take() {
                output.read_to_end(&mut stdout)?;
            }
            let mut stderr = String::new();
            if let Some(mut output) = child.stderr.take() {
                output.read_to_string(&mut stderr)?;
            }
            if !status.success() {
                let detail = stderr.split_whitespace().collect::<Vec<_>>().join(" ");
                let message = format!(
                    "VST3 scan helper exited with status {status}{}",
                    if detail.is_empty() {
                        String::new()
                    } else {
                        format!(": {detail}")
                    }
                );
                return Err(if status.code() == Some(65) {
                    io::Error::new(io::ErrorKind::InvalidData, message)
                } else {
                    io::Error::other(message)
                });
            }
            let snapshot = decode_scan_helper_snapshot(&stdout)?;
            return Ok((snapshot.vendor, snapshot.classes));
        }
        if Instant::now() >= deadline {
            let _ = child.kill();
            let _ = child.wait();
            return Err(io::Error::new(
                io::ErrorKind::TimedOut,
                "VST3 scan helper timed out",
            ));
        }
        thread::sleep(Duration::from_millis(10));
    }
}

fn decode_scan_helper_snapshot(stdout: &[u8]) -> io::Result<Vst3FactorySnapshotWire> {
    stdout
        .split(|byte| *byte == b'\n')
        .rev()
        .find_map(|line| serde_json::from_slice::<Vst3FactorySnapshotWire>(line).ok())
        .ok_or_else(|| {
            let error = serde_json::from_slice::<Vst3FactorySnapshotWire>(stdout)
                .err()
                .expect("unmatched helper output should remain invalid");
            io::Error::new(io::ErrorKind::InvalidData, error.to_string())
        })
}

#[cfg(all(test, unix))]
mod scan_helper_tests {
    use super::*;

    fn shell_child(script: &str) -> Child {
        Command::new("/bin/sh")
            .args(["-c", script])
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("spawn deterministic scan helper fixture")
    }

    #[test]
    fn scan_helper_timeout_kills_and_reaps_child() {
        let child = shell_child("sleep 5");
        let error = read_vst3_scan_helper_child(child, Duration::from_millis(20))
            .expect_err("slow helper should time out");

        assert_eq!(error.kind(), io::ErrorKind::TimedOut);
    }

    #[test]
    fn scan_helper_abnormal_exit_is_reported() {
        let child = shell_child("echo fixture-reason >&2; exit 7");
        let error = read_vst3_scan_helper_child(child, Duration::from_secs(1))
            .expect_err("failed helper should be reported");

        assert_eq!(error.kind(), io::ErrorKind::Other);
        assert!(error.to_string().contains("status"));
        assert!(error.to_string().contains("fixture-reason"));
    }

    #[test]
    fn scan_helper_inspection_failure_is_invalid_not_crashed() {
        let child = shell_child("echo invalid-fixture >&2; exit 65");
        let error = read_vst3_scan_helper_child(child, Duration::from_secs(1))
            .expect_err("inspection failure should be reported");

        assert_eq!(error.kind(), io::ErrorKind::InvalidData);
        assert!(error.to_string().contains("invalid-fixture"));
    }

    #[test]
    fn scan_helper_invalid_output_is_reported() {
        let child = shell_child("printf not-json");
        let error = read_vst3_scan_helper_child(child, Duration::from_secs(1))
            .expect_err("invalid helper output should be reported");

        assert_eq!(error.kind(), io::ErrorKind::InvalidData);
    }

    #[test]
    fn scan_helper_ignores_plugin_logs_around_json_payload() {
        let payload = r#"{"vendor":"Example","classes":[]}"#;
        for script in [
            format!("printf 'plugin log\\n%s\\n' '{payload}'"),
            format!("printf '%s\\nplugin shutdown log\\n' '{payload}'"),
        ] {
            let child = shell_child(&script);
            let (vendor, classes) = read_vst3_scan_helper_child(child, Duration::from_secs(1))
                .expect("embedded helper payload");
            assert_eq!(vendor.as_deref(), Some("Example"));
            assert!(classes.is_empty());
        }
    }

    #[test]
    fn native_instruments_bundle_detection_is_vendor_scoped() {
        assert!(is_native_instruments_bundle(
            "com.native-instruments.Raum.vst3"
        ));
        assert!(!is_native_instruments_bundle("com.example.Raum.vst3"));
    }
}

fn scan_helper_command() -> io::Result<Command> {
    if let Some(path) = std::env::var_os(VST3_SCAN_HELPER_ENV).filter(|path| !path.is_empty()) {
        return Ok(Command::new(path));
    }
    if let Some(path) = nearby_scan_helper_binary()? {
        return Ok(Command::new(path));
    }
    Ok(Command::new(std::env::current_exe()?))
}

fn nearby_scan_helper_binary() -> io::Result<Option<PathBuf>> {
    let current_exe = std::env::current_exe()?;
    let Some(current_dir) = current_exe.parent() else {
        return Ok(None);
    };
    let candidates = [
        current_dir.join(helper_binary_name()),
        current_dir
            .parent()
            .map(|parent| parent.join(helper_binary_name()))
            .unwrap_or_else(|| current_dir.join(helper_binary_name())),
    ];
    Ok(candidates.into_iter().find(|path| path.is_file()))
}

fn helper_binary_name() -> String {
    #[cfg(target_os = "windows")]
    {
        format!("{VST3_SCAN_HELPER_BINARY}.exe")
    }
    #[cfg(not(target_os = "windows"))]
    {
        VST3_SCAN_HELPER_BINARY.to_string()
    }
}

fn scan_helper_timeout() -> Duration {
    std::env::var(VST3_SCAN_HELPER_TIMEOUT_MS_ENV)
        .ok()
        .and_then(|raw| raw.parse::<u64>().ok())
        .map(Duration::from_millis)
        .unwrap_or(VST3_SCAN_HELPER_DEFAULT_TIMEOUT)
}

fn platform_arg(platform: Vst3HostPlatform) -> &'static str {
    match platform {
        Vst3HostPlatform::MacOs => "macos",
        Vst3HostPlatform::Linux => "linux",
        Vst3HostPlatform::Windows => "windows",
    }
}

fn parse_platform_arg(value: &OsString) -> Option<Vst3HostPlatform> {
    match value.to_str()? {
        "macos" => Some(Vst3HostPlatform::MacOs),
        "linux" => Some(Vst3HostPlatform::Linux),
        "windows" => Some(Vst3HostPlatform::Windows),
        _ => None,
    }
}

#[cfg(not(target_os = "macos"))]
fn entry_symbol(platform: Vst3HostPlatform) -> &'static [u8] {
    match platform {
        Vst3HostPlatform::MacOs => b"bundleEntry\0",
        Vst3HostPlatform::Linux => b"ModuleEntry\0",
        Vst3HostPlatform::Windows => b"InitDll\0",
    }
}

#[cfg(not(target_os = "macos"))]
fn exit_symbol(platform: Vst3HostPlatform) -> &'static [u8] {
    match platform {
        Vst3HostPlatform::MacOs => b"bundleExit\0",
        Vst3HostPlatform::Linux => b"ModuleExit\0",
        Vst3HostPlatform::Windows => b"ExitDll\0",
    }
}

#[cfg(target_os = "macos")]
fn load_vst3_factory_classes_from_module(
    bundle_root: &Path,
    _platform: Vst3HostPlatform,
) -> io::Result<(Option<String>, Vec<Vst3FactoryClass>)> {
    let bundle = macos_bundle::MacVst3Bundle::load(bundle_root)?;
    unsafe {
        if let Some(entry) = bundle.entry() {
            if !entry(bundle.bundle_ref()) {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "VST3 bundleEntry returned false",
                ));
            }
        }
        let get_plugin_factory = bundle.factory().ok_or_else(|| {
            io::Error::new(io::ErrorKind::InvalidData, "VST3 GetPluginFactory missing")
        })?;
        let snapshot = read_factory_classes(
            get_plugin_factory(),
            should_set_factory_host_context(bundle_root),
        );
        if let Some(exit) = bundle.exit() {
            exit();
        }
        snapshot
    }
}

#[cfg(not(target_os = "macos"))]
fn load_vst3_factory_classes_from_module(
    bundle_root: &Path,
    platform: Vst3HostPlatform,
) -> io::Result<(Option<String>, Vec<Vst3FactoryClass>)> {
    let module_path = resolve_module_binary_path(bundle_root, platform)?;
    let library = unsafe { Library::new(&module_path) }.map_err(libloading_to_io)?;
    unsafe {
        if let Ok(entry) = library.get::<EntryProc>(entry_symbol(platform)) {
            if !entry(std::ptr::null_mut()) {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "VST3 module entry returned false",
                ));
            }
        }
        let get_plugin_factory = library
            .get::<GetPluginFactoryProc>(b"GetPluginFactory\0")
            .map_err(libloading_to_io)?;
        let snapshot = read_factory_classes(
            get_plugin_factory(),
            should_set_factory_host_context(bundle_root),
        );
        if let Ok(exit) = library.get::<ExitProc>(exit_symbol(platform)) {
            exit();
        }
        snapshot
    }
}

fn read_factory_classes(
    factory_ptr: *mut c_void,
    set_host_context: bool,
) -> io::Result<(Option<String>, Vec<Vst3FactoryClass>)> {
    if factory_ptr.is_null() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "VST3 GetPluginFactory returned null",
        ));
    }

    let factory = factory_ptr as *mut RawPluginFactory;
    let vtable = unsafe { (*factory).vtable };
    if vtable.is_null() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "VST3 factory vtable was null",
        ));
    }
    let context_set = set_host_context && unsafe { set_factory_host_context(factory_ptr) };

    let mut factory_info = PFactoryInfo {
        vendor: [0; 64],
        url: [0; 256],
        email: [0; 128],
        flags: 0,
    };
    let vendor = if unsafe { ((*vtable).get_factory_info)(factory_ptr, &mut factory_info) } == 0 {
        Some(c_char_array_to_string(&factory_info.vendor))
    } else {
        None
    };

    let mut class_count = unsafe { ((*vtable).count_classes)(factory_ptr) };
    if class_count <= 0 && context_set {
        unsafe {
            clear_factory_host_context(factory_ptr);
        }
        class_count = unsafe { ((*vtable).count_classes)(factory_ptr) };
    }
    if class_count <= 0 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "VST3 factory exposed no classes",
        ));
    }
    let mut factory_2_ptr = std::ptr::null_mut();
    let factory_2 = if unsafe {
        ((*vtable).query_interface)(
            factory_ptr,
            IPLUGIN_FACTORY_2_IID.as_ptr().cast(),
            &mut factory_2_ptr,
        )
    } == 0
        && !factory_2_ptr.is_null()
    {
        Some(factory_2_ptr)
    } else {
        None
    };
    let mut classes = Vec::new();
    for index in 0..class_count {
        let mut class_info = PClassInfo {
            cid: [0; 16],
            cardinality: 0,
            category: [0; 32],
            name: [0; 64],
        };
        if unsafe { ((*vtable).get_class_info)(factory_ptr, index, &mut class_info) } != 0 {
            continue;
        }
        let category = c_char_array_to_string(&class_info.category);
        let class_info_2 = factory_2.and_then(|factory_2_ptr| {
            let factory_2 = factory_2_ptr as *mut RawPluginFactory;
            let factory_2_vtable = unsafe { (*factory_2).vtable as *const PluginFactory2VTable };
            let mut info = PClassInfo2 {
                cid: [0; 16],
                cardinality: 0,
                category: [0; 32],
                name: [0; 64],
                class_flags: 0,
                subcategories: [0; 128],
                vendor: [0; 64],
                version: [0; 64],
                sdk_version: [0; 64],
            };
            if !factory_2_vtable.is_null()
                && unsafe {
                    ((*factory_2_vtable).get_class_info_2)(factory_2_ptr, index, &mut info)
                } == 0
            {
                Some(info)
            } else {
                None
            }
        });
        let subcategories = class_info_2
            .as_ref()
            .map(|info| c_char_array_to_string(&info.subcategories))
            .unwrap_or_default()
            .split('|')
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_string)
            .collect();
        classes.push(Vst3FactoryClass {
            role: role_from_category(&category),
            class_id: bytes_to_upper_hex(&class_info.cid),
            category,
            name: c_char_array_to_string(&class_info.name),
            vendor: class_info_2
                .as_ref()
                .map(|info| c_char_array_to_string(&info.vendor))
                .filter(|value| !value.is_empty()),
            version: class_info_2
                .as_ref()
                .map(|info| c_char_array_to_string(&info.version))
                .filter(|value| !value.is_empty()),
            subcategories,
        });
    }

    if let Some(factory_2_ptr) = factory_2 {
        let factory_2 = factory_2_ptr as *mut RawPluginFactory;
        let factory_2_vtable = unsafe { (*factory_2).vtable };
        if !factory_2_vtable.is_null() {
            unsafe { ((*factory_2_vtable).release)(factory_2_ptr) };
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

#[cfg(not(target_os = "macos"))]
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
        || category.eq_ignore_ascii_case("Audio Mix Processor")
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

#[cfg(not(target_os = "macos"))]
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
