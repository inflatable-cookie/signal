use crate::{
    RuntimeAdvancedControlFeedbackAuthority, RuntimeAdvancedControlFeedbackOutcome,
    RuntimeAdvancedHardwareDeviceDescriptor, RuntimeControlSurfaceDeviceDescriptor,
    RuntimeControlSurfaceFeedbackReadiness, RuntimeControlSurfaceMappingPosture,
    RuntimeControlSurfaceWorkflowAuthority, RuntimeDisplayContentClass,
    RuntimeDisplayTransportPosture, RuntimeFeedbackPageClass, RuntimeFeedbackPagePosture,
    RuntimeGuardedFeedbackChannelPosture, RuntimeHapticTransportPosture,
    RuntimeMotorTransportPosture, RuntimeSafeActionGraphPosture, RuntimeSafeActionOutcome,
    RuntimeSceneMappingPosture, RuntimeScriptingSafeDevicePolicyPosture,
};

#[path = "device_descriptor/capability.rs"]
mod capability;

use capability::build_capability;

#[derive(Clone, Debug, Default)]
pub(super) struct DeviceProjectionCounts {
    pub portable_device_count: usize,
    pub guarded_device_count: usize,
    pub context_only_device_count: usize,
    pub denied_device_count: usize,
    pub feedback_channel_device_count: usize,
    pub display_transport_device_count: usize,
    pub motor_transport_device_count: usize,
    pub haptic_transport_device_count: usize,
    pub scene_mapping_device_count: usize,
    pub feedback_page_device_count: usize,
    pub safe_action_graph_device_count: usize,
}

impl DeviceProjectionCounts {
    pub(super) fn accumulate(&mut self, other: &Self) {
        self.portable_device_count += other.portable_device_count;
        self.guarded_device_count += other.guarded_device_count;
        self.context_only_device_count += other.context_only_device_count;
        self.denied_device_count += other.denied_device_count;
        self.feedback_channel_device_count += other.feedback_channel_device_count;
        self.display_transport_device_count += other.display_transport_device_count;
        self.motor_transport_device_count += other.motor_transport_device_count;
        self.haptic_transport_device_count += other.haptic_transport_device_count;
        self.scene_mapping_device_count += other.scene_mapping_device_count;
        self.feedback_page_device_count += other.feedback_page_device_count;
        self.safe_action_graph_device_count += other.safe_action_graph_device_count;
    }
}

pub(super) struct DeviceProjection {
    pub descriptor: RuntimeAdvancedHardwareDeviceDescriptor,
    pub counts: DeviceProjectionCounts,
}

