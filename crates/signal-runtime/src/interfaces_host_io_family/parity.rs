use signal_hardware::BackendHealth;

use crate::{
    RuntimeHostAudioStreamState, RuntimeHostClockDiscontinuityState, RuntimeHostClockDomain,
    RuntimeHostClockDriftState, RuntimeHostClockFallbackState, RuntimeHostClockTransitionState,
    RuntimeHostDuplexMismatchState, RuntimeHostEndpointTopology, RuntimeHostIoSummary,
    RuntimeLinuxAudioBackendClockingParityBand, RuntimeLinuxAudioBackendDuplexParityState,
    RuntimeLinuxAudioBackendEndpointTopologyParityState, RuntimeLinuxAudioBackendIdentity,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RuntimeLinuxHostIoParityInput {
    pub linux_backend_identity: RuntimeLinuxAudioBackendIdentity,
    pub backend_health: BackendHealth,
    pub stream_state: RuntimeHostAudioStreamState,
    pub clock_domain: RuntimeHostClockDomain,
    pub fallback_state: RuntimeHostClockFallbackState,
    pub transition_state: RuntimeHostClockTransitionState,
    pub drift_state: RuntimeHostClockDriftState,
    pub discontinuity_state: RuntimeHostClockDiscontinuityState,
    pub duplex_mismatch_state: RuntimeHostDuplexMismatchState,
    pub endpoint_topology: RuntimeHostEndpointTopology,
    pub partial_availability: bool,
}

impl RuntimeHostIoSummary {
    #[allow(clippy::too_many_arguments)]
    pub fn linux_parity_input(
        linux_backend_identity: RuntimeLinuxAudioBackendIdentity,
        backend_health: BackendHealth,
        stream_state: RuntimeHostAudioStreamState,
        clock_domain: RuntimeHostClockDomain,
        fallback_state: RuntimeHostClockFallbackState,
        transition_state: RuntimeHostClockTransitionState,
        drift_state: RuntimeHostClockDriftState,
        discontinuity_state: RuntimeHostClockDiscontinuityState,
        duplex_mismatch_state: RuntimeHostDuplexMismatchState,
        endpoint_topology: RuntimeHostEndpointTopology,
        partial_availability: bool,
    ) -> RuntimeLinuxHostIoParityInput {
        RuntimeLinuxHostIoParityInput {
            linux_backend_identity,
            backend_health,
            stream_state,
            clock_domain,
            fallback_state,
            transition_state,
            drift_state,
            discontinuity_state,
            duplex_mismatch_state,
            endpoint_topology,
            partial_availability,
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn classify_linux_clocking_parity(
        parity: RuntimeLinuxHostIoParityInput,
    ) -> RuntimeLinuxAudioBackendClockingParityBand {
        match parity.linux_backend_identity {
            RuntimeLinuxAudioBackendIdentity::Alsa
            | RuntimeLinuxAudioBackendIdentity::Jack
            | RuntimeLinuxAudioBackendIdentity::PipeWire => {
                if !matches!(parity.backend_health, BackendHealth::Healthy)
                    || parity.stream_state == RuntimeHostAudioStreamState::Faulted
                    || parity.clock_domain != RuntimeHostClockDomain::SameClock
                    || parity.fallback_state != RuntimeHostClockFallbackState::Direct
                    || parity.transition_state != RuntimeHostClockTransitionState::Stable
                    || parity.drift_state != RuntimeHostClockDriftState::Stable
                    || parity.discontinuity_state != RuntimeHostClockDiscontinuityState::Continuous
                {
                    RuntimeLinuxAudioBackendClockingParityBand::Guarded
                } else {
                    RuntimeLinuxAudioBackendClockingParityBand::Portable
                }
            }
            RuntimeLinuxAudioBackendIdentity::NotLinux
            | RuntimeLinuxAudioBackendIdentity::Unavailable
            | RuntimeLinuxAudioBackendIdentity::Unsupported => {
                RuntimeLinuxAudioBackendClockingParityBand::Unsupported
            }
        }
    }

    pub fn classify_linux_duplex_parity(
        parity: RuntimeLinuxHostIoParityInput,
    ) -> RuntimeLinuxAudioBackendDuplexParityState {
        match parity.linux_backend_identity {
            RuntimeLinuxAudioBackendIdentity::Alsa
            | RuntimeLinuxAudioBackendIdentity::Jack
            | RuntimeLinuxAudioBackendIdentity::PipeWire => {
                if matches!(
                    parity.endpoint_topology,
                    RuntimeHostEndpointTopology::Unconfigured
                ) {
                    RuntimeLinuxAudioBackendDuplexParityState::Unsupported
                } else if parity.partial_availability
                    || matches!(
                        parity.endpoint_topology,
                        RuntimeHostEndpointTopology::OutputOnly
                            | RuntimeHostEndpointTopology::InputOnly
                    )
                {
                    RuntimeLinuxAudioBackendDuplexParityState::Partial
                } else if !matches!(parity.backend_health, BackendHealth::Healthy)
                    || parity.stream_state == RuntimeHostAudioStreamState::Faulted
                    || parity.clock_domain != RuntimeHostClockDomain::SameClock
                    || parity.fallback_state != RuntimeHostClockFallbackState::Direct
                    || parity.transition_state != RuntimeHostClockTransitionState::Stable
                    || !matches!(
                        parity.duplex_mismatch_state,
                        RuntimeHostDuplexMismatchState::NotApplicable
                            | RuntimeHostDuplexMismatchState::Aligned
                    )
                    || parity.endpoint_topology == RuntimeHostEndpointTopology::Aggregate
                {
                    RuntimeLinuxAudioBackendDuplexParityState::Guarded
                } else {
                    RuntimeLinuxAudioBackendDuplexParityState::Aligned
                }
            }
            RuntimeLinuxAudioBackendIdentity::NotLinux
            | RuntimeLinuxAudioBackendIdentity::Unavailable
            | RuntimeLinuxAudioBackendIdentity::Unsupported => {
                RuntimeLinuxAudioBackendDuplexParityState::Unsupported
            }
        }
    }

    pub fn classify_linux_endpoint_topology_parity(
        linux_backend_identity: RuntimeLinuxAudioBackendIdentity,
        backend_health: BackendHealth,
        transition_state: RuntimeHostClockTransitionState,
        discontinuity_state: RuntimeHostClockDiscontinuityState,
        duplex_mismatch_state: RuntimeHostDuplexMismatchState,
        endpoint_topology: RuntimeHostEndpointTopology,
        partial_availability: bool,
    ) -> RuntimeLinuxAudioBackendEndpointTopologyParityState {
        match linux_backend_identity {
            RuntimeLinuxAudioBackendIdentity::Alsa
            | RuntimeLinuxAudioBackendIdentity::Jack
            | RuntimeLinuxAudioBackendIdentity::PipeWire => {
                if endpoint_topology == RuntimeHostEndpointTopology::Unconfigured {
                    RuntimeLinuxAudioBackendEndpointTopologyParityState::Unsupported
                } else if partial_availability {
                    RuntimeLinuxAudioBackendEndpointTopologyParityState::Partial
                } else if !matches!(backend_health, BackendHealth::Healthy)
                    || transition_state != RuntimeHostClockTransitionState::Stable
                    || discontinuity_state != RuntimeHostClockDiscontinuityState::Continuous
                    || endpoint_topology == RuntimeHostEndpointTopology::Aggregate
                    || duplex_mismatch_state == RuntimeHostDuplexMismatchState::CrossClockDiverged
                {
                    RuntimeLinuxAudioBackendEndpointTopologyParityState::Guarded
                } else {
                    RuntimeLinuxAudioBackendEndpointTopologyParityState::Portable
                }
            }
            RuntimeLinuxAudioBackendIdentity::NotLinux
            | RuntimeLinuxAudioBackendIdentity::Unavailable
            | RuntimeLinuxAudioBackendIdentity::Unsupported => {
                RuntimeLinuxAudioBackendEndpointTopologyParityState::Unsupported
            }
        }
    }

    pub(crate) fn restart_failure_count(&self) -> u64 {
        self.hardware.restart_failure_count
    }
}
