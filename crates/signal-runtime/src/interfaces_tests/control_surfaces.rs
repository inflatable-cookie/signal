use super::*;

#[test]
fn runtime_control_surface_snapshot_derives_from_external_midi_baselines() {
    let unavailable = RuntimeControlSurfaceSnapshot::from_external_midi_snapshot(
        &RuntimeExternalMidiEndpointGraphSnapshot::unavailable(),
    );
    assert_eq!(
        unavailable.discovery_state,
        RuntimeExternalMidiDiscoveryState::Unavailable
    );
    assert_eq!(
        unavailable.graph_state,
        RuntimeControlSurfaceGraphState::Unavailable
    );
    assert_eq!(unavailable.device_count, 0);

    let empty = RuntimeControlSurfaceSnapshot::from_external_midi_snapshot(
        &RuntimeExternalMidiEndpointGraphSnapshot::empty("signal-host-local"),
    );
    assert_eq!(
        empty.discovery_state,
        RuntimeExternalMidiDiscoveryState::Idle
    );
    assert_eq!(empty.graph_state, RuntimeControlSurfaceGraphState::Empty);
    assert_eq!(empty.provider_name, "signal-host-local");
    assert_eq!(empty.device_count, 0);

    let capability = RuntimeExternalMidiEndpointCapabilitySummary {
        supports_bounded_midi_input: true,
        supports_bounded_midi_output: true,
        supports_transport_clock: true,
        supports_note_events: true,
        supports_controller_events: true,
        supports_note_pressure_expression: true,
        supports_note_timbre_expression: false,
        supports_note_tuning_expression: false,
        supports_mpe: false,
        midi2_posture: RuntimeControllerExpressionMidi2Posture::Unsupported,
        control_surface_guarded: true,
        summary:
            "transport-clock=true controller-events=true pressure=true control-surface=guarded"
                .into(),
    };
    let derived = RuntimeControlSurfaceSnapshot::from_external_midi_snapshot(
        &RuntimeExternalMidiEndpointGraphSnapshot {
            discovery_state: RuntimeExternalMidiDiscoveryState::Enumerated,
            graph_state: RuntimeExternalMidiGraphState::Stable,
            live_ownership:
                RuntimeExternalMidiLiveOwnershipSummary::detached_without_backend_context(),
            provider_name: "control-surface-provider".into(),
            device_count: 1,
            endpoint_count: 1,
            input_endpoint_count: 1,
            output_endpoint_count: 1,
            duplex_endpoint_count: 1,
            active_route_count: 1,
            guarded_route_count: 1,
            devices: vec![RuntimeExternalMidiDeviceDescriptor {
                device_id: "device:surface".into(),
                device_name: "Surface".into(),
                lifecycle_state: RuntimeExternalMidiDeviceLifecycleState::Discovered,
                endpoint_count: 1,
                summary: "surface device".into(),
            }],
            endpoints: vec![RuntimeExternalMidiEndpointDescriptor {
                endpoint_id: "endpoint:surface".into(),
                endpoint_name: "Surface Duplex".into(),
                device_id: "device:surface".into(),
                direction: RuntimeExternalMidiEndpointDirection::Duplex,
                lifecycle_state: RuntimeExternalMidiEndpointLifecycleState::Active,
                route_state: RuntimeExternalMidiRouteState::DuplexObserved,
                capability,
                summary: "surface endpoint".into(),
            }],
            summary: "control surface external midi".into(),
        },
    );
    assert_eq!(
        derived.graph_state,
        RuntimeControlSurfaceGraphState::Guarded
    );
    assert_eq!(derived.device_count, 1);
    assert_eq!(derived.mapped_device_count, 1);
    assert_eq!(derived.feedback_ready_device_count, 0);
    assert_eq!(derived.guarded_device_count, 1);
    assert_eq!(
        derived.devices[0].transport_posture,
        RuntimeControlSurfaceTransportPosture::Guarded
    );
    assert_eq!(
        derived.devices[0].mapping_posture,
        RuntimeControlSurfaceMappingPosture::Guarded
    );
    assert_eq!(
        derived.devices[0].feedback_readiness,
        RuntimeControlSurfaceFeedbackReadiness::Guarded
    );
    assert!(derived.devices[0].capability.supports_widened_expression);
}

