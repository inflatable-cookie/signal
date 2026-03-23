use super::*;

fn json_runtime_control_surface_capability_summary(
    summary: &RuntimeControlSurfaceCapabilitySummary,
) -> String {
    format!(
        concat!(
            "{{",
            "\"supports_transport_control\":{},",
            "\"supports_mapping_input\":{},",
            "\"supports_feedback_output\":{},",
            "\"supports_widened_expression\":{},",
            "\"summary\":{}",
            "}}"
        ),
        summary.supports_transport_control,
        summary.supports_mapping_input,
        summary.supports_feedback_output,
        summary.supports_widened_expression,
        json_option_string(Some(summary.summary.as_str())),
    )
}

fn json_runtime_control_surface_device_descriptor(
    descriptor: &RuntimeControlSurfaceDeviceDescriptor,
) -> String {
    format!(
        concat!(
            "{{",
            "\"device_id\":{},",
            "\"device_name\":{},",
            "\"transport_posture\":{},",
            "\"mapping_posture\":{},",
            "\"feedback_readiness\":{},",
            "\"capability\":{},",
            "\"summary\":{}",
            "}}"
        ),
        json_option_string(Some(descriptor.device_id.as_str())),
        json_option_string(Some(descriptor.device_name.as_str())),
        json_string(&format!("{:?}", descriptor.transport_posture)),
        json_string(&format!("{:?}", descriptor.mapping_posture)),
        json_string(&format!("{:?}", descriptor.feedback_readiness)),
        json_runtime_control_surface_capability_summary(&descriptor.capability),
        json_option_string(Some(descriptor.summary.as_str())),
    )
}

pub(super) fn json_runtime_control_surface_snapshot(
    snapshot: &RuntimeControlSurfaceSnapshot,
) -> String {
    format!(
        concat!(
            "{{",
            "\"discovery_state\":{},",
            "\"graph_state\":{},",
            "\"provider_name\":{},",
            "\"device_count\":{},",
            "\"mapped_device_count\":{},",
            "\"feedback_ready_device_count\":{},",
            "\"guarded_device_count\":{},",
            "\"devices\":{},",
            "\"summary\":{}",
            "}}"
        ),
        json_string(&format!("{:?}", snapshot.discovery_state)),
        json_string(&format!("{:?}", snapshot.graph_state)),
        json_option_string(Some(snapshot.provider_name.as_str())),
        snapshot.device_count,
        snapshot.mapped_device_count,
        snapshot.feedback_ready_device_count,
        snapshot.guarded_device_count,
        format!(
            "[{}]",
            snapshot
                .devices
                .iter()
                .map(json_runtime_control_surface_device_descriptor)
                .collect::<Vec<_>>()
                .join(",")
        ),
        json_option_string(Some(snapshot.summary.as_str())),
    )
}
