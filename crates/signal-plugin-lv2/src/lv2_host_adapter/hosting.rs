//! In-child LV2 instance hosting: bundle re-parse, dlopen +
//! `lv2_descriptor(index)` walk, instance lifecycle (instantiate at
//! activate / connect ports / run / deactivate / cleanup), control ports as
//! the parameter inventory, and a raw process session for the sandbox
//! audio thread — the LV2 mirror of the CLAP/VST3/AU hosting modules.
//!
//! # FFI design
//!
//! The LV2 C ABI is tiny and handwritten here (house precedent — no
//! `lv2`-crate dependency): one `#[repr(C)]` descriptor struct returned by
//! the library's `lv2_descriptor(uint32_t)` export, walked until the entry
//! whose `URI` matches the load key. The binary self-describes nothing but
//! that URI — the port model comes from re-parsing the bundle TTL at load
//! (`introspection::parse_lv2_bundle`, the same functions discovery uses),
//! paralleling AU's rebuild-description-from-load-key.
//!
//! LV2 is a pure push model: no COM, no pull callback, no
//! start/stop-processing handshake. `instantiate` fixes the sample rate,
//! so it runs at ACTIVATE (the wire delivers the rate there), not load.
//! Activation connects every port once — audio ports to preallocated
//! planar buffers, control ports to boxed slots holding their TTL
//! defaults — and `run(n)` per block does the rest. Drop order is
//! deactivate → cleanup → dlclose (the `Library` field is declared last).
//!
//! # Features
//!
//! The host provides `urid:map` ONLY (packet g11.033 decision 3): an
//! interned string→u32 map behind a `Mutex`, handed over as a boxed
//! feature in a NULL-terminated array kept alive for the instance's
//! lifetime. Any other `lv2:requiredFeature` fails the load with the typed
//! `unsupported_required_feature` token (scan pre-filters the same set).

use std::collections::HashMap;
use std::ffi::{c_char, c_void, CStr, CString};
use std::path::Path;
use std::ptr;
use std::sync::{Arc, Mutex};

use libloading::Library;
use signal_plugin::{
    PluginParamChange, PluginParamChangeQueue, PluginParameterDescriptor,
    PLUGIN_PARAM_CHANGE_CAPACITY,
};

use super::introspection::{
    parameter_descriptors_from_model, parse_lv2_bundle, Lv2PluginModel, URID_MAP_FEATURE,
};

/// Error surface for LV2 hosting operations; carries a stable snake_case
/// token suitable for broker receipt details (mirrors `ClapHostingError`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Lv2HostingError {
    /// Stable snake_case failure token (e.g. `library_open_failed`).
    pub token: String,
}

impl Lv2HostingError {
    fn new(token: impl Into<String>) -> Self {
        Self {
            token: token.into(),
        }
    }
}

impl std::fmt::Display for Lv2HostingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.token)
    }
}

impl std::error::Error for Lv2HostingError {}

// ── LV2 C ABI ───────────────────────────────────────────────────────────────

/// `LV2_Feature`.
#[repr(C)]
pub(crate) struct Lv2Feature {
    uri: *const c_char,
    data: *mut c_void,
}

/// `LV2_Descriptor` (lv2core.h layout).
#[repr(C)]
struct Lv2DescriptorRaw {
    uri: *const c_char,
    instantiate: Option<
        unsafe extern "C" fn(
            *const Lv2DescriptorRaw,
            f64,
            *const c_char,
            *const *const Lv2Feature,
        ) -> *mut c_void,
    >,
    connect_port: Option<unsafe extern "C" fn(*mut c_void, u32, *mut c_void)>,
    activate: Option<unsafe extern "C" fn(*mut c_void)>,
    run: Option<unsafe extern "C" fn(*mut c_void, u32)>,
    deactivate: Option<unsafe extern "C" fn(*mut c_void)>,
    cleanup: Option<unsafe extern "C" fn(*mut c_void)>,
    extension_data: Option<unsafe extern "C" fn(*const c_char) -> *const c_void>,
}

/// `const LV2_Descriptor* lv2_descriptor(uint32_t index)`.
type Lv2DescriptorProc = unsafe extern "C" fn(u32) -> *const Lv2DescriptorRaw;

