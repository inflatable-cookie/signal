use std::ffi::c_void;
use std::path::Path;
use std::ptr;

use super::super::com::*;
use super::super::stream::*;
use super::attribute_list::new_host_attribute_list;
use super::message::new_host_message;

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

pub(crate) unsafe extern "C" fn host_create_instance(
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
