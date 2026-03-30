use signal_hardware::BackendHealth;

use crate::{
    RuntimeHostAudioStreamState, RuntimeHostClockDiscontinuityState, RuntimeHostClockDomain,
    RuntimeHostClockDriftState, RuntimeHostClockFallbackState, RuntimeHostClockTransitionState,
    RuntimeHostIoSummary, RuntimeHostLifecycleOwnership, RuntimeHostRestartPolicy,
    RuntimeLinuxAudioBackendIdentity, RuntimeLinuxAudioBackendPortabilityBand,
    RuntimeLinuxBackendDeviceClaimPosture, RuntimeLinuxBackendOwnershipFallbackState,
    RuntimeLinuxBackendSessionLifecycleState, RuntimeLinuxBackendSessionRole,
    RuntimeLinuxBackendSessionSnapshot, RuntimePipeWireAlsaDeviceClaimParity,
    RuntimePipeWireAlsaGuardedParityState, RuntimePipeWireAlsaParitySnapshot,
    RuntimePipeWireAlsaSessionRoleParity, RuntimePipeWireAlsaStreamPolicyParity,
};

impl RuntimePipeWireAlsaParitySnapshot {
    pub fn unavailable() -> Self {
        Self {
            backend_identity: RuntimeLinuxAudioBackendIdentity::Unavailable,
            backend_name: "runtime-unavailable".into(),
            portability_band: RuntimeLinuxAudioBackendPortabilityBand::Unsupported,
            session_role_parity: RuntimePipeWireAlsaSessionRoleParity::Unavailable,
            device_claim_parity: RuntimePipeWireAlsaDeviceClaimParity::Unavailable,
            stream_policy_parity: RuntimePipeWireAlsaStreamPolicyParity::Unavailable,
            guarded_state: RuntimePipeWireAlsaGuardedParityState::Unavailable,
            lifecycle_ownership: RuntimeHostLifecycleOwnership::HostDrivenCallback,
            restart_policy: RuntimeHostRestartPolicy::HostMustRestart,
            clock_domain: RuntimeHostClockDomain::Degraded,
            fallback_state: RuntimeHostClockFallbackState::Unconfigured,
            device_id: "runtime:unavailable".into(),
            device_name: "Unavailable PipeWire/ALSA Parity".into(),
            stream_state: RuntimeHostAudioStreamState::Stopped,
            backend_health: BackendHealth::Healthy,
            simulated: false,
            device_loss_count: 0,
            restart_attempt_count: 0,
            restart_failure_count: 0,
            summary:
                "backend=Unavailable role=Unavailable claim=Unavailable policy=Unavailable guard=Unavailable"
                    .into(),
        }
    }

    pub fn from_host_io(host_io: &RuntimeHostIoSummary) -> Self {
        let linux_session = RuntimeLinuxBackendSessionSnapshot::from_host_io(host_io);
        Self::from_host_io_and_linux_session(host_io, &linux_session)
    }

