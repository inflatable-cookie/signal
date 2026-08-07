use std::ffi::c_void;

use super::super::hosting::{Tresult, Tuid};
use super::types::ViewRect;

/// `IPlugView` (FUnknown + view methods, declaration order per iplugview.h).
#[repr(C)]
pub(crate) struct PlugViewVTable {
    pub(crate) query_interface:
        unsafe extern "C" fn(*mut c_void, *const Tuid, *mut *mut c_void) -> Tresult,
    pub(crate) add_ref: unsafe extern "C" fn(*mut c_void) -> u32,
    pub(crate) release: unsafe extern "C" fn(*mut c_void) -> u32,
    pub(crate) is_platform_type_supported:
        unsafe extern "C" fn(*mut c_void, *const std::ffi::c_char) -> Tresult,
    pub(crate) attached:
        unsafe extern "C" fn(*mut c_void, *mut c_void, *const std::ffi::c_char) -> Tresult,
    pub(crate) removed: unsafe extern "C" fn(*mut c_void) -> Tresult,
    pub(crate) on_wheel: unsafe extern "C" fn(*mut c_void, f32) -> Tresult,
    pub(crate) on_key_down: unsafe extern "C" fn(*mut c_void, u16, i16, i16) -> Tresult,
    pub(crate) on_key_up: unsafe extern "C" fn(*mut c_void, u16, i16, i16) -> Tresult,
    pub(crate) get_size: unsafe extern "C" fn(*mut c_void, *mut ViewRect) -> Tresult,
    pub(crate) on_size: unsafe extern "C" fn(*mut c_void, *mut ViewRect) -> Tresult,
    pub(crate) on_focus: unsafe extern "C" fn(*mut c_void, u8) -> Tresult,
    pub(crate) set_frame: unsafe extern "C" fn(*mut c_void, *mut c_void) -> Tresult,
    pub(crate) can_resize: unsafe extern "C" fn(*mut c_void) -> Tresult,
    pub(crate) check_size_constraint: unsafe extern "C" fn(*mut c_void, *mut ViewRect) -> Tresult,
}
