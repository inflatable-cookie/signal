use signal_host_server::ServerRuntimeHost;
use signal_runtime::{RuntimeConfig, RuntimeExternalMidiGraphState, SignalRuntime};

#[test]
fn server_shared_host_edge_exports_runtime_external_midi_truth() {
    let runtime = SignalRuntime::new(RuntimeConfig::server(48_000, 512));
    let host = ServerRuntimeHost::new(runtime);
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
        "signal-host-server"
    );
    assert_eq!(report.observation.external_midi_snapshot.device_count, 0);
    assert_eq!(report.observation.external_midi_snapshot.endpoint_count, 0);
    assert_eq!(
        report
            .observation
            .external_midi_snapshot
            .guarded_route_count,
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
        signal_runtime::RuntimeExternalMidiBackendParity::Guarded
    );
    assert_eq!(
        report
            .observation
            .external_midi_snapshot
            .live_ownership
            .guarded_parity_outcome,
        signal_runtime::RuntimeExternalMidiGuardedParityOutcome::BackendManaged
    );

    let rendered = report.render_json();
    assert!(rendered.contains("\"external_midi_snapshot\":{"));
    assert!(rendered.contains("\"live_ownership\":{"));
    assert!(rendered.contains("\"discovery_state\":\"Idle\""));
    assert!(rendered.contains("\"graph_state\":\"Empty\""));
    assert!(rendered.contains("\"ownership_posture\":\"NoLiveOwnership\""));
    assert!(rendered.contains("\"backend_parity\":\"Guarded\""));
    assert!(rendered.contains("\"guarded_parity_outcome\":\"BackendManaged\""));
    assert!(rendered.contains("\"provider_name\":\"signal-host-server\""));
}
