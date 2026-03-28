use super::super::super::super::*;

#[test]
fn local_host_shared_report_surfaces_runtime_external_midi_endpoint_baseline() {
    let runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
    let mut host = LocalRuntimeHost::new(runtime);
    host.boot_default().expect("default local host boot");
    let report = host.host_supervisor_report();

    assert_eq!(
        report.observation.observation.external_midi_snapshot.discovery_state,
        signal_runtime::RuntimeExternalMidiDiscoveryState::Idle
    );
    assert_eq!(
        report.observation.observation.external_midi_snapshot.graph_state,
        signal_runtime::RuntimeExternalMidiGraphState::Empty
    );
    assert_eq!(
        report.observation.observation.external_midi_snapshot.provider_name,
        "signal-host-local"
    );
    assert_eq!(report.observation.observation.external_midi_snapshot.device_count, 0);
    assert_eq!(
        report.observation.observation.external_midi_snapshot.endpoint_count,
        0
    );
    assert_eq!(
        report.observation.observation.external_midi_snapshot.live_ownership.ownership_posture,
        signal_runtime::RuntimeExternalMidiLiveOwnershipPosture::NoLiveOwnership
    );
    assert_eq!(
        report.observation.observation.external_midi_snapshot.live_ownership.backend_parity,
        signal_runtime::RuntimeExternalMidiBackendParity::NotLinux
    );
    assert!(report.observation.observation.external_midi_snapshot.devices.is_empty());
    assert!(report
        .observation
        .observation
        .external_midi_snapshot
        .endpoints
        .is_empty());

    let rendered = report.render_json();
    assert!(rendered.contains("\"external_midi_snapshot\":{"));
    assert!(rendered.contains("\"live_ownership\":{"));
    assert!(rendered.contains("\"discovery_state\":\"Idle\""));
    assert!(rendered.contains("\"graph_state\":\"Empty\""));
    assert!(rendered.contains("\"backend_parity\":\"NotLinux\""));
    assert!(rendered.contains("\"provider_name\":\"signal-host-local\""));
}

#[test]
fn local_host_shared_report_surfaces_runtime_control_surface_baseline() {
    let runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
    let mut host = LocalRuntimeHost::new(runtime);
    host.boot_default().expect("default local host boot");
    let report = host.host_supervisor_report();

    assert_eq!(
        report.observation.observation.control_surface_snapshot.discovery_state,
        signal_runtime::RuntimeExternalMidiDiscoveryState::Idle
    );
    assert_eq!(
        report.observation.observation.control_surface_snapshot.graph_state,
        signal_runtime::RuntimeControlSurfaceGraphState::Empty
    );
    assert_eq!(
        report.observation.observation.control_surface_snapshot.provider_name,
        "signal-host-local"
    );
    assert_eq!(report.observation.observation.control_surface_snapshot.device_count, 0);
    assert!(report
        .observation
        .observation
        .control_surface_snapshot
        .devices
        .is_empty());

    let rendered = report.render_json();
    assert!(rendered.contains("\"control_surface_snapshot\":{"));
    assert!(rendered.contains("\"graph_state\":\"Empty\""));
    assert!(rendered.contains("\"provider_name\":\"signal-host-local\""));
}

