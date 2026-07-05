//! In-process (InProcess tier) backend: direct FFI processing in the host.
//!
//! The plugin's library is dlopen'd IN THE HOST PROCESS and `process()` is
//! called directly on the audio thread — no shared-memory round trip, no
//! wait budget, and honestly NO crash isolation: a crashing plugin takes
//! the host down. That is the documented tradeoff of choosing this tier.

use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Mutex;

use signal_plugin::PluginParameterDescriptor;
use signal_plugin_au::{AuHostedInstance, AuProcessSession};
use signal_plugin_clap::{ClapHostedInstance, ClapProcessSession};
use signal_plugin_lv2::{Lv2HostedInstance, Lv2ProcessSession};
use signal_plugin_vst3::{Vst3HostedInstance, Vst3ProcessSession};
use signal_render_plane::PluginBlockProcessor;

/// One format-neutral plugin GUI callback drained from an in-process
/// backend (g12.024): the union of the CLAP `clap.gui` host callbacks and
/// the VST3 `IPlugFrame` resize requests, so the embedding host pumps one
/// event shape across formats (AU Cocoa views manage themselves — no
/// events).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PluginGuiEvent {
    /// The plugin's resize constraints changed (CLAP `resize_hints_changed`).
    ResizeHintsChanged,
    /// The plugin asks the host to resize its window (CLAP
    /// `request_resize` / VST3 `IPlugFrame::resizeView`).
    RequestResize {
        /// Requested content width (logical units on macOS).
        width: u32,
        /// Requested content height (logical units on macOS).
        height: u32,
    },
    /// The plugin asks the host to show its window (CLAP `request_show`).
    RequestShow,
    /// The plugin asks the host to hide its window (CLAP `request_hide`).
    RequestHide,
    /// The editor closed itself (CLAP `closed`).
    Closed {
        /// `true` when the plugin also destroyed the gui.
        was_destroyed: bool,
    },
}

impl From<signal_plugin_clap::ClapGuiEvent> for PluginGuiEvent {
    fn from(event: signal_plugin_clap::ClapGuiEvent) -> Self {
        match event {
            signal_plugin_clap::ClapGuiEvent::ResizeHintsChanged => Self::ResizeHintsChanged,
            signal_plugin_clap::ClapGuiEvent::RequestResize { width, height } => {
                Self::RequestResize { width, height }
            }
            signal_plugin_clap::ClapGuiEvent::RequestShow => Self::RequestShow,
            signal_plugin_clap::ClapGuiEvent::RequestHide => Self::RequestHide,
            signal_plugin_clap::ClapGuiEvent::Closed { was_destroyed } => {
                Self::Closed { was_destroyed }
            }
        }
    }
}

impl From<signal_plugin_vst3::Vst3GuiEvent> for PluginGuiEvent {
    fn from(event: signal_plugin_vst3::Vst3GuiEvent) -> Self {
        match event {
            signal_plugin_vst3::Vst3GuiEvent::RequestResize { width, height } => {
                Self::RequestResize { width, height }
            }
        }
    }
}

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
pub struct InProcessClapProcessor {
    /// Keeps the plugin instance (and its library) alive; lifecycle runs on
    /// drop. Field order matters: the session must drop before the
    /// instance.
    session: Mutex<ClapProcessSession>,
    instance: Mutex<ClapHostedInstance>,
    parameters: Vec<PluginParameterDescriptor>,
    max_frames: u32,
    /// Cleared at teardown so late callbacks bypass instead of racing the
    /// lifecycle.
    alive: AtomicBool,
    /// Blocks bypassed (unsupported layout, plugin error, teardown race).
    misses: AtomicU64,
}

// Safety: the raw plugin pointers inside the instance and session are only
// dereferenced behind the two mutexes; the type's public surface serializes
// all lifecycle and processing access.
unsafe impl Send for InProcessClapProcessor {}
unsafe impl Sync for InProcessClapProcessor {}

impl InProcessClapProcessor {
    /// Load `plugin_id` from `library_path` in the host process, activate
    /// it at `sample_rate_hz` / `max_frames`, and build the processing
    /// session. Rejects plugins outside the v1 stereo-effect layout with a
    /// stable token (`layout_unsupported`).
    pub fn load_and_activate(
        library_path: &std::path::Path,
        plugin_id: &str,
        sample_rate_hz: u32,
        max_frames: u32,
    ) -> Result<Self, String> {
        let mut instance =
            ClapHostedInstance::load(library_path, plugin_id).map_err(|error| error.token)?;
        if !instance.port_layout().is_stereo_effect() {
            return Err("layout_unsupported".to_string());
        }
        instance
            .activate(f64::from(sample_rate_hz), 1, max_frames)
            .map_err(|error| error.token)?;
        let session = instance.process_session().map_err(|error| error.token)?;
        let parameters = instance.parameters().to_vec();
        Ok(Self {
            session: Mutex::new(session),
            instance: Mutex::new(instance),
            parameters,
            max_frames,
            alive: AtomicBool::new(true),
            misses: AtomicU64::new(0),
        })
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
        instance
            .gui_open_embedded(parent_view as *mut std::ffi::c_void, scale)
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

    /// Propose a new editor content size (user drag or a granted
    /// `RequestResize`); returns the accepted size. MAIN THREAD ONLY.
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
        if !session.is_processing() && session.start().is_err() {
            self.misses.fetch_add(1, Ordering::Relaxed);
            return false;
        }
        let samples = frame_count * channels;
        if session.process_in_place(&mut scratch[..samples], frame_count) {
            true
        } else {
            self.misses.fetch_add(1, Ordering::Relaxed);
            false
        }
    }
}

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
    parameters: Vec<PluginParameterDescriptor>,
    max_frames: u32,
    /// Cleared at teardown so late callbacks bypass instead of racing the
    /// lifecycle.
    alive: AtomicBool,
    /// Blocks bypassed (unsupported layout, plugin error, teardown race).
    misses: AtomicU64,
}

