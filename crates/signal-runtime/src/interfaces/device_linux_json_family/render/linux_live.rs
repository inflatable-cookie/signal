use super::*;

pub(crate) fn format_runtime_linux_backend_session_snapshot_compact(
    snapshot: &RuntimeLinuxBackendSessionSnapshot,
) -> String {
    format!(
        " linux_session={:?}/{:?}/{:?}/{:?}/{:?}/{:?} backend={} device={} stream={:?} simulated={} device_losses={} restart_attempts={} restart_failures={}",
        snapshot.backend_identity,
        snapshot.ownership,
        snapshot.lifecycle_state,
        snapshot.device_claim_posture,
        snapshot.session_role,
        snapshot.ownership_fallback,
        snapshot.backend_name,
        snapshot.device_id,
        snapshot.stream_state,
        snapshot.simulated,
        snapshot.device_loss_count,
        snapshot.restart_attempt_count,
        snapshot.restart_failure_count,
    )
}

pub(crate) fn format_runtime_linux_backend_session_snapshot_multiline(
    snapshot: &RuntimeLinuxBackendSessionSnapshot,
) -> String {
    format!(
        concat!(
            "\nlinux_backend_session_identity={:?}",
            "\nlinux_backend_session_backend_name={}",
            "\nlinux_backend_session_portability={:?}",
            "\nlinux_backend_session_ownership={:?}",
            "\nlinux_backend_session_lifecycle_state={:?}",
            "\nlinux_backend_session_device_claim_posture={:?}",
            "\nlinux_backend_session_role={:?}",
            "\nlinux_backend_session_ownership_fallback={:?}",
            "\nlinux_backend_session_device_id={}",
            "\nlinux_backend_session_device_name={}",
            "\nlinux_backend_session_stream_state={:?}",
            "\nlinux_backend_session_backend_health={:?}",
            "\nlinux_backend_session_simulated={}",
            "\nlinux_backend_session_device_loss_count={}",
            "\nlinux_backend_session_restart_attempt_count={}",
            "\nlinux_backend_session_restart_failure_count={}",
            "\nlinux_backend_session_summary={}",
        ),
        snapshot.backend_identity,
        snapshot.backend_name,
        snapshot.portability_band,
        snapshot.ownership,
        snapshot.lifecycle_state,
        snapshot.device_claim_posture,
        snapshot.session_role,
        snapshot.ownership_fallback,
        snapshot.device_id,
        snapshot.device_name,
        snapshot.stream_state,
        snapshot.backend_health,
        snapshot.simulated,
        snapshot.device_loss_count,
        snapshot.restart_attempt_count,
        snapshot.restart_failure_count,
        snapshot.summary,
    )
}

pub(crate) fn format_runtime_pipewire_alsa_parity_snapshot_compact(
    snapshot: &RuntimePipeWireAlsaParitySnapshot,
) -> String {
    format!(
        " pipewire_alsa={:?}/{:?}/{:?}/{:?} backend={} device={} stream={:?} simulated={} device_losses={} restart_attempts={} restart_failures={}",
        snapshot.session_role_parity,
        snapshot.device_claim_parity,
        snapshot.stream_policy_parity,
        snapshot.guarded_state,
        snapshot.backend_name,
        snapshot.device_id,
        snapshot.stream_state,
        snapshot.simulated,
        snapshot.device_loss_count,
        snapshot.restart_attempt_count,
        snapshot.restart_failure_count,
    )
}

