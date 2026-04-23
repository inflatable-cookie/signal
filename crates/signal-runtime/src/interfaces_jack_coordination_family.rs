use signal_hardware::BackendHealth;

use crate::{
    RuntimeHostAudioStreamState, RuntimeHostIoSummary, RuntimeLinuxAudioBackendIdentity,
    RuntimeLinuxAudioBackendPortabilityBand, RuntimeLinuxBackendOwnershipFallbackState,
    RuntimeLinuxBackendSessionOwnership, RuntimeLinuxBackendSessionRole,
    RuntimeLinuxBackendSessionSnapshot, TransportDispatchState, TransportHeartbeatFreshness,
    TransportSessionState, TransportSessionSummary,
};

/// JACK transport integration posture.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RuntimeJackTransportPosture {
    /// Running on a non-JACK backend.
    NotJack,
    /// JACK transport state is unavailable.
    Unavailable,
    /// Not currently attached to a JACK transport session.
    Detached,
    /// Following an external JACK transport master.
    FollowingExternal,
    /// Runtime is leading the JACK transport.
    RuntimeLed,
    /// JACK transport integration is in a guarded state.
    Guarded,
}

/// State of JACK graph coordination.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RuntimeJackGraphCoordinationState {
    /// Running on a non-JACK backend.
    NotJack,
    /// JACK graph coordination state is unavailable.
    Unavailable,
    /// JACK client is not currently attached to the graph.
    NotAttached,
    /// JACK client is attached and the graph is stable.
    AttachedStable,
    /// JACK client is attached but the graph is in a guarded state.
    AttachedGuarded,
    /// JACK client is recovering from an interruption.
    Recovering,
    /// JACK client has been released from the graph.
    Released,
}

/// Role of the runtime as a JACK client.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RuntimeJackClientRole {
    /// Running on a non-JACK backend.
    NotJack,
    /// JACK client role is unavailable.
    Unavailable,
    /// Client is the primary audio I/O path in the JACK graph.
    PrimaryAudioIo,
    /// Client is capable of monitoring but not full duplex I/O.
    MonitoringCapable,
    /// Client is following an external JACK transport master.
    TransportFollower,
    /// Client is serving as a fallback continuation path.
    FallbackContinuation,
}

/// Guarded coordination state of the JACK client.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RuntimeJackGuardedCoordinationState {
    /// Running on a non-JACK backend.
    NotJack,
    /// Guarded coordination state is unavailable.
    Unavailable,
    /// JACK client has direct coordination with no guarded constraint.
    Direct,
    /// JACK transport session is introducing a guarded constraint.
    TransportGuarded,
    /// JACK graph attachment is in a guarded state.
    GraphGuarded,
    /// JACK client is recovering from an interruption.
    Recovering,
}

/// Full JACK coordination snapshot: transport posture, graph state, client
/// role, guarded state, session attachment, and heartbeat freshness.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeJackCoordinationSnapshot {
    /// Linux audio backend identity (expected to be JACK).
    pub backend_identity: RuntimeLinuxAudioBackendIdentity,
    /// Human-readable name of the active backend.
    pub backend_name: String,
    /// Portability band for this backend.
    pub portability_band: RuntimeLinuxAudioBackendPortabilityBand,
    /// JACK transport integration posture.
    pub transport_posture: RuntimeJackTransportPosture,
    /// State of JACK graph coordination.
    pub graph_state: RuntimeJackGraphCoordinationState,
    /// Role of the runtime as a JACK client.
    pub client_role: RuntimeJackClientRole,
    /// Guarded coordination constraint summary.
    pub guarded_state: RuntimeJackGuardedCoordinationState,
    /// Stable identifier for the active audio device.
    pub device_id: String,
    /// Human-readable name of the active audio device.
    pub device_name: String,
    /// Current JACK transport session state.
    pub session_state: TransportSessionState,
    /// Whether the JACK transport session is currently attached.
    pub currently_attached: bool,
    /// Freshness of the most recent transport heartbeat.
    pub heartbeat_freshness: TransportHeartbeatFreshness,
    /// Current transport dispatch state.
    pub dispatch_state: TransportDispatchState,
    /// Number of times the JACK transport session was attached.
    pub attach_events: usize,
    /// Number of times a JACK transport detach was requested.
    pub detach_requested_events: usize,
    /// Number of times the JACK transport session was detached.
    pub detached_events: usize,
    /// Health state reported by the JACK backend.
    pub backend_health: BackendHealth,
    /// Whether the backend is operating in simulated mode.
    pub simulated: bool,
    /// Human-readable summary of the coordination snapshot.
    pub summary: String,
}

