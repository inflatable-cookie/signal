//! VST3 hosted instance body: load, lifecycle, GUI, and process session.

use std::ffi::c_void;
use std::path::Path;
use std::ptr;
use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use std::sync::Arc;

use signal_plugin::{PluginParamChangeQueue, PluginParameterDescriptor};

use crate::vst3_host_adapter::ara::AraInspectionSession;
use crate::vst3_host_adapter::gui::{Vst3GuiEvent, Vst3GuiSession};

use crate::vst3_host_adapter::hosting::Vst3HostingError;

use super::super::process::Vst3ProcessSession;
use super::super::wire::*;
use super::controller::{
    acquire_controller, controller_create_view, parameter_inventory,
    synchronize_controller_from_component, ControllerConnection, ControllerHandle,
};
use super::layout::{
    audio_bus_layout, bus_arrangements, pointer_or_null, HostedInstanceState, Vst3AudioBusLayout,
    Vst3HostedPortLayout,
};

/// One live VST3 plugin instance hosted in this process: owns the loaded
/// module, the `IComponent`/`IAudioProcessor` pair, and the (optional)
/// `IEditController`.
///
/// Threading: create/activate/deactivate/destroy run on the owning (main)
/// thread; audio processing runs through [`Vst3ProcessSession`], which the
/// sandbox moves onto its audio thread. While a process session is live the
/// owner must not run lifecycle transitions until the session stops.
pub struct Vst3HostedInstance {
    pub(crate) component: *mut c_void,
    pub(crate) processor: *mut c_void,
    pub(crate) controller: Option<ControllerHandle>,
    /// Bidirectional component/controller messaging for controllers exposed
    /// as a separate VST3 class. Dropped before either endpoint terminates.
    pub(crate) controller_connection: Option<ControllerConnection>,
    /// Stable host callback object installed on the edit controller.
    pub(crate) component_handler: Option<Box<ComponentHandler>>,
    /// Restart requests accepted by the component handler and serviced by
    /// the owning host control thread.
    pub(crate) pending_restart_flags: Arc<AtomicU32>,
    pub(crate) parameters: Vec<PluginParameterDescriptor>,
    pub(crate) port_layout: Vst3HostedPortLayout,
    pub(crate) audio_bus_layout: Vst3AudioBusLayout,
    pub(crate) state: HostedInstanceState,
    pub(crate) activated_sample_rate_hz: f64,
    pub(crate) activated_max_frames: u32,
    /// The live editor view, when open. Torn down (removed + released)
    /// BEFORE the controller in `Drop` — the mandated release ordering.
    pub(crate) gui_session: Option<Vst3GuiSession>,
    /// Pending param writes bound for the audio thread's
    /// `IParameterChanges` (g12.023); shared with every process session
    /// built from this instance.
    pub(crate) param_changes: Arc<PluginParamChangeQueue>,
    /// Bus 0 / channel 0 CC → parameter assignments from the controller's
    /// `IMidiMapping`, queried once at load. `None` = no mapping exposed
    /// (CC events are dropped for this plugin — the honest fallback).
    pub(crate) midi_cc_params: Option<Arc<[Option<u32>; VST3_MIDI_CONTROLLER_COUNT]>>,
    /// Empty ARA document used only by the isolated inspector. Ordinary
    /// processing loads leave this unset.
    ara_inspection: Option<AraInspectionSession>,
    /// Keeps the module mapped for the instance lifetime; declared last so
    /// it drops after the COM pointers above are released in `drop`.
    pub(crate) _module: LoadedVst3Module,
}

impl Vst3HostedInstance {
    /// Load the module inside `bundle_root`, create the component class
    /// identified by `class_id_hex` (the catalog load key), initialize it
    /// against the minimal host context, and enumerate its parameter
    /// inventory and main-bus port layout.
    ///
    /// The edit controller is acquired by `queryInterface` on the component
    /// (single-component plugins) or, failing that, by
    /// `IComponent::getControllerClassId` + a second factory
    /// `createInstance`. If neither works the inventory degrades to empty.
    pub fn load(bundle_root: &Path, class_id_hex: &str) -> Result<Self, Vst3HostingError> {
        Self::load_internal(bundle_root, class_id_hex, false)
    }

