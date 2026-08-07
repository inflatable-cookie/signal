use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;

use signal_plugin_vst3::Vst3HostedInstance;

use super::super::common::PluginGuiEvent;

/// In-process VST3 editor host for components that expose a native editor but
/// no processable audio layout.
///
/// This deliberately does not implement [`PluginBlockProcessor`]. It owns the
/// component/controller lifecycle needed for editor inspection, state capture,
/// and state restoration without implying that the component can process
/// audio.
pub struct InProcessVst3Editor {
    instance: Mutex<Vst3HostedInstance>,
    alive: AtomicBool,
}

// Safety: raw COM pointers remain behind `instance`; every public operation is
// serialized by that mutex and the embedding host retains the main-thread
// contract for editor calls.
unsafe impl Send for InProcessVst3Editor {}
unsafe impl Sync for InProcessVst3Editor {}

impl InProcessVst3Editor {
    /// Load one exact VST3 class for isolated editor inspection without
    /// activating an audio process session.
    pub fn load_for_inspection(
        bundle_root: &std::path::Path,
        class_id_hex: &str,
    ) -> Result<Self, String> {
        let instance = Vst3HostedInstance::load_for_inspection(bundle_root, class_id_hex)
            .map_err(|error| error.token)?;
        Ok(Self {
            instance: Mutex::new(instance),
            alive: AtomicBool::new(true),
        })
    }

    /// Whether this component exposes an edit controller that can create an
    /// editor view.
    pub fn gui_supported(&self) -> bool {
        self.alive.load(Ordering::Relaxed)
            && self
                .instance
                .lock()
                .map(|instance| instance.gui_supported())
                .unwrap_or(false)
    }

    /// Open the editor inside the supplied native parent view. Main thread
    /// only.
    pub fn gui_open_embedded(
        &self,
        parent_view: usize,
        scale: Option<f64>,
    ) -> Result<(u32, u32), String> {
        if !self.alive.load(Ordering::Relaxed) {
            return Err("backend_dead".to_string());
        }
        let mut instance = self
            .instance
            .lock()
            .map_err(|_| "instance_lock_poisoned".to_string())?;
        // SAFETY: `parent_view` is the caller's live main-thread view handle,
        // laundered through `usize` so this backend stays `Send`. The caller
        // owns the window and the main-thread contract; this type can only
        // serialize access, it cannot verify either.
        unsafe { instance.gui_open_embedded(parent_view as *mut std::ffi::c_void, scale) }
            .map_err(|error| error.token)
    }

    /// Last observed editor content size, when open.
    pub fn gui_size(&self) -> Option<(u32, u32)> {
        self.instance
            .lock()
            .ok()
            .and_then(|instance| instance.gui_session().map(|session| session.size()))
    }

    /// Whether the open editor accepts host/user resize proposals. Main
    /// thread only.
    pub fn gui_can_resize(&self) -> bool {
        self.instance
            .lock()
            .ok()
            .and_then(|instance| instance.gui_session().map(|session| session.can_resize()))
            .unwrap_or(false)
    }

    /// Propose a host/user editor size. Main thread only.
    pub fn gui_set_size(&self, width: u32, height: u32) -> Option<(u32, u32)> {
        self.instance.lock().ok().and_then(|mut instance| {
            instance
                .gui_session_mut()
                .and_then(|session| session.set_size(width, height))
        })
    }

    /// Accept a plugin-initiated resize request. Main thread only.
    pub fn gui_accept_plugin_resize(&self, width: u32, height: u32) -> Option<(u32, u32)> {
        self.instance.lock().ok().and_then(|mut instance| {
            instance
                .gui_session_mut()
                .and_then(|session| session.accept_plugin_resize(width, height))
        })
    }

    /// Destroy the open editor while keeping the component alive. Main thread
    /// only.
    pub fn gui_close(&self) {
        if let Ok(mut instance) = self.instance.lock() {
            instance.gui_destroy();
        }
    }

    /// Drain queued editor callbacks for the embedding host.
    pub fn gui_take_events(&self) -> Vec<PluginGuiEvent> {
        self.instance
            .lock()
            .map(|instance| {
                instance
                    .take_gui_events()
                    .into_iter()
                    .map(PluginGuiEvent::from)
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Capture opaque component/controller state.
    pub fn save_state(&self) -> Result<Vec<u8>, String> {
        if !self.alive.load(Ordering::Relaxed) {
            return Err("backend_dead".to_string());
        }
        self.instance
            .lock()
            .map_err(|_| "instance_lock_poisoned".to_string())?
            .save_state()
            .map_err(|error| error.token)
    }

    /// Restore opaque component/controller state.
    pub fn load_state(&self, bytes: &[u8]) -> Result<(), String> {
        if !self.alive.load(Ordering::Relaxed) {
            return Err("backend_dead".to_string());
        }
        self.instance
            .lock()
            .map_err(|_| "instance_lock_poisoned".to_string())?
            .load_state(bytes)
            .map_err(|error| error.token)
    }

    /// Mark the editor backend unavailable to future operations.
    pub fn shutdown(&self) {
        self.alive.store(false, Ordering::Relaxed);
    }
}