#[test]
fn runtime_advanced_hardware_snapshot_derives_from_control_surface_baselines() {
    let unavailable = RuntimeAdvancedHardwareSnapshot::from_control_surface_snapshot(
        &RuntimeControlSurfaceSnapshot::unavailable(),
    );
    assert_eq!(
        unavailable.discovery_state,
        RuntimeExternalMidiDiscoveryState::Unavailable
    );
    assert_eq!(
        unavailable.graph_state,
        RuntimeAdvancedHardwareGraphState::Unavailable
    );
    assert_eq!(unavailable.device_count, 0);

    let empty = RuntimeAdvancedHardwareSnapshot::from_control_surface_snapshot(
        &RuntimeControlSurfaceSnapshot::empty("signal-host-local"),
    );
    assert_eq!(
        empty.discovery_state,
        RuntimeExternalMidiDiscoveryState::Idle
    );
    assert_eq!(empty.graph_state, RuntimeAdvancedHardwareGraphState::Empty);
    assert_eq!(empty.provider_name, "signal-host-local");
    assert_eq!(empty.device_count, 0);
    assert_eq!(empty.display_transport_device_count, 0);
    assert_eq!(empty.motor_transport_device_count, 0);
    assert_eq!(empty.haptic_transport_device_count, 0);
    assert_eq!(empty.scene_mapping_device_count, 0);
    assert_eq!(empty.feedback_page_device_count, 0);
    assert_eq!(empty.safe_action_graph_device_count, 0);

    let advanced = RuntimeAdvancedHardwareSnapshot::from_control_surface_snapshot(
        &RuntimeControlSurfaceSnapshot {
            discovery_state: RuntimeExternalMidiDiscoveryState::Enumerated,
            graph_state: RuntimeControlSurfaceGraphState::Guarded,
            provider_name: "advanced-hardware-provider".into(),
            device_count: 1,
            mapped_device_count: 1,
            feedback_ready_device_count: 0,
            guarded_device_count: 1,
            devices: vec![RuntimeControlSurfaceDeviceDescriptor {
                device_id: "device:surface".into(),
                device_name: "Surface".into(),
                transport_posture: RuntimeControlSurfaceTransportPosture::Guarded,
                mapping_posture: RuntimeControlSurfaceMappingPosture::Guarded,
                feedback_readiness: RuntimeControlSurfaceFeedbackReadiness::Guarded,
                capability: RuntimeControlSurfaceCapabilitySummary {
                    supports_transport_control: true,
                    supports_mapping_input: true,
                    supports_feedback_output: true,
                    supports_widened_expression: true,
                    summary: "guarded control surface".into(),
                },
                summary: "guarded surface".into(),
            }],
            summary: "advanced control surface".into(),
        },
    );
    assert_eq!(
        advanced.graph_state,
        RuntimeAdvancedHardwareGraphState::Guarded
    );
    assert_eq!(advanced.device_count, 1);
    assert_eq!(advanced.portable_device_count, 0);
    assert_eq!(advanced.guarded_device_count, 1);
    assert_eq!(advanced.context_only_device_count, 0);
    assert_eq!(advanced.denied_device_count, 0);
    assert_eq!(advanced.feedback_channel_device_count, 1);
    assert_eq!(advanced.display_transport_device_count, 1);
    assert_eq!(advanced.motor_transport_device_count, 0);
    assert_eq!(advanced.haptic_transport_device_count, 0);
    assert_eq!(advanced.scene_mapping_device_count, 1);
    assert_eq!(advanced.feedback_page_device_count, 1);
    assert_eq!(advanced.safe_action_graph_device_count, 1);
    assert_eq!(
        advanced.devices[0].scripting_safe_posture,
        RuntimeScriptingSafeDevicePolicyPosture::Guarded
    );
    assert_eq!(
        advanced.devices[0].feedback_channel_posture,
        RuntimeGuardedFeedbackChannelPosture::Guarded
    );
    assert!(advanced.devices[0].capability.supports_display_feedback);
    assert!(advanced.devices[0].capability.supports_bank_navigation);
    assert!(advanced.devices[0].capability.supports_macro_triggers);
    assert!(
        advanced.devices[0]
            .capability
            .supports_device_state_observation
    );
    assert_eq!(
        advanced.devices[0].display_transport_posture,
        RuntimeDisplayTransportPosture::GuardedDisplay
    );
    assert_eq!(
        advanced.devices[0].display_content_class,
        RuntimeDisplayContentClass::GuardedVendorDisplay
    );
    assert_eq!(
        advanced.devices[0].motor_transport_posture,
        RuntimeMotorTransportPosture::NoMotorTransport
    );
    assert_eq!(
        advanced.devices[0].haptic_transport_posture,
        RuntimeHapticTransportPosture::NoHapticTransport
    );
    assert_eq!(
        advanced.devices[0].feedback_authority,
        RuntimeAdvancedControlFeedbackAuthority::RuntimeDefault
    );
    assert_eq!(
        advanced.devices[0].feedback_outcome,
        RuntimeAdvancedControlFeedbackOutcome::CollapseToGuardedFeedback
    );
    assert_eq!(
        advanced.devices[0].scene_mapping_posture,
        RuntimeSceneMappingPosture::GuardedSceneMapping
    );
    assert_eq!(
        advanced.devices[0].feedback_page_posture,
        RuntimeFeedbackPagePosture::GuardedFeedbackPages
    );
    assert_eq!(
        advanced.devices[0].feedback_page_class,
        RuntimeFeedbackPageClass::GuardedVendorPage
    );
    assert_eq!(
        advanced.devices[0].safe_action_graph_posture,
        RuntimeSafeActionGraphPosture::GuardedSafeActionGraph
    );
    assert_eq!(
        advanced.devices[0].action_authority,
        RuntimeControlSurfaceWorkflowAuthority::RuntimeDefault
    );
    assert_eq!(
        advanced.devices[0].safe_action_outcome,
        RuntimeSafeActionOutcome::CollapseToGuardedAction
    );
    assert!(advanced.devices[0]
        .capability
        .action_classes
        .contains(&RuntimeAdvancedHardwareActionClass::DisplayFeedback));
    assert!(advanced.devices[0]
        .capability
        .action_classes
        .contains(&RuntimeAdvancedHardwareActionClass::MacroTrigger));
}
