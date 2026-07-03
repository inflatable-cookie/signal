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

    /// Parameter inventory enumerated at load (read-only phase 1).
    pub fn parameters(&self) -> &[PluginParameterDescriptor] {
        &self.parameters
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

    /// Parameter inventory enumerated at load (read-only phase 1).
    pub fn parameters(&self) -> &[PluginParameterDescriptor] {
        &self.parameters
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

    /// Parameter inventory enumerated at load (read-only phase 1).
    pub fn parameters(&self) -> &[PluginParameterDescriptor] {
        &self.parameters
    }

    /// Set one parameter's plain value on the hosted unit
    /// (`AudioUnitSetParameter`). Phase 1 uses this from tests only; it is
    /// the natural phase-2 param-set seam. Not part of the audio path —
    /// takes the instance lock.
    pub fn set_parameter(&self, parameter_id: u32, value: f32) -> Result<(), String> {
        let mut instance = self
            .instance
            .lock()
            .map_err(|_| "instance_lock_poisoned".to_string())?;
        instance
            .set_parameter(parameter_id, value)
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

    /// Parameter inventory from the bundle TTL (control input ports;
    /// read-only phase 1).
    pub fn parameters(&self) -> &[PluginParameterDescriptor] {
        &self.parameters
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
