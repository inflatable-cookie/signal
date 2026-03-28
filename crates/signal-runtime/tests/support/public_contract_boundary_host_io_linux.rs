use signal_hardware::{
    AudioSampleFormat, BackendHealth, HardwareBackendIdentity, LinuxAudioBackendKind,
};
use signal_runtime::{
    RuntimeHostAudioPumpSummary, RuntimeHostAudioStreamState, RuntimeHostAudioTransferPolicy,
    RuntimeHostClockDiscontinuityState, RuntimeHostClockDomain, RuntimeHostClockDriftState,
    RuntimeHostClockFallbackState, RuntimeHostClockSource, RuntimeHostClockTransitionState,
    RuntimeHostClockingSummary, RuntimeHostDuplexMismatchState, RuntimeHostEndpointTopology,
    RuntimeHostHardwareSummary, RuntimeHostIoSummary, RuntimeHostLatencySummary,
    RuntimeHostLifecycleOwnership, RuntimeHostRestartPolicy,
};

pub struct PublicLinuxBackendHostIoConfig<'a> {
    pub backend_identity: HardwareBackendIdentity,
    pub backend_name: &'a str,
    pub device_id: &'a str,
    pub device_name: &'a str,
    pub simulated: bool,
    pub backend_health: BackendHealth,
    pub device_loss_count: u64,
    pub restart_attempt_count: u64,
    pub restart_failure_count: u64,
}

