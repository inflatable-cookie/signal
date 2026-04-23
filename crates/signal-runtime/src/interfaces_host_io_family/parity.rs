use signal_hardware::BackendHealth;

use crate::{
    RuntimeHostAudioStreamState, RuntimeHostClockDiscontinuityState, RuntimeHostClockDomain,
    RuntimeHostClockDriftState, RuntimeHostClockFallbackState, RuntimeHostClockTransitionState,
    RuntimeHostDuplexMismatchState, RuntimeHostEndpointTopology, RuntimeHostIoSummary,
    RuntimeLinuxAudioBackendClockingParityBand, RuntimeLinuxAudioBackendDuplexParityState,
    RuntimeLinuxAudioBackendEndpointTopologyParityState, RuntimeLinuxAudioBackendIdentity,
};

/// Flattened input bundle used by `RuntimeHostIoSummary` to classify Linux clocking, duplex, and endpoint topology parity bands.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RuntimeLinuxHostIoParityInput {
    /// Linux audio backend identity classification.
    pub linux_backend_identity: RuntimeLinuxAudioBackendIdentity,
    /// Health state reported by the audio backend.
    pub backend_health: BackendHealth,
    /// Current state of the host audio stream.
    pub stream_state: RuntimeHostAudioStreamState,
    /// Clock domain relationship between the host and runtime.
    pub clock_domain: RuntimeHostClockDomain,
    /// Active clock fallback mode.
    pub fallback_state: RuntimeHostClockFallbackState,
    /// Most recent clock topology transition event.
    pub transition_state: RuntimeHostClockTransitionState,
    /// Clock drift management state.
    pub drift_state: RuntimeHostClockDriftState,
    /// Whether the stream has experienced a clock discontinuity.
    pub discontinuity_state: RuntimeHostClockDiscontinuityState,
    /// Duplex alignment state between input and output endpoints.
    pub duplex_mismatch_state: RuntimeHostDuplexMismatchState,
    /// I/O endpoint topology of the active backend.
    pub endpoint_topology: RuntimeHostEndpointTopology,
    /// Whether only partial I/O availability is present.
    pub partial_availability: bool,
}

impl RuntimeHostIoSummary {
    /// Constructs a [`RuntimeLinuxHostIoParityInput`] bundle from individual classification inputs.
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

    /// Classifies the Linux audio backend clocking parity band from the parity input bundle.
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

    /// Classifies the Linux audio backend duplex parity state from the parity input bundle.
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

    /// Classifies the Linux audio backend endpoint topology parity state from individual inputs.
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
