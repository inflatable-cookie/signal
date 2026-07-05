//! In-child CLAP instance hosting: entry/factory loading, instance
//! lifecycle (create/init/activate/start-processing), parameter inventory,
//! and a raw process session for the sandbox audio thread.
//!
//! This module is the FFI half of phase-1 plugin hosting. It runs inside the
//! sandbox child process only — the parent never touches plugin code. The
//! entry/factory loading is shared with discovery (`entry_loading` below), so
//! hosting and scanning speak the same dlopen path.

use std::{
    ffi::{c_char, c_void, CStr, CString},
    path::Path,
    ptr,
    sync::{
        atomic::{AtomicI64, Ordering},
        Mutex,
    },
};

use clap_sys::{
    audio_buffer::clap_audio_buffer,
    entry::clap_plugin_entry,
    events::{clap_event_header, clap_input_events, clap_output_events},
    ext::audio_ports::{clap_plugin_audio_ports, CLAP_EXT_AUDIO_PORTS},
    ext::gui::{clap_host_gui, clap_plugin_gui, CLAP_EXT_GUI},
    factory::plugin_factory::{clap_plugin_factory, CLAP_PLUGIN_FACTORY_ID},
    host::clap_host,
    plugin::clap_plugin,
    process::{clap_process, CLAP_PROCESS_ERROR},
    version::clap_version,
};
use libloading::Library;
use signal_plugin::{PluginAudioBusDirection, PluginParameterDescriptor};

use crate::gui::{ClapGuiEvent, ClapGuiSession};

use crate::discovery::{
    audio_buses_from_extension, parameter_descriptors_from_extension, PluginAudioBusDescriptorList,
};

/// Error surface for hosting operations; carries a stable snake_case token
/// suitable for broker receipt details.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ClapHostingError {
    /// Stable snake_case failure token (e.g. `library_open_failed`).
    pub token: String,
}

impl ClapHostingError {
    pub(crate) fn new(token: impl Into<String>) -> Self {
        Self {
            token: token.into(),
        }
    }
}

impl std::fmt::Display for ClapHostingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.token)
    }
}

impl std::error::Error for ClapHostingError {}

/// A dlopen'd CLAP library with its entry initialized. Shared by discovery
/// and hosting so both speak the same loading path. Deinitializes the entry
/// and closes the library on drop.
pub struct LoadedClapEntry {
    /// Keeps the dynamic library mapped for the entry's lifetime.
    _library: Library,
    entry: *const clap_plugin_entry,
}

impl LoadedClapEntry {
    /// dlopen `library_path`, resolve `clap_entry`, and run its `init`.
    pub fn load(library_path: &Path) -> Result<Self, ClapHostingError> {
        let library = unsafe { Library::new(library_path) }
            .map_err(|_| ClapHostingError::new("library_open_failed"))?;
        let entry = unsafe {
            library
                .get::<*const clap_plugin_entry>(b"clap_entry\0")
                .map_err(|_| ClapHostingError::new("clap_entry_missing"))
                .map(|symbol| *symbol)?
        };
        if entry.is_null() {
            return Err(ClapHostingError::new("clap_entry_null"));
        }
        let plugin_path = CString::new(library_path.to_string_lossy().to_string())
            .map_err(|_| ClapHostingError::new("library_path_invalid"))?;
        if let Some(init) = unsafe { (*entry).init } {
            if !unsafe { init(plugin_path.as_ptr()) } {
                return Err(ClapHostingError::new("entry_init_failed"));
            }
        }
        Ok(Self {
            _library: library,
            entry,
        })
    }

    /// The library's plugin factory, when it exposes one.
    pub fn plugin_factory(&self) -> Option<*const clap_plugin_factory> {
        let get_factory = unsafe { (*self.entry).get_factory }?;
        let factory = unsafe { get_factory(CLAP_PLUGIN_FACTORY_ID.as_ptr()) };
        (!factory.is_null()).then(|| factory.cast::<clap_plugin_factory>())
    }

    /// The raw initialized entry.
    pub(crate) fn entry(&self) -> clap_plugin_entry {
        unsafe { *self.entry }
    }
}

