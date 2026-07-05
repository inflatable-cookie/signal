//! CLAP `clap.gui` extension hosting (g12.022, phase 1).
//!
//! Embedded (parented) editor support for the IN-PROCESS tier: the host
//! process owns a native window, hands its content view to the plugin via
//! `set_parent`, and the plugin renders into it. All functions here follow
//! CLAP's main-thread contract — the caller (the Tauri host) must dispatch
//! every call onto the application main thread; this module cannot enforce
//! that (the raw pointers are behind the bridge's locks), it can only
//! document it.
//!
//! Host-side gui callbacks (`request_resize`, `closed`, …) arrive on
//! whatever thread the plugin fires them from and are queued into the host
//! shim's event list; the owner drains them with
//! [`ClapHostedInstance::take_gui_events`](crate::ClapHostedInstance::take_gui_events)
//! and applies window changes itself.

use std::ffi::c_void;

use clap_sys::ext::gui::{clap_plugin_gui, clap_window, clap_window_handle};
use clap_sys::plugin::clap_plugin;

use crate::hosting::ClapHostingError;

/// The window-system API this build targets (macOS first; the packet's
/// phase-1 scope). Used for `is_api_supported` / `create` / `set_parent`.
#[cfg(target_os = "macos")]
pub(crate) const WINDOW_API: &std::ffi::CStr = clap_sys::ext::gui::CLAP_WINDOW_API_COCOA;
#[cfg(target_os = "linux")]
pub(crate) const WINDOW_API: &std::ffi::CStr = clap_sys::ext::gui::CLAP_WINDOW_API_X11;
#[cfg(target_os = "windows")]
pub(crate) const WINDOW_API: &std::ffi::CStr = clap_sys::ext::gui::CLAP_WINDOW_API_WIN32;

/// One host-side gui callback observed from the plugin, drained by the
/// embedding host (which owns the actual window and applies the change).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ClapGuiEvent {
    /// The plugin's resize constraints changed (`resize_hints_changed`).
    ResizeHintsChanged,
    /// The plugin asks the host to resize its window (`request_resize`).
    RequestResize {
        /// Requested content width (logical units on macOS).
        width: u32,
        /// Requested content height (logical units on macOS).
        height: u32,
    },
    /// The plugin asks the host to show its window (`request_show`).
    RequestShow,
    /// The plugin asks the host to hide its window (`request_hide`).
    RequestHide,
    /// The floating window was closed (`closed`). Phase 1 hosts embedded
    /// editors only, but a plugin may still fire this.
    Closed {
        /// `true` when the plugin also destroyed the gui.
        was_destroyed: bool,
    },
}

/// One created `clap.gui` editor on a hosted instance: tracks created/
/// visible state so lifecycle calls stay legal and destroy-on-drop never
/// double-fires. Owned by [`crate::ClapHostedInstance`]; every method runs
/// under the instance's lifecycle serialization and — per the CLAP spec —
/// on the application MAIN THREAD.
pub struct ClapGuiSession {
    plugin: *const clap_plugin,
    gui: *const clap_plugin_gui,
    visible: bool,
    width: u32,
    height: u32,
}

