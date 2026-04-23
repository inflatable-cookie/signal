use super::*;

use crate::RuntimeControllerExpressionMidi2Posture;

#[path = "interfaces_external_midi_family/projection.rs"]
mod projection;

/// Discovery phase of the external MIDI device graph.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RuntimeExternalMidiDiscoveryState {
    /// MIDI discovery backend is unavailable.
    Unavailable,
    /// Discovery backend is idle; no enumeration in progress.
    Idle,
    /// Device list has been successfully enumerated.
    Enumerated,
    /// Device list has changed since the last enumeration.
    Changed,
    /// Discovery encountered a fault condition.
    Faulted,
}

/// Overall health of the external MIDI routing graph.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RuntimeExternalMidiGraphState {
    /// MIDI routing graph backend is unavailable.
    Unavailable,
    /// Graph is available but no devices are connected.
    Empty,
    /// All routes are established and healthy.
    Stable,
    /// One or more routes or devices are in a guarded state.
    Guarded,
    /// Graph encountered a fault condition.
    Faulted,
}

/// Lifecycle state of a physical MIDI device.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RuntimeExternalMidiDeviceLifecycleState {
    /// Device lifecycle state is unavailable.
    Unavailable,
    /// Device has been discovered and is present.
    Discovered,
    /// Device is present but in a guarded state.
    Guarded,
    /// Device has been detached or removed.
    Detached,
    /// Device lifecycle encountered a fault.
    Faulted,
}

/// Lifecycle state of a single MIDI endpoint (port).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RuntimeExternalMidiEndpointLifecycleState {
    /// Endpoint lifecycle state is unavailable.
    Unavailable,
    /// Endpoint is present but no data is flowing.
    Idle,
    /// Endpoint is actively sending or receiving MIDI data.
    Active,
    /// Endpoint is present but in a guarded state.
    Guarded,
    /// Endpoint has been detached or removed.
    Detached,
    /// Endpoint lifecycle encountered a fault.
    Faulted,
}

/// Data-flow direction of a MIDI endpoint.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RuntimeExternalMidiEndpointDirection {
    /// Endpoint receives MIDI data from the device.
    Input,
    /// Endpoint sends MIDI data to the device.
    Output,
    /// Endpoint both receives and sends MIDI data.
    Duplex,
}

/// Current routing state of a MIDI endpoint.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RuntimeExternalMidiRouteState {
    /// Route state is unavailable.
    Unavailable,
    /// Endpoint is not routed.
    Detached,
    /// MIDI input route is active and observed.
    InputObserved,
    /// MIDI output route is active and observed.
    OutputObserved,
    /// Both input and output routes are active and observed.
    DuplexObserved,
    /// Route is present but in a guarded state.
    Guarded,
}

/// Who owns the live MIDI I/O session.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RuntimeExternalMidiLiveOwnershipPosture {
    /// Live ownership state is unavailable.
    Unavailable,
    /// No entity currently owns the live MIDI session.
    NoLiveOwnership,
    /// The runtime has explicitly declared ownership of the live session.
    RuntimeDeclaredLiveOwnership,
    /// Live ownership is present but in a guarded state.
    GuardedLiveOwnership,
    /// The backend is advisory; the runtime defers to the backend's preference.
    BackendAdvisoryLiveOwnership,
}

/// Continuity class of the MIDI live-session attachment.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RuntimeExternalMidiAttachContinuity {
    /// Attach continuity state is unavailable.
    Unavailable,
    /// Session is not attached.
    Detached,
    /// Session is attached and active.
    Attached,
    /// Session can be resumed after a brief interruption.
    Resumable,
    /// Session requires a full restart to reattach.
    Restartable,
    /// Session has entered a terminal state and cannot recover.
    Terminal,
}

/// Platform portability band for MIDI backend support.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RuntimeExternalMidiBackendParity {
    /// Running on a non-Linux platform; MIDI parity classification is not applicable.
    NotLinux,
    /// MIDI backend parity is unavailable.
    Unavailable,
    /// MIDI backend is portable across platforms.
    Portable,
    /// MIDI backend portability is guarded by a platform-specific constraint.
    Guarded,
}

/// Resolved Linux parity outcome for guarded MIDI backends.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RuntimeExternalMidiGuardedParityOutcome {
    /// Running on a non-Linux platform; guarded parity outcome is not applicable.
    NotLinux,
    /// Guarded parity outcome is unavailable.
    Unavailable,
    /// Direct ownership; no guarded constraint is active.
    Direct,
    /// MIDI backend is managing the session; direct ownership is not available.
    BackendManaged,
    /// Session is constrained to a recovery-guarded state.
    RecoveryGuarded,
}

