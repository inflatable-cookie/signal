use signal_host_local::LocalRuntimeHost;
use signal_runtime::{
    RuntimeConfig, RuntimeErrorKind, RuntimeExternalIoHealthState, RuntimeExternalIoLoopbackState,
    RuntimeExternalIoMonitoringState, RuntimeExternalIoMonitoringTapPoint,
    RuntimeExternalIoPrimaryRole, RuntimeHostClockDiscontinuityState, RuntimeHostClockDriftState,
    RuntimeHostDuplexMismatchState, RuntimeHostEndpointTopology, SignalRuntime,
};

#[test]
fn local_shared_host_edge_exports_runtime_clock_topology_truth() {
    let runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
    let mut steady_host = LocalRuntimeHost::new(runtime);
    steady_host
        .boot_default()
        .expect("public local clock topology default boot should succeed");
    let steady = steady_host.host_supervisor_report();

    assert_eq!(
        steady.observation.host_io.clocking.drift_state,
        RuntimeHostClockDriftState::Stable
    );
    assert_eq!(
        steady.observation.host_io.clocking.discontinuity_state,
        RuntimeHostClockDiscontinuityState::Continuous
    );
    assert_eq!(
        steady.observation.host_io.clocking.duplex_mismatch_state,
        RuntimeHostDuplexMismatchState::NotApplicable
    );
    assert_eq!(
        steady.observation.host_io.clocking.endpoint_topology,
        RuntimeHostEndpointTopology::OutputOnly
    );
    assert!(!steady.observation.host_io.clocking.partial_availability);

    let runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
    let mut faulted_host = LocalRuntimeHost::new(runtime);
    let error = faulted_host
        .boot_with_device_loss_restart_failure()
        .expect_err("public local clock topology restart failure should fail");
    assert_eq!(error.kind, RuntimeErrorKind::HardwareFailure);
    let faulted = faulted_host.host_supervisor_report();

    assert_eq!(
        faulted.observation.host_io.clocking.drift_state,
        RuntimeHostClockDriftState::Resyncing
    );
    assert_eq!(
        faulted.observation.host_io.clocking.discontinuity_state,
        RuntimeHostClockDiscontinuityState::Faulted
    );
    assert_eq!(
        faulted.observation.host_io.clocking.duplex_mismatch_state,
        RuntimeHostDuplexMismatchState::NotApplicable
    );
    assert_eq!(
        faulted.observation.host_io.clocking.endpoint_topology,
        RuntimeHostEndpointTopology::OutputOnly
    );
    assert!(!faulted.observation.host_io.clocking.partial_availability);

    let rendered = faulted.render_json();
    assert!(rendered.contains("\"drift_state\":\"Resyncing\""));
    assert!(rendered.contains("\"discontinuity_state\":\"Faulted\""));
    assert!(rendered.contains("\"endpoint_topology\":\"OutputOnly\""));
}

#[test]
fn local_shared_host_edge_exports_runtime_linux_backend_clock_topology_truth() {
    let runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
    let mut host = LocalRuntimeHost::new(runtime);
    host.boot_default()
        .expect("public local linux backend clock topology default boot should succeed");
    let report = host.host_supervisor_report();

    assert_eq!(
        report.observation.host_io.hardware.linux_backend_identity,
        signal_runtime::RuntimeLinuxAudioBackendIdentity::NotLinux
    );
    assert_eq!(
        report
            .observation
            .host_io
            .hardware
            .linux_backend_portability,
        signal_runtime::RuntimeLinuxAudioBackendPortabilityBand::Unsupported
    );
    assert_eq!(
        report.observation.host_io.clocking.linux_clocking_parity,
        signal_runtime::RuntimeLinuxAudioBackendClockingParityBand::Unsupported
    );
    assert_eq!(
        report.observation.host_io.clocking.linux_duplex_parity,
        signal_runtime::RuntimeLinuxAudioBackendDuplexParityState::Unsupported
    );
    assert_eq!(
        report
            .observation
            .host_io
            .clocking
            .linux_endpoint_topology_parity,
        signal_runtime::RuntimeLinuxAudioBackendEndpointTopologyParityState::Unsupported
    );

    let rendered = report.render_json();
    assert!(rendered.contains("\"linux_backend_identity\":\"NotLinux\""));
    assert!(rendered.contains("\"linux_clocking_parity\":\"Unsupported\""));
    assert!(rendered.contains("\"linux_duplex_parity\":\"Unsupported\""));
    assert!(rendered.contains("\"linux_endpoint_topology_parity\":\"Unsupported\""));
}

#[test]
fn local_shared_host_edge_exports_runtime_external_io_truth() {
    let runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
    let mut direct_host = LocalRuntimeHost::new(runtime);
    direct_host
        .boot_default()
        .expect("public local external io default boot should succeed");
    let direct = direct_host.supervisor_report();

    assert_eq!(
        direct.observation.external_io_snapshot.primary_role,
        RuntimeExternalIoPrimaryRole::ProgramOutput
    );
    assert_eq!(
        direct.observation.external_io_snapshot.monitoring_state,
        RuntimeExternalIoMonitoringState::Direct
    );
    assert_eq!(
        direct.observation.external_io_snapshot.monitoring_tap_point,
        RuntimeExternalIoMonitoringTapPoint::PostHardwareOutput
    );
    assert_eq!(
        direct.observation.external_io_snapshot.loopback_state,
        RuntimeExternalIoLoopbackState::Unavailable
    );

    let faulted_runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
    let mut faulted_host = LocalRuntimeHost::new(faulted_runtime);
    let error = faulted_host
        .boot_with_device_loss_restart_failure()
        .expect_err("public local external io restart failure should fail");
    assert_eq!(error.kind, RuntimeErrorKind::HardwareFailure);
    let faulted = faulted_host.supervisor_report();

    assert_eq!(
        faulted.observation.external_io_snapshot.health_state,
        RuntimeExternalIoHealthState::Faulted
    );
    assert_eq!(
        faulted.observation.external_io_snapshot.monitoring_state,
        RuntimeExternalIoMonitoringState::Faulted
    );
    assert_eq!(
        faulted.observation.external_io_snapshot.loopback_state,
        RuntimeExternalIoLoopbackState::Faulted
    );

    let rendered = faulted.render_json();
    assert!(rendered.contains("\"external_io_snapshot\":{"));
    assert!(rendered.contains("\"health_state\":\"Faulted\""));
    assert!(rendered.contains("\"monitoring_state\":\"Faulted\""));
    assert!(rendered.contains("\"loopback_state\":\"Faulted\""));
}