#[test]
fn local_host_shared_report_surfaces_runtime_linux_backend_session_as_not_linux() {
    let runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
    let mut host = LocalRuntimeHost::new(runtime);
    host.boot_default().expect("default local host boot");
    let report = host.host_supervisor_report();

    let snapshot = &report.observation.observation.linux_backend_session_snapshot;
    assert_eq!(
        snapshot.backend_identity,
        signal_runtime::RuntimeLinuxAudioBackendIdentity::NotLinux
    );
    assert_eq!(
        snapshot.ownership,
        signal_runtime::RuntimeLinuxBackendSessionOwnership::NotLinux
    );
    assert_eq!(
        snapshot.lifecycle_state,
        signal_runtime::RuntimeLinuxBackendSessionLifecycleState::NotLinux
    );
    assert_eq!(
        snapshot.device_claim_posture,
        signal_runtime::RuntimeLinuxBackendDeviceClaimPosture::NotLinux
    );
    assert_eq!(
        snapshot.session_role,
        signal_runtime::RuntimeLinuxBackendSessionRole::NotLinux
    );
    assert_eq!(
        snapshot.ownership_fallback,
        signal_runtime::RuntimeLinuxBackendOwnershipFallbackState::NotLinux
    );

    let rendered = report.render_json();
    assert!(rendered.contains("\"linux_backend_session_snapshot\":{"));
    assert!(rendered.contains("\"backend_identity\":\"NotLinux\""));
    assert!(rendered.contains("\"ownership\":\"NotLinux\""));
}

#[test]
fn local_host_shared_report_surfaces_runtime_jack_coordination_baseline() {
    let runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
    let mut host = LocalRuntimeHost::new(runtime);
    host.boot_default().expect("default local host boot");
    let report = host.host_supervisor_report();

    let snapshot = &report.observation.observation.jack_coordination_snapshot;
    assert_eq!(
        snapshot.transport_posture,
        signal_runtime::RuntimeJackTransportPosture::NotJack
    );
    assert_eq!(
        snapshot.graph_state,
        signal_runtime::RuntimeJackGraphCoordinationState::NotJack
    );
    assert_eq!(
        snapshot.client_role,
        signal_runtime::RuntimeJackClientRole::NotJack
    );
    assert_eq!(
        snapshot.guarded_state,
        signal_runtime::RuntimeJackGuardedCoordinationState::NotJack
    );

    let rendered = report.render_json();
    assert!(rendered.contains("\"jack_coordination_snapshot\":{"));
    assert!(rendered.contains("\"transport_posture\":\"NotJack\""));
    assert!(rendered.contains("\"graph_state\":\"NotJack\""));
}

#[test]
fn local_host_shared_report_surfaces_runtime_advanced_hardware_baseline() {
    let runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
    let mut host = LocalRuntimeHost::new(runtime);
    host.boot_default().expect("default local host boot");
    let report = host.host_supervisor_report();

    assert_eq!(
        report.observation.observation.advanced_hardware_snapshot.discovery_state,
        signal_runtime::RuntimeExternalMidiDiscoveryState::Idle
    );
    assert_eq!(
        report.observation.observation.advanced_hardware_snapshot.graph_state,
        signal_runtime::RuntimeAdvancedHardwareGraphState::Empty
    );
    assert_eq!(
        report.observation.observation.advanced_hardware_snapshot.provider_name,
        "signal-host-local"
    );
    assert_eq!(report.observation.observation.advanced_hardware_snapshot.device_count, 0);
    assert!(report
        .observation
        .observation
        .advanced_hardware_snapshot
        .devices
        .is_empty());

    let rendered = report.render_json();
    assert!(rendered.contains("\"advanced_hardware_snapshot\":{"));
    assert!(rendered.contains("\"graph_state\":\"Empty\""));
    assert!(rendered.contains("\"provider_name\":\"signal-host-local\""));
}

#[test]
fn local_host_shared_report_surfaces_runtime_stretch_engine_baseline() {
    let runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
    let mut host = LocalRuntimeHost::new(runtime);
    host.boot_default().expect("default local host boot");
    let report = host.host_supervisor_report();

    assert_eq!(report.observation.observation.stretch_engine_snapshot.clip_count, 0);
    assert_eq!(
        report.observation.observation.stretch_engine_snapshot.ready_clip_count,
        0
    );
    assert!(report
        .observation
        .observation
        .stretch_engine_snapshot
        .clips
        .is_empty());

    let rendered = report.render_json();
    assert!(rendered.contains("\"stretch_engine_snapshot\":{"));
    assert!(rendered.contains("\"clip_count\":0"));
    assert!(rendered.contains("\"sample_domain_clip_count\":0"));
}
