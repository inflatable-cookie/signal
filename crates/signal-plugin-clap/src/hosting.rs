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
        atomic::{AtomicI64, AtomicU64, Ordering},
        Mutex,
    },
};

use clap_sys::{
    audio_buffer::clap_audio_buffer,
    entry::clap_plugin_entry,
    events::{
        clap_event_header, clap_event_midi, clap_event_note, clap_event_note_expression,
        clap_event_param_value, clap_event_transport, clap_input_events, clap_output_events,
        CLAP_CORE_EVENT_SPACE_ID, CLAP_EVENT_MIDI, CLAP_EVENT_NOTE_EXPRESSION, CLAP_EVENT_NOTE_OFF,
        CLAP_EVENT_NOTE_ON, CLAP_EVENT_PARAM_VALUE, CLAP_EVENT_TRANSPORT,
        CLAP_NOTE_EXPRESSION_BRIGHTNESS, CLAP_NOTE_EXPRESSION_PRESSURE,
        CLAP_NOTE_EXPRESSION_TUNING, CLAP_TRANSPORT_HAS_BEATS_TIMELINE,
        CLAP_TRANSPORT_HAS_SECONDS_TIMELINE, CLAP_TRANSPORT_HAS_TEMPO,
        CLAP_TRANSPORT_HAS_TIME_SIGNATURE,
    },
    ext::audio_ports::{clap_plugin_audio_ports, CLAP_EXT_AUDIO_PORTS},
    ext::gui::{clap_host_gui, clap_plugin_gui, CLAP_EXT_GUI},
    ext::latency::{clap_plugin_latency, CLAP_EXT_LATENCY},
    ext::params::{
        clap_host_params, clap_param_clear_flags, clap_param_rescan_flags, clap_plugin_params,
        CLAP_EXT_PARAMS,
    },
    ext::state::{clap_host_state, clap_plugin_state, CLAP_EXT_STATE},
    factory::plugin_factory::{clap_plugin_factory, CLAP_PLUGIN_FACTORY_ID},
    fixedpoint::{CLAP_BEATTIME_FACTOR, CLAP_SECTIME_FACTOR},
    host::clap_host,
    plugin::clap_plugin,
    process::{clap_process, CLAP_PROCESS_ERROR},
    stream::{clap_istream, clap_ostream},
    version::clap_version,
};
use libloading::Library;
use signal_plugin::{
    NoteEventKind, NoteExpressionKind, PluginAudioBusDirection, PluginEvent, PluginParamChange,
    PluginParamChangeQueue, PluginParameterDescriptor, PLUGIN_PARAM_CHANGE_CAPACITY,
};
use std::sync::Arc;

use crate::gui::{ClapGuiEvent, ClapGuiSession};

