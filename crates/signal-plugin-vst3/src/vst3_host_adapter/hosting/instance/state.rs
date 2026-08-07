use crate::vst3_host_adapter::hosting::Vst3HostingError;

use super::super::wire::*;
use super::hosted::Vst3HostedInstance;

impl Vst3HostedInstance {
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
}
