use signal_hardware::BackendHealth;

use crate::{
    RuntimeHostAudioStreamState, RuntimeHostClockFallbackState, RuntimeHostEndpointTopology,
    RuntimeHostIoSummary, RuntimeHostLifecycleOwnership, RuntimeLinuxAudioBackendIdentity,
    RuntimeLinuxAudioBackendPortabilityBand,
};

/// Who owns the Linux audio backend session.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RuntimeLinuxBackendSessionOwnership {
    /// Running on a non-Linux platform; ownership is not applicable.
    NotLinux,
    /// Session ownership state is unavailable.
    Unavailable,
    /// The runtime owns the Linux audio backend session directly.
    RuntimeOwnedDirect,
    /// Ownership is brokered through a host-driven callback.
    HostBrokeredCallback,
    /// The backend manages the audio graph and owns the session.
    BackendManagedGraph,
}

/// Lifecycle state of the Linux audio backend session.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RuntimeLinuxBackendSessionLifecycleState {
    /// Running on a non-Linux platform; lifecycle state is not applicable.
    NotLinux,
    /// Session lifecycle state is unavailable.
    Unavailable,
    /// Device is available and the session can be claimed.
    Claimable,
    /// Session has been attached but has not yet started running.
    Attached,
    /// Session is running and processing audio.
    Running,
    /// Session was interrupted by a fault or discontinuity.
    Interrupted,
    /// Session is recovering from an interruption.
    Recovering,
    /// Session has been released.
    Released,
}

/// Device claim posture of the Linux audio backend.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RuntimeLinuxBackendDeviceClaimPosture {
    /// Running on a non-Linux platform; device claim posture is not applicable.
    NotLinux,
    /// Device claim posture is unavailable.
    Unavailable,
    /// Device has not been claimed by any session.
    Unclaimed,
    /// Session holds a direct exclusive claim on the device.
    DirectClaim,
    /// Session participates in a shared backend graph.
    SharedGraph,
    /// Device was lost and has not been reacquired.
    Lost,
    /// Device claim has been released.
    Released,
}

/// Functional role of the Linux audio backend session within the runtime graph.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RuntimeLinuxBackendSessionRole {
    /// Running on a non-Linux platform; session role is not applicable.
    NotLinux,
    /// Session role is unavailable.
    Unavailable,
    /// Session is the primary audio I/O path.
    PrimaryAudioIo,
    /// Session is capable of monitoring but not full duplex I/O.
    MonitoringCapable,
    /// Session is unavailable for offline rendering.
    OfflineUnavailable,
    /// Session is serving as a fallback continuation path.
    FallbackContinuation,
}

/// Fallback ownership state when the Linux backend session cannot be directly claimed.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RuntimeLinuxBackendOwnershipFallbackState {
    /// Running on a non-Linux platform; fallback state is not applicable.
    NotLinux,
    /// Ownership fallback state is unavailable.
    Unavailable,
    /// Session has direct ownership with no fallback active.
    Direct,
    /// Backend manages the session; direct ownership is guarded.
    BackendManagedGuarded,
    /// Session is actively reacquiring direct ownership after an interruption.
    Reacquiring,
    /// Ownership is constrained to a recovery fallback mode.
    RecoveryConstrained,
}

/// Full snapshot of a Linux audio backend session: identity, ownership, lifecycle, device claim posture, role, fallback state, and fault counters.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeLinuxBackendSessionSnapshot {
    /// Linux audio backend identity.
    pub backend_identity: RuntimeLinuxAudioBackendIdentity,
    /// Human-readable name of the active backend.
    pub backend_name: String,
    /// Portability band for this backend.
    pub portability_band: RuntimeLinuxAudioBackendPortabilityBand,
    /// Who owns the Linux audio backend session.
    pub ownership: RuntimeLinuxBackendSessionOwnership,
    /// Lifecycle state of the session.
    pub lifecycle_state: RuntimeLinuxBackendSessionLifecycleState,
    /// Device claim posture of the session.
    pub device_claim_posture: RuntimeLinuxBackendDeviceClaimPosture,
    /// Functional role of the session within the runtime graph.
    pub session_role: RuntimeLinuxBackendSessionRole,
    /// Fallback ownership state when direct ownership is not available.
    pub ownership_fallback: RuntimeLinuxBackendOwnershipFallbackState,
    /// Stable identifier for the active audio device.
    pub device_id: String,
    /// Human-readable name of the active audio device.
    pub device_name: String,
    /// Current state of the host audio stream.
    pub stream_state: RuntimeHostAudioStreamState,
    /// Health state reported by the audio backend.
    pub backend_health: BackendHealth,
    /// Whether the backend is operating in simulated mode.
    pub simulated: bool,
    /// Number of times the audio device was lost and had to be reacquired.
    pub device_loss_count: u64,
    /// Number of backend restart attempts.
    pub restart_attempt_count: u64,
    /// Number of backend restart attempts that failed.
    pub restart_failure_count: u64,
    /// Human-readable summary of the session snapshot.
    pub summary: String,
}