    pub fn from_host_io_and_linux_session(
        host_io: &RuntimeHostIoSummary,
        linux_session: &RuntimeLinuxBackendSessionSnapshot,
    ) -> Self {
        let backend_identity = linux_session.backend_identity;
        let targets_pipewire_or_alsa = matches!(
            backend_identity,
            RuntimeLinuxAudioBackendIdentity::Alsa | RuntimeLinuxAudioBackendIdentity::PipeWire
        );
        if !targets_pipewire_or_alsa {
            let unavailable = matches!(
                backend_identity,
                RuntimeLinuxAudioBackendIdentity::Unavailable
                    | RuntimeLinuxAudioBackendIdentity::Unsupported
            );
            let session_role_parity = if unavailable {
                RuntimePipeWireAlsaSessionRoleParity::Unavailable
            } else {
                RuntimePipeWireAlsaSessionRoleParity::NotPipeWireOrAlsa
            };
            let device_claim_parity = if unavailable {
                RuntimePipeWireAlsaDeviceClaimParity::Unavailable
            } else {
                RuntimePipeWireAlsaDeviceClaimParity::NotPipeWireOrAlsa
            };
            let stream_policy_parity = if unavailable {
                RuntimePipeWireAlsaStreamPolicyParity::Unavailable
            } else {
                RuntimePipeWireAlsaStreamPolicyParity::NotPipeWireOrAlsa
            };
            let guarded_state = if unavailable {
                RuntimePipeWireAlsaGuardedParityState::Unavailable
            } else {
                RuntimePipeWireAlsaGuardedParityState::NotPipeWireOrAlsa
            };
            return Self {
                backend_identity,
                backend_name: host_io.hardware.backend_name.clone(),
                portability_band: host_io.hardware.linux_backend_portability,
                session_role_parity,
                device_claim_parity,
                stream_policy_parity,
                guarded_state,
                lifecycle_ownership: host_io.clocking.ownership,
                restart_policy: host_io.clocking.restart_policy,
                clock_domain: host_io.clocking.clock_domain,
                fallback_state: host_io.clocking.fallback_state,
                device_id: host_io.hardware.device_id.clone(),
                device_name: host_io.hardware.device_name.clone(),
                stream_state: host_io.audio_pump.stream_state,
                backend_health: host_io.hardware.backend_health,
                simulated: host_io.hardware.simulated,
                device_loss_count: host_io.hardware.device_loss_count,
                restart_attempt_count: host_io.hardware.restart_attempt_count,
                restart_failure_count: host_io.hardware.restart_failure_count,
                summary: format!(
                    "backend={:?} role={:?} claim={:?} policy={:?} guard={:?}",
                    backend_identity,
                    session_role_parity,
                    device_claim_parity,
                    stream_policy_parity,
                    guarded_state
                ),
            };
        }

        let session_role_parity = match linux_session.session_role {
            RuntimeLinuxBackendSessionRole::PrimaryAudioIo => {
                RuntimePipeWireAlsaSessionRoleParity::PrimaryAudioIo
            }
            RuntimeLinuxBackendSessionRole::MonitoringCapable => {
                RuntimePipeWireAlsaSessionRoleParity::MonitoringCapable
            }
            RuntimeLinuxBackendSessionRole::OfflineUnavailable => {
                RuntimePipeWireAlsaSessionRoleParity::OfflineUnavailable
            }
            RuntimeLinuxBackendSessionRole::FallbackContinuation => {
                RuntimePipeWireAlsaSessionRoleParity::FallbackContinuation
            }
            RuntimeLinuxBackendSessionRole::Unavailable => {
                RuntimePipeWireAlsaSessionRoleParity::Unavailable
            }
            RuntimeLinuxBackendSessionRole::NotLinux => {
                RuntimePipeWireAlsaSessionRoleParity::NotPipeWireOrAlsa
            }
        };
        let device_claim_parity = match linux_session.device_claim_posture {
            RuntimeLinuxBackendDeviceClaimPosture::Unclaimed => {
                RuntimePipeWireAlsaDeviceClaimParity::NoClaim
            }
            RuntimeLinuxBackendDeviceClaimPosture::DirectClaim => {
                RuntimePipeWireAlsaDeviceClaimParity::DirectClaim
            }
            RuntimeLinuxBackendDeviceClaimPosture::SharedGraph => {
                RuntimePipeWireAlsaDeviceClaimParity::SharedGraph
            }
            RuntimeLinuxBackendDeviceClaimPosture::Lost => {
                RuntimePipeWireAlsaDeviceClaimParity::Lost
            }
            RuntimeLinuxBackendDeviceClaimPosture::Released => {
                RuntimePipeWireAlsaDeviceClaimParity::Released
            }
            RuntimeLinuxBackendDeviceClaimPosture::Unavailable => {
                RuntimePipeWireAlsaDeviceClaimParity::Unavailable
            }
            RuntimeLinuxBackendDeviceClaimPosture::NotLinux => {
                RuntimePipeWireAlsaDeviceClaimParity::NotPipeWireOrAlsa
            }
        };

        let recovering = matches!(
            linux_session.lifecycle_state,
            RuntimeLinuxBackendSessionLifecycleState::Interrupted
                | RuntimeLinuxBackendSessionLifecycleState::Recovering
        ) || matches!(
            linux_session.ownership_fallback,
            RuntimeLinuxBackendOwnershipFallbackState::Reacquiring
                | RuntimeLinuxBackendOwnershipFallbackState::RecoveryConstrained
        ) || host_io.hardware.restart_attempt_count > 0
            || host_io.hardware.restart_failure_count > 0
            || host_io.hardware.device_loss_count > 0
            || host_io.audio_pump.stream_state == RuntimeHostAudioStreamState::Faulted;
        let unavailable = matches!(
            session_role_parity,
            RuntimePipeWireAlsaSessionRoleParity::OfflineUnavailable
                | RuntimePipeWireAlsaSessionRoleParity::Unavailable
        );
        let stream_policy_parity = if unavailable {
            RuntimePipeWireAlsaStreamPolicyParity::Unavailable
        } else if recovering {
            RuntimePipeWireAlsaStreamPolicyParity::Restarting
        } else {
            match host_io.clocking.ownership {
                RuntimeHostLifecycleOwnership::HostDrivenCallback => {
                    RuntimePipeWireAlsaStreamPolicyParity::DirectHostCallback
                }
                RuntimeHostLifecycleOwnership::BackendManagedCallback => {
                    RuntimePipeWireAlsaStreamPolicyParity::BackendManagedGraph
                }
            }
        };

        let transfer_guarded = host_io.audio_pump.transfer_policy.max_callback_frames
            < host_io.hardware.buffer_size
            || host_io.audio_pump.transfer_policy.max_transfer_channels
                < host_io
                    .hardware
                    .input_channels
                    .max(host_io.hardware.output_channels)
            || !host_io
                .audio_pump
                .transfer_policy
                .zero_fill_unwritten_output;
        let clock_guarded = host_io.clocking.clock_domain != RuntimeHostClockDomain::SameClock
            || host_io.clocking.fallback_state != RuntimeHostClockFallbackState::Direct
            || host_io.clocking.transition_state != RuntimeHostClockTransitionState::Stable
            || host_io.clocking.drift_state != RuntimeHostClockDriftState::Stable
            || host_io.clocking.discontinuity_state
                != RuntimeHostClockDiscontinuityState::Continuous;
        let guarded_state = if unavailable {
            RuntimePipeWireAlsaGuardedParityState::Unavailable
        } else if recovering {
            RuntimePipeWireAlsaGuardedParityState::RecoveryGuarded
        } else if transfer_guarded {
            RuntimePipeWireAlsaGuardedParityState::TransferGuarded
        } else if clock_guarded {
            RuntimePipeWireAlsaGuardedParityState::ClockGuarded
        } else if host_io.clocking.ownership
            == RuntimeHostLifecycleOwnership::BackendManagedCallback
        {
            RuntimePipeWireAlsaGuardedParityState::BackendManaged
        } else {
            RuntimePipeWireAlsaGuardedParityState::Direct
        };

        Self {
            backend_identity,
            backend_name: host_io.hardware.backend_name.clone(),
            portability_band: host_io.hardware.linux_backend_portability,
            session_role_parity,
            device_claim_parity,
            stream_policy_parity,
            guarded_state,
            lifecycle_ownership: host_io.clocking.ownership,
            restart_policy: host_io.clocking.restart_policy,
            clock_domain: host_io.clocking.clock_domain,
            fallback_state: host_io.clocking.fallback_state,
            device_id: host_io.hardware.device_id.clone(),
            device_name: host_io.hardware.device_name.clone(),
            stream_state: host_io.audio_pump.stream_state,
            backend_health: host_io.hardware.backend_health,
            simulated: host_io.hardware.simulated,
            device_loss_count: host_io.hardware.device_loss_count,
            restart_attempt_count: host_io.hardware.restart_attempt_count,
            restart_failure_count: host_io.hardware.restart_failure_count,
            summary: format!(
                "backend={:?} role={:?} claim={:?} policy={:?} guard={:?}",
                backend_identity,
                session_role_parity,
                device_claim_parity,
                stream_policy_parity,
                guarded_state
            ),
        }
    }
}