    /// Load a component for isolated UI inspection, binding an empty ARA
    /// document before activation when the component exposes ARA entry
    /// points. This is not full ARA host support.
    pub fn load_for_inspection(
        bundle_root: &Path,
        class_id_hex: &str,
    ) -> Result<Self, Vst3HostingError> {
        Self::load_internal(bundle_root, class_id_hex, true)
    }

    pub(crate) fn load_internal(
        bundle_root: &Path,
        class_id_hex: &str,
        enable_ara_inspection: bool,
    ) -> Result<Self, Vst3HostingError> {
        let cid = tuid_from_class_id_hex(class_id_hex)
            .ok_or_else(|| Vst3HostingError::new("class_id_invalid"))?;
        let module = LoadedVst3Module::load(bundle_root)?;

        let component = unsafe { module.create_instance(&cid, &ICOMPONENT_IID) }
            .or_else(|| {
                if !crate::vst3_host_adapter::introspection::moduleinfo_declares_component_class(
                    bundle_root,
                    class_id_hex,
                ) {
                    return None;
                }
                let factory_cid = unsafe { module.unique_component_class_id() }?;
                (factory_cid != cid)
                    .then(|| unsafe { module.create_instance(&factory_cid, &ICOMPONENT_IID) })
                    .flatten()
            })
            .ok_or_else(|| Vst3HostingError::new("create_component_failed"))?;
        let host = host_context();
        unsafe {
            let vtable = vtable_of::<ComponentVTable>(component);
            if ((*vtable).initialize)(component, host) != K_RESULT_OK {
                com_release(component);
                return Err(Vst3HostingError::new("component_initialize_failed"));
            }
        }

        // IAudioProcessor: usually the same object, sometimes separate.
        let Some(processor) = (unsafe { com_query_interface(component, &IAUDIO_PROCESSOR_IID) })
        else {
            unsafe {
                let vtable = vtable_of::<ComponentVTable>(component);
                ((*vtable).terminate)(component);
                com_release(component);
            }
            return Err(Vst3HostingError::new("audio_processor_missing"));
        };

        let ara_inspection = if enable_ara_inspection {
            match unsafe { AraInspectionSession::try_bind(component) } {
                Ok(session) => session,
                Err(error) => {
                    unsafe {
                        com_release(processor);
                        let vtable = vtable_of::<ComponentVTable>(component);
                        ((*vtable).terminate)(component);
                        com_release(component);
                    }
                    return Err(error);
                }
            }
        } else {
            None
        };

        let controller = unsafe { acquire_controller(component, &module, host) };
        let pending_restart_flags = Arc::new(AtomicU32::new(0));
        let component_handler = controller.as_ref().and_then(|controller| unsafe {
            let mut handler = Box::new(ComponentHandler {
                vtable: &COMPONENT_HANDLER_VTABLE,
                latency_changes: AtomicU64::new(0),
                pending_restart_flags: Arc::clone(&pending_restart_flags),
            });
            let vtable = vtable_of::<EditControllerVTable>(controller.ptr());
            let ptr = (&mut *handler as *mut ComponentHandler).cast();
            (((*vtable).set_component_handler)(controller.ptr(), ptr) == K_RESULT_OK)
                .then_some(handler)
        });
        let controller_connection = match controller.as_ref() {
            Some(ControllerHandle::Separate(controller)) => unsafe {
                ControllerConnection::establish(component, *controller)
            },
            _ => None,
        };
        if let Some(controller) = &controller {
            unsafe { synchronize_controller_from_component(component, controller.ptr()) };
        }
        let parameters = controller
            .as_ref()
            .map(|handle| unsafe { parameter_inventory(handle.ptr()) })
            .unwrap_or_default();
        let audio_bus_layout = unsafe { audio_bus_layout(component) };
        let port_layout = audio_bus_layout.port_layout();
        let midi_cc_params = controller
            .as_ref()
            .and_then(|handle| unsafe { midi_cc_parameter_map(handle.ptr()) });

        Ok(Self {
            component,
            processor,
            controller,
            controller_connection,
            component_handler,
            pending_restart_flags,
            parameters,
            port_layout,
            audio_bus_layout,
            state: HostedInstanceState::Created,
            activated_sample_rate_hz: 0.0,
            activated_max_frames: 0,
            gui_session: None,
            param_changes: Arc::new(PluginParamChangeQueue::new()),
            midi_cc_params,
            ara_inspection,
            _module: module,
        })
    }

