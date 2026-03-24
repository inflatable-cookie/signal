#[path = "support/public_contract_boundary_advanced_hardware.rs"]
mod public_contract_boundary_advanced_hardware_support;

use public_contract_boundary_advanced_hardware_support::{
    assert_advanced_hardware_json, assert_advanced_hardware_supervisor_json,
    assert_guarded_advanced_hardware, assert_unavailable_advanced_hardware,
};
use signal_runtime::{
    RuntimeConfig, RuntimeEventRecorder, RuntimeObservationReport, SignalRuntime,
};

#[test]
fn public_runtime_advanced_hardware_boundary_reports_runtime_owned_policy_and_feedback_truth() {
    let runtime = SignalRuntime::new(RuntimeConfig::server(48_000, 512));
    let recorder = RuntimeEventRecorder::default();

    let unavailable = RuntimeObservationReport::capture(&runtime, &recorder);
    assert_unavailable_advanced_hardware(&unavailable);

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
    let advanced_hardware_graph = signal_runtime::RuntimeExternalMidiEndpointGraphSnapshot {
        discovery_state: signal_runtime::RuntimeExternalMidiDiscoveryState::Enumerated,
        graph_state: signal_runtime::RuntimeExternalMidiGraphState::Stable,
        live_ownership:
            signal_runtime::RuntimeExternalMidiLiveOwnershipSummary::detached_without_backend_context(),
        provider_name: "public-advanced-hardware".into(),
        device_count: 1,
        endpoint_count: 2,
        input_endpoint_count: 1,
        output_endpoint_count: 1,
        duplex_endpoint_count: 1,
        active_route_count: 1,
        guarded_route_count: 1,
        devices: vec![signal_runtime::RuntimeExternalMidiDeviceDescriptor {
            device_id: "device:advanced-hardware:1".into(),
            device_name: "Advanced Surface".into(),
            lifecycle_state: signal_runtime::RuntimeExternalMidiDeviceLifecycleState::Discovered,
            endpoint_count: 2,
            summary: "device=Advanced Surface endpoints=2".into(),
        }],
        endpoints: vec![
            signal_runtime::RuntimeExternalMidiEndpointDescriptor {
                endpoint_id: "endpoint:advanced-hardware:input".into(),
                endpoint_name: "Advanced Surface Input".into(),
                device_id: "device:advanced-hardware:1".into(),
                direction: signal_runtime::RuntimeExternalMidiEndpointDirection::Input,
                lifecycle_state: signal_runtime::RuntimeExternalMidiEndpointLifecycleState::Active,
                route_state: signal_runtime::RuntimeExternalMidiRouteState::InputObserved,
                capability: capability.clone(),
                summary: "input".into(),
            },
            signal_runtime::RuntimeExternalMidiEndpointDescriptor {
                endpoint_id: "endpoint:advanced-hardware:output".into(),
                endpoint_name: "Advanced Surface Output".into(),
                device_id: "device:advanced-hardware:1".into(),
                direction: signal_runtime::RuntimeExternalMidiEndpointDirection::Output,
                lifecycle_state: signal_runtime::RuntimeExternalMidiEndpointLifecycleState::Active,
                route_state: signal_runtime::RuntimeExternalMidiRouteState::OutputObserved,
                capability,
                summary: "output".into(),
            },
        ],
        summary: "provider=public-advanced-hardware state=Stable devices=1 endpoints=2 routes=1 guarded-routes=1".into(),
    };

    let observation = unavailable
        .clone()
        .with_external_midi_snapshot(advanced_hardware_graph.clone());
    assert_guarded_advanced_hardware(&observation);
    assert_advanced_hardware_json(&observation);
    assert_advanced_hardware_supervisor_json(&runtime, &recorder, advanced_hardware_graph);
}