pub(super) fn project_device(device: &RuntimeControlSurfaceDeviceDescriptor) -> DeviceProjection {
    let scripting_safe_posture = match device.mapping_posture {
        RuntimeControlSurfaceMappingPosture::ObserveOnly => {
            RuntimeScriptingSafeDevicePolicyPosture::ContextOnly
        }
        RuntimeControlSurfaceMappingPosture::Unsupported => {
            RuntimeScriptingSafeDevicePolicyPosture::Denied
        }
        RuntimeControlSurfaceMappingPosture::Guarded => {
            RuntimeScriptingSafeDevicePolicyPosture::Guarded
        }
        RuntimeControlSurfaceMappingPosture::Portable => {
            if matches!(
                device.feedback_readiness,
                RuntimeControlSurfaceFeedbackReadiness::Ready
            ) {
                RuntimeScriptingSafeDevicePolicyPosture::Portable
            } else {
                RuntimeScriptingSafeDevicePolicyPosture::Guarded
            }
        }
    };
    let feedback_channel_posture = if !device.capability.supports_feedback_output {
        RuntimeGuardedFeedbackChannelPosture::Unavailable
    } else if matches!(
        device.feedback_readiness,
        RuntimeControlSurfaceFeedbackReadiness::Ready
    ) && matches!(
        scripting_safe_posture,
        RuntimeScriptingSafeDevicePolicyPosture::Portable
    ) {
        RuntimeGuardedFeedbackChannelPosture::Portable
    } else {
        RuntimeGuardedFeedbackChannelPosture::Guarded
    };

    let display_transport_posture = if !device.capability.supports_feedback_output {
        RuntimeDisplayTransportPosture::NotPresent
    } else if matches!(
        feedback_channel_posture,
        RuntimeGuardedFeedbackChannelPosture::Portable
    ) {
        RuntimeDisplayTransportPosture::TextOnlyDisplay
    } else {
        RuntimeDisplayTransportPosture::GuardedDisplay
    };
    let display_content_class = match display_transport_posture {
        RuntimeDisplayTransportPosture::NotPresent
        | RuntimeDisplayTransportPosture::UnavailableDisplay => {
            RuntimeDisplayContentClass::NoDisplayContent
        }
        RuntimeDisplayTransportPosture::GuardedDisplay => {
            RuntimeDisplayContentClass::GuardedVendorDisplay
        }
        RuntimeDisplayTransportPosture::TextOnlyDisplay => RuntimeDisplayContentClass::StatusText,
        RuntimeDisplayTransportPosture::PageAwareDisplay => {
            RuntimeDisplayContentClass::PagedStatusView
        }
    };
    let motor_transport_posture = RuntimeMotorTransportPosture::NoMotorTransport;
    let haptic_transport_posture = RuntimeHapticTransportPosture::NoHapticTransport;
    let feedback_authority = RuntimeAdvancedControlFeedbackAuthority::RuntimeDefault;
    let feedback_outcome = if !device.capability.supports_feedback_output {
        RuntimeAdvancedControlFeedbackOutcome::BypassFeedbackTransport
    } else if matches!(
        feedback_channel_posture,
        RuntimeGuardedFeedbackChannelPosture::Portable
    ) {
        RuntimeAdvancedControlFeedbackOutcome::PreserveDeclaredFeedback
    } else {
        RuntimeAdvancedControlFeedbackOutcome::CollapseToGuardedFeedback
    };
    let scene_mapping_posture = match device.mapping_posture {
        RuntimeControlSurfaceMappingPosture::Unsupported => {
            RuntimeSceneMappingPosture::NoSceneMapping
        }
        RuntimeControlSurfaceMappingPosture::ObserveOnly => {
            RuntimeSceneMappingPosture::ContextualSceneMapping
        }
        RuntimeControlSurfaceMappingPosture::Guarded => {
            RuntimeSceneMappingPosture::GuardedSceneMapping
        }
        RuntimeControlSurfaceMappingPosture::Portable => {
            RuntimeSceneMappingPosture::PortableSceneMapping
        }
    };
    let feedback_page_posture = match display_transport_posture {
        RuntimeDisplayTransportPosture::NotPresent => RuntimeFeedbackPagePosture::NoFeedbackPages,
        RuntimeDisplayTransportPosture::GuardedDisplay => {
            RuntimeFeedbackPagePosture::GuardedFeedbackPages
        }
        RuntimeDisplayTransportPosture::TextOnlyDisplay => {
            RuntimeFeedbackPagePosture::StatusFeedbackPages
        }
        RuntimeDisplayTransportPosture::PageAwareDisplay => {
            RuntimeFeedbackPagePosture::SceneAwareFeedbackPages
        }
        RuntimeDisplayTransportPosture::UnavailableDisplay => {
            RuntimeFeedbackPagePosture::UnavailableFeedbackPages
        }
    };
    let feedback_page_class = match feedback_page_posture {
        RuntimeFeedbackPagePosture::NoFeedbackPages
        | RuntimeFeedbackPagePosture::UnavailableFeedbackPages => {
            RuntimeFeedbackPageClass::NoFeedbackPageClass
        }
        RuntimeFeedbackPagePosture::GuardedFeedbackPages => {
            RuntimeFeedbackPageClass::GuardedVendorPage
        }
        RuntimeFeedbackPagePosture::StatusFeedbackPages => RuntimeFeedbackPageClass::StatusPage,
        RuntimeFeedbackPagePosture::SceneAwareFeedbackPages => RuntimeFeedbackPageClass::ScenePage,
    };
    let safe_action_graph_posture = match scene_mapping_posture {
        RuntimeSceneMappingPosture::NoSceneMapping => {
            RuntimeSafeActionGraphPosture::NoSafeActionGraph
        }
        RuntimeSceneMappingPosture::GuardedSceneMapping
        | RuntimeSceneMappingPosture::ContextualSceneMapping => {
            RuntimeSafeActionGraphPosture::GuardedSafeActionGraph
        }
        RuntimeSceneMappingPosture::PortableSceneMapping => {
            if matches!(
                feedback_page_posture,
                RuntimeFeedbackPagePosture::SceneAwareFeedbackPages
            ) {
                RuntimeSafeActionGraphPosture::SceneSafeActionGraph
            } else {
                RuntimeSafeActionGraphPosture::TransportSafeActionGraph
            }
        }
        RuntimeSceneMappingPosture::UnavailableSceneMapping => {
            RuntimeSafeActionGraphPosture::UnavailableSafeActionGraph
        }
    };
    let action_authority = RuntimeControlSurfaceWorkflowAuthority::RuntimeDefault;
    let safe_action_outcome = match safe_action_graph_posture {
        RuntimeSafeActionGraphPosture::NoSafeActionGraph => {
            RuntimeSafeActionOutcome::BypassUnsafeAction
        }
        RuntimeSafeActionGraphPosture::GuardedSafeActionGraph => {
            RuntimeSafeActionOutcome::CollapseToGuardedAction
        }
        RuntimeSafeActionGraphPosture::TransportSafeActionGraph
        | RuntimeSafeActionGraphPosture::SceneSafeActionGraph => {
            RuntimeSafeActionOutcome::PreserveDeclaredAction
        }
        RuntimeSafeActionGraphPosture::UnavailableSafeActionGraph => {
            RuntimeSafeActionOutcome::ObserveOnlyAction
        }
    };

    let capability = build_capability(device, feedback_channel_posture, &device.mapping_posture);

    let descriptor = RuntimeAdvancedHardwareDeviceDescriptor {
        device_id: device.device_id.clone(),
        device_name: device.device_name.clone(),
        scripting_safe_posture,
        feedback_channel_posture,
        display_transport_posture,
        display_content_class,
        motor_transport_posture,
        haptic_transport_posture,
        feedback_authority,
        feedback_outcome,
        scene_mapping_posture,
        feedback_page_posture,
        feedback_page_class,
        safe_action_graph_posture,
        action_authority,
        safe_action_outcome,
        capability: capability.clone(),
        summary: format!(
            "policy={:?} feedback={:?} display={:?}/{:?} motor={:?} haptic={:?} feedback_authority={:?} feedback_outcome={:?} scene={:?} page={:?}/{:?} action_graph={:?} action_authority={:?} action_outcome={:?} capability={}",
            scripting_safe_posture,
            feedback_channel_posture,
            display_transport_posture,
            display_content_class,
            motor_transport_posture,
            haptic_transport_posture,
            feedback_authority,
            feedback_outcome,
            scene_mapping_posture,
            feedback_page_posture,
            feedback_page_class,
            safe_action_graph_posture,
            action_authority,
            safe_action_outcome,
            capability.summary
        ),
    };

    let mut counts = DeviceProjectionCounts::default();
    match scripting_safe_posture {
        RuntimeScriptingSafeDevicePolicyPosture::Portable => counts.portable_device_count += 1,
        RuntimeScriptingSafeDevicePolicyPosture::Guarded => counts.guarded_device_count += 1,
        RuntimeScriptingSafeDevicePolicyPosture::ContextOnly => {
            counts.context_only_device_count += 1
        }
        RuntimeScriptingSafeDevicePolicyPosture::Denied => counts.denied_device_count += 1,
        RuntimeScriptingSafeDevicePolicyPosture::Unsupported => {}
    }
    if !matches!(
        feedback_channel_posture,
        RuntimeGuardedFeedbackChannelPosture::Unavailable
    ) {
        counts.feedback_channel_device_count += 1;
    }
    if !matches!(
        display_transport_posture,
        RuntimeDisplayTransportPosture::NotPresent
            | RuntimeDisplayTransportPosture::UnavailableDisplay
    ) {
        counts.display_transport_device_count += 1;
    }
    if !matches!(
        motor_transport_posture,
        RuntimeMotorTransportPosture::NoMotorTransport
            | RuntimeMotorTransportPosture::UnavailableMotorTransport
    ) {
        counts.motor_transport_device_count += 1;
    }
    if !matches!(
        haptic_transport_posture,
        RuntimeHapticTransportPosture::NoHapticTransport
            | RuntimeHapticTransportPosture::UnavailableHapticTransport
    ) {
        counts.haptic_transport_device_count += 1;
    }
    if !matches!(
        scene_mapping_posture,
        RuntimeSceneMappingPosture::NoSceneMapping
            | RuntimeSceneMappingPosture::UnavailableSceneMapping
    ) {
        counts.scene_mapping_device_count += 1;
    }
    if !matches!(
        feedback_page_posture,
        RuntimeFeedbackPagePosture::NoFeedbackPages
            | RuntimeFeedbackPagePosture::UnavailableFeedbackPages
    ) {
        counts.feedback_page_device_count += 1;
    }
    if !matches!(
        safe_action_graph_posture,
        RuntimeSafeActionGraphPosture::NoSafeActionGraph
            | RuntimeSafeActionGraphPosture::UnavailableSafeActionGraph
    ) {
        counts.safe_action_graph_device_count += 1;
    }

    DeviceProjection { descriptor, counts }
}
