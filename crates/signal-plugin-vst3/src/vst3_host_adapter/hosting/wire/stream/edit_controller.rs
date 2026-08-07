//! IEditController vtable and ParameterInfo.

use std::ffi::c_void;

use super::super::com::*;

/// `Steinberg::Vst::ParameterInfo`.
#[repr(C)]
pub(crate) struct ParameterInfo {
    pub(crate) id: u32,
    pub(crate) title: [i16; 128],
    pub(crate) short_title: [i16; 128],
    pub(crate) units: [i16; 128],
    pub(crate) step_count: i32,
    pub(crate) default_normalized_value: f64,
    pub(crate) unit_id: i32,
    pub(crate) flags: i32,
}

impl ParameterInfo {
    pub(crate) fn zeroed() -> Self {
        Self {
            id: 0,
            title: [0; 128],
            short_title: [0; 128],
            units: [0; 128],
            step_count: 0,
            default_normalized_value: 0.0,
            unit_id: 0,
            flags: 0,
        }
    }
}

/// `IEditController` (FUnknown + IPluginBase + IEditController, in order).
#[repr(C)]
pub(crate) struct EditControllerVTable {
    pub(crate) query_interface:
        unsafe extern "C" fn(*mut c_void, *const Tuid, *mut *mut c_void) -> Tresult,
    pub(crate) add_ref: unsafe extern "C" fn(*mut c_void) -> u32,
    pub(crate) release: unsafe extern "C" fn(*mut c_void) -> u32,
    pub(crate) initialize: unsafe extern "C" fn(*mut c_void, *mut c_void) -> Tresult,
    pub(crate) terminate: unsafe extern "C" fn(*mut c_void) -> Tresult,
    pub(crate) set_component_state: unsafe extern "C" fn(*mut c_void, *mut c_void) -> Tresult,
    pub(crate) set_state: unsafe extern "C" fn(*mut c_void, *mut c_void) -> Tresult,
    pub(crate) get_state: unsafe extern "C" fn(*mut c_void, *mut c_void) -> Tresult,
    pub(crate) get_parameter_count: unsafe extern "C" fn(*mut c_void) -> i32,
    pub(crate) get_parameter_info:
        unsafe extern "C" fn(*mut c_void, i32, *mut ParameterInfo) -> Tresult,
    pub(crate) get_param_string_by_value:
        unsafe extern "C" fn(*mut c_void, u32, f64, *mut i16) -> Tresult,
    pub(crate) get_param_value_by_string:
        unsafe extern "C" fn(*mut c_void, u32, *mut i16, *mut f64) -> Tresult,
    pub(crate) normalized_param_to_plain: unsafe extern "C" fn(*mut c_void, u32, f64) -> f64,
    pub(crate) plain_param_to_normalized: unsafe extern "C" fn(*mut c_void, u32, f64) -> f64,
    pub(crate) get_param_normalized: unsafe extern "C" fn(*mut c_void, u32) -> f64,
    pub(crate) set_param_normalized: unsafe extern "C" fn(*mut c_void, u32, f64) -> Tresult,
    pub(crate) set_component_handler: unsafe extern "C" fn(*mut c_void, *mut c_void) -> Tresult,
    pub(crate) create_view:
        unsafe extern "C" fn(*mut c_void, *const std::ffi::c_char) -> *mut c_void,
}
