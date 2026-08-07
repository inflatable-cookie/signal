use std::collections::HashMap;
use std::ffi::{c_char, c_void, CStr};
use std::ptr;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Mutex;

use super::super::com::*;

pub(crate) enum HostAttribute {
    Integer(i64),
    Float(f64),
    String(Vec<i16>),
    Binary(Vec<u8>),
}

#[repr(C)]
pub(crate) struct HostAttributeListVTable {
    pub(crate) query_interface:
        unsafe extern "C" fn(*mut c_void, *const Tuid, *mut *mut c_void) -> Tresult,
    pub(crate) add_ref: unsafe extern "C" fn(*mut c_void) -> u32,
    pub(crate) release: unsafe extern "C" fn(*mut c_void) -> u32,
    pub(crate) set_int: unsafe extern "C" fn(*mut c_void, *const c_char, i64) -> Tresult,
    pub(crate) get_int: unsafe extern "C" fn(*mut c_void, *const c_char, *mut i64) -> Tresult,
    pub(crate) set_float: unsafe extern "C" fn(*mut c_void, *const c_char, f64) -> Tresult,
    pub(crate) get_float: unsafe extern "C" fn(*mut c_void, *const c_char, *mut f64) -> Tresult,
    pub(crate) set_string: unsafe extern "C" fn(*mut c_void, *const c_char, *const i16) -> Tresult,
    pub(crate) get_string:
        unsafe extern "C" fn(*mut c_void, *const c_char, *mut i16, u32) -> Tresult,
    pub(crate) set_binary:
        unsafe extern "C" fn(*mut c_void, *const c_char, *const c_void, u32) -> Tresult,
    pub(crate) get_binary:
        unsafe extern "C" fn(*mut c_void, *const c_char, *mut *const c_void, *mut u32) -> Tresult,
}

#[repr(C)]
pub(crate) struct HostAttributeList {
    pub(crate) vtable: *const HostAttributeListVTable,
    pub(crate) refs: AtomicU32,
    pub(crate) values: Mutex<HashMap<Vec<u8>, HostAttribute>>,
}

pub(crate) static HOST_ATTRIBUTE_LIST_VTABLE: HostAttributeListVTable = HostAttributeListVTable {
    query_interface: host_attribute_list_query_interface,
    add_ref: host_attribute_list_add_ref,
    release: host_attribute_list_release,
    set_int: host_attribute_list_set_int,
    get_int: host_attribute_list_get_int,
    set_float: host_attribute_list_set_float,
    get_float: host_attribute_list_get_float,
    set_string: host_attribute_list_set_string,
    get_string: host_attribute_list_get_string,
    set_binary: host_attribute_list_set_binary,
    get_binary: host_attribute_list_get_binary,
};

pub(crate) fn new_host_attribute_list() -> *mut c_void {
    Box::into_raw(Box::new(HostAttributeList {
        vtable: &HOST_ATTRIBUTE_LIST_VTABLE,
        refs: AtomicU32::new(1),
        values: Mutex::new(HashMap::new()),
    }))
    .cast()
}

pub(crate) unsafe fn attribute_key(id: *const c_char) -> Option<Vec<u8>> {
    (!id.is_null()).then(|| CStr::from_ptr(id).to_bytes().to_vec())
}

unsafe extern "C" fn host_attribute_list_query_interface(
    this: *mut c_void,
    iid: *const Tuid,
    out: *mut *mut c_void,
) -> Tresult {
    if out.is_null() {
        return K_NO_INTERFACE;
    }
    if !iid.is_null() && (*iid == FUNKNOWN_IID || *iid == IATTRIBUTE_LIST_IID) {
        *out = this;
        host_attribute_list_add_ref(this);
        return K_RESULT_OK;
    }
    *out = ptr::null_mut();
    K_NO_INTERFACE
}

unsafe extern "C" fn host_attribute_list_add_ref(this: *mut c_void) -> u32 {
    (*(this.cast::<HostAttributeList>()))
        .refs
        .fetch_add(1, Ordering::Relaxed)
        + 1
}

pub(crate) unsafe extern "C" fn host_attribute_list_release(this: *mut c_void) -> u32 {
    let remaining = (*(this.cast::<HostAttributeList>()))
        .refs
        .fetch_sub(1, Ordering::Release)
        - 1;
    if remaining == 0 {
        std::sync::atomic::fence(Ordering::Acquire);
        drop(Box::from_raw(this.cast::<HostAttributeList>()));
    }
    remaining
}

unsafe extern "C" fn host_attribute_list_set_int(
    this: *mut c_void,
    id: *const c_char,
    value: i64,
) -> Tresult {
    let Some(key) = attribute_key(id) else {
        return K_RESULT_FALSE;
    };
    (*(this.cast::<HostAttributeList>()))
        .values
        .lock()
        .expect("host attribute list poisoned")
        .insert(key, HostAttribute::Integer(value));
    K_RESULT_OK
}

