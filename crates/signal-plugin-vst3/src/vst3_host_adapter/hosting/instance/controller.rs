//! VST3 edit-controller acquisition, connection, and parameter inventory.

use std::ffi::c_void;
use std::ptr;

use signal_plugin::{PluginParameterDescriptor, PluginParameterDomain, PluginParameterFlags};

use crate::vst3_host_adapter::gui::{IPLUG_VIEW_IID, VIEW_TYPE_EDITOR};

use super::super::wire::*;

/// How the edit controller was obtained (drop/teardown differs).
pub(crate) enum ControllerHandle {
    /// `queryInterface` facet of the component object itself: release only.
    ComponentFacet(*mut c_void),
    /// Separate class created through the factory: terminate then release.
    Separate(*mut c_void),
}

impl ControllerHandle {
    pub(crate) fn ptr(&self) -> *mut c_void {
        match self {
            Self::ComponentFacet(ptr) | Self::Separate(ptr) => *ptr,
        }
    }
}

/// Connected `IConnectionPoint` facets for a separate component/controller
/// pair. The connection is established in both directions and must be torn
/// down before either plugin object is terminated.
pub(crate) struct ControllerConnection {
    pub(crate) component: *mut c_void,
    pub(crate) controller: *mut c_void,
}

#[repr(C)]
pub(crate) struct ConnectionPointVTable {
    pub(crate) query_interface:
        unsafe extern "C" fn(*mut c_void, *const Tuid, *mut *mut c_void) -> Tresult,
    pub(crate) add_ref: unsafe extern "C" fn(*mut c_void) -> u32,
    pub(crate) release: unsafe extern "C" fn(*mut c_void) -> u32,
    pub(crate) connect: unsafe extern "C" fn(*mut c_void, *mut c_void) -> Tresult,
    pub(crate) disconnect: unsafe extern "C" fn(*mut c_void, *mut c_void) -> Tresult,
    pub(crate) notify: unsafe extern "C" fn(*mut c_void, *mut c_void) -> Tresult,
}

impl ControllerConnection {
    pub(crate) unsafe fn establish(
        component: *mut c_void,
        controller: *mut c_void,
    ) -> Option<Self> {
        let component_point = com_query_interface(component, &ICONNECTION_POINT_IID)?;
        let Some(controller_point) = com_query_interface(controller, &ICONNECTION_POINT_IID) else {
            com_release(component_point);
            return None;
        };
        let component_vtable = vtable_of::<ConnectionPointVTable>(component_point);
        if ((*component_vtable).connect)(component_point, controller_point) != K_RESULT_OK {
            com_release(controller_point);
            com_release(component_point);
            return None;
        }
        let controller_vtable = vtable_of::<ConnectionPointVTable>(controller_point);
        if ((*controller_vtable).connect)(controller_point, component_point) != K_RESULT_OK {
            let _ = ((*component_vtable).disconnect)(component_point, controller_point);
            com_release(controller_point);
            com_release(component_point);
            return None;
        }
        Some(Self {
            component: component_point,
            controller: controller_point,
        })
    }
}

impl Drop for ControllerConnection {
    fn drop(&mut self) {
        unsafe {
            let component_vtable = vtable_of::<ConnectionPointVTable>(self.component);
            let controller_vtable = vtable_of::<ConnectionPointVTable>(self.controller);
            let _ = ((*controller_vtable).disconnect)(self.controller, self.component);
            let _ = ((*component_vtable).disconnect)(self.component, self.controller);
            com_release(self.controller);
            com_release(self.component);
        }
    }
}

/// Give the edit controller the component's initial state before querying
/// parameters or creating its editor. Separate-controller plugins commonly
/// build their UI from this state during `setComponentState`.
pub(crate) unsafe fn synchronize_controller_from_component(
    component: *mut c_void,
    controller: *mut c_void,
) {
    let component_vtable = vtable_of::<ComponentVTable>(component);
    let mut state = MemoryStream::writer();
    if ((*component_vtable).get_state)(component, state.as_raw()) != K_RESULT_OK {
        return;
    }
    state.position = 0;
    let controller_vtable = vtable_of::<EditControllerVTable>(controller);
    let _ = ((*controller_vtable).set_component_state)(controller, state.as_raw());
}