    /// Whether the controller exposed an `IMidiMapping` at load: with one,
    /// CC events deliver as mapped parameter changes; without one they are
    /// dropped (VST3 has no input CC event type).
    pub fn midi_cc_mapping_available(&self) -> bool {
        self.midi_cc_params.is_some()
    }

    /// Whether `IMidiMapping` assigns this ordinary or extended controller
    /// number to a processor parameter.
    pub fn midi_controller_mapping_available(&self, controller: u16) -> bool {
        self.midi_cc_params
            .as_ref()
            .and_then(|map| map.get(usize::from(controller)))
            .copied()
            .flatten()
            .is_some()
    }

    /// Parameter inventory enumerated at load via `IEditController`
    /// (empty when no controller could be acquired).
    pub fn parameters(&self) -> &[PluginParameterDescriptor] {
        &self.parameters
    }

    /// Capture component and optional controller state into a small
    /// host-owned envelope. The payload remains opaque to Signal.
    pub fn save_state(&self) -> Result<Vec<u8>, Vst3HostingError> {
        unsafe {
            let component_vtable = vtable_of::<ComponentVTable>(self.component);
            let mut component = MemoryStream::writer();
            if ((*component_vtable).get_state)(self.component, component.as_raw()) != K_RESULT_OK {
                return Err(Vst3HostingError::new("state_capture_failed"));
            }

            let mut controller_bytes = Vec::new();
            if let Some(controller) = &self.controller {
                let controller_vtable = vtable_of::<EditControllerVTable>(controller.ptr());
                let mut controller_stream = MemoryStream::writer();
                if ((*controller_vtable).get_state)(controller.ptr(), controller_stream.as_raw())
                    == K_RESULT_OK
                {
                    controller_bytes = controller_stream.bytes;
                }
            }
            Ok(encode_state_envelope(&component.bytes, &controller_bytes))
        }
    }

    /// Restore component and optional controller state captured by
    /// [`Self::save_state`].
    pub fn load_state(&mut self, bytes: &[u8]) -> Result<(), Vst3HostingError> {
        let (component_bytes, controller_bytes) = decode_state_envelope(bytes)
            .ok_or_else(|| Vst3HostingError::new("state_deserialize_failed"))?;
        unsafe {
            let component_vtable = vtable_of::<ComponentVTable>(self.component);
            let mut component_stream = MemoryStream::reader(component_bytes);
            if ((*component_vtable).set_state)(self.component, component_stream.as_raw())
                != K_RESULT_OK
            {
                return Err(Vst3HostingError::new("state_restore_failed"));
            }

            if let Some(controller) = &self.controller {
                let controller_vtable = vtable_of::<EditControllerVTable>(controller.ptr());
                let mut component_for_controller = MemoryStream::reader(component_bytes);
                let _ = ((*controller_vtable).set_component_state)(
                    controller.ptr(),
                    component_for_controller.as_raw(),
                );
                if !controller_bytes.is_empty() {
                    let mut controller_stream = MemoryStream::reader(controller_bytes);
                    if ((*controller_vtable).set_state)(
                        controller.ptr(),
                        controller_stream.as_raw(),
                    ) != K_RESULT_OK
                    {
                        return Err(Vst3HostingError::new("controller_state_restore_failed"));
                    }
                }
            }
        }
        Ok(())
    }