impl Drop for LoadedClapEntry {
    fn drop(&mut self) {
        if let Some(deinit) = unsafe { (*self.entry).deinit } {
            unsafe { deinit() };
        }
    }
}

/// Main-bus stereo port layout summary for a hosted instance.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ClapHostedPortLayout {
    /// Channel count of the main input bus (0 = none).
    pub main_input_channels: u16,
    /// Channel count of the main output bus (0 = none).
    pub main_output_channels: u16,
}

impl ClapHostedPortLayout {
    /// Phase 1 supports exactly a stereo main in + stereo main out effect.
    pub fn is_stereo_effect(&self) -> bool {
        self.main_input_channels == 2 && self.main_output_channels == 2
    }
}

/// Lifecycle state of a hosted instance.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum HostedInstanceState {
    Created,
    Active,
}

/// One live CLAP plugin instance hosted in this process: owns the loaded
/// entry, the host struct handed to the plugin, and the instance pointer.
///
/// Threading: create/activate/deactivate/destroy run on the owning (main)
/// thread; audio processing runs through [`ClapProcessSession`], which the
/// sandbox moves onto its audio thread. While a process session is live the
/// owner must not call lifecycle methods other than after stopping it.
pub struct ClapHostedInstance {
    /// Keeps the library and entry alive for the instance lifetime.
    _entry: LoadedClapEntry,
    /// Host struct + callback state the plugin may retain a pointer to;
    /// boxed so it never moves.
    host_shim: Box<ClapHostShim>,
    plugin: *const clap_plugin,
    state: HostedInstanceState,
    parameters: Vec<PluginParameterDescriptor>,
    port_layout: ClapHostedPortLayout,
    activated_max_frames: u32,
    /// The plugin's `clap.gui` extension, queried once at load (null when
    /// the plugin has no gui).
    gui_extension: *const clap_plugin_gui,
    /// Whether the gui extension supports this platform's embedded window
    /// API (cached `is_api_supported` result from load).
    gui_api_supported: bool,
    /// The live editor, when open. Ordered before `plugin` teardown in
    /// `Drop` (gui destroy must precede plugin destroy).
    gui_session: Option<ClapGuiSession>,
}

impl ClapHostedInstance {
    /// Load `library_path`, create the plugin with `plugin_id` through the
    /// factory, and run its `init`. Enumerates the parameter inventory and
    /// main-bus port layout (descriptor walk, no activation).
    pub fn load(library_path: &Path, plugin_id: &str) -> Result<Self, ClapHostingError> {
        let entry = LoadedClapEntry::load(library_path)?;
        let factory = entry
            .plugin_factory()
            .ok_or_else(|| ClapHostingError::new("plugin_factory_missing"))?;
        let create_plugin = unsafe { (*factory).create_plugin }
            .ok_or_else(|| ClapHostingError::new("factory_create_missing"))?;
        let plugin_id =
            CString::new(plugin_id).map_err(|_| ClapHostingError::new("plugin_id_invalid"))?;
        let mut host_shim = Box::new(ClapHostShim {
            host: sandbox_host(),
            gui_events: Mutex::new(Vec::new()),
        });
        // Self-referential host_data: the shim is boxed (stable address)
        // and outlives the plugin, so callbacks can always recover it.
        host_shim.host.host_data = (&mut *host_shim as *mut ClapHostShim).cast();
        let plugin = unsafe { create_plugin(factory, &host_shim.host, plugin_id.as_ptr()) };
        if plugin.is_null() {
            return Err(ClapHostingError::new("create_plugin_failed"));
        }
        let init_ok = unsafe { (*plugin).init.map(|init| init(plugin)).unwrap_or(true) };
        if !init_ok {
            unsafe {
                if let Some(destroy) = (*plugin).destroy {
                    destroy(plugin);
                }
            }
            return Err(ClapHostingError::new("plugin_init_failed"));
        }

        let (parameters, port_layout) = unsafe { instance_shape(plugin) };
        let (gui_extension, gui_api_supported) = unsafe { gui_shape(plugin) };
        Ok(Self {
            _entry: entry,
            host_shim,
            plugin,
            state: HostedInstanceState::Created,
            parameters,
            port_layout,
            activated_max_frames: 0,
            gui_extension,
            gui_api_supported,
            gui_session: None,
        })
    }