/// `LV2_URID_Map` (urid.h layout).
#[repr(C)]
struct Lv2UridMap {
    handle: *mut c_void,
    map: unsafe extern "C" fn(*mut c_void, *const c_char) -> u32,
}

// ── urid:map feature ────────────────────────────────────────────────────────

/// Interned string→u32 URID map state. URIDs start at 1 (0 is the LV2
/// reserved "no URID" value).
struct UridMapState {
    interned: Mutex<HashMap<Vec<u8>, u32>>,
}

unsafe extern "C" fn urid_map_callback(handle: *mut c_void, uri: *const c_char) -> u32 {
    if handle.is_null() || uri.is_null() {
        return 0;
    }
    // Safety: `handle` is the boxed `UridMapState` owned by the hosting
    // instance, alive for the plugin's whole lifetime; `uri` is a
    // NUL-terminated string per the LV2 URID contract.
    let state = unsafe { &*(handle as *const UridMapState) };
    let key = unsafe { CStr::from_ptr(uri) }.to_bytes().to_vec();
    let Ok(mut interned) = state.interned.lock() else {
        return 0;
    };
    let next = interned.len() as u32 + 1;
    *interned.entry(key).or_insert(next)
}

/// The urid:map feature bundle: boxed state, boxed `LV2_URID_Map`, boxed
/// `LV2_Feature`, and the NULL-terminated features array — all owned here
/// so every pointer the plugin may retain stays valid for the instance
/// lifetime (boxes give stable addresses even when this struct moves).
struct UridMapFeatureSet {
    _state: Box<UridMapState>,
    _map: Box<Lv2UridMap>,
    _uri: CString,
    _feature: Box<Lv2Feature>,
    features: Vec<*const Lv2Feature>,
}

impl UridMapFeatureSet {
    fn new() -> Self {
        let state = Box::new(UridMapState {
            interned: Mutex::new(HashMap::new()),
        });
        let map = Box::new(Lv2UridMap {
            handle: (&*state as *const UridMapState) as *mut c_void,
            map: urid_map_callback,
        });
        let uri = CString::new(URID_MAP_FEATURE).expect("static feature URI has no NUL");
        let feature = Box::new(Lv2Feature {
            uri: uri.as_ptr(),
            data: (&*map as *const Lv2UridMap) as *mut c_void,
        });
        let features = vec![&*feature as *const Lv2Feature, ptr::null()];
        Self {
            _state: state,
            _map: map,
            _uri: uri,
            _feature: feature,
            features,
        }
    }

    fn as_ptr(&self) -> *const *const Lv2Feature {
        self.features.as_ptr()
    }
}

// ── Hosted instance ─────────────────────────────────────────────────────────

/// Main audio port layout summary for a hosted LV2 instance (mirrors
/// `ClapHostedPortLayout`, plus the atom/event input count the LV2 stereo
/// gate needs).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Lv2HostedPortLayout {
    /// Number of audio input ports.
    pub main_input_channels: u16,
    /// Number of audio output ports.
    pub main_output_channels: u16,
    /// Atom/event INPUT ports that are not `lv2:connectionOptional`.
    pub required_event_inputs: u16,
}

impl Lv2HostedPortLayout {
    /// Phase 1 supports exactly 2 audio in + 2 audio out AND zero required
    /// atom/event input ports (packet g11.033 decision 5: event-needing
    /// plugins — instruments mostly — are out; effects are in).
    pub fn is_stereo_effect(&self) -> bool {
        self.main_input_channels == 2
            && self.main_output_channels == 2
            && self.required_event_inputs == 0
    }
}

/// Lifecycle state of a hosted instance.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum HostedInstanceState {
    Created,
    Active,
}

