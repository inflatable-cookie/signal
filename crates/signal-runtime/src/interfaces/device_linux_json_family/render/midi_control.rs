use super::*;

pub(crate) fn format_runtime_external_midi_snapshot_compact(
    snapshot: &RuntimeExternalMidiEndpointGraphSnapshot,
) -> String {
    format!(
        " external_midi={:?}/{:?}/{:?}/{:?}/{:?}/{:?} provider={} devices={} endpoints={}/{}/{}/{} routes={}/{}",
        snapshot.discovery_state,
        snapshot.graph_state,
        snapshot.live_ownership.ownership_posture,
        snapshot.live_ownership.attach_continuity,
        snapshot.live_ownership.backend_parity,
        snapshot.live_ownership.guarded_parity_outcome,
        snapshot.provider_name,
        snapshot.device_count,
        snapshot.endpoint_count,
        snapshot.input_endpoint_count,
        snapshot.output_endpoint_count,
        snapshot.duplex_endpoint_count,
        snapshot.active_route_count,
        snapshot.guarded_route_count,
    )
}

pub(crate) fn format_runtime_external_midi_snapshot_multiline(
    snapshot: &RuntimeExternalMidiEndpointGraphSnapshot,
) -> String {
    let device_lines = snapshot
        .devices
        .iter()
        .enumerate()
        .map(|(index, device)| {
            format!(
                "\nexternal_midi_device_{}={}/state={:?}/endpoints={}/summary={}",
                index,
                device.device_id,
                device.lifecycle_state,
                device.endpoint_count,
                device.summary,
            )
        })
        .collect::<String>();
    let endpoint_lines = snapshot
        .endpoints
        .iter()
        .enumerate()
        .map(|(index, endpoint)| {
            format!(
                "\nexternal_midi_endpoint_{}={}/device={}/direction={:?}/state={:?}/route={:?}/capability={}",
                index,
                endpoint.endpoint_id,
                endpoint.device_id,
                endpoint.direction,
                endpoint.lifecycle_state,
                endpoint.route_state,
                endpoint.capability.summary,
            )
        })
        .collect::<String>();
    format!(
        concat!(
            "\nexternal_midi_discovery_state={:?}",
            "\nexternal_midi_graph_state={:?}",
            "\nexternal_midi_live_ownership_posture={:?}",
            "\nexternal_midi_attach_continuity={:?}",
            "\nexternal_midi_backend_parity={:?}",
            "\nexternal_midi_guarded_parity_outcome={:?}",
            "\nexternal_midi_backend_identity={:?}",
            "\nexternal_midi_device_loss_count={}",
            "\nexternal_midi_restart_attempt_count={}",
            "\nexternal_midi_restart_failure_count={}",
            "\nexternal_midi_provider_name={}",
            "\nexternal_midi_device_count={}",
            "\nexternal_midi_endpoint_count={}",
            "\nexternal_midi_input_endpoint_count={}",
            "\nexternal_midi_output_endpoint_count={}",
            "\nexternal_midi_duplex_endpoint_count={}",
            "\nexternal_midi_active_route_count={}",
            "\nexternal_midi_guarded_route_count={}",
            "\nexternal_midi_summary={}",
        ),
        snapshot.discovery_state,
        snapshot.graph_state,
        snapshot.live_ownership.ownership_posture,
        snapshot.live_ownership.attach_continuity,
        snapshot.live_ownership.backend_parity,
        snapshot.live_ownership.guarded_parity_outcome,
        snapshot.live_ownership.backend_identity,
        snapshot.live_ownership.device_loss_count,
        snapshot.live_ownership.restart_attempt_count,
        snapshot.live_ownership.restart_failure_count,
        snapshot.provider_name,
        snapshot.device_count,
        snapshot.endpoint_count,
        snapshot.input_endpoint_count,
        snapshot.output_endpoint_count,
        snapshot.duplex_endpoint_count,
        snapshot.active_route_count,
        snapshot.guarded_route_count,
        snapshot.summary,
    ) + &device_lines
        + &endpoint_lines
}

pub(crate) fn format_runtime_control_surface_snapshot_multiline(
    snapshot: &RuntimeControlSurfaceSnapshot,
) -> String {
    let device_lines = snapshot
        .devices
        .iter()
        .enumerate()
        .map(|(index, device)| {
            format!(
                "\ncontrol_surface_device_{}={}/transport={:?}/mapping={:?}/feedback={:?}/capability={}",
                index,
                device.device_id,
                device.transport_posture,
                device.mapping_posture,
                device.feedback_readiness,
                device.capability.summary,
            )
        })
        .collect::<String>();
    format!(
        concat!(
            "\ncontrol_surface_discovery_state={:?}",
            "\ncontrol_surface_graph_state={:?}",
            "\ncontrol_surface_provider_name={}",
            "\ncontrol_surface_device_count={}",
            "\ncontrol_surface_mapped_device_count={}",
            "\ncontrol_surface_feedback_ready_device_count={}",
            "\ncontrol_surface_guarded_device_count={}",
            "\ncontrol_surface_summary={}",
        ),
        snapshot.discovery_state,
        snapshot.graph_state,
        snapshot.provider_name,
        snapshot.device_count,
        snapshot.mapped_device_count,
        snapshot.feedback_ready_device_count,
        snapshot.guarded_device_count,
        snapshot.summary,
    ) + &device_lines
}

pub(crate) fn format_runtime_advanced_hardware_snapshot_multiline(
    snapshot: &RuntimeAdvancedHardwareSnapshot,
) -> String {
    let device_lines = snapshot
        .devices
        .iter()
        .enumerate()
        .map(|(index, device)| {
            format!(
                "\nadvanced_hardware_device_{}={}/policy={:?}/feedback={:?}/display={:?}/{:?}/motor={:?}/haptic={:?}/feedback_authority={:?}/feedback_outcome={:?}/scene={:?}/page={:?}/{:?}/action_graph={:?}/action_authority={:?}/action_outcome={:?}/capability={}",
                index,
                device.device_id,
                device.scripting_safe_posture,
                device.feedback_channel_posture,
                device.display_transport_posture,
                device.display_content_class,
                device.motor_transport_posture,
                device.haptic_transport_posture,
                device.feedback_authority,
                device.feedback_outcome,
                device.scene_mapping_posture,
                device.feedback_page_posture,
                device.feedback_page_class,
                device.safe_action_graph_posture,
                device.action_authority,
                device.safe_action_outcome,
                device.capability.summary,
            )
        })
        .collect::<String>();
    format!(
        concat!(
            "\nadvanced_hardware_discovery_state={:?}",
            "\nadvanced_hardware_graph_state={:?}",
            "\nadvanced_hardware_provider_name={}",
            "\nadvanced_hardware_device_count={}",
            "\nadvanced_hardware_portable_device_count={}",
            "\nadvanced_hardware_guarded_device_count={}",
            "\nadvanced_hardware_context_only_device_count={}",
            "\nadvanced_hardware_denied_device_count={}",
            "\nadvanced_hardware_feedback_channel_device_count={}",
            "\nadvanced_hardware_display_transport_device_count={}",
            "\nadvanced_hardware_motor_transport_device_count={}",
            "\nadvanced_hardware_haptic_transport_device_count={}",
            "\nadvanced_hardware_scene_mapping_device_count={}",
            "\nadvanced_hardware_feedback_page_device_count={}",
            "\nadvanced_hardware_safe_action_graph_device_count={}",
            "\nadvanced_hardware_summary={}",
        ),
        snapshot.discovery_state,
        snapshot.graph_state,
        snapshot.provider_name,
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
        snapshot.summary,
    ) + &device_lines
}
