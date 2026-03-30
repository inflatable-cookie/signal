use signal_hardware::BackendHealth;

use crate::{
    RuntimeHostAudioStreamState, RuntimeHostIoSummary, RuntimeLinuxAudioBackendIdentity,
    RuntimeLinuxAudioBackendPortabilityBand, RuntimeLinuxBackendOwnershipFallbackState,
    RuntimeLinuxBackendSessionOwnership, RuntimeLinuxBackendSessionRole,
    RuntimeLinuxBackendSessionSnapshot, TransportDispatchState, TransportHeartbeatFreshness,
    TransportSessionState, TransportSessionSummary,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RuntimeJackTransportPosture {
    NotJack,
    Unavailable,
    Detached,
    FollowingExternal,
    RuntimeLed,
    Guarded,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RuntimeJackGraphCoordinationState {
    NotJack,
    Unavailable,
    NotAttached,
    AttachedStable,
    AttachedGuarded,
    Recovering,
    Released,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RuntimeJackClientRole {
    NotJack,
    Unavailable,
    PrimaryAudioIo,
    MonitoringCapable,
    TransportFollower,
    FallbackContinuation,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RuntimeJackGuardedCoordinationState {
    NotJack,
    Unavailable,
    Direct,
    TransportGuarded,
    GraphGuarded,
    Recovering,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeJackCoordinationSnapshot {
    pub backend_identity: RuntimeLinuxAudioBackendIdentity,
    pub backend_name: String,
    pub portability_band: RuntimeLinuxAudioBackendPortabilityBand,
    pub transport_posture: RuntimeJackTransportPosture,
    pub graph_state: RuntimeJackGraphCoordinationState,
    pub client_role: RuntimeJackClientRole,
    pub guarded_state: RuntimeJackGuardedCoordinationState,
    pub device_id: String,
    pub device_name: String,
    pub session_state: TransportSessionState,
    pub currently_attached: bool,
    pub heartbeat_freshness: TransportHeartbeatFreshness,
    pub dispatch_state: TransportDispatchState,
    pub attach_events: usize,
    pub detach_requested_events: usize,
    pub detached_events: usize,
    pub backend_health: BackendHealth,
    pub simulated: bool,
    pub summary: String,
}

impl RuntimeJackCoordinationSnapshot {
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