/// Summary of MIDI live-session ownership, continuity, parity, and fault counters.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeExternalMidiLiveOwnershipSummary {
    /// Who owns the live MIDI I/O session.
    pub ownership_posture: RuntimeExternalMidiLiveOwnershipPosture,
    /// Continuity class of the MIDI live-session attachment.
    pub attach_continuity: RuntimeExternalMidiAttachContinuity,
    /// Platform portability band for MIDI backend support.
    pub backend_parity: RuntimeExternalMidiBackendParity,
    /// Resolved Linux parity outcome for guarded MIDI backends.
    pub guarded_parity_outcome: RuntimeExternalMidiGuardedParityOutcome,
    /// Linux audio backend identity used for parity classification.
    pub backend_identity: RuntimeLinuxAudioBackendIdentity,
    /// Number of times the MIDI device was lost and had to be reacquired.
    pub device_loss_count: u64,
    /// Number of MIDI backend restart attempts.
    pub restart_attempt_count: u64,
    /// Number of MIDI backend restart attempts that failed.
    pub restart_failure_count: u64,
    /// Human-readable summary of the live ownership state.
    pub summary: String,
}

/// Capability flags for a single MIDI endpoint.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeExternalMidiEndpointCapabilitySummary {
    /// Whether the endpoint supports bounded MIDI input.
    pub supports_bounded_midi_input: bool,
    /// Whether the endpoint supports bounded MIDI output.
    pub supports_bounded_midi_output: bool,
    /// Whether the endpoint can source a MIDI transport clock.
    pub supports_transport_clock: bool,
    /// Whether the endpoint can generate note-on/off events.
    pub supports_note_events: bool,
    /// Whether the endpoint can generate controller (CC) events.
    pub supports_controller_events: bool,
    /// Whether the endpoint supports per-note pressure expression.
    pub supports_note_pressure_expression: bool,
    /// Whether the endpoint supports per-note timbre expression.
    pub supports_note_timbre_expression: bool,
    /// Whether the endpoint supports per-note tuning expression.
    pub supports_note_tuning_expression: bool,
    /// Whether the endpoint supports MIDI Polyphonic Expression (MPE).
    pub supports_mpe: bool,
    /// MIDI 2.0 expression posture for this endpoint.
    pub midi2_posture: RuntimeControllerExpressionMidi2Posture,
    /// Whether this endpoint is guarded from use as a control surface.
    pub control_surface_guarded: bool,
    /// Human-readable summary of the capability flags.
    pub summary: String,
}

impl RuntimeExternalMidiEndpointCapabilitySummary {
    /// Returns a capability summary that marks the endpoint as unavailable and guarded.
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

/// Descriptor for a single physical MIDI device and its endpoint count.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeExternalMidiDeviceDescriptor {
    /// Stable identifier for this MIDI device.
    pub device_id: String,
    /// Human-readable name for this MIDI device.
    pub device_name: String,
    /// Current lifecycle state of this device.
    pub lifecycle_state: RuntimeExternalMidiDeviceLifecycleState,
    /// Number of endpoints belonging to this device.
    pub endpoint_count: usize,
    /// Human-readable summary of this device descriptor.
    pub summary: String,
}

/// Descriptor for a single MIDI endpoint including direction, state, and capability.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeExternalMidiEndpointDescriptor {
    /// Stable identifier for this MIDI endpoint.
    pub endpoint_id: String,
    /// Human-readable name for this MIDI endpoint.
    pub endpoint_name: String,
    /// Identifier of the device this endpoint belongs to.
    pub device_id: String,
    /// Data-flow direction of this endpoint.
    pub direction: RuntimeExternalMidiEndpointDirection,
    /// Current lifecycle state of this endpoint.
    pub lifecycle_state: RuntimeExternalMidiEndpointLifecycleState,
    /// Current routing state of this endpoint.
    pub route_state: RuntimeExternalMidiRouteState,
    /// Capability flags for this endpoint.
    pub capability: RuntimeExternalMidiEndpointCapabilitySummary,
    /// Human-readable summary of this endpoint descriptor.
    pub summary: String,
}

/// Aggregate snapshot of all external MIDI devices, endpoints, and routing state.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeExternalMidiEndpointGraphSnapshot {
    /// Current discovery phase of the external MIDI device graph.
    pub discovery_state: RuntimeExternalMidiDiscoveryState,
    /// Overall health of the external MIDI routing graph.
    pub graph_state: RuntimeExternalMidiGraphState,
    /// Live MIDI session ownership summary.
    pub live_ownership: RuntimeExternalMidiLiveOwnershipSummary,
    /// Name of the backend provider supplying the device list.
    pub provider_name: String,
    /// Total number of discovered MIDI devices.
    pub device_count: usize,
    /// Total number of discovered MIDI endpoints across all devices.
    pub endpoint_count: usize,
    /// Number of input-direction endpoints.
    pub input_endpoint_count: usize,
    /// Number of output-direction endpoints.
    pub output_endpoint_count: usize,
    /// Number of duplex (bidirectional) endpoints.
    pub duplex_endpoint_count: usize,
    /// Number of endpoints with an active route.
    pub active_route_count: usize,
    /// Number of endpoints with a guarded route.
    pub guarded_route_count: usize,
    /// Per-device descriptors for all discovered MIDI devices.
    pub devices: Vec<RuntimeExternalMidiDeviceDescriptor>,
    /// Per-endpoint descriptors for all discovered MIDI endpoints.
    pub endpoints: Vec<RuntimeExternalMidiEndpointDescriptor>,
    /// Human-readable summary of the snapshot.
    pub summary: String,
}
