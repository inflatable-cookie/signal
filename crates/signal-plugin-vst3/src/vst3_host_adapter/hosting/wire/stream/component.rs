//! IComponent vtable and bus metadata.

use std::ffi::c_void;

use super::super::com::*;

/// `Steinberg::Vst::BusInfo`.
#[repr(C)]
pub(crate) struct BusInfo {
    pub(crate) media_type: i32,
    pub(crate) direction: i32,
    pub(crate) channel_count: i32,
    pub(crate) name: [i16; 128],
    pub(crate) bus_type: i32,
    pub(crate) flags: u32,
}

impl BusInfo {
    pub(crate) fn zeroed() -> Self {
        Self {
            media_type: 0,
            direction: 0,
            channel_count: 0,
            name: [0; 128],
            bus_type: 0,
            flags: 0,
        }
    }
}

/// `IComponent` (FUnknown + IPluginBase + IComponent methods, in order).
#[repr(C)]
pub(crate) struct ComponentVTable {
    pub(crate) query_interface:
        unsafe extern "C" fn(*mut c_void, *const Tuid, *mut *mut c_void) -> Tresult,
    pub(crate) add_ref: unsafe extern "C" fn(*mut c_void) -> u32,
    pub(crate) release: unsafe extern "C" fn(*mut c_void) -> u32,
    pub(crate) initialize: unsafe extern "C" fn(*mut c_void, *mut c_void) -> Tresult,
    pub(crate) terminate: unsafe extern "C" fn(*mut c_void) -> Tresult,
    pub(crate) get_controller_class_id: unsafe extern "C" fn(*mut c_void, *mut Tuid) -> Tresult,
    pub(crate) set_io_mode: unsafe extern "C" fn(*mut c_void, i32) -> Tresult,
    pub(crate) get_bus_count: unsafe extern "C" fn(*mut c_void, i32, i32) -> i32,
    pub(crate) get_bus_info:
        unsafe extern "C" fn(*mut c_void, i32, i32, i32, *mut BusInfo) -> Tresult,
    pub(crate) get_routing_info:
        unsafe extern "C" fn(*mut c_void, *mut c_void, *mut c_void) -> Tresult,
    pub(crate) activate_bus: unsafe extern "C" fn(*mut c_void, i32, i32, i32, u8) -> Tresult,
    pub(crate) set_active: unsafe extern "C" fn(*mut c_void, u8) -> Tresult,
    pub(crate) set_state: unsafe extern "C" fn(*mut c_void, *mut c_void) -> Tresult,
    pub(crate) get_state: unsafe extern "C" fn(*mut c_void, *mut c_void) -> Tresult,
}
