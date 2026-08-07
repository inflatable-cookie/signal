use std::ffi::c_void;

use super::super::hosting::{Tresult, Tuid, K_NO_INTERFACE, K_RESULT_OK};
use super::session::Vst3GuiSession;
use super::types::ViewRect;
use super::view::PlugViewVTable;

#[repr(C)]
struct AttachSizedView {
    vtable: *const PlugViewVTable,
    attached: bool,
    removed: bool,
    get_size_calls: u32,
    width: u32,
    height: u32,
    on_size_calls: u32,
    constraint_calls: u32,
}

unsafe extern "C" fn query_interface(
    this: *mut c_void,
    _iid: *const Tuid,
    out: *mut *mut c_void,
) -> Tresult {
    *out = this;
    K_RESULT_OK
}

unsafe extern "C" fn add_ref(_this: *mut c_void) -> u32 {
    1
}

unsafe extern "C" fn release(_this: *mut c_void) -> u32 {
    1
}

unsafe extern "C" fn platform_supported(
    _this: *mut c_void,
    _platform: *const std::ffi::c_char,
) -> Tresult {
    K_RESULT_OK
}

unsafe extern "C" fn attached(
    this: *mut c_void,
    _parent: *mut c_void,
    _platform: *const std::ffi::c_char,
) -> Tresult {
    (*this.cast::<AttachSizedView>()).attached = true;
    K_RESULT_OK
}

unsafe extern "C" fn removed(this: *mut c_void) -> Tresult {
    (*this.cast::<AttachSizedView>()).removed = true;
    K_RESULT_OK
}

unsafe extern "C" fn no_op(_this: *mut c_void) -> Tresult {
    K_RESULT_OK
}

unsafe extern "C" fn no_op_wheel(_this: *mut c_void, _distance: f32) -> Tresult {
    K_RESULT_OK
}

unsafe extern "C" fn no_op_key(
    _this: *mut c_void,
    _key: u16,
    _key_code: i16,
    _modifiers: i16,
) -> Tresult {
    K_RESULT_OK
}

unsafe extern "C" fn get_size(this: *mut c_void, rect: *mut ViewRect) -> Tresult {
    let view = &mut *this.cast::<AttachSizedView>();
    view.get_size_calls += 1;
    if !view.attached {
        return K_NO_INTERFACE;
    }
    *rect = ViewRect::from_size(view.width, view.height);
    K_RESULT_OK
}

unsafe extern "C" fn on_size(this: *mut c_void, rect: *mut ViewRect) -> Tresult {
    let view = &mut *this.cast::<AttachSizedView>();
    (view.width, view.height) = (*rect).size();
    view.on_size_calls += 1;
    K_RESULT_OK
}

unsafe extern "C" fn check_size_constraint(this: *mut c_void, _rect: *mut ViewRect) -> Tresult {
    (*this.cast::<AttachSizedView>()).constraint_calls += 1;
    K_RESULT_OK
}

unsafe extern "C" fn focus_no_op(_this: *mut c_void, _state: u8) -> Tresult {
    K_RESULT_OK
}

unsafe extern "C" fn set_frame_no_op(_this: *mut c_void, _frame: *mut c_void) -> Tresult {
    K_RESULT_OK
}

static ATTACH_SIZED_VIEW_VTABLE: PlugViewVTable = PlugViewVTable {
    query_interface,
    add_ref,
    release,
    is_platform_type_supported: platform_supported,
    attached,
    removed,
    on_wheel: no_op_wheel,
    on_key_down: no_op_key,
    on_key_up: no_op_key,
    get_size,
    on_size,
    on_focus: focus_no_op,
    set_frame: set_frame_no_op,
    can_resize: no_op,
    check_size_constraint,
};

#[test]
fn retries_size_after_attaching_views_that_initialize_late() {
    let mut view = Box::new(AttachSizedView {
        vtable: &ATTACH_SIZED_VIEW_VTABLE,
        attached: false,
        removed: false,
        get_size_calls: 0,
        width: 800,
        height: 600,
        on_size_calls: 0,
        constraint_calls: 0,
    });
    let view_ptr = (&mut *view as *mut AttachSizedView).cast();
    let parent = std::ptr::NonNull::<u8>::dangling().as_ptr().cast();

    let session = unsafe { Vst3GuiSession::open_embedded(view_ptr, parent) }
        .expect("late-initializing view should open");
    assert_eq!(session.size(), (800, 600));
    assert!(view.attached);
    assert_eq!(view.get_size_calls, 2);

    drop(session);
    assert!(view.removed);
}

#[test]
fn plugin_resize_requests_bypass_host_constraints() {
    let mut view = Box::new(AttachSizedView {
        vtable: &ATTACH_SIZED_VIEW_VTABLE,
        attached: false,
        removed: false,
        get_size_calls: 0,
        width: 800,
        height: 600,
        on_size_calls: 0,
        constraint_calls: 0,
    });
    let view_ptr = (&mut *view as *mut AttachSizedView).cast();
    let parent = std::ptr::NonNull::<u8>::dangling().as_ptr().cast();
    let mut session =
        unsafe { Vst3GuiSession::open_embedded(view_ptr, parent) }.expect("view should open");

    assert_eq!(session.accept_plugin_resize(900, 700), Some((900, 700)));
    assert_eq!(view.constraint_calls, 0);
    assert_eq!(view.on_size_calls, 1);

    // A repeated request for the current size is already satisfied.
    assert_eq!(session.accept_plugin_resize(900, 700), Some((900, 700)));
    assert_eq!(view.on_size_calls, 1);

    assert_eq!(session.set_size(1000, 750), Some((1000, 750)));
    assert_eq!(view.constraint_calls, 1);
    assert_eq!(view.on_size_calls, 2);
}