/// One live LV2 plugin instance hosted in this process: owns the loaded
/// library, the matched descriptor, the TTL port model, the urid:map
/// feature set, and (while active) the plugin handle plus every buffer and
/// control slot its ports are connected to.
///
/// Threading: load/activate/deactivate/destroy run on the owning (main)
/// thread; audio processing runs through [`Lv2ProcessSession`], which the
/// sandbox moves onto its audio thread. While a process session is live
/// the owner must not run lifecycle transitions until the session stops.
pub struct Lv2HostedInstance {
    model: Lv2PluginModel,
    bundle_path: CString,
    descriptor: *const Lv2DescriptorRaw,
    /// Live plugin handle; null while not instantiated (LV2 instantiates
    /// at activate because the sample rate is fixed then).
    handle: *mut c_void,
    /// Boxed control-port value slots `(port_index, slot)` — stable
    /// addresses the plugin's control ports stay connected to. Inputs hold
    /// their TTL defaults; outputs give the plugin somewhere to write.
    control_slots: Vec<(u32, Box<f32>)>,
    /// Preallocated planar audio buffers, index-ordered per direction.
    /// Boxed slices: stable data addresses for `connect_port`.
    audio_inputs: Vec<Box<[f32]>>,
    audio_outputs: Vec<Box<[f32]>>,
    /// urid:map feature set the plugin may retain pointers into.
    urid: UridMapFeatureSet,
    parameters: Vec<PluginParameterDescriptor>,
    port_layout: Lv2HostedPortLayout,
    state: HostedInstanceState,
    activated_max_frames: u32,
    /// Pending param writes bound for the audio thread (g12.023): the
    /// process session drains them into the connected control slots at the
    /// top of each `run()` — the slots are only ever written from the
    /// audio thread once processing starts.
    param_changes: Arc<PluginParamChangeQueue>,
    /// Keeps the binary mapped for the instance lifetime; declared last so
    /// dlclose happens after deactivate/cleanup in `drop`.
    _library: Library,
}

impl Lv2HostedInstance {
    /// Load `plugin_uri` from the `.lv2` bundle at `bundle_root`: re-parse
    /// the bundle TTL (same-crate discovery functions), resolve the plugin
    /// model by URI, check the phase-1 feature allowlist, dlopen
    /// `lv2:binary`, and walk `lv2_descriptor(index)` until the URI
    /// matches. No plugin code beyond the descriptor walk runs at load —
    /// instantiation happens at [`Self::activate`].
    pub fn load(bundle_root: &Path, plugin_uri: &str) -> Result<Self, Lv2HostingError> {
        let bundle = parse_lv2_bundle(bundle_root)
            .map_err(|_| Lv2HostingError::new("bundle_parse_failed"))?;
        let model = bundle
            .plugins
            .into_iter()
            .find(|plugin| plugin.plugin_uri == plugin_uri)
            .ok_or_else(|| Lv2HostingError::new("plugin_uri_not_found"))?;

        if !model.unsupported_required_features().is_empty() {
            return Err(Lv2HostingError::new("unsupported_required_feature"));
        }

        let library = unsafe { Library::new(&model.binary_path) }
            .map_err(|_| Lv2HostingError::new("library_open_failed"))?;
        let lv2_descriptor = unsafe {
            library
                .get::<Lv2DescriptorProc>(b"lv2_descriptor\0")
                .map_err(|_| Lv2HostingError::new("lv2_descriptor_missing"))
                .map(|symbol| *symbol)?
        };
        let mut descriptor: *const Lv2DescriptorRaw = ptr::null();
        for index in 0..u32::MAX {
            let candidate = unsafe { lv2_descriptor(index) };
            if candidate.is_null() {
                break;
            }
            let uri = unsafe { (*candidate).uri };
            if !uri.is_null() && unsafe { CStr::from_ptr(uri) }.to_bytes() == plugin_uri.as_bytes()
            {
                descriptor = candidate;
                break;
            }
        }
        if descriptor.is_null() {
            return Err(Lv2HostingError::new("descriptor_not_found"));
        }

        let bundle_path = CString::new(bundle_root.to_string_lossy().to_string())
            .map_err(|_| Lv2HostingError::new("bundle_path_invalid"))?;
        let parameters = parameter_descriptors_from_model(&model);
        let port_layout = Lv2HostedPortLayout {
            main_input_channels: model.audio_inputs().len() as u16,
            main_output_channels: model.audio_outputs().len() as u16,
            required_event_inputs: model.required_event_inputs() as u16,
        };
        Ok(Self {
            model,
            bundle_path,
            descriptor,
            handle: ptr::null_mut(),
            control_slots: Vec::new(),
            audio_inputs: Vec::new(),
            audio_outputs: Vec::new(),
            urid: UridMapFeatureSet::new(),
            parameters,
            port_layout,
            state: HostedInstanceState::Created,
            activated_max_frames: 0,
            param_changes: Arc::new(PluginParamChangeQueue::new()),
            _library: library,
        })
    }

