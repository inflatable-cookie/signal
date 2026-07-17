//! VST3 `IPlugView` hosting (g12.024, GUI phase 2).
//!
//! Embedded (parented) editor support for the IN-PROCESS tier: the host
//! process owns a native window and hands its content view to the plugin
//! via `IPlugView::attached(parent, "NSView")` — the VST3 mirror of the
//! CLAP `clap.gui` session in `signal-plugin-clap`. The COM surface is
//! handwritten per the g11.031 vtable idiom (`FUnknown` prefix, base
//! methods before derived, `extern "C"` on macOS/Linux).
//!
//! All view methods are UI-THREAD functions per the VST3 threading model;
//! the embedding host must dispatch every call here onto the application
//! main thread (Tauri `run_on_main_thread`) — this module can only
//! document that contract, not enforce it.
//!
//! Plugin-initiated resizes arrive through the host's `IPlugFrame` object
//! (`resizeView`), which queues a [`Vst3GuiEvent`] for the owner to drain
//! and apply to its window (then grant via
//! [`Vst3GuiSession::accept_plugin_resize`], which calls `onSize` without
//! applying the host-only `checkSizeConstraint` step).

#![allow(unsafe_op_in_unsafe_fn)]

use std::ffi::c_void;
use std::ptr;
use std::sync::{Arc, Mutex};

use super::hosting::{
    com_release, tuid_from_uid, vtable_of, Tresult, Tuid, Vst3HostingError, FUNKNOWN_IID,
    K_NO_INTERFACE, K_RESULT_OK,
};

/// `IPlugView` IID (published interface definition, iplugview.h).
pub(crate) const IPLUG_VIEW_IID: Tuid =
    tuid_from_uid(0x5BC32507, 0xD06049EA, 0xA6151B52, 0x2B755B29);
/// `IPlugFrame` IID (published interface definition, iplugview.h).
const IPLUG_FRAME_IID: Tuid = tuid_from_uid(0x367FAF01, 0xAFA94693, 0x8D4DA2A0, 0xED0882A3);

/// The platform window type this build hands to `attached` /
/// `isPlatformTypeSupported` (`kPlatformTypeNSView` on macOS).
#[cfg(target_os = "macos")]
pub(crate) const PLATFORM_TYPE: &std::ffi::CStr = c"NSView";
#[cfg(target_os = "linux")]
pub(crate) const PLATFORM_TYPE: &std::ffi::CStr = c"X11EmbedWindowID";
#[cfg(target_os = "windows")]
pub(crate) const PLATFORM_TYPE: &std::ffi::CStr = c"HWND";

/// The view name passed to `IEditController::createView` (`ViewType::kEditor`).
pub(crate) const VIEW_TYPE_EDITOR: &std::ffi::CStr = c"editor";

/// One host-side view callback observed from the plugin, drained by the
/// embedding host (which owns the actual window and applies the change).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Vst3GuiEvent {
    /// The plugin asks the host to resize its window (`IPlugFrame::resizeView`).
    RequestResize {
        /// Requested content width (logical units on macOS).
        width: u32,
        /// Requested content height (logical units on macOS).
        height: u32,
    },
}

/// `Steinberg::ViewRect`.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct ViewRect {
    left: i32,
    top: i32,
    right: i32,
    bottom: i32,
}

impl ViewRect {
    fn size(&self) -> (u32, u32) {
        (
            (self.right - self.left).max(0) as u32,
            (self.bottom - self.top).max(0) as u32,
        )
    }

    fn from_size(width: u32, height: u32) -> Self {
        Self {
            left: 0,
            top: 0,
            right: width.min(i32::MAX as u32) as i32,
            bottom: height.min(i32::MAX as u32) as i32,
        }
    }
}

/// `IPlugView` (FUnknown + view methods, declaration order per iplugview.h).
#[repr(C)]
pub(crate) struct PlugViewVTable {
    query_interface: unsafe extern "C" fn(*mut c_void, *const Tuid, *mut *mut c_void) -> Tresult,
    add_ref: unsafe extern "C" fn(*mut c_void) -> u32,
    release: unsafe extern "C" fn(*mut c_void) -> u32,
    is_platform_type_supported:
        unsafe extern "C" fn(*mut c_void, *const std::ffi::c_char) -> Tresult,
    attached: unsafe extern "C" fn(*mut c_void, *mut c_void, *const std::ffi::c_char) -> Tresult,
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
}

