//! In-process LV2 plugin processor.

use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Mutex;

use signal_plugin::PluginParameterDescriptor;
use signal_plugin_lv2::{Lv2HostedInstance, Lv2ProcessSession};
use signal_render_plane::{PluginBlockProcessor, RenderBlockPluginEvent};

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
    unsupported_events: AtomicU64,
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
            unsupported_events: AtomicU64::new(0),
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
        self.unsupported_events
            .fetch_add(events.len() as u64, Ordering::Relaxed);
        self.process(scratch, frame_count, channels)
    }
}
