use signal_plugin::EventPacketSummary;
use signal_runtime::{
    RuntimeConfig, RuntimeEventRecorder, RuntimeObservationReport, RuntimeSupervisorReport,
    SignalRuntime,
};

#[test]
fn public_runtime_controller_expression_boundary_reports_runtime_owned_expression_truth() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
    let recorder = RuntimeEventRecorder::default();

    runtime.record_plugin_event_summary(
        13,
        "lease-public-controller-expression",
        21,
        288,
        EventPacketSummary {
            total_events: 10,
            parameter_value_events: 1,
            parameter_modulation_events: 1,
            parameter_gesture_events: 1,
            note_events: 2,
            note_expression_events: 4,
            note_expression_pressure_events: 1,
            note_expression_timbre_events: 1,
            note_expression_tuning_events: 2,
            midi_events: 1,
        },
    );

    let capability = signal_runtime::RuntimeExternalMidiEndpointCapabilitySummary {
        supports_bounded_midi_input: true,
        supports_bounded_midi_output: true,
        supports_transport_clock: true,
        supports_note_events: true,
        supports_controller_events: true,
        supports_note_pressure_expression: true,
        supports_note_timbre_expression: true,
        supports_note_tuning_expression: true,
        supports_mpe: true,
        midi2_posture: signal_runtime::RuntimeControllerExpressionMidi2Posture::Guarded,
        control_surface_guarded: true,
        summary: "midi-input=true midi-output=true transport-clock=true note-events=true controller-events=true pressure=true timbre=true tuning=true mpe=true midi2=Guarded control-surface=guarded".into(),
    };
    let controller_graph = signal_runtime::RuntimeExternalMidiEndpointGraphSnapshot {
        discovery_state: signal_runtime::RuntimeExternalMidiDiscoveryState::Enumerated,
        graph_state: signal_runtime::RuntimeExternalMidiGraphState::Stable,
        live_ownership:
            signal_runtime::RuntimeExternalMidiLiveOwnershipSummary::detached_without_backend_context(),
        provider_name: "controller-expression-runtime".into(),
        device_count: 1,
        endpoint_count: 1,
        input_endpoint_count: 1,
        output_endpoint_count: 0,
        duplex_endpoint_count: 0,
        active_route_count: 1,
        guarded_route_count: 1,
        devices: vec![signal_runtime::RuntimeExternalMidiDeviceDescriptor {
            device_id: "device:surface:1".into(),
            device_name: "Surface One".into(),
            lifecycle_state: signal_runtime::RuntimeExternalMidiDeviceLifecycleState::Discovered,
            endpoint_count: 1,
            summary: "device=Surface One endpoints=1".into(),
        }],
        endpoints: vec![signal_runtime::RuntimeExternalMidiEndpointDescriptor {
            endpoint_id: "endpoint:surface:1".into(),
            endpoint_name: "Surface One Input".into(),
            device_id: "device:surface:1".into(),
            direction: signal_runtime::RuntimeExternalMidiEndpointDirection::Input,
            lifecycle_state: signal_runtime::RuntimeExternalMidiEndpointLifecycleState::Active,
            route_state: signal_runtime::RuntimeExternalMidiRouteState::InputObserved,
            capability: capability.clone(),
            summary: "direction=Input route=InputObserved pressure=true timbre=true tuning=true mpe=true midi2=Guarded".into(),
        }],
        summary: "discovery=Enumerated graph=Stable provider=controller-expression-runtime devices=1 endpoints=1 routes=1".into(),
    };

    let observation = RuntimeObservationReport::capture(&runtime, &recorder)
        .with_external_midi_snapshot(controller_graph.clone());
    let mut supervisor = RuntimeSupervisorReport::capture(&runtime, &recorder);
    supervisor.observation = supervisor
        .observation
        .clone()
        .with_external_midi_snapshot(controller_graph);

    let snapshot = &observation.plugin_event_snapshot;
    assert_eq!(snapshot.note_expression_events, 4);
    assert_eq!(snapshot.note_expression_pressure_events, 1);
    assert_eq!(snapshot.note_expression_timbre_events, 1);
    assert_eq!(snapshot.note_expression_tuning_events, 2);
    assert_eq!(
        snapshot.mpe_posture,
        signal_runtime::RuntimeControllerExpressionMpePosture::Guarded
    );
    assert_eq!(
        snapshot.midi2_posture,
        signal_runtime::RuntimeControllerExpressionMidi2Posture::Guarded
    );

    let endpoint = &observation.external_midi_snapshot.endpoints[0];
    assert!(endpoint.capability.supports_note_pressure_expression);
    assert!(endpoint.capability.supports_note_timbre_expression);
    assert!(endpoint.capability.supports_note_tuning_expression);
    assert!(endpoint.capability.supports_mpe);
    assert_eq!(
        endpoint.capability.midi2_posture,
        signal_runtime::RuntimeControllerExpressionMidi2Posture::Guarded
    );

    let observation_json = observation.render_json();
    assert!(observation_json.contains("\"note_expression_pressure_events\":1"));
    assert!(observation_json.contains("\"note_expression_timbre_events\":1"));
    assert!(observation_json.contains("\"note_expression_tuning_events\":2"));
    assert!(observation_json.contains("\"mpe_posture\":\"Guarded\""));
    assert!(observation_json.contains("\"midi2_posture\":\"Guarded\""));
    assert!(observation_json.contains("\"supports_note_pressure_expression\":true"));
    assert!(observation_json.contains("\"supports_note_timbre_expression\":true"));
    assert!(observation_json.contains("\"supports_note_tuning_expression\":true"));
    assert!(observation_json.contains("\"supports_mpe\":true"));

    let supervisor_json = supervisor.render_json();
    assert!(supervisor_json.contains("\"plugin_events\":{"));
    assert!(supervisor_json.contains("\"external_midi_snapshot\":{"));
    assert!(supervisor_json.contains("\"midi2_posture\":\"Guarded\""));
}
