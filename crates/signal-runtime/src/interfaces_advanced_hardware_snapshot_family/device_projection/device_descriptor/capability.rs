use crate::{
    RuntimeAdvancedHardwareActionClass, RuntimeAdvancedHardwareCapabilitySummary,
    RuntimeControlSurfaceDeviceDescriptor, RuntimeControlSurfaceMappingPosture,
    RuntimeGuardedFeedbackChannelPosture,
};

pub(super) fn build_capability(
    device: &RuntimeControlSurfaceDeviceDescriptor,
    feedback_channel_posture: RuntimeGuardedFeedbackChannelPosture,
    mapping_posture: &RuntimeControlSurfaceMappingPosture,
) -> RuntimeAdvancedHardwareCapabilitySummary {
    let supports_display_feedback = !matches!(
        feedback_channel_posture,
        RuntimeGuardedFeedbackChannelPosture::Unavailable
    );
    let supports_motor_feedback = false;
    let supports_haptic_feedback = false;
    let supports_bank_navigation = !matches!(
        mapping_posture,
        RuntimeControlSurfaceMappingPosture::Unsupported
    );
    let supports_macro_triggers = device.capability.supports_transport_control;
    let supports_device_state_observation = true;

    let mut action_classes = Vec::new();
    if supports_display_feedback {
        action_classes.push(RuntimeAdvancedHardwareActionClass::DisplayFeedback);
    }
    if supports_bank_navigation {
        action_classes.push(RuntimeAdvancedHardwareActionClass::BankNavigation);
    }
    if supports_macro_triggers {
        action_classes.push(RuntimeAdvancedHardwareActionClass::MacroTrigger);
    }
    if supports_device_state_observation {
        action_classes.push(RuntimeAdvancedHardwareActionClass::DeviceStateObservation);
    }

    RuntimeAdvancedHardwareCapabilitySummary {
        supports_display_feedback,
        supports_motor_feedback,
        supports_haptic_feedback,
        supports_bank_navigation,
        supports_macro_triggers,
        supports_device_state_observation,
        action_classes: action_classes.clone(),
        summary: format!(
            "display-feedback={} motor-feedback={} haptic-feedback={} bank-navigation={} macro-triggers={} device-state-observation={} action-classes={}",
            supports_display_feedback,
            supports_motor_feedback,
            supports_haptic_feedback,
            supports_bank_navigation,
            supports_macro_triggers,
            supports_device_state_observation,
            action_classes.len()
        ),
    }
}
