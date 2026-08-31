//! In-process CLAP plugin processor.

use std::sync::atomic::{AtomicBool, AtomicU32, AtomicU64, Ordering};
use std::sync::Mutex;

use signal_plugin::{PluginEvent, PluginParameterDescriptor};
use signal_plugin_clap::{ClapHostedInstance, ClapProcessSession};
use signal_render_plane::{PluginBlockProcessor, RenderBlockPluginEvent, RenderPluginEventSupport};

use super::common::{
    convert_block_events, PluginGuiEvent, CLAP_EVENT_SUPPORT, EVENT_SCRATCH_CAPACITY,
};

/// In-process CLAP processing backend.
///
/// Owns the hosted instance (library, plugin, activation) for its whole
/// lifetime, so the render plane's handle can never outlive the plugin
/// code it calls. The process session sits behind a `Mutex` taken with
/// `try_lock` only — the audio thread never blocks; a contended lock
/// (teardown racing a callback) bypasses that block.
///
/// `start_processing` runs lazily on the first processed block, which is
/// the audio thread — matching CLAP's threading contract.
#[derive(Debug)]
pub struct InProcessClapProcessor {
    /// Keeps the plugin instance (and its library) alive; lifecycle runs on
    /// drop. Field order matters: the session must drop before the
    /// instance.
    session: Mutex<ClapProcessSession>,
    instance: Mutex<ClapHostedInstance>,
    /// Preallocated conversion scratch for per-block note/CC delivery
    /// (taken with `try_lock` on the audio thread, like the session).
    events_scratch: Mutex<Vec<PluginEvent>>,
    parameters: Vec<PluginParameterDescriptor>,
    latency_frames: AtomicU32,
    latency_revision: AtomicU64,
    observed_restart_requests: AtomicU64,
    max_frames: u32,
    /// Cleared at teardown so late callbacks bypass instead of racing the
    /// lifecycle.
    alive: AtomicBool,
    /// Blocks bypassed (unsupported layout, plugin error, teardown race).
    misses: AtomicU64,
    unsupported_events: AtomicU64,
}

// Safety: the raw plugin pointers inside the instance and session are only
// dereferenced behind the two mutexes; the type's public surface serializes
// all lifecycle and processing access.
unsafe impl Send for InProcessClapProcessor {}
unsafe impl Sync for InProcessClapProcessor {}

impl InProcessClapProcessor {
    /// Load `plugin_id` from `library_path` in the host process, activate
    /// it at `sample_rate_hz` / `max_frames`, and build the processing
    /// session. Accepts a stereo effect (2-in/2-out) or instrument
    /// (0-in/2-out); every other layout rejects with `layout_unsupported`.
    pub fn load_and_activate(
        library_path: &std::path::Path,
        plugin_id: &str,
        sample_rate_hz: u32,
        max_frames: u32,
    ) -> Result<Self, String> {
        Self::load_and_activate_internal(library_path, plugin_id, sample_rate_hz, max_frames, false)
    }

    /// Inspection may open a plugin whose main buses have extra channels (for
    /// example a bridged VST2 sidechain pair or Reaktor's 16-channel buses).
    /// The first input/output pair carries stereo; remaining inputs stay
    /// silent and remaining outputs are ignored.
    pub fn load_and_activate_for_inspection(
        library_path: &std::path::Path,
        plugin_id: &str,
        sample_rate_hz: u32,
        max_frames: u32,
    ) -> Result<Self, String> {
        Self::load_and_activate_internal(library_path, plugin_id, sample_rate_hz, max_frames, true)
    }

    fn load_and_activate_internal(
        library_path: &std::path::Path,
        plugin_id: &str,
        sample_rate_hz: u32,
        max_frames: u32,
        inspection: bool,
    ) -> Result<Self, String> {
        let mut instance =
            ClapHostedInstance::load(library_path, plugin_id).map_err(|error| error.token)?;
        let layout = instance.port_layout();
        let layout_supported = if inspection {
            layout.is_supported_stereo_inspection_processor()
        } else {
            layout.is_supported_stereo_processor()
        };
        if !layout_supported {
            return Err("layout_unsupported".to_string());
        }
        instance
            .activate(f64::from(sample_rate_hz), 1, max_frames)
            .map_err(|error| error.token)?;
        let session = instance.process_session().map_err(|error| error.token)?;
        let parameters = instance.parameters().to_vec();
        let latency_frames = instance.latency_frames();
        Ok(Self {
            session: Mutex::new(session),
            instance: Mutex::new(instance),
            events_scratch: Mutex::new(Vec::with_capacity(EVENT_SCRATCH_CAPACITY)),
            parameters,
            latency_frames: AtomicU32::new(latency_frames),
            latency_revision: AtomicU64::new(0),
            observed_restart_requests: AtomicU64::new(0),
            max_frames,
            alive: AtomicBool::new(true),
            misses: AtomicU64::new(0),
            unsupported_events: AtomicU64::new(0),
        })
    }

