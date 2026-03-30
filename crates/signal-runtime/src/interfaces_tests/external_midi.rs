use super::*;

#[test]
fn runtime_external_midi_endpoint_graph_snapshot_distinguishes_unavailable_from_empty() {
    let unavailable = RuntimeExternalMidiEndpointGraphSnapshot::unavailable();
    assert_eq!(
        unavailable.discovery_state,
        RuntimeExternalMidiDiscoveryState::Unavailable
    );
    assert_eq!(
        unavailable.graph_state,
        RuntimeExternalMidiGraphState::Unavailable
    );
    assert_eq!(unavailable.provider_name, "runtime-unavailable");
    assert_eq!(unavailable.device_count, 0);
    assert_eq!(unavailable.endpoint_count, 0);
    assert_eq!(
        unavailable.live_ownership.ownership_posture,
        RuntimeExternalMidiLiveOwnershipPosture::Unavailable
    );
    assert!(unavailable.devices.is_empty());
    assert!(unavailable.endpoints.is_empty());
    assert!(unavailable.summary.contains("graph=Unavailable"));

    let empty = RuntimeExternalMidiEndpointGraphSnapshot::empty("signal-host-local");
    assert_eq!(
        empty.discovery_state,
        RuntimeExternalMidiDiscoveryState::Idle
    );
    assert_eq!(empty.graph_state, RuntimeExternalMidiGraphState::Empty);
    assert_eq!(empty.provider_name, "signal-host-local");
    assert_eq!(empty.device_count, 0);
    assert_eq!(empty.endpoint_count, 0);
    assert_eq!(empty.active_route_count, 0);
    assert_eq!(
        empty.live_ownership.ownership_posture,
        RuntimeExternalMidiLiveOwnershipPosture::NoLiveOwnership
    );
    assert_eq!(
        empty.live_ownership.attach_continuity,
        RuntimeExternalMidiAttachContinuity::Detached
    );
    assert!(empty.summary.contains("graph=Empty"));
}

#[test]
fn runtime_external_midi_live_ownership_summary_derives_runtime_owned_baselines() {
    let unavailable = RuntimeExternalMidiEndpointGraphSnapshot::empty("runtime-test")
        .with_live_ownership_summary(
            &RuntimeLinuxBackendSessionSnapshot::unavailable(),
            &RuntimeInterruptionSummary {
                active: false,
                class: RuntimeInterruptionClass::Steady,
                rebindable: false,
                recovery_state: RuntimeRecoveryState::Steady,
                primary_fault_cause: None,
                safe_mode_enabled: false,
                deferred_service_class: None,
                deferred_service_decision: None,
                summary: "steady".into(),
            },
        );
    assert_eq!(
        unavailable.live_ownership.ownership_posture,
        RuntimeExternalMidiLiveOwnershipPosture::Unavailable
    );
    assert_eq!(
        unavailable.live_ownership.backend_parity,
        RuntimeExternalMidiBackendParity::Unavailable
    );

    let not_linux_host = host_io_summary(
        RuntimeHostClockFallbackState::Direct,
        RuntimeHostClockTransitionState::Stable,
        RuntimeHostAudioStreamState::Running,
        BackendHealth::Healthy,
        0,
        0,
        0,
    );
    let not_linux = RuntimeExternalMidiEndpointGraphSnapshot::empty("coreaudio-test")
        .with_live_ownership_summary(
            &RuntimeLinuxBackendSessionSnapshot::from_host_io(&not_linux_host),
            &RuntimeInterruptionSummary {
                active: false,
                class: RuntimeInterruptionClass::Steady,
                rebindable: false,
                recovery_state: RuntimeRecoveryState::Steady,
                primary_fault_cause: None,
                safe_mode_enabled: false,
                deferred_service_class: None,
                deferred_service_decision: None,
                summary: "steady".into(),
            },
        );
    assert_eq!(
        not_linux.live_ownership.ownership_posture,
        RuntimeExternalMidiLiveOwnershipPosture::NoLiveOwnership
    );
    assert_eq!(
        not_linux.live_ownership.backend_parity,
        RuntimeExternalMidiBackendParity::NotLinux
    );
    assert_eq!(
        not_linux.live_ownership.guarded_parity_outcome,
        RuntimeExternalMidiGuardedParityOutcome::NotLinux
    );

    let pipewire_host = linux_host_io_summary(
        HardwareBackendIdentity::Linux(LinuxAudioBackendKind::PipeWire),
        RuntimeHostLifecycleOwnership::BackendManagedCallback,
        RuntimeHostAudioStreamState::Running,
        BackendHealth::Healthy,
        0,
        0,
        0,
    );
    let pipewire = RuntimeExternalMidiEndpointGraphSnapshot::empty("pipewire-test")
        .with_live_ownership_summary(
            &RuntimeLinuxBackendSessionSnapshot::from_host_io(&pipewire_host),
            &RuntimeInterruptionSummary {
                active: false,
                class: RuntimeInterruptionClass::Steady,
                rebindable: false,
                recovery_state: RuntimeRecoveryState::Steady,
                primary_fault_cause: None,
                safe_mode_enabled: false,
                deferred_service_class: None,
                deferred_service_decision: None,
                summary: "steady".into(),
            },
        );
    assert_eq!(
        pipewire.live_ownership.ownership_posture,
        RuntimeExternalMidiLiveOwnershipPosture::NoLiveOwnership
    );
    assert_eq!(
        pipewire.live_ownership.attach_continuity,
        RuntimeExternalMidiAttachContinuity::Detached
    );
    assert_eq!(
        pipewire.live_ownership.backend_parity,
        RuntimeExternalMidiBackendParity::Guarded
    );
    assert_eq!(
        pipewire.live_ownership.guarded_parity_outcome,
        RuntimeExternalMidiGuardedParityOutcome::BackendManaged
    );
}
