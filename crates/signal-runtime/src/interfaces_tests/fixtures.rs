use super::*;

pub(super) fn host_io_summary(
    fallback_state: RuntimeHostClockFallbackState,
    transition_state: RuntimeHostClockTransitionState,
    stream_state: RuntimeHostAudioStreamState,
    backend_health: BackendHealth,
    restart_attempt_count: u64,
    restart_failure_count: u64,
    device_loss_count: u64,
) -> RuntimeHostIoSummary {
    let clock_domain = RuntimeHostClockDomain::SameClock;
    let drift_state = RuntimeHostClockDriftState::Stable;
    let discontinuity_state = RuntimeHostClockDiscontinuityState::Continuous;
    let duplex_mismatch_state = RuntimeHostDuplexMismatchState::NotApplicable;
    let endpoint_topology = RuntimeHostEndpointTopology::OutputOnly;
    let partial_availability = false;
    RuntimeHostIoSummary {
        hardware: RuntimeHostHardwareSummary {
            backend_identity: HardwareBackendIdentity::CoreAudio,
            backend_name: "coreaudio".to_string(),
            device_id: "device:main".to_string(),
            device_name: "Main Output".to_string(),
            sample_rate: 48_000,
            buffer_size: 256,
            input_channels: 0,
            output_channels: 2,
            sample_format: AudioSampleFormat::F32,
            simulated: false,
            backend_health,
            xrun_count: 0,
            callback_overrun_count: 0,
            device_loss_count,
            restart_attempt_count,
            restart_failure_count,
        },
        audio_pump: RuntimeHostAudioPumpSummary {
            stream_state,
            transfer_policy: RuntimeHostAudioTransferPolicy {
                max_callback_frames: 256,
                max_transfer_channels: 2,
                zero_fill_unwritten_output: true,
            },
            callback_count: 32,
            total_callback_frames: 8_192,
            total_runtime_output_frames: 8_192,
            copied_output_samples: 16_384,
            zero_filled_output_samples: 0,
            dropped_output_samples: 0,
            last_callback_output_peak: Some(0.42),
            last_runtime_graph_id: Some("graph:main".to_string()),
        },
        clocking: RuntimeHostClockingSummary {
            clock_source: RuntimeHostClockSource::Internal,
            ownership: RuntimeHostLifecycleOwnership::HostDrivenCallback,
            restart_policy: RuntimeHostRestartPolicy::HostMustRestart,
            processing_sample_rate_hz: 48_000,
            hardware_sample_rate_hz: 48_000,
            clock_domain,
            fallback_state,
            transition_state,
            drift_state,
            discontinuity_state,
            duplex_mismatch_state,
            endpoint_topology,
            partial_availability,
            crossing_required: false,
            callback_interval_ms: 5.333,
        },
        latency: RuntimeHostLatencySummary {
            input_latency_samples: None,
            output_latency_samples: 256,
            round_trip_latency_samples: None,
            graph_latency_samples: 128,
            estimated_output_latency_samples: 384,
            estimated_round_trip_latency_samples: None,
            output_latency_ms: 5.333,
            graph_latency_ms: 2.667,
            estimated_output_latency_ms: 8.0,
            estimated_round_trip_latency_ms: None,
        },
        runtime_graph_id_matches_pump: true,
    }
}