    /// Refresh cached latency after a CLAP restart request. Called only by
    /// control-side observation methods; processing never takes this lock.
    fn refresh_latency(&self) {
        let Ok(instance) = self.instance.lock() else {
            return;
        };
        let restart_requests = instance.restart_request_count();
        let observed = self.observed_restart_requests.load(Ordering::Relaxed);
        if restart_requests == observed {
            return;
        }
        self.observed_restart_requests
            .store(restart_requests, Ordering::Relaxed);
        let latency_frames = instance.latency_frames();
        let previous = self.latency_frames.swap(latency_frames, Ordering::Relaxed);
        if previous != latency_frames {
            self.latency_revision.fetch_add(1, Ordering::Relaxed);
        }
    }

    /// Parameter inventory enumerated at load.
    pub fn parameters(&self) -> &[PluginParameterDescriptor] {
        &self.parameters
    }

    /// Queue one normalized 0..1 parameter write (g12.023): delivered to
    /// the plugin as a `CLAP_EVENT_PARAM_VALUE` in-event at the top of the
    /// next processed block. Not part of the audio path — takes the
    /// instance lock.
    pub fn set_parameter_normalized(
        &self,
        parameter_id: u32,
        normalized: f32,
    ) -> Result<(), String> {
        if !self.alive.load(Ordering::Relaxed) {
            return Err("backend_dead".to_string());
        }
        let instance = self
            .instance
            .lock()
            .map_err(|_| "instance_lock_poisoned".to_string())?;
        instance
            .set_parameter_normalized(parameter_id, normalized)
            .map_err(|error| error.token)
    }

    /// Capture opaque plugin project state on the control thread. Taking the
    /// session lock first makes an audio callback bypass rather than racing
    /// the format state serializer.
    pub fn save_state(&self) -> Result<Vec<u8>, String> {
        if !self.alive.load(Ordering::Relaxed) {
            return Err("backend_dead".to_string());
        }
        let _session = self
            .session
            .lock()
            .map_err(|_| "session_lock_poisoned".to_string())?;
        self.instance
            .lock()
            .map_err(|_| "instance_lock_poisoned".to_string())?
            .save_state()
            .map_err(|error| error.token)
    }

    /// Restore opaque plugin project state on the control thread.
    pub fn load_state(&self, bytes: &[u8]) -> Result<(), String> {
        if !self.alive.load(Ordering::Relaxed) {
            return Err("backend_dead".to_string());
        }
        let _session = self
            .session
            .lock()
            .map_err(|_| "session_lock_poisoned".to_string())?;
        self.instance
            .lock()
            .map_err(|_| "instance_lock_poisoned".to_string())?
            .load_state(bytes)
            .map_err(|error| error.token)
    }

    /// Monotonic count of plugin `clap.state` dirty notifications observed
    /// by the host. Used by control-side autosave scheduling.
    pub fn state_dirty_request_count(&self) -> u64 {
        self.instance
            .lock()
            .map(|instance| instance.state_dirty_request_count())
            .unwrap_or(0)
    }

    /// Blocks bypassed so far, cumulative.
    pub fn miss_count(&self) -> u64 {
        self.misses.load(Ordering::Relaxed)
    }

    // ── clap.gui delegation (g12.022 phase 1) ───────────────────────────
    //
    // All gui methods take the INSTANCE lock, never the audio-path session
    // lock, so an open editor cannot contend with `process()`. MAIN-THREAD
    // CONTRACT: CLAP gui functions are main-thread; the embedding host must
    // dispatch every call below onto the application main thread (Tauri
    // `run_on_main_thread`) — the classic plugin-crash source when missed.

    /// Whether the plugin exposes an embeddable `clap.gui` for this
    /// platform's window API.
    pub fn gui_supported(&self) -> bool {
        self.alive.load(Ordering::Relaxed)
            && self
                .instance
                .lock()
                .map(|instance| instance.gui_supported())
                .unwrap_or(false)
    }

    /// Whether an editor is currently created on this instance.
    pub fn gui_is_open(&self) -> bool {
        self.instance
            .lock()
            .map(|instance| instance.gui_is_open())
            .unwrap_or(false)
    }

    /// Open the embedded editor parented into `parent_view` (an `NSView*`
    /// on macOS, passed as `usize` so callers stay `Send`). Returns the
    /// plugin's initial content size in logical units. MAIN THREAD ONLY.
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

