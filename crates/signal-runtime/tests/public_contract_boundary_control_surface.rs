use signal_runtime::{
    RuntimeConfig, RuntimeEventRecorder, RuntimeExternalMidiDiscoveryState,
    RuntimeObservationReport, RuntimeSupervisorReport, SignalRuntime,
};

#[test]
fn public_runtime_control_surface_boundary_reports_runtime_owned_transport_and_feedback_truth() {
    let runtime = SignalRuntime::new(RuntimeConfig::server(48_000, 512));
    let recorder = RuntimeEventRecorder::default();

    let unavailable = RuntimeObservationReport::capture(&runtime, &recorder);
    assert_eq!(
        unavailable.control_surface_snapshot.discovery_state,
        RuntimeExternalMidiDiscoveryState::Unavailable
    );
    assert_eq!(
        unavailable.control_surface_snapshot.graph_state,
        signal_runtime::RuntimeControlSurfaceGraphState::Unavailable
    );
    assert_eq!(
        unavailable.control_surface_snapshot.provider_name,
        "runtime-unavailable"
    );
    assert_eq!(unavailable.control_surface_snapshot.device_count, 0);
    assert!(unavailable.control_surface_snapshot.devices.is_empty());

    let capability = signal_runtime::RuntimeExternalMidiEndpointCapabilitySummary {
        supports_bounded_midi_input: true,
        supports_bounded_midi_output: true,
        supports_transport_clock: true,
        supports_note_events: true,
        supports_controller_events: true,
        supports_note_pressure_expression: true,
        supports_note_timbre_expression: false,
        supports_note_tuning_expression: false,
        supports_mpe: false,
        midi2_posture: signal_runtime::RuntimeControllerExpressionMidi2Posture::Unsupported,
        control_surface_guarded: true,
        summary: "midi-input=true midi-output=true transport-clock=true note-events=true controller-events=true pressure=true timbre=false tuning=false mpe=false midi2=Unsupported control-surface=guarded".into(),
    };
    let control_surface_graph = signal_runtime::RuntimeExternalMidiEndpointGraphSnapshot {
        discovery_state: signal_runtime::RuntimeExternalMidiDiscoveryState::Enumerated,
        graph_state: signal_runtime::RuntimeExternalMidiGraphState::Stable,
        live_ownership:
            signal_runtime::RuntimeExternalMidiLiveOwnershipSummary::detached_without_backend_context(),
        provider_name: "public-control-surface".into(),
        device_count: 1,
        endpoint_count: 2,
        input_endpoint_count: 1,
        output_endpoint_count: 1,
        duplex_endpoint_count: 1,
        active_route_count: 1,
        guarded_route_count: 1,
        devices: vec![signal_runtime::RuntimeExternalMidiDeviceDescriptor {
            device_id: "device:control-surface:1".into(),
            device_name: "Control Surface".into(),
            lifecycle_state: signal_runtime::RuntimeExternalMidiDeviceLifecycleState::Discovered,
            endpoint_count: 2,
            summary: "device=Control Surface endpoints=2".into(),
        }],
        endpoints: vec![
            signal_runtime::RuntimeExternalMidiEndpointDescriptor {
                endpoint_id: "endpoint:control-surface:input".into(),
                endpoint_name: "Control Surface Input".into(),
                device_id: "device:control-surface:1".into(),
                direction: signal_runtime::RuntimeExternalMidiEndpointDirection::Input,
                lifecycle_state: signal_runtime::RuntimeExternalMidiEndpointLifecycleState::Active,
                route_state: signal_runtime::RuntimeExternalMidiRouteState::InputObserved,
                capability: capability.clone(),
                summary: "input".into(),
            },
            signal_runtime::RuntimeExternalMidiEndpointDescriptor {
                endpoint_id: "endpoint:control-surface:output".into(),
                endpoint_name: "Control Surface Output".into(),
                device_id: "device:control-surface:1".into(),
                direction: signal_runtime::RuntimeExternalMidiEndpointDirection::Output,
                lifecycle_state: signal_runtime::RuntimeExternalMidiEndpointLifecycleState::Active,
                route_state: signal_runtime::RuntimeExternalMidiRouteState::OutputObserved,
                capability,
                summary: "output".into(),
            },
        ],
        summary: "provider=public-control-surface state=Stable devices=1 endpoints=2 routes=1 guarded-routes=1".into(),
    };

    let observation = unavailable
        .clone()
        .with_external_midi_snapshot(control_surface_graph.clone());
    assert_eq!(
        observation.control_surface_snapshot.discovery_state,
        RuntimeExternalMidiDiscoveryState::Enumerated
    );
    assert_eq!(
        observation.control_surface_snapshot.graph_state,
        signal_runtime::RuntimeControlSurfaceGraphState::Guarded
    );
    assert_eq!(
        observation.control_surface_snapshot.provider_name,
        "public-control-surface"
    );
    assert_eq!(observation.control_surface_snapshot.device_count, 1);
    assert_eq!(observation.control_surface_snapshot.mapped_device_count, 1);
    assert_eq!(
        observation
            .control_surface_snapshot
            .feedback_ready_device_count,
        0
    );
    assert_eq!(observation.control_surface_snapshot.guarded_device_count, 1);
    assert_eq!(
        observation.control_surface_snapshot.devices[0].transport_posture,
        signal_runtime::RuntimeControlSurfaceTransportPosture::Guarded
    );
    assert_eq!(
        observation.control_surface_snapshot.devices[0].mapping_posture,
        signal_runtime::RuntimeControlSurfaceMappingPosture::Guarded
    );
    assert_eq!(
        observation.control_surface_snapshot.devices[0].feedback_readiness,
        signal_runtime::RuntimeControlSurfaceFeedbackReadiness::Guarded
    );
    assert!(
        observation.control_surface_snapshot.devices[0]
            .capability
            .supports_feedback_output
    );

    let observation_json = observation.render_json();
    assert!(observation_json.contains("\"control_surface_snapshot\":{"));
    assert!(observation_json.contains("\"graph_state\":\"Guarded\""));
    assert!(observation_json.contains("\"provider_name\":\"public-control-surface\""));
    assert!(observation_json.contains("\"feedback_ready_device_count\":0"));
    assert!(observation_json.contains("\"mapping_posture\":\"Guarded\""));

    let mut supervisor = RuntimeSupervisorReport::capture(&runtime, &recorder);
    supervisor.observation = supervisor
        .observation
        .clone()
        .with_external_midi_snapshot(control_surface_graph);
    let supervisor_json = supervisor.render_json();
    assert!(supervisor_json.contains("\"control_surface_snapshot\":{"));
    assert!(supervisor_json.contains("\"transport_posture\":\"Guarded\""));
    assert!(supervisor_json.contains("\"feedback_readiness\":\"Guarded\""));
}
