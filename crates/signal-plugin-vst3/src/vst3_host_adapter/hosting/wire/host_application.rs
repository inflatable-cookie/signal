//! VST3 hosting wire: host_application.

use std::collections::HashMap;
use std::ffi::{c_char, c_void, CStr, CString};
use std::path::Path;
use std::ptr;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Mutex;

#[cfg(not(target_os = "macos"))]
use libloading::Library;

#[cfg(not(target_os = "macos"))]
use crate::vst3_host_adapter::introspection::resolve_module_binary_path;

use super::com::*;
use super::stream::*;

// ── Minimal host context (IHostApplication) ────────────────────────────────

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

unsafe extern "C" fn host_attribute_list_release(this: *mut c_void) -> u32 {
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

#[repr(C)]
pub(crate) struct HostApplicationVTable {
    pub(crate) query_interface:
        unsafe extern "C" fn(*mut c_void, *const Tuid, *mut *mut c_void) -> Tresult,
    pub(crate) add_ref: unsafe extern "C" fn(*mut c_void) -> u32,
    pub(crate) release: unsafe extern "C" fn(*mut c_void) -> u32,
    pub(crate) get_name: unsafe extern "C" fn(*mut c_void, *mut i16) -> Tresult,
    pub(crate) create_instance:
        unsafe extern "C" fn(*mut c_void, *mut u8, *mut u8, *mut *mut c_void) -> Tresult,
}

#[repr(C)]
pub(crate) struct StaticHostApplication {
    pub(crate) vtable: *const HostApplicationVTable,
}

// Safety: the static host object is immutable and its methods are
// stateless/thread-safe.
unsafe impl Sync for StaticHostApplication {}

pub(crate) static HOST_APPLICATION_VTABLE: HostApplicationVTable = HostApplicationVTable {
    query_interface: host_query_interface,
    add_ref: host_add_ref,
    release: host_release,
    get_name: host_get_name,
    create_instance: host_create_instance,
};

pub(crate) static HOST_APPLICATION: StaticHostApplication = StaticHostApplication {
    vtable: &HOST_APPLICATION_VTABLE,
};

unsafe extern "C" fn host_query_interface(
    this: *mut c_void,
    iid: *const Tuid,
    out: *mut *mut c_void,
) -> Tresult {
    if out.is_null() {
        return K_NO_INTERFACE;
    }
    if !iid.is_null() && (*iid == FUNKNOWN_IID || *iid == IHOST_APPLICATION_IID) {
        *out = this;
        return K_RESULT_OK;
    }
    *out = ptr::null_mut();
    K_NO_INTERFACE
}

unsafe extern "C" fn host_add_ref(_this: *mut c_void) -> u32 {
    1
}

unsafe extern "C" fn host_release(_this: *mut c_void) -> u32 {
    1
}

unsafe extern "C" fn host_get_name(_this: *mut c_void, name: *mut i16) -> Tresult {
    if name.is_null() {
        return K_NO_INTERFACE;
    }
    let label = "Signal Sandbox Host";
    for (index, unit) in label.encode_utf16().take(127).enumerate() {
        *name.add(index) = unit as i16;
    }
    *name.add(label.encode_utf16().take(127).count()) = 0;
    K_RESULT_OK
}

unsafe extern "C" fn host_create_instance(
    _this: *mut c_void,
    cid: *mut u8,
    iid: *mut u8,
    out: *mut *mut c_void,
) -> Tresult {
    if out.is_null() {
        return K_RESULT_FALSE;
    }
    *out = ptr::null_mut();
    if cid.is_null() || iid.is_null() {
        return K_RESULT_FALSE;
    }
    let cid = &*cid.cast::<Tuid>();
    let iid = &*iid.cast::<Tuid>();
    if *cid == IMESSAGE_IID && *iid == IMESSAGE_IID {
        *out = new_host_message();
        return K_RESULT_OK;
    }
    if *cid == IATTRIBUTE_LIST_IID && *iid == IATTRIBUTE_LIST_IID {
        *out = new_host_attribute_list();
        return K_RESULT_OK;
    }
    K_RESULT_FALSE
}

pub(crate) fn host_context() -> *mut c_void {
    &HOST_APPLICATION as *const StaticHostApplication as *mut c_void
}

/// Supply Signal's standard host context to factories implementing
/// `IPluginFactory3`. Older factories remain valid and require no action.
pub(crate) unsafe fn set_factory_host_context(factory: *mut c_void) -> bool {
    configure_factory_host_context(factory, host_context())
}

/// Clear a factory context before retrying a legacy or application-private
/// factory that rejects ordinary VST3 creation after receiving host context.
pub(crate) unsafe fn clear_factory_host_context(factory: *mut c_void) -> bool {
    configure_factory_host_context(factory, ptr::null_mut())
}

pub(crate) fn should_set_factory_host_context(bundle_root: &Path) -> bool {
    !bundle_root
        .extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| extension.eq_ignore_ascii_case("bundle"))
}