    /// Parameter inventory enumerated at load (read-only phase 1).
    pub fn parameters(&self) -> &[PluginParameterDescriptor] {
        &self.parameters
    }

    /// Main-bus port layout enumerated at load.
    pub fn port_layout(&self) -> ClapHostedPortLayout {
        self.port_layout
    }

    /// Activate the instance for processing at `sample_rate_hz` with the
    /// given block bounds.
    pub fn activate(
        &mut self,
        sample_rate_hz: f64,
        min_frames: u32,
        max_frames: u32,
    ) -> Result<(), ClapHostingError> {
        if self.state == HostedInstanceState::Active {
            return Err(ClapHostingError::new("already_active"));
        }
        let Some(activate) = (unsafe { (*self.plugin).activate }) else {
            return Err(ClapHostingError::new("activate_missing"));
        };
        if !unsafe { activate(self.plugin, sample_rate_hz, min_frames, max_frames) } {
            return Err(ClapHostingError::new("activate_failed"));
        }
        self.state = HostedInstanceState::Active;
        self.activated_max_frames = max_frames;
        Ok(())
    }

    /// Deactivate an active instance (no-op tokened error when inactive).
    pub fn deactivate(&mut self) -> Result<(), ClapHostingError> {
        if self.state != HostedInstanceState::Active {
            return Err(ClapHostingError::new("not_active"));
        }
        if let Some(deactivate) = unsafe { (*self.plugin).deactivate } {
            unsafe { deactivate(self.plugin) };
        }
        self.state = HostedInstanceState::Created;
        Ok(())
    }

    // ── clap.gui hosting (g12.022 phase 1, embedded editors) ───────────
    //
    // MAIN-THREAD CONTRACT: every gui_* method below maps to a CLAP
    // main-thread function. The embedding host must dispatch these onto
    // the application main thread (Tauri `run_on_main_thread`); this type
    // only serializes access, it cannot pick the thread.

    /// Whether the plugin exposes `clap.gui` supporting this platform's
    /// embedded window API (cocoa on macOS). Cached at load.
    pub fn gui_supported(&self) -> bool {
        !self.gui_extension.is_null() && self.gui_api_supported
    }

    /// Whether an editor is currently created.
    pub fn gui_is_open(&self) -> bool {
        self.gui_session.is_some()
    }

    /// Open the embedded editor parented into `parent` (an `NSView*` on
    /// macOS): create → get_size → set_parent → show. Returns the plugin's
    /// initial content size (logical units). Errors with stable tokens
    /// (`gui_unsupported`, `gui_already_open`, `gui_create_failed`, …).
    pub fn gui_open_embedded(
        &mut self,
        parent: *mut c_void,
        scale: Option<f64>,
    ) -> Result<(u32, u32), ClapHostingError> {
        if !self.gui_supported() {
            return Err(ClapHostingError::new("gui_unsupported"));
        }
        if self.gui_session.is_some() {
            return Err(ClapHostingError::new("gui_already_open"));
        }
        let session =
            ClapGuiSession::open_embedded(self.plugin, self.gui_extension, parent, scale)?;
        let size = session.size();
        self.gui_session = Some(session);
        Ok(size)
    }

    /// The open editor, for size/show/hide interaction.
    pub fn gui_session_mut(&mut self) -> Option<&mut ClapGuiSession> {
        self.gui_session.as_mut()
    }

    /// The open editor, read-only.
    pub fn gui_session(&self) -> Option<&ClapGuiSession> {
        self.gui_session.as_ref()
    }

    /// Destroy the open editor (idempotent; the plugin instance stays
    /// live).
    pub fn gui_destroy(&mut self) {
        self.gui_session = None;
    }