    /// Parameter inventory from the bundle TTL: control input ports double
    /// as parameters (`parameter_id` = port index).
    pub fn parameters(&self) -> &[PluginParameterDescriptor] {
        &self.parameters
    }

    /// Queue one parameter write (g12.023). LV2 control ports carry PLAIN
    /// values, so the host's normalized 0..1 value maps linearly onto the
    /// TTL min/max before queueing; the audio thread writes the connected
    /// control slot at the top of the next `run()` (block-boundary posture
    /// v1). Writes queued while inactive apply on the first processed
    /// block after activation.
    pub fn set_parameter_normalized(
        &self,
        parameter_id: u32,
        normalized: f32,
    ) -> Result<(), Lv2HostingError> {
        let descriptor = self
            .parameters
            .iter()
            .find(|parameter| parameter.parameter_id == parameter_id)
            .ok_or_else(|| Lv2HostingError::new("unknown_parameter"))?;
        let normalized = f64::from(normalized.clamp(0.0, 1.0));
        let plain = f64::from(descriptor.min_plain)
            + normalized * f64::from(descriptor.max_plain - descriptor.min_plain);
        if !self.param_changes.push(parameter_id, plain) {
            return Err(Lv2HostingError::new("param_queue_full"));
        }
        Ok(())
    }

    /// Audio port layout from the bundle TTL.
    pub fn port_layout(&self) -> Lv2HostedPortLayout {
        self.port_layout
    }

    /// Activate for processing: `instantiate` at `sample_rate_hz` (LV2
    /// fixes the rate here — that is why instantiation waits for activate),
    /// connect every control port to a boxed slot at its TTL default,
    /// preallocate planar audio buffers at `max_frames` and connect the
    /// audio ports, connect any remaining port to NULL, then run the
    /// plugin's `activate()`. No allocation happens after this point.
    pub fn activate(
        &mut self,
        sample_rate_hz: f64,
        _min_frames: u32,
        max_frames: u32,
    ) -> Result<(), Lv2HostingError> {
        if self.state == HostedInstanceState::Active {
            return Err(Lv2HostingError::new("already_active"));
        }
        let descriptor = unsafe { &*self.descriptor };
        let Some(instantiate) = descriptor.instantiate else {
            return Err(Lv2HostingError::new("instantiate_missing"));
        };
        let Some(connect_port) = descriptor.connect_port else {
            return Err(Lv2HostingError::new("connect_port_missing"));
        };
        if descriptor.run.is_none() {
            return Err(Lv2HostingError::new("run_missing"));
        }

        let handle = unsafe {
            instantiate(
                self.descriptor,
                sample_rate_hz,
                self.bundle_path.as_ptr(),
                self.urid.as_ptr(),
            )
        };
        if handle.is_null() {
            return Err(Lv2HostingError::new("instantiate_failed"));
        }

        // Preallocate the connection targets, then connect once. Boxed
        // storage keeps every address stable for the instance lifetime.
        self.control_slots = self
            .model
            .ports
            .iter()
            .filter(|port| port.classes.control)
            .map(|port| (port.index, Box::new(port.effective_default())))
            .collect();
        self.audio_inputs = self
            .model
            .audio_inputs()
            .iter()
            .map(|_| vec![0.0f32; max_frames as usize].into_boxed_slice())
            .collect();
        self.audio_outputs = self
            .model
            .audio_outputs()
            .iter()
            .map(|_| vec![0.0f32; max_frames as usize].into_boxed_slice())
            .collect();

        let mut next_input = 0usize;
        let mut next_output = 0usize;
        for port in &self.model.ports {
            let data: *mut c_void = if port.classes.audio && port.classes.input {
                let buffer = self.audio_inputs[next_input].as_mut_ptr();
                next_input += 1;
                buffer.cast()
            } else if port.classes.audio && port.classes.output {
                let buffer = self.audio_outputs[next_output].as_mut_ptr();
                next_output += 1;
                buffer.cast()
            } else if port.classes.control {
                let slot = self
                    .control_slots
                    .iter_mut()
                    .find(|(index, _)| *index == port.index)
                    .map(|(_, slot)| &mut **slot as *mut f32)
                    .expect("control slot exists for every control port");
                slot.cast()
            } else {
                // Remaining ports (atom/event) are only reachable here when
                // optional or output-side — the stereo gate rejects
                // required event inputs before activation. NULL is the LV2
                // connection for unconnected optional ports.
                ptr::null_mut()
            };
            // Safety: `handle` is the live instance from `instantiate`
            // above; buffers and slots outlive it (owned by self, freed
            // only after cleanup).
            unsafe { connect_port(handle, port.index, data) };
        }

        if let Some(activate) = descriptor.activate {
            // Safety: handle is live, all ports are connected.
            unsafe { activate(handle) };
        }
        self.handle = handle;
        self.state = HostedInstanceState::Active;
        self.activated_max_frames = max_frames;
        Ok(())
    }

