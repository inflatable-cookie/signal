use std::ffi::{c_char, c_void, CStr, CString};
use std::ptr;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Mutex;

use super::super::com::*;
use super::attribute_list::{host_attribute_list_release, new_host_attribute_list};

#[repr(C)]
pub(crate) struct HostMessageVTable {
    pub(crate) query_interface:
        unsafe extern "C" fn(*mut c_void, *const Tuid, *mut *mut c_void) -> Tresult,
    pub(crate) add_ref: unsafe extern "C" fn(*mut c_void) -> u32,
    pub(crate) release: unsafe extern "C" fn(*mut c_void) -> u32,
    pub(crate) get_message_id: unsafe extern "C" fn(*mut c_void) -> *const c_char,
    pub(crate) set_message_id: unsafe extern "C" fn(*mut c_void, *const c_char),
    pub(crate) get_attributes: unsafe extern "C" fn(*mut c_void) -> *mut c_void,
}

#[repr(C)]
pub(crate) struct HostMessage {
    pub(crate) vtable: *const HostMessageVTable,
    pub(crate) refs: AtomicU32,
    pub(crate) message_id: Mutex<Option<CString>>,
    pub(crate) attributes: *mut c_void,
}

pub(crate) static HOST_MESSAGE_VTABLE: HostMessageVTable = HostMessageVTable {
    query_interface: host_message_query_interface,
    add_ref: host_message_add_ref,
    release: host_message_release,
    get_message_id: host_message_get_id,
    set_message_id: host_message_set_id,
    get_attributes: host_message_get_attributes,
};

pub(crate) fn new_host_message() -> *mut c_void {
    Box::into_raw(Box::new(HostMessage {
        vtable: &HOST_MESSAGE_VTABLE,
        refs: AtomicU32::new(1),
        message_id: Mutex::new(None),
        attributes: new_host_attribute_list(),
    }))
    .cast()
}

unsafe extern "C" fn host_message_query_interface(
    this: *mut c_void,
    iid: *const Tuid,
    out: *mut *mut c_void,
) -> Tresult {
    if out.is_null() {
        return K_NO_INTERFACE;
    }
    if !iid.is_null() && (*iid == FUNKNOWN_IID || *iid == IMESSAGE_IID) {
        *out = this;
        host_message_add_ref(this);
        return K_RESULT_OK;
    }
    *out = ptr::null_mut();
    K_NO_INTERFACE
}

unsafe extern "C" fn host_message_add_ref(this: *mut c_void) -> u32 {
    (*(this.cast::<HostMessage>()))
        .refs
        .fetch_add(1, Ordering::Relaxed)
        + 1
}

unsafe extern "C" fn host_message_release(this: *mut c_void) -> u32 {
    let message = this.cast::<HostMessage>();
    let remaining = (*message).refs.fetch_sub(1, Ordering::Release) - 1;
    if remaining == 0 {
        std::sync::atomic::fence(Ordering::Acquire);
        host_attribute_list_release((*message).attributes);
        drop(Box::from_raw(message));
    }
    remaining
}

unsafe extern "C" fn host_message_get_id(this: *mut c_void) -> *const c_char {
    (*(this.cast::<HostMessage>()))
        .message_id
        .lock()
        .expect("host message ID poisoned")
        .as_ref()
        .map_or(ptr::null(), |id| id.as_ptr())
}

unsafe extern "C" fn host_message_set_id(this: *mut c_void, id: *const c_char) {
    let value = (!id.is_null()).then(|| CStr::from_ptr(id).to_owned());
    *(*(this.cast::<HostMessage>()))
        .message_id
        .lock()
        .expect("host message ID poisoned") = value;
}

unsafe extern "C" fn host_message_get_attributes(this: *mut c_void) -> *mut c_void {
    (*(this.cast::<HostMessage>())).attributes
}
