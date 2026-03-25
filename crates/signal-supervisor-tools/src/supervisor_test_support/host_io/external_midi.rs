pub(crate) fn sample_g07_external_midi_snapshot(
) -> signal_runtime::RuntimeExternalMidiEndpointGraphSnapshot {
    signal_runtime::RuntimeExternalMidiEndpointGraphSnapshot {
        discovery_state: signal_runtime::RuntimeExternalMidiDiscoveryState::Enumerated,
        graph_state: signal_runtime::RuntimeExternalMidiGraphState::Stable,
        live_ownership:
            signal_runtime::RuntimeExternalMidiLiveOwnershipSummary::detached_without_backend_context(),
        provider_name: "signal-host-local".into(),
        device_count: 1,
        endpoint_count: 1,
        input_endpoint_count: 1,
        output_endpoint_count: 1,
        duplex_endpoint_count: 1,
        active_route_count: 1,
        guarded_route_count: 0,
        devices: vec![signal_runtime::RuntimeExternalMidiDeviceDescriptor {
            device_id: "device:controller:main".into(),
            device_name: "Signal Controller".into(),
            lifecycle_state: signal_runtime::RuntimeExternalMidiDeviceLifecycleState::Discovered,
            endpoint_count: 1,
            summary: "device Signal Controller lifecycle=Discovered endpoints=1".into(),
        }],
        endpoints: vec![signal_runtime::RuntimeExternalMidiEndpointDescriptor {
            endpoint_id: "endpoint:controller:duplex".into(),
            endpoint_name: "Signal Controller Duplex".into(),
            device_id: "device:controller:main".into(),
            direction: signal_runtime::RuntimeExternalMidiEndpointDirection::Duplex,
            lifecycle_state: signal_runtime::RuntimeExternalMidiEndpointLifecycleState::Active,
            route_state: signal_runtime::RuntimeExternalMidiRouteState::DuplexObserved,
            capability: signal_runtime::RuntimeExternalMidiEndpointCapabilitySummary {
                supports_bounded_midi_input: true,
                supports_bounded_midi_output: true,
                supports_transport_clock: true,
                supports_note_events: true,
                supports_controller_events: true,
                supports_note_pressure_expression: true,
                supports_note_timbre_expression: true,
                supports_note_tuning_expression: false,
                supports_mpe: true,
                midi2_posture: signal_runtime::RuntimeControllerExpressionMidi2Posture::Guarded,
                control_surface_guarded: false,
                summary: "midi-input=true midi-output=true transport-clock=true note-events=true controller-events=true pressure=true timbre=true tuning=false mpe=true midi2=Guarded control-surface=portable".into(),
            },
            summary:
                "endpoint Signal Controller Duplex direction=Duplex route=DuplexObserved lifecycle=Active"
                    .into(),
        }],
        summary: "discovery=Ready graph=Ready provider=signal-host-local devices=1 endpoints=1 routes=1".into(),
    }
}

pub(crate) fn sample_control_preview_workflow_external_midi_snapshot(
) -> signal_runtime::RuntimeExternalMidiEndpointGraphSnapshot {
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
    signal_runtime::RuntimeExternalMidiEndpointGraphSnapshot {
        discovery_state: signal_runtime::RuntimeExternalMidiDiscoveryState::Enumerated,
        graph_state: signal_runtime::RuntimeExternalMidiGraphState::Stable,
        live_ownership:
            signal_runtime::RuntimeExternalMidiLiveOwnershipSummary::detached_without_backend_context(),
        provider_name: "public-control-preview-workflow".into(),
        device_count: 1,
        endpoint_count: 2,
        input_endpoint_count: 1,
        output_endpoint_count: 1,
        duplex_endpoint_count: 1,
        active_route_count: 1,
        guarded_route_count: 1,
        devices: vec![signal_runtime::RuntimeExternalMidiDeviceDescriptor {
            device_id: "device:control-preview-workflow:1".into(),
            device_name: "Control Preview Workflow Surface".into(),
            lifecycle_state: signal_runtime::RuntimeExternalMidiDeviceLifecycleState::Discovered,
            endpoint_count: 2,
            summary: "device=Control Preview Workflow Surface endpoints=2".into(),
        }],
        endpoints: vec![
            signal_runtime::RuntimeExternalMidiEndpointDescriptor {
                endpoint_id: "endpoint:control-preview-workflow:input".into(),
                endpoint_name: "Control Preview Workflow Input".into(),
                device_id: "device:control-preview-workflow:1".into(),
                direction: signal_runtime::RuntimeExternalMidiEndpointDirection::Input,
                lifecycle_state: signal_runtime::RuntimeExternalMidiEndpointLifecycleState::Active,
                route_state: signal_runtime::RuntimeExternalMidiRouteState::InputObserved,
                capability: capability.clone(),
                summary: "input".into(),
            },
            signal_runtime::RuntimeExternalMidiEndpointDescriptor {
                endpoint_id: "endpoint:control-preview-workflow:output".into(),
                endpoint_name: "Control Preview Workflow Output".into(),
                device_id: "device:control-preview-workflow:1".into(),
                direction: signal_runtime::RuntimeExternalMidiEndpointDirection::Output,
                lifecycle_state: signal_runtime::RuntimeExternalMidiEndpointLifecycleState::Active,
                route_state: signal_runtime::RuntimeExternalMidiRouteState::OutputObserved,
                capability,
                summary: "output".into(),
            },
        ],
        summary: "provider=public-control-preview-workflow state=Stable devices=1 endpoints=2 routes=1 guarded-routes=1".into(),
    }
}