// Safety: the raw COM pointers inside the instance and session are only
// dereferenced behind the two mutexes; the type's public surface serializes
// all lifecycle and processing access.
unsafe impl Send for InProcessVst3Processor {}
unsafe impl Sync for InProcessVst3Processor {}

impl InProcessVst3Processor {
    /// Load the component class `class_id_hex` from the bundle at
    /// `bundle_root` in the host process, activate it at `sample_rate_hz` /
    /// `max_frames`, and build the processing session. Rejects plugins
    /// outside the v1 stereo-effect layout with a stable token
    /// (`layout_unsupported`).
    pub fn load_and_activate(
        bundle_root: &std::path::Path,
        class_id_hex: &str,
        sample_rate_hz: u32,
        max_frames: u32,
    ) -> Result<Self, String> {
        let mut instance =
            Vst3HostedInstance::load(bundle_root, class_id_hex).map_err(|error| error.token)?;
        if !instance.port_layout().is_stereo_effect() {
            return Err("layout_unsupported".to_string());
        }
        instance
            .activate(f64::from(sample_rate_hz), 1, max_frames)
            .map_err(|error| error.token)?;
        let session = instance.process_session().map_err(|error| error.token)?;
        let parameters = instance.parameters().to_vec();
        Ok(Self {
            session: Mutex::new(session),
            instance: Mutex::new(instance),
            parameters,
            max_frames,
            alive: AtomicBool::new(true),
            misses: AtomicU64::new(0),
        })
    }

    /// Parameter inventory enumerated at load.
    pub fn parameters(&self) -> &[PluginParameterDescriptor] {
        &self.parameters
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
        instance
            .gui_open_embedded(parent_view as *mut std::ffi::c_void, scale)
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
        if !session.is_processing() && session.start().is_err() {
            self.misses.fetch_add(1, Ordering::Relaxed);
            return false;
        }
        let samples = frame_count * channels;
        if session.process_in_place(&mut scratch[..samples], frame_count) {
            true
        } else {
            self.misses.fetch_add(1, Ordering::Relaxed);
            false
        }
    }
}

/// In-process Audio Unit processing backend (g11.032): the exact mirror of
/// [`InProcessVst3Processor`] over the AU pull-model hosting FFI.
///
/// Owns the hosted instance (component instance, activation) for its whole
/// lifetime. The process session sits behind a `Mutex` taken with
/// `try_lock` only — the audio thread never blocks; a contended lock
/// (teardown racing a callback) bypasses that block.
///
/// The load path takes the same (path, key) pair as the other backends;
/// for AU the path is the registry sentinel (`au-registry.component`) and
/// the key is the fourcc triple resolved through the system registry.
pub struct InProcessAuProcessor {
    /// Field order matters: the session must drop before the instance (the
    /// session's drop uninstalls the render callback from the live unit).
    session: Mutex<AuProcessSession>,
    instance: Mutex<AuHostedInstance>,
    parameters: Vec<PluginParameterDescriptor>,
    max_frames: u32,
    /// Cleared at teardown so late callbacks bypass instead of racing the
    /// lifecycle.
    alive: AtomicBool,
    /// Blocks bypassed (unsupported layout, unit error, teardown race).
    misses: AtomicU64,
}

// Safety: the raw AudioUnit handle inside the instance and session is only
// dereferenced behind the two mutexes; the type's public surface serializes
// all lifecycle and processing access.
unsafe impl Send for InProcessAuProcessor {}
unsafe impl Sync for InProcessAuProcessor {}

impl InProcessAuProcessor {
    /// Resolve the fourcc `load_key` through the system AudioComponent
    /// registry (`library_path` is the never-opened sentinel), activate the
    /// unit at `sample_rate_hz` / `max_frames`, and build the processing
    /// session. Rejects units outside the v1 stereo-effect layout with a
    /// stable token (`layout_unsupported`); off macOS the load fails with
    /// `unsupported_platform`.
    pub fn load_and_activate(
        library_path: &std::path::Path,
        load_key: &str,
        sample_rate_hz: u32,
        max_frames: u32,
    ) -> Result<Self, String> {
        let mut instance =
            AuHostedInstance::load(library_path, load_key).map_err(|error| error.token)?;
        if !instance.port_layout().is_stereo_effect() {
            return Err("layout_unsupported".to_string());
        }
        instance
            .activate(f64::from(sample_rate_hz), 1, max_frames)
            .map_err(|error| error.token)?;
        let session = instance.process_session().map_err(|error| error.token)?;
        let parameters = instance.parameters().to_vec();
        Ok(Self {
            session: Mutex::new(session),
            instance: Mutex::new(instance),
            parameters,
            max_frames,
            alive: AtomicBool::new(true),
            misses: AtomicU64::new(0),
        })
    }

