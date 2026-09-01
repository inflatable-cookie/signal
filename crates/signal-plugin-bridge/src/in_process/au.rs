//! In-process AU plugin processor.

use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Mutex;

use signal_plugin::{PluginEvent, PluginParameterDescriptor};
use signal_plugin_au::{AuHostedInstance, AuProcessSession};
use signal_render_plane::{
    PluginBlockProcessor, RenderBlockPluginEvent, RenderPluginEventKind, RenderPluginEventSupport,
};

use super::common::{
    convert_block_events, PluginGuiEvent, AU_EVENT_SUPPORT, EVENT_SCRATCH_CAPACITY,
};

/// In-process Audio Unit processing backend (g11.032): the exact mirror of
/// `InProcessVst3Processor` over the AU pull-model hosting FFI.
///
/// Owns the hosted instance (component instance, activation) for its whole
/// lifetime. The process session sits behind a `Mutex` taken with
/// `try_lock` only — the audio thread never blocks; a contended lock
/// (teardown racing a callback) bypasses that block.
///
/// The load path takes the same (path, key) pair as the other backends;
/// for AU the path identifies the discovered bundle when available and the
/// key is the fourcc triple resolved through the system registry.
#[derive(Debug)]
pub struct InProcessAuProcessor {
    /// Field order matters: the session must drop before the instance (the
    /// session's drop uninstalls the render callback from the live unit).
    session: Mutex<AuProcessSession>,
    instance: Mutex<AuHostedInstance>,
    /// Preallocated conversion scratch for per-block note/CC delivery
    /// (taken with `try_lock` on the audio thread, like the session).
    events_scratch: Mutex<Vec<PluginEvent>>,
    parameters: Vec<PluginParameterDescriptor>,
    max_frames: u32,
    /// Cleared at teardown so late callbacks bypass instead of racing the
    /// lifecycle.
    alive: AtomicBool,
    /// Blocks bypassed (unsupported layout, unit error, teardown race).
    misses: AtomicU64,
    unsupported_events: AtomicU64,
}

// Safety: the raw AudioUnit handle inside the instance and session is only
// dereferenced behind the two mutexes; the type's public surface serializes
// all lifecycle and processing access.
unsafe impl Send for InProcessAuProcessor {}
unsafe impl Sync for InProcessAuProcessor {}

impl InProcessAuProcessor {
    /// Resolve the fourcc `load_key` through the system AudioComponent
    /// registry, negotiate the stereo processing format while activating the
    /// unit at `sample_rate_hz` / `max_frames`, and build the processing
    /// session. Units that reject the negotiated format fail with the stable
    /// `layout_unsupported` token; off macOS the load fails with
    /// `unsupported_platform`.
    pub fn load_and_activate(
        library_path: &std::path::Path,
        load_key: &str,
        sample_rate_hz: u32,
        max_frames: u32,
    ) -> Result<Self, String> {
        let mut instance =
            AuHostedInstance::load(library_path, load_key).map_err(|error| error.token)?;
        instance
            .activate(f64::from(sample_rate_hz), 1, max_frames)
            .map_err(|error| error.token)?;
        let session = instance.process_session().map_err(|error| error.token)?;
        let parameters = instance.parameters().to_vec();
        Ok(Self {
            session: Mutex::new(session),
            instance: Mutex::new(instance),
            events_scratch: Mutex::new(Vec::with_capacity(EVENT_SCRATCH_CAPACITY)),
            parameters,
            max_frames,
            alive: AtomicBool::new(true),
            misses: AtomicU64::new(0),
            unsupported_events: AtomicU64::new(0),
        })
    }

    /// Parameter inventory enumerated at load.
    pub fn parameters(&self) -> &[PluginParameterDescriptor] {
        &self.parameters
    }

    /// Capture opaque AU class-info state on the control thread. Audio
    /// bypasses while the session lock is held.
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

    /// Restore opaque AU class-info state on the control thread.
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

    /// Set one parameter's plain value on the hosted unit
    /// (`AudioUnitSetParameter`). Not part of the audio path — takes the
    /// instance lock.
    pub fn set_parameter(&self, parameter_id: u32, value: f32) -> Result<(), String> {
        let mut instance = self
            .instance
            .lock()
            .map_err(|_| "instance_lock_poisoned".to_string())?;
        instance
            .set_parameter(parameter_id, value)
            .map_err(|error| error.token)
    }

    /// Normalized 0..1 parameter write (g12.023): maps onto the unit's
    /// plain range and applies via `AudioUnitSetParameter` — the unit picks
    /// it up on its next render pull. Not part of the audio path — takes
    /// the instance lock.
    pub fn set_parameter_normalized(
        &self,
        parameter_id: u32,
        normalized: f32,
    ) -> Result<(), String> {
        if !self.alive.load(Ordering::Relaxed) {
            return Err("backend_dead".to_string());
        }
        let mut instance = self
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

    // ── Cocoa view delegation (g12.024, GUI phase 2) ────────────────────
    //
    // All gui methods take the INSTANCE lock, never the audio-path session
    // lock. MAIN-THREAD CONTRACT: open/close touch AppKit; the embedding
    // host must dispatch them onto the application main thread.

    /// Whether the unit provides a custom Cocoa editor
    /// (`kAudioUnitProperty_CocoaUI`, cached at load). Units without one
    /// report unsupported — the generic view is deliberately not built.
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

    /// Open the unit's Cocoa editor child-attached into `parent_view` (an
    /// `NSView*`, passed as `usize` so callers stay `Send`). Returns the
    /// view's reported frame size in logical units. MAIN THREAD ONLY.
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

    /// AU Cocoa views size themselves; the host window stays fixed at the
    /// view's reported frame (no resize negotiation surface in AUv2).
    pub fn gui_can_resize(&self) -> bool {
        false
    }

    /// No resize negotiation surface in AUv2 — proposals are refused.
    pub fn gui_set_size(&self, _width: u32, _height: u32) -> Option<(u32, u32)> {
        None
    }

    /// Destroy the open editor (idempotent; processing continues). MAIN
    /// THREAD ONLY.
    pub fn gui_close(&self) {
        if let Ok(mut instance) = self.instance.lock() {
            instance.gui_destroy();
        }
    }

    /// AU has no host-callback channel here; always empty.
    pub fn gui_take_events(&self) -> Vec<PluginGuiEvent> {
        Vec::new()
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

impl Drop for InProcessAuProcessor {
    fn drop(&mut self) {
        self.alive.store(false, Ordering::Relaxed);
        if let Ok(mut session) = self.session.lock() {
            session.stop();
        }
        if let Ok(mut instance) = self.instance.lock() {
            let _ = instance.deactivate();
        }
        // The session field drops first (uninstalling the render callback),
        // then the instance's own Drop uninitializes and disposes the unit.
    }
}

impl PluginBlockProcessor for InProcessAuProcessor {
    fn process(&self, scratch: &mut [f32], frame_count: usize, channels: usize) -> bool {
        self.process_with_events(scratch, frame_count, channels, &[])
    }

    fn event_support(&self) -> RenderPluginEventSupport {
        AU_EVENT_SUPPORT
    }

    fn unsupported_event_count(&self) -> u64 {
        self.unsupported_events.load(Ordering::Relaxed)
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
            .filter(|event| matches!(event.kind, RenderPluginEventKind::NoteExpression { .. }))
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
