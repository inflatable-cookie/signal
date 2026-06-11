use super::super::super::super::*;

#[test]
fn local_host_shared_report_tracks_device_loss_recovery() {
    let runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
    let mut host = LocalRuntimeHost::new(runtime);
    let summary = host
        .boot_with_device_loss_recovery()
        .expect("device loss recovery local host boot");
    let supervisor = host.supervisor_report();
    let report = host.host_supervisor_report();

    assert_eq!(
        summary.execution.last_stop_reason,
        Some(StopReason::DeviceReconfigure)
    );
    assert_eq!(
        report.observation.host_io.audio_pump.stream_state,
        RuntimeHostAudioStreamState::Running
    );
    assert_eq!(
        report.observation.host_io.hardware.backend_health,
        BackendHealth::Healthy
    );
    assert_eq!(report.observation.host_io.hardware.device_loss_count, 1);
    assert_eq!(report.observation.host_io.hardware.restart_attempt_count, 1);
    assert_eq!(report.observation.host_io.hardware.restart_failure_count, 0);
    assert_eq!(
        supervisor.observation.device_supervision_snapshot.state,
        signal_runtime::RuntimeDeviceSupervisionState::Stable
    );
    assert_eq!(
        supervisor.observation.device_supervision_snapshot.restart_state,
        signal_runtime::RuntimeDeviceRestartState::Recovered
    );
    assert_eq!(
        supervisor.observation.device_supervision_snapshot.fault_boundary,
        signal_runtime::RuntimeDeviceFaultBoundaryState::Clear
    );
    assert_eq!(
        report
            .observation
            .observation
            .device_supervision_snapshot
            .restart_attempt_count,
        Some(1)
    );
    assert_eq!(report.observation.host_io.latency.output_latency_samples, 512);
    assert!(report.observation.host_io.runtime_graph_id_matches_pump);
    assert_eq!(
        report
            .observation
            .observation
            .execution_topology_summary
            .track_lane_node_count,
        2
    );
}

#[test]
fn local_host_shared_report_tracks_device_loss_restart_failure() {
    let runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
    let mut host = LocalRuntimeHost::new(runtime);
    let error = host
        .boot_with_device_loss_restart_failure()
        .expect_err("device loss restart should fail");
    let supervisor = host.supervisor_report();
    let report = host.host_supervisor_report();

    assert_eq!(error.kind, RuntimeErrorKind::HardwareFailure);
    assert_eq!(
        report.observation.host_io.audio_pump.stream_state,
        RuntimeHostAudioStreamState::Faulted
    );
    assert_eq!(
        report.observation.host_io.hardware.backend_health,
        BackendHealth::Degraded
    );
    assert_eq!(report.observation.host_io.hardware.device_loss_count, 1);
    assert_eq!(report.observation.host_io.hardware.restart_attempt_count, 1);
    assert_eq!(report.observation.host_io.hardware.restart_failure_count, 1);
    assert_eq!(
        supervisor.observation.device_supervision_snapshot.state,
        signal_runtime::RuntimeDeviceSupervisionState::Exhausted
    );
    assert_eq!(
        supervisor.observation.device_supervision_snapshot.restart_state,
        signal_runtime::RuntimeDeviceRestartState::Exhausted
    );
    assert_eq!(
        supervisor.observation.device_supervision_snapshot.fault_boundary,
        signal_runtime::RuntimeDeviceFaultBoundaryState::Exhausted
    );
    assert_eq!(
        report.observation.host_io.clocking.clock_source,
        RuntimeHostClockSource::Internal
    );
    assert_eq!(
        report.observation.host_io.clocking.clock_domain,
        RuntimeHostClockDomain::Degraded
    );
    assert_eq!(
        report.observation.host_io.clocking.fallback_state,
        RuntimeHostClockFallbackState::RecoveryConstrained
    );
    assert_eq!(
        report.observation.host_io.clocking.transition_state,
        RuntimeHostClockTransitionState::Stable
    );
    assert_eq!(
        report.observation.host_io.clocking.drift_state,
        RuntimeHostClockDriftState::Resyncing
    );
    assert_eq!(
        report.observation.host_io.clocking.discontinuity_state,
        RuntimeHostClockDiscontinuityState::Faulted
    );
    assert_eq!(
        report.observation.host_io.clocking.duplex_mismatch_state,
        RuntimeHostDuplexMismatchState::NotApplicable
    );
    assert_eq!(
        report.observation.host_io.clocking.endpoint_topology,
        RuntimeHostEndpointTopology::OutputOnly
    );
    assert!(!report.observation.host_io.clocking.partial_availability);
    assert_eq!(
        report.observation.observation.external_io_snapshot.monitoring_state,
        signal_runtime::RuntimeExternalIoMonitoringState::Faulted
    );
    assert_eq!(
        report.observation.observation.external_io_snapshot.loopback_state,
        signal_runtime::RuntimeExternalIoLoopbackState::Faulted
    );
    assert!(!report.observation.host_io.clocking.crossing_required);
    assert!(!report.observation.host_io.runtime_graph_id_matches_pump);
    assert_eq!(
        report.observation.observation.control_snapshot.last_stop_reason,
        Some(StopReason::DeviceReconfigure)
    );
}
