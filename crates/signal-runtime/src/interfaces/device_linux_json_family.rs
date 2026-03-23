use super::*;

mod advanced_hardware;
mod control_surface;
mod external_midi;
mod linux_live;

pub(super) fn json_runtime_external_midi_snapshot(
    snapshot: &RuntimeExternalMidiEndpointGraphSnapshot,
) -> String {
    external_midi::json_runtime_external_midi_snapshot(snapshot)
}

pub(super) fn json_runtime_control_surface_snapshot(
    snapshot: &RuntimeControlSurfaceSnapshot,
) -> String {
    control_surface::json_runtime_control_surface_snapshot(snapshot)
}

pub(super) fn json_runtime_advanced_hardware_snapshot(
    snapshot: &RuntimeAdvancedHardwareSnapshot,
) -> String {
    advanced_hardware::json_runtime_advanced_hardware_snapshot(snapshot)
}

pub(super) fn json_runtime_linux_backend_session_snapshot(
    snapshot: &RuntimeLinuxBackendSessionSnapshot,
) -> String {
    linux_live::json_runtime_linux_backend_session_snapshot(snapshot)
}

pub(super) fn json_runtime_pipewire_alsa_parity_snapshot(
    snapshot: &RuntimePipeWireAlsaParitySnapshot,
) -> String {
    linux_live::json_runtime_pipewire_alsa_parity_snapshot(snapshot)
}

pub(super) fn json_runtime_jack_coordination_snapshot(
    snapshot: &RuntimeJackCoordinationSnapshot,
) -> String {
    linux_live::json_runtime_jack_coordination_snapshot(snapshot)
}
