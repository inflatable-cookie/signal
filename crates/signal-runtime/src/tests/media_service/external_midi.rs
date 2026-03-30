use super::super::*;

#[test]
fn runtime_observation_and_supervisor_reports_surface_external_midi_endpoint_baseline() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::server(48_000, 256));
    handshake_and_configure_with_anticipative(&mut runtime, true);

    let observation = RuntimeObservationReport::capture(&runtime, &RuntimeEventRecorder::default());
    assert_eq!(
        observation.external_midi_snapshot.discovery_state,
        crate::RuntimeExternalMidiDiscoveryState::Unavailable
    );
    assert_eq!(
        observation.external_midi_snapshot.graph_state,
        crate::RuntimeExternalMidiGraphState::Unavailable
    );
    assert_eq!(observation.external_midi_snapshot.device_count, 0);
    assert_eq!(observation.external_midi_snapshot.endpoint_count, 0);

    let supervisor = RuntimeSupervisorReport::capture(&runtime, &RuntimeEventRecorder::default());
    let multiline = supervisor.render_multiline();
    assert!(multiline.contains("external_midi_discovery_state=Unavailable"));
    assert!(multiline.contains("external_midi_graph_state=Unavailable"));

    let json = supervisor.render_json();
    assert!(json.contains("\"external_midi_snapshot\":{"));
    assert!(json.contains("\"discovery_state\":\"Unavailable\""));
    assert!(json.contains("\"graph_state\":\"Unavailable\""));
    assert!(json.contains("\"provider_name\":\"runtime-unavailable\""));
}