impl RuntimeJackCoordinationSnapshot {
    /// Returns a snapshot representing unavailable JACK coordination.
    pub fn unavailable() -> Self {
        Self {
            backend_identity: RuntimeLinuxAudioBackendIdentity::Unavailable,
            backend_name: "runtime-unavailable".into(),
            portability_band: RuntimeLinuxAudioBackendPortabilityBand::Unsupported,
            transport_posture: RuntimeJackTransportPosture::Unavailable,
            graph_state: RuntimeJackGraphCoordinationState::Unavailable,
            client_role: RuntimeJackClientRole::Unavailable,
            guarded_state: RuntimeJackGuardedCoordinationState::Unavailable,
            device_id: "runtime:unavailable".into(),
            device_name: "Unavailable JACK Coordination".into(),
            session_state: TransportSessionState::Detached,
            currently_attached: false,
            heartbeat_freshness: TransportHeartbeatFreshness::Unknown,
            dispatch_state: TransportDispatchState::Idle,
            attach_events: 0,
            detach_requested_events: 0,
            detached_events: 0,
            backend_health: BackendHealth::Healthy,
            simulated: false,
            summary:
                "backend=Unavailable transport=Unavailable graph=Unavailable role=Unavailable guard=Unavailable"
                    .into(),
        }
    }