    /// Parameter inventory enumerated at load.
    pub fn parameters(&self) -> &[PluginParameterDescriptor] {
        &self.parameters
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
        instance
            .gui_open_embedded(parent_view as *mut std::ffi::c_void, scale)
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
        if !session.is_processing() && session.start().is_err() {
            self.misses.fetch_add(1, Ordering::Relaxed);
            return false;
        }
        let samples = frame_count * channels;
        if session.process_in_place(&mut scratch[..samples], frame_count) {
            true
        } else {
            self.misses.fetch_add(1, Ordering::Relaxed);
            false
        }
    }
}

/// In-process LV2 processing backend (g11.033): the exact mirror of
/// [`InProcessClapProcessor`] over the LV2 dlopen hosting FFI.
///
/// Owns the hosted instance (library, descriptor, instantiated handle,
/// connected buffers) for its whole lifetime. The process session sits
/// behind a `Mutex` taken with `try_lock` only — the audio thread never
/// blocks; a contended lock (teardown racing a callback) bypasses that
/// block.
///
/// LV2 is a push model with no start/stop-processing handshake; the lazy
/// `start()` on the first processed block is surface parity only.
pub struct InProcessLv2Processor {
    /// Field order matters: the session must drop before the instance
    /// (its raw pointers target the instance-owned port buffers).
    session: Mutex<Lv2ProcessSession>,
    instance: Mutex<Lv2HostedInstance>,
    parameters: Vec<PluginParameterDescriptor>,
    max_frames: u32,
    /// Cleared at teardown so late callbacks bypass instead of racing the
    /// lifecycle.
    alive: AtomicBool,
    /// Blocks bypassed (unsupported layout, dead handle, teardown race).
    misses: AtomicU64,
}

// Safety: the raw plugin handle and buffer pointers inside the instance
// and session are only dereferenced behind the two mutexes; the type's
// public surface serializes all lifecycle and processing access.
unsafe impl Send for InProcessLv2Processor {}
unsafe impl Sync for InProcessLv2Processor {}

impl InProcessLv2Processor {
    /// Load `plugin_uri` from the `.lv2` bundle at `bundle_root` in the
    /// host process (re-parsing the bundle TTL for the port model),
    /// activate it at `sample_rate_hz` / `max_frames` (LV2 instantiates
    /// here — the rate is fixed at instantiate), and build the processing
    /// session. Rejects plugins outside the v1 stereo-effect layout
    /// (including required atom/event inputs) with a stable token
    /// (`layout_unsupported`).
    pub fn load_and_activate(
        bundle_root: &std::path::Path,
        plugin_uri: &str,
        sample_rate_hz: u32,
        max_frames: u32,
    ) -> Result<Self, String> {
        let mut instance =
            Lv2HostedInstance::load(bundle_root, plugin_uri).map_err(|error| error.token)?;
        if !instance.port_layout().is_stereo_effect() {
            return Err("layout_unsupported".to_string());
        }
        instance
            .activate(f64::from(sample_rate_hz), 1, max_frames)
            .map_err(|error| error.token)?;
        let session = instance.process_session().map_err(|error| error.token)?;
        let parameters = instance.parameters().to_vec();
        Ok(Self {
            session: Mutex::new(session),
            instance: Mutex::new(instance),
            parameters,
            max_frames,
            alive: AtomicBool::new(true),
            misses: AtomicU64::new(0),
        })
    }

    /// Parameter inventory from the bundle TTL (control input ports).
    pub fn parameters(&self) -> &[PluginParameterDescriptor] {
        &self.parameters
    }

    /// Queue one normalized 0..1 parameter write (g12.023): maps onto the
    /// control port's TTL range and lands in the connected slot at the top
    /// of the next `run()`. Not part of the audio path — takes the
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