impl RuntimeLinuxBackendSessionSnapshot {
    /// Returns a snapshot representing an unavailable Linux backend session.
    pub fn unavailable() -> Self {
        Self {
            backend_identity: RuntimeLinuxAudioBackendIdentity::Unavailable,
            backend_name: "runtime-unavailable".into(),
            portability_band: RuntimeLinuxAudioBackendPortabilityBand::Unsupported,
            ownership: RuntimeLinuxBackendSessionOwnership::Unavailable,
            lifecycle_state: RuntimeLinuxBackendSessionLifecycleState::Unavailable,
            device_claim_posture: RuntimeLinuxBackendDeviceClaimPosture::Unavailable,
            session_role: RuntimeLinuxBackendSessionRole::Unavailable,
            ownership_fallback: RuntimeLinuxBackendOwnershipFallbackState::Unavailable,
            device_id: "runtime:unavailable".into(),
            device_name: "Unavailable Linux Backend Session".into(),
            stream_state: RuntimeHostAudioStreamState::Stopped,
            backend_health: BackendHealth::Healthy,
            simulated: false,
            device_loss_count: 0,
            restart_attempt_count: 0,
            restart_failure_count: 0,
            summary:
                "backend=Unavailable ownership=Unavailable lifecycle=Unavailable claim=Unavailable role=Unavailable fallback=Unavailable"
                    .into(),
        }
    }