    /// Drain host-side gui callbacks queued since the last call
    /// (`request_resize`, `closed`, …). The embedding host applies them to
    /// its window.
    pub fn take_gui_events(&self) -> Vec<ClapGuiEvent> {
        self.host_shim
            .gui_events
            .lock()
            .map(|mut events| std::mem::take(&mut *events))
            .unwrap_or_default()
    }

    /// Build the raw process session for the sandbox audio thread. Only
    /// valid while active; the session preallocates its planar buffers at
    /// the activated max block size, so processing never allocates.
    pub fn process_session(&self) -> Result<ClapProcessSession, ClapHostingError> {
        if self.state != HostedInstanceState::Active {
            return Err(ClapHostingError::new("not_active"));
        }
        Ok(ClapProcessSession::new(
            self.plugin,
            self.activated_max_frames as usize,
        ))
    }
}

impl Drop for ClapHostedInstance {
    fn drop(&mut self) {
        // Gui destroy must precede plugin destroy. This is the fallback
        // path (teardown with an editor still open); the orderly path
        // closes the editor on the main thread first.
        self.gui_session = None;
        unsafe {
            if self.state == HostedInstanceState::Active {
                if let Some(deactivate) = (*self.plugin).deactivate {
                    deactivate(self.plugin);
                }
            }
            if let Some(destroy) = (*self.plugin).destroy {
                destroy(self.plugin);
            }
        }
    }
}

/// Enumerate a live instance's parameters and main-bus port layout.
unsafe fn instance_shape(
    plugin: *const clap_plugin,
) -> (Vec<PluginParameterDescriptor>, ClapHostedPortLayout) {
    let mut parameters = Vec::new();
    let mut layout = ClapHostedPortLayout {
        main_input_channels: 0,
        main_output_channels: 0,
    };
    let Some(get_extension) = (*plugin).get_extension else {
        return (parameters, layout);
    };

    let params_extension = get_extension(plugin, clap_sys::ext::params::CLAP_EXT_PARAMS.as_ptr());
    if !params_extension.is_null() {
        parameters = parameter_descriptors_from_extension(
            plugin,
            params_extension.cast::<clap_sys::ext::params::clap_plugin_params>(),
        );
    }

    let audio_ports = get_extension(plugin, CLAP_EXT_AUDIO_PORTS.as_ptr());
    if !audio_ports.is_null() {
        let buses: PluginAudioBusDescriptorList =
            audio_buses_from_extension(plugin, audio_ports.cast::<clap_plugin_audio_ports>());
        for bus in &buses {
            if !bus.is_main {
                continue;
            }
            match bus.direction {
                PluginAudioBusDirection::Input => layout.main_input_channels = bus.channels,
                PluginAudioBusDirection::Output => layout.main_output_channels = bus.channels,
            }
        }
    }
    (parameters, layout)
}

/// Query the plugin's `clap.gui` extension and whether it supports this
/// platform's embedded window API. Runs at load, on the lifecycle thread.
unsafe fn gui_shape(plugin: *const clap_plugin) -> (*const clap_plugin_gui, bool) {
    let Some(get_extension) = (*plugin).get_extension else {
        return (ptr::null(), false);
    };
    let extension = get_extension(plugin, CLAP_EXT_GUI.as_ptr());
    if extension.is_null() {
        return (ptr::null(), false);
    }
    let gui = extension.cast::<clap_plugin_gui>();
    let api_supported = (*gui)
        .is_api_supported
        .map(|is_api_supported| is_api_supported(plugin, crate::gui::WINDOW_API.as_ptr(), false))
        .unwrap_or(false);
    (gui, api_supported)
}

// ── Host shim (host struct + callback state) ────────────────────────────────

