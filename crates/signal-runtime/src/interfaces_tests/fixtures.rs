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
    let linux_backend_identity = RuntimeHostHardwareSummary::classify_linux_backend_identity(
        HardwareBackendIdentity::CoreAudio,
    );
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
            linux_backend_identity,
            linux_backend_portability:
                RuntimeHostHardwareSummary::classify_linux_backend_portability(
                    HardwareBackendIdentity::CoreAudio,
                    false,
                    backend_health,
                    device_loss_count,
                    restart_attempt_count,
                    restart_failure_count,
                ),
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
            linux_clocking_parity: RuntimeHostIoSummary::classify_linux_clocking_parity(
                RuntimeHostIoSummary::linux_parity_input(
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
                ),
            ),
            linux_duplex_parity: RuntimeHostIoSummary::classify_linux_duplex_parity(
                RuntimeHostIoSummary::linux_parity_input(
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
                ),
            ),
            linux_endpoint_topology_parity:
                RuntimeHostIoSummary::classify_linux_endpoint_topology_parity(
                    linux_backend_identity,
                    backend_health,
                    transition_state,
                    discontinuity_state,
                    duplex_mismatch_state,
                    endpoint_topology,
                    partial_availability,
                ),
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

pub(super) fn linux_host_io_summary(
    backend_identity: HardwareBackendIdentity,
    ownership: RuntimeHostLifecycleOwnership,
    stream_state: RuntimeHostAudioStreamState,
    backend_health: BackendHealth,
    device_loss_count: u64,
    restart_attempt_count: u64,
    restart_failure_count: u64,
) -> RuntimeHostIoSummary {
    let linux_backend_identity =
        RuntimeHostHardwareSummary::classify_linux_backend_identity(backend_identity);
    let endpoint_topology = RuntimeHostEndpointTopology::Duplex;
    RuntimeHostIoSummary {
        hardware: RuntimeHostHardwareSummary {
            backend_identity,
            backend_name: match backend_identity {
                HardwareBackendIdentity::Linux(LinuxAudioBackendKind::Alsa) => "alsa",
                HardwareBackendIdentity::Linux(LinuxAudioBackendKind::Jack) => "jack",
                HardwareBackendIdentity::Linux(LinuxAudioBackendKind::PipeWire) => "pipewire",
                HardwareBackendIdentity::CoreAudio => "coreaudio",
                HardwareBackendIdentity::Unsupported => "unsupported",
            }
            .into(),
            linux_backend_identity,
            linux_backend_portability:
                RuntimeHostHardwareSummary::classify_linux_backend_portability(
                    backend_identity,
                    true,
                    backend_health,
                    device_loss_count,
                    restart_attempt_count,
                    restart_failure_count,
                ),
            device_id: format!("{:?}:device", linux_backend_identity),
            device_name: format!("{:?} Device", linux_backend_identity),
            sample_rate: 48_000,
            buffer_size: 256,
            input_channels: 2,
            output_channels: 2,
            sample_format: AudioSampleFormat::F32,
            simulated: true,
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
            callback_count: 8,
            total_callback_frames: 2_048,
            total_runtime_output_frames: 2_048,
            copied_output_samples: 4_096,
            zero_filled_output_samples: 0,
            dropped_output_samples: 0,
            last_callback_output_peak: Some(0.5),
            last_runtime_graph_id: Some("graph:linux".into()),
        },
        clocking: RuntimeHostClockingSummary {
            clock_source: match backend_identity {
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
            },
            ownership,
            restart_policy: match ownership {
                RuntimeHostLifecycleOwnership::HostDrivenCallback => {
                    RuntimeHostRestartPolicy::HostMustRestart
                }
                RuntimeHostLifecycleOwnership::BackendManagedCallback => {
                    RuntimeHostRestartPolicy::BackendMayRestart
                }
            },
            processing_sample_rate_hz: 48_000,
            hardware_sample_rate_hz: 48_000,
            clock_domain: RuntimeHostClockDomain::SameClock,
            fallback_state: RuntimeHostClockFallbackState::Direct,
            transition_state: RuntimeHostClockTransitionState::Stable,
            drift_state: RuntimeHostClockDriftState::Stable,
            discontinuity_state: RuntimeHostClockDiscontinuityState::Continuous,
            duplex_mismatch_state: RuntimeHostDuplexMismatchState::Aligned,
            endpoint_topology,
            linux_clocking_parity: RuntimeHostIoSummary::classify_linux_clocking_parity(
                RuntimeHostIoSummary::linux_parity_input(
                    linux_backend_identity,
                    backend_health,
                    stream_state,
                    RuntimeHostClockDomain::SameClock,
                    RuntimeHostClockFallbackState::Direct,
                    RuntimeHostClockTransitionState::Stable,
                    RuntimeHostClockDriftState::Stable,
                    RuntimeHostClockDiscontinuityState::Continuous,
                    RuntimeHostDuplexMismatchState::Aligned,
                    endpoint_topology,
                    false,
                ),
            ),
            linux_duplex_parity: RuntimeHostIoSummary::classify_linux_duplex_parity(
                RuntimeHostIoSummary::linux_parity_input(
                    linux_backend_identity,
                    backend_health,
                    stream_state,
                    RuntimeHostClockDomain::SameClock,
                    RuntimeHostClockFallbackState::Direct,
                    RuntimeHostClockTransitionState::Stable,
                    RuntimeHostClockDriftState::Stable,
                    RuntimeHostClockDiscontinuityState::Continuous,
                    RuntimeHostDuplexMismatchState::Aligned,
                    endpoint_topology,
                    false,
                ),
            ),
            linux_endpoint_topology_parity:
                RuntimeHostIoSummary::classify_linux_endpoint_topology_parity(
                    linux_backend_identity,
                    backend_health,
                    RuntimeHostClockTransitionState::Stable,
                    RuntimeHostClockDiscontinuityState::Continuous,
                    RuntimeHostDuplexMismatchState::Aligned,
                    endpoint_topology,
                    false,
                ),
            partial_availability: false,
            crossing_required: false,
            callback_interval_ms: 5.333,
        },
        latency: RuntimeHostLatencySummary {
            input_latency_samples: Some(128),
            output_latency_samples: 256,
            round_trip_latency_samples: Some(384),
            graph_latency_samples: 128,
            estimated_output_latency_samples: 384,
            estimated_round_trip_latency_samples: Some(512),
            output_latency_ms: 5.333,
            graph_latency_ms: 2.667,
            estimated_output_latency_ms: 8.0,
            estimated_round_trip_latency_ms: Some(10.667),
        },
        runtime_graph_id_matches_pump: true,
    }
}

