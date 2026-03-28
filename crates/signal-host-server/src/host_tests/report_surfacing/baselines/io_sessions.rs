use super::super::super::*;

#[test]
fn server_host_shared_report_surfaces_unavailable_external_io_monitoring_state() {
    let runtime = SignalRuntime::new(RuntimeConfig::server(48_000, 512));
    let host = ServerRuntimeHost::new(runtime);
    let report = host.supervisor_report();

    assert_eq!(
        report.observation.external_io_snapshot.health_state,
        RuntimeExternalIoHealthState::Unavailable
    );
    assert_eq!(
        report.observation.external_io_snapshot.device_change_state,
        RuntimeExternalIoDeviceChangeState::Unavailable
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
    assert_eq!(
        report.observation.external_io_snapshot.linux_clocking_parity,
        signal_runtime::RuntimeLinuxAudioBackendClockingParityBand::Unsupported
    );
    assert_eq!(
        report.observation.external_io_snapshot.linux_duplex_parity,
        signal_runtime::RuntimeLinuxAudioBackendDuplexParityState::Unsupported
    );
    assert_eq!(
        report.observation.external_io_snapshot.linux_endpoint_topology_parity,
        signal_runtime::RuntimeLinuxAudioBackendEndpointTopologyParityState::Unsupported
    );
    assert_eq!(
        report.observation.external_io_snapshot.endpoint_topology,
        signal_runtime::RuntimeHostEndpointTopology::Unconfigured
    );
    assert_eq!(
        report.observation.external_io_snapshot.fallback_state,
        signal_runtime::RuntimeHostClockFallbackState::Unconfigured
    );

    let rendered = report.render_json();
    assert!(rendered.contains("\"external_io_snapshot\":{"));
    assert!(rendered.contains("\"health_state\":\"Unavailable\""));
    assert!(rendered.contains("\"monitoring_state\":\"Unavailable\""));
    assert!(rendered.contains("\"loopback_state\":\"Unavailable\""));
    assert!(rendered.contains("\"linux_clocking_parity\":\"Unsupported\""));
}

#[test]
fn server_host_shared_report_surfaces_runtime_external_midi_endpoint_baseline() {
    let runtime = SignalRuntime::new(RuntimeConfig::server(48_000, 512));
    let host = ServerRuntimeHost::new(runtime);
    let report = host.supervisor_report();

    assert_eq!(
        report.observation.external_midi_snapshot.discovery_state,
        signal_runtime::RuntimeExternalMidiDiscoveryState::Idle
    );
    assert_eq!(
        report.observation.external_midi_snapshot.graph_state,
        signal_runtime::RuntimeExternalMidiGraphState::Empty
    );
    assert_eq!(
        report.observation.external_midi_snapshot.provider_name,
        "signal-host-server"
    );
    assert_eq!(report.observation.external_midi_snapshot.device_count, 0);
    assert_eq!(report.observation.external_midi_snapshot.endpoint_count, 0);
    assert_eq!(
        report
            .observation
            .external_midi_snapshot
            .live_ownership
            .ownership_posture,
        signal_runtime::RuntimeExternalMidiLiveOwnershipPosture::NoLiveOwnership
    );
    assert_eq!(
        report
            .observation
            .external_midi_snapshot
            .live_ownership
            .backend_parity,
        signal_runtime::RuntimeExternalMidiBackendParity::Guarded
    );
    assert!(report.observation.external_midi_snapshot.devices.is_empty());
    assert!(report.observation.external_midi_snapshot.endpoints.is_empty());

    let rendered = report.render_json();
    assert!(rendered.contains("\"external_midi_snapshot\":{"));
    assert!(rendered.contains("\"live_ownership\":{"));
    assert!(rendered.contains("\"discovery_state\":\"Idle\""));
    assert!(rendered.contains("\"graph_state\":\"Empty\""));
    assert!(rendered.contains("\"backend_parity\":\"Guarded\""));
    assert!(rendered.contains("\"provider_name\":\"signal-host-server\""));
}

#[test]
fn server_host_shared_report_surfaces_runtime_linux_backend_session_baseline() {
    let runtime = SignalRuntime::new(RuntimeConfig::server(48_000, 512));
    let host = ServerRuntimeHost::new(runtime);
    let report = host.supervisor_report();

    let snapshot = &report.observation.linux_backend_session_snapshot;
    assert_eq!(
        snapshot.backend_identity,
        signal_runtime::RuntimeLinuxAudioBackendIdentity::PipeWire
    );
    assert_eq!(
        snapshot.ownership,
        signal_runtime::RuntimeLinuxBackendSessionOwnership::BackendManagedGraph
    );
    assert_eq!(
        snapshot.lifecycle_state,
        signal_runtime::RuntimeLinuxBackendSessionLifecycleState::Running
    );
    assert_eq!(
        snapshot.device_claim_posture,
        signal_runtime::RuntimeLinuxBackendDeviceClaimPosture::SharedGraph
    );
    assert_eq!(
        snapshot.session_role,
        signal_runtime::RuntimeLinuxBackendSessionRole::PrimaryAudioIo
    );
    assert_eq!(
        snapshot.ownership_fallback,
        signal_runtime::RuntimeLinuxBackendOwnershipFallbackState::BackendManagedGuarded
    );
    assert_eq!(snapshot.backend_name, "pipewire");
    assert_eq!(snapshot.device_id, "pipewire:default-graph");
    assert!(snapshot.simulated);

    let rendered = report.render_json();
    assert!(rendered.contains("\"linux_backend_session_snapshot\":{"));
    assert!(rendered.contains("\"backend_identity\":\"PipeWire\""));
    assert!(rendered.contains("\"ownership\":\"BackendManagedGraph\""));
}

#[test]
fn server_host_shared_report_surfaces_runtime_jack_coordination_baseline() {
    let runtime = SignalRuntime::new(RuntimeConfig::server(48_000, 512));
    let host = ServerRuntimeHost::new(runtime);
    let report = host.supervisor_report();

    let snapshot = &report.observation.jack_coordination_snapshot;
    assert_eq!(
        snapshot.backend_identity,
        signal_runtime::RuntimeLinuxAudioBackendIdentity::Jack
    );
    assert_eq!(snapshot.backend_name, "jack");
    assert_eq!(
        snapshot.transport_posture,
        signal_runtime::RuntimeJackTransportPosture::Detached
    );
    assert_eq!(
        snapshot.graph_state,
        signal_runtime::RuntimeJackGraphCoordinationState::AttachedGuarded
    );
    assert_eq!(
        snapshot.client_role,
        signal_runtime::RuntimeJackClientRole::PrimaryAudioIo
    );
    assert_eq!(
        snapshot.guarded_state,
        signal_runtime::RuntimeJackGuardedCoordinationState::GraphGuarded
    );
    assert_eq!(snapshot.device_id, "jack:graph-main");
    assert!(snapshot.simulated);

    let rendered = report.render_json();
    assert!(rendered.contains("\"jack_coordination_snapshot\":{"));
    assert!(rendered.contains("\"backend_identity\":\"Jack\""));
    assert!(rendered.contains("\"transport_posture\":\"Detached\""));
    assert!(rendered.contains("\"graph_state\":\"AttachedGuarded\""));
}

#[test]
fn server_host_shared_report_surfaces_runtime_control_surface_baseline() {
    let runtime = SignalRuntime::new(RuntimeConfig::server(48_000, 512));
    let host = ServerRuntimeHost::new(runtime);
    let report = host.supervisor_report();

    assert_eq!(
        report.observation.control_surface_snapshot.discovery_state,
        signal_runtime::RuntimeExternalMidiDiscoveryState::Idle
    );
    assert_eq!(
        report.observation.control_surface_snapshot.graph_state,
        signal_runtime::RuntimeControlSurfaceGraphState::Empty
    );
    assert_eq!(
        report.observation.control_surface_snapshot.provider_name,
        "signal-host-server"
    );
    assert_eq!(report.observation.control_surface_snapshot.device_count, 0);
    assert!(report.observation.control_surface_snapshot.devices.is_empty());

    let rendered = report.render_json();
    assert!(rendered.contains("\"control_surface_snapshot\":{"));
    assert!(rendered.contains("\"graph_state\":\"Empty\""));
    assert!(rendered.contains("\"provider_name\":\"signal-host-server\""));
}

#[test]
fn server_host_shared_report_surfaces_runtime_advanced_hardware_baseline() {
    let runtime = SignalRuntime::new(RuntimeConfig::server(48_000, 512));
    let host = ServerRuntimeHost::new(runtime);
    let report = host.supervisor_report();

    assert_eq!(
        report.observation.advanced_hardware_snapshot.discovery_state,
        signal_runtime::RuntimeExternalMidiDiscoveryState::Idle
    );
    assert_eq!(
        report.observation.advanced_hardware_snapshot.graph_state,
        signal_runtime::RuntimeAdvancedHardwareGraphState::Empty
    );
    assert_eq!(
        report.observation.advanced_hardware_snapshot.provider_name,
        "signal-host-server"
    );
    assert_eq!(report.observation.advanced_hardware_snapshot.device_count, 0);
    assert!(report.observation.advanced_hardware_snapshot.devices.is_empty());

    let rendered = report.render_json();
    assert!(rendered.contains("\"advanced_hardware_snapshot\":{"));
    assert!(rendered.contains("\"graph_state\":\"Empty\""));
    assert!(rendered.contains("\"provider_name\":\"signal-host-server\""));
}
