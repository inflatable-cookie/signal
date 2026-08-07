pub(super) fn controller_vtable_fragment() -> &'static str {
    r#"static CONTROLLER_VTABLE: EditControllerVTable = EditControllerVTable {
    query_interface: controller_query_interface,
    add_ref: no_op_add_ref,
    release: no_op_release,
    initialize: base_initialize,
    terminate: base_terminate,
    set_component_state: state_noop,
    set_state: state_noop,
    get_state: state_noop,
    get_parameter_count: controller_get_parameter_count,
    get_parameter_info: controller_get_parameter_info,
    get_param_string_by_value: controller_get_param_string_by_value,
    get_param_value_by_string: controller_get_param_value_by_string,
    normalized_param_to_plain: controller_normalized_param_to_plain,
    plain_param_to_normalized: controller_plain_param_to_normalized,
    get_param_normalized: controller_get_param_normalized,
    set_param_normalized: controller_set_param_normalized,
    set_component_handler: controller_set_component_handler,
    create_view: controller_create_view,
};

"#
}

pub(super) fn controller_impl_fragment() -> &'static str {
    r#"unsafe extern "C" fn controller_get_parameter_count(_this: *mut c_void) -> i32 { 2 }

unsafe extern "C" fn controller_get_parameter_info(
    _this: *mut c_void,
    index: i32,
    info: *mut ParameterInfo,
) -> Tresult {
    if info.is_null() {
        return K_RESULT_FALSE;
    }
    let (id, title, unit, flags, step_count, default_value) = match index {
        0 => (4096u32, "Gain", "dB", PARAM_CAN_AUTOMATE, 0, 0.5f64),
        1 => (0u32, "Bypass", "", PARAM_CAN_AUTOMATE | PARAM_IS_BYPASS, 1, 0.0f64),
        _ => return K_RESULT_FALSE,
    };
    let info = &mut *info;
    info.id = id;
    let mut buffer = [0i16; 128];
    write_utf16(&mut buffer, title);
    info.title = buffer;
    info.short_title = buffer;
    let mut unit_buffer = [0i16; 128];
    write_utf16(&mut unit_buffer, unit);
    info.units = unit_buffer;
    info.step_count = step_count;
    info.default_normalized_value = default_value;
    info.unit_id = 0;
    info.flags = flags;
    K_RESULT_OK
}

unsafe extern "C" fn controller_get_param_string_by_value(
    _this: *mut c_void,
    _id: u32,
    value: f64,
    string: *mut i16,
) -> Tresult {
    write_utf16_ptr(string, &format!("{value:.2}"));
    K_RESULT_OK
}

unsafe extern "C" fn controller_get_param_value_by_string(
    _this: *mut c_void,
    _id: u32,
    _string: *mut i16,
    _value: *mut f64,
) -> Tresult {
    K_RESULT_FALSE
}

unsafe extern "C" fn controller_normalized_param_to_plain(
    _this: *mut c_void,
    _id: u32,
    normalized: f64,
) -> f64 {
    normalized
}

unsafe extern "C" fn controller_plain_param_to_normalized(
    _this: *mut c_void,
    _id: u32,
    plain: f64,
) -> f64 {
    plain
}

unsafe extern "C" fn controller_get_param_normalized(_this: *mut c_void, id: u32) -> f64 {
    if id == 4096 { 0.5 } else { 0.0 }
}

unsafe extern "C" fn controller_set_param_normalized(
    _this: *mut c_void,
    _id: u32,
    _value: f64,
) -> Tresult {
    K_RESULT_OK
}

unsafe extern "C" fn controller_set_component_handler(
    _this: *mut c_void,
    _handler: *mut c_void,
) -> Tresult {
    K_RESULT_OK
}

unsafe extern "C" fn controller_create_view(
    _this: *mut c_void,
    name: *const c_char,
) -> *mut c_void {
    // Editor views only, per spec; the returned object is static so the
    // host's create/release probe and open/close cycles are all no-ops.
    if name.is_null() {
        return ptr::null_mut();
    }
    let mut len = 0usize;
    while *name.add(len) != 0 {
        len += 1;
    }
    let requested = std::slice::from_raw_parts(name as *const u8, len);
    if requested == b"editor" {
        view_object()
    } else {
        ptr::null_mut()
    }
}
"#
}
