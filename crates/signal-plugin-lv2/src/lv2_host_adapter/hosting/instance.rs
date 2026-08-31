//! Hosted LV2 plugin instance lifecycle.

use std::ffi::{c_void, CStr, CString};
use std::path::Path;
use std::ptr;
use std::sync::Arc;

use libloading::Library;
use signal_plugin::{
    PluginParamChangeQueue, PluginParameterDescriptor, PLUGIN_PARAM_CHANGE_CAPACITY,
};

use super::super::introspection::{
    parameter_descriptors_from_model, parse_lv2_bundle, Lv2PluginModel,
};

use super::process::Lv2ProcessSession;
use super::support::*;

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
pub(crate) enum HostedInstanceState {
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

impl std::fmt::Debug for Lv2HostedInstance {
    /// Reports bundle identity, lifecycle state, and port shape. The
    /// descriptor, instance handle, URID feature set, and loaded library are
    /// LV2 ABI objects and are not formatted.
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("Lv2HostedInstance")
            .field("bundle_path", &self.bundle_path)
            .field("handle", &self.handle)
            .field("state", &self.state)
            .field("parameters", &self.parameters.len())
            .field("control_ports", &self.control_slots.len())
            .field("audio_inputs", &self.audio_inputs.len())
            .field("audio_outputs", &self.audio_outputs.len())
            .field("activated_max_frames", &self.activated_max_frames)
            .finish_non_exhaustive()
    }
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
    ///
    /// # Panics
    ///
    /// Panics if a control port has no preallocated slot, or if the port
    /// layout names more audio ports than were preallocated. Both are built
    /// from the same parsed TTL model immediately above, so a panic here
    /// means the model and the preallocation disagree — not that the bundle
    /// was malformed, which is reported as a typed error instead.
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