use crate::discovery::{
    audio_buses_from_extension, clap_bundle_binary, parameter_descriptors_from_extension,
    PluginAudioBusDescriptorList,
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
        let load_path = clap_library_binary_path(library_path)?;
        let library = unsafe { Library::new(&load_path) }
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
        let plugin_path =
            CString::new(clap_plugin_path(library_path).to_string_lossy().to_string())
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

fn clap_library_binary_path(library_path: &Path) -> Result<std::path::PathBuf, ClapHostingError> {
    if library_path.is_dir()
        && library_path
            .extension()
            .and_then(|extension| extension.to_str())
            .is_some_and(|extension| extension.eq_ignore_ascii_case("clap"))
    {
        return clap_bundle_binary(library_path)
            .ok_or_else(|| ClapHostingError::new("bundle_binary_missing"));
    }
    Ok(library_path.to_path_buf())
}

fn clap_plugin_path(library_path: &Path) -> &Path {
    library_path
        .ancestors()
        .find(|path| {
            path.extension()
                .and_then(|extension| extension.to_str())
                .is_some_and(|extension| extension.eq_ignore_ascii_case("clap"))
        })
        .unwrap_or(library_path)
}

#[cfg(test)]
mod plugin_path_tests {
    use super::{clap_library_binary_path, clap_plugin_path};
    use std::path::Path;

    #[test]
    fn macos_bundle_binary_initializes_with_outer_clap_path() {
        let binary = Path::new("/Plug-Ins/Example.clap/Contents/MacOS/Example");
        assert_eq!(
            clap_plugin_path(binary),
            Path::new("/Plug-Ins/Example.clap")
        );
    }

    #[test]
    fn standalone_library_initializes_with_its_own_path() {
        let library = Path::new("/usr/lib/clap/example.clap.so");
        assert_eq!(clap_plugin_path(library), library);
    }

    #[test]
    fn macos_bundle_loads_its_contents_binary() {
        let root = std::env::temp_dir().join(format!(
            "signal-clap-bundle-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos(),
        ));
        let bundle = root.join("Example.clap");
        let binary = bundle.join("Contents/MacOS/Example");
        std::fs::create_dir_all(binary.parent().expect("binary parent")).expect("bundle dirs");
        std::fs::write(&binary, b"fixture").expect("bundle binary");

        assert_eq!(
            clap_library_binary_path(&bundle).expect("resolve bundle binary"),
            binary,
        );

        let _ = std::fs::remove_dir_all(root);
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

    /// MIDI instrument layout supported by the current host: no main audio
    /// input and one stereo main output.
    pub fn is_stereo_instrument(&self) -> bool {
        self.main_input_channels == 0 && self.main_output_channels == 2
    }

    /// Whether the current stereo process session can host this layout.
    pub fn is_supported_stereo_processor(&self) -> bool {
        self.is_stereo_effect() || self.is_stereo_instrument()
    }

    /// Whether stereo inspection can safely drive this layout. The first
    /// input/output pair carries the inspection signal; extra input channels
    /// remain silent and extra outputs are ignored. Runtime hosting keeps the
    /// stricter exact-layout gate above.
    pub fn is_supported_stereo_inspection_processor(&self) -> bool {
        self.main_output_channels >= 2
            && (self.main_input_channels == 0 || self.main_input_channels >= 2)
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
    audio_buses: PluginAudioBusDescriptorList,
    activated_sample_rate_hz: f64,
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
    /// Pending param writes bound for the audio thread's process in-events
    /// (g12.023); shared with every process session built from this
    /// instance.
    param_changes: Arc<PluginParamChangeQueue>,
    /// Plugin-originated param values read OUT of the process out-events
    /// (g12.024, plugin GUI → host sync); the audio thread pushes, the
    /// host drains via [`Self::take_param_out_events`]. Values are PLAIN
    /// (CLAP's event domain); the drain converts to normalized.
    param_out: Arc<PluginParamChangeQueue>,
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
            params_events: Mutex::new(Vec::new()),
            restart_requests: AtomicU64::new(0),
            state_dirty_requests: AtomicU64::new(0),
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

        let (parameters, port_layout, audio_buses) = unsafe { instance_shape(plugin) };
        let (gui_extension, gui_api_supported) = unsafe { gui_shape(plugin) };
        Ok(Self {
            _entry: entry,
            host_shim,
            plugin,
            state: HostedInstanceState::Created,
            parameters,
            port_layout,
            audio_buses,
            activated_sample_rate_hz: 0.0,
            activated_max_frames: 0,
            gui_extension,
            gui_api_supported,
            gui_session: None,
            param_changes: Arc::new(PluginParamChangeQueue::new()),
            param_out: Arc::new(PluginParamChangeQueue::new()),
        })
    }

    /// Parameter inventory enumerated at load.
    pub fn parameters(&self) -> &[PluginParameterDescriptor] {
        &self.parameters
    }

    /// Queue one parameter write (g12.023). `normalized` is the host's
    /// 0..1 value; CLAP param events carry PLAIN values, so it maps
    /// linearly onto the descriptor's plain range before queueing. The
    /// audio thread delivers it as a `CLAP_EVENT_PARAM_VALUE` in-event at
    /// the top of the next processed block (block-boundary posture v1).
    pub fn set_parameter_normalized(
        &self,
        parameter_id: u32,
        normalized: f32,
    ) -> Result<(), ClapHostingError> {
        let descriptor = self
            .parameters
            .iter()
            .find(|parameter| parameter.parameter_id == parameter_id)
            .ok_or_else(|| ClapHostingError::new("unknown_parameter"))?;
        let normalized = f64::from(normalized.clamp(0.0, 1.0));
        let plain = f64::from(descriptor.min_plain)
            + normalized * f64::from(descriptor.max_plain - descriptor.min_plain);
        if !self.param_changes.push(parameter_id, plain) {
            return Err(ClapHostingError::new("param_queue_full"));
        }
        Ok(())
    }

    /// Apply queued control-thread parameter writes before an operation such
    /// as state capture that must observe them without waiting for an audio
    /// block. CLAP's params flush is the non-processing delivery path.
    fn flush_parameter_changes(&self) -> Result<(), ClapHostingError> {
        if self.param_changes.is_empty() {
            return Ok(());
        }
        let extension = unsafe {
            (*self.plugin)
                .get_extension
                .map(|get| get(self.plugin, CLAP_EXT_PARAMS.as_ptr()))
                .unwrap_or(ptr::null())
                .cast::<clap_plugin_params>()
        };
        if extension.is_null() {
            return Err(ClapHostingError::new("params_extension_missing"));
        }
        let flush = unsafe { (*extension).flush }
            .ok_or_else(|| ClapHostingError::new("params_flush_missing"))?;

        let mut scratch = Vec::with_capacity(PLUGIN_PARAM_CHANGE_CAPACITY);
        self.param_changes.drain_coalesced(&mut scratch);
        let mut input = Box::new(ParamEventList {
            params: Vec::with_capacity(scratch.len()),
            notes: Vec::new(),
            note_expressions: Vec::new(),
            midi: Vec::new(),
            order: Vec::with_capacity(scratch.len()),
            list: clap_input_events {
                ctx: ptr::null_mut(),
                size: Some(param_in_events_size),
                get: Some(param_in_events_get),
            },
        });
        for change in scratch {
            input.params.push(clap_event_param_value {
                header: clap_event_header {
                    size: std::mem::size_of::<clap_event_param_value>() as u32,
                    time: 0,
                    space_id: CLAP_CORE_EVENT_SPACE_ID,
                    type_: CLAP_EVENT_PARAM_VALUE,
                    flags: 0,
                },
                param_id: change.parameter_id,
                cookie: ptr::null_mut(),
                note_id: -1,
                port_index: -1,
                channel: -1,
                key: -1,
                value: change.value,
            });
            input
                .order
                .push(InEventSlot::Param(input.params.len() as u32 - 1));
        }
        input.list.ctx = (&mut *input as *mut ParamEventList).cast();

        let mut output = Box::new(ParamOutCapture {
            queue: Arc::clone(&self.param_out),
            list: clap_output_events {
                ctx: ptr::null_mut(),
                try_push: Some(param_out_events_try_push),
            },
        });
        output.list.ctx = (&mut *output as *mut ParamOutCapture).cast();
        unsafe { flush(self.plugin, &input.list, &output.list) };
        Ok(())
    }

    /// Capture the plugin's opaque project state through `clap.state`.
    /// Control-thread only; callers must exclude concurrent processing.
    pub fn save_state(&self) -> Result<Vec<u8>, ClapHostingError> {
        self.flush_parameter_changes()?;
        let extension = self.state_extension()?;
        let save = unsafe { (*extension).save }
            .ok_or_else(|| ClapHostingError::new("state_save_missing"))?;
        let mut bytes = Vec::new();
        let stream = clap_ostream {
            ctx: (&mut bytes as *mut Vec<u8>).cast(),
            write: Some(clap_state_write),
        };
        if !unsafe { save(self.plugin, &stream) } {
            return Err(ClapHostingError::new("state_save_failed"));
        }
        Ok(bytes)
    }

    /// Restore an opaque project-state blob through `clap.state`.
    /// Control-thread only; callers must exclude concurrent processing.
    pub fn load_state(&self, bytes: &[u8]) -> Result<(), ClapHostingError> {
        let extension = self.state_extension()?;
        let load = unsafe { (*extension).load }
            .ok_or_else(|| ClapHostingError::new("state_load_missing"))?;
        let mut source = ClapStateReadCursor { bytes, offset: 0 };
        let stream = clap_istream {
            ctx: (&mut source as *mut ClapStateReadCursor<'_>).cast(),
            read: Some(clap_state_read),
        };
        if !unsafe { load(self.plugin, &stream) } {
            return Err(ClapHostingError::new("state_load_failed"));
        }
        Ok(())
    }

    fn state_extension(&self) -> Result<*const clap_plugin_state, ClapHostingError> {
        let extension = unsafe {
            (*self.plugin)
                .get_extension
                .map(|get| get(self.plugin, CLAP_EXT_STATE.as_ptr()))
                .unwrap_or(ptr::null())
        };
        if extension.is_null() {
            return Err(ClapHostingError::new("state_unsupported"));
        }
        Ok(extension.cast())
    }

    /// Main-bus port layout enumerated at load.
    pub fn port_layout(&self) -> ClapHostedPortLayout {
        self.port_layout
    }

    /// Current plugin-reported processing latency in sample frames.
    pub fn latency_frames(&self) -> u32 {
        let extension = unsafe {
            (*self.plugin)
                .get_extension
                .map(|get| get(self.plugin, CLAP_EXT_LATENCY.as_ptr()))
                .unwrap_or(ptr::null())
        };
        if extension.is_null() {
            return 0;
        }
        unsafe {
            (*extension.cast::<clap_plugin_latency>())
                .get
                .map(|get| get(self.plugin))
                .unwrap_or(0)
        }
    }

    /// Number of plugin `request_restart` callbacks observed. CLAP uses a
    /// restart request when processing facts such as latency change; the
    /// embedding host re-queries the latency extension control-side.
    pub fn restart_request_count(&self) -> u64 {
        self.host_shim.restart_requests.load(Ordering::Relaxed)
    }

    /// Number of plugin `clap.state` dirty notifications observed.
    pub fn state_dirty_request_count(&self) -> u64 {
        self.host_shim.state_dirty_requests.load(Ordering::Relaxed)
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
        self.activated_sample_rate_hz = sample_rate_hz;
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

    /// The raw `clap.gui` pair for CHILD-PROCESS (sandbox) editor hosting
    /// (g13.027): the sandbox child's main thread opens and owns the
    /// session itself (see [`crate::ClapGuiRawParts`]). `None` when the
    /// plugin has no gui or the platform window API is unsupported. The
    /// caller assumes the main-thread contract and must drop any session
    /// opened from these parts before this instance is destroyed.
    pub fn gui_raw_parts(&self) -> Option<crate::ClapGuiRawParts> {
        if !self.gui_supported() {
            return None;
        }
        Some(crate::ClapGuiRawParts {
            plugin: self.plugin,
            gui: self.gui_extension,
        })
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

    /// Drain plugin-originated param values captured from the process
    /// out-events since the last call (g12.024, plugin GUI → host sync).
    /// CLAP param events carry PLAIN values, so each drains as
    /// `(parameter_id, normalized 0..1)` mapped through the descriptor's
    /// plain range; parameters missing from the inventory are dropped.
    pub fn take_param_out_events(&self) -> Vec<(u32, f32)> {
        if self.param_out.is_empty() {
            return Vec::new();
        }
        let mut scratch: Vec<PluginParamChange> = Vec::with_capacity(PLUGIN_PARAM_CHANGE_CAPACITY);
        self.param_out.drain_coalesced(&mut scratch);
        scratch
            .iter()
            .filter_map(|change| {
                let descriptor = self
                    .parameters
                    .iter()
                    .find(|parameter| parameter.parameter_id == change.parameter_id)?;
                let range = f64::from(descriptor.max_plain) - f64::from(descriptor.min_plain);
                let normalized = if range.abs() > f64::EPSILON {
                    ((change.value - f64::from(descriptor.min_plain)) / range).clamp(0.0, 1.0)
                } else {
                    0.0
                };
                Some((change.parameter_id, normalized as f32))
            })
            .collect()
    }

    /// Drain host-side `clap.params` callbacks queued since the last call
    /// (rescan / clear / request_flush). The active audio path already
    /// pumps in/out events every block, so `RequestFlush` needs no extra
    /// host action while a plan runs; the events stay observable for the
    /// embedding host's bookkeeping.
    pub fn take_params_events(&self) -> Vec<ClapHostParamsEvent> {
        self.host_shim
            .params_events
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
            self.activated_sample_rate_hz,
            self.activated_max_frames as usize,
            &self.audio_buses,
            Arc::clone(&self.param_changes),
            Arc::clone(&self.param_out),
        ))
    }
}

unsafe extern "C" fn clap_state_write(
    stream: *const clap_ostream,
    buffer: *const c_void,
    size: u64,
) -> i64 {
    if stream.is_null() || buffer.is_null() || size > i64::MAX as u64 {
        return -1;
    }
    let bytes = &mut *((*stream).ctx as *mut Vec<u8>);
    let input = std::slice::from_raw_parts(buffer.cast::<u8>(), size as usize);
    bytes.extend_from_slice(input);
    size as i64
}

struct ClapStateReadCursor<'a> {
    bytes: &'a [u8],
    offset: usize,
}

unsafe extern "C" fn clap_state_read(
    stream: *const clap_istream,
    buffer: *mut c_void,
    size: u64,
) -> i64 {
    if stream.is_null() || buffer.is_null() || size > i64::MAX as u64 {
        return -1;
    }
    let source = &mut *((*stream).ctx as *mut ClapStateReadCursor<'_>);
    let remaining = source.bytes.len().saturating_sub(source.offset);
    let count = remaining.min(size as usize);
    if count > 0 {
        ptr::copy_nonoverlapping(
            source.bytes.as_ptr().add(source.offset),
            buffer.cast(),
            count,
        );
        source.offset += count;
    }
    count as i64
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
) -> (
    Vec<PluginParameterDescriptor>,
    ClapHostedPortLayout,
    PluginAudioBusDescriptorList,
) {
    let mut parameters = Vec::new();
    let mut buses = Vec::new();
    let mut layout = ClapHostedPortLayout {
        main_input_channels: 0,
        main_output_channels: 0,
    };
    let Some(get_extension) = (*plugin).get_extension else {
        return (parameters, layout, buses);
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
        buses = audio_buses_from_extension(plugin, audio_ports.cast::<clap_plugin_audio_ports>());
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
    (parameters, layout, buses)
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

// ── Raw process session (audio thread) ─────────────────────────────────────

/// Empty input event list, served when no param change is pending
/// (g12.023: pending changes ride a session-owned event list instead).
static EMPTY_IN_EVENTS: clap_input_events = clap_input_events {
    ctx: ptr::null_mut(),
    size: Some(empty_in_events_size),
    get: Some(empty_in_events_get),
};

/// The session-owned out-events capture served to the plugin through
/// `clap_output_events` (g12.024): PARAM_VALUE events land in the shared
/// plugin→host queue (alloc-free ring push on the audio thread); every
/// other event type is accepted-and-dropped (no event transport yet).
/// Boxed by the session so the `ctx` pointer stays stable.
struct ParamOutCapture {
    queue: Arc<PluginParamChangeQueue>,
    /// The `clap_output_events` handed to the plugin; `ctx` points back at
    /// this boxed struct.
    list: clap_output_events,
}

unsafe extern "C" fn param_out_events_try_push(
    list: *const clap_output_events,
    event: *const clap_event_header,
) -> bool {
    if list.is_null() || (*list).ctx.is_null() || event.is_null() {
        return false;
    }
    if (*event).space_id == CLAP_CORE_EVENT_SPACE_ID
        && (*event).type_ == CLAP_EVENT_PARAM_VALUE
        && (*event).size as usize >= std::mem::size_of::<clap_event_param_value>()
    {
        let capture = &*(*list).ctx.cast::<ParamOutCapture>();
        let value_event = &*event.cast::<clap_event_param_value>();
        // A full ring still reports the push accepted: the ring coalesces
        // last-write-wins per drain, and refusing would make plugins spin.
        let _ = capture.queue.push(value_event.param_id, value_event.value);
    }
    true
}

unsafe extern "C" fn empty_in_events_size(_list: *const clap_input_events) -> u32 {
    0
}

unsafe extern "C" fn empty_in_events_get(
    _list: *const clap_input_events,
    _index: u32,
) -> *const clap_event_header {
    ptr::null()
}

/// Per-block cap on note/MIDI in-events forwarded to the plugin (matches
/// the render plane's per-block event capacity; overflow drops, earliest
/// wins — never an allocation on the audio thread).
const IN_EVENT_CAPACITY: usize = 1024;

/// Which backing array an in-event order entry points into.
#[derive(Clone, Copy)]
enum InEventSlot {
    Param(u32),
    Note(u32),
    NoteExpression(u32),
    Midi(u32),
}

/// The session-owned in-event list served to the plugin through
/// `clap_input_events` (g12.023, widened for note/CC delivery). Boxed by
/// the session so the `ctx` pointer inside the embedded
/// `clap_input_events` stays stable while the session moves between
/// threads. Rebuilt at the top of every processed block — param writes
/// from the shared change queue land at time offset 0 (block-boundary
/// posture), note/MIDI events keep their intra-block sample offsets. All
/// vecs are preallocated; the audio thread never allocates.
///
/// This is the MIDI 1.0 downconversion boundary for CLAP CC delivery:
/// [`PluginEvent::ControlChange`] values (normalized f32) become 3-byte
/// `clap_event_midi` messages here (`round(value * 127)`); note events use
/// CLAP's native `clap_event_note` and keep full float velocity.
struct ParamEventList {
    params: Vec<clap_event_param_value>,
    notes: Vec<clap_event_note>,
    note_expressions: Vec<clap_event_note_expression>,
    midi: Vec<clap_event_midi>,
    /// Delivery order (nondecreasing header time, params first at 0).
    order: Vec<InEventSlot>,
    /// The `clap_input_events` handed to the plugin; `ctx` points back at
    /// this boxed struct.
    list: clap_input_events,
}

unsafe extern "C" fn param_in_events_size(list: *const clap_input_events) -> u32 {
    if list.is_null() || (*list).ctx.is_null() {
        return 0;
    }
    (*(*list).ctx.cast::<ParamEventList>()).order.len() as u32
}

unsafe extern "C" fn param_in_events_get(
    list: *const clap_input_events,
    index: u32,
) -> *const clap_event_header {
    if list.is_null() || (*list).ctx.is_null() {
        return ptr::null();
    }
    let events = &(*(*list).ctx.cast::<ParamEventList>());
    match events.order.get(index as usize) {
        Some(InEventSlot::Param(slot)) => events
            .params
            .get(*slot as usize)
            .map(|event| (&event.header as *const clap_event_header).cast())
            .unwrap_or(ptr::null()),
        Some(InEventSlot::Note(slot)) => events
            .notes
            .get(*slot as usize)
            .map(|event| (&event.header as *const clap_event_header).cast())
            .unwrap_or(ptr::null()),
        Some(InEventSlot::NoteExpression(slot)) => events
            .note_expressions
            .get(*slot as usize)
            .map(|event| (&event.header as *const clap_event_header).cast())
            .unwrap_or(ptr::null()),
        Some(InEventSlot::Midi(slot)) => events
            .midi
            .get(*slot as usize)
            .map(|event| (&event.header as *const clap_event_header).cast())
            .unwrap_or(ptr::null()),
        None => ptr::null(),
    }
}

struct ClapAudioBusBuffers {
    samples: Vec<Vec<Vec<f32>>>,
    _channel_pointers: Vec<Vec<*mut f32>>,
    descriptors: Vec<clap_audio_buffer>,
}

impl ClapAudioBusBuffers {
    fn new(channel_counts: &[usize], max_frames: usize) -> Self {
        let mut samples = channel_counts
            .iter()
            .map(|&channel_count| vec![vec![0.0; max_frames]; channel_count])
            .collect::<Vec<_>>();
        let mut channel_pointers = samples
            .iter_mut()
            .map(|channels| {
                channels
                    .iter_mut()
                    .map(|channel| channel.as_mut_ptr())
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();
        let descriptors = channel_pointers
            .iter_mut()
            .map(|channels| clap_audio_buffer {
                data32: if channels.is_empty() {
                    ptr::null_mut()
                } else {
                    channels.as_mut_ptr()
                },
                data64: ptr::null_mut(),
                channel_count: channels.len() as u32,
                latency: 0,
                constant_mask: 0,
            })
            .collect();
        Self {
            samples,
            _channel_pointers: channel_pointers,
            descriptors,
        }
    }

    fn clear(&mut self, frames: usize) {
        for bus in &mut self.samples {
            for channel in bus {
                channel[..frames].fill(0.0);
            }
        }
    }

    fn copy_interleaved_stereo_into(&mut self, bus_index: usize, input: &[f32], frames: usize) {
        let Some(bus) = self.samples.get_mut(bus_index) else {
            return;
        };
        let [left, right, ..] = bus.as_mut_slice() else {
            return;
        };
        for frame in 0..frames {
            left[frame] = input[frame * 2];
            right[frame] = input[frame * 2 + 1];
        }
    }

    fn copy_interleaved_stereo_from(&self, bus_index: usize, output: &mut [f32], frames: usize) {
        let Some(bus) = self.samples.get(bus_index) else {
            return;
        };
        let [left, right, ..] = bus.as_slice() else {
            return;
        };
        for frame in 0..frames {
            output[frame * 2] = left[frame];
            output[frame * 2 + 1] = right[frame];
        }
    }

    fn as_ptr(&self) -> *const clap_audio_buffer {
        if self.descriptors.is_empty() {
            ptr::null()
        } else {
            self.descriptors.as_ptr()
        }
    }

    fn as_mut_ptr(&mut self) -> *mut clap_audio_buffer {
        if self.descriptors.is_empty() {
            ptr::null_mut()
        } else {
            self.descriptors.as_mut_ptr()
        }
    }

    fn len(&self) -> u32 {
        self.descriptors.len() as u32
    }
}

/// Raw, movable process handle for one activated instance: the plugin
/// pointer plus preallocated planar audio-bus buffers. The sandbox moves this
/// onto its audio thread; the owning [`ClapHostedInstance`] must outlive it
/// and must not run lifecycle transitions while the session is live.
pub struct ClapProcessSession {
    plugin: *const clap_plugin,
    sample_rate_hz: f64,
    input_buses: ClapAudioBusBuffers,
    output_buses: ClapAudioBusBuffers,
    main_input_bus: Option<usize>,
    main_output_bus: usize,
    max_frames: usize,
    steady_time: AtomicI64,
    processing: bool,
    /// Pending param writes shared with the owning instance (g12.023).
    param_changes: Arc<PluginParamChangeQueue>,
    /// Drain scratch (preallocated; audio thread never allocates).
    param_scratch: Vec<PluginParamChange>,
    /// The in-event list served to the plugin, rebuilt per block.
    param_events: Box<ParamEventList>,
    /// The out-events capture served to the plugin (g12.024).
    param_out: Box<ParamOutCapture>,
}

// Safety: the session is handed to exactly one audio thread; CLAP's process
// and start/stop_processing are audio-thread functions, and the owner
// serializes lifecycle against the session per the type contract above.
unsafe impl Send for ClapProcessSession {}

impl ClapProcessSession {
    fn new(
        plugin: *const clap_plugin,
        sample_rate_hz: f64,
        max_frames: usize,
        audio_buses: &PluginAudioBusDescriptorList,
        param_changes: Arc<PluginParamChangeQueue>,
        param_out_queue: Arc<PluginParamChangeQueue>,
    ) -> Self {
        let input_buses = audio_buses
            .iter()
            .filter(|bus| bus.direction == PluginAudioBusDirection::Input)
            .collect::<Vec<_>>();
        let output_buses = audio_buses
            .iter()
            .filter(|bus| bus.direction == PluginAudioBusDirection::Output)
            .collect::<Vec<_>>();
        let main_input_bus = input_buses.iter().position(|bus| bus.is_main);
        let main_output_bus = output_buses
            .iter()
            .position(|bus| bus.is_main)
            .expect("supported CLAP layouts always have a main output bus");
        let input_channel_counts = input_buses
            .iter()
            .map(|bus| usize::from(bus.channels))
            .collect::<Vec<_>>();
        let output_channel_counts = output_buses
            .iter()
            .map(|bus| usize::from(bus.channels))
            .collect::<Vec<_>>();
        let mut param_events = Box::new(ParamEventList {
            params: Vec::with_capacity(PLUGIN_PARAM_CHANGE_CAPACITY),
            notes: Vec::with_capacity(IN_EVENT_CAPACITY),
            note_expressions: Vec::with_capacity(IN_EVENT_CAPACITY),
            midi: Vec::with_capacity(IN_EVENT_CAPACITY),
            order: Vec::with_capacity(PLUGIN_PARAM_CHANGE_CAPACITY + IN_EVENT_CAPACITY),
            list: clap_input_events {
                ctx: ptr::null_mut(),
                size: Some(param_in_events_size),
                get: Some(param_in_events_get),
            },
        });
        // Self-referential ctx: the list lives inside the box (stable
        // address) for the session's whole lifetime.
        param_events.list.ctx = (&mut *param_events as *mut ParamEventList).cast();
        let mut param_out = Box::new(ParamOutCapture {
            queue: param_out_queue,
            list: clap_output_events {
                ctx: ptr::null_mut(),
                try_push: Some(param_out_events_try_push),
            },
        });
        param_out.list.ctx = (&mut *param_out as *mut ParamOutCapture).cast();
        Self {
            plugin,
            sample_rate_hz,
            input_buses: ClapAudioBusBuffers::new(&input_channel_counts, max_frames),
            output_buses: ClapAudioBusBuffers::new(&output_channel_counts, max_frames),
            main_input_bus,
            main_output_bus,
            max_frames,
            steady_time: AtomicI64::new(0),
            processing: false,
            param_changes,
            param_scratch: Vec::with_capacity(PLUGIN_PARAM_CHANGE_CAPACITY),
            param_events,
            param_out,
        }
    }

    /// Build a valid stopped transport snapshot for the current block.
    ///
    /// CLAP permits a null `process.transport`, but a number of otherwise
    /// conforming plugins assume the pointer is always present. Supplying a
    /// conservative stopped timeline is harmless to plugins that honour the
    /// optional contract and avoids crashing those that do not.
    fn transport(&self, steady_time: i64) -> clap_event_transport {
        let seconds = steady_time as f64 / self.sample_rate_hz;
        let beats = seconds * (120.0 / 60.0);
        let beats_fixed = (beats * CLAP_BEATTIME_FACTOR as f64) as i64;
        let seconds_fixed = (seconds * CLAP_SECTIME_FACTOR as f64) as i64;
        let beats_per_bar = 4_i64 * CLAP_BEATTIME_FACTOR;
        let bar_number = beats_fixed.div_euclid(beats_per_bar) as i32;

        clap_event_transport {
            header: clap_event_header {
                size: std::mem::size_of::<clap_event_transport>() as u32,
                time: 0,
                space_id: CLAP_CORE_EVENT_SPACE_ID,
                type_: CLAP_EVENT_TRANSPORT,
                flags: 0,
            },
            flags: CLAP_TRANSPORT_HAS_TEMPO
                | CLAP_TRANSPORT_HAS_BEATS_TIMELINE
                | CLAP_TRANSPORT_HAS_SECONDS_TIMELINE
                | CLAP_TRANSPORT_HAS_TIME_SIGNATURE,
            song_pos_beats: beats_fixed,
            song_pos_seconds: seconds_fixed,
            tempo: 120.0,
            tempo_inc: 0.0,
            loop_start_beats: 0,
            loop_end_beats: 0,
            loop_start_seconds: 0,
            loop_end_seconds: 0,
            bar_start: i64::from(bar_number) * beats_per_bar,
            bar_number,
            tsig_num: 4,
            tsig_denom: 4,
        }
    }

    /// Rebuild the block's in-events: param writes from the shared change
    /// queue (block-boundary application, time offset 0) followed by the
    /// block's note/CC events at their intra-block sample offsets (`events`
    /// must be sorted by `offset_frames`; the render plane's delivery
    /// contract). Alloc-free; returns the `clap_input_events` to hand to
    /// the plugin (the empty static list when nothing is pending).
    fn prepare_in_events(&mut self, events: &[PluginEvent]) -> *const clap_input_events {
        let list = &mut *self.param_events;
        list.params.clear();
        list.notes.clear();
        list.note_expressions.clear();
        list.midi.clear();
        list.order.clear();
        if !self.param_changes.is_empty() {
            self.param_changes.drain_coalesced(&mut self.param_scratch);
            for change in &self.param_scratch {
                list.params.push(clap_event_param_value {
                    header: clap_event_header {
                        size: std::mem::size_of::<clap_event_param_value>() as u32,
                        time: 0,
                        space_id: CLAP_CORE_EVENT_SPACE_ID,
                        type_: CLAP_EVENT_PARAM_VALUE,
                        flags: 0,
                    },
                    param_id: change.parameter_id,
                    cookie: ptr::null_mut(),
                    note_id: -1,
                    port_index: -1,
                    channel: -1,
                    key: -1,
                    value: change.value,
                });
                list.order
                    .push(InEventSlot::Param(list.params.len() as u32 - 1));
            }
        }
        for event in events {
            match event {
                PluginEvent::Note(note) => {
                    if list.notes.len() == IN_EVENT_CAPACITY {
                        continue;
                    }
                    list.notes.push(clap_event_note {
                        header: clap_event_header {
                            size: std::mem::size_of::<clap_event_note>() as u32,
                            time: note.offset_frames,
                            space_id: CLAP_CORE_EVENT_SPACE_ID,
                            type_: match note.kind {
                                NoteEventKind::NoteOn => CLAP_EVENT_NOTE_ON,
                                NoteEventKind::NoteOff => CLAP_EVENT_NOTE_OFF,
                            },
                            flags: 0,
                        },
                        note_id: note.note_id,
                        port_index: note.port_index as i16,
                        channel: i16::from(note.channel),
                        key: i16::from(note.key),
                        velocity: f64::from(note.velocity.clamp(0.0, 1.0)),
                    });
                    list.order
                        .push(InEventSlot::Note(list.notes.len() as u32 - 1));
                }
                PluginEvent::NoteExpression(expression) => {
                    if list.note_expressions.len() == IN_EVENT_CAPACITY {
                        continue;
                    }
                    let (expression_id, value) = match expression.expression {
                        NoteExpressionKind::Pressure => (
                            CLAP_NOTE_EXPRESSION_PRESSURE,
                            f64::from(expression.value.clamp(0.0, 1.0)),
                        ),
                        NoteExpressionKind::Timbre => (
                            CLAP_NOTE_EXPRESSION_BRIGHTNESS,
                            f64::from(expression.value.clamp(0.0, 1.0)),
                        ),
                        NoteExpressionKind::Tuning => (
                            CLAP_NOTE_EXPRESSION_TUNING,
                            f64::from(expression.value) / 100.0,
                        ),
                    };
                    list.note_expressions.push(clap_event_note_expression {
                        header: clap_event_header {
                            size: std::mem::size_of::<clap_event_note_expression>() as u32,
                            time: expression.offset_frames,
                            space_id: CLAP_CORE_EVENT_SPACE_ID,
                            type_: CLAP_EVENT_NOTE_EXPRESSION,
                            flags: 0,
                        },
                        expression_id,
                        note_id: expression.note_id,
                        port_index: expression.port_index as i16,
                        channel: i16::from(expression.channel),
                        key: i16::from(expression.key),
                        value,
                    });
                    list.order.push(InEventSlot::NoteExpression(
                        list.note_expressions.len() as u32 - 1,
                    ));
                }
                PluginEvent::ControlChange(change) => {
                    // The CLAP CC boundary: normalized f32 → 3-byte MIDI 1.0
                    // (CLAP has no float CC event).
                    if list.midi.len() == IN_EVENT_CAPACITY {
                        continue;
                    }
                    list.midi.push(clap_event_midi {
                        header: clap_event_header {
                            size: std::mem::size_of::<clap_event_midi>() as u32,
                            time: change.offset_frames,
                            space_id: CLAP_CORE_EVENT_SPACE_ID,
                            type_: CLAP_EVENT_MIDI,
                            flags: 0,
                        },
                        port_index: change.port_index,
                        data: [
                            0xB0 | (change.channel & 0x0F),
                            change.controller & 0x7F,
                            (change.value.clamp(0.0, 1.0) * 127.0).round() as u8,
                        ],
                    });
                    list.order
                        .push(InEventSlot::Midi(list.midi.len() as u32 - 1));
                }
                PluginEvent::Midi(midi) => {
                    if list.midi.len() == IN_EVENT_CAPACITY {
                        continue;
                    }
                    list.midi.push(clap_event_midi {
                        header: clap_event_header {
                            size: std::mem::size_of::<clap_event_midi>() as u32,
                            time: midi.offset_frames,
                            space_id: CLAP_CORE_EVENT_SPACE_ID,
                            type_: CLAP_EVENT_MIDI,
                            flags: 0,
                        },
                        port_index: 0,
                        data: [midi.status, midi.data1, midi.data2],
                    });
                    list.order
                        .push(InEventSlot::Midi(list.midi.len() as u32 - 1));
                }
                // Parameter events ride the wire queue; gestures have no
                // process input representation here.
                _ => {}
            }
        }
        if list.order.is_empty() {
            return &EMPTY_IN_EVENTS;
        }
        &list.list
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

    /// Process one block: optional interleaved stereo in, stereo out.
    /// Alloc-free (buffers preallocated at activate). On plugin error the
    /// input passes through unchanged. Returns `false` on error.
    pub fn process_interleaved_stereo(
        &mut self,
        input: &[f32],
        output: &mut [f32],
        frame_count: usize,
    ) -> bool {
        let frames = frame_count
            .min(self.max_frames)
            .min(input.len() / 2)
            .min(output.len() / 2);
        let in_events = self.prepare_in_events(&[]);
        self.input_buses.clear(frames);
        self.output_buses.clear(frames);
        if let Some(main_input_bus) = self.main_input_bus {
            self.input_buses
                .copy_interleaved_stereo_into(main_input_bus, input, frames);
        }
        let steady_time = self.steady_time.load(Ordering::Relaxed);
        let audio_inputs = self.input_buses.as_ptr();
        let audio_inputs_count = self.input_buses.len();
        let audio_outputs = self.output_buses.as_mut_ptr();
        let audio_outputs_count = self.output_buses.len();
        let transport = self.transport(steady_time);
        let process = clap_process {
            steady_time,
            frames_count: frames as u32,
            transport: &transport,
            audio_inputs,
            audio_outputs,
            audio_inputs_count,
            audio_outputs_count,
            in_events,
            out_events: &self.param_out.list,
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
        self.output_buses
            .copy_interleaved_stereo_from(self.main_output_bus, output, frames);
        true
    }

    /// In-place variant for the in-process isolation tier: processes the
    /// interleaved stereo buffer and writes the result back over it ONLY on
    /// success; on plugin error the buffer is left untouched (bypass
    /// semantics). Alloc-free. `true` = buffer transformed.
    pub fn process_in_place(&mut self, io: &mut [f32], frame_count: usize) -> bool {
        self.process_in_place_with_events(io, frame_count, &[])
    }

    /// [`Self::process_in_place`] with a per-block plugin event slice
    /// (sorted by `offset_frames`): note events map to CLAP note in-events
    /// (float velocity preserved), CC events downconvert to 3-byte MIDI at
    /// this boundary. Alloc-free. `true` = buffer transformed.
    pub fn process_in_place_with_events(
        &mut self,
        io: &mut [f32],
        frame_count: usize,
        events: &[PluginEvent],
    ) -> bool {
        let frames = frame_count.min(self.max_frames).min(io.len() / 2);
        let in_events = self.prepare_in_events(events);
        self.input_buses.clear(frames);
        self.output_buses.clear(frames);
        if let Some(main_input_bus) = self.main_input_bus {
            self.input_buses
                .copy_interleaved_stereo_into(main_input_bus, io, frames);
        }
        let steady_time = self.steady_time.load(Ordering::Relaxed);
        let audio_inputs = self.input_buses.as_ptr();
        let audio_inputs_count = self.input_buses.len();
        let audio_outputs = self.output_buses.as_mut_ptr();
        let audio_outputs_count = self.output_buses.len();
        let transport = self.transport(steady_time);
        let process = clap_process {
            steady_time,
            frames_count: frames as u32,
            transport: &transport,
            audio_inputs,
            audio_outputs,
            audio_inputs_count,
            audio_outputs_count,
            in_events,
            out_events: &self.param_out.list,
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
        self.output_buses
            .copy_interleaved_stereo_from(self.main_output_bus, io, frames);
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
