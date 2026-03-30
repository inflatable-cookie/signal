use super::*;

use crate::RuntimeControllerExpressionMidi2Posture;

#[path = "interfaces_external_midi_family/projection.rs"]
mod projection;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RuntimeExternalMidiDiscoveryState {
    Unavailable,
    Idle,
    Enumerated,
    Changed,
    Faulted,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RuntimeExternalMidiGraphState {
    Unavailable,
    Empty,
    Stable,
    Guarded,
    Faulted,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RuntimeExternalMidiDeviceLifecycleState {
    Unavailable,
    Discovered,
    Guarded,
    Detached,
    Faulted,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RuntimeExternalMidiEndpointLifecycleState {
    Unavailable,
    Idle,
    Active,
    Guarded,
    Detached,
    Faulted,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RuntimeExternalMidiEndpointDirection {
    Input,
    Output,
    Duplex,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RuntimeExternalMidiRouteState {
    Unavailable,
    Detached,
    InputObserved,
    OutputObserved,
    DuplexObserved,
    Guarded,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RuntimeExternalMidiLiveOwnershipPosture {
    Unavailable,
    NoLiveOwnership,
    RuntimeDeclaredLiveOwnership,
    GuardedLiveOwnership,
    BackendAdvisoryLiveOwnership,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RuntimeExternalMidiAttachContinuity {
    Unavailable,
    Detached,
    Attached,
    Resumable,
    Restartable,
    Terminal,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RuntimeExternalMidiBackendParity {
    NotLinux,
    Unavailable,
    Portable,
    Guarded,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RuntimeExternalMidiGuardedParityOutcome {
    NotLinux,
    Unavailable,
    Direct,
    BackendManaged,
    RecoveryGuarded,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeExternalMidiLiveOwnershipSummary {
    pub ownership_posture: RuntimeExternalMidiLiveOwnershipPosture,
    pub attach_continuity: RuntimeExternalMidiAttachContinuity,
    pub backend_parity: RuntimeExternalMidiBackendParity,
    pub guarded_parity_outcome: RuntimeExternalMidiGuardedParityOutcome,
    pub backend_identity: RuntimeLinuxAudioBackendIdentity,
    pub device_loss_count: u64,
    pub restart_attempt_count: u64,
    pub restart_failure_count: u64,
    pub summary: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeExternalMidiEndpointCapabilitySummary {
    pub supports_bounded_midi_input: bool,
    pub supports_bounded_midi_output: bool,
    pub supports_transport_clock: bool,
    pub supports_note_events: bool,
    pub supports_controller_events: bool,
    pub supports_note_pressure_expression: bool,
    pub supports_note_timbre_expression: bool,
    pub supports_note_tuning_expression: bool,
    pub supports_mpe: bool,
    pub midi2_posture: RuntimeControllerExpressionMidi2Posture,
    pub control_surface_guarded: bool,
    pub summary: String,
}

impl RuntimeExternalMidiEndpointCapabilitySummary {
    pub fn unavailable() -> Self {
        Self {
            supports_bounded_midi_input: false,
            supports_bounded_midi_output: false,
            supports_transport_clock: false,
            supports_note_events: false,
            supports_controller_events: false,
            supports_note_pressure_expression: false,
            supports_note_timbre_expression: false,
            supports_note_tuning_expression: false,
            supports_mpe: false,
            midi2_posture: RuntimeControllerExpressionMidi2Posture::Unsupported,
            control_surface_guarded: true,
            summary: "midi-input=false midi-output=false transport-clock=false note-events=false controller-events=false pressure=false timbre=false tuning=false mpe=false midi2=Unsupported control-surface=guarded".into(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeExternalMidiDeviceDescriptor {
    pub device_id: String,
    pub device_name: String,
    pub lifecycle_state: RuntimeExternalMidiDeviceLifecycleState,
    pub endpoint_count: usize,
    pub summary: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeExternalMidiEndpointDescriptor {
    pub endpoint_id: String,
    pub endpoint_name: String,
    pub device_id: String,
    pub direction: RuntimeExternalMidiEndpointDirection,
    pub lifecycle_state: RuntimeExternalMidiEndpointLifecycleState,
    pub route_state: RuntimeExternalMidiRouteState,
    pub capability: RuntimeExternalMidiEndpointCapabilitySummary,
    pub summary: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeExternalMidiEndpointGraphSnapshot {
    pub discovery_state: RuntimeExternalMidiDiscoveryState,
    pub graph_state: RuntimeExternalMidiGraphState,
    pub live_ownership: RuntimeExternalMidiLiveOwnershipSummary,
    pub provider_name: String,
    pub device_count: usize,
    pub endpoint_count: usize,
    pub input_endpoint_count: usize,
    pub output_endpoint_count: usize,
    pub duplex_endpoint_count: usize,
    pub active_route_count: usize,
    pub guarded_route_count: usize,
    pub devices: Vec<RuntimeExternalMidiDeviceDescriptor>,
    pub endpoints: Vec<RuntimeExternalMidiEndpointDescriptor>,
    pub summary: String,
}