// ── Host IPlugFrame (plugin-initiated resize requests) ──────────────────────

/// `IPlugFrame` (FUnknown + `resizeView`).
#[repr(C)]
struct PlugFrameVTable {
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
struct HostPlugFrame {
    vtable: *const PlugFrameVTable,
    events: Arc<Mutex<Vec<Vst3GuiEvent>>>,
}

static PLUG_FRAME_VTABLE: PlugFrameVTable = PlugFrameVTable {
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

// ── Session ──────────────────────────────────────────────────────────────────

/// One live `IPlugView` on a hosted VST3 instance: tracks attach state so
/// lifecycle calls stay legal and teardown never double-fires. Owned by
/// [`crate::Vst3HostedInstance`]; every method runs under the instance's
/// lifecycle serialization and — per the VST3 threading model — on the
/// application MAIN THREAD.
pub struct Vst3GuiSession {
    view: *mut c_void,
    /// Boxed for a stable address; the plugin keeps the raw pointer until
    /// `setFrame(null)` at teardown.
    frame: Box<HostPlugFrame>,
    events: Arc<Mutex<Vec<Vst3GuiEvent>>>,
    attached: bool,
    width: u32,
    height: u32,
}

impl Vst3GuiSession {
    /// Platform check + `setFrame` + `getSize` + `attached`: the
    /// embedded-editor open sequence, in spec order, over a freshly
    /// `createView`-ed `view`. `parent` is the native parent view (an
    /// `NSView*` on macOS). Takes ownership of `view` (released on every
    /// error path and on drop). Returns the session with the plugin's
    /// initial content size.
    ///
    /// # Safety
    /// `view` must be an owned, live `IPlugView` pointer (or null, which
    /// errors); `parent` must be a valid platform window handle. Main
    /// thread only.
    pub(crate) unsafe fn open_embedded(
        view: *mut c_void,
        parent: *mut c_void,
    ) -> Result<Self, Vst3HostingError> {
        if view.is_null() {
            return Err(Vst3HostingError::new("gui_unsupported"));
        }
        if parent.is_null() {
            com_release(view);
            return Err(Vst3HostingError::new("gui_parent_null"));
        }
        let vtable = vtable_of::<PlugViewVTable>(view);
        if ((*vtable).is_platform_type_supported)(view, PLATFORM_TYPE.as_ptr()) != K_RESULT_OK {
            com_release(view);
            return Err(Vst3HostingError::new("gui_platform_unsupported"));
        }

        let events: Arc<Mutex<Vec<Vst3GuiEvent>>> = Arc::new(Mutex::new(Vec::new()));
        let frame = Box::new(HostPlugFrame {
            vtable: &PLUG_FRAME_VTABLE,
            events: Arc::clone(&events),
        });
        let frame_ptr = (&*frame as *const HostPlugFrame as *mut HostPlugFrame).cast::<c_void>();
        // setFrame before attached, per the hosting samples; a plugin may
        // legally answer kNotImplemented — resize requests just never come.
        let _ = ((*vtable).set_frame)(view, frame_ptr);

        let mut rect = ViewRect::default();
        let mut size_available = ((*vtable).get_size)(view, &mut rect) == K_RESULT_OK;
        let mut size = rect.size();
        size_available &= size.0 != 0 && size.1 != 0;

        // A few editors (including Native Instruments' QML-based views)
        // report kNotInitialized until they have a native parent. Keep the
        // standard pre-attach sizing path, but attach and retry when needed.
        let mut attached = false;
        if !size_available {
            if ((*vtable).attached)(view, parent, PLATFORM_TYPE.as_ptr()) != K_RESULT_OK {
                let _ = ((*vtable).set_frame)(view, ptr::null_mut());
                com_release(view);
                return Err(Vst3HostingError::new("gui_attached_failed"));
            }
            attached = true;
            rect = ViewRect::default();
            size_available = ((*vtable).get_size)(view, &mut rect) == K_RESULT_OK;
            size = rect.size();
            size_available &= size.0 != 0 && size.1 != 0;
        }

        if !size_available {
            if attached {
                let _ = ((*vtable).removed)(view);
            }
            let _ = ((*vtable).set_frame)(view, ptr::null_mut());
            com_release(view);
            return Err(Vst3HostingError::new("gui_get_size_failed"));
        }
        let (width, height) = size;

        if !attached && ((*vtable).attached)(view, parent, PLATFORM_TYPE.as_ptr()) != K_RESULT_OK {
            let _ = ((*vtable).set_frame)(view, ptr::null_mut());
            com_release(view);
            return Err(Vst3HostingError::new("gui_attached_failed"));
        }

        Ok(Self {
            view,
            frame,
            events,
            attached: true,
            width,
            height,
        })
    }

    /// Last observed content size (updated by [`Self::set_size`]).
    pub fn size(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    /// Whether the editor is user-resizable (`canResize`). MAIN THREAD ONLY.
    pub fn can_resize(&self) -> bool {
        unsafe {
            let vtable = vtable_of::<PlugViewVTable>(self.view);
            ((*vtable).can_resize)(self.view) == K_RESULT_OK
        }
    }

    /// Propose `width`×`height` after a host/user-initiated window resize:
    /// `checkSizeConstraint` negotiation first (the plugin may adjust the
    /// rect), then `onSize`. Returns the accepted size on success. MAIN
    /// THREAD ONLY.
    pub fn set_size(&mut self, width: u32, height: u32) -> Option<(u32, u32)> {
        let mut rect = ViewRect::from_size(width, height);
        unsafe {
            let vtable = vtable_of::<PlugViewVTable>(self.view);
            // checkSizeConstraint may adjust the rect in place; a
            // kNotImplemented answer means "no constraints" — proceed.
            let _ = ((*vtable).check_size_constraint)(self.view, &mut rect);
            let (constrained_width, constrained_height) = rect.size();
            if constrained_width == 0 || constrained_height == 0 {
                return None;
            }
            let mut apply = ViewRect::from_size(constrained_width, constrained_height);
            if ((*vtable).on_size)(self.view, &mut apply) != K_RESULT_OK {
                return None;
            }
            self.width = constrained_width;
            self.height = constrained_height;
            Some((constrained_width, constrained_height))
        }
    }

    /// Accept a size requested through `IPlugFrame::resizeView`. Unlike a
    /// host/user-initiated resize, this path must not call
    /// `checkSizeConstraint`; the plugin has already chosen its size.
    /// MAIN THREAD ONLY.
    pub fn accept_plugin_resize(&mut self, width: u32, height: u32) -> Option<(u32, u32)> {
        if width == 0 || height == 0 {
            return None;
        }
        if (width, height) == self.size() {
            return Some((width, height));
        }
        let mut apply = ViewRect::from_size(width, height);
        unsafe {
            let vtable = vtable_of::<PlugViewVTable>(self.view);
            if ((*vtable).on_size)(self.view, &mut apply) != K_RESULT_OK {
                return None;
            }
        }
        self.width = width;
        self.height = height;
        Some((width, height))
    }

    /// Drain host-side view callbacks queued since the last call
    /// (`resizeView` requests). The embedding host applies them to its
    /// window and grants via [`Self::accept_plugin_resize`].
    pub fn take_events(&self) -> Vec<Vst3GuiEvent> {
        self.events
            .lock()
            .map(|mut events| std::mem::take(&mut *events))
            .unwrap_or_default()
    }

    // Keep the frame box reachable for the linter: the plugin holds a raw
    // pointer into it until teardown, which is the whole point of the box.
    fn frame_ptr(&self) -> *const HostPlugFrame {
        &*self.frame
    }
}

impl Drop for Vst3GuiSession {
    fn drop(&mut self) {
        // Owner-driven `gui_destroy` and instance teardown both land here:
        // removed() while attached, drop the frame reference, then release
        // the view — the mandated ordering before the controller goes away.
        unsafe {
            let vtable = vtable_of::<PlugViewVTable>(self.view);
            if self.attached {
                let _ = ((*vtable).removed)(self.view);
                self.attached = false;
            }
            let _ = ((*vtable).set_frame)(self.view, ptr::null_mut());
            let _ = self.frame_ptr();
            com_release(self.view);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