pub(crate) unsafe fn configure_factory_host_context(
    factory: *mut c_void,
    context: *mut c_void,
) -> bool {
    if factory.is_null() {
        return false;
    }
    let vtable = vtable_of::<PluginFactoryVTable>(factory);
    let mut factory_3 = ptr::null_mut();
    if ((*vtable).query_interface)(factory, &IPLUGIN_FACTORY_3_IID, &mut factory_3) != K_RESULT_OK
        || factory_3.is_null()
    {
        return false;
    }
    let factory_3_vtable = vtable_of::<PluginFactory3VTable>(factory_3);
    ((*factory_3_vtable).set_host_context)(factory_3, context);
    com_release(factory_3);
    true
}

#[cfg(test)]
pub(crate) mod host_application_tests {
    use super::*;

    #[test]
    fn skips_factory_context_for_application_private_bundle_components() {
        assert!(!should_set_factory_host_context(Path::new(
            "/Applications/Cubase.app/Contents/Components/Modulation FX.bundle"
        )));
        assert!(should_set_factory_host_context(Path::new(
            "/Library/Audio/Plug-Ins/VST3/Example.vst3"
        )));
    }

    #[test]
    fn creates_messages_with_writable_attributes() {
        unsafe {
            let mut cid = IMESSAGE_IID;
            let mut iid = IMESSAGE_IID;
            let mut message = ptr::null_mut();
            assert_eq!(
                host_create_instance(
                    host_context(),
                    cid.as_mut_ptr(),
                    iid.as_mut_ptr(),
                    &mut message,
                ),
                K_RESULT_OK
            );
            assert!(!message.is_null());

            let message_vtable = vtable_of::<HostMessageVTable>(message);
            let message_id = c"slate-ui-message";
            ((*message_vtable).set_message_id)(message, message_id.as_ptr());
            assert_eq!(
                CStr::from_ptr(((*message_vtable).get_message_id)(message)),
                message_id
            );

            let attributes = ((*message_vtable).get_attributes)(message);
            assert!(!attributes.is_null());
            let attributes_vtable = vtable_of::<HostAttributeListVTable>(attributes);
            let key = c"parameter";
            assert_eq!(
                ((*attributes_vtable).set_int)(attributes, key.as_ptr(), 42),
                K_RESULT_OK
            );
            let mut value = 0;
            assert_eq!(
                ((*attributes_vtable).get_int)(attributes, key.as_ptr(), &mut value),
                K_RESULT_OK
            );
            assert_eq!(value, 42);

            assert_eq!(((*message_vtable).release)(message), 0);
        }
    }

    #[test]
    fn creates_standalone_attribute_lists() {
        unsafe {
            let mut cid = IATTRIBUTE_LIST_IID;
            let mut iid = IATTRIBUTE_LIST_IID;
            let mut attributes = ptr::null_mut();
            assert_eq!(
                host_create_instance(
                    host_context(),
                    cid.as_mut_ptr(),
                    iid.as_mut_ptr(),
                    &mut attributes,
                ),
                K_RESULT_OK
            );
            assert!(!attributes.is_null());
            assert_eq!(host_attribute_list_release(attributes), 0);
        }
    }
}