    /// Blocks bypassed so far, cumulative.
    pub fn miss_count(&self) -> u64 {
        self.misses.load(Ordering::Relaxed)
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

impl Drop for InProcessLv2Processor {
    fn drop(&mut self) {
        self.alive.store(false, Ordering::Relaxed);
        if let Ok(mut session) = self.session.lock() {
            session.stop();
        }
        if let Ok(mut instance) = self.instance.lock() {
            let _ = instance.deactivate();
        }
        // The session field drops first (raw pointers go inert), then the
        // instance's own Drop runs deactivate → cleanup → dlclose.
    }
}

impl PluginBlockProcessor for InProcessLv2Processor {
    fn process(&self, scratch: &mut [f32], frame_count: usize, channels: usize) -> bool {
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
        if !session.is_processing() && session.start().is_err() {
            self.misses.fetch_add(1, Ordering::Relaxed);
            return false;
        }
        let samples = frame_count * channels;
        if session.process_in_place(&mut scratch[..samples], frame_count) {
            true
        } else {
            self.misses.fetch_add(1, Ordering::Relaxed);
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use signal_plugin_clap::fixture::{compile_clap_fixture, rustc_available, CLAP_FIXTURE_GAIN};
    use signal_plugin_vst3::fixture::{
        compile_vst3_fixture, VST3_FIXTURE_CLASS_ID_HEX, VST3_FIXTURE_GAIN,
    };
    use signal_render_plane::RenderPluginProcessor;
    use std::sync::Arc;

    #[test]
    fn in_process_backend_loads_and_processes_the_fixture() {
        if !rustc_available() {
            eprintln!("skipping: rustc unavailable for the CLAP fixture");
            return;
        }
        let directory = std::env::temp_dir().join(format!(
            "signal-plugin-bridge-inproc-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos(),
        ));
        let library = compile_clap_fixture(
            &directory,
            "com.signal.bridge-inproc",
            "Signal Bridge InProc",
            0,
        )
        .expect("fixture should compile");

        let backend = Arc::new(
            InProcessClapProcessor::load_and_activate(
                &library,
                "com.signal.bridge-inproc",
                48_000,
                256,
            )
            .expect("backend should load and activate"),
        );
        assert_eq!(backend.parameters().len(), 2);
        let handle = RenderPluginProcessor::new(Arc::clone(&backend) as Arc<_>);

        let mut scratch: Vec<f32> = (0..256).map(|index| index as f32 / 256.0).collect();
        let reference = scratch.clone();
        assert!(handle.process(&mut scratch, 128, 2));
        for (index, (output, input)) in scratch.iter().zip(reference.iter()).enumerate() {
            assert!(
                (output - input * CLAP_FIXTURE_GAIN).abs() < 1e-7,
                "sample {index}: {output} vs {input} * {CLAP_FIXTURE_GAIN}",
            );
        }
        assert_eq!(backend.miss_count(), 0);

        // Shutdown: later blocks bypass and leave scratch untouched.
        backend.shutdown();
        let mut scratch = reference.clone();
        assert!(!handle.process(&mut scratch, 128, 2));
        assert_eq!(scratch, reference);
        assert_eq!(backend.miss_count(), 1);

        drop(handle);
        drop(backend);
        let _ = std::fs::remove_dir_all(&directory);
    }

    /// g12.023: the CLAP set-then-process proof — a wire param write lands
    /// in the plugin's DSP via process in-events, byte-exact, at the next
    /// block (block-boundary posture).
    #[test]
    fn in_process_clap_param_set_reaches_the_dsp_next_block() {
        use signal_plugin_clap::fixture::CLAP_FIXTURE_GAIN_PARAM_ID;

        if !rustc_available() {
            eprintln!("skipping: rustc unavailable for the CLAP fixture");
            return;
        }
        let directory = std::env::temp_dir().join(format!(
            "signal-plugin-bridge-inproc-set-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos(),
        ));
        let library = compile_clap_fixture(
            &directory,
            "com.signal.bridge-inproc-set",
            "Signal Bridge InProc Set",
            0,
        )
        .expect("fixture should compile");

        let backend = Arc::new(
            InProcessClapProcessor::load_and_activate(
                &library,
                "com.signal.bridge-inproc-set",
                48_000,
                256,
            )
            .expect("backend should load and activate"),
        );
        let handle = RenderPluginProcessor::new(Arc::clone(&backend) as Arc<_>);

        // Block 1: fixture default gain.
        let reference: Vec<f32> = (0..256).map(|index| index as f32 / 256.0).collect();
        let mut scratch = reference.clone();
        assert!(handle.process(&mut scratch, 128, 2));
        for (output, input) in scratch.iter().zip(reference.iter()) {
            assert!((output - input * CLAP_FIXTURE_GAIN).abs() < 1e-7);
        }

        // Set Gain (plain range 0..1, so normalized == plain) mid-stream:
        // the NEXT block applies the new value exactly.
        backend
            .set_parameter_normalized(CLAP_FIXTURE_GAIN_PARAM_ID, 0.25)
            .expect("param set queues");
        let mut scratch = reference.clone();
        assert!(handle.process(&mut scratch, 128, 2));
        for (index, (output, input)) in scratch.iter().zip(reference.iter()).enumerate() {
            assert!(
                (output - input * 0.25).abs() < 1e-7,
                "sample {index}: {output} vs {input} * 0.25",
            );
        }

        // Unknown parameters and dead backends fail typed.
        assert_eq!(
            backend.set_parameter_normalized(9999, 0.5).unwrap_err(),
            "unknown_parameter",
        );
        backend.shutdown();
        assert_eq!(
            backend
                .set_parameter_normalized(CLAP_FIXTURE_GAIN_PARAM_ID, 0.5)
                .unwrap_err(),
            "backend_dead",
        );

        drop(handle);
        drop(backend);
        let _ = std::fs::remove_dir_all(&directory);
    }

    /// g12.023: the VST3 mirror — the write rides the block's input
    /// `IParameterChanges` and the fixture's processor applies it.
    #[test]
    fn in_process_vst3_param_set_reaches_the_dsp_next_block() {
        use signal_plugin_vst3::fixture::VST3_FIXTURE_GAIN_PARAM_ID;

        if !rustc_available() {
            eprintln!("skipping: rustc unavailable for the VST3 fixture");
            return;
        }
        let directory = std::env::temp_dir().join(format!(
            "signal-plugin-bridge-inproc-vst3-set-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos(),
        ));
        let bundle = compile_vst3_fixture(
            &directory,
            "plugin:vst3:bridge-inproc-set",
            "Signal Bridge InProc VST3 Set",
        )
        .expect("vst3 fixture should compile");

        let backend = Arc::new(
            InProcessVst3Processor::load_and_activate(
                &bundle,
                VST3_FIXTURE_CLASS_ID_HEX,
                48_000,
                256,
            )
            .expect("backend should load and activate"),
        );
        let handle = RenderPluginProcessor::new(Arc::clone(&backend) as Arc<_>);

        let reference: Vec<f32> = (0..256).map(|index| index as f32 / 256.0).collect();
        let mut scratch = reference.clone();
        assert!(handle.process(&mut scratch, 128, 2));
        for (output, input) in scratch.iter().zip(reference.iter()) {
            assert!((output - input * VST3_FIXTURE_GAIN).abs() < 1e-7);
        }

        backend
            .set_parameter_normalized(VST3_FIXTURE_GAIN_PARAM_ID, 0.75)
            .expect("param set queues");
        let mut scratch = reference.clone();
        assert!(handle.process(&mut scratch, 128, 2));
        for (index, (output, input)) in scratch.iter().zip(reference.iter()).enumerate() {
            assert!(
                (output - input * 0.75).abs() < 1e-7,
                "sample {index}: {output} vs {input} * 0.75",
            );
        }
        assert_eq!(
            backend.set_parameter_normalized(9999, 0.5).unwrap_err(),
            "unknown_parameter",
        );

        drop(handle);
        drop(backend);
        let _ = std::fs::remove_dir_all(&directory);
    }

    /// g12.023: the LV2 mirror — the write lands in the connected Gain
    /// control slot before the next `run()`.
    #[test]
    fn in_process_lv2_param_set_reaches_the_dsp_next_block() {
        use signal_plugin_lv2::fixture::{
            compile_lv2_fixture, rustc_available as lv2_rustc_available, LV2_FIXTURE_GAIN,
            LV2_FIXTURE_GAIN_PORT_INDEX,
        };
        if !lv2_rustc_available() {
            eprintln!("skipping: rustc unavailable for the LV2 fixture");
            return;
        }
        let directory = std::env::temp_dir().join(format!(
            "signal-plugin-bridge-inproc-lv2-set-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos(),
        ));
        let plugin_uri = "https://signal.dev/fixtures/lv2/bridge-inproc-set";
        let bundle = compile_lv2_fixture(&directory, plugin_uri, "Signal Bridge InProc LV2 Set")
            .expect("lv2 fixture should compile");

        let backend = Arc::new(
            InProcessLv2Processor::load_and_activate(&bundle, plugin_uri, 48_000, 256)
                .expect("backend should load and activate"),
        );
        let handle = RenderPluginProcessor::new(Arc::clone(&backend) as Arc<_>);

        let reference: Vec<f32> = (0..256).map(|index| index as f32 / 256.0).collect();
        let mut scratch = reference.clone();
        assert!(handle.process(&mut scratch, 128, 2));
        for (output, input) in scratch.iter().zip(reference.iter()) {
            assert!((output - input * LV2_FIXTURE_GAIN).abs() < 1e-7);
        }

        // Gain port TTL range is 0..1, so normalized == plain.
        backend
            .set_parameter_normalized(LV2_FIXTURE_GAIN_PORT_INDEX, 1.0)
            .expect("param set queues");
        let mut scratch = reference.clone();
        assert!(handle.process(&mut scratch, 128, 2));
        for (index, (output, input)) in scratch.iter().zip(reference.iter()).enumerate() {
            assert!(
                (output - input).abs() < 1e-7,
                "sample {index}: {output} vs {input} (unity gain)",
            );
        }
        assert_eq!(
            backend.set_parameter_normalized(9999, 0.5).unwrap_err(),
            "unknown_parameter",
        );

        drop(handle);
        drop(backend);
        let _ = std::fs::remove_dir_all(&directory);
    }

    /// g12.022: gui lifecycle through the in-process backend's delegates —
    /// the exact surface the Tauri host calls (open/size/resize/events/
    /// close), offscreen against the fixture's bookkeeping gui, while the
    /// audio path keeps processing (gui takes the instance lock, never the
    /// session lock).
    #[test]
    fn in_process_backend_hosts_the_fixture_gui_offscreen() {
        use signal_plugin_clap::fixture::{
            CLAP_FIXTURE_GUI_INITIAL_SIZE, CLAP_FIXTURE_GUI_REQUESTED_SIZE,
        };

        if !rustc_available() {
            eprintln!("skipping: rustc unavailable for the CLAP fixture");
            return;
        }
        let directory = std::env::temp_dir().join(format!(
            "signal-plugin-bridge-inproc-gui-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos(),
        ));
        let library = compile_clap_fixture(
            &directory,
            "com.signal.bridge-inproc-gui",
            "Signal Bridge InProc Gui",
            0,
        )
        .expect("fixture should compile");

        let backend = Arc::new(
            InProcessClapProcessor::load_and_activate(
                &library,
                "com.signal.bridge-inproc-gui",
                48_000,
                256,
            )
            .expect("backend should load and activate"),
        );
        assert!(backend.gui_supported());
        assert!(!backend.gui_is_open());
        assert_eq!(backend.gui_size(), None);

        let mut fake_parent = 0u8;
        let size = backend
            .gui_open_embedded(&mut fake_parent as *mut u8 as usize, None)
            .expect("gui opens");
        assert_eq!(size, CLAP_FIXTURE_GUI_INITIAL_SIZE);
        assert!(backend.gui_is_open());
        assert_eq!(backend.gui_size(), Some(CLAP_FIXTURE_GUI_INITIAL_SIZE));
        assert!(backend.gui_can_resize());

        // Audio still processes with the editor open.
        let handle = RenderPluginProcessor::new(Arc::clone(&backend) as Arc<_>);
        let mut scratch: Vec<f32> = (0..256).map(|index| index as f32 / 256.0).collect();
        assert!(handle.process(&mut scratch, 128, 2));

        // Fixture show() queued a host resize request.
        let events = backend.gui_take_events();
        assert!(events.contains(&PluginGuiEvent::RequestResize {
            width: CLAP_FIXTURE_GUI_REQUESTED_SIZE.0,
            height: CLAP_FIXTURE_GUI_REQUESTED_SIZE.1,
        }));

        // Granting the request through set_size sticks.
        assert_eq!(
            backend.gui_set_size(
                CLAP_FIXTURE_GUI_REQUESTED_SIZE.0,
                CLAP_FIXTURE_GUI_REQUESTED_SIZE.1
            ),
            Some(CLAP_FIXTURE_GUI_REQUESTED_SIZE)
        );
        assert_eq!(backend.gui_size(), Some(CLAP_FIXTURE_GUI_REQUESTED_SIZE));

        backend.gui_close();
        assert!(!backend.gui_is_open());
        backend.gui_close(); // idempotent

        // Dead backends refuse to open editors.
        backend.shutdown();
        let refused = backend.gui_open_embedded(&mut fake_parent as *mut u8 as usize, None);
        assert_eq!(refused.unwrap_err(), "backend_dead");

        drop(handle);
        drop(backend);
        let _ = std::fs::remove_dir_all(&directory);
    }

    #[test]
    fn in_process_vst3_backend_loads_and_processes_the_fixture() {
        if !rustc_available() {
            eprintln!("skipping: rustc unavailable for the VST3 fixture");
            return;
        }
        let directory = std::env::temp_dir().join(format!(
            "signal-plugin-bridge-inproc-vst3-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos(),
        ));
        let bundle = compile_vst3_fixture(
            &directory,
            "plugin:vst3:bridge-inproc",
            "Signal Bridge InProc VST3",
        )
        .expect("vst3 fixture should compile");

        let backend = Arc::new(
            InProcessVst3Processor::load_and_activate(
                &bundle,
                VST3_FIXTURE_CLASS_ID_HEX,
                48_000,
                256,
            )
            .expect("backend should load and activate"),
        );
        assert_eq!(backend.parameters().len(), 2);
        let handle = RenderPluginProcessor::new(Arc::clone(&backend) as Arc<_>);

        let mut scratch: Vec<f32> = (0..256).map(|index| index as f32 / 256.0).collect();
        let reference = scratch.clone();
        assert!(handle.process(&mut scratch, 128, 2));
        for (index, (output, input)) in scratch.iter().zip(reference.iter()).enumerate() {
            assert!(
                (output - input * VST3_FIXTURE_GAIN).abs() < 1e-7,
                "sample {index}: {output} vs {input} * {VST3_FIXTURE_GAIN}",
            );
        }
        assert_eq!(backend.miss_count(), 0);

        // Shutdown: later blocks bypass and leave scratch untouched.
        backend.shutdown();
        let mut scratch = reference.clone();
        assert!(!handle.process(&mut scratch, 128, 2));
        assert_eq!(scratch, reference);
        assert_eq!(backend.miss_count(), 1);

        drop(handle);
        drop(backend);
        let _ = std::fs::remove_dir_all(&directory);
    }

    /// The LV2 mirror of the in-process gain proof: wet = dry × the Gain
    /// control port's non-unity TTL default (no param set exists phase 1),
    /// byte-exact through the render handle, then shutdown-bypass leaves
    /// the scratch untouched.
    #[test]
    fn in_process_lv2_backend_loads_and_processes_the_fixture() {
        use signal_plugin_lv2::fixture::{
            compile_lv2_fixture, rustc_available as lv2_rustc_available, LV2_FIXTURE_GAIN,
        };
        if !lv2_rustc_available() {
            eprintln!("skipping: rustc unavailable for the LV2 fixture");
            return;
        }
        let directory = std::env::temp_dir().join(format!(
            "signal-plugin-bridge-inproc-lv2-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos(),
        ));
        let plugin_uri = "https://signal.dev/fixtures/lv2/bridge-inproc";
        let bundle = compile_lv2_fixture(&directory, plugin_uri, "Signal Bridge InProc LV2")
            .expect("lv2 fixture should compile");

        let backend = Arc::new(
            InProcessLv2Processor::load_and_activate(&bundle, plugin_uri, 48_000, 256)
                .expect("backend should load and activate"),
        );
        assert_eq!(backend.parameters().len(), 2);
        let handle = RenderPluginProcessor::new(Arc::clone(&backend) as Arc<_>);

        let mut scratch: Vec<f32> = (0..256).map(|index| index as f32 / 256.0).collect();
        let reference = scratch.clone();
        assert!(handle.process(&mut scratch, 128, 2));
        for (index, (output, input)) in scratch.iter().zip(reference.iter()).enumerate() {
            assert!(
                (output - input * LV2_FIXTURE_GAIN).abs() < 1e-7,
                "sample {index}: {output} vs {input} * {LV2_FIXTURE_GAIN}",
            );
        }
        assert_eq!(backend.miss_count(), 0);

        // Shutdown: later blocks bypass and leave scratch untouched.
        backend.shutdown();
        let mut scratch = reference.clone();
        assert!(!handle.process(&mut scratch, 128, 2));
        assert_eq!(scratch, reference);
        assert_eq!(backend.miss_count(), 1);

        drop(handle);
        drop(backend);
        let _ = std::fs::remove_dir_all(&directory);
    }

    /// g12.024: the VST3 IPlugView mirror of the CLAP gui lifecycle test —
    /// the exact surface the Tauri host calls (open/size/resize/events/
    /// close), offscreen against the fixture's bookkeeping view, while the
    /// audio path keeps processing (gui takes the instance lock, never the
    /// session lock).
    #[test]
    fn in_process_vst3_backend_hosts_the_fixture_view_offscreen() {
        use signal_plugin_vst3::fixture::{
            VST3_FIXTURE_VIEW_INITIAL_SIZE, VST3_FIXTURE_VIEW_REQUESTED_SIZE,
        };

        if !rustc_available() {
            eprintln!("skipping: rustc unavailable for the VST3 fixture");
            return;
        }
        let directory = std::env::temp_dir().join(format!(
            "signal-plugin-bridge-inproc-vst3-gui-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos(),
        ));
        let bundle = compile_vst3_fixture(
            &directory,
            "plugin:vst3:bridge-inproc-gui",
            "Signal Bridge InProc VST3 Gui",
        )
        .expect("vst3 fixture should compile");

        let backend = Arc::new(
            InProcessVst3Processor::load_and_activate(
                &bundle,
                VST3_FIXTURE_CLASS_ID_HEX,
                48_000,
                256,
            )
            .expect("backend should load and activate"),
        );
        assert!(backend.gui_supported(), "load-time createView probe");
        assert!(!backend.gui_is_open());
        assert_eq!(backend.gui_size(), None);

        let mut fake_parent = 0u8;
        let size = backend
            .gui_open_embedded(&mut fake_parent as *mut u8 as usize, None)
            .expect("view opens");
        assert_eq!(size, VST3_FIXTURE_VIEW_INITIAL_SIZE);
        assert!(backend.gui_is_open());
        assert_eq!(backend.gui_size(), Some(VST3_FIXTURE_VIEW_INITIAL_SIZE));
        assert!(backend.gui_can_resize());

        // Audio still processes with the editor open.
        let handle = RenderPluginProcessor::new(Arc::clone(&backend) as Arc<_>);
        let mut scratch: Vec<f32> = (0..256).map(|index| index as f32 / 256.0).collect();
        assert!(handle.process(&mut scratch, 128, 2));

        // Fixture attached() asked the host IPlugFrame for a resize.
        let events = backend.gui_take_events();
        assert!(events.contains(&PluginGuiEvent::RequestResize {
            width: VST3_FIXTURE_VIEW_REQUESTED_SIZE.0,
            height: VST3_FIXTURE_VIEW_REQUESTED_SIZE.1,
        }));

        // Granting the request through set_size (checkSizeConstraint →
        // onSize) sticks.
        assert_eq!(
            backend.gui_set_size(
                VST3_FIXTURE_VIEW_REQUESTED_SIZE.0,
                VST3_FIXTURE_VIEW_REQUESTED_SIZE.1
            ),
            Some(VST3_FIXTURE_VIEW_REQUESTED_SIZE)
        );
        assert_eq!(backend.gui_size(), Some(VST3_FIXTURE_VIEW_REQUESTED_SIZE));

        backend.gui_close();
        assert!(!backend.gui_is_open());
        backend.gui_close(); // idempotent

        // Dead backends refuse to open editors.
        backend.shutdown();
        let refused = backend.gui_open_embedded(&mut fake_parent as *mut u8 as usize, None);
        assert_eq!(refused.unwrap_err(), "backend_dead");

        drop(handle);
        drop(backend);
        let _ = std::fs::remove_dir_all(&directory);
    }

    /// g12.024: plugin GUI → host param sync — the fixture's gui `show`
    /// stands in for an editor tweak, pushing a Gain PARAM_VALUE out-event
    /// at the next processed block; the host drains it normalized and the
    /// DSP already runs at the tweaked gain.
    #[test]
    fn in_process_clap_gui_param_tweak_reaches_the_host_via_out_events() {
        use signal_plugin_clap::fixture::{
            CLAP_FIXTURE_GAIN_PARAM_ID, CLAP_FIXTURE_GUI_PARAM_OUT_VALUE,
        };

        if !rustc_available() {
            eprintln!("skipping: rustc unavailable for the CLAP fixture");
            return;
        }
        let directory = std::env::temp_dir().join(format!(
            "signal-plugin-bridge-inproc-gui-out-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos(),
        ));
        let library = compile_clap_fixture(
            &directory,
            "com.signal.bridge-inproc-gui-out",
            "Signal Bridge InProc Gui Out",
            0,
        )
        .expect("fixture should compile");

        let backend = Arc::new(
            InProcessClapProcessor::load_and_activate(
                &library,
                "com.signal.bridge-inproc-gui-out",
                48_000,
                256,
            )
            .expect("backend should load and activate"),
        );
        let handle = RenderPluginProcessor::new(Arc::clone(&backend) as Arc<_>);

        // No editor interaction yet: no out-events.
        let reference: Vec<f32> = (0..256).map(|index| index as f32 / 256.0).collect();
        let mut scratch = reference.clone();
        assert!(handle.process(&mut scratch, 128, 2));
        assert!(backend.take_param_out_events().is_empty());

        // Open + the fixture's show() queues the stand-in editor tweak.
        let mut fake_parent = 0u8;
        backend
            .gui_open_embedded(&mut fake_parent as *mut u8 as usize, None)
            .expect("gui opens");

        // The tweak lands at the next processed block: audible in the DSP
        // and drained by the host as a normalized (id, value) pair (the
        // fixture Gain's plain range is 0..1, so plain == normalized).
        let mut scratch = reference.clone();
        assert!(handle.process(&mut scratch, 128, 2));
        for (index, (output, input)) in scratch.iter().zip(reference.iter()).enumerate() {
            assert!(
                (f64::from(*output) - f64::from(*input) * CLAP_FIXTURE_GUI_PARAM_OUT_VALUE).abs()
                    < 1e-7,
                "sample {index}: {output} vs {input} * {CLAP_FIXTURE_GUI_PARAM_OUT_VALUE}",
            );
        }
        let drained = backend.take_param_out_events();
        assert_eq!(drained.len(), 1);
        assert_eq!(drained[0].0, CLAP_FIXTURE_GAIN_PARAM_ID);
        assert!((f64::from(drained[0].1) - CLAP_FIXTURE_GUI_PARAM_OUT_VALUE).abs() < 1e-6);
        // Drained means drained: the next take is empty.
        assert!(backend.take_param_out_events().is_empty());

        // The fixture's show() also exercised the host clap.params wiring
        // (request_flush) — observable through the params-event drain.
        let params_events = backend.take_params_events();
        assert!(params_events.contains(&signal_plugin_clap::ClapHostParamsEvent::FlushRequested));
        assert!(backend.take_params_events().is_empty());

        backend.gui_close();
        drop(handle);
        drop(backend);
        let _ = std::fs::remove_dir_all(&directory);
    }

    /// The AU mirror of the in-process identity proof — against the stock
    /// Apple AUDelay (no compiled fixture; the AudioComponent registrar
    /// cannot see temp bundles). WetDryMix=0 makes the delay line inert, so
    /// output ≈ input within 1e-6 per sample (AU float paths are
    /// unspecified — never byte-exact).
    #[cfg(target_os = "macos")]
    #[test]
    fn in_process_au_backend_is_identity_when_fully_dry() {
        const AUDELAY_WET_DRY_MIX: u32 = 0;
        let backend = Arc::new(
            InProcessAuProcessor::load_and_activate(
                std::path::Path::new(signal_plugin_au::AU_REGISTRY_COMPONENT_PATH),
                "aufx:dely:appl",
                48_000,
                256,
            )
            .expect("stock AUDelay should load and activate in-process"),
        );
        assert!(!backend.parameters().is_empty());
        backend
            .set_parameter(AUDELAY_WET_DRY_MIX, 0.0)
            .expect("wet/dry mix set");
        let handle = RenderPluginProcessor::new(Arc::clone(&backend) as Arc<_>);

        let mut scratch: Vec<f32> = (0..256).map(|index| index as f32 / 256.0 - 0.5).collect();
        let reference = scratch.clone();
        assert!(handle.process(&mut scratch, 128, 2));
        for (index, (output, input)) in scratch.iter().zip(reference.iter()).enumerate() {
            assert!(
                (output - input).abs() <= 1e-6,
                "sample {index}: {output} vs {input} (identity, epsilon 1e-6)",
            );
        }
        assert_eq!(backend.miss_count(), 0);

        // Shutdown: later blocks bypass and leave scratch untouched.
        backend.shutdown();
        let mut scratch = reference.clone();
        assert!(!handle.process(&mut scratch, 128, 2));
        assert_eq!(scratch, reference);
        assert_eq!(backend.miss_count(), 1);

        drop(handle);
        drop(backend);
    }
}
