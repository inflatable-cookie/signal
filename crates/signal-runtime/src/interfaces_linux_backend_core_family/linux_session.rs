use signal_hardware::BackendHealth;

use crate::{
    RuntimeHostAudioStreamState, RuntimeHostClockFallbackState, RuntimeHostEndpointTopology,
    RuntimeHostIoSummary, RuntimeHostLifecycleOwnership, RuntimeLinuxAudioBackendIdentity,
    RuntimeLinuxAudioBackendPortabilityBand,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RuntimeLinuxBackendSessionOwnership {
    NotLinux,
    Unavailable,
    RuntimeOwnedDirect,
    HostBrokeredCallback,
    BackendManagedGraph,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RuntimeLinuxBackendSessionLifecycleState {
    NotLinux,
    Unavailable,
    Claimable,
    Attached,
    Running,
    Interrupted,
    Recovering,
    Released,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RuntimeLinuxBackendDeviceClaimPosture {
    NotLinux,
    Unavailable,
    Unclaimed,
    DirectClaim,
    SharedGraph,
    Lost,
    Released,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RuntimeLinuxBackendSessionRole {
    NotLinux,
    Unavailable,
    PrimaryAudioIo,
    MonitoringCapable,
    OfflineUnavailable,
    FallbackContinuation,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RuntimeLinuxBackendOwnershipFallbackState {
    NotLinux,
    Unavailable,
    Direct,
    BackendManagedGuarded,
    Reacquiring,
    RecoveryConstrained,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeLinuxBackendSessionSnapshot {
    pub backend_identity: RuntimeLinuxAudioBackendIdentity,
    pub backend_name: String,
    pub portability_band: RuntimeLinuxAudioBackendPortabilityBand,
    pub ownership: RuntimeLinuxBackendSessionOwnership,
    pub lifecycle_state: RuntimeLinuxBackendSessionLifecycleState,
    pub device_claim_posture: RuntimeLinuxBackendDeviceClaimPosture,
    pub session_role: RuntimeLinuxBackendSessionRole,
    pub ownership_fallback: RuntimeLinuxBackendOwnershipFallbackState,
    pub device_id: String,
    pub device_name: String,
    pub stream_state: RuntimeHostAudioStreamState,
    pub backend_health: BackendHealth,
    pub simulated: bool,
    pub device_loss_count: u64,
    pub restart_attempt_count: u64,
    pub restart_failure_count: u64,
    pub summary: String,
}

impl RuntimeLinuxBackendSessionSnapshot {
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