/// Acquire the edit controller: component facet first, else the separate
/// controller class through the factory. `None` = no parameter inventory.
pub(crate) unsafe fn acquire_controller(
    component: *mut c_void,
    module: &LoadedVst3Module,
    host: *mut c_void,
) -> Option<ControllerHandle> {
    if let Some(facet) = com_query_interface(component, &IEDIT_CONTROLLER_IID) {
        return Some(ControllerHandle::ComponentFacet(facet));
    }
    let component_vtable = vtable_of::<ComponentVTable>(component);
    let mut controller_cid: Tuid = [0; 16];
    if ((*component_vtable).get_controller_class_id)(component, &mut controller_cid) != K_RESULT_OK
        || controller_cid == [0; 16]
    {
        return None;
    }
    let controller = module.create_instance(&controller_cid, &IEDIT_CONTROLLER_IID)?;
    let controller_vtable = vtable_of::<EditControllerVTable>(controller);
    if ((*controller_vtable).initialize)(controller, host) != K_RESULT_OK {
        com_release(controller);
        return None;
    }
    Some(ControllerHandle::Separate(controller))
}

/// `IEditController::createView(ViewType::kEditor)`: the plugin's editor
/// view, owned by the caller (null when the plugin has no editor).
pub(crate) unsafe fn controller_create_view(controller: *mut c_void) -> *mut c_void {
    let vtable = vtable_of::<EditControllerVTable>(controller);
    let view = ((*vtable).create_view)(controller, VIEW_TYPE_EDITOR.as_ptr());
    // Some plugins return a view that fails the IPlugView identity check;
    // trust queryInterface over the raw pointer.
    if view.is_null() {
        return ptr::null_mut();
    }
    match com_query_interface(view, &IPLUG_VIEW_IID) {
        Some(typed) => {
            // createView's reference plus queryInterface's addRef: drop one.
            com_release(view);
            typed
        }
        None => {
            com_release(view);
            ptr::null_mut()
        }
    }
}

/// Enumerate the controller's parameter inventory into Signal descriptors.
pub(crate) unsafe fn parameter_inventory(
    controller: *mut c_void,
) -> Vec<PluginParameterDescriptor> {
    let vtable = vtable_of::<EditControllerVTable>(controller);
    let count = ((*vtable).get_parameter_count)(controller).max(0);
    let mut parameters = Vec::with_capacity(count as usize);
    for index in 0..count {
        let mut info = ParameterInfo::zeroed();
        if ((*vtable).get_parameter_info)(controller, index, &mut info) != K_RESULT_OK {
            continue;
        }
        let min_plain = ((*vtable).normalized_param_to_plain)(controller, info.id, 0.0) as f32;
        let max_plain = ((*vtable).normalized_param_to_plain)(controller, info.id, 1.0) as f32;
        let is_bypass = info.flags & PARAM_IS_BYPASS != 0;
        let unit = utf16_field_to_string(&info.units);
        parameters.push(PluginParameterDescriptor {
            parameter_id: info.id,
            name: utf16_field_to_string(&info.title).unwrap_or_else(|| format!("Param {index}")),
            unit,
            domain: if is_bypass {
                PluginParameterDomain::Bypass
            } else {
                PluginParameterDomain::GenericNormalized
            },
            default_normalized: info.default_normalized_value as f32,
            min_plain: min_plain.min(max_plain),
            max_plain: max_plain.max(min_plain),
            // VST3 reports the step count directly: 0 = continuous,
            // n = n discrete steps (n + 1 values, 1 = toggle).
            step_count: (info.step_count > 0).then_some(info.step_count as u32),
            flags: PluginParameterFlags {
                automatable: info.flags & PARAM_CAN_AUTOMATE != 0,
                modulatable: false,
                supports_gesture: false,
                stepped: info.step_count > 0,
                hidden: info.flags & PARAM_IS_HIDDEN != 0,
                read_only: info.flags & PARAM_IS_READ_ONLY != 0,
            },
        });
    }
    parameters
}

pub(crate) fn utf16_field_to_string(field: &[i16]) -> Option<String> {
    let units: Vec<u16> = field
        .iter()
        .copied()
        .take_while(|unit| *unit != 0)
        .map(|unit| unit as u16)
        .collect();
    if units.is_empty() {
        return None;
    }
    let text = String::from_utf16_lossy(&units).trim().to_string();
    (!text.is_empty()).then_some(text)
}
