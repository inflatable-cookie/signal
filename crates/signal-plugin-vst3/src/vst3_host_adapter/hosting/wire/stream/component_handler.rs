//! Minimal IComponentHandler receiving controller edit and restart calls.

use std::ffi::c_void;
use std::ptr;
use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use std::sync::Arc;

use super::super::com::*;
use super::constants::{K_NOT_IMPLEMENTED, RESTART_PROCESSING_MASK, VST3_RESTART_LATENCY_CHANGED};

/// Minimal `IComponentHandler` receiving controller edit and restart calls.
#[repr(C)]
pub(crate) struct ComponentHandlerVTable {
    pub(crate) query_interface:
        unsafe extern "C" fn(*mut c_void, *const Tuid, *mut *mut c_void) -> Tresult,
    pub(crate) add_ref: unsafe extern "C" fn(*mut c_void) -> u32,
    pub(crate) release: unsafe extern "C" fn(*mut c_void) -> u32,
    pub(crate) begin_edit: unsafe extern "C" fn(*mut c_void, u32) -> Tresult,
    pub(crate) perform_edit: unsafe extern "C" fn(*mut c_void, u32, f64) -> Tresult,
    pub(crate) end_edit: unsafe extern "C" fn(*mut c_void, u32) -> Tresult,
    pub(crate) restart_component: unsafe extern "C" fn(*mut c_void, i32) -> Tresult,
}

#[repr(C)]
pub(crate) struct ComponentHandler {
    pub(crate) vtable: *const ComponentHandlerVTable,
    pub(crate) latency_changes: AtomicU64,
    pub(crate) pending_restart_flags: Arc<AtomicU32>,
}

unsafe impl Send for ComponentHandler {}
unsafe impl Sync for ComponentHandler {}

pub(crate) static COMPONENT_HANDLER_VTABLE: ComponentHandlerVTable = ComponentHandlerVTable {
    query_interface: component_handler_query_interface,
    add_ref: component_handler_add_ref,
    release: component_handler_release,
    begin_edit: component_handler_begin_edit,
    perform_edit: component_handler_perform_edit,
    end_edit: component_handler_end_edit,
    restart_component: component_handler_restart_component,
};

unsafe extern "C" fn component_handler_query_interface(
    this: *mut c_void,
    iid: *const Tuid,
    out: *mut *mut c_void,
) -> Tresult {
    if out.is_null() {
        return K_NO_INTERFACE;
    }
    if !iid.is_null() && (*iid == FUNKNOWN_IID || *iid == ICOMPONENT_HANDLER_IID) {
        *out = this;
        return K_RESULT_OK;
    }
    *out = ptr::null_mut();
    K_NO_INTERFACE
}

unsafe extern "C" fn component_handler_add_ref(_this: *mut c_void) -> u32 {
    1
}

unsafe extern "C" fn component_handler_release(_this: *mut c_void) -> u32 {
    1
}

unsafe extern "C" fn component_handler_begin_edit(_this: *mut c_void, _id: u32) -> Tresult {
    K_RESULT_OK
}

unsafe extern "C" fn component_handler_perform_edit(
    _this: *mut c_void,
    _id: u32,
    _value: f64,
) -> Tresult {
    K_RESULT_OK
}

unsafe extern "C" fn component_handler_end_edit(_this: *mut c_void, _id: u32) -> Tresult {
    K_RESULT_OK
}

unsafe extern "C" fn component_handler_restart_component(this: *mut c_void, flags: i32) -> Tresult {
    if this.is_null() {
        return K_NOT_IMPLEMENTED;
    }
    let supported = (flags as u32) & RESTART_PROCESSING_MASK;
    if supported == 0 || supported != flags as u32 {
        return K_NOT_IMPLEMENTED;
    }
    let handler = &*(this.cast::<ComponentHandler>());
    if supported & VST3_RESTART_LATENCY_CHANGED != 0 {
        handler.latency_changes.fetch_add(1, Ordering::Relaxed);
    }
    handler
        .pending_restart_flags
        .fetch_or(supported, Ordering::Release);
    K_RESULT_OK
}

#[cfg(test)]
mod component_handler_tests {
    use super::super::constants::{
        RESTART_PROCESSING_MASK, VST3_RESTART_IO_CHANGED, VST3_RESTART_LATENCY_CHANGED,
    };
    use super::*;

    #[test]
    fn only_supported_processing_restart_flags_are_accepted_and_queued() {
        let pending_restart_flags = Arc::new(AtomicU32::new(0));
        let mut handler = Box::new(ComponentHandler {
            vtable: &COMPONENT_HANDLER_VTABLE,
            latency_changes: AtomicU64::new(0),
            pending_restart_flags: Arc::clone(&pending_restart_flags),
        });
        let ptr = (&mut *handler as *mut ComponentHandler).cast();

        let io =
            unsafe { component_handler_restart_component(ptr, VST3_RESTART_IO_CHANGED as i32) };
        let latency = unsafe {
            component_handler_restart_component(ptr, VST3_RESTART_LATENCY_CHANGED as i32)
        };
        let reload = unsafe { component_handler_restart_component(ptr, 1) };

        assert_eq!(io, K_RESULT_OK);
        assert_eq!(latency, K_RESULT_OK);
        assert_eq!(reload, K_NOT_IMPLEMENTED);
        assert_eq!(handler.latency_changes.load(Ordering::Relaxed), 1);
        assert_eq!(
            pending_restart_flags.load(Ordering::Acquire),
            RESTART_PROCESSING_MASK
        );
    }
}