/// The `clap_host` handed to the plugin plus the state its callbacks write
/// into. Boxed by the instance so both have stable addresses for the
/// plugin's lifetime; `host.host_data` points back at the shim.
pub(crate) struct ClapHostShim {
    pub(crate) host: clap_host,
    /// Gui callbacks queued for the embedding host (g12.022). Plugins may
    /// fire these from any thread, hence the mutex.
    pub(crate) gui_events: Mutex<Vec<ClapGuiEvent>>,
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

// ── Raw process session (audio thread) ─────────────────────────────────────

/// Empty input event list: the phase-1 bridge carries audio only.
static EMPTY_IN_EVENTS: clap_input_events = clap_input_events {
    ctx: ptr::null_mut(),
    size: Some(empty_in_events_size),
    get: Some(empty_in_events_get),
};

/// Output event sink that rejects every push (no event transport phase 1).
static SINK_OUT_EVENTS: clap_output_events = clap_output_events {
    ctx: ptr::null_mut(),
    try_push: Some(sink_out_events_try_push),
};

unsafe extern "C" fn empty_in_events_size(_list: *const clap_input_events) -> u32 {
    0
}

unsafe extern "C" fn empty_in_events_get(
    _list: *const clap_input_events,
    _index: u32,
) -> *const clap_event_header {
    ptr::null()
}

unsafe extern "C" fn sink_out_events_try_push(
    _list: *const clap_output_events,
    _event: *const clap_event_header,
) -> bool {
    false
}

/// Raw, movable process handle for one activated instance: the plugin
/// pointer plus preallocated planar stereo buffers. The sandbox moves this
/// onto its audio thread; the owning [`ClapHostedInstance`] must outlive it
/// and must not run lifecycle transitions while the session is live.
pub struct ClapProcessSession {
    plugin: *const clap_plugin,
    input_left: Vec<f32>,
    input_right: Vec<f32>,
    output_left: Vec<f32>,
    output_right: Vec<f32>,
    steady_time: AtomicI64,
    processing: bool,
}

// Safety: the session is handed to exactly one audio thread; CLAP's process
// and start/stop_processing are audio-thread functions, and the owner
// serializes lifecycle against the session per the type contract above.
unsafe impl Send for ClapProcessSession {}

impl ClapProcessSession {
    fn new(plugin: *const clap_plugin, max_frames: usize) -> Self {
        Self {
            plugin,
            input_left: vec![0.0; max_frames],
            input_right: vec![0.0; max_frames],
            output_left: vec![0.0; max_frames],
            output_right: vec![0.0; max_frames],
            steady_time: AtomicI64::new(0),
            processing: false,
        }
    }

    /// `start_processing` on the audio thread; must precede `process`.
    pub fn start(&mut self) -> Result<(), ClapHostingError> {
        if self.processing {
            return Ok(());
        }
        let ok = unsafe {
            (*self.plugin)
                .start_processing
                .map(|start| start(self.plugin))
                .unwrap_or(true)
        };
        if !ok {
            return Err(ClapHostingError::new("start_processing_failed"));
        }
        self.processing = true;
        Ok(())
    }

    /// `stop_processing` on the audio thread.
    pub fn stop(&mut self) {
        if !self.processing {
            return;
        }
        if let Some(stop) = unsafe { (*self.plugin).stop_processing } {
            unsafe { stop(self.plugin) };
        }
        self.processing = false;
    }

