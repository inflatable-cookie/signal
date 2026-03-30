use crate::{
    RuntimeExternalMidiAttachContinuity, RuntimeExternalMidiBackendParity,
    RuntimeExternalMidiEndpointGraphSnapshot, RuntimeExternalMidiGraphState,
    RuntimeExternalMidiGuardedParityOutcome, RuntimeExternalMidiLiveOwnershipPosture,
    RuntimeExternalMidiLiveOwnershipSummary, RuntimeInterruptionClass, RuntimeInterruptionSummary,
    RuntimeLinuxAudioBackendIdentity, RuntimeLinuxAudioBackendPortabilityBand,
    RuntimeLinuxBackendOwnershipFallbackState, RuntimeLinuxBackendSessionOwnership,
    RuntimeLinuxBackendSessionSnapshot,
};

impl RuntimeExternalMidiLiveOwnershipSummary {
    pub fn unavailable() -> Self {
        Self {
            ownership_posture: RuntimeExternalMidiLiveOwnershipPosture::Unavailable,
            attach_continuity: RuntimeExternalMidiAttachContinuity::Unavailable,
            backend_parity: RuntimeExternalMidiBackendParity::Unavailable,
            guarded_parity_outcome: RuntimeExternalMidiGuardedParityOutcome::Unavailable,
            backend_identity: RuntimeLinuxAudioBackendIdentity::Unavailable,
            device_loss_count: 0,
            restart_attempt_count: 0,
            restart_failure_count: 0,
            summary:
                "ownership=Unavailable continuity=Unavailable parity=Unavailable guarded=Unavailable backend=Unavailable"
                    .into(),
        }
    }

    pub fn detached_without_backend_context() -> Self {
        Self {
            ownership_posture: RuntimeExternalMidiLiveOwnershipPosture::NoLiveOwnership,
            attach_continuity: RuntimeExternalMidiAttachContinuity::Detached,
            backend_parity: RuntimeExternalMidiBackendParity::Unavailable,
            guarded_parity_outcome: RuntimeExternalMidiGuardedParityOutcome::Unavailable,
            backend_identity: RuntimeLinuxAudioBackendIdentity::Unavailable,
            device_loss_count: 0,
            restart_attempt_count: 0,
            restart_failure_count: 0,
            summary:
                "ownership=NoLiveOwnership continuity=Detached parity=Unavailable guarded=Unavailable backend=Unavailable"
                    .into(),
        }
    }

    pub fn from_linux_session_and_interruption(
        linux_session: &RuntimeLinuxBackendSessionSnapshot,
        interruption_summary: &RuntimeInterruptionSummary,
        graph_state: RuntimeExternalMidiGraphState,
        device_count: usize,
        endpoint_count: usize,
    ) -> Self {
        let backend_identity = linux_session.backend_identity;
        let backend_parity = match backend_identity {
            RuntimeLinuxAudioBackendIdentity::NotLinux => {
                RuntimeExternalMidiBackendParity::NotLinux
            }
            RuntimeLinuxAudioBackendIdentity::Unavailable
            | RuntimeLinuxAudioBackendIdentity::Unsupported => {
                RuntimeExternalMidiBackendParity::Unavailable
            }
            RuntimeLinuxAudioBackendIdentity::Alsa
            | RuntimeLinuxAudioBackendIdentity::Jack
            | RuntimeLinuxAudioBackendIdentity::PipeWire => {
                if linux_session.portability_band
                    == RuntimeLinuxAudioBackendPortabilityBand::Portable
                {
                    RuntimeExternalMidiBackendParity::Portable
                } else {
                    RuntimeExternalMidiBackendParity::Guarded
                }
            }
        };
        let guarded_parity_outcome = match backend_parity {
            RuntimeExternalMidiBackendParity::NotLinux => {
                RuntimeExternalMidiGuardedParityOutcome::NotLinux
            }
            RuntimeExternalMidiBackendParity::Unavailable => {
                RuntimeExternalMidiGuardedParityOutcome::Unavailable
            }
            RuntimeExternalMidiBackendParity::Portable => {
                RuntimeExternalMidiGuardedParityOutcome::Direct
            }
            RuntimeExternalMidiBackendParity::Guarded => match linux_session.ownership_fallback {
                RuntimeLinuxBackendOwnershipFallbackState::BackendManagedGuarded => {
                    RuntimeExternalMidiGuardedParityOutcome::BackendManaged
                }
                RuntimeLinuxBackendOwnershipFallbackState::Reacquiring
                | RuntimeLinuxBackendOwnershipFallbackState::RecoveryConstrained => {
                    RuntimeExternalMidiGuardedParityOutcome::RecoveryGuarded
                }
                RuntimeLinuxBackendOwnershipFallbackState::Direct => {
                    RuntimeExternalMidiGuardedParityOutcome::Direct
                }
                RuntimeLinuxBackendOwnershipFallbackState::NotLinux => {
                    RuntimeExternalMidiGuardedParityOutcome::NotLinux
                }
                RuntimeLinuxBackendOwnershipFallbackState::Unavailable => {
                    RuntimeExternalMidiGuardedParityOutcome::Unavailable
                }
            },
        };
        let attach_continuity = match backend_parity {
            RuntimeExternalMidiBackendParity::Unavailable => {
                RuntimeExternalMidiAttachContinuity::Unavailable
            }
            _ if matches!(
                interruption_summary.class,
                RuntimeInterruptionClass::Terminal
            ) || graph_state == RuntimeExternalMidiGraphState::Faulted =>
            {
                RuntimeExternalMidiAttachContinuity::Terminal
            }
            _ if endpoint_count == 0 || graph_state == RuntimeExternalMidiGraphState::Empty => {
                RuntimeExternalMidiAttachContinuity::Detached
            }
            _ => match interruption_summary.class {
                RuntimeInterruptionClass::Steady => RuntimeExternalMidiAttachContinuity::Attached,
                RuntimeInterruptionClass::Resumable => {
                    RuntimeExternalMidiAttachContinuity::Resumable
                }
                RuntimeInterruptionClass::Restartable | RuntimeInterruptionClass::Recoverable => {
                    RuntimeExternalMidiAttachContinuity::Restartable
                }
                RuntimeInterruptionClass::Terminal => RuntimeExternalMidiAttachContinuity::Terminal,
            },
        };
        let ownership_posture = match backend_parity {
            RuntimeExternalMidiBackendParity::Unavailable => {
                RuntimeExternalMidiLiveOwnershipPosture::Unavailable
            }
            _ if device_count == 0 || endpoint_count == 0 => {
                RuntimeExternalMidiLiveOwnershipPosture::NoLiveOwnership
            }
            _ if linux_session.ownership
                == RuntimeLinuxBackendSessionOwnership::BackendManagedGraph =>
            {
                RuntimeExternalMidiLiveOwnershipPosture::BackendAdvisoryLiveOwnership
            }
            _ if guarded_parity_outcome != RuntimeExternalMidiGuardedParityOutcome::Direct
                || matches!(
                    interruption_summary.class,
                    RuntimeInterruptionClass::Resumable
                        | RuntimeInterruptionClass::Restartable
                        | RuntimeInterruptionClass::Recoverable
                        | RuntimeInterruptionClass::Terminal
                ) =>
            {
                RuntimeExternalMidiLiveOwnershipPosture::GuardedLiveOwnership
            }
            _ => RuntimeExternalMidiLiveOwnershipPosture::RuntimeDeclaredLiveOwnership,
        };

        let mut summary = Self {
            ownership_posture,
            attach_continuity,
            backend_parity,
            guarded_parity_outcome,
            backend_identity,
            device_loss_count: linux_session.device_loss_count,
            restart_attempt_count: linux_session.restart_attempt_count,
            restart_failure_count: linux_session.restart_failure_count,
            summary: String::new(),
        };
        summary.summary = format!(
            "ownership={:?} continuity={:?} parity={:?} guarded={:?} backend={:?} device_losses={} restart_attempts={} restart_failures={}",
            summary.ownership_posture,
            summary.attach_continuity,
            summary.backend_parity,
            summary.guarded_parity_outcome,
            summary.backend_identity,
            summary.device_loss_count,
            summary.restart_attempt_count,
            summary.restart_failure_count,
        );
        summary
    }
}

