//! CLAP host shim and host-extension callbacks.

use std::{
    ffi::{c_char, c_void, CStr},
    ptr,
    sync::{
        atomic::{AtomicU64, Ordering},
        Mutex,
    },
};

use clap_sys::{
    ext::gui::{clap_host_gui, CLAP_EXT_GUI},
    ext::params::{
        clap_host_params, clap_param_clear_flags, clap_param_rescan_flags, CLAP_EXT_PARAMS,
    },
    ext::state::{clap_host_state, CLAP_EXT_STATE},
    host::clap_host,
    version::clap_version,
};

use crate::gui::ClapGuiEvent;

// ── Host shim (host struct + callback state) ────────────────────────────────

/// The `clap_host` handed to the plugin plus the state its callbacks write
/// into. Boxed by the instance so both have stable addresses for the
/// plugin's lifetime; `host.host_data` points back at the shim.
pub(crate) struct ClapHostShim {
    pub(crate) host: clap_host,
    /// Gui callbacks queued for the embedding host (g12.022). Plugins may
    /// fire these from any thread, hence the mutex.
    pub(crate) gui_events: Mutex<Vec<ClapGuiEvent>>,
    /// Host-side `clap.params` callbacks observed from the plugin
    /// (g12.024): rescan/clear/request_flush, queued for the embedding
    /// host to drain.
    pub(crate) params_events: Mutex<Vec<ClapHostParamsEvent>>,
    /// Monotonic, allocation-free notification from `request_restart`.
    pub(crate) restart_requests: AtomicU64,
    /// Monotonic `clap.state` dirty notification for host autosave capture.
    pub(crate) state_dirty_requests: AtomicU64,
}

/// Recover the shim from a host pointer inside a callback. Null when the
/// plugin passed a foreign/never-initialized host.
unsafe fn shim_from_host<'a>(host: *const clap_host) -> Option<&'a ClapHostShim> {
    if host.is_null() {
        return None;
    }
    let shim = (*host).host_data.cast::<ClapHostShim>();
    if shim.is_null() {
        return None;
    }
    Some(&*shim)
}

fn push_gui_event(host: *const clap_host, event: ClapGuiEvent) {
    if let Some(shim) = unsafe { shim_from_host(host) } {
        if let Ok(mut events) = shim.gui_events.lock() {
            events.push(event);
        }
    }
}

/// Host-side `clap.gui` extension (g12.022): every callback queues an event
/// for the embedding host to drain and apply to its window.
static HOST_GUI_EXTENSION: clap_host_gui = clap_host_gui {
    resize_hints_changed: Some(host_gui_resize_hints_changed),
    request_resize: Some(host_gui_request_resize),
    request_show: Some(host_gui_request_show),
    request_hide: Some(host_gui_request_hide),
    closed: Some(host_gui_closed),
};

unsafe extern "C" fn host_gui_resize_hints_changed(host: *const clap_host) {
    push_gui_event(host, ClapGuiEvent::ResizeHintsChanged);
}

unsafe extern "C" fn host_gui_request_resize(
    host: *const clap_host,
    width: u32,
    height: u32,
) -> bool {
    push_gui_event(host, ClapGuiEvent::RequestResize { width, height });
    true
}

unsafe extern "C" fn host_gui_request_show(host: *const clap_host) -> bool {
    push_gui_event(host, ClapGuiEvent::RequestShow);
    true
}

unsafe extern "C" fn host_gui_request_hide(host: *const clap_host) -> bool {
    push_gui_event(host, ClapGuiEvent::RequestHide);
    true
}

unsafe extern "C" fn host_gui_closed(host: *const clap_host, was_destroyed: bool) {
    push_gui_event(host, ClapGuiEvent::Closed { was_destroyed });
}

/// One host-side `clap.params` callback observed from the plugin
/// (g12.024), drained by the embedding host.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ClapHostParamsEvent {
    /// The plugin's parameter inventory or facts changed (`rescan`).
    RescanRequested {
        /// `CLAP_PARAM_RESCAN_*` bit set.
        flags: u32,
    },
    /// The host should clear references to one parameter (`clear`).
    ClearRequested {
        /// The parameter being cleared.
        parameter_id: u32,
        /// `CLAP_PARAM_CLEAR_*` bit set.
        flags: u32,
    },
    /// The plugin asks for a `flush` when the host is not processing
    /// (`request_flush`). The always-running audio path already pumps
    /// events per block, so this is bookkeeping.
    FlushRequested,
}

fn push_params_event(host: *const clap_host, event: ClapHostParamsEvent) {
    if let Some(shim) = unsafe { shim_from_host(host) } {
        if let Ok(mut events) = shim.params_events.lock() {
            events.push(event);
        }
    }
}