    /// Derives the JACK coordination snapshot from host I/O state and a JACK transport session summary.
    pub fn from_host_io_and_transport_session(
        host_io: &RuntimeHostIoSummary,
        transport_session: &TransportSessionSummary,
    ) -> Self {
        let linux_session = RuntimeLinuxBackendSessionSnapshot::from_host_io(host_io);
        let backend_identity = host_io.hardware.linux_backend_identity;

        if backend_identity != RuntimeLinuxAudioBackendIdentity::Jack {
            let unavailable = matches!(
                backend_identity,
                RuntimeLinuxAudioBackendIdentity::Unavailable
                    | RuntimeLinuxAudioBackendIdentity::Unsupported
            );
            let transport_posture = if unavailable {
                RuntimeJackTransportPosture::Unavailable
            } else {
                RuntimeJackTransportPosture::NotJack
            };
            let graph_state = if unavailable {
                RuntimeJackGraphCoordinationState::Unavailable
            } else {
                RuntimeJackGraphCoordinationState::NotJack
            };
            let client_role = if unavailable {
                RuntimeJackClientRole::Unavailable
            } else {
                RuntimeJackClientRole::NotJack
            };
            let guarded_state = if unavailable {
                RuntimeJackGuardedCoordinationState::Unavailable
            } else {
                RuntimeJackGuardedCoordinationState::NotJack
            };
            return Self {
                backend_identity,
                backend_name: host_io.hardware.backend_name.clone(),
                portability_band: host_io.hardware.linux_backend_portability,
                transport_posture,
                graph_state,
                client_role,
                guarded_state,
                device_id: host_io.hardware.device_id.clone(),
                device_name: host_io.hardware.device_name.clone(),
                session_state: TransportSessionState::Detached,
                currently_attached: false,
                heartbeat_freshness: TransportHeartbeatFreshness::Unknown,
                dispatch_state: TransportDispatchState::Idle,
                attach_events: 0,
                detach_requested_events: 0,
                detached_events: 0,
                backend_health: host_io.hardware.backend_health,
                simulated: host_io.hardware.simulated,
                summary: format!(
                    "backend={:?} transport={:?} graph={:?} role={:?} guard={:?}",
                    backend_identity, transport_posture, graph_state, client_role, guarded_state
                ),
            };
        }

        let recovering = host_io.audio_pump.stream_state == RuntimeHostAudioStreamState::Faulted
            || matches!(
                host_io.hardware.backend_health,
                BackendHealth::Degraded | BackendHealth::Recovering
            )
            || host_io.hardware.device_loss_count > 0
            || host_io.hardware.restart_attempt_count > 0
            || host_io.hardware.restart_failure_count > 0;
        let graph_attached = matches!(
            host_io.audio_pump.stream_state,
            RuntimeHostAudioStreamState::Running | RuntimeHostAudioStreamState::Stopped
        );
        let released = !graph_attached
            && !transport_session.currently_attached
            && transport_session.attach_events > 0
            && transport_session.detached_events > 0;

        let transport_posture = if recovering {
            RuntimeJackTransportPosture::Guarded
        } else if !transport_session.currently_attached {
            RuntimeJackTransportPosture::Detached
        } else if matches!(
            transport_session.dispatch_state,
            TransportDispatchState::Requested | TransportDispatchState::Completed
        ) || matches!(
            transport_session.heartbeat_freshness,
            TransportHeartbeatFreshness::Requested | TransportHeartbeatFreshness::Fresh
        ) {
            RuntimeJackTransportPosture::FollowingExternal
        } else {
            RuntimeJackTransportPosture::Guarded
        };

        let graph_state = if recovering {
            RuntimeJackGraphCoordinationState::Recovering
        } else if released {
            RuntimeJackGraphCoordinationState::Released
        } else if !graph_attached {
            RuntimeJackGraphCoordinationState::NotAttached
        } else if linux_session.ownership
            == RuntimeLinuxBackendSessionOwnership::BackendManagedGraph
            || linux_session.ownership_fallback
                == RuntimeLinuxBackendOwnershipFallbackState::BackendManagedGuarded
            || host_io.hardware.linux_backend_portability
                == RuntimeLinuxAudioBackendPortabilityBand::Guarded
        {
            RuntimeJackGraphCoordinationState::AttachedGuarded
        } else {
            RuntimeJackGraphCoordinationState::AttachedStable
        };

        let client_role = if linux_session.session_role
            == RuntimeLinuxBackendSessionRole::FallbackContinuation
        {
            RuntimeJackClientRole::FallbackContinuation
        } else if transport_session.currently_attached {
            RuntimeJackClientRole::TransportFollower
        } else if linux_session.session_role == RuntimeLinuxBackendSessionRole::MonitoringCapable {
            RuntimeJackClientRole::MonitoringCapable
        } else {
            RuntimeJackClientRole::PrimaryAudioIo
        };

        let guarded_state = if recovering {
            RuntimeJackGuardedCoordinationState::Recovering
        } else if transport_session.currently_attached {
            RuntimeJackGuardedCoordinationState::TransportGuarded
        } else if graph_state == RuntimeJackGraphCoordinationState::AttachedGuarded {
            RuntimeJackGuardedCoordinationState::GraphGuarded
        } else {
            RuntimeJackGuardedCoordinationState::Direct
        };

        Self {
            backend_identity,
            backend_name: host_io.hardware.backend_name.clone(),
            portability_band: host_io.hardware.linux_backend_portability,
            transport_posture,
            graph_state,
            client_role,
            guarded_state,
            device_id: host_io.hardware.device_id.clone(),
            device_name: host_io.hardware.device_name.clone(),
            session_state: transport_session.current_state,
            currently_attached: transport_session.currently_attached,
            heartbeat_freshness: transport_session.heartbeat_freshness,
            dispatch_state: transport_session.dispatch_state,
            attach_events: transport_session.attach_events,
            detach_requested_events: transport_session.detach_requested_events,
            detached_events: transport_session.detached_events,
            backend_health: host_io.hardware.backend_health,
            simulated: host_io.hardware.simulated,
            summary: format!(
                "backend={:?} transport={:?} graph={:?} role={:?} guard={:?} session={:?}/{} heartbeat={:?} dispatch={:?}",
                backend_identity,
                transport_posture,
                graph_state,
                client_role,
                guarded_state,
                transport_session.current_state,
                transport_session.currently_attached,
                transport_session.heartbeat_freshness,
                transport_session.dispatch_state,
            ),
        }
    }
}