impl RuntimeExternalMidiEndpointGraphSnapshot {
    pub fn unavailable() -> Self {
        Self {
            discovery_state: crate::RuntimeExternalMidiDiscoveryState::Unavailable,
            graph_state: RuntimeExternalMidiGraphState::Unavailable,
            live_ownership: RuntimeExternalMidiLiveOwnershipSummary::unavailable(),
            provider_name: "runtime-unavailable".into(),
            device_count: 0,
            endpoint_count: 0,
            input_endpoint_count: 0,
            output_endpoint_count: 0,
            duplex_endpoint_count: 0,
            active_route_count: 0,
            guarded_route_count: 0,
            devices: Vec::new(),
            endpoints: Vec::new(),
            summary: "discovery=Unavailable graph=Unavailable ownership=Unavailable continuity=Unavailable parity=Unavailable provider=runtime-unavailable devices=0 endpoints=0 routes=0".into(),
        }
    }

    pub fn empty(provider_name: impl Into<String>) -> Self {
        let provider_name = provider_name.into();
        Self {
            discovery_state: crate::RuntimeExternalMidiDiscoveryState::Idle,
            graph_state: RuntimeExternalMidiGraphState::Empty,
            live_ownership:
                RuntimeExternalMidiLiveOwnershipSummary::detached_without_backend_context(),
            provider_name: provider_name.clone(),
            device_count: 0,
            endpoint_count: 0,
            input_endpoint_count: 0,
            output_endpoint_count: 0,
            duplex_endpoint_count: 0,
            active_route_count: 0,
            guarded_route_count: 0,
            devices: Vec::new(),
            endpoints: Vec::new(),
            summary: format!(
                "discovery=Idle graph=Empty ownership=NoLiveOwnership continuity=Detached parity=Unavailable provider={} devices=0 endpoints=0 routes=0",
                provider_name,
            ),
        }
    }

    pub fn with_live_ownership_summary(
        mut self,
        linux_session: &RuntimeLinuxBackendSessionSnapshot,
        interruption_summary: &RuntimeInterruptionSummary,
    ) -> Self {
        self.live_ownership =
            RuntimeExternalMidiLiveOwnershipSummary::from_linux_session_and_interruption(
                linux_session,
                interruption_summary,
                self.graph_state,
                self.device_count,
                self.endpoint_count,
            );
        self.summary = format!(
            "discovery={:?} graph={:?} ownership={:?} continuity={:?} parity={:?} provider={} devices={} endpoints={} routes={}/{}",
            self.discovery_state,
            self.graph_state,
            self.live_ownership.ownership_posture,
            self.live_ownership.attach_continuity,
            self.live_ownership.backend_parity,
            self.provider_name,
            self.device_count,
            self.endpoint_count,
            self.active_route_count,
            self.guarded_route_count,
        );
        self
    }
}
