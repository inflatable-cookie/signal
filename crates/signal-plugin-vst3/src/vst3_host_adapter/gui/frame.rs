use std::ffi::c_void;
use std::ptr;
use std::sync::{Arc, Mutex};

use super::super::hosting::{Tresult, Tuid, FUNKNOWN_IID, K_NO_INTERFACE, K_RESULT_OK};
use super::constants::IPLUG_FRAME_IID;
use super::types::{ViewRect, Vst3GuiEvent};

/// `IPlugFrame` (FUnknown + `resizeView`).
#[repr(C)]
pub(crate) struct PlugFrameVTable {
    query_interface: unsafe extern "C" fn(*mut c_void, *const Tuid, *mut *mut c_void) -> Tresult,
    add_ref: unsafe extern "C" fn(*mut c_void) -> u32,
    release: unsafe extern "C" fn(*mut c_void) -> u32,
    resize_view: unsafe extern "C" fn(*mut c_void, *mut c_void, *mut ViewRect) -> Tresult,
}

/// The host frame object handed to `IPlugView::setFrame`: queues the
/// plugin's resize requests for the embedding host. Boxed by the session so
/// its address stays stable for the view's lifetime; refcounting is a no-op
/// (the session owns it and outlives every plugin callback — `setFrame(null)`
/// runs before teardown).
#[repr(C)]
pub(crate) struct HostPlugFrame {
    pub(crate) vtable: *const PlugFrameVTable,
    pub(crate) events: Arc<Mutex<Vec<Vst3GuiEvent>>>,
}

pub(crate) static PLUG_FRAME_VTABLE: PlugFrameVTable = PlugFrameVTable {
    query_interface: frame_query_interface,
    add_ref: frame_add_ref,
    release: frame_release,
    resize_view: frame_resize_view,
};

unsafe extern "C" fn frame_query_interface(
    this: *mut c_void,
    iid: *const Tuid,
    out: *mut *mut c_void,
) -> Tresult {
    if out.is_null() {
        return K_NO_INTERFACE;
    }
    if !iid.is_null() && (*iid == FUNKNOWN_IID || *iid == IPLUG_FRAME_IID) {
        *out = this;
        return K_RESULT_OK;
    }
    *out = ptr::null_mut();
    K_NO_INTERFACE
}

unsafe extern "C" fn frame_add_ref(_this: *mut c_void) -> u32 {
    1
}

unsafe extern "C" fn frame_release(_this: *mut c_void) -> u32 {
    1
}

unsafe extern "C" fn frame_resize_view(
    this: *mut c_void,
    _view: *mut c_void,
    new_size: *mut ViewRect,
) -> Tresult {
    if this.is_null() || new_size.is_null() {
        return K_NO_INTERFACE;
    }
    let frame = &*this.cast::<HostPlugFrame>();
    let (width, height) = (*new_size).size();
    if let Ok(mut events) = frame.events.lock() {
        events.push(Vst3GuiEvent::RequestResize { width, height });
    }
    // Accepted for asynchronous handling: the embedding host drains the
    // event, resizes its window, and grants via `set_size` (→ `onSize`).
    K_RESULT_OK
}
