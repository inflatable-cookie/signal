use std::path::Path;
use std::sync::atomic::{AtomicU32, AtomicU64};
use std::sync::Arc;

use signal_plugin::{PluginParamChangeQueue, PluginParameterDescriptor};

use crate::vst3_host_adapter::ara::AraInspectionSession;

use crate::vst3_host_adapter::hosting::Vst3HostingError;

use super::super::wire::*;
use super::controller::{
    acquire_controller, parameter_inventory, synchronize_controller_from_component,
    ControllerConnection, ControllerHandle,
};
use super::hosted::Vst3HostedInstance;
use super::layout::{audio_bus_layout, HostedInstanceState};

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
}