    /// Deactivate an active instance: `deactivate()` then `cleanup()` (the
    /// LV2 pair to instantiate-at-activate — a later activate
    /// re-instantiates at the new rate). Tokened error when inactive.
    pub fn deactivate(&mut self) -> Result<(), Lv2HostingError> {
        if self.state != HostedInstanceState::Active {
            return Err(Lv2HostingError::new("not_active"));
        }
        self.teardown_instance();
        Ok(())
    }

    fn teardown_instance(&mut self) {
        let descriptor = unsafe { &*self.descriptor };
        if !self.handle.is_null() {
            unsafe {
                if let Some(deactivate) = descriptor.deactivate {
                    deactivate(self.handle);
                }
                if let Some(cleanup) = descriptor.cleanup {
                    cleanup(self.handle);
                }
            }
        }
        self.handle = ptr::null_mut();
        self.control_slots.clear();
        self.audio_inputs.clear();
        self.audio_outputs.clear();
        self.state = HostedInstanceState::Created;
        self.activated_max_frames = 0;
    }

    /// Build the raw process session for the sandbox audio thread. Only
    /// valid while active on a stereo-effect layout; the session reuses the
    /// planar buffers the audio ports were connected to at activate, so
    /// processing never allocates.
    pub fn process_session(&self) -> Result<Lv2ProcessSession, Lv2HostingError> {
        if self.state != HostedInstanceState::Active {
            return Err(Lv2HostingError::new("not_active"));
        }
        if self.audio_inputs.len() != 2 || self.audio_outputs.len() != 2 {
            return Err(Lv2HostingError::new("layout_unsupported"));
        }
        let descriptor = unsafe { &*self.descriptor };
        let run = descriptor
            .run
            .ok_or_else(|| Lv2HostingError::new("run_missing"))?;
        // Control INPUT slots the session may write param changes into
        // (g12.023). Output control slots stay plugin-owned.
        let control_inputs: Vec<(u32, *mut f32)> = self
            .model
            .ports
            .iter()
            .filter(|port| port.classes.control && port.classes.input)
            .filter_map(|port| {
                self.control_slots
                    .iter()
                    .find(|(index, _)| *index == port.index)
                    .map(|(index, slot)| (*index, (&**slot as *const f32) as *mut f32))
            })
            .collect();
        Ok(Lv2ProcessSession {
            handle: self.handle,
            run,
            input_left: self.audio_inputs[0].as_ptr() as *mut f32,
            input_right: self.audio_inputs[1].as_ptr() as *mut f32,
            output_left: self.audio_outputs[0].as_ptr() as *mut f32,
            output_right: self.audio_outputs[1].as_ptr() as *mut f32,
            max_frames: self.activated_max_frames as usize,
            processing: false,
            param_changes: Arc::clone(&self.param_changes),
            param_scratch: Vec::with_capacity(PLUGIN_PARAM_CHANGE_CAPACITY),
            control_inputs,
        })
    }
}

impl Drop for Lv2HostedInstance {
    fn drop(&mut self) {
        // deactivate → cleanup; `_library` (declared last) then dlcloses.
        self.teardown_instance();
    }
}

// ── Raw process session (audio thread) ──────────────────────────────────────

/// Raw, movable process handle for one activated LV2 instance: the plugin
/// handle, its `run` entry point, and raw pointers into the planar buffers
/// the audio ports were connected to at activate. The sandbox moves this
/// onto its audio thread; the owning [`Lv2HostedInstance`] must outlive it
/// and must not run lifecycle transitions while the session is live.
/// Ports stay connected from activation, so a block is exactly: copy in,
/// `run(n)`, copy out — alloc-free.
pub struct Lv2ProcessSession {
    handle: *mut c_void,
    run: unsafe extern "C" fn(*mut c_void, u32),
    input_left: *mut f32,
    input_right: *mut f32,
    output_left: *mut f32,
    output_right: *mut f32,
    max_frames: usize,
    processing: bool,
    /// Pending param writes shared with the owning instance (g12.023).
    param_changes: Arc<PluginParamChangeQueue>,
    /// Drain scratch (preallocated; audio thread never allocates).
    param_scratch: Vec<PluginParamChange>,
    /// `(port_index, slot)` for every control INPUT port; the slots live
    /// in the owning instance until after the session stops.
    control_inputs: Vec<(u32, *mut f32)>,
}

