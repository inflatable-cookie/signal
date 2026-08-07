//! In-process VST3 plugin processor.

use std::sync::atomic::{AtomicBool, AtomicU32, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use signal_plugin::{PluginEvent, PluginParameterDescriptor};
use signal_plugin_vst3::{
    Vst3HostedInstance, Vst3ProcessSession, VST3_RESTART_IO_CHANGED, VST3_RESTART_LATENCY_CHANGED,
};
use signal_render_plane::{
    PluginBlockProcessor, RenderBlockPluginEvent, RenderPluginEventKind, RenderPluginEventSupport,
};

use super::common::{convert_block_events, PluginGuiEvent, EVENT_SCRATCH_CAPACITY};

/// In-process VST3 processing backend (g11.031): the exact mirror of
/// [`InProcessClapProcessor`] over the VST3 COM hosting FFI.
///
/// Owns the hosted instance (module, component/processor/controller,
/// activation) for its whole lifetime. The process session sits behind a
/// `Mutex` taken with `try_lock` only — the audio thread never blocks; a
/// contended lock (teardown racing a callback) bypasses that block.
///
/// `setProcessing(true)` runs lazily on the first processed block, which is
/// the audio thread — matching VST3's processing-thread contract.
pub struct InProcessVst3Processor {
    /// Field order matters: the session must drop before the instance.
    session: Mutex<Vst3ProcessSession>,
    instance: Mutex<Vst3HostedInstance>,
    /// Preallocated conversion scratch for per-block note/CC delivery
    /// (taken with `try_lock` on the audio thread, like the session).
    events_scratch: Mutex<Vec<PluginEvent>>,
    /// Whether the controller exposed an `IMidiMapping` at load (CC events
    /// deliver as mapped parameter changes; without one they drop).
    midi_cc_mapping: bool,
    midi_cc_mappings: [bool; 128],
    pitch_bend_mapping: bool,
    channel_pressure_mapping: bool,
    parameters: Vec<PluginParameterDescriptor>,
    latency_frames: AtomicU32,
    latency_revision: AtomicU64,
    observed_latency_changes: AtomicU64,
    pub(crate) pending_restart_flags: Arc<AtomicU32>,
    max_frames: u32,
    /// Cleared at teardown so late callbacks bypass instead of racing the
    /// lifecycle.
    alive: AtomicBool,
    /// Blocks bypassed (unsupported layout, plugin error, teardown race).
    misses: AtomicU64,
    unsupported_events: AtomicU64,
}

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

// Safety: the raw COM pointers inside the instance and session are only
// dereferenced behind the two mutexes; the type's public surface serializes
// all lifecycle and processing access.
unsafe impl Send for InProcessVst3Processor {}
unsafe impl Sync for InProcessVst3Processor {}

impl InProcessVst3Processor {
    /// Load the component class `class_id_hex` from the bundle at
    /// `bundle_root` in the host process, activate it at `sample_rate_hz` /
    /// `max_frames`, negotiate a stereo effect (2-in/2-out) or instrument
    /// (0-in/2-out) layout, and build the processing session. Other layouts
    /// fail with `layout_unsupported`; components with no audio buses fail
    /// with `no_audio_buses` and may instead use [`InProcessVst3Editor`].
    pub fn load_and_activate(
        bundle_root: &std::path::Path,
        class_id_hex: &str,
        sample_rate_hz: u32,
        max_frames: u32,
    ) -> Result<Self, String> {
        Self::load_and_activate_internal(
            bundle_root,
            class_id_hex,
            sample_rate_hz,
            max_frames,
            false,
        )
    }

    /// Inspection-specific load path. ARA-capable components receive an
    /// empty document with the editor-view role before activation so their
    /// native editor can be inspected. This does not provide ARA playback or
    /// model editing.
    pub fn load_and_activate_for_inspection(
        bundle_root: &std::path::Path,
        class_id_hex: &str,
        sample_rate_hz: u32,
        max_frames: u32,
    ) -> Result<Self, String> {
        Self::load_and_activate_internal(
            bundle_root,
            class_id_hex,
            sample_rate_hz,
            max_frames,
            true,
        )
    }

    fn load_and_activate_internal(
        bundle_root: &std::path::Path,
        class_id_hex: &str,
        sample_rate_hz: u32,
        max_frames: u32,
        enable_ara_inspection: bool,
    ) -> Result<Self, String> {
        let mut instance = if enable_ara_inspection {
            Vst3HostedInstance::load_for_inspection(bundle_root, class_id_hex)
        } else {
            Vst3HostedInstance::load(bundle_root, class_id_hex)
        }
        .map_err(|error| error.token)?;
        instance
            .activate(f64::from(sample_rate_hz), 1, max_frames)
            .map_err(|error| error.token)?;
        let session = instance.process_session().map_err(|error| error.token)?;
        let parameters = instance.parameters().to_vec();
        let latency_frames = instance.latency_frames();
        let midi_cc_mapping = instance.midi_cc_mapping_available();
        let pending_restart_flags = instance.pending_restart_flags();
        let midi_cc_mappings = std::array::from_fn(|controller| {
            instance.midi_controller_mapping_available(controller as u16)
        });
        let pitch_bend_mapping = instance.midi_controller_mapping_available(128);
        let channel_pressure_mapping = instance.midi_controller_mapping_available(129);
        Ok(Self {
            session: Mutex::new(session),
            instance: Mutex::new(instance),
            events_scratch: Mutex::new(Vec::with_capacity(EVENT_SCRATCH_CAPACITY)),
            midi_cc_mapping,
            midi_cc_mappings,
            pitch_bend_mapping,
            channel_pressure_mapping,
            parameters,
            latency_frames: AtomicU32::new(latency_frames),
            latency_revision: AtomicU64::new(0),
            observed_latency_changes: AtomicU64::new(0),
            pending_restart_flags,
            max_frames,
            alive: AtomicBool::new(true),
            misses: AtomicU64::new(0),
            unsupported_events: AtomicU64::new(0),
        })
    }

    /// Refresh cached latency after `kLatencyChanged`. Observation runs on
    /// the host control thread; the audio callback never takes this lock.
    fn refresh_latency(&self) {
        let Ok(instance) = self.instance.lock() else {
            return;
        };
        let changes = instance.latency_change_count();
        let observed = self.observed_latency_changes.load(Ordering::Relaxed);
        if changes == observed {
            return;
        }
        self.observed_latency_changes
            .store(changes, Ordering::Relaxed);
        let latency_frames = instance.latency_frames();
        let previous = self.latency_frames.swap(latency_frames, Ordering::Relaxed);
        if previous != latency_frames {
            self.latency_revision.fetch_add(1, Ordering::Relaxed);
        }
    }

    /// Whether the edit controller has requested an accepted processing
    /// lifecycle restart that the host control thread has not serviced yet.
    pub fn processing_restart_pending(&self) -> bool {
        self.pending_restart_flags.load(Ordering::Acquire)
            & (VST3_RESTART_IO_CHANGED | VST3_RESTART_LATENCY_CHANGED)
            != 0
    }

    /// Service accepted VST3 processing restart flags on the owning control
    /// thread. Audio callbacks bypass while the session is replaced.
    pub fn service_processing_restart(&self) -> Result<bool, String> {
        let flags = self.pending_restart_flags.swap(0, Ordering::AcqRel);
        if flags == 0 {
            return Ok(false);
        }
        let mut session = self
            .session
            .lock()
            .map_err(|_| "session_lock_poisoned".to_string())?;
        session.stop();
        let replacement = self
            .instance
            .lock()
            .map_err(|_| "instance_lock_poisoned".to_string())?
            .restart_processing(flags)
            .map_err(|error| error.token)?;
        *session = replacement;
        self.refresh_latency();
        Ok(true)
    }

    /// Parameter inventory enumerated at load.
    pub fn parameters(&self) -> &[PluginParameterDescriptor] {
        &self.parameters
    }

    /// Capture opaque VST3 component/controller state on the control
    /// thread. Audio bypasses while the session lock is held.
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

    /// Restore opaque VST3 component/controller state on the control
    /// thread.
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

    /// Whether the plugin exposed a VST3 `IMidiMapping` at load: with one,
    /// delivered CC events reach the DSP as mapped parameter changes;
    /// without one CC events drop (VST3 has no input CC event type) — the
    /// honest fallback, surfaced so hosts can tell users why a CC lane is
    /// inert on this plugin.
    pub fn midi_cc_mapping_available(&self) -> bool {
        self.midi_cc_mapping
    }

    /// Queue one normalized 0..1 parameter write (g12.023): syncs the edit
    /// controller and rides the next block's input `IParameterChanges`.
    /// Not part of the audio path — takes the instance lock.
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

    /// Blocks bypassed so far, cumulative.
    pub fn miss_count(&self) -> u64 {
        self.misses.load(Ordering::Relaxed)
    }

    // ── IPlugView delegation (g12.024, GUI phase 2) ─────────────────────
    //
    // All gui methods take the INSTANCE lock, never the audio-path session
    // lock, so an open editor cannot contend with `process()`. MAIN-THREAD
    // CONTRACT: VST3 view functions are UI-thread; the embedding host must
    // dispatch every call below onto the application main thread (Tauri
    // `run_on_main_thread`) — the classic plugin-crash source when missed.

    /// Whether the controller produced an editor view at the load-time
    /// probe.
    pub fn gui_supported(&self) -> bool {
        self.alive.load(Ordering::Relaxed)
            && self
                .instance
                .lock()
                .map(|instance| instance.gui_supported())
                .unwrap_or(false)
    }

    /// Whether an editor view is currently attached on this instance.
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

    /// Whether the open editor is user-resizable (`canResize`). MAIN
    /// THREAD ONLY.
    pub fn gui_can_resize(&self) -> bool {
        self.instance
            .lock()
            .ok()
            .and_then(|instance| instance.gui_session().map(|session| session.can_resize()))
            .unwrap_or(false)
    }

    /// Propose a new editor content size (user drag or a granted
    /// `RequestResize`); returns the accepted size. MAIN THREAD ONLY.
    pub fn gui_set_size(&self, width: u32, height: u32) -> Option<(u32, u32)> {
        self.instance.lock().ok().and_then(|mut instance| {
            instance
                .gui_session_mut()
                .and_then(|session| session.set_size(width, height))
        })
    }

    /// Grant a plugin-initiated `IPlugFrame::resizeView` request without
    /// applying the constraint negotiation reserved for host/user resizes.
    /// MAIN THREAD ONLY.
    pub fn gui_accept_plugin_resize(&self, width: u32, height: u32) -> Option<(u32, u32)> {
        self.instance.lock().ok().and_then(|mut instance| {
            instance
                .gui_session_mut()
                .and_then(|session| session.accept_plugin_resize(width, height))
        })
    }

    /// Destroy the open editor (idempotent; processing continues). MAIN
    /// THREAD ONLY.
    pub fn gui_close(&self) {
        if let Ok(mut instance) = self.instance.lock() {
            instance.gui_destroy();
        }
    }

    /// Drain queued host-side view callbacks (`resizeView` requests) for
    /// the embedding host to apply to its window.
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

    /// Stop processing and mark the backend dead: subsequent blocks bypass.
    /// Call before dropping the last handle while a plan may still run.
    pub fn shutdown(&self) {
        self.alive.store(false, Ordering::Relaxed);
        if let Ok(mut session) = self.session.lock() {
            session.stop();
        }
    }
}

impl Drop for InProcessVst3Processor {
    fn drop(&mut self) {
        self.alive.store(false, Ordering::Relaxed);
        if let Ok(mut session) = self.session.lock() {
            session.stop();
        }
        if let Ok(mut instance) = self.instance.lock() {
            let _ = instance.deactivate();
        }
        // The instance's own Drop releases the COM objects and closes the
        // module after the session (holding the raw processor pointer) is
        // already inert.
    }
}

impl PluginBlockProcessor for InProcessVst3Processor {
    fn process(&self, scratch: &mut [f32], frame_count: usize, channels: usize) -> bool {
        self.process_with_events(scratch, frame_count, channels, &[])
    }

    fn event_support(&self) -> RenderPluginEventSupport {
        RenderPluginEventSupport {
            notes: true,
            control_change: self.midi_cc_mappings.iter().any(|mapped| *mapped),
            pitch_bend: self.pitch_bend_mapping,
            channel_pressure: self.channel_pressure_mapping,
            note_expression: false,
        }
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
        let unsupported = events
            .iter()
            .filter(|event| match event.kind {
                RenderPluginEventKind::NoteOn { .. } | RenderPluginEventKind::NoteOff { .. } => {
                    false
                }
                RenderPluginEventKind::ControlChange { controller, .. } => {
                    !self.midi_cc_mappings[usize::from(controller)]
                }
                RenderPluginEventKind::PitchBend { .. } => !self.pitch_bend_mapping,
                RenderPluginEventKind::ChannelPressure { .. } => !self.channel_pressure_mapping,
                RenderPluginEventKind::NoteExpression { .. } => true,
                RenderPluginEventKind::VoiceStart { .. }
                | RenderPluginEventKind::VoiceStop { .. }
                | RenderPluginEventKind::VoiceParam { .. } => true,
            })
            .count() as u64;
        self.unsupported_events
            .fetch_add(unsupported, Ordering::Relaxed);
        if !self.alive.load(Ordering::Relaxed)
            || channels != 2
            || frame_count > self.max_frames as usize
        {
            self.misses.fetch_add(1, Ordering::Relaxed);
            return false;
        }
        if self.processing_restart_pending() {
            if let Ok(mut session) = self.session.try_lock() {
                session.stop();
            }
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