/// Host-side `clap.params` extension (g12.024): every callback queues an
/// event for the embedding host to drain — plugin GUI value changes
/// themselves ride the process OUT-EVENTS, not these callbacks.
static HOST_PARAMS_EXTENSION: clap_host_params = clap_host_params {
    rescan: Some(host_params_rescan),
    clear: Some(host_params_clear),
    request_flush: Some(host_params_request_flush),
};

static HOST_STATE_EXTENSION: clap_host_state = clap_host_state {
    mark_dirty: Some(host_state_mark_dirty),
};

unsafe extern "C" fn host_state_mark_dirty(host: *const clap_host) {
    if let Some(shim) = shim_from_host(host) {
        shim.state_dirty_requests.fetch_add(1, Ordering::Relaxed);
    }
}

unsafe extern "C" fn host_params_rescan(host: *const clap_host, flags: clap_param_rescan_flags) {
    push_params_event(host, ClapHostParamsEvent::RescanRequested { flags });
}

unsafe extern "C" fn host_params_clear(
    host: *const clap_host,
    param_id: u32,
    flags: clap_param_clear_flags,
) {
    push_params_event(
        host,
        ClapHostParamsEvent::ClearRequested {
            parameter_id: param_id,
            flags,
        },
    );
}

unsafe extern "C" fn host_params_request_flush(host: *const clap_host) {
    push_params_event(host, ClapHostParamsEvent::FlushRequested);
}

pub(crate) fn sandbox_host() -> clap_host {
    clap_host {
        clap_version: clap_version {
            major: 1,
            minor: 0,
            revision: 0,
        },
        host_data: ptr::null_mut(),
        name: c"Signal Sandbox Host".as_ptr(),
        vendor: c"Signal".as_ptr(),
        url: c"https://signal.dev".as_ptr(),
        version: c"0.1.0".as_ptr(),
        get_extension: Some(sandbox_host_get_extension),
        request_restart: Some(sandbox_host_request_restart),
        request_process: Some(sandbox_host_request_process),
        request_callback: Some(sandbox_host_request_callback),
    }
}

unsafe extern "C" fn sandbox_host_get_extension(
    _host: *const clap_host,
    extension_id: *const c_char,
) -> *const c_void {
    if extension_id.is_null() {
        return ptr::null();
    }
    let extension_id = CStr::from_ptr(extension_id);
    if extension_id == CLAP_EXT_GUI {
        return (&HOST_GUI_EXTENSION as *const clap_host_gui).cast();
    }
    if extension_id == CLAP_EXT_PARAMS {
        return (&HOST_PARAMS_EXTENSION as *const clap_host_params).cast();
    }
    if extension_id == CLAP_EXT_STATE {
        return (&HOST_STATE_EXTENSION as *const clap_host_state).cast();
    }
    ptr::null()
}

unsafe extern "C" fn sandbox_host_request_restart(host: *const clap_host) {
    if let Some(shim) = shim_from_host(host) {
        shim.restart_requests.fetch_add(1, Ordering::Relaxed);
    }
}
unsafe extern "C" fn sandbox_host_request_process(_host: *const clap_host) {}
unsafe extern "C" fn sandbox_host_request_callback(_host: *const clap_host) {}

#[cfg(test)]
mod host_callback_tests {
    use super::*;

    #[test]
    fn restart_callback_advances_the_host_revision() {
        let mut shim = Box::new(ClapHostShim {
            host: sandbox_host(),
            gui_events: Mutex::new(Vec::new()),
            params_events: Mutex::new(Vec::new()),
            restart_requests: AtomicU64::new(0),
            state_dirty_requests: AtomicU64::new(0),
        });
        shim.host.host_data = (&mut *shim as *mut ClapHostShim).cast();

        unsafe { sandbox_host_request_restart(&shim.host) };

        assert_eq!(shim.restart_requests.load(Ordering::Relaxed), 1);
    }

    #[test]
    fn state_dirty_callback_advances_the_host_revision() {
        let mut shim = Box::new(ClapHostShim {
            host: sandbox_host(),
            gui_events: Mutex::new(Vec::new()),
            params_events: Mutex::new(Vec::new()),
            restart_requests: AtomicU64::new(0),
            state_dirty_requests: AtomicU64::new(0),
        });
        shim.host.host_data = (&mut *shim as *mut ClapHostShim).cast();

        unsafe { host_state_mark_dirty(&shim.host) };

        assert_eq!(shim.state_dirty_requests.load(Ordering::Relaxed), 1);
    }
}
