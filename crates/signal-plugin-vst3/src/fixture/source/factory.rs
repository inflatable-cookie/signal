pub(crate) fn factory_fragment() -> String {
    r#"// ── Factory ─────────────────────────────────────────────────────────────────

#[repr(C)]
struct FixtureFactory {
    vtable: *const FactoryVTable,
}

unsafe impl Sync for FixtureFactory {}

static FACTORY_VTABLE: FactoryVTable = FactoryVTable {
    query_interface: factory_query_interface,
    add_ref: no_op_add_ref,
    release: no_op_release,
    get_factory_info: factory_get_factory_info,
    count_classes: factory_count_classes,
    get_class_info: factory_get_class_info,
    create_instance: factory_create_instance,
};

static FACTORY: FixtureFactory = FixtureFactory {
    vtable: &FACTORY_VTABLE,
};

fn write_c_chars(dst: &mut [c_char], text: &str) {
    for (slot, byte) in dst.iter_mut().zip(text.bytes().chain(std::iter::once(0))) {
        *slot = byte as c_char;
    }
}

unsafe extern "C" fn factory_query_interface(
    this: *mut c_void,
    iid: *const Tuid,
    out: *mut *mut c_void,
) -> Tresult {
    if out.is_null() {
        return K_NO_INTERFACE;
    }
    if !iid.is_null() && (*iid == FUNKNOWN_IID || *iid == IPLUGIN_FACTORY_IID) {
        *out = this;
        return K_RESULT_OK;
    }
    *out = ptr::null_mut();
    K_NO_INTERFACE
}

unsafe extern "C" fn factory_get_factory_info(
    _this: *mut c_void,
    info: *mut PFactoryInfo,
) -> Tresult {
    if info.is_null() {
        return K_RESULT_FALSE;
    }
    let info = &mut *info;
    info.vendor = [0; 64];
    info.url = [0; 256];
    info.email = [0; 128];
    info.flags = 0x10; // kUnicode
    write_c_chars(&mut info.vendor, "Signal");
    write_c_chars(&mut info.url, "https://signal.dev");
    K_RESULT_OK
}

unsafe extern "C" fn factory_count_classes(_this: *mut c_void) -> i32 { 1 }

unsafe extern "C" fn factory_get_class_info(
    _this: *mut c_void,
    index: i32,
    info: *mut PClassInfo,
) -> Tresult {
    if index != 0 || info.is_null() {
        return K_RESULT_FALSE;
    }
    let info = &mut *info;
    info.cid = FIXTURE_CID;
    info.cardinality = 0x7FFFFFFF; // kManyInstances
    info.category = [0; 32];
    info.name = [0; 64];
    write_c_chars(&mut info.category, "Audio Module Class");
    write_c_chars(&mut info.name, PLUGIN_NAME);
    K_RESULT_OK
}

unsafe extern "C" fn factory_create_instance(
    _this: *mut c_void,
    cid: *const u8,
    iid: *const u8,
    out: *mut *mut c_void,
) -> Tresult {
    if out.is_null() {
        return K_NO_INTERFACE;
    }
    *out = ptr::null_mut();
    if cid.is_null() || iid.is_null() {
        return K_NO_INTERFACE;
    }
    let mut requested_cid: Tuid = [0; 16];
    ptr::copy_nonoverlapping(cid, requested_cid.as_mut_ptr(), 16);
    if requested_cid != FIXTURE_CID {
        return K_NO_INTERFACE;
    }
    shared_query_interface(iid as *const Tuid, out)
}

// ── Module entry points ─────────────────────────────────────────────────────

#[no_mangle]
pub unsafe extern "C" fn GetPluginFactory() -> *mut c_void {
    &FACTORY as *const FixtureFactory as *mut c_void
}

#[no_mangle]
#[cfg(target_os = "macos")]
pub unsafe extern "C" fn bundleEntry(bundle_ref: *mut c_void) -> bool { !bundle_ref.is_null() }
#[cfg(not(target_os = "macos"))]
#[no_mangle]
pub unsafe extern "C" fn bundleEntry(_bundle_ref: *mut c_void) -> bool { true }

#[no_mangle]
pub unsafe extern "C" fn bundleExit() {}

#[no_mangle]
pub unsafe extern "C" fn ModuleEntry(_shared_library_handle: *mut c_void) -> bool { true }

#[no_mangle]
pub unsafe extern "C" fn ModuleExit() {}

#[no_mangle]
pub unsafe extern "C" fn InitDll() -> bool { true }

#[no_mangle]
pub unsafe extern "C" fn ExitDll() {}"#
        .to_string()
}