impl ClapGuiSession {
    /// `create` + `get_size` + `set_parent` + `show`: the embedded-editor
    /// open sequence, in spec order. `parent` is the native parent view
    /// (an `NSView*` on macOS). Returns the plugin's initial content size.
    ///
    /// `set_scale` is deliberately not called on macOS (the spec forbids
    /// it there — scaling is the OS's job).
    pub(crate) fn open_embedded(
        plugin: *const clap_plugin,
        gui: *const clap_plugin_gui,
        parent: *mut c_void,
        scale: Option<f64>,
    ) -> Result<Self, ClapHostingError> {
        if parent.is_null() {
            return Err(ClapHostingError::new("gui_parent_null"));
        }
        let create = unsafe { (*gui).create }.ok_or(ClapHostingError::new("gui_create_missing"))?;
        if !unsafe { create(plugin, WINDOW_API.as_ptr(), false) } {
            return Err(ClapHostingError::new("gui_create_failed"));
        }
        // Best-effort scale (non-macOS only; failures are non-fatal).
        #[cfg(not(target_os = "macos"))]
        if let (Some(scale), Some(set_scale)) = (scale, unsafe { (*gui).set_scale }) {
            let _ = unsafe { set_scale(plugin, scale) };
        }
        #[cfg(target_os = "macos")]
        let _ = scale;

        let mut width: u32 = 0;
        let mut height: u32 = 0;
        let sized = unsafe { (*gui).get_size }
            .map(|get_size| unsafe { get_size(plugin, &mut width, &mut height) })
            .unwrap_or(false);
        if !sized || width == 0 || height == 0 {
            unsafe { Self::destroy_raw(plugin, gui) };
            return Err(ClapHostingError::new("gui_get_size_failed"));
        }

        let window = clap_window {
            api: WINDOW_API.as_ptr(),
            specific: parent_handle(parent),
        };
        let parented = unsafe { (*gui).set_parent }
            .map(|set_parent| unsafe { set_parent(plugin, &window) })
            .unwrap_or(false);
        if !parented {
            unsafe { Self::destroy_raw(plugin, gui) };
            return Err(ClapHostingError::new("gui_set_parent_failed"));
        }

        if let Some(show) = unsafe { (*gui).show } {
            // A false return is tolerated: some plugins report un-showable
            // states transiently; the window is still parented.
            let _ = unsafe { show(plugin) };
        }
        Ok(Self {
            plugin,
            gui,
            visible: true,
            width,
            height,
        })
    }

    /// Last observed content size (updated by [`Self::set_size`]).
    pub fn size(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    /// Whether the editor is user-resizable (`can_resize`).
    pub fn can_resize(&self) -> bool {
        unsafe { (*self.gui).can_resize }
            .map(|can_resize| unsafe { can_resize(self.plugin) })
            .unwrap_or(false)
    }

    /// Propose `width`×`height` (host window resized, or granting a
    /// `RequestResize`). Runs the spec's `adjust_size` negotiation first
    /// when available; returns the accepted size on success.
    pub fn set_size(&mut self, width: u32, height: u32) -> Option<(u32, u32)> {
        let mut width = width;
        let mut height = height;
        if let Some(adjust_size) = unsafe { (*self.gui).adjust_size } {
            if !unsafe { adjust_size(self.plugin, &mut width, &mut height) } {
                return None;
            }
        }
        let accepted = unsafe { (*self.gui).set_size }
            .map(|set_size| unsafe { set_size(self.plugin, width, height) })
            .unwrap_or(false);
        if !accepted {
            return None;
        }
        self.width = width;
        self.height = height;
        Some((width, height))
    }

    /// `show` the editor (idempotent per spec).
    pub fn show(&mut self) {
        if let Some(show) = unsafe { (*self.gui).show } {
            if unsafe { show(self.plugin) } {
                self.visible = true;
            }
        }
    }

    /// `hide` the editor (window stays created).
    pub fn hide(&mut self) {
        if let Some(hide) = unsafe { (*self.gui).hide } {
            if unsafe { hide(self.plugin) } {
                self.visible = false;
            }
        }
    }

    /// Whether the last show/hide left the editor visible.
    pub fn is_visible(&self) -> bool {
        self.visible
    }

    pub(crate) unsafe fn destroy_raw(plugin: *const clap_plugin, gui: *const clap_plugin_gui) {
        if let Some(destroy) = (*gui).destroy {
            destroy(plugin);
        }
    }
}

impl Drop for ClapGuiSession {
    fn drop(&mut self) {
        // Owner-driven `gui_destroy` and instance teardown both land here;
        // the session existing implies the gui is still created.
        unsafe { Self::destroy_raw(self.plugin, self.gui) };
    }
}

#[cfg(target_os = "macos")]
fn parent_handle(parent: *mut c_void) -> clap_window_handle {
    clap_window_handle { cocoa: parent }
}

#[cfg(target_os = "linux")]
fn parent_handle(parent: *mut c_void) -> clap_window_handle {
    clap_window_handle {
        x11: parent as std::ffi::c_ulong,
    }
}

#[cfg(target_os = "windows")]
fn parent_handle(parent: *mut c_void) -> clap_window_handle {
    clap_window_handle { win32: parent }
}
