//! IPluginFactory vtables and class metadata.

use std::ffi::{c_char, c_void};

use super::super::com::*;

/// `FUnknown` method prefix shared by every vtable below.
#[repr(C)]
pub(crate) struct FUnknownVTable {
    pub(crate) query_interface:
        unsafe extern "C" fn(*mut c_void, *const Tuid, *mut *mut c_void) -> Tresult,
    pub(crate) add_ref: unsafe extern "C" fn(*mut c_void) -> u32,
    pub(crate) release: unsafe extern "C" fn(*mut c_void) -> u32,
}

/// `IPluginFactory` (mirrors the introspection module's layout, plus typed
/// `createInstance` arguments for hosting).
#[repr(C)]
pub(crate) struct PluginFactoryVTable {
    pub(crate) query_interface:
        unsafe extern "C" fn(*mut c_void, *const Tuid, *mut *mut c_void) -> Tresult,
    pub(crate) add_ref: unsafe extern "C" fn(*mut c_void) -> u32,
    pub(crate) release: unsafe extern "C" fn(*mut c_void) -> u32,
    pub(crate) get_factory_info: unsafe extern "C" fn(*mut c_void, *mut c_void) -> Tresult,
    pub(crate) count_classes: unsafe extern "C" fn(*mut c_void) -> i32,
    pub(crate) get_class_info: unsafe extern "C" fn(*mut c_void, i32, *mut c_void) -> Tresult,
    pub(crate) create_instance:
        unsafe extern "C" fn(*mut c_void, *const u8, *const u8, *mut *mut c_void) -> Tresult,
}

/// `PClassInfo` prefix used to distinguish component classes when a vendor's
/// `moduleinfo.json` advertises a stale class ID.
#[repr(C)]
pub(crate) struct FactoryClassInfo {
    pub(crate) cid: Tuid,
    pub(crate) cardinality: i32,
    pub(crate) category: [c_char; 32],
    pub(crate) name: [c_char; 64],
}

/// `IPluginFactory2` prefix needed to reach the `IPluginFactory3` extension.
#[repr(C)]
pub(crate) struct PluginFactory2VTable {
    pub(crate) base: PluginFactoryVTable,
    pub(crate) get_class_info_2: unsafe extern "C" fn(*mut c_void, i32, *mut c_void) -> Tresult,
}

/// `IPluginFactory3` adds Unicode class metadata and a factory-level host
/// context supplied before class enumeration or instance creation.
#[repr(C)]
pub(crate) struct PluginFactory3VTable {
    pub(crate) base: PluginFactory2VTable,
    pub(crate) get_class_info_unicode:
        unsafe extern "C" fn(*mut c_void, i32, *mut c_void) -> Tresult,
    pub(crate) set_host_context: unsafe extern "C" fn(*mut c_void, *mut c_void) -> Tresult,
}
