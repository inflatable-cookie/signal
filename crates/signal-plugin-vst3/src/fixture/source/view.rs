use super::super::VST3_FIXTURE_VIEW_INITIAL_SIZE;
use super::super::VST3_FIXTURE_VIEW_REQUESTED_SIZE;

pub(crate) fn view_fragment() -> String {
    format!(
        r#"// ── Minimal IPlugView (offscreen bookkeeping, g12.024) ──────────────────────

#[repr(C)]
struct ViewRect {{
    left: i32,
    top: i32,
    right: i32,
    bottom: i32,
}}

#[repr(C)]
struct PlugViewVTable {{
    query_interface: unsafe extern "C" fn(*mut c_void, *const Tuid, *mut *mut c_void) -> Tresult,
    add_ref: unsafe extern "C" fn(*mut c_void) -> u32,
    release: unsafe extern "C" fn(*mut c_void) -> u32,
    is_platform_type_supported: unsafe extern "C" fn(*mut c_void, *const c_char) -> Tresult,
    attached: unsafe extern "C" fn(*mut c_void, *mut c_void, *const c_char) -> Tresult,
    removed: unsafe extern "C" fn(*mut c_void) -> Tresult,
    on_wheel: unsafe extern "C" fn(*mut c_void, f32) -> Tresult,
    on_key_down: unsafe extern "C" fn(*mut c_void, u16, i16, i16) -> Tresult,
    on_key_up: unsafe extern "C" fn(*mut c_void, u16, i16, i16) -> Tresult,
    get_size: unsafe extern "C" fn(*mut c_void, *mut ViewRect) -> Tresult,
    on_size: unsafe extern "C" fn(*mut c_void, *mut ViewRect) -> Tresult,
    on_focus: unsafe extern "C" fn(*mut c_void, u8) -> Tresult,
    set_frame: unsafe extern "C" fn(*mut c_void, *mut c_void) -> Tresult,
    can_resize: unsafe extern "C" fn(*mut c_void) -> Tresult,
    check_size_constraint: unsafe extern "C" fn(*mut c_void, *mut ViewRect) -> Tresult,
}}

#[repr(C)]
struct PlugFrameVTable {{
    query_interface: unsafe extern "C" fn(*mut c_void, *const Tuid, *mut *mut c_void) -> Tresult,
    add_ref: unsafe extern "C" fn(*mut c_void) -> u32,
    release: unsafe extern "C" fn(*mut c_void) -> u32,
    resize_view: unsafe extern "C" fn(*mut c_void, *mut c_void, *mut ViewRect) -> Tresult,
}}

#[repr(C)]
struct FixtureView {{
    vtable: *const PlugViewVTable,
}}

unsafe impl Sync for FixtureView {{}}

static VIEW_VTABLE: PlugViewVTable = PlugViewVTable {{
    query_interface: view_query_interface,
    add_ref: no_op_add_ref,
    release: no_op_release,
    is_platform_type_supported: view_is_platform_type_supported,
    attached: view_attached,
    removed: view_removed,
    on_wheel: view_on_wheel,
    on_key_down: view_on_key,
    on_key_up: view_on_key,
    get_size: view_get_size,
    on_size: view_on_size,
    on_focus: view_on_focus,
    set_frame: view_set_frame,
    can_resize: view_can_resize,
    check_size_constraint: view_check_size_constraint,
}};

static FIXTURE_VIEW: FixtureView = FixtureView {{
    vtable: &VIEW_VTABLE,
}};

/// Offscreen view bookkeeping: parent handle recorded, never dereferenced.
static VIEW_ATTACHED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
static VIEW_WIDTH: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new({view_initial_width});
static VIEW_HEIGHT: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new({view_initial_height});
static VIEW_FRAME: std::sync::atomic::AtomicPtr<c_void> =
    std::sync::atomic::AtomicPtr::new(ptr::null_mut());

fn view_object() -> *mut c_void {{
    &FIXTURE_VIEW as *const FixtureView as *mut c_void
}}

unsafe extern "C" fn view_query_interface(
    this: *mut c_void,
    iid: *const Tuid,
    out: *mut *mut c_void,
) -> Tresult {{
    if out.is_null() {{
        return K_NO_INTERFACE;
    }}
    if !iid.is_null() && (*iid == FUNKNOWN_IID || *iid == IPLUG_VIEW_IID) {{
        *out = this;
        return K_RESULT_OK;
    }}
    *out = ptr::null_mut();
    K_NO_INTERFACE
}}

unsafe extern "C" fn view_is_platform_type_supported(
    _this: *mut c_void,
    _platform_type: *const c_char,
) -> Tresult {{
    // Every platform type: the handle is bookkeeping, never dereferenced.
    K_RESULT_OK
}}

unsafe extern "C" fn view_attached(
    this: *mut c_void,
    parent: *mut c_void,
    _platform_type: *const c_char,
) -> Tresult {{
    if parent.is_null() {{
        return K_RESULT_FALSE;
    }}
    VIEW_WIDTH.store({view_initial_width}, std::sync::atomic::Ordering::SeqCst);
    VIEW_HEIGHT.store({view_initial_height}, std::sync::atomic::Ordering::SeqCst);
    VIEW_ATTACHED.store(true, std::sync::atomic::Ordering::SeqCst);
    // Exercise the host-callback path: ask the host frame for a resize.
    let frame = VIEW_FRAME.load(std::sync::atomic::Ordering::SeqCst);
    if !frame.is_null() {{
        let frame_vtable = *(frame as *mut *const PlugFrameVTable);
        let mut rect = ViewRect {{
            left: 0,
            top: 0,
            right: {view_request_width},
            bottom: {view_request_height},
        }};
        let _ = ((*frame_vtable).resize_view)(frame, this, &mut rect);
    }}
    K_RESULT_OK
}}

unsafe extern "C" fn view_removed(_this: *mut c_void) -> Tresult {{
    VIEW_ATTACHED.store(false, std::sync::atomic::Ordering::SeqCst);
    K_RESULT_OK
}}

unsafe extern "C" fn view_on_wheel(_this: *mut c_void, _distance: f32) -> Tresult {{
    K_RESULT_FALSE
}}

unsafe extern "C" fn view_on_key(
    _this: *mut c_void,
    _key: u16,
    _key_code: i16,
    _modifiers: i16,
) -> Tresult {{
    K_RESULT_FALSE
}}

unsafe extern "C" fn view_get_size(_this: *mut c_void, size: *mut ViewRect) -> Tresult {{
    if size.is_null() {{
        return K_RESULT_FALSE;
    }}
    let size = &mut *size;
    size.left = 0;
    size.top = 0;
    size.right = VIEW_WIDTH.load(std::sync::atomic::Ordering::SeqCst) as i32;
    size.bottom = VIEW_HEIGHT.load(std::sync::atomic::Ordering::SeqCst) as i32;
    K_RESULT_OK
}}

unsafe extern "C" fn view_on_size(_this: *mut c_void, new_size: *mut ViewRect) -> Tresult {{
    if new_size.is_null() {{
        return K_RESULT_FALSE;
    }}
    let new_size = &*new_size;
    VIEW_WIDTH.store(
        (new_size.right - new_size.left).max(0) as u32,
        std::sync::atomic::Ordering::SeqCst,
    );
    VIEW_HEIGHT.store(
        (new_size.bottom - new_size.top).max(0) as u32,
        std::sync::atomic::Ordering::SeqCst,
    );
    K_RESULT_OK
}}

unsafe extern "C" fn view_on_focus(_this: *mut c_void, _state: u8) -> Tresult {{
    K_RESULT_OK
}}

unsafe extern "C" fn view_set_frame(_this: *mut c_void, frame: *mut c_void) -> Tresult {{
    VIEW_FRAME.store(frame, std::sync::atomic::Ordering::SeqCst);
    K_RESULT_OK
}}

unsafe extern "C" fn view_can_resize(_this: *mut c_void) -> Tresult {{
    K_RESULT_OK
}}

unsafe extern "C" fn view_check_size_constraint(
    _this: *mut c_void,
    rect: *mut ViewRect,
) -> Tresult {{
    // No constraints: any proposed rect is accepted unchanged.
    if rect.is_null() {{ K_RESULT_FALSE }} else {{ K_RESULT_OK }}
}}"#,
        view_initial_width = VST3_FIXTURE_VIEW_INITIAL_SIZE.0,
        view_initial_height = VST3_FIXTURE_VIEW_INITIAL_SIZE.1,
        view_request_width = VST3_FIXTURE_VIEW_REQUESTED_SIZE.0,
        view_request_height = VST3_FIXTURE_VIEW_REQUESTED_SIZE.1,
    )
}
