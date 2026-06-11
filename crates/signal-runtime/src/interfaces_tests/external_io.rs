use super::*;

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
    assert!(!snapshot.partial_availability);
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
}
