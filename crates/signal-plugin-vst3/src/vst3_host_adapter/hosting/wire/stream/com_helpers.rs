//! Shared COM vtable helpers.

use std::ffi::c_void;
use std::ptr;

use super::super::com::*;
use super::factory::FUnknownVTable;

/// Read a COM object's vtable of type `V`.
///
/// # Safety
/// `object` must be a live COM interface pointer whose vtable matches `V`.
pub(crate) unsafe fn vtable_of<V>(object: *mut c_void) -> *const V {
    *(object as *mut *const V)
}

/// `FUnknown::queryInterface` returning an owned (addRef'd) pointer.
pub(crate) unsafe fn com_query_interface(object: *mut c_void, iid: &Tuid) -> Option<*mut c_void> {
    let vtable = vtable_of::<FUnknownVTable>(object);
    let mut out: *mut c_void = ptr::null_mut();
    let result = ((*vtable).query_interface)(object, iid, &mut out);
    (result == K_RESULT_OK && !out.is_null()).then_some(out)
}

/// `FUnknown::release`.
pub(crate) unsafe fn com_release(object: *mut c_void) {
    let vtable = vtable_of::<FUnknownVTable>(object);
    ((*vtable).release)(object);
}
