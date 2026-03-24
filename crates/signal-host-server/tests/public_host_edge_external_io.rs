use signal_host_server::ServerRuntimeHost;
use signal_runtime::{
    RuntimeConfig, RuntimeExternalIoHealthState, RuntimeExternalIoLoopbackState,
    RuntimeExternalIoMonitoringState, RuntimeExternalIoMonitoringTapPoint,
    RuntimeExternalIoPrimaryRole, RuntimeHostEndpointTopology, SignalRuntime,
};

#[test]
fn server_shared_host_edge_exports_runtime_external_io_truth() {
    let runtime = SignalRuntime::new(RuntimeConfig::server(48_000, 512));
    let host = ServerRuntimeHost::new(runtime);
    let report = host.supervisor_report();

    assert_eq!(
        report.observation.external_io_snapshot.health_state,
        RuntimeExternalIoHealthState::Unavailable
    );
    assert_eq!(
        report.observation.external_io_snapshot.primary_role,
        RuntimeExternalIoPrimaryRole::Unavailable
    );
    assert_eq!(
        report.observation.external_io_snapshot.monitoring_state,
        RuntimeExternalIoMonitoringState::Unavailable
    );
    assert_eq!(
        report.observation.external_io_snapshot.monitoring_tap_point,
        RuntimeExternalIoMonitoringTapPoint::Unavailable
    );
    assert_eq!(
        report.observation.external_io_snapshot.loopback_state,
        RuntimeExternalIoLoopbackState::Unavailable
    );

    let rendered = report.render_json();
    assert!(rendered.contains("\"external_io_snapshot\":{"));
    assert!(rendered.contains("\"health_state\":\"Unavailable\""));
    assert!(rendered.contains("\"monitoring_state\":\"Unavailable\""));
    assert!(rendered.contains("\"loopback_state\":\"Unavailable\""));
}

#[test]
fn server_shared_host_edge_exports_runtime_linux_audio_backend_truth() {
    let runtime = SignalRuntime::new(RuntimeConfig::server(48_000, 512));
    let host = ServerRuntimeHost::new(runtime);
    let report = host.supervisor_report();

    assert_eq!(
        report
            .observation
            .external_io_snapshot
            .linux_backend_identity,
        signal_runtime::RuntimeLinuxAudioBackendIdentity::Unavailable
    );
    assert_eq!(
        report
            .observation
            .external_io_snapshot
            .linux_backend_portability,
        signal_runtime::RuntimeLinuxAudioBackendPortabilityBand::Unsupported
    );
    assert_eq!(
        report.observation.external_io_snapshot.fallback_state,
        signal_runtime::RuntimeHostClockFallbackState::Unconfigured
    );

    let rendered = report.render_json();
    assert!(rendered.contains("\"external_io_snapshot\":{"));
    assert!(rendered.contains("\"linux_backend_identity\":\"Unavailable\""));
    assert!(rendered.contains("\"linux_backend_portability\":\"Unsupported\""));
    assert!(rendered.contains("\"fallback_state\":\"Unconfigured\""));
}

#[test]
fn server_shared_host_edge_exports_runtime_linux_backend_clock_topology_truth() {
    let runtime = SignalRuntime::new(RuntimeConfig::server(48_000, 512));
    let host = ServerRuntimeHost::new(runtime);
    let report = host.supervisor_report();

    assert_eq!(
        report
            .observation
            .external_io_snapshot
            .linux_clocking_parity,
        signal_runtime::RuntimeLinuxAudioBackendClockingParityBand::Unsupported
    );
    assert_eq!(
        report.observation.external_io_snapshot.linux_duplex_parity,
        signal_runtime::RuntimeLinuxAudioBackendDuplexParityState::Unsupported
    );
    assert_eq!(
        report
            .observation
            .external_io_snapshot
            .linux_endpoint_topology_parity,
        signal_runtime::RuntimeLinuxAudioBackendEndpointTopologyParityState::Unsupported
    );
    assert_eq!(
        report.observation.external_io_snapshot.endpoint_topology,
        RuntimeHostEndpointTopology::Unconfigured
    );

    let rendered = report.render_json();
    assert!(rendered.contains("\"linux_clocking_parity\":\"Unsupported\""));
    assert!(rendered.contains("\"linux_duplex_parity\":\"Unsupported\""));
    assert!(rendered.contains("\"linux_endpoint_topology_parity\":\"Unsupported\""));
    assert!(rendered.contains("\"endpoint_topology\":\"Unconfigured\""));
}