unsafe extern "C" fn host_attribute_list_get_int(
    this: *mut c_void,
    id: *const c_char,
    value: *mut i64,
) -> Tresult {
    let Some(key) = attribute_key(id) else {
        return K_RESULT_FALSE;
    };
    if value.is_null() {
        return K_RESULT_FALSE;
    }
    match (*(this.cast::<HostAttributeList>()))
        .values
        .lock()
        .expect("host attribute list poisoned")
        .get(&key)
    {
        Some(HostAttribute::Integer(stored)) => {
            *value = *stored;
            K_RESULT_OK
        }
        _ => K_RESULT_FALSE,
    }
}

unsafe extern "C" fn host_attribute_list_set_float(
    this: *mut c_void,
    id: *const c_char,
    value: f64,
) -> Tresult {
    let Some(key) = attribute_key(id) else {
        return K_RESULT_FALSE;
    };
    (*(this.cast::<HostAttributeList>()))
        .values
        .lock()
        .expect("host attribute list poisoned")
        .insert(key, HostAttribute::Float(value));
    K_RESULT_OK
}

unsafe extern "C" fn host_attribute_list_get_float(
    this: *mut c_void,
    id: *const c_char,
    value: *mut f64,
) -> Tresult {
    let Some(key) = attribute_key(id) else {
        return K_RESULT_FALSE;
    };
    if value.is_null() {
        return K_RESULT_FALSE;
    }
    match (*(this.cast::<HostAttributeList>()))
        .values
        .lock()
        .expect("host attribute list poisoned")
        .get(&key)
    {
        Some(HostAttribute::Float(stored)) => {
            *value = *stored;
            K_RESULT_OK
        }
        _ => K_RESULT_FALSE,
    }
}

unsafe extern "C" fn host_attribute_list_set_string(
    this: *mut c_void,
    id: *const c_char,
    value: *const i16,
) -> Tresult {
    let Some(key) = attribute_key(id) else {
        return K_RESULT_FALSE;
    };
    if value.is_null() {
        return K_RESULT_FALSE;
    }
    let length = (0..).position(|index| *value.add(index) == 0).unwrap_or(0);
    let mut stored = std::slice::from_raw_parts(value, length).to_vec();
    stored.push(0);
    (*(this.cast::<HostAttributeList>()))
        .values
        .lock()
        .expect("host attribute list poisoned")
        .insert(key, HostAttribute::String(stored));
    K_RESULT_OK
}

unsafe extern "C" fn host_attribute_list_get_string(
    this: *mut c_void,
    id: *const c_char,
    value: *mut i16,
    size_in_bytes: u32,
) -> Tresult {
    let Some(key) = attribute_key(id) else {
        return K_RESULT_FALSE;
    };
    if value.is_null() {
        return K_RESULT_FALSE;
    }
    match (*(this.cast::<HostAttributeList>()))
        .values
        .lock()
        .expect("host attribute list poisoned")
        .get(&key)
    {
        Some(HostAttribute::String(stored)) => {
            let units = stored.len().min(size_in_bytes as usize / size_of::<i16>());
            ptr::copy_nonoverlapping(stored.as_ptr(), value, units);
            K_RESULT_OK
        }
        _ => K_RESULT_FALSE,
    }
}

unsafe extern "C" fn host_attribute_list_set_binary(
    this: *mut c_void,
    id: *const c_char,
    value: *const c_void,
    size_in_bytes: u32,
) -> Tresult {
    let Some(key) = attribute_key(id) else {
        return K_RESULT_FALSE;
    };
    if value.is_null() && size_in_bytes != 0 {
        return K_RESULT_FALSE;
    }
    let stored = if size_in_bytes == 0 {
        Vec::new()
    } else {
        std::slice::from_raw_parts(value.cast::<u8>(), size_in_bytes as usize).to_vec()
    };
    (*(this.cast::<HostAttributeList>()))
        .values
        .lock()
        .expect("host attribute list poisoned")
        .insert(key, HostAttribute::Binary(stored));
    K_RESULT_OK
}

unsafe extern "C" fn host_attribute_list_get_binary(
    this: *mut c_void,
    id: *const c_char,
    value: *mut *const c_void,
    size_in_bytes: *mut u32,
) -> Tresult {
    let Some(key) = attribute_key(id) else {
        return K_RESULT_FALSE;
    };
    if value.is_null() || size_in_bytes.is_null() {
        return K_RESULT_FALSE;
    }
    match (*(this.cast::<HostAttributeList>()))
        .values
        .lock()
        .expect("host attribute list poisoned")
        .get(&key)
    {
        Some(HostAttribute::Binary(stored)) => {
            *value = stored.as_ptr().cast();
            *size_in_bytes = stored.len() as u32;
            K_RESULT_OK
        }
        _ => {
            *value = ptr::null();
            *size_in_bytes = 0;
            K_RESULT_FALSE
        }
    }
}