    /// Queue one parameter write (g12.023). VST3's set domain is the
    /// normalized 0..1 value itself: it lands in the processor through the
    /// next block's `IParameterChanges` (block-boundary posture v1), and
    /// `IEditController::setParamNormalized` runs here so the controller's
    /// state (GUIs, `getParamNormalized`) stays in sync — the documented
    /// host duty for host-driven changes.
    pub fn set_parameter_normalized(
        &self,
        parameter_id: u32,
        normalized: f32,
    ) -> Result<(), Vst3HostingError> {
        if !self
            .parameters
            .iter()
            .any(|parameter| parameter.parameter_id == parameter_id)
        {
            return Err(Vst3HostingError::new("unknown_parameter"));
        }
        let normalized = f64::from(normalized.clamp(0.0, 1.0));
        if let Some(controller) = &self.controller {
            unsafe {
                let vtable = vtable_of::<EditControllerVTable>(controller.ptr());
                let _ =
                    ((*vtable).set_param_normalized)(controller.ptr(), parameter_id, normalized);
            }
        }
        if !self.param_changes.push(parameter_id, normalized) {
            return Err(Vst3HostingError::new("param_queue_full"));
        }
        Ok(())
    }

    /// Current main-bus port layout, including successful activation-time
    /// negotiation.
    pub fn port_layout(&self) -> Vst3HostedPortLayout {
        self.port_layout
    }

    /// Current processor-reported latency in sample frames.
    pub fn latency_frames(&self) -> u32 {
        unsafe {
            let vtable = vtable_of::<AudioProcessorVTable>(self.processor);
            ((*vtable).get_latency_samples)(self.processor)
        }
    }

    /// Number of controller `kLatencyChanged` restart notifications.
    pub fn latency_change_count(&self) -> u64 {
        self.component_handler
            .as_ref()
            .map(|handler| handler.latency_changes.load(Ordering::Relaxed))
            .unwrap_or(0)
    }

    /// Shared restart flags accepted from `IComponentHandler`. Audio hosts
    /// use this to stop at a block boundary before the control thread
    /// services the requested lifecycle transition.
    pub fn pending_restart_flags(&self) -> Arc<AtomicU32> {
        Arc::clone(&self.pending_restart_flags)
    }

    /// Deactivate, refresh dynamic I/O when requested, reactivate, and build
    /// a replacement process session on the owning control thread.
    pub fn restart_processing(
        &mut self,
        flags: u32,
    ) -> Result<Vst3ProcessSession, Vst3HostingError> {
        let sample_rate_hz = self.activated_sample_rate_hz;
        let max_frames = self.activated_max_frames;
        self.deactivate()?;
        if flags & VST3_RESTART_IO_CHANGED != 0 {
            self.audio_bus_layout = unsafe { audio_bus_layout(self.component) };
            self.port_layout = self.audio_bus_layout.port_layout();
        }
        self.activate(sample_rate_hz, 1, max_frames)?;
        self.process_session()
    }

