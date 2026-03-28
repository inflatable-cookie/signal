// Tests for signal-runtime interfaces
use super::*;
use crate::{RuntimeConfig, SignalRuntime};
use signal_hardware::{
    AudioSampleFormat, BackendHealth, HardwareBackendIdentity, LinuxAudioBackendKind,
};

fn host_io_summary(
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

fn linux_host_io_summary(
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

fn transport_session_summary(
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

#[test]
fn runtime_multichannel_layout_summary_maps_canonical_and_custom_roles() {
    let stereo = RuntimeMultichannelLayoutSummary::from_channel_layout(ChannelLayout::Stereo);
    assert_eq!(
        stereo.canonical_layout,
        Some(RuntimeCanonicalChannelLayout::Stereo)
    );
    assert_eq!(
        stereo.channel_roles,
        vec![
            RuntimeChannelRole::FrontLeft,
            RuntimeChannelRole::FrontRight
        ]
    );
    assert!(!stereo.uses_custom_fallback);

    let custom = RuntimeMultichannelLayoutSummary::from_channel_layout(ChannelLayout::Count(
        signal_primitives::ChannelCount(7),
    ));
    assert_eq!(custom.canonical_layout, None);
    assert_eq!(custom.channel_roles.len(), 7);
    assert!(matches!(
        custom.channel_roles.last(),
        Some(RuntimeChannelRole::Discrete(6))
    ));
    assert!(custom.uses_custom_fallback);
}

#[test]
fn runtime_execution_topology_summary_carries_multichannel_layout_and_bus_intents() {
    let snapshot = RuntimeEngineBlockSnapshot {
        planned_nodes: vec![RuntimePlannedGraphNode {
            node_id: "track-main".into(),
            execution_class: GraphNodeExecutionClass::PluginBacked,
            group: GraphNodePlanningGroup::InlineRealtime,
            latency_samples: 32,
            topology_role: GraphNodeTopologyRole::TrackLane,
            track_lane_id: Some("track:main".into()),
            bus_group_id: Some("bus:main".into()),
            console_group_id: None,
            send_return_id: None,
            input_bus_id: "track:main-in".into(),
            output_bus_id: "track:main-out".into(),
            input_channels: ChannelLayout::Stereo,
            output_channels: ChannelLayout::Count(signal_primitives::ChannelCount(6)),
            input_layout: RuntimeMultichannelLayoutSummary::from_channel_layout(
                ChannelLayout::Stereo,
            ),
            output_layout: RuntimeMultichannelLayoutSummary::from_channel_layout(
                ChannelLayout::Count(signal_primitives::ChannelCount(6)),
            ),
            input_bus_intent: RuntimeBusIntent::MainProgram,
            output_bus_intent: RuntimeBusIntent::MainProgram,
            secondary_input: None,
            spatial_execution: None,
            plugin_sandbox_id: Some("sandbox:track-main".into()),
        }],
        lane_order: vec![GraphExecutionLane::Realtime],
        ..RuntimeEngineBlockSnapshot::default()
    };

    let topology = RuntimeExecutionTopologySummary::from_snapshot(&snapshot);
    assert_eq!(topology.node_count, 1);
    assert_eq!(
        topology.nodes[0].input_layout.canonical_layout,
        Some(RuntimeCanonicalChannelLayout::Stereo)
    );
    assert_eq!(
        topology.nodes[0].output_layout.canonical_layout,
        Some(RuntimeCanonicalChannelLayout::Surround5_1)
    );
    assert_eq!(
        topology.nodes[0].input_bus_intent,
        RuntimeBusIntent::MainProgram
    );
    assert_eq!(
        topology.nodes[0].output_bus_intent,
        RuntimeBusIntent::MainProgram
    );
}

#[test]
fn runtime_external_io_snapshot_marks_clock_fallback_active() {
    let summary = host_io_summary(
        RuntimeHostClockFallbackState::RuntimeResampled,
        RuntimeHostClockTransitionState::EnteredCrossClockFallback,
        RuntimeHostAudioStreamState::Running,
        BackendHealth::Recovering,
        1,
        0,
        0,
    );

    let snapshot = summary.build_external_io_snapshot();

    assert_eq!(
        snapshot.health_state,
        RuntimeExternalIoHealthState::FallbackActive
    );
    assert_eq!(
        snapshot.device_change_state,
        RuntimeExternalIoDeviceChangeState::Recovering
    );
    assert_eq!(
        snapshot.primary_role,
        RuntimeExternalIoPrimaryRole::ProgramOutput
    );
    assert_eq!(
        snapshot.monitoring_state,
        RuntimeExternalIoMonitoringState::Degraded
    );
    assert_eq!(
        snapshot.monitoring_tap_point,
        RuntimeExternalIoMonitoringTapPoint::PostHardwareOutput
    );
    assert_eq!(
        snapshot.loopback_state,
        RuntimeExternalIoLoopbackState::Recovering
    );
    assert!(snapshot.fallback_active);
    assert_eq!(
        snapshot.fallback_state,
        RuntimeHostClockFallbackState::RuntimeResampled
    );
    assert_eq!(snapshot.drift_state, RuntimeHostClockDriftState::Stable);
    assert_eq!(
        snapshot.discontinuity_state,
        RuntimeHostClockDiscontinuityState::Continuous
    );
    assert_eq!(
        snapshot.duplex_mismatch_state,
        RuntimeHostDuplexMismatchState::NotApplicable
    );
    assert_eq!(
        snapshot.endpoint_topology,
        RuntimeHostEndpointTopology::OutputOnly
    );
    assert_eq!(
        snapshot.linux_clocking_parity,
        RuntimeLinuxAudioBackendClockingParityBand::Unsupported
    );
    assert_eq!(
        snapshot.linux_duplex_parity,
        RuntimeLinuxAudioBackendDuplexParityState::Unsupported
    );
    assert_eq!(
        snapshot.linux_endpoint_topology_parity,
        RuntimeLinuxAudioBackendEndpointTopologyParityState::Unsupported
    );
    assert!(!snapshot.partial_availability);
    assert!(snapshot.summary.contains("fallback=true"));
}

#[test]
fn runtime_external_io_snapshot_distinguishes_recovering_from_terminal_failure() {
    let recovering = host_io_summary(
        RuntimeHostClockFallbackState::Direct,
        RuntimeHostClockTransitionState::EnteredRecoveryFallback,
        RuntimeHostAudioStreamState::Running,
        BackendHealth::Recovering,
        2,
        1,
        1,
    )
    .build_external_io_snapshot();
    assert_eq!(
        recovering.health_state,
        RuntimeExternalIoHealthState::Recovering
    );
    assert_eq!(
        recovering.device_change_state,
        RuntimeExternalIoDeviceChangeState::Recovering
    );
    assert_eq!(
        recovering.monitoring_state,
        RuntimeExternalIoMonitoringState::Degraded
    );
    assert_eq!(
        recovering.loopback_state,
        RuntimeExternalIoLoopbackState::Recovering
    );
    assert_eq!(recovering.io_layout.input_layout.channel_count, 0);
    assert_eq!(
        recovering.io_layout.output_layout.canonical_layout,
        Some(RuntimeCanonicalChannelLayout::Stereo)
    );

    let failed = host_io_summary(
        RuntimeHostClockFallbackState::RecoveryConstrained,
        RuntimeHostClockTransitionState::EnteredRecoveryFallback,
        RuntimeHostAudioStreamState::Faulted,
        BackendHealth::Recovering,
        2,
        1,
        1,
    )
    .build_external_io_snapshot();
    assert_eq!(failed.health_state, RuntimeExternalIoHealthState::Faulted);
    assert_eq!(
        failed.device_change_state,
        RuntimeExternalIoDeviceChangeState::Failed
    );
    assert_eq!(
        failed.monitoring_state,
        RuntimeExternalIoMonitoringState::Faulted
    );
    assert_eq!(
        failed.loopback_state,
        RuntimeExternalIoLoopbackState::Faulted
    );
    assert!(failed.fallback_active);
    assert_eq!(failed.drift_state, RuntimeHostClockDriftState::Stable);
    assert_eq!(
        failed.endpoint_topology,
        RuntimeHostEndpointTopology::OutputOnly
    );
    assert_eq!(
        failed.io_layout.output_bus_intent,
        RuntimeBusIntent::HardwareOutput
    );
}

#[test]
fn runtime_external_io_snapshot_surfaces_duplex_and_topology_receipts() {
    let mut summary = host_io_summary(
        RuntimeHostClockFallbackState::RuntimeResampled,
        RuntimeHostClockTransitionState::EnteredCrossClockFallback,
        RuntimeHostAudioStreamState::Running,
        BackendHealth::Healthy,
        0,
        0,
        0,
    );
    summary.clocking.drift_state = RuntimeHostClockDriftState::CrossClockManaged;
    summary.clocking.discontinuity_state = RuntimeHostClockDiscontinuityState::Reconfigured;
    summary.clocking.duplex_mismatch_state = RuntimeHostDuplexMismatchState::CrossClockDiverged;
    summary.clocking.endpoint_topology = RuntimeHostEndpointTopology::Duplex;
    summary.clocking.partial_availability = false;

    let snapshot = summary.build_external_io_snapshot();

    assert_eq!(
        snapshot.drift_state,
        RuntimeHostClockDriftState::CrossClockManaged
    );
    assert_eq!(
        snapshot.discontinuity_state,
        RuntimeHostClockDiscontinuityState::Reconfigured
    );
    assert_eq!(
        snapshot.duplex_mismatch_state,
        RuntimeHostDuplexMismatchState::CrossClockDiverged
    );
    assert_eq!(
        snapshot.endpoint_topology,
        RuntimeHostEndpointTopology::Duplex
    );
    assert_eq!(
        snapshot.linux_clocking_parity,
        RuntimeLinuxAudioBackendClockingParityBand::Unsupported
    );
    assert_eq!(
        snapshot.linux_duplex_parity,
        RuntimeLinuxAudioBackendDuplexParityState::Unsupported
    );
    assert_eq!(
        snapshot.linux_endpoint_topology_parity,
        RuntimeLinuxAudioBackendEndpointTopologyParityState::Unsupported
    );
    assert_eq!(
        snapshot.primary_role,
        RuntimeExternalIoPrimaryRole::ProgramDuplex
    );
    assert_eq!(
        snapshot.monitoring_state,
        RuntimeExternalIoMonitoringState::Guarded
    );
    assert_eq!(
        snapshot.loopback_state,
        RuntimeExternalIoLoopbackState::Ready
    );
    assert_eq!(snapshot.io_layout.input_layout.channel_count, 0);
    assert_eq!(
        snapshot.io_layout.output_layout.canonical_layout,
        Some(RuntimeCanonicalChannelLayout::Stereo)
    );
    assert!(!snapshot.partial_availability);
    assert!(snapshot.summary.contains("CrossClockManaged"));
}

#[test]
fn runtime_external_io_snapshot_defaults_to_unavailable_without_host_context() {
    let effective_config = EffectiveRuntimeConfig {
        sample_rate: SampleRate(48_000),
        block_size: 256,
        anticipative_enabled: true,
        safe_mode_enabled: false,
        active_output_device: None,
    };
    let device_supervision_snapshot = RuntimeDeviceSupervisionSnapshot {
        state: RuntimeDeviceSupervisionState::Stable,
        restart_state: RuntimeDeviceRestartState::Unneeded,
        fault_boundary: RuntimeDeviceFaultBoundaryState::Clear,
        recovery_state: RuntimeRecoveryState::Steady,
        interruption_class: RuntimeInterruptionClass::Steady,
        primary_fault_cause: None,
        safe_mode_enabled: false,
        device_loss_active: false,
        active_output_device: None,
        device_id: None,
        device_name: None,
        restart_policy: None,
        backend_health: None,
        stream_state: None,
        device_loss_count: 0,
        restart_attempt_count: None,
        restart_failure_count: None,
        watchdog_restart_count: 0,
        last_watchdog_trigger: None,
        summary: "steady".into(),
    };

    let snapshot = RuntimeHostIoSummary::unavailable_external_io_snapshot(
        &effective_config,
        &device_supervision_snapshot,
    );

    assert_eq!(
        snapshot.health_state,
        RuntimeExternalIoHealthState::Unavailable
    );
    assert_eq!(
        snapshot.device_change_state,
        RuntimeExternalIoDeviceChangeState::Unavailable
    );
    assert_eq!(
        snapshot.primary_role,
        RuntimeExternalIoPrimaryRole::Unavailable
    );
    assert_eq!(
        snapshot.monitoring_state,
        RuntimeExternalIoMonitoringState::Unavailable
    );
    assert_eq!(
        snapshot.monitoring_tap_point,
        RuntimeExternalIoMonitoringTapPoint::Unavailable
    );
    assert_eq!(
        snapshot.loopback_state,
        RuntimeExternalIoLoopbackState::Unavailable
    );
    assert_eq!(
        snapshot.endpoint_topology,
        RuntimeHostEndpointTopology::Unconfigured
    );
    assert_eq!(
        snapshot.linux_clocking_parity,
        RuntimeLinuxAudioBackendClockingParityBand::Unsupported
    );
    assert_eq!(
        snapshot.linux_duplex_parity,
        RuntimeLinuxAudioBackendDuplexParityState::Unsupported
    );
    assert_eq!(
        snapshot.linux_endpoint_topology_parity,
        RuntimeLinuxAudioBackendEndpointTopologyParityState::Unsupported
    );
    assert!(snapshot.summary.contains("runtime-unavailable"));
}

#[test]
fn runtime_external_midi_endpoint_graph_snapshot_distinguishes_unavailable_from_empty() {
    let unavailable = RuntimeExternalMidiEndpointGraphSnapshot::unavailable();
    assert_eq!(
        unavailable.discovery_state,
        RuntimeExternalMidiDiscoveryState::Unavailable
    );
    assert_eq!(
        unavailable.graph_state,
        RuntimeExternalMidiGraphState::Unavailable
    );
    assert_eq!(unavailable.provider_name, "runtime-unavailable");
    assert_eq!(unavailable.device_count, 0);
    assert_eq!(unavailable.endpoint_count, 0);
    assert_eq!(
        unavailable.live_ownership.ownership_posture,
        RuntimeExternalMidiLiveOwnershipPosture::Unavailable
    );
    assert!(unavailable.devices.is_empty());
    assert!(unavailable.endpoints.is_empty());
    assert!(unavailable.summary.contains("graph=Unavailable"));

    let empty = RuntimeExternalMidiEndpointGraphSnapshot::empty("signal-host-local");
    assert_eq!(
        empty.discovery_state,
        RuntimeExternalMidiDiscoveryState::Idle
    );
    assert_eq!(empty.graph_state, RuntimeExternalMidiGraphState::Empty);
    assert_eq!(empty.provider_name, "signal-host-local");
    assert_eq!(empty.device_count, 0);
    assert_eq!(empty.endpoint_count, 0);
    assert_eq!(empty.active_route_count, 0);
    assert_eq!(
        empty.live_ownership.ownership_posture,
        RuntimeExternalMidiLiveOwnershipPosture::NoLiveOwnership
    );
    assert_eq!(
        empty.live_ownership.attach_continuity,
        RuntimeExternalMidiAttachContinuity::Detached
    );
    assert!(empty.summary.contains("graph=Empty"));
}

#[test]
fn runtime_external_midi_live_ownership_summary_derives_runtime_owned_baselines() {
    let unavailable = RuntimeExternalMidiEndpointGraphSnapshot::empty("runtime-test")
        .with_live_ownership_summary(
            &RuntimeLinuxBackendSessionSnapshot::unavailable(),
            &RuntimeInterruptionSummary {
                active: false,
                class: RuntimeInterruptionClass::Steady,
                rebindable: false,
                recovery_state: RuntimeRecoveryState::Steady,
                primary_fault_cause: None,
                safe_mode_enabled: false,
                deferred_service_class: None,
                deferred_service_decision: None,
                summary: "steady".into(),
            },
        );
    assert_eq!(
        unavailable.live_ownership.ownership_posture,
        RuntimeExternalMidiLiveOwnershipPosture::Unavailable
    );
    assert_eq!(
        unavailable.live_ownership.backend_parity,
        RuntimeExternalMidiBackendParity::Unavailable
    );

    let not_linux_host = host_io_summary(
        RuntimeHostClockFallbackState::Direct,
        RuntimeHostClockTransitionState::Stable,
        RuntimeHostAudioStreamState::Running,
        BackendHealth::Healthy,
        0,
        0,
        0,
    );
    let not_linux = RuntimeExternalMidiEndpointGraphSnapshot::empty("coreaudio-test")
        .with_live_ownership_summary(
            &RuntimeLinuxBackendSessionSnapshot::from_host_io(&not_linux_host),
            &RuntimeInterruptionSummary {
                active: false,
                class: RuntimeInterruptionClass::Steady,
                rebindable: false,
                recovery_state: RuntimeRecoveryState::Steady,
                primary_fault_cause: None,
                safe_mode_enabled: false,
                deferred_service_class: None,
                deferred_service_decision: None,
                summary: "steady".into(),
            },
        );
    assert_eq!(
        not_linux.live_ownership.ownership_posture,
        RuntimeExternalMidiLiveOwnershipPosture::NoLiveOwnership
    );
    assert_eq!(
        not_linux.live_ownership.backend_parity,
        RuntimeExternalMidiBackendParity::NotLinux
    );
    assert_eq!(
        not_linux.live_ownership.guarded_parity_outcome,
        RuntimeExternalMidiGuardedParityOutcome::NotLinux
    );

    let pipewire_host = linux_host_io_summary(
        HardwareBackendIdentity::Linux(LinuxAudioBackendKind::PipeWire),
        RuntimeHostLifecycleOwnership::BackendManagedCallback,
        RuntimeHostAudioStreamState::Running,
        BackendHealth::Healthy,
        0,
        0,
        0,
    );
    let pipewire = RuntimeExternalMidiEndpointGraphSnapshot::empty("pipewire-test")
        .with_live_ownership_summary(
            &RuntimeLinuxBackendSessionSnapshot::from_host_io(&pipewire_host),
            &RuntimeInterruptionSummary {
                active: false,
                class: RuntimeInterruptionClass::Steady,
                rebindable: false,
                recovery_state: RuntimeRecoveryState::Steady,
                primary_fault_cause: None,
                safe_mode_enabled: false,
                deferred_service_class: None,
                deferred_service_decision: None,
                summary: "steady".into(),
            },
        );
    assert_eq!(
        pipewire.live_ownership.ownership_posture,
        RuntimeExternalMidiLiveOwnershipPosture::NoLiveOwnership
    );
    assert_eq!(
        pipewire.live_ownership.attach_continuity,
        RuntimeExternalMidiAttachContinuity::Detached
    );
    assert_eq!(
        pipewire.live_ownership.backend_parity,
        RuntimeExternalMidiBackendParity::Guarded
    );
    assert_eq!(
        pipewire.live_ownership.guarded_parity_outcome,
        RuntimeExternalMidiGuardedParityOutcome::BackendManaged
    );
}

#[test]
fn runtime_control_surface_snapshot_derives_from_external_midi_baselines() {
    let unavailable = RuntimeControlSurfaceSnapshot::from_external_midi_snapshot(
        &RuntimeExternalMidiEndpointGraphSnapshot::unavailable(),
    );
    assert_eq!(
        unavailable.discovery_state,
        RuntimeExternalMidiDiscoveryState::Unavailable
    );
    assert_eq!(
        unavailable.graph_state,
        RuntimeControlSurfaceGraphState::Unavailable
    );
    assert_eq!(unavailable.device_count, 0);

    let empty = RuntimeControlSurfaceSnapshot::from_external_midi_snapshot(
        &RuntimeExternalMidiEndpointGraphSnapshot::empty("signal-host-local"),
    );
    assert_eq!(
        empty.discovery_state,
        RuntimeExternalMidiDiscoveryState::Idle
    );
    assert_eq!(empty.graph_state, RuntimeControlSurfaceGraphState::Empty);
    assert_eq!(empty.provider_name, "signal-host-local");
    assert_eq!(empty.device_count, 0);

    let capability = RuntimeExternalMidiEndpointCapabilitySummary {
        supports_bounded_midi_input: true,
        supports_bounded_midi_output: true,
        supports_transport_clock: true,
        supports_note_events: true,
        supports_controller_events: true,
        supports_note_pressure_expression: true,
        supports_note_timbre_expression: false,
        supports_note_tuning_expression: false,
        supports_mpe: false,
        midi2_posture: RuntimeControllerExpressionMidi2Posture::Unsupported,
        control_surface_guarded: true,
        summary:
            "transport-clock=true controller-events=true pressure=true control-surface=guarded"
                .into(),
    };
    let derived = RuntimeControlSurfaceSnapshot::from_external_midi_snapshot(
        &RuntimeExternalMidiEndpointGraphSnapshot {
            discovery_state: RuntimeExternalMidiDiscoveryState::Enumerated,
            graph_state: RuntimeExternalMidiGraphState::Stable,
            live_ownership:
                RuntimeExternalMidiLiveOwnershipSummary::detached_without_backend_context(),
            provider_name: "control-surface-provider".into(),
            device_count: 1,
            endpoint_count: 1,
            input_endpoint_count: 1,
            output_endpoint_count: 1,
            duplex_endpoint_count: 1,
            active_route_count: 1,
            guarded_route_count: 1,
            devices: vec![RuntimeExternalMidiDeviceDescriptor {
                device_id: "device:surface".into(),
                device_name: "Surface".into(),
                lifecycle_state: RuntimeExternalMidiDeviceLifecycleState::Discovered,
                endpoint_count: 1,
                summary: "surface device".into(),
            }],
            endpoints: vec![RuntimeExternalMidiEndpointDescriptor {
                endpoint_id: "endpoint:surface".into(),
                endpoint_name: "Surface Duplex".into(),
                device_id: "device:surface".into(),
                direction: RuntimeExternalMidiEndpointDirection::Duplex,
                lifecycle_state: RuntimeExternalMidiEndpointLifecycleState::Active,
                route_state: RuntimeExternalMidiRouteState::DuplexObserved,
                capability,
                summary: "surface endpoint".into(),
            }],
            summary: "control surface external midi".into(),
        },
    );
    assert_eq!(
        derived.graph_state,
        RuntimeControlSurfaceGraphState::Guarded
    );
    assert_eq!(derived.device_count, 1);
    assert_eq!(derived.mapped_device_count, 1);
    assert_eq!(derived.feedback_ready_device_count, 0);
    assert_eq!(derived.guarded_device_count, 1);
    assert_eq!(
        derived.devices[0].transport_posture,
        RuntimeControlSurfaceTransportPosture::Guarded
    );
    assert_eq!(
        derived.devices[0].mapping_posture,
        RuntimeControlSurfaceMappingPosture::Guarded
    );
    assert_eq!(
        derived.devices[0].feedback_readiness,
        RuntimeControlSurfaceFeedbackReadiness::Guarded
    );
    assert!(derived.devices[0].capability.supports_widened_expression);
}

#[test]
fn runtime_advanced_hardware_snapshot_derives_from_control_surface_baselines() {
    let unavailable = RuntimeAdvancedHardwareSnapshot::from_control_surface_snapshot(
        &RuntimeControlSurfaceSnapshot::unavailable(),
    );
    assert_eq!(
        unavailable.discovery_state,
        RuntimeExternalMidiDiscoveryState::Unavailable
    );
    assert_eq!(
        unavailable.graph_state,
        RuntimeAdvancedHardwareGraphState::Unavailable
    );
    assert_eq!(unavailable.device_count, 0);

    let empty = RuntimeAdvancedHardwareSnapshot::from_control_surface_snapshot(
        &RuntimeControlSurfaceSnapshot::empty("signal-host-local"),
    );
    assert_eq!(
        empty.discovery_state,
        RuntimeExternalMidiDiscoveryState::Idle
    );
    assert_eq!(empty.graph_state, RuntimeAdvancedHardwareGraphState::Empty);
    assert_eq!(empty.provider_name, "signal-host-local");
    assert_eq!(empty.device_count, 0);
    assert_eq!(empty.display_transport_device_count, 0);
    assert_eq!(empty.motor_transport_device_count, 0);
    assert_eq!(empty.haptic_transport_device_count, 0);
    assert_eq!(empty.scene_mapping_device_count, 0);
    assert_eq!(empty.feedback_page_device_count, 0);
    assert_eq!(empty.safe_action_graph_device_count, 0);

    let advanced = RuntimeAdvancedHardwareSnapshot::from_control_surface_snapshot(
        &RuntimeControlSurfaceSnapshot {
            discovery_state: RuntimeExternalMidiDiscoveryState::Enumerated,
            graph_state: RuntimeControlSurfaceGraphState::Guarded,
            provider_name: "advanced-hardware-provider".into(),
            device_count: 1,
            mapped_device_count: 1,
            feedback_ready_device_count: 0,
            guarded_device_count: 1,
            devices: vec![RuntimeControlSurfaceDeviceDescriptor {
                device_id: "device:surface".into(),
                device_name: "Surface".into(),
                transport_posture: RuntimeControlSurfaceTransportPosture::Guarded,
                mapping_posture: RuntimeControlSurfaceMappingPosture::Guarded,
                feedback_readiness: RuntimeControlSurfaceFeedbackReadiness::Guarded,
                capability: RuntimeControlSurfaceCapabilitySummary {
                    supports_transport_control: true,
                    supports_mapping_input: true,
                    supports_feedback_output: true,
                    supports_widened_expression: true,
                    summary: "guarded control surface".into(),
                },
                summary: "guarded surface".into(),
            }],
            summary: "advanced control surface".into(),
        },
    );
    assert_eq!(
        advanced.graph_state,
        RuntimeAdvancedHardwareGraphState::Guarded
    );
    assert_eq!(advanced.device_count, 1);
    assert_eq!(advanced.portable_device_count, 0);
    assert_eq!(advanced.guarded_device_count, 1);
    assert_eq!(advanced.context_only_device_count, 0);
    assert_eq!(advanced.denied_device_count, 0);
    assert_eq!(advanced.feedback_channel_device_count, 1);
    assert_eq!(advanced.display_transport_device_count, 1);
    assert_eq!(advanced.motor_transport_device_count, 0);
    assert_eq!(advanced.haptic_transport_device_count, 0);
    assert_eq!(advanced.scene_mapping_device_count, 1);
    assert_eq!(advanced.feedback_page_device_count, 1);
    assert_eq!(advanced.safe_action_graph_device_count, 1);
    assert_eq!(
        advanced.devices[0].scripting_safe_posture,
        RuntimeScriptingSafeDevicePolicyPosture::Guarded
    );
    assert_eq!(
        advanced.devices[0].feedback_channel_posture,
        RuntimeGuardedFeedbackChannelPosture::Guarded
    );
    assert!(advanced.devices[0].capability.supports_display_feedback);
    assert!(advanced.devices[0].capability.supports_bank_navigation);
    assert!(advanced.devices[0].capability.supports_macro_triggers);
    assert!(
        advanced.devices[0]
            .capability
            .supports_device_state_observation
    );
    assert_eq!(
        advanced.devices[0].display_transport_posture,
        RuntimeDisplayTransportPosture::GuardedDisplay
    );
    assert_eq!(
        advanced.devices[0].display_content_class,
        RuntimeDisplayContentClass::GuardedVendorDisplay
    );
    assert_eq!(
        advanced.devices[0].motor_transport_posture,
        RuntimeMotorTransportPosture::NoMotorTransport
    );
    assert_eq!(
        advanced.devices[0].haptic_transport_posture,
        RuntimeHapticTransportPosture::NoHapticTransport
    );
    assert_eq!(
        advanced.devices[0].feedback_authority,
        RuntimeAdvancedControlFeedbackAuthority::RuntimeDefault
    );
    assert_eq!(
        advanced.devices[0].feedback_outcome,
        RuntimeAdvancedControlFeedbackOutcome::CollapseToGuardedFeedback
    );
    assert_eq!(
        advanced.devices[0].scene_mapping_posture,
        RuntimeSceneMappingPosture::GuardedSceneMapping
    );
    assert_eq!(
        advanced.devices[0].feedback_page_posture,
        RuntimeFeedbackPagePosture::GuardedFeedbackPages
    );
    assert_eq!(
        advanced.devices[0].feedback_page_class,
        RuntimeFeedbackPageClass::GuardedVendorPage
    );
    assert_eq!(
        advanced.devices[0].safe_action_graph_posture,
        RuntimeSafeActionGraphPosture::GuardedSafeActionGraph
    );
    assert_eq!(
        advanced.devices[0].action_authority,
        RuntimeControlSurfaceWorkflowAuthority::RuntimeDefault
    );
    assert_eq!(
        advanced.devices[0].safe_action_outcome,
        RuntimeSafeActionOutcome::CollapseToGuardedAction
    );
    assert!(advanced.devices[0]
        .capability
        .action_classes
        .contains(&RuntimeAdvancedHardwareActionClass::DisplayFeedback));
    assert!(advanced.devices[0]
        .capability
        .action_classes
        .contains(&RuntimeAdvancedHardwareActionClass::MacroTrigger));
}

#[test]
fn runtime_stretch_engine_snapshot_derives_from_clip_processing_baselines() {
    let pipeline = RuntimeClipProcessingPipelineSnapshot {
        clip_count: 4,
        ready_clip_count: 3,
        pending_media_clip_count: 0,
        pending_warp_clip_count: 0,
        invalid_clip_count: 1,
        faded_clip_count: 0,
        gain_shaped_clip_count: 0,
        warped_clip_count: 3,
        treatment_stage_count: 3,
        clips: vec![
            RuntimeClipProcessingSnapshot {
                clip_id: "clip:stretch-disabled".into(),
                media_asset_id: None,
                warp_mode: RuntimeWarpMode::Off,
                start_samples: 0,
                duration_samples: 64,
                fade_in: RuntimeClipFadeEnvelope::default(),
                fade_out: RuntimeClipFadeEnvelope::default(),
                fade_in_end_samples: 0,
                fade_out_start_samples: 64,
                clip_gain: RuntimeClipGainEnvelope::default(),
                treatment_stages: Vec::new(),
                realized_warp_ratio: None,
                project_tempo_source: None,
                project_tempo_segment_id: None,
                readiness: RuntimeClipProcessingReadiness::Ready,
                last_error: None,
                summary: "disabled clip".into(),
            },
            RuntimeClipProcessingSnapshot {
                clip_id: "clip:stretch-ratio".into(),
                media_asset_id: Some("asset:ratio".into()),
                warp_mode: RuntimeWarpMode::Repitch,
                start_samples: 0,
                duration_samples: 64,
                fade_in: RuntimeClipFadeEnvelope::default(),
                fade_out: RuntimeClipFadeEnvelope::default(),
                fade_in_end_samples: 0,
                fade_out_start_samples: 64,
                clip_gain: RuntimeClipGainEnvelope::default(),
                treatment_stages: vec![RuntimeClipProcessingStage::Warp],
                realized_warp_ratio: Some(0.75),
                project_tempo_source: Some(RuntimeTempoSource::TransportProjection),
                project_tempo_segment_id: None,
                readiness: RuntimeClipProcessingReadiness::Ready,
                last_error: None,
                summary: "ratio clip".into(),
            },
            RuntimeClipProcessingSnapshot {
                clip_id: "clip:stretch-sample-domain".into(),
                media_asset_id: Some("asset:sample-domain".into()),
                warp_mode: RuntimeWarpMode::ElastiqueDraft,
                start_samples: 0,
                duration_samples: 64,
                fade_in: RuntimeClipFadeEnvelope::default(),
                fade_out: RuntimeClipFadeEnvelope::default(),
                fade_in_end_samples: 0,
                fade_out_start_samples: 64,
                clip_gain: RuntimeClipGainEnvelope::default(),
                treatment_stages: vec![RuntimeClipProcessingStage::Warp],
                realized_warp_ratio: Some(1.5),
                project_tempo_source: Some(RuntimeTempoSource::TransportProjection),
                project_tempo_segment_id: None,
                readiness: RuntimeClipProcessingReadiness::Ready,
                last_error: None,
                summary: "sample-domain clip".into(),
            },
            RuntimeClipProcessingSnapshot {
                clip_id: "clip:stretch-fallback".into(),
                media_asset_id: Some("asset:fallback".into()),
                warp_mode: RuntimeWarpMode::ElastiqueDraft,
                start_samples: 0,
                duration_samples: 64,
                fade_in: RuntimeClipFadeEnvelope::default(),
                fade_out: RuntimeClipFadeEnvelope::default(),
                fade_in_end_samples: 0,
                fade_out_start_samples: 64,
                clip_gain: RuntimeClipGainEnvelope::default(),
                treatment_stages: vec![RuntimeClipProcessingStage::Warp],
                realized_warp_ratio: Some(0.6),
                project_tempo_source: Some(RuntimeTempoSource::TransportProjection),
                project_tempo_segment_id: None,
                readiness: RuntimeClipProcessingReadiness::Invalid,
                last_error: Some("outside baseline support".into()),
                summary: "fallback clip".into(),
            },
        ],
        summary: "clip processing stretch baseline".into(),
    };

    let stretch = RuntimeStretchEngineSnapshot::from_clip_processing_pipeline(&pipeline);

    assert_eq!(stretch.clip_count, 4);
    assert_eq!(stretch.disabled_clip_count, 1);
    assert_eq!(stretch.ready_clip_count, 2);
    assert_eq!(stretch.pending_media_clip_count, 0);
    assert_eq!(stretch.pending_warp_clip_count, 0);
    assert_eq!(stretch.degraded_clip_count, 1);
    assert_eq!(stretch.sample_domain_clip_count, 1);
    assert_eq!(stretch.ratio_only_clip_count, 1);
    assert_eq!(stretch.fallback_clip_count, 1);
    assert_eq!(
        stretch.clips[0].engine_class,
        RuntimeStretchEngineClass::Disabled
    );
    assert_eq!(
        stretch.clips[1].engine_class,
        RuntimeStretchEngineClass::RatioOnly
    );
    assert_eq!(stretch.clips[1].readiness, RuntimeStretchReadiness::Ready);
    assert_eq!(
        stretch.clips[2].engine_class,
        RuntimeStretchEngineClass::SampleDomain
    );
    assert_eq!(stretch.clips[2].readiness, RuntimeStretchReadiness::Ready);
    assert_eq!(
        stretch.clips[3].engine_class,
        RuntimeStretchEngineClass::Fallback
    );
    assert_eq!(
        stretch.clips[3].readiness,
        RuntimeStretchReadiness::Degraded
    );
    assert_eq!(
        stretch.clips[3].fallback_kind,
        RuntimeStretchFallbackKind::RatioOnly
    );
    assert!(stretch.summary.contains("sample_domain=1"));
    assert!(stretch.summary.contains("fallback=1"));
}

#[test]
fn runtime_observation_report_render_json_surfaces_external_midi_snapshot() {
    let runtime = SignalRuntime::new(RuntimeConfig::server(48_000, 256));
    let recorder = RuntimeEventRecorder::default();
    let report = RuntimeObservationReport::capture(&runtime, &recorder)
        .with_external_midi_snapshot(RuntimeExternalMidiEndpointGraphSnapshot::empty(
            "signal-host-server",
        ));

    assert_eq!(
        report.external_midi_snapshot.discovery_state,
        RuntimeExternalMidiDiscoveryState::Idle
    );
    assert_eq!(
        report.external_midi_snapshot.graph_state,
        RuntimeExternalMidiGraphState::Empty
    );

    let compact = report.render_compact();
    assert!(compact.contains("external_midi=Idle/Empty"));

    let json = report.render_json();
    assert!(json.contains("\"external_midi_snapshot\":{"));
    assert!(json.contains("\"control_surface_snapshot\":{"));
    assert!(json.contains("\"advanced_hardware_snapshot\":{"));
    assert!(json.contains("\"display_transport_device_count\":0"));
    assert!(json.contains("\"motor_transport_device_count\":0"));
    assert!(json.contains("\"haptic_transport_device_count\":0"));
    assert!(json.contains("\"scene_mapping_device_count\":0"));
    assert!(json.contains("\"feedback_page_device_count\":0"));
    assert!(json.contains("\"safe_action_graph_device_count\":0"));
    assert!(json.contains("\"stretch_engine_snapshot\":{\"clip_count\":0"));
    assert!(json.contains("\"discovery_state\":\"Idle\""));
    assert!(json.contains("\"graph_state\":\"Empty\""));
    assert!(json.contains("\"provider_name\":\"signal-host-server\""));
}

#[test]
fn runtime_host_hardware_summary_classifies_linux_backend_baselines() {
    let alsa = RuntimeHostHardwareSummary {
        backend_identity: HardwareBackendIdentity::Linux(LinuxAudioBackendKind::Alsa),
        backend_name: "alsa".into(),
        linux_backend_identity: RuntimeHostHardwareSummary::classify_linux_backend_identity(
            HardwareBackendIdentity::Linux(LinuxAudioBackendKind::Alsa),
        ),
        linux_backend_portability: RuntimeHostHardwareSummary::classify_linux_backend_portability(
            HardwareBackendIdentity::Linux(LinuxAudioBackendKind::Alsa),
            false,
            BackendHealth::Healthy,
            0,
            0,
            0,
        ),
        device_id: "alsa:default-output".into(),
        device_name: "ALSA Default Output".into(),
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
    };
    assert_eq!(
        alsa.linux_backend_identity,
        RuntimeLinuxAudioBackendIdentity::Alsa
    );
    assert_eq!(
        alsa.linux_backend_portability,
        RuntimeLinuxAudioBackendPortabilityBand::Portable
    );

    let jack = RuntimeHostHardwareSummary {
        backend_identity: HardwareBackendIdentity::Linux(LinuxAudioBackendKind::Jack),
        backend_name: "jack".into(),
        linux_backend_identity: RuntimeHostHardwareSummary::classify_linux_backend_identity(
            HardwareBackendIdentity::Linux(LinuxAudioBackendKind::Jack),
        ),
        linux_backend_portability: RuntimeHostHardwareSummary::classify_linux_backend_portability(
            HardwareBackendIdentity::Linux(LinuxAudioBackendKind::Jack),
            true,
            BackendHealth::Recovering,
            1,
            1,
            0,
        ),
        device_id: "jack:graph-main".into(),
        device_name: "JACK Graph Main".into(),
        sample_rate: 48_000,
        buffer_size: 128,
        input_channels: 2,
        output_channels: 2,
        sample_format: AudioSampleFormat::F32,
        simulated: true,
        backend_health: BackendHealth::Recovering,
        xrun_count: 2,
        callback_overrun_count: 0,
        device_loss_count: 1,
        restart_attempt_count: 1,
        restart_failure_count: 0,
    };
    assert_eq!(
        jack.linux_backend_identity,
        RuntimeLinuxAudioBackendIdentity::Jack
    );
    assert_eq!(
        jack.linux_backend_portability,
        RuntimeLinuxAudioBackendPortabilityBand::Guarded
    );

    let not_linux = RuntimeHostHardwareSummary::classify_linux_backend_portability(
        HardwareBackendIdentity::CoreAudio,
        false,
        BackendHealth::Healthy,
        0,
        0,
        0,
    );
    assert_eq!(
        not_linux,
        RuntimeLinuxAudioBackendPortabilityBand::Unsupported
    );
}

#[test]
fn runtime_host_io_classifies_linux_clocking_duplex_and_endpoint_parity() {
    let alsa_identity = RuntimeHostHardwareSummary::classify_linux_backend_identity(
        HardwareBackendIdentity::Linux(LinuxAudioBackendKind::Alsa),
    );
    assert_eq!(
        RuntimeHostIoSummary::classify_linux_clocking_parity(
            RuntimeHostIoSummary::linux_parity_input(
                alsa_identity,
                BackendHealth::Healthy,
                RuntimeHostAudioStreamState::Running,
                RuntimeHostClockDomain::SameClock,
                RuntimeHostClockFallbackState::Direct,
                RuntimeHostClockTransitionState::Stable,
                RuntimeHostClockDriftState::Stable,
                RuntimeHostClockDiscontinuityState::Continuous,
                RuntimeHostDuplexMismatchState::Aligned,
                RuntimeHostEndpointTopology::Duplex,
                false,
            )
        ),
        RuntimeLinuxAudioBackendClockingParityBand::Portable
    );
    assert_eq!(
        RuntimeHostIoSummary::classify_linux_duplex_parity(
            RuntimeHostIoSummary::linux_parity_input(
                alsa_identity,
                BackendHealth::Healthy,
                RuntimeHostAudioStreamState::Running,
                RuntimeHostClockDomain::SameClock,
                RuntimeHostClockFallbackState::Direct,
                RuntimeHostClockTransitionState::Stable,
                RuntimeHostClockDriftState::Stable,
                RuntimeHostClockDiscontinuityState::Continuous,
                RuntimeHostDuplexMismatchState::Aligned,
                RuntimeHostEndpointTopology::Duplex,
                false,
            )
        ),
        RuntimeLinuxAudioBackendDuplexParityState::Aligned
    );
    assert_eq!(
        RuntimeHostIoSummary::classify_linux_endpoint_topology_parity(
            alsa_identity,
            BackendHealth::Healthy,
            RuntimeHostClockTransitionState::Stable,
            RuntimeHostClockDiscontinuityState::Continuous,
            RuntimeHostDuplexMismatchState::Aligned,
            RuntimeHostEndpointTopology::Duplex,
            false,
        ),
        RuntimeLinuxAudioBackendEndpointTopologyParityState::Portable
    );

    let jack_identity = RuntimeHostHardwareSummary::classify_linux_backend_identity(
        HardwareBackendIdentity::Linux(LinuxAudioBackendKind::Jack),
    );
    assert_eq!(
        RuntimeHostIoSummary::classify_linux_clocking_parity(
            RuntimeHostIoSummary::linux_parity_input(
                jack_identity,
                BackendHealth::Recovering,
                RuntimeHostAudioStreamState::Running,
                RuntimeHostClockDomain::Aggregate,
                RuntimeHostClockFallbackState::RuntimeResampled,
                RuntimeHostClockTransitionState::EnteredAggregateClock,
                RuntimeHostClockDriftState::AggregateManaged,
                RuntimeHostClockDiscontinuityState::Reconfigured,
                RuntimeHostDuplexMismatchState::CrossClockDiverged,
                RuntimeHostEndpointTopology::Aggregate,
                false,
            )
        ),
        RuntimeLinuxAudioBackendClockingParityBand::Guarded
    );
    assert_eq!(
        RuntimeHostIoSummary::classify_linux_duplex_parity(
            RuntimeHostIoSummary::linux_parity_input(
                jack_identity,
                BackendHealth::Recovering,
                RuntimeHostAudioStreamState::Running,
                RuntimeHostClockDomain::Aggregate,
                RuntimeHostClockFallbackState::RuntimeResampled,
                RuntimeHostClockTransitionState::EnteredAggregateClock,
                RuntimeHostClockDriftState::AggregateManaged,
                RuntimeHostClockDiscontinuityState::Reconfigured,
                RuntimeHostDuplexMismatchState::CrossClockDiverged,
                RuntimeHostEndpointTopology::Aggregate,
                false,
            )
        ),
        RuntimeLinuxAudioBackendDuplexParityState::Guarded
    );
    assert_eq!(
        RuntimeHostIoSummary::classify_linux_endpoint_topology_parity(
            jack_identity,
            BackendHealth::Recovering,
            RuntimeHostClockTransitionState::EnteredAggregateClock,
            RuntimeHostClockDiscontinuityState::Reconfigured,
            RuntimeHostDuplexMismatchState::CrossClockDiverged,
            RuntimeHostEndpointTopology::Aggregate,
            false,
        ),
        RuntimeLinuxAudioBackendEndpointTopologyParityState::Guarded
    );

    let not_linux_identity = RuntimeHostHardwareSummary::classify_linux_backend_identity(
        HardwareBackendIdentity::CoreAudio,
    );
    assert_eq!(
        RuntimeHostIoSummary::classify_linux_clocking_parity(
            RuntimeHostIoSummary::linux_parity_input(
                not_linux_identity,
                BackendHealth::Healthy,
                RuntimeHostAudioStreamState::Running,
                RuntimeHostClockDomain::SameClock,
                RuntimeHostClockFallbackState::Direct,
                RuntimeHostClockTransitionState::Stable,
                RuntimeHostClockDriftState::Stable,
                RuntimeHostClockDiscontinuityState::Continuous,
                RuntimeHostDuplexMismatchState::Aligned,
                RuntimeHostEndpointTopology::Duplex,
                false,
            )
        ),
        RuntimeLinuxAudioBackendClockingParityBand::Unsupported
    );
    assert_eq!(
        RuntimeHostIoSummary::classify_linux_duplex_parity(
            RuntimeHostIoSummary::linux_parity_input(
                not_linux_identity,
                BackendHealth::Healthy,
                RuntimeHostAudioStreamState::Running,
                RuntimeHostClockDomain::SameClock,
                RuntimeHostClockFallbackState::Direct,
                RuntimeHostClockTransitionState::Stable,
                RuntimeHostClockDriftState::Stable,
                RuntimeHostClockDiscontinuityState::Continuous,
                RuntimeHostDuplexMismatchState::Aligned,
                RuntimeHostEndpointTopology::Duplex,
                false,
            )
        ),
        RuntimeLinuxAudioBackendDuplexParityState::Unsupported
    );
    assert_eq!(
        RuntimeHostIoSummary::classify_linux_endpoint_topology_parity(
            not_linux_identity,
            BackendHealth::Healthy,
            RuntimeHostClockTransitionState::Stable,
            RuntimeHostClockDiscontinuityState::Continuous,
            RuntimeHostDuplexMismatchState::Aligned,
            RuntimeHostEndpointTopology::Duplex,
            false,
        ),
        RuntimeLinuxAudioBackendEndpointTopologyParityState::Unsupported
    );
}

#[test]
fn runtime_linux_backend_session_snapshot_classifies_live_ownership_baselines() {
    let alsa = RuntimeLinuxBackendSessionSnapshot::from_host_io(&linux_host_io_summary(
        HardwareBackendIdentity::Linux(LinuxAudioBackendKind::Alsa),
        RuntimeHostLifecycleOwnership::HostDrivenCallback,
        RuntimeHostAudioStreamState::Running,
        BackendHealth::Healthy,
        0,
        0,
        0,
    ));
    assert_eq!(
        alsa.backend_identity,
        RuntimeLinuxAudioBackendIdentity::Alsa
    );
    assert_eq!(
        alsa.ownership,
        RuntimeLinuxBackendSessionOwnership::HostBrokeredCallback
    );
    assert_eq!(
        alsa.lifecycle_state,
        RuntimeLinuxBackendSessionLifecycleState::Running
    );
    assert_eq!(
        alsa.device_claim_posture,
        RuntimeLinuxBackendDeviceClaimPosture::DirectClaim
    );
    assert_eq!(
        alsa.session_role,
        RuntimeLinuxBackendSessionRole::PrimaryAudioIo
    );
    assert_eq!(
        alsa.ownership_fallback,
        RuntimeLinuxBackendOwnershipFallbackState::Direct
    );

    let jack = RuntimeLinuxBackendSessionSnapshot::from_host_io(&linux_host_io_summary(
        HardwareBackendIdentity::Linux(LinuxAudioBackendKind::Jack),
        RuntimeHostLifecycleOwnership::BackendManagedCallback,
        RuntimeHostAudioStreamState::Running,
        BackendHealth::Healthy,
        0,
        0,
        0,
    ));
    assert_eq!(
        jack.backend_identity,
        RuntimeLinuxAudioBackendIdentity::Jack
    );
    assert_eq!(
        jack.ownership,
        RuntimeLinuxBackendSessionOwnership::BackendManagedGraph
    );
    assert_eq!(
        jack.device_claim_posture,
        RuntimeLinuxBackendDeviceClaimPosture::SharedGraph
    );
    assert_eq!(
        jack.ownership_fallback,
        RuntimeLinuxBackendOwnershipFallbackState::BackendManagedGuarded
    );

    let pipewire_recovering =
        RuntimeLinuxBackendSessionSnapshot::from_host_io(&linux_host_io_summary(
            HardwareBackendIdentity::Linux(LinuxAudioBackendKind::PipeWire),
            RuntimeHostLifecycleOwnership::BackendManagedCallback,
            RuntimeHostAudioStreamState::Faulted,
            BackendHealth::Recovering,
            1,
            1,
            1,
        ));
    assert_eq!(
        pipewire_recovering.backend_identity,
        RuntimeLinuxAudioBackendIdentity::PipeWire
    );
    assert_eq!(
        pipewire_recovering.lifecycle_state,
        RuntimeLinuxBackendSessionLifecycleState::Recovering
    );
    assert_eq!(
        pipewire_recovering.device_claim_posture,
        RuntimeLinuxBackendDeviceClaimPosture::Lost
    );
    assert_eq!(
        pipewire_recovering.session_role,
        RuntimeLinuxBackendSessionRole::FallbackContinuation
    );
    assert_eq!(
        pipewire_recovering.ownership_fallback,
        RuntimeLinuxBackendOwnershipFallbackState::RecoveryConstrained
    );
}

#[test]
fn runtime_pipewire_alsa_parity_snapshot_derives_runtime_owned_parity_baselines() {
    let alsa_host_io = linux_host_io_summary(
        HardwareBackendIdentity::Linux(LinuxAudioBackendKind::Alsa),
        RuntimeHostLifecycleOwnership::HostDrivenCallback,
        RuntimeHostAudioStreamState::Running,
        BackendHealth::Healthy,
        0,
        0,
        0,
    );
    let alsa_linux_session = RuntimeLinuxBackendSessionSnapshot::from_host_io(&alsa_host_io);
    let alsa = RuntimePipeWireAlsaParitySnapshot::from_host_io_and_linux_session(
        &alsa_host_io,
        &alsa_linux_session,
    );
    assert_eq!(
        alsa.session_role_parity,
        RuntimePipeWireAlsaSessionRoleParity::PrimaryAudioIo
    );
    assert_eq!(
        alsa.device_claim_parity,
        RuntimePipeWireAlsaDeviceClaimParity::DirectClaim
    );
    assert_eq!(
        alsa.stream_policy_parity,
        RuntimePipeWireAlsaStreamPolicyParity::DirectHostCallback
    );
    assert_eq!(
        alsa.guarded_state,
        RuntimePipeWireAlsaGuardedParityState::Direct
    );

    let pipewire_host_io = linux_host_io_summary(
        HardwareBackendIdentity::Linux(LinuxAudioBackendKind::PipeWire),
        RuntimeHostLifecycleOwnership::BackendManagedCallback,
        RuntimeHostAudioStreamState::Running,
        BackendHealth::Healthy,
        0,
        0,
        0,
    );
    let pipewire_linux_session =
        RuntimeLinuxBackendSessionSnapshot::from_host_io(&pipewire_host_io);
    let pipewire = RuntimePipeWireAlsaParitySnapshot::from_host_io_and_linux_session(
        &pipewire_host_io,
        &pipewire_linux_session,
    );
    assert_eq!(
        pipewire.session_role_parity,
        RuntimePipeWireAlsaSessionRoleParity::PrimaryAudioIo
    );
    assert_eq!(
        pipewire.device_claim_parity,
        RuntimePipeWireAlsaDeviceClaimParity::SharedGraph
    );
    assert_eq!(
        pipewire.stream_policy_parity,
        RuntimePipeWireAlsaStreamPolicyParity::BackendManagedGraph
    );
    assert_eq!(
        pipewire.guarded_state,
        RuntimePipeWireAlsaGuardedParityState::BackendManaged
    );

    let pipewire_recovering_host_io = linux_host_io_summary(
        HardwareBackendIdentity::Linux(LinuxAudioBackendKind::PipeWire),
        RuntimeHostLifecycleOwnership::BackendManagedCallback,
        RuntimeHostAudioStreamState::Faulted,
        BackendHealth::Recovering,
        1,
        1,
        1,
    );
    let pipewire_recovering_linux_session =
        RuntimeLinuxBackendSessionSnapshot::from_host_io(&pipewire_recovering_host_io);
    let pipewire_recovering = RuntimePipeWireAlsaParitySnapshot::from_host_io_and_linux_session(
        &pipewire_recovering_host_io,
        &pipewire_recovering_linux_session,
    );
    assert_eq!(
        pipewire_recovering.session_role_parity,
        RuntimePipeWireAlsaSessionRoleParity::FallbackContinuation
    );
    assert_eq!(
        pipewire_recovering.device_claim_parity,
        RuntimePipeWireAlsaDeviceClaimParity::Lost
    );
    assert_eq!(
        pipewire_recovering.stream_policy_parity,
        RuntimePipeWireAlsaStreamPolicyParity::Restarting
    );
    assert_eq!(
        pipewire_recovering.guarded_state,
        RuntimePipeWireAlsaGuardedParityState::RecoveryGuarded
    );

    let jack_host_io = linux_host_io_summary(
        HardwareBackendIdentity::Linux(LinuxAudioBackendKind::Jack),
        RuntimeHostLifecycleOwnership::BackendManagedCallback,
        RuntimeHostAudioStreamState::Running,
        BackendHealth::Healthy,
        0,
        0,
        0,
    );
    let jack_linux_session = RuntimeLinuxBackendSessionSnapshot::from_host_io(&jack_host_io);
    let jack = RuntimePipeWireAlsaParitySnapshot::from_host_io_and_linux_session(
        &jack_host_io,
        &jack_linux_session,
    );
    assert_eq!(
        jack.session_role_parity,
        RuntimePipeWireAlsaSessionRoleParity::NotPipeWireOrAlsa
    );
    assert_eq!(
        jack.device_claim_parity,
        RuntimePipeWireAlsaDeviceClaimParity::NotPipeWireOrAlsa
    );
    assert_eq!(
        jack.stream_policy_parity,
        RuntimePipeWireAlsaStreamPolicyParity::NotPipeWireOrAlsa
    );
    assert_eq!(
        jack.guarded_state,
        RuntimePipeWireAlsaGuardedParityState::NotPipeWireOrAlsa
    );
}

#[test]
fn runtime_jack_coordination_snapshot_derives_from_linux_session_and_transport_baselines() {
    let not_jack = RuntimeJackCoordinationSnapshot::from_host_io_and_transport_session(
        &linux_host_io_summary(
            HardwareBackendIdentity::CoreAudio,
            RuntimeHostLifecycleOwnership::HostDrivenCallback,
            RuntimeHostAudioStreamState::Running,
            BackendHealth::Healthy,
            0,
            0,
            0,
        ),
        &transport_session_summary(
            TransportSessionState::Detached,
            false,
            TransportHeartbeatFreshness::Unknown,
            TransportDispatchState::Idle,
            0,
            0,
            0,
        ),
    );
    assert_eq!(
        not_jack.transport_posture,
        RuntimeJackTransportPosture::NotJack
    );
    assert_eq!(
        not_jack.graph_state,
        RuntimeJackGraphCoordinationState::NotJack
    );
    assert_eq!(not_jack.client_role, RuntimeJackClientRole::NotJack);
    assert_eq!(
        not_jack.guarded_state,
        RuntimeJackGuardedCoordinationState::NotJack
    );

    let detached = RuntimeJackCoordinationSnapshot::from_host_io_and_transport_session(
        &linux_host_io_summary(
            HardwareBackendIdentity::Linux(LinuxAudioBackendKind::Jack),
            RuntimeHostLifecycleOwnership::BackendManagedCallback,
            RuntimeHostAudioStreamState::Running,
            BackendHealth::Healthy,
            0,
            0,
            0,
        ),
        &transport_session_summary(
            TransportSessionState::Detached,
            false,
            TransportHeartbeatFreshness::Unknown,
            TransportDispatchState::Idle,
            0,
            0,
            0,
        ),
    );
    assert_eq!(
        detached.transport_posture,
        RuntimeJackTransportPosture::Detached
    );
    assert_eq!(
        detached.graph_state,
        RuntimeJackGraphCoordinationState::AttachedGuarded
    );
    assert_eq!(detached.client_role, RuntimeJackClientRole::PrimaryAudioIo);
    assert_eq!(
        detached.guarded_state,
        RuntimeJackGuardedCoordinationState::GraphGuarded
    );

    let following = RuntimeJackCoordinationSnapshot::from_host_io_and_transport_session(
        &linux_host_io_summary(
            HardwareBackendIdentity::Linux(LinuxAudioBackendKind::Jack),
            RuntimeHostLifecycleOwnership::BackendManagedCallback,
            RuntimeHostAudioStreamState::Running,
            BackendHealth::Healthy,
            0,
            0,
            0,
        ),
        &transport_session_summary(
            TransportSessionState::AttachActive,
            true,
            TransportHeartbeatFreshness::Fresh,
            TransportDispatchState::Completed,
            1,
            0,
            0,
        ),
    );
    assert_eq!(
        following.transport_posture,
        RuntimeJackTransportPosture::FollowingExternal
    );
    assert_eq!(
        following.client_role,
        RuntimeJackClientRole::TransportFollower
    );
    assert_eq!(
        following.guarded_state,
        RuntimeJackGuardedCoordinationState::TransportGuarded
    );

    let recovering = RuntimeJackCoordinationSnapshot::from_host_io_and_transport_session(
        &linux_host_io_summary(
            HardwareBackendIdentity::Linux(LinuxAudioBackendKind::Jack),
            RuntimeHostLifecycleOwnership::BackendManagedCallback,
            RuntimeHostAudioStreamState::Faulted,
            BackendHealth::Recovering,
            1,
            1,
            0,
        ),
        &transport_session_summary(
            TransportSessionState::DetachFaulted,
            true,
            TransportHeartbeatFreshness::Missed,
            TransportDispatchState::TimedOut,
            2,
            1,
            1,
        ),
    );
    assert_eq!(
        recovering.transport_posture,
        RuntimeJackTransportPosture::Guarded
    );
    assert_eq!(
        recovering.graph_state,
        RuntimeJackGraphCoordinationState::Recovering
    );
    assert_eq!(
        recovering.guarded_state,
        RuntimeJackGuardedCoordinationState::Recovering
    );
}

#[test]
fn runtime_device_supervision_snapshot_tracks_recovered_device_episode() {
    let effective_config = EffectiveRuntimeConfig {
        sample_rate: SampleRate(48_000),
        block_size: 256,
        anticipative_enabled: true,
        safe_mode_enabled: false,
        active_output_device: Some("device:main".into()),
    };
    let supervision_snapshot = RuntimeSupervisionSnapshot {
        watchdog_restart_count: 1,
        safe_mode_enabled: false,
        xrun_overload_active: false,
        last_watchdog_trigger: Some(RuntimeWatchdogTrigger::HeartbeatMisses),
        last_sandbox_id: Some("sandbox:main".into()),
        last_processing_epoch: Some(7),
    };
    let fault_status = RuntimeFaultStatusSnapshot {
        recovery_state: RuntimeRecoveryState::Steady,
        primary_fault_cause: None,
        active_fault_count: 0,
        xrun_overload_active: false,
        plugin_fault_active: false,
        watchdog_active: false,
        device_loss_active: false,
        transport_fault_active: false,
        missing_plugin_binding_active: false,
        safe_mode_enabled: false,
        restart_count: 0,
        watchdog_restart_count: 1,
        plugin_fault_count: 0,
        transport_faulted_session_count: 0,
        device_loss_count: 1,
        summary: "steady".into(),
    };
    let interruption_summary = RuntimeInterruptionSummary {
        active: false,
        class: RuntimeInterruptionClass::Steady,
        rebindable: false,
        recovery_state: RuntimeRecoveryState::Steady,
        primary_fault_cause: None,
        safe_mode_enabled: false,
        deferred_service_class: None,
        deferred_service_decision: None,
        summary: "steady".into(),
    };
    let host_io = host_io_summary(
        RuntimeHostClockFallbackState::Direct,
        RuntimeHostClockTransitionState::ReturnedToDirect,
        RuntimeHostAudioStreamState::Running,
        BackendHealth::Healthy,
        1,
        0,
        1,
    );

    let snapshot = RuntimeDeviceSupervisionSnapshot::capture(
        &effective_config,
        &supervision_snapshot,
        &fault_status,
        &interruption_summary,
        Some(&host_io),
    );

    assert_eq!(snapshot.state, RuntimeDeviceSupervisionState::Stable);
    assert_eq!(snapshot.restart_state, RuntimeDeviceRestartState::Recovered);
    assert_eq!(
        snapshot.fault_boundary,
        RuntimeDeviceFaultBoundaryState::Clear
    );
    assert_eq!(snapshot.device_loss_count, 1);
    assert_eq!(snapshot.restart_attempt_count, Some(1));
    assert_eq!(snapshot.restart_failure_count, Some(0));
    assert_eq!(snapshot.backend_health, Some(BackendHealth::Healthy));
}

#[test]
fn runtime_device_supervision_snapshot_distinguishes_exhausted_from_faulted() {
    let effective_config = EffectiveRuntimeConfig {
        sample_rate: SampleRate(48_000),
        block_size: 256,
        anticipative_enabled: true,
        safe_mode_enabled: true,
        active_output_device: Some("device:main".into()),
    };
    let supervision_snapshot = RuntimeSupervisionSnapshot {
        watchdog_restart_count: 2,
        safe_mode_enabled: true,
        xrun_overload_active: false,
        last_watchdog_trigger: Some(RuntimeWatchdogTrigger::DeadlineMisses),
        last_sandbox_id: Some("sandbox:main".into()),
        last_processing_epoch: Some(11),
    };
    let exhausted_status = RuntimeFaultStatusSnapshot {
        recovery_state: RuntimeRecoveryState::Recovering,
        primary_fault_cause: Some(RuntimeFaultCause::DeviceLoss),
        active_fault_count: 1,
        xrun_overload_active: false,
        plugin_fault_active: false,
        watchdog_active: false,
        device_loss_active: true,
        transport_fault_active: false,
        missing_plugin_binding_active: false,
        safe_mode_enabled: true,
        restart_count: 0,
        watchdog_restart_count: 2,
        plugin_fault_count: 0,
        transport_faulted_session_count: 0,
        device_loss_count: 1,
        summary: "recovering".into(),
    };
    let exhausted_interruption = RuntimeInterruptionSummary {
        active: true,
        class: RuntimeInterruptionClass::Restartable,
        rebindable: true,
        recovery_state: RuntimeRecoveryState::Recovering,
        primary_fault_cause: Some(RuntimeFaultCause::DeviceLoss),
        safe_mode_enabled: true,
        deferred_service_class: None,
        deferred_service_decision: None,
        summary: "restartable".into(),
    };
    let exhausted_host_io = host_io_summary(
        RuntimeHostClockFallbackState::RecoveryConstrained,
        RuntimeHostClockTransitionState::EnteredRecoveryFallback,
        RuntimeHostAudioStreamState::Faulted,
        BackendHealth::Recovering,
        1,
        1,
        1,
    );

    let exhausted = RuntimeDeviceSupervisionSnapshot::capture(
        &effective_config,
        &supervision_snapshot,
        &exhausted_status,
        &exhausted_interruption,
        Some(&exhausted_host_io),
    );
    assert_eq!(exhausted.state, RuntimeDeviceSupervisionState::Exhausted);
    assert_eq!(
        exhausted.restart_state,
        RuntimeDeviceRestartState::Exhausted
    );
    assert_eq!(
        exhausted.fault_boundary,
        RuntimeDeviceFaultBoundaryState::Exhausted
    );

    let faulted_status = RuntimeFaultStatusSnapshot {
        recovery_state: RuntimeRecoveryState::Faulted,
        primary_fault_cause: Some(RuntimeFaultCause::RuntimeError),
        active_fault_count: 1,
        xrun_overload_active: false,
        plugin_fault_active: false,
        watchdog_active: false,
        device_loss_active: false,
        transport_fault_active: false,
        missing_plugin_binding_active: false,
        safe_mode_enabled: true,
        restart_count: 0,
        watchdog_restart_count: 2,
        plugin_fault_count: 0,
        transport_faulted_session_count: 0,
        device_loss_count: 1,
        summary: "faulted".into(),
    };
    let faulted_interruption = RuntimeInterruptionSummary {
        active: true,
        class: RuntimeInterruptionClass::Terminal,
        rebindable: false,
        recovery_state: RuntimeRecoveryState::Faulted,
        primary_fault_cause: Some(RuntimeFaultCause::RuntimeError),
        safe_mode_enabled: true,
        deferred_service_class: None,
        deferred_service_decision: None,
        summary: "terminal".into(),
    };

    let faulted = RuntimeDeviceSupervisionSnapshot::capture(
        &effective_config,
        &supervision_snapshot,
        &faulted_status,
        &faulted_interruption,
        None,
    );
    assert_eq!(faulted.state, RuntimeDeviceSupervisionState::Faulted);
    assert_eq!(faulted.restart_state, RuntimeDeviceRestartState::Faulted);
    assert_eq!(
        faulted.fault_boundary,
        RuntimeDeviceFaultBoundaryState::Faulted
    );
}