pub fn sample_public_linux_backend_host_io(
    config: PublicLinuxBackendHostIoConfig<'_>,
) -> RuntimeHostIoSummary {
    let linux_backend_identity =
        signal_runtime::RuntimeHostHardwareSummary::classify_linux_backend_identity(
            config.backend_identity,
        );
    let clock_source = match config.backend_identity {
        HardwareBackendIdentity::Linux(LinuxAudioBackendKind::Alsa) => {
            RuntimeHostClockSource::Internal
        }
        HardwareBackendIdentity::Linux(LinuxAudioBackendKind::Jack) => {
            RuntimeHostClockSource::ExternalWordClock
        }
        HardwareBackendIdentity::Linux(LinuxAudioBackendKind::PipeWire) => {
            RuntimeHostClockSource::Virtual
        }
        _ => RuntimeHostClockSource::Internal,
    };
    let clock_domain = match config.backend_identity {
        HardwareBackendIdentity::Linux(LinuxAudioBackendKind::Alsa) => {
            RuntimeHostClockDomain::SameClock
        }
        HardwareBackendIdentity::Linux(LinuxAudioBackendKind::Jack)
        | HardwareBackendIdentity::Linux(LinuxAudioBackendKind::PipeWire) => {
            RuntimeHostClockDomain::Aggregate
        }
        _ => RuntimeHostClockDomain::SameClock,
    };
    let fallback_state = match config.backend_identity {
        HardwareBackendIdentity::Linux(LinuxAudioBackendKind::Alsa) => {
            RuntimeHostClockFallbackState::Direct
        }
        HardwareBackendIdentity::Linux(LinuxAudioBackendKind::Jack)
        | HardwareBackendIdentity::Linux(LinuxAudioBackendKind::PipeWire) => {
            RuntimeHostClockFallbackState::RuntimeResampled
        }
        _ => RuntimeHostClockFallbackState::Direct,
    };
    let transition_state = match config.backend_identity {
        HardwareBackendIdentity::Linux(LinuxAudioBackendKind::Alsa) => {
            RuntimeHostClockTransitionState::Stable
        }
        HardwareBackendIdentity::Linux(LinuxAudioBackendKind::Jack)
        | HardwareBackendIdentity::Linux(LinuxAudioBackendKind::PipeWire) => {
            RuntimeHostClockTransitionState::EnteredAggregateClock
        }
        _ => RuntimeHostClockTransitionState::Stable,
    };
    let drift_state = match config.backend_identity {
        HardwareBackendIdentity::Linux(LinuxAudioBackendKind::Alsa) => {
            RuntimeHostClockDriftState::Stable
        }
        HardwareBackendIdentity::Linux(LinuxAudioBackendKind::Jack)
        | HardwareBackendIdentity::Linux(LinuxAudioBackendKind::PipeWire) => {
            RuntimeHostClockDriftState::AggregateManaged
        }
        _ => RuntimeHostClockDriftState::Stable,
    };
    let discontinuity_state = match config.backend_identity {
        HardwareBackendIdentity::Linux(LinuxAudioBackendKind::Alsa) => {
            RuntimeHostClockDiscontinuityState::Continuous
        }
        HardwareBackendIdentity::Linux(LinuxAudioBackendKind::Jack)
        | HardwareBackendIdentity::Linux(LinuxAudioBackendKind::PipeWire) => {
            RuntimeHostClockDiscontinuityState::Reconfigured
        }
        _ => RuntimeHostClockDiscontinuityState::Continuous,
    };
    let duplex_mismatch_state = match config.backend_identity {
        HardwareBackendIdentity::Linux(LinuxAudioBackendKind::Alsa) => {
            RuntimeHostDuplexMismatchState::Aligned
        }
        HardwareBackendIdentity::Linux(LinuxAudioBackendKind::Jack)
        | HardwareBackendIdentity::Linux(LinuxAudioBackendKind::PipeWire) => {
            RuntimeHostDuplexMismatchState::CrossClockDiverged
        }
        _ => RuntimeHostDuplexMismatchState::NotApplicable,
    };
    let endpoint_topology = match config.backend_identity {
        HardwareBackendIdentity::Linux(LinuxAudioBackendKind::Alsa) => {
            RuntimeHostEndpointTopology::Duplex
        }
        HardwareBackendIdentity::Linux(LinuxAudioBackendKind::Jack)
        | HardwareBackendIdentity::Linux(LinuxAudioBackendKind::PipeWire) => {
            RuntimeHostEndpointTopology::Aggregate
        }
        _ => RuntimeHostEndpointTopology::Duplex,
    };
    let partial_availability = false;
    let stream_state = RuntimeHostAudioStreamState::Running;
    RuntimeHostIoSummary {
        hardware: RuntimeHostHardwareSummary {
            backend_identity: config.backend_identity,
            backend_name: config.backend_name.into(),
            linux_backend_identity,
            linux_backend_portability:
                signal_runtime::RuntimeHostHardwareSummary::classify_linux_backend_portability(
                    config.backend_identity,
                    config.simulated,
                    config.backend_health,
                    config.device_loss_count,
                    config.restart_attempt_count,
                    config.restart_failure_count,
                ),
            device_id: config.device_id.into(),
            device_name: config.device_name.into(),
            sample_rate: 48_000,
            buffer_size: 256,
            input_channels: 2,
            output_channels: 2,
            sample_format: AudioSampleFormat::F32,
            simulated: config.simulated,
            backend_health: config.backend_health,
            xrun_count: 0,
            callback_overrun_count: 0,
            device_loss_count: config.device_loss_count,
            restart_attempt_count: config.restart_attempt_count,
            restart_failure_count: config.restart_failure_count,
        },
        audio_pump: RuntimeHostAudioPumpSummary {
            stream_state,
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
            last_callback_output_peak: Some(0.25),
            last_runtime_graph_id: Some("graph:public-linux-backend".into()),
        },
        clocking: RuntimeHostClockingSummary {
            clock_source,
            ownership: RuntimeHostLifecycleOwnership::BackendManagedCallback,
            restart_policy: RuntimeHostRestartPolicy::BackendMayRestart,
            processing_sample_rate_hz: 48_000,
            hardware_sample_rate_hz: 48_000,
            clock_domain,
            fallback_state,
            transition_state,
            drift_state,
            discontinuity_state,
            duplex_mismatch_state,
            endpoint_topology,
            linux_clocking_parity:
                signal_runtime::RuntimeHostIoSummary::classify_linux_clocking_parity(
                    signal_runtime::RuntimeLinuxHostIoParityInput {
                        linux_backend_identity,
                        backend_health: config.backend_health,
                        stream_state,
                        clock_domain,
                        fallback_state,
                        transition_state,
                        drift_state,
                        discontinuity_state,
                        duplex_mismatch_state,
                        endpoint_topology,
                        partial_availability,
                    },
                ),
            linux_duplex_parity: signal_runtime::RuntimeHostIoSummary::classify_linux_duplex_parity(
                signal_runtime::RuntimeLinuxHostIoParityInput {
                    linux_backend_identity,
                    backend_health: config.backend_health,
                    stream_state,
                    clock_domain,
                    fallback_state,
                    transition_state,
                    drift_state,
                    discontinuity_state,
                    duplex_mismatch_state,
                    endpoint_topology,
                    partial_availability,
                },
            ),
            linux_endpoint_topology_parity:
                signal_runtime::RuntimeHostIoSummary::classify_linux_endpoint_topology_parity(
                    linux_backend_identity,
                    config.backend_health,
                    transition_state,
                    discontinuity_state,
                    duplex_mismatch_state,
                    endpoint_topology,
                    partial_availability,
                ),
            partial_availability,
            crossing_required: matches!(
                clock_domain,
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