    /// Activate for processing by negotiating the available main buses to a
    /// stereo effect (2-in/2-out) or instrument (0-in/2-out), then selecting
    /// 32-bit samples, calling `setupProcessing`, activating the main buses,
    /// and calling `setActive(true)`. Unsupported negotiation fails with the
    /// stable `layout_unsupported` token, same as the CLAP path. Components
    /// without any audio output fail with `no_audio_buses`; their editors may
    /// still be hosted without creating a process session.
    pub fn activate(
        &mut self,
        sample_rate_hz: f64,
        _min_frames: u32,
        max_frames: u32,
    ) -> Result<(), Vst3HostingError> {
        if self.state == HostedInstanceState::Active {
            return Err(Vst3HostingError::new("already_active"));
        }
        if self.audio_bus_layout.main_output.is_none() {
            return Err(Vst3HostingError::new("no_audio_buses"));
        }
        unsafe {
            let processor = vtable_of::<AudioProcessorVTable>(self.processor);
            let has_audio_input = self.audio_bus_layout.main_input.is_some();

            // VST3 requires the arrangement array to cover every declared bus,
            // including inactive auxiliaries. Preserve each auxiliary layout
            // and negotiate only the main bus to stereo.
            let mut input_arrangements = bus_arrangements(
                self.processor,
                K_INPUT,
                &self.audio_bus_layout.input_channels,
            );
            let mut output_arrangements = bus_arrangements(
                self.processor,
                K_OUTPUT,
                &self.audio_bus_layout.output_channels,
            );
            if let Some(index) = self.audio_bus_layout.main_input {
                input_arrangements[index] = STEREO_ARRANGEMENT;
            }
            if let Some(index) = self.audio_bus_layout.main_output {
                output_arrangements[index] = STEREO_ARRANGEMENT;
            }
            let _ = ((*processor).set_bus_arrangements)(
                self.processor,
                pointer_or_null(&mut input_arrangements),
                input_arrangements.len() as i32,
                pointer_or_null(&mut output_arrangements),
                output_arrangements.len() as i32,
            );
            let mut verified_input = 0u64;
            let mut verified_output = 0u64;
            let input_verified = !has_audio_input
                || (((*processor).get_bus_arrangement)(
                    self.processor,
                    K_INPUT,
                    self.audio_bus_layout.main_input.unwrap_or(0) as i32,
                    &mut verified_input,
                ) == K_RESULT_OK
                    && verified_input == STEREO_ARRANGEMENT);
            let output_result = ((*processor).get_bus_arrangement)(
                self.processor,
                K_OUTPUT,
                self.audio_bus_layout.main_output.unwrap_or(0) as i32,
                &mut verified_output,
            );
            if !input_verified
                || output_result != K_RESULT_OK
                || verified_output != STEREO_ARRANGEMENT
            {
                return Err(Vst3HostingError::new("layout_unsupported"));
            }
            if let Some(index) = self.audio_bus_layout.main_input {
                self.audio_bus_layout.input_channels[index] = 2;
            }
            if let Some(index) = self.audio_bus_layout.main_output {
                self.audio_bus_layout.output_channels[index] = 2;
            }
            self.port_layout = self.audio_bus_layout.port_layout();

            if ((*processor).can_process_sample_size)(self.processor, K_SAMPLE32) != K_RESULT_OK {
                return Err(Vst3HostingError::new("sample_size_unsupported"));
            }

            let mut setup = ProcessSetup {
                process_mode: K_REALTIME,
                symbolic_sample_size: K_SAMPLE32,
                max_samples_per_block: max_frames as i32,
                sample_rate: sample_rate_hz,
            };
            if ((*processor).setup_processing)(self.processor, &mut setup) != K_RESULT_OK {
                return Err(Vst3HostingError::new("setup_processing_failed"));
            }

            let component = vtable_of::<ComponentVTable>(self.component);
            if let Some(index) = self.audio_bus_layout.main_input {
                let _ =
                    ((*component).activate_bus)(self.component, K_AUDIO, K_INPUT, index as i32, 1);
            }
            if let Some(index) = self.audio_bus_layout.main_output {
                let _ =
                    ((*component).activate_bus)(self.component, K_AUDIO, K_OUTPUT, index as i32, 1);
            }
            if ((*component).set_active)(self.component, 1) != K_RESULT_OK {
                return Err(Vst3HostingError::new("set_active_failed"));
            }
        }
        self.state = HostedInstanceState::Active;
        self.activated_sample_rate_hz = sample_rate_hz;
        self.activated_max_frames = max_frames;
        Ok(())
    }

    /// Deactivate an active instance (no-op tokened error when inactive).
    pub fn deactivate(&mut self) -> Result<(), Vst3HostingError> {
        if self.state != HostedInstanceState::Active {
            return Err(Vst3HostingError::new("not_active"));
        }
        unsafe {
            let component = vtable_of::<ComponentVTable>(self.component);
            let _ = ((*component).set_active)(self.component, 0);
        }
        self.state = HostedInstanceState::Created;
        Ok(())
    }

    // ── IPlugView hosting (g12.024, GUI phase 2) ───────────────────────
    //
    // MAIN-THREAD CONTRACT: every gui_* method below maps to a VST3
    // UI-thread function. The embedding host must dispatch these onto the
    // application main thread (Tauri `run_on_main_thread`); this type only
    // serializes access, it cannot pick the thread.

    /// Whether the plugin exposes an edit controller that may provide an
    /// editor. `createView("editor")` is deliberately deferred until the
    /// real GUI open: some plugins do not tolerate probe-and-discard or
    /// require their processor to be active first.
    pub fn gui_supported(&self) -> bool {
        self.controller.is_some()
    }