// Safety: the session is handed to exactly one audio thread; LV2's `run`
// is the audio-class function, and the owner serializes lifecycle against
// the session per the type contract above (the buffers behind the raw
// pointers live in the owning instance until after the session stops).
unsafe impl Send for Lv2ProcessSession {}

impl Lv2ProcessSession {
    /// Mark the session processing. LV2 has no start-processing handshake
    /// (push model); this exists for surface parity with the other
    /// formats' sessions and always succeeds.
    pub fn start(&mut self) -> Result<(), Lv2HostingError> {
        self.processing = true;
        Ok(())
    }

    /// Mark the session stopped (no LV2 call to make — push model).
    pub fn stop(&mut self) {
        self.processing = false;
    }

    /// Whether `start()` has run and `stop()` has not yet.
    pub fn is_processing(&self) -> bool {
        self.processing
    }

    /// Drain pending param writes into the connected control-input slots
    /// (g12.023, block-boundary application). Audio thread only —
    /// alloc-free; the slot pointers stay valid per the session contract.
    fn apply_param_changes(&mut self) {
        if self.param_changes.is_empty() {
            return;
        }
        self.param_changes.drain_coalesced(&mut self.param_scratch);
        for change in &self.param_scratch {
            if let Some((_, slot)) = self
                .control_inputs
                .iter()
                .find(|(index, _)| *index == change.parameter_id)
            {
                // Safety: control slots are instance-owned boxes that
                // outlive the session; only this thread writes them while
                // processing runs.
                unsafe { **slot = change.value as f32 };
            }
        }
    }

    /// Run one block through the connected planar buffers.
    ///
    /// # Safety
    /// `frames` must be within the activated max block size (callers
    /// clamp) and the owning instance must still be active.
    unsafe fn run_block(&mut self, frames: usize) {
        unsafe { (self.run)(self.handle, frames as u32) };
    }

    /// Process one block: interleaved stereo in, interleaved stereo out.
    /// Alloc-free (ports stay connected to the activate-time buffers).
    /// Returns `false` only when the handle is dead (input passes through
    /// unchanged) — LV2 `run` itself cannot report failure.
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
        if self.handle.is_null() {
            output[..frames * 2].copy_from_slice(&input[..frames * 2]);
            return false;
        }
        self.apply_param_changes();
        // Safety: pointers target the instance-owned boxed buffers sized
        // at max_frames; frames is clamped above.
        unsafe {
            for frame in 0..frames {
                *self.input_left.add(frame) = input[frame * 2];
                *self.input_right.add(frame) = input[frame * 2 + 1];
            }
            self.run_block(frames);
            for frame in 0..frames {
                output[frame * 2] = *self.output_left.add(frame);
                output[frame * 2 + 1] = *self.output_right.add(frame);
            }
        }
        true
    }

    /// In-place variant for the in-process isolation tier: processes the
    /// interleaved stereo buffer and writes the result back over it ONLY
    /// on success; on a dead handle the buffer is left untouched (bypass
    /// semantics). Alloc-free. `true` = buffer transformed.
    pub fn process_in_place(&mut self, io: &mut [f32], frame_count: usize) -> bool {
        let frames = frame_count.min(self.max_frames).min(io.len() / 2);
        if self.handle.is_null() {
            return false;
        }
        self.apply_param_changes();
        // Safety: as in `process_interleaved_stereo`.
        unsafe {
            for frame in 0..frames {
                *self.input_left.add(frame) = io[frame * 2];
                *self.input_right.add(frame) = io[frame * 2 + 1];
            }
            self.run_block(frames);
            for frame in 0..frames {
                io[frame * 2] = *self.output_left.add(frame);
                io[frame * 2 + 1] = *self.output_right.add(frame);
            }
        }
        true
    }
}
