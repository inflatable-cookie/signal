use std::{
    ffi::{c_void, CString},
    path::Path,
    ptr,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc, Mutex,
    },
};

use clap_sys::{
    events::{
        clap_event_header, clap_event_param_value, clap_input_events, clap_output_events,
        CLAP_CORE_EVENT_SPACE_ID, CLAP_EVENT_PARAM_VALUE,
    },
    ext::gui::clap_plugin_gui,
    ext::latency::{clap_plugin_latency, CLAP_EXT_LATENCY},
    ext::params::{clap_plugin_params, CLAP_EXT_PARAMS},
    ext::state::{clap_plugin_state, CLAP_EXT_STATE},
    plugin::clap_plugin,
    stream::{clap_istream, clap_ostream},
};
use signal_plugin::{
    PluginParamChange, PluginParamChangeQueue, PluginParameterDescriptor,
    PLUGIN_PARAM_CHANGE_CAPACITY,
};

use crate::discovery::PluginAudioBusDescriptorList;
use crate::gui::{ClapGuiEvent, ClapGuiSession};

use super::super::entry::{ClapHostingError, LoadedClapEntry};
use super::super::host::{sandbox_host, ClapHostParamsEvent, ClapHostShim};
use super::super::process::{
    param_in_events_get, param_in_events_size, param_out_events_try_push, ClapProcessSession,
    InEventSlot, ParamEventList, ParamOutCapture,
};
use super::layout::{ClapHostedPortLayout, HostedInstanceState};
use super::shape::{gui_shape, instance_shape};
use super::state_io::{clap_state_read, clap_state_write, ClapStateReadCursor};

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

impl std::fmt::Debug for ClapHostedInstance {
    /// Reports plugin identity and lifecycle state. The host shim, entry, and
    /// gui extension are raw CLAP ABI objects and are not formatted.
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("ClapHostedInstance")
            .field("plugin", &self.plugin)
            .field("state", &self.state)
            .field("parameters", &self.parameters.len())
            .field("activated_sample_rate_hz", &self.activated_sample_rate_hz)
            .field("activated_max_frames", &self.activated_max_frames)
            .field("gui_api_supported", &self.gui_api_supported)
            .field("gui_open", &self.gui_session.is_some())
            .finish_non_exhaustive()
    }
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