    /// Whether an editor view is currently attached.
    pub fn gui_is_open(&self) -> bool {
        self.gui_session.is_some()
    }

    /// Open the embedded editor parented into `parent` (an `NSView*` on
    /// macOS): `createView("editor")` → platform check → `setFrame` →
    /// `getSize` → `attached`. Returns the plugin's initial content size
    /// (logical units). Errors with stable tokens (`gui_unsupported`,
    /// `gui_already_open`, `gui_attached_failed`, …).
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
        _scale: Option<f64>,
    ) -> Result<(u32, u32), Vst3HostingError> {
        if self.gui_session.is_some() {
            return Err(Vst3HostingError::new("gui_already_open"));
        }
        let controller = self
            .controller
            .as_ref()
            .ok_or_else(|| Vst3HostingError::new("gui_unsupported"))?;
        let view = unsafe { controller_create_view(controller.ptr()) };
        let session = unsafe { Vst3GuiSession::open_embedded(view, parent) }?;
        let size = session.size();
        self.gui_session = Some(session);
        Ok(size)
    }

    /// The open editor view, for size/resize interaction.
    pub fn gui_session_mut(&mut self) -> Option<&mut Vst3GuiSession> {
        self.gui_session.as_mut()
    }

    /// The open editor view, read-only.
    pub fn gui_session(&self) -> Option<&Vst3GuiSession> {
        self.gui_session.as_ref()
    }

    /// Destroy the open editor view (idempotent; `removed` + release — the
    /// plugin instance stays live).
    pub fn gui_destroy(&mut self) {
        self.gui_session = None;
    }

    /// Drain host-side view callbacks queued since the last call
    /// (`resizeView` requests). Empty when no editor is open.
    pub fn take_gui_events(&self) -> Vec<Vst3GuiEvent> {
        self.gui_session
            .as_ref()
            .map(|session| session.take_events())
            .unwrap_or_default()
    }

    /// Build the raw process session for the sandbox audio thread. Only
    /// valid while active; the session preallocates its planar buffers at
    /// the activated max block size, so processing never allocates.
    pub fn process_session(&self) -> Result<Vst3ProcessSession, Vst3HostingError> {
        if self.state != HostedInstanceState::Active {
            return Err(Vst3HostingError::new("not_active"));
        }
        Ok(Vst3ProcessSession::new(
            self.processor,
            self.activated_sample_rate_hz,
            self.activated_max_frames as usize,
            self.audio_bus_layout.clone(),
            Arc::clone(&self.param_changes),
            self.midi_cc_params.clone(),
        ))
    }
}

impl Drop for Vst3HostedInstance {
    fn drop(&mut self) {
        // View teardown (removed + release) must precede controller
        // teardown. This is the fallback path (teardown with an editor
        // still open); the orderly path closes the editor on the main
        // thread first.
        self.gui_session = None;
        self.controller_connection = None;
        unsafe {
            if self.state == HostedInstanceState::Active {
                let component = vtable_of::<ComponentVTable>(self.component);
                let _ = ((*component).set_active)(self.component, 0);
            }
            com_release(self.processor);
            if let Some(controller) = self.controller.take() {
                let vtable = vtable_of::<EditControllerVTable>(controller.ptr());
                let _ = ((*vtable).set_component_handler)(controller.ptr(), ptr::null_mut());
                match controller {
                    ControllerHandle::ComponentFacet(ptr) => com_release(ptr),
                    ControllerHandle::Separate(ptr) => {
                        let vtable = vtable_of::<EditControllerVTable>(ptr);
                        let _ = ((*vtable).terminate)(ptr);
                        com_release(ptr);
                    }
                }
            }
            let component = vtable_of::<ComponentVTable>(self.component);
            let _ = ((*component).terminate)(self.component);
            com_release(self.component);
        }
        // Drop the ARA document after the bound component, but before the
        // module unloads. The ARA contract permits either component/document
        // destruction order.
        self.ara_inspection = None;
        // `_module` drops after this body: exit proc, then dlclose.
    }
}