    /// Derives a Linux backend session snapshot from the current host I/O summary.
    pub fn from_host_io(host_io: &RuntimeHostIoSummary) -> Self {
        let backend_identity = host_io.hardware.linux_backend_identity;
        if backend_identity == RuntimeLinuxAudioBackendIdentity::NotLinux {
            return Self {
                backend_identity,
                backend_name: host_io.hardware.backend_name.clone(),
                portability_band: host_io.hardware.linux_backend_portability,
                ownership: RuntimeLinuxBackendSessionOwnership::NotLinux,
                lifecycle_state: RuntimeLinuxBackendSessionLifecycleState::NotLinux,
                device_claim_posture: RuntimeLinuxBackendDeviceClaimPosture::NotLinux,
                session_role: RuntimeLinuxBackendSessionRole::NotLinux,
                ownership_fallback: RuntimeLinuxBackendOwnershipFallbackState::NotLinux,
                device_id: host_io.hardware.device_id.clone(),
                device_name: host_io.hardware.device_name.clone(),
                stream_state: host_io.audio_pump.stream_state,
                backend_health: host_io.hardware.backend_health,
                simulated: host_io.hardware.simulated,
                device_loss_count: host_io.hardware.device_loss_count,
                restart_attempt_count: host_io.hardware.restart_attempt_count,
                restart_failure_count: host_io.hardware.restart_failure_count,
                summary: format!(
                    "backend={:?} ownership=NotLinux lifecycle=NotLinux claim=NotLinux role=NotLinux fallback=NotLinux",
                    backend_identity
                ),
            };
        }

        if matches!(
            backend_identity,
            RuntimeLinuxAudioBackendIdentity::Unavailable
                | RuntimeLinuxAudioBackendIdentity::Unsupported
        ) {
            return Self {
                backend_identity,
                backend_name: host_io.hardware.backend_name.clone(),
                portability_band: host_io.hardware.linux_backend_portability,
                ownership: RuntimeLinuxBackendSessionOwnership::Unavailable,
                lifecycle_state: RuntimeLinuxBackendSessionLifecycleState::Unavailable,
                device_claim_posture: RuntimeLinuxBackendDeviceClaimPosture::Unavailable,
                session_role: RuntimeLinuxBackendSessionRole::Unavailable,
                ownership_fallback: RuntimeLinuxBackendOwnershipFallbackState::Unavailable,
                device_id: host_io.hardware.device_id.clone(),
                device_name: host_io.hardware.device_name.clone(),
                stream_state: host_io.audio_pump.stream_state,
                backend_health: host_io.hardware.backend_health,
                simulated: host_io.hardware.simulated,
                device_loss_count: host_io.hardware.device_loss_count,
                restart_attempt_count: host_io.hardware.restart_attempt_count,
                restart_failure_count: host_io.hardware.restart_failure_count,
                summary: format!(
                    "backend={:?} ownership=Unavailable lifecycle=Unavailable claim=Unavailable role=Unavailable fallback=Unavailable",
                    backend_identity
                ),
            };
        }

        let recovering = host_io.hardware.device_loss_count > 0
            || host_io.hardware.restart_attempt_count > 0
            || matches!(
                host_io.hardware.backend_health,
                BackendHealth::Degraded | BackendHealth::Recovering
            )
            || host_io.audio_pump.stream_state == RuntimeHostAudioStreamState::Faulted;
        let release_like = host_io.audio_pump.stream_state == RuntimeHostAudioStreamState::Stopped
            && matches!(
                host_io.clocking.endpoint_topology,
                RuntimeHostEndpointTopology::Unconfigured
            );
        let ownership = if release_like {
            RuntimeLinuxBackendSessionOwnership::Unavailable
        } else {
            match host_io.clocking.ownership {
                RuntimeHostLifecycleOwnership::HostDrivenCallback => {
                    RuntimeLinuxBackendSessionOwnership::HostBrokeredCallback
                }
                RuntimeHostLifecycleOwnership::BackendManagedCallback => {
                    RuntimeLinuxBackendSessionOwnership::BackendManagedGraph
                }
            }
        };
        let lifecycle_state = if release_like {
            RuntimeLinuxBackendSessionLifecycleState::Released
        } else if recovering {
            RuntimeLinuxBackendSessionLifecycleState::Recovering
        } else {
            match host_io.audio_pump.stream_state {
                RuntimeHostAudioStreamState::Running => {
                    RuntimeLinuxBackendSessionLifecycleState::Running
                }
                RuntimeHostAudioStreamState::Stopped => {
                    RuntimeLinuxBackendSessionLifecycleState::Claimable
                }
                RuntimeHostAudioStreamState::Faulted => {
                    RuntimeLinuxBackendSessionLifecycleState::Interrupted
                }
            }
        };
        let device_claim_posture = if release_like {
            RuntimeLinuxBackendDeviceClaimPosture::Released
        } else if host_io.hardware.device_loss_count > 0 {
            RuntimeLinuxBackendDeviceClaimPosture::Lost
        } else if host_io.audio_pump.stream_state == RuntimeHostAudioStreamState::Stopped {
            RuntimeLinuxBackendDeviceClaimPosture::Unclaimed
        } else {
            match host_io.clocking.ownership {
                RuntimeHostLifecycleOwnership::HostDrivenCallback => {
                    RuntimeLinuxBackendDeviceClaimPosture::DirectClaim
                }
                RuntimeHostLifecycleOwnership::BackendManagedCallback => {
                    RuntimeLinuxBackendDeviceClaimPosture::SharedGraph
                }
            }
        };
        let fallback_active =
            host_io.clocking.fallback_state != RuntimeHostClockFallbackState::Direct;
        let session_role = if release_like {
            RuntimeLinuxBackendSessionRole::OfflineUnavailable
        } else if recovering || fallback_active {
            RuntimeLinuxBackendSessionRole::FallbackContinuation
        } else if matches!(
            host_io.clocking.endpoint_topology,
            RuntimeHostEndpointTopology::OutputOnly
        ) {
            RuntimeLinuxBackendSessionRole::MonitoringCapable
        } else {
            RuntimeLinuxBackendSessionRole::PrimaryAudioIo
        };
        let ownership_fallback = if release_like {
            RuntimeLinuxBackendOwnershipFallbackState::Unavailable
        } else if host_io.hardware.restart_failure_count > 0
            || host_io.audio_pump.stream_state == RuntimeHostAudioStreamState::Faulted
        {
            RuntimeLinuxBackendOwnershipFallbackState::RecoveryConstrained
        } else if recovering {
            RuntimeLinuxBackendOwnershipFallbackState::Reacquiring
        } else if host_io.clocking.ownership
            == RuntimeHostLifecycleOwnership::BackendManagedCallback
        {
            RuntimeLinuxBackendOwnershipFallbackState::BackendManagedGuarded
        } else {
            RuntimeLinuxBackendOwnershipFallbackState::Direct
        };

        Self {
            backend_identity,
            backend_name: host_io.hardware.backend_name.clone(),
            portability_band: host_io.hardware.linux_backend_portability,
            ownership,
            lifecycle_state,
            device_claim_posture,
            session_role,
            ownership_fallback,
            device_id: host_io.hardware.device_id.clone(),
            device_name: host_io.hardware.device_name.clone(),
            stream_state: host_io.audio_pump.stream_state,
            backend_health: host_io.hardware.backend_health,
            simulated: host_io.hardware.simulated,
            device_loss_count: host_io.hardware.device_loss_count,
            restart_attempt_count: host_io.hardware.restart_attempt_count,
            restart_failure_count: host_io.hardware.restart_failure_count,
            summary: format!(
                "backend={:?} ownership={:?} lifecycle={:?} claim={:?} role={:?} fallback={:?}",
                backend_identity,
                ownership,
                lifecycle_state,
                device_claim_posture,
                session_role,
                ownership_fallback
            ),
        }
    }
}
