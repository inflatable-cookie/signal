use signal_host_local::LocalRuntimeHost;
use signal_runtime::{RuntimeConfig, RuntimeExternalMidiGraphState, SignalRuntime};

#[test]
fn local_shared_host_edge_exports_runtime_external_midi_truth() {
    let runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
    let mut host = LocalRuntimeHost::new(runtime);
    host.boot_default()
        .expect("public local external midi default boot should succeed");
    let report = host.supervisor_report();

    assert_eq!(
        report.observation.external_midi_snapshot.discovery_state,
        signal_runtime::RuntimeExternalMidiDiscoveryState::Idle
    );
    assert_eq!(
        report.observation.external_midi_snapshot.graph_state,
        RuntimeExternalMidiGraphState::Empty
    );
    assert_eq!(
        report.observation.external_midi_snapshot.provider_name,
        "signal-host-local"
    );
    assert_eq!(report.observation.external_midi_snapshot.device_count, 0);
    assert_eq!(report.observation.external_midi_snapshot.endpoint_count, 0);
    assert_eq!(
        report.observation.external_midi_snapshot.active_route_count,
        0
    );
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
            .attach_continuity,
        signal_runtime::RuntimeExternalMidiAttachContinuity::Detached
    );
    assert_eq!(
        report
            .observation
            .external_midi_snapshot
            .live_ownership
            .backend_parity,
        signal_runtime::RuntimeExternalMidiBackendParity::NotLinux
    );

    let rendered = report.render_json();
    assert!(rendered.contains("\"external_midi_snapshot\":{"));
    assert!(rendered.contains("\"live_ownership\":{"));
    assert!(rendered.contains("\"discovery_state\":\"Idle\""));
    assert!(rendered.contains("\"graph_state\":\"Empty\""));
    assert!(rendered.contains("\"ownership_posture\":\"NoLiveOwnership\""));
    assert!(rendered.contains("\"backend_parity\":\"NotLinux\""));
    assert!(rendered.contains("\"provider_name\":\"signal-host-local\""));
}
