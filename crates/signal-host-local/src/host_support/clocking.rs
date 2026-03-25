use signal_hardware::{
    AudioStreamDirection, BackendHealth, HardwareClockTopology, HardwareStreamConfig,
};
use signal_runtime::{
    RuntimeHostAudioStreamState, RuntimeHostClockDiscontinuityState, RuntimeHostClockDomain,
    RuntimeHostClockDriftState, RuntimeHostClockFallbackState, RuntimeHostClockTransitionState,
    RuntimeHostDuplexMismatchState, RuntimeHostEndpointTopology,
};

pub(crate) fn samples_to_ms(samples: u32, sample_rate: u32) -> f32 {
    if sample_rate == 0 {
        return 0.0;
    }
    samples as f32 / sample_rate as f32 * 1000.0
}

pub(crate) fn host_clock_domain(
    clock_topology: Option<HardwareClockTopology>,
    processing_sample_rate_hz: u32,
    hardware_sample_rate_hz: u32,
    backend_health: BackendHealth,
) -> RuntimeHostClockDomain {
    if backend_health != BackendHealth::Healthy {
        return RuntimeHostClockDomain::Degraded;
    }
    if matches!(clock_topology, Some(HardwareClockTopology::Aggregate)) {
        return RuntimeHostClockDomain::Aggregate;
    }
    if processing_sample_rate_hz != hardware_sample_rate_hz {
        return RuntimeHostClockDomain::CrossClock;
    }
    RuntimeHostClockDomain::SameClock
}

pub(crate) fn host_clock_fallback_state(
    configured_stream: bool,
    clock_domain: RuntimeHostClockDomain,
    backend_health: BackendHealth,
) -> RuntimeHostClockFallbackState {
    if !configured_stream {
        return RuntimeHostClockFallbackState::Unconfigured;
    }
    if backend_health != BackendHealth::Healthy {
        return RuntimeHostClockFallbackState::RecoveryConstrained;
    }
    if clock_domain == RuntimeHostClockDomain::CrossClock {
        return RuntimeHostClockFallbackState::RuntimeResampled;
    }
    RuntimeHostClockFallbackState::Direct
}

pub(crate) fn host_clock_drift_state(
    configured_stream: bool,
    clock_domain: RuntimeHostClockDomain,
    backend_health: BackendHealth,
) -> RuntimeHostClockDriftState {
    if !configured_stream {
        return RuntimeHostClockDriftState::Unconfigured;
    }
    if backend_health != BackendHealth::Healthy {
        return RuntimeHostClockDriftState::Resyncing;
    }
    match clock_domain {
        RuntimeHostClockDomain::SameClock => RuntimeHostClockDriftState::Stable,
        RuntimeHostClockDomain::CrossClock => RuntimeHostClockDriftState::CrossClockManaged,
        RuntimeHostClockDomain::Aggregate => RuntimeHostClockDriftState::AggregateManaged,
        RuntimeHostClockDomain::Degraded => RuntimeHostClockDriftState::Resyncing,
    }
}

pub(crate) fn host_clock_discontinuity_state(
    configured_stream: bool,
    transition_state: RuntimeHostClockTransitionState,
    backend_health: BackendHealth,
    stream_state: RuntimeHostAudioStreamState,
) -> RuntimeHostClockDiscontinuityState {
    if !configured_stream {
        return RuntimeHostClockDiscontinuityState::LostConfiguration;
    }
    if stream_state == RuntimeHostAudioStreamState::Faulted {
        return RuntimeHostClockDiscontinuityState::Faulted;
    }
    if backend_health != BackendHealth::Healthy
        || transition_state == RuntimeHostClockTransitionState::EnteredRecoveryFallback
    {
        return RuntimeHostClockDiscontinuityState::Recovering;
    }
    match transition_state {
        RuntimeHostClockTransitionState::InitialObservation
        | RuntimeHostClockTransitionState::Stable => RuntimeHostClockDiscontinuityState::Continuous,
        RuntimeHostClockTransitionState::LostConfiguration => {
            RuntimeHostClockDiscontinuityState::LostConfiguration
        }
        RuntimeHostClockTransitionState::EnteredAggregateClock
        | RuntimeHostClockTransitionState::EnteredCrossClockFallback
        | RuntimeHostClockTransitionState::ReturnedToDirect
        | RuntimeHostClockTransitionState::Reconfigured => {
            RuntimeHostClockDiscontinuityState::Reconfigured
        }
        RuntimeHostClockTransitionState::EnteredRecoveryFallback => {
            RuntimeHostClockDiscontinuityState::Recovering
        }
    }
}

pub(crate) fn host_endpoint_topology(
    active_stream: Option<&HardwareStreamConfig>,
) -> RuntimeHostEndpointTopology {
    let Some(stream) = active_stream else {
        return RuntimeHostEndpointTopology::Unconfigured;
    };
    if stream.clock_topology == HardwareClockTopology::Aggregate {
        return RuntimeHostEndpointTopology::Aggregate;
    }
    match stream.direction {
        AudioStreamDirection::Output => RuntimeHostEndpointTopology::OutputOnly,
        AudioStreamDirection::Input => RuntimeHostEndpointTopology::InputOnly,
        AudioStreamDirection::Duplex => RuntimeHostEndpointTopology::Duplex,
    }
}

pub(crate) fn host_partial_availability(active_stream: Option<&HardwareStreamConfig>) -> bool {
    active_stream
        .map(|stream| {
            stream.direction == AudioStreamDirection::Duplex
                && (stream.input_channels == 0 || stream.output_channels == 0)
        })
        .unwrap_or(false)
}

pub(crate) fn host_duplex_mismatch_state(
    active_stream: Option<&HardwareStreamConfig>,
    clock_domain: RuntimeHostClockDomain,
    backend_health: BackendHealth,
    stream_state: RuntimeHostAudioStreamState,
    partial_availability: bool,
) -> RuntimeHostDuplexMismatchState {
    let Some(stream) = active_stream else {
        return RuntimeHostDuplexMismatchState::NotApplicable;
    };
    if stream.direction != AudioStreamDirection::Duplex {
        return RuntimeHostDuplexMismatchState::NotApplicable;
    }
    if stream_state == RuntimeHostAudioStreamState::Faulted
        || backend_health != BackendHealth::Healthy
    {
        return RuntimeHostDuplexMismatchState::Degraded;
    }
    if partial_availability {
        return RuntimeHostDuplexMismatchState::PartialAvailability;
    }
    match clock_domain {
        RuntimeHostClockDomain::CrossClock | RuntimeHostClockDomain::Aggregate => {
            RuntimeHostDuplexMismatchState::CrossClockDiverged
        }
        RuntimeHostClockDomain::SameClock => RuntimeHostDuplexMismatchState::Aligned,
        RuntimeHostClockDomain::Degraded => RuntimeHostDuplexMismatchState::Degraded,
    }
}
