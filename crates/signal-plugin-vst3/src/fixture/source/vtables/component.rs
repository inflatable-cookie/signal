pub(super) fn component_vtable_fragment(_default_bus_channels: u16) -> &'static str {
    r#"static COMPONENT_VTABLE: ComponentVTable = ComponentVTable {
    query_interface: component_query_interface,
    add_ref: no_op_add_ref,
    release: no_op_release,
    initialize: base_initialize,
    terminate: base_terminate,
    get_controller_class_id: component_get_controller_class_id,
    set_io_mode: component_set_io_mode,
    get_bus_count: component_get_bus_count,
    get_bus_info: component_get_bus_info,
    get_routing_info: component_get_routing_info,
    activate_bus: component_activate_bus,
    set_active: component_set_active,
    set_state: state_noop,
    get_state: state_noop,
};

"#
}

pub(super) fn component_impl_fragment(default_bus_channels: u16) -> String {
    format!(
        r#"unsafe extern "C" fn component_get_controller_class_id(
    _this: *mut c_void,
    _class_id: *mut Tuid,
) -> Tresult {{
    // Single-component plugin: the controller is a facet of this object.
    K_RESULT_FALSE
}}

unsafe extern "C" fn component_set_io_mode(_this: *mut c_void, _mode: i32) -> Tresult {{
    K_RESULT_OK
}}

unsafe extern "C" fn component_get_bus_count(
    _this: *mut c_void,
    media_type: i32,
    _direction: i32,
) -> i32 {{
    if media_type == K_AUDIO {{ 1 }} else {{ 0 }}
}}

unsafe extern "C" fn component_get_bus_info(
    _this: *mut c_void,
    media_type: i32,
    direction: i32,
    index: i32,
    info: *mut BusInfo,
) -> Tresult {{
    if media_type != K_AUDIO || index != 0 || info.is_null() {{
        return K_RESULT_FALSE;
    }}
    let info = &mut *info;
    info.media_type = K_AUDIO;
    info.direction = direction;
    info.channel_count = {default_bus_channels};
    info.bus_type = 0; // kMain
    info.flags = 1; // kDefaultActive
    let mut name = [0i16; 128];
    write_utf16(
        &mut name,
        if direction == K_INPUT {{ "Main Input" }} else {{ "Main Output" }},
    );
    info.name = name;
    K_RESULT_OK
}}

unsafe extern "C" fn component_get_routing_info(
    _this: *mut c_void,
    _input: *mut c_void,
    _output: *mut c_void,
) -> Tresult {{
    K_RESULT_FALSE
}}

unsafe extern "C" fn component_activate_bus(
    _this: *mut c_void,
    media_type: i32,
    _direction: i32,
    index: i32,
    _state: u8,
) -> Tresult {{
    if media_type == K_AUDIO && index == 0 {{ K_RESULT_OK }} else {{ K_RESULT_FALSE }}
}}

unsafe extern "C" fn component_set_active(_this: *mut c_void, _state: u8) -> Tresult {{
    K_RESULT_OK
}}
"#,
        default_bus_channels = default_bus_channels,
    )
}