    /// Process one block: interleaved stereo in, interleaved stereo out.
    /// Alloc-free (buffers preallocated at activate). On plugin error the
    /// input passes through unchanged. Returns `false` on error.
    pub fn process_interleaved_stereo(
        &mut self,
        input: &[f32],
        output: &mut [f32],
        frame_count: usize,
    ) -> bool {
        let frames = frame_count
            .min(self.input_left.len())
            .min(input.len() / 2)
            .min(output.len() / 2);
        for frame in 0..frames {
            self.input_left[frame] = input[frame * 2];
            self.input_right[frame] = input[frame * 2 + 1];
        }

        let mut input_channels = [self.input_left.as_mut_ptr(), self.input_right.as_mut_ptr()];
        let mut output_channels = [
            self.output_left.as_mut_ptr(),
            self.output_right.as_mut_ptr(),
        ];
        let input_buffer = clap_audio_buffer {
            data32: input_channels.as_mut_ptr(),
            data64: ptr::null_mut(),
            channel_count: 2,
            latency: 0,
            constant_mask: 0,
        };
        let mut output_buffer = clap_audio_buffer {
            data32: output_channels.as_mut_ptr(),
            data64: ptr::null_mut(),
            channel_count: 2,
            latency: 0,
            constant_mask: 0,
        };
        let steady_time = self.steady_time.load(Ordering::Relaxed);
        let process = clap_process {
            steady_time,
            frames_count: frames as u32,
            transport: ptr::null(),
            audio_inputs: &input_buffer,
            audio_outputs: &mut output_buffer,
            audio_inputs_count: 1,
            audio_outputs_count: 1,
            in_events: &EMPTY_IN_EVENTS,
            out_events: &SINK_OUT_EVENTS,
        };
        self.steady_time
            .store(steady_time + frames as i64, Ordering::Relaxed);

        let status = unsafe {
            (*self.plugin)
                .process
                .map(|process_fn| process_fn(self.plugin, &process))
                .unwrap_or(CLAP_PROCESS_ERROR)
        };
        if status == CLAP_PROCESS_ERROR {
            output[..frames * 2].copy_from_slice(&input[..frames * 2]);
            return false;
        }
        for frame in 0..frames {
            output[frame * 2] = self.output_left[frame];
            output[frame * 2 + 1] = self.output_right[frame];
        }
        true
    }

    /// In-place variant for the in-process isolation tier: processes the
    /// interleaved stereo buffer and writes the result back over it ONLY on
    /// success; on plugin error the buffer is left untouched (bypass
    /// semantics). Alloc-free. `true` = buffer transformed.
    pub fn process_in_place(&mut self, io: &mut [f32], frame_count: usize) -> bool {
        let frames = frame_count.min(self.input_left.len()).min(io.len() / 2);
        for frame in 0..frames {
            self.input_left[frame] = io[frame * 2];
            self.input_right[frame] = io[frame * 2 + 1];
        }

        let mut input_channels = [self.input_left.as_mut_ptr(), self.input_right.as_mut_ptr()];
        let mut output_channels = [
            self.output_left.as_mut_ptr(),
            self.output_right.as_mut_ptr(),
        ];
        let input_buffer = clap_audio_buffer {
            data32: input_channels.as_mut_ptr(),
            data64: ptr::null_mut(),
            channel_count: 2,
            latency: 0,
            constant_mask: 0,
        };
        let mut output_buffer = clap_audio_buffer {
            data32: output_channels.as_mut_ptr(),
            data64: ptr::null_mut(),
            channel_count: 2,
            latency: 0,
            constant_mask: 0,
        };
        let steady_time = self.steady_time.load(Ordering::Relaxed);
        let process = clap_process {
            steady_time,
            frames_count: frames as u32,
            transport: ptr::null(),
            audio_inputs: &input_buffer,
            audio_outputs: &mut output_buffer,
            audio_inputs_count: 1,
            audio_outputs_count: 1,
            in_events: &EMPTY_IN_EVENTS,
            out_events: &SINK_OUT_EVENTS,
        };
        self.steady_time
            .store(steady_time + frames as i64, Ordering::Relaxed);

        let status = unsafe {
            (*self.plugin)
                .process
                .map(|process_fn| process_fn(self.plugin, &process))
                .unwrap_or(CLAP_PROCESS_ERROR)
        };
        if status == CLAP_PROCESS_ERROR {
            return false;
        }
        for frame in 0..frames {
            io[frame * 2] = self.output_left[frame];
            io[frame * 2 + 1] = self.output_right[frame];
        }
        true
    }

    /// Whether `start()` has succeeded and `stop()` has not yet run.
    pub fn is_processing(&self) -> bool {
        self.processing
    }
}

fn sandbox_host() -> clap_host {
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
    if CStr::from_ptr(extension_id) == CLAP_EXT_GUI {
        return (&HOST_GUI_EXTENSION as *const clap_host_gui).cast();
    }
    ptr::null()
}

unsafe extern "C" fn sandbox_host_request_restart(_host: *const clap_host) {}
unsafe extern "C" fn sandbox_host_request_process(_host: *const clap_host) {}
unsafe extern "C" fn sandbox_host_request_callback(_host: *const clap_host) {}
