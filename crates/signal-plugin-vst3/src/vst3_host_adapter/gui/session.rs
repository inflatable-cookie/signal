use std::ffi::c_void;
use std::ptr;
use std::sync::{Arc, Mutex};

use super::super::hosting::{com_release, vtable_of, Vst3HostingError, K_RESULT_OK};
use super::constants::PLATFORM_TYPE;
use super::frame::{HostPlugFrame, PLUG_FRAME_VTABLE};
use super::types::{ViewRect, Vst3GuiEvent};
use super::view::PlugViewVTable;

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

impl std::fmt::Debug for Vst3GuiSession {
    /// Reports the view handle and window geometry. The plug frame is a host
    /// callback object handed to the plugin over the VST3 ABI.
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("Vst3GuiSession")
            .field("view", &self.view)
            .field("attached", &self.attached)
            .field("width", &self.width)
            .field("height", &self.height)
            .finish_non_exhaustive()
    }
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
