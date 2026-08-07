//! VST3 introspection types and factory COM layouts.

use signal_plugin::PluginFeature;
use std::{
    ffi::{c_char, c_void},
    time::Duration,
};

#[cfg(not(target_os = "macos"))]
use libloading::Library;

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
pub(crate) struct Vst3FactorySnapshotWire {
    pub(crate) vendor: Option<String>,
    pub(crate) classes: Vec<Vst3FactoryClass>,
}

pub(crate) const VST3_SCAN_HELPER_ENV: &str = "SIGNAL_VST3_SCAN_HELPER";
pub(crate) const VST3_SCAN_HELPER_TIMEOUT_MS_ENV: &str = "SIGNAL_VST3_SCAN_HELPER_TIMEOUT_MS";
pub(crate) const VST3_SCAN_HELPER_DEFAULT_TIMEOUT: Duration = Duration::from_secs(10);
pub(crate) const VST3_SCAN_HELPER_BINARY: &str = "signal-vst3-scan-helper";

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Vst3BundleSnapshot {
    pub(crate) plugins: Vec<Vst3ModuleMetadata>,
    pub(crate) factory_classes: Vec<Vst3FactoryClass>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct Vst3BundleInfo {
    pub(crate) bundle_identifier: Option<String>,
    pub(crate) bundle_name: Option<String>,
    pub(crate) executable_name: Option<String>,
    pub(crate) version: Option<String>,
    pub(crate) signal_plugin_type_id: Option<String>,
    pub(crate) signal_audio_inputs: Option<u16>,
    pub(crate) signal_audio_outputs: Option<u16>,
    pub(crate) signal_midi_inputs: Option<u16>,
    pub(crate) signal_midi_outputs: Option<u16>,
    pub(crate) signal_features: Option<Vec<PluginFeature>>,
}

#[derive(serde::Deserialize)]
pub(crate) struct ModuleInfoDocument {
    #[serde(rename = "Factory Info")]
    pub(crate) factory_info: Option<ModuleFactoryInfo>,
    #[serde(rename = "Classes", default)]
    pub(crate) classes: Vec<ModuleInfoClass>,
}

#[derive(serde::Deserialize)]
pub(crate) struct ModuleFactoryInfo {
    #[serde(rename = "Vendor")]
    pub(crate) vendor: Option<String>,
}

#[derive(serde::Deserialize)]
pub(crate) struct ModuleInfoClass {
    #[serde(rename = "CID")]
    pub(crate) cid: String,
    #[serde(rename = "Category")]
    pub(crate) category: String,
    #[serde(rename = "Name")]
    pub(crate) name: String,
    #[serde(rename = "Vendor")]
    pub(crate) vendor: Option<String>,
    #[serde(rename = "Version")]
    pub(crate) version: Option<String>,
    #[serde(rename = "Sub Categories", default)]
    pub(crate) subcategories: Vec<String>,
}

#[repr(C)]
pub(crate) struct RawPluginFactory {
    pub(crate) vtable: *const PluginFactoryVTable,
}

#[repr(C)]
pub(crate) struct PluginFactoryVTable {
    pub(crate) query_interface:
        unsafe extern "C" fn(*mut c_void, *const c_void, *mut *mut c_void) -> i32,
    pub(crate) add_ref: unsafe extern "C" fn(*mut c_void) -> u32,
    pub(crate) release: unsafe extern "C" fn(*mut c_void) -> u32,
    pub(crate) get_factory_info: unsafe extern "C" fn(*mut c_void, *mut PFactoryInfo) -> i32,
    pub(crate) count_classes: unsafe extern "C" fn(*mut c_void) -> i32,
    pub(crate) get_class_info: unsafe extern "C" fn(*mut c_void, i32, *mut PClassInfo) -> i32,
    pub(crate) create_instance:
        unsafe extern "C" fn(*mut c_void, *const u8, *const c_void, *mut *mut c_void) -> i32,
}

#[repr(C)]
pub(crate) struct PluginFactory2VTable {
    pub(crate) base: PluginFactoryVTable,
    pub(crate) get_class_info_2: unsafe extern "C" fn(*mut c_void, i32, *mut PClassInfo2) -> i32,
}

#[repr(C)]
pub(crate) struct PFactoryInfo {
    pub(crate) vendor: [c_char; 64],
    pub(crate) url: [c_char; 256],
    pub(crate) email: [c_char; 128],
    pub(crate) flags: i32,
}

#[repr(C)]
pub(crate) struct PClassInfo {
    pub(crate) cid: [u8; 16],
    pub(crate) cardinality: i32,
    pub(crate) category: [c_char; 32],
    pub(crate) name: [c_char; 64],
}

#[repr(C)]
pub(crate) struct PClassInfo2 {
    pub(crate) cid: [u8; 16],
    pub(crate) cardinality: i32,
    pub(crate) category: [c_char; 32],
    pub(crate) name: [c_char; 64],
    pub(crate) class_flags: u32,
    pub(crate) subcategories: [c_char; 128],
    pub(crate) vendor: [c_char; 64],
    pub(crate) version: [c_char; 64],
    pub(crate) sdk_version: [c_char; 64],
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

pub(crate) const IPLUGIN_FACTORY_2_IID: [u8; 16] =
    vst3_tuid(0x0007B650, 0xF24B4C0B, 0xA464EDB9, 0xF00B2ABB);

pub(crate) type EntryProc = unsafe extern "C" fn(*mut c_void) -> bool;
pub(crate) type ExitProc = unsafe extern "C" fn();
pub(crate) type GetPluginFactoryProc = unsafe extern "C" fn() -> *mut c_void;
