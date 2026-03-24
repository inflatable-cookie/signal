use signal_runtime::{
    RuntimeConfig, RuntimeEventRecorder, RuntimeExternalMidiDiscoveryState,
    RuntimeExternalMidiGraphState, RuntimeObservationReport, RuntimeSupervisorReport,
    SignalRuntime,
};

#[test]
fn public_runtime_external_midi_boundary_reports_runtime_owned_endpoint_graph_truth() {
    let runtime = SignalRuntime::new(RuntimeConfig::server(48_000, 512));
    let recorder = RuntimeEventRecorder::default();

    let unavailable = RuntimeObservationReport::capture(&runtime, &recorder);
    assert_eq!(
        unavailable.external_midi_snapshot.discovery_state,
        RuntimeExternalMidiDiscoveryState::Unavailable
    );
    assert_eq!(
        unavailable.external_midi_snapshot.graph_state,
        RuntimeExternalMidiGraphState::Unavailable
    );
    assert_eq!(
        unavailable.external_midi_snapshot.provider_name,
        "runtime-unavailable"
    );
    assert_eq!(unavailable.external_midi_snapshot.device_count, 0);
    assert_eq!(unavailable.external_midi_snapshot.endpoint_count, 0);
    assert_eq!(
        unavailable
            .external_midi_snapshot
            .live_ownership
            .ownership_posture,
        signal_runtime::RuntimeExternalMidiLiveOwnershipPosture::Unavailable
    );
    assert_eq!(
        unavailable
            .external_midi_snapshot
            .live_ownership
            .backend_parity,
        signal_runtime::RuntimeExternalMidiBackendParity::Unavailable
    );
    assert!(unavailable.external_midi_snapshot.devices.is_empty());
    assert!(unavailable.external_midi_snapshot.endpoints.is_empty());

    let empty = signal_runtime::RuntimeExternalMidiEndpointGraphSnapshot::empty("public-runtime");
    let empty_observation = unavailable
        .clone()
        .with_external_midi_snapshot(empty.clone());
    assert_eq!(
        empty_observation.external_midi_snapshot.discovery_state,
        RuntimeExternalMidiDiscoveryState::Idle
    );
    assert_eq!(
        empty_observation.external_midi_snapshot.graph_state,
        RuntimeExternalMidiGraphState::Empty
    );
    assert_eq!(
        empty_observation.external_midi_snapshot.provider_name,
        "public-runtime"
    );
    assert_eq!(empty_observation.external_midi_snapshot.device_count, 0);
    assert_eq!(empty_observation.external_midi_snapshot.endpoint_count, 0);
    assert_eq!(
        empty_observation
            .external_midi_snapshot
            .live_ownership
            .ownership_posture,
        signal_runtime::RuntimeExternalMidiLiveOwnershipPosture::Unavailable
    );
    assert_eq!(
        empty_observation
            .external_midi_snapshot
            .live_ownership
            .attach_continuity,
        signal_runtime::RuntimeExternalMidiAttachContinuity::Unavailable
    );
    assert_eq!(
        empty_observation
            .external_midi_snapshot
            .live_ownership
            .backend_parity,
        signal_runtime::RuntimeExternalMidiBackendParity::Unavailable
    );
    assert_eq!(
        empty_observation.external_midi_snapshot.active_route_count,
        0
    );
    assert_eq!(
        empty_observation.external_midi_snapshot.guarded_route_count,
        0
    );

    let observation_json = empty_observation.render_json();
    assert!(observation_json.contains("\"external_midi_snapshot\":{"));
    assert!(observation_json.contains("\"live_ownership\":{"));
    assert!(observation_json.contains("\"discovery_state\":\"Idle\""));
    assert!(observation_json.contains("\"graph_state\":\"Empty\""));
    assert!(observation_json.contains("\"ownership_posture\":\"Unavailable\""));
    assert!(observation_json.contains("\"backend_parity\":\"Unavailable\""));
    assert!(observation_json.contains("\"provider_name\":\"public-runtime\""));

    let mut supervisor = RuntimeSupervisorReport::capture(&runtime, &recorder);
    supervisor.observation = supervisor
        .observation
        .clone()
        .with_external_midi_snapshot(empty);
    let supervisor_json = supervisor.render_json();
    assert!(supervisor_json.contains("\"external_midi_snapshot\":{"));
    assert!(supervisor_json.contains("\"discovery_state\":\"Idle\""));
    assert!(supervisor_json.contains("\"live_ownership\":{"));
    assert!(supervisor_json.contains("\"ownership_posture\":\"Unavailable\""));
    assert!(supervisor_json.contains("\"provider_name\":\"public-runtime\""));
}
