use super::*;

#[test]
fn runtime_device_supervision_snapshot_tracks_recovered_device_episode() {
    let effective_config = EffectiveRuntimeConfig {
        sample_rate: SampleRate(48_000),
        block_size: 256,
        anticipative_enabled: true,
        safe_mode_enabled: false,
        active_output_device: Some("device:main".into()),
    };
    let supervision_snapshot = RuntimeSupervisionSnapshot {
        watchdog_restart_count: 1,
        safe_mode_enabled: false,
        xrun_overload_active: false,
        last_watchdog_trigger: Some(RuntimeWatchdogTrigger::HeartbeatMisses),
        last_sandbox_id: Some("sandbox:main".into()),
        last_processing_epoch: Some(7),
    };
    let fault_status = RuntimeFaultStatusSnapshot {
        recovery_state: RuntimeRecoveryState::Steady,
        primary_fault_cause: None,
        active_fault_count: 0,
        xrun_overload_active: false,
        plugin_fault_active: false,
        watchdog_active: false,
        device_loss_active: false,
        transport_fault_active: false,
        missing_plugin_binding_active: false,
        safe_mode_enabled: false,
        restart_count: 0,
        watchdog_restart_count: 1,
        plugin_fault_count: 0,
        transport_faulted_session_count: 0,
        device_loss_count: 1,
    };
    let interruption_summary = RuntimeInterruptionSummary {
        active: false,
        class: RuntimeInterruptionClass::Steady,
        rebindable: false,
        recovery_state: RuntimeRecoveryState::Steady,
        primary_fault_cause: None,
        safe_mode_enabled: false,
        deferred_service_class: None,
        deferred_service_decision: None,
    };
    let host_io = host_io_summary(
        RuntimeHostClockFallbackState::Direct,
        RuntimeHostClockTransitionState::ReturnedToDirect,
        RuntimeHostAudioStreamState::Running,
        BackendHealth::Healthy,
        1,
        0,
        1,
    );

    let snapshot = RuntimeDeviceSupervisionSnapshot::capture(
        &effective_config,
        &supervision_snapshot,
        &fault_status,
        &interruption_summary,
        Some(&host_io),
    );

    assert_eq!(snapshot.state, RuntimeDeviceSupervisionState::Stable);
    assert_eq!(snapshot.restart_state, RuntimeDeviceRestartState::Recovered);
    assert_eq!(
        snapshot.fault_boundary,
        RuntimeDeviceFaultBoundaryState::Clear
    );
    assert_eq!(snapshot.device_loss_count, 1);
    assert_eq!(snapshot.restart_attempt_count, Some(1));
    assert_eq!(snapshot.restart_failure_count, Some(0));
    assert_eq!(snapshot.backend_health, Some(BackendHealth::Healthy));
}

#[test]
fn runtime_device_supervision_snapshot_distinguishes_exhausted_from_faulted() {
    let effective_config = EffectiveRuntimeConfig {
        sample_rate: SampleRate(48_000),
        block_size: 256,
        anticipative_enabled: true,
        safe_mode_enabled: true,
        active_output_device: Some("device:main".into()),
    };
    let supervision_snapshot = RuntimeSupervisionSnapshot {
        watchdog_restart_count: 2,
        safe_mode_enabled: true,
        xrun_overload_active: false,
        last_watchdog_trigger: Some(RuntimeWatchdogTrigger::DeadlineMisses),
        last_sandbox_id: Some("sandbox:main".into()),
        last_processing_epoch: Some(11),
    };
    let exhausted_status = RuntimeFaultStatusSnapshot {
        recovery_state: RuntimeRecoveryState::Recovering,
        primary_fault_cause: Some(RuntimeFaultCause::DeviceLoss),
        active_fault_count: 1,
        xrun_overload_active: false,
        plugin_fault_active: false,
        watchdog_active: false,
        device_loss_active: true,
        transport_fault_active: false,
        missing_plugin_binding_active: false,
        safe_mode_enabled: true,
        restart_count: 0,
        watchdog_restart_count: 2,
        plugin_fault_count: 0,
        transport_faulted_session_count: 0,
        device_loss_count: 1,
    };
    let exhausted_interruption = RuntimeInterruptionSummary {
        active: true,
        class: RuntimeInterruptionClass::Restartable,
        rebindable: true,
        recovery_state: RuntimeRecoveryState::Recovering,
        primary_fault_cause: Some(RuntimeFaultCause::DeviceLoss),
        safe_mode_enabled: true,
        deferred_service_class: None,
        deferred_service_decision: None,
    };
    let exhausted_host_io = host_io_summary(
        RuntimeHostClockFallbackState::RecoveryConstrained,
        RuntimeHostClockTransitionState::EnteredRecoveryFallback,
        RuntimeHostAudioStreamState::Faulted,
        BackendHealth::Recovering,
        1,
        1,
        1,
    );

    let exhausted = RuntimeDeviceSupervisionSnapshot::capture(
        &effective_config,
        &supervision_snapshot,
        &exhausted_status,
        &exhausted_interruption,
        Some(&exhausted_host_io),
    );
    assert_eq!(exhausted.state, RuntimeDeviceSupervisionState::Exhausted);
    assert_eq!(
        exhausted.restart_state,
        RuntimeDeviceRestartState::Exhausted
    );
    assert_eq!(
        exhausted.fault_boundary,
        RuntimeDeviceFaultBoundaryState::Exhausted
    );

    let faulted_status = RuntimeFaultStatusSnapshot {
        recovery_state: RuntimeRecoveryState::Faulted,
        primary_fault_cause: Some(RuntimeFaultCause::RuntimeError),
        active_fault_count: 1,
        xrun_overload_active: false,
        plugin_fault_active: false,
        watchdog_active: false,
        device_loss_active: false,
        transport_fault_active: false,
        missing_plugin_binding_active: false,
        safe_mode_enabled: true,
        restart_count: 0,
        watchdog_restart_count: 2,
        plugin_fault_count: 0,
        transport_faulted_session_count: 0,
        device_loss_count: 1,
    };
    let faulted_interruption = RuntimeInterruptionSummary {
        active: true,
        class: RuntimeInterruptionClass::Terminal,
        rebindable: false,
        recovery_state: RuntimeRecoveryState::Faulted,
        primary_fault_cause: Some(RuntimeFaultCause::RuntimeError),
        safe_mode_enabled: true,
        deferred_service_class: None,
        deferred_service_decision: None,
    };

    let faulted = RuntimeDeviceSupervisionSnapshot::capture(
        &effective_config,
        &supervision_snapshot,
        &faulted_status,
        &faulted_interruption,
        None,
    );
    assert_eq!(faulted.state, RuntimeDeviceSupervisionState::Faulted);
    assert_eq!(faulted.restart_state, RuntimeDeviceRestartState::Faulted);
    assert_eq!(
        faulted.fault_boundary,
        RuntimeDeviceFaultBoundaryState::Faulted
    );
}