    /// Whether the open editor is user-resizable. MAIN THREAD ONLY.
    pub fn gui_can_resize(&self) -> bool {
        self.instance
            .lock()
            .ok()
            .and_then(|instance| instance.gui_session().map(|session| session.can_resize()))
            .unwrap_or(false)
    }

    /// Propose a new editor content size from a host/user resize; returns the
    /// accepted size. MAIN THREAD ONLY.
    pub fn gui_set_size(&self, width: u32, height: u32) -> Option<(u32, u32)> {
        self.instance.lock().ok().and_then(|mut instance| {
            instance
                .gui_session_mut()
                .and_then(|session| session.set_size(width, height))
        })
    }

    /// Destroy the open editor (idempotent; processing continues). MAIN
    /// THREAD ONLY.
    pub fn gui_close(&self) {
        if let Ok(mut instance) = self.instance.lock() {
            instance.gui_destroy();
        }
    }

    /// Drain queued host-side gui callbacks (`request_resize`, `closed`, …)
    /// for the embedding host to apply to its window.
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

    /// Drain plugin-originated param values captured from the process
    /// out-events (g12.024, plugin GUI → host sync): `(parameter_id,
    /// normalized 0..1)`, coalesced last-write-wins per parameter.
    pub fn take_param_out_events(&self) -> Vec<(u32, f32)> {
        if !self.alive.load(Ordering::Relaxed) {
            return Vec::new();
        }
        self.instance
            .lock()
            .map(|instance| instance.take_param_out_events())
            .unwrap_or_default()
    }

    /// Drain host-side `clap.params` callbacks (rescan / clear /
    /// request_flush) observed from the plugin (g12.024).
    pub fn take_params_events(&self) -> Vec<signal_plugin_clap::ClapHostParamsEvent> {
        self.instance
            .lock()
            .map(|instance| instance.take_params_events())
            .unwrap_or_default()
    }

    /// Stop processing and mark the backend dead: subsequent blocks bypass.
    /// Call before dropping the last handle while a plan may still run.
    pub fn shutdown(&self) {
        self.alive.store(false, Ordering::Relaxed);
        if let Ok(mut session) = self.session.lock() {
            session.stop();
        }
    }
}

impl Drop for InProcessClapProcessor {
    fn drop(&mut self) {
        self.alive.store(false, Ordering::Relaxed);
        if let Ok(mut session) = self.session.lock() {
            session.stop();
        }
        if let Ok(mut instance) = self.instance.lock() {
            let _ = instance.deactivate();
        }
        // The instance's own Drop destroys the plugin and closes the
        // library after the session (holding the raw plugin pointer) is
        // already inert.
    }
}

impl PluginBlockProcessor for InProcessClapProcessor {
    fn process(&self, scratch: &mut [f32], frame_count: usize, channels: usize) -> bool {
        self.process_with_events(scratch, frame_count, channels, &[])
    }

    fn event_support(&self) -> RenderPluginEventSupport {
        CLAP_EVENT_SUPPORT
    }

    fn unsupported_event_count(&self) -> u64 {
        self.unsupported_events.load(Ordering::Relaxed)
    }

    fn latency_frames(&self) -> u32 {
        self.refresh_latency();
        self.latency_frames.load(Ordering::Relaxed)
    }

    fn latency_revision(&self) -> u64 {
        self.refresh_latency();
        self.latency_revision.load(Ordering::Relaxed)
    }

    fn process_with_events(
        &self,
        scratch: &mut [f32],
        frame_count: usize,
        channels: usize,
        events: &[RenderBlockPluginEvent],
    ) -> bool {
        if !self.alive.load(Ordering::Relaxed)
            || channels != 2
            || frame_count > self.max_frames as usize
        {
            self.misses.fetch_add(1, Ordering::Relaxed);
            return false;
        }
        // try_lock: never block the audio thread. Contention only happens
        // against teardown, which is about to mark the backend dead anyway.
        let Ok(mut session) = self.session.try_lock() else {
            self.misses.fetch_add(1, Ordering::Relaxed);
            return false;
        };
        let Ok(mut events_scratch) = self.events_scratch.try_lock() else {
            self.misses.fetch_add(1, Ordering::Relaxed);
            return false;
        };
        if !session.is_processing() && session.start().is_err() {
            self.misses.fetch_add(1, Ordering::Relaxed);
            return false;
        }
        convert_block_events(events, &mut events_scratch);
        let samples = frame_count * channels;
        if session.process_in_place_with_events(
            &mut scratch[..samples],
            frame_count,
            &events_scratch,
        ) {
            true
        } else {
            self.misses.fetch_add(1, Ordering::Relaxed);
            false
        }
    }
}
