//! VST3 hosted instance body: struct definition and teardown ordering.

use std::ffi::c_void;
use std::ptr;
use std::sync::atomic::AtomicU32;
use std::sync::Arc;

use signal_plugin::{PluginParamChangeQueue, PluginParameterDescriptor};

use crate::vst3_host_adapter::ara::AraInspectionSession;
use crate::vst3_host_adapter::gui::Vst3GuiSession;

use super::super::wire::*;
use super::controller::{ControllerConnection, ControllerHandle};
use super::layout::{HostedInstanceState, Vst3AudioBusLayout, Vst3HostedPortLayout};

/// One live VST3 plugin instance hosted in this process: owns the loaded
/// module, the `IComponent`/`IAudioProcessor` pair, and the (optional)
/// `IEditController`.
///
/// Threading: create/activate/deactivate/destroy run on the owning (main)
/// thread; audio processing runs through `Vst3ProcessSession`, which the
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
    pub(super) ara_inspection: Option<AraInspectionSession>,
    /// Keeps the module mapped for the instance lifetime; declared last so
    /// it drops after the COM pointers above are released in `drop`.
    pub(crate) _module: LoadedVst3Module,
}

impl std::fmt::Debug for Vst3HostedInstance {
    /// Reports COM identity and lifecycle state. The controller handles,
    /// component handler, ARA session, and loaded module are VST3 ABI objects
    /// with a mandated release ordering and are not formatted.
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("Vst3HostedInstance")
            .field("component", &self.component)
            .field("processor", &self.processor)
            .field("has_controller", &self.controller.is_some())
            .field("state", &self.state)
            .field("parameters", &self.parameters.len())
            .field("activated_sample_rate_hz", &self.activated_sample_rate_hz)
            .field("activated_max_frames", &self.activated_max_frames)
            .field("gui_open", &self.gui_session.is_some())
            .field("midi_cc_mapping_available", &self.midi_cc_params.is_some())
            .finish_non_exhaustive()
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
