use super::*;

fn json_runtime_advanced_hardware_action_class(
    action_class: &RuntimeAdvancedHardwareActionClass,
) -> String {
    json_string(&format!("{:?}", action_class))
}

fn json_runtime_advanced_hardware_capability_summary(
    summary: &RuntimeAdvancedHardwareCapabilitySummary,
) -> String {
    format!(
        concat!(
            "{{",
            "\"supports_display_feedback\":{},",
            "\"supports_motor_feedback\":{},",
            "\"supports_haptic_feedback\":{},",
            "\"supports_bank_navigation\":{},",
            "\"supports_macro_triggers\":{},",
            "\"supports_device_state_observation\":{},",
            "\"action_classes\":{},",
            "\"summary\":{}",
            "}}"
        ),
        summary.supports_display_feedback,
        summary.supports_motor_feedback,
        summary.supports_haptic_feedback,
        summary.supports_bank_navigation,
        summary.supports_macro_triggers,
        summary.supports_device_state_observation,
        format!(
            "[{}]",
            summary
                .action_classes
                .iter()
                .map(json_runtime_advanced_hardware_action_class)
                .collect::<Vec<_>>()
                .join(",")
        ),
        json_option_string(Some(summary.summary.as_str())),
    )
}

fn json_runtime_advanced_hardware_device_descriptor(
    descriptor: &RuntimeAdvancedHardwareDeviceDescriptor,
) -> String {
    format!(
        concat!(
            "{{",
            "\"device_id\":{},",
            "\"device_name\":{},",
            "\"scripting_safe_posture\":{},",
            "\"feedback_channel_posture\":{},",
            "\"display_transport_posture\":{},",
            "\"display_content_class\":{},",
            "\"motor_transport_posture\":{},",
            "\"haptic_transport_posture\":{},",
            "\"feedback_authority\":{},",
            "\"feedback_outcome\":{},",
            "\"scene_mapping_posture\":{},",
            "\"feedback_page_posture\":{},",
            "\"feedback_page_class\":{},",
            "\"safe_action_graph_posture\":{},",
            "\"action_authority\":{},",
            "\"safe_action_outcome\":{},",
            "\"capability\":{},",
            "\"summary\":{}",
            "}}"
        ),
        json_option_string(Some(descriptor.device_id.as_str())),
        json_option_string(Some(descriptor.device_name.as_str())),
        json_string(&format!("{:?}", descriptor.scripting_safe_posture)),
        json_string(&format!("{:?}", descriptor.feedback_channel_posture)),
        json_string(&format!("{:?}", descriptor.display_transport_posture)),
        json_string(&format!("{:?}", descriptor.display_content_class)),
        json_string(&format!("{:?}", descriptor.motor_transport_posture)),
        json_string(&format!("{:?}", descriptor.haptic_transport_posture)),
        json_string(&format!("{:?}", descriptor.feedback_authority)),
        json_string(&format!("{:?}", descriptor.feedback_outcome)),
        json_string(&format!("{:?}", descriptor.scene_mapping_posture)),
        json_string(&format!("{:?}", descriptor.feedback_page_posture)),
        json_string(&format!("{:?}", descriptor.feedback_page_class)),
        json_string(&format!("{:?}", descriptor.safe_action_graph_posture)),
        json_string(&format!("{:?}", descriptor.action_authority)),
        json_string(&format!("{:?}", descriptor.safe_action_outcome)),
        json_runtime_advanced_hardware_capability_summary(&descriptor.capability),
        json_option_string(Some(descriptor.summary.as_str())),
    )
}

pub(super) fn json_runtime_advanced_hardware_snapshot(
    snapshot: &RuntimeAdvancedHardwareSnapshot,
) -> String {
    format!(
        concat!(
            "{{",
            "\"discovery_state\":{},",
            "\"graph_state\":{},",
            "\"provider_name\":{},",
            "\"device_count\":{},",
            "\"portable_device_count\":{},",
            "\"guarded_device_count\":{},",
            "\"context_only_device_count\":{},",
            "\"denied_device_count\":{},",
            "\"feedback_channel_device_count\":{},",
            "\"display_transport_device_count\":{},",
            "\"motor_transport_device_count\":{},",
            "\"haptic_transport_device_count\":{},",
            "\"scene_mapping_device_count\":{},",
            "\"feedback_page_device_count\":{},",
            "\"safe_action_graph_device_count\":{},",
            "\"devices\":{},",
            "\"summary\":{}",
            "}}"
        ),
        json_string(&format!("{:?}", snapshot.discovery_state)),
        json_string(&format!("{:?}", snapshot.graph_state)),
        json_option_string(Some(snapshot.provider_name.as_str())),
        snapshot.device_count,
        snapshot.portable_device_count,
        snapshot.guarded_device_count,
        snapshot.context_only_device_count,
        snapshot.denied_device_count,
        snapshot.feedback_channel_device_count,
        snapshot.display_transport_device_count,
        snapshot.motor_transport_device_count,
        snapshot.haptic_transport_device_count,
        snapshot.scene_mapping_device_count,
        snapshot.feedback_page_device_count,
        snapshot.safe_action_graph_device_count,
        format!(
            "[{}]",
            snapshot
                .devices
                .iter()
                .map(json_runtime_advanced_hardware_device_descriptor)
                .collect::<Vec<_>>()
                .join(",")
        ),
        json_option_string(Some(snapshot.summary.as_str())),
    )
}
