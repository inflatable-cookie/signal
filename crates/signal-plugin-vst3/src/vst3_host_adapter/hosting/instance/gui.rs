use std::ffi::c_void;

use crate::vst3_host_adapter::gui::{Vst3GuiEvent, Vst3GuiSession};

use crate::vst3_host_adapter::hosting::Vst3HostingError;

use super::controller::controller_create_view;
use super::hosted::Vst3HostedInstance;

impl Vst3HostedInstance {
    // ── IPlugView hosting (g12.024, GUI phase 2) ───────────────────────
    //
    // MAIN-THREAD CONTRACT: every gui_* method below maps to a VST3
    // UI-thread function. The embedding host must dispatch these onto the
    // application main thread (Tauri `run_on_main_thread`); this type only
    // serializes access, it cannot pick the thread.

    /// Whether the plugin exposes an edit controller that may provide an
    /// editor. `createView("editor")` is deliberately deferred until the
    /// real GUI open: some plugins do not tolerate probe-and-discard or
    /// require their processor to be active first.
    pub fn gui_supported(&self) -> bool {
        self.controller.is_some()
    }

    /// Whether an editor view is currently attached.
    pub fn gui_is_open(&self) -> bool {
        self.gui_session.is_some()
    }

    /// Open the embedded editor parented into `parent` (an `NSView*` on
    /// macOS): `createView("editor")` → platform check → `setFrame` →
    /// `getSize` → `attached`. Returns the plugin's initial content size
    /// (logical units). Errors with stable tokens (`gui_unsupported`,
    /// `gui_already_open`, `gui_attached_failed`, …).
    ///
    /// # Safety
    ///
    /// `parent` must be a live, valid `NSView*` (macOS) or platform window handle owned by the caller, and must
    /// outlive the returned editor session. It is handed straight to the
    /// plugin, which attaches its own view to it. Must be called on the
    /// application main thread.
    pub unsafe fn gui_open_embedded(
        &mut self,
        parent: *mut c_void,
        _scale: Option<f64>,
    ) -> Result<(u32, u32), Vst3HostingError> {
        if self.gui_session.is_some() {
            return Err(Vst3HostingError::new("gui_already_open"));
        }
        let controller = self
            .controller
            .as_ref()
            .ok_or_else(|| Vst3HostingError::new("gui_unsupported"))?;
        let view = unsafe { controller_create_view(controller.ptr()) };
        let session = unsafe { Vst3GuiSession::open_embedded(view, parent) }?;
        let size = session.size();
        self.gui_session = Some(session);
        Ok(size)
    }

    /// The open editor view, for size/resize interaction.
    pub fn gui_session_mut(&mut self) -> Option<&mut Vst3GuiSession> {
        self.gui_session.as_mut()
    }

    /// The open editor view, read-only.
    pub fn gui_session(&self) -> Option<&Vst3GuiSession> {
        self.gui_session.as_ref()
    }

    /// Destroy the open editor view (idempotent; `removed` + release — the
    /// plugin instance stays live).
    pub fn gui_destroy(&mut self) {
        self.gui_session = None;
    }

    /// Drain host-side view callbacks queued since the last call
    /// (`resizeView` requests). Empty when no editor is open.
    pub fn take_gui_events(&self) -> Vec<Vst3GuiEvent> {
        self.gui_session
            .as_ref()
            .map(|session| session.take_events())
            .unwrap_or_default()
    }
}