pub(crate) fn format_runtime_pipewire_alsa_parity_snapshot_multiline(
    snapshot: &RuntimePipeWireAlsaParitySnapshot,
) -> String {
    format!(
        concat!(
            "\npipewire_alsa_parity_backend_identity={:?}",
            "\npipewire_alsa_parity_backend_name={}",
            "\npipewire_alsa_parity_portability={:?}",
            "\npipewire_alsa_parity_session_role={:?}",
            "\npipewire_alsa_parity_device_claim={:?}",
            "\npipewire_alsa_parity_stream_policy={:?}",
            "\npipewire_alsa_parity_guarded_state={:?}",
            "\npipewire_alsa_parity_lifecycle_ownership={:?}",
            "\npipewire_alsa_parity_restart_policy={:?}",
            "\npipewire_alsa_parity_clock_domain={:?}",
            "\npipewire_alsa_parity_fallback_state={:?}",
            "\npipewire_alsa_parity_device_id={}",
            "\npipewire_alsa_parity_device_name={}",
            "\npipewire_alsa_parity_stream_state={:?}",
            "\npipewire_alsa_parity_backend_health={:?}",
            "\npipewire_alsa_parity_simulated={}",
            "\npipewire_alsa_parity_device_loss_count={}",
            "\npipewire_alsa_parity_restart_attempt_count={}",
            "\npipewire_alsa_parity_restart_failure_count={}",
            "\npipewire_alsa_parity_summary={}",
        ),
        snapshot.backend_identity,
        snapshot.backend_name,
        snapshot.portability_band,
        snapshot.session_role_parity,
        snapshot.device_claim_parity,
        snapshot.stream_policy_parity,
        snapshot.guarded_state,
        snapshot.lifecycle_ownership,
        snapshot.restart_policy,
        snapshot.clock_domain,
        snapshot.fallback_state,
        snapshot.device_id,
        snapshot.device_name,
        snapshot.stream_state,
        snapshot.backend_health,
        snapshot.simulated,
        snapshot.device_loss_count,
        snapshot.restart_attempt_count,
        snapshot.restart_failure_count,
        snapshot.summary,
    )
}

pub(crate) fn format_runtime_jack_coordination_snapshot_compact(
    snapshot: &RuntimeJackCoordinationSnapshot,
) -> String {
    format!(
        " jack={:?}/{:?}/{:?}/{:?} backend={} device={} session={:?}/{} heartbeat={:?} dispatch={:?} simulated={}",
        snapshot.transport_posture,
        snapshot.graph_state,
        snapshot.client_role,
        snapshot.guarded_state,
        snapshot.backend_name,
        snapshot.device_id,
        snapshot.session_state,
        snapshot.currently_attached,
        snapshot.heartbeat_freshness,
        snapshot.dispatch_state,
        snapshot.simulated,
    )
}

pub(crate) fn format_runtime_jack_coordination_snapshot_multiline(
    snapshot: &RuntimeJackCoordinationSnapshot,
) -> String {
    format!(
        concat!(
            "\njack_coordination_backend_identity={:?}",
            "\njack_coordination_backend_name={}",
            "\njack_coordination_portability={:?}",
            "\njack_coordination_transport_posture={:?}",
            "\njack_coordination_graph_state={:?}",
            "\njack_coordination_client_role={:?}",
            "\njack_coordination_guarded_state={:?}",
            "\njack_coordination_device_id={}",
            "\njack_coordination_device_name={}",
            "\njack_coordination_session_state={:?}",
            "\njack_coordination_currently_attached={}",
            "\njack_coordination_heartbeat_freshness={:?}",
            "\njack_coordination_dispatch_state={:?}",
            "\njack_coordination_attach_events={}",
            "\njack_coordination_detach_requested_events={}",
            "\njack_coordination_detached_events={}",
            "\njack_coordination_backend_health={:?}",
            "\njack_coordination_simulated={}",
            "\njack_coordination_summary={}",
        ),
        snapshot.backend_identity,
        snapshot.backend_name,
        snapshot.portability_band,
        snapshot.transport_posture,
        snapshot.graph_state,
        snapshot.client_role,
        snapshot.guarded_state,
        snapshot.device_id,
        snapshot.device_name,
        snapshot.session_state,
        snapshot.currently_attached,
        snapshot.heartbeat_freshness,
        snapshot.dispatch_state,
        snapshot.attach_events,
        snapshot.detach_requested_events,
        snapshot.detached_events,
        snapshot.backend_health,
        snapshot.simulated,
        snapshot.summary,
    )
}
