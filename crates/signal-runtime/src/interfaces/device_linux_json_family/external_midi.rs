use super::*;

fn json_runtime_external_midi_endpoint_capability_summary(
    summary: &RuntimeExternalMidiEndpointCapabilitySummary,
) -> String {
    format!(
        concat!(
            "{{",
            "\"supports_bounded_midi_input\":{},",
            "\"supports_bounded_midi_output\":{},",
            "\"supports_transport_clock\":{},",
            "\"supports_note_events\":{},",
            "\"supports_controller_events\":{},",
            "\"supports_note_pressure_expression\":{},",
            "\"supports_note_timbre_expression\":{},",
            "\"supports_note_tuning_expression\":{},",
            "\"supports_mpe\":{},",
            "\"midi2_posture\":{},",
            "\"control_surface_guarded\":{},",
            "\"summary\":{}",
            "}}"
        ),
        summary.supports_bounded_midi_input,
        summary.supports_bounded_midi_output,
        summary.supports_transport_clock,
        summary.supports_note_events,
        summary.supports_controller_events,
        summary.supports_note_pressure_expression,
        summary.supports_note_timbre_expression,
        summary.supports_note_tuning_expression,
        summary.supports_mpe,
        json_string(&format!("{:?}", summary.midi2_posture)),
        summary.control_surface_guarded,
        json_option_string(Some(summary.summary.as_str())),
    )
}

fn json_runtime_external_midi_device_descriptor(
    descriptor: &RuntimeExternalMidiDeviceDescriptor,
) -> String {
    format!(
        concat!(
            "{{",
            "\"device_id\":{},",
            "\"device_name\":{},",
            "\"lifecycle_state\":{},",
            "\"endpoint_count\":{},",
            "\"summary\":{}",
            "}}"
        ),
        json_option_string(Some(descriptor.device_id.as_str())),
        json_option_string(Some(descriptor.device_name.as_str())),
        json_string(&format!("{:?}", descriptor.lifecycle_state)),
        descriptor.endpoint_count,
        json_option_string(Some(descriptor.summary.as_str())),
    )
}

fn json_runtime_external_midi_endpoint_descriptor(
    descriptor: &RuntimeExternalMidiEndpointDescriptor,
) -> String {
    format!(
        concat!(
            "{{",
            "\"endpoint_id\":{},",
            "\"endpoint_name\":{},",
            "\"device_id\":{},",
            "\"direction\":{},",
            "\"lifecycle_state\":{},",
            "\"route_state\":{},",
            "\"capability\":{},",
            "\"summary\":{}",
            "}}"
        ),
        json_option_string(Some(descriptor.endpoint_id.as_str())),
        json_option_string(Some(descriptor.endpoint_name.as_str())),
        json_option_string(Some(descriptor.device_id.as_str())),
        json_string(&format!("{:?}", descriptor.direction)),
        json_string(&format!("{:?}", descriptor.lifecycle_state)),
        json_string(&format!("{:?}", descriptor.route_state)),
        json_runtime_external_midi_endpoint_capability_summary(&descriptor.capability),
        json_option_string(Some(descriptor.summary.as_str())),
    )
}

fn json_runtime_external_midi_live_ownership_summary(
    summary: &RuntimeExternalMidiLiveOwnershipSummary,
) -> String {
    format!(
        concat!(
            "{{",
            "\"ownership_posture\":{},",
            "\"attach_continuity\":{},",
            "\"backend_parity\":{},",
            "\"guarded_parity_outcome\":{},",
            "\"backend_identity\":{},",
            "\"device_loss_count\":{},",
            "\"restart_attempt_count\":{},",
            "\"restart_failure_count\":{},",
            "\"summary\":{}",
            "}}"
        ),
        json_string(&format!("{:?}", summary.ownership_posture)),
        json_string(&format!("{:?}", summary.attach_continuity)),
        json_string(&format!("{:?}", summary.backend_parity)),
        json_string(&format!("{:?}", summary.guarded_parity_outcome)),
        json_string(&format!("{:?}", summary.backend_identity)),
        summary.device_loss_count,
        summary.restart_attempt_count,
        summary.restart_failure_count,
        json_option_string(Some(summary.summary.as_str())),
    )
}

pub(super) fn json_runtime_external_midi_snapshot(
    snapshot: &RuntimeExternalMidiEndpointGraphSnapshot,
) -> String {
    format!(
        concat!(
            "{{",
            "\"discovery_state\":{},",
            "\"graph_state\":{},",
            "\"live_ownership\":{},",
            "\"provider_name\":{},",
            "\"device_count\":{},",
            "\"endpoint_count\":{},",
            "\"input_endpoint_count\":{},",
            "\"output_endpoint_count\":{},",
            "\"duplex_endpoint_count\":{},",
            "\"active_route_count\":{},",
            "\"guarded_route_count\":{},",
            "\"devices\":{},",
            "\"endpoints\":{},",
            "\"summary\":{}",
            "}}"
        ),
        json_string(&format!("{:?}", snapshot.discovery_state)),
        json_string(&format!("{:?}", snapshot.graph_state)),
        json_runtime_external_midi_live_ownership_summary(&snapshot.live_ownership),
        json_option_string(Some(snapshot.provider_name.as_str())),
        snapshot.device_count,
        snapshot.endpoint_count,
        snapshot.input_endpoint_count,
        snapshot.output_endpoint_count,
        snapshot.duplex_endpoint_count,
        snapshot.active_route_count,
        snapshot.guarded_route_count,
        format!(
            "[{}]",
            snapshot
                .devices
                .iter()
                .map(json_runtime_external_midi_device_descriptor)
                .collect::<Vec<_>>()
                .join(",")
        ),
        format!(
            "[{}]",
            snapshot
                .endpoints
                .iter()
                .map(json_runtime_external_midi_endpoint_descriptor)
                .collect::<Vec<_>>()
                .join(",")
        ),
        json_option_string(Some(snapshot.summary.as_str())),
    )
}