pub(super) fn transport_session_summary(
    current_state: TransportSessionState,
    currently_attached: bool,
    heartbeat_freshness: TransportHeartbeatFreshness,
    dispatch_state: TransportDispatchState,
    attach_events: usize,
    detach_requested_events: usize,
    detached_events: usize,
) -> TransportSessionSummary {
    TransportSessionSummary {
        boundary_mode: TransportSessionBoundaryMode::HealthyPathVisible,
        current_state,
        currently_attached,
        heartbeat_freshness,
        dispatch_state,
        current_attached_session_count: usize::from(currently_attached),
        max_concurrent_attached_sessions: usize::from(currently_attached),
        attach_events,
        detach_requested_events,
        detached_events,
        detach_fault_events: 0,
        heartbeat_requested_events: usize::from(matches!(
            heartbeat_freshness,
            TransportHeartbeatFreshness::Requested
                | TransportHeartbeatFreshness::Fresh
                | TransportHeartbeatFreshness::Missed
        )),
        heartbeat_responded_events: usize::from(matches!(
            heartbeat_freshness,
            TransportHeartbeatFreshness::Fresh
        )),
        heartbeat_missed_events: usize::from(matches!(
            heartbeat_freshness,
            TransportHeartbeatFreshness::Missed
        )),
        dispatch_requested_events: usize::from(matches!(
            dispatch_state,
            TransportDispatchState::Requested
                | TransportDispatchState::Completed
                | TransportDispatchState::TimedOut
        )),
        dispatch_completed_events: usize::from(matches!(
            dispatch_state,
            TransportDispatchState::Completed
        )),
        dispatch_timed_out_events: usize::from(matches!(
            dispatch_state,
            TransportDispatchState::TimedOut
        )),
        first_processing_epoch: None,
        last_processing_epoch: None,
        first_block_sequence: None,
        last_block_sequence: None,
        active_sandbox_id: None,
        active_lease_id: None,
        active_region_id: None,
        active_block_sequence: None,
        active_sessions: Vec::new(),
        last_sandbox_id: None,
        last_lease_id: None,
        last_region_id: None,
    }
}
