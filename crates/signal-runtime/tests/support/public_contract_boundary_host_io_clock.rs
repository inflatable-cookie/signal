use signal_hardware::{AudioSampleFormat, BackendHealth, HardwareBackendIdentity};
use signal_runtime::{
    RuntimeHostAudioPumpSummary, RuntimeHostAudioStreamState, RuntimeHostAudioTransferPolicy,
    RuntimeHostClockDiscontinuityState, RuntimeHostClockDomain, RuntimeHostClockDriftState,
    RuntimeHostClockFallbackState, RuntimeHostClockSource, RuntimeHostClockTransitionState,
    RuntimeHostClockingSummary, RuntimeHostDuplexMismatchState, RuntimeHostEndpointTopology,
    RuntimeHostHardwareSummary, RuntimeHostIoSummary, RuntimeHostLatencySummary,
    RuntimeHostLifecycleOwnership, RuntimeHostRestartPolicy,
};

pub struct PublicClockTopologyHostIoConfig {
    pub clock_domain: RuntimeHostClockDomain,
    pub fallback_state: RuntimeHostClockFallbackState,
    pub transition_state: RuntimeHostClockTransitionState,
    pub drift_state: RuntimeHostClockDriftState,
    pub discontinuity_state: RuntimeHostClockDiscontinuityState,
    pub duplex_mismatch_state: RuntimeHostDuplexMismatchState,
    pub endpoint_topology: RuntimeHostEndpointTopology,
    pub partial_availability: bool,
}

pub fn sample_public_clock_topology_host_io(
    config: PublicClockTopologyHostIoConfig,
) -> RuntimeHostIoSummary {
    RuntimeHostIoSummary {
        hardware: RuntimeHostHardwareSummary {
            backend_identity: HardwareBackendIdentity::CoreAudio,
            backend_name: "coreaudio".into(),
            device_id: "device:public-clock-topology".into(),
            device_name: "Public Clock Topology".into(),
            sample_rate: 48_000,
            buffer_size: 256,
            input_channels: 0,
            output_channels: 2,
            sample_format: AudioSampleFormat::F32,
            simulated: false,
            backend_health: BackendHealth::Healthy,
            xrun_count: 0,
            callback_overrun_count: 0,
            device_loss_count: 0,
            restart_attempt_count: 0,
            restart_failure_count: 0,
        },
        audio_pump: RuntimeHostAudioPumpSummary {
            stream_state: RuntimeHostAudioStreamState::Running,
            transfer_policy: RuntimeHostAudioTransferPolicy {
                max_callback_frames: 256,
                max_transfer_channels: 2,
                zero_fill_unwritten_output: true,
            },
            callback_count: 12,
            total_callback_frames: 3_072,
            total_runtime_output_frames: 3_072,
            copied_output_samples: 6_144,
            zero_filled_output_samples: 0,
            dropped_output_samples: 0,
            last_callback_output_peak: Some(0.35),
            last_runtime_graph_id: Some("graph:public-clock-topology".into()),
        },
        clocking: RuntimeHostClockingSummary {
            clock_source: RuntimeHostClockSource::Internal,
            ownership: RuntimeHostLifecycleOwnership::HostDrivenCallback,
            restart_policy: RuntimeHostRestartPolicy::HostMustRestart,
            processing_sample_rate_hz: 44_100,
            hardware_sample_rate_hz: 48_000,
            clock_domain: config.clock_domain,
            fallback_state: config.fallback_state,
            transition_state: config.transition_state,
            drift_state: config.drift_state,
            discontinuity_state: config.discontinuity_state,
            duplex_mismatch_state: config.duplex_mismatch_state,
            endpoint_topology: config.endpoint_topology,
            partial_availability: config.partial_availability,
            crossing_required: matches!(
                config.clock_domain,
                RuntimeHostClockDomain::CrossClock | RuntimeHostClockDomain::Aggregate
            ),
            callback_interval_ms: 5.333,
        },
        latency: RuntimeHostLatencySummary {
            input_latency_samples: Some(128),
            output_latency_samples: 256,
            round_trip_latency_samples: Some(384),
            graph_latency_samples: 24,
            estimated_output_latency_samples: 280,
            estimated_round_trip_latency_samples: Some(408),
            output_latency_ms: 5.333,
            graph_latency_ms: 0.5,
            estimated_output_latency_ms: 5.833,
            estimated_round_trip_latency_ms: Some(8.5),
        },
        runtime_graph_id_matches_pump: true,
    }
}
