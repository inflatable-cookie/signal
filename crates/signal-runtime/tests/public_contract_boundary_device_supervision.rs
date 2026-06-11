use signal_runtime::{
    HandshakeRequest, RuntimeConfig, RuntimeConfigRequest, RuntimeDeviceFaultBoundaryState,
    RuntimeDeviceRestartState, RuntimeDeviceSupervisionState, RuntimeError, RuntimeErrorKind,
    RuntimeEventRecorder, RuntimeInterruptionClass, RuntimeLifecycleApi, RuntimeObservationReport,
    RuntimeRecoveryState, RuntimeWatchdogTrigger, SignalRuntime, WatchdogRestartRecord,
};

#[test]
fn public_runtime_device_supervision_boundary_reports_recovering_and_faulted_runtime_states() {
    let mut recovering = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
    recovering
        .handshake(HandshakeRequest {
            client_version: "public-runtime-device-supervision-recovering".into(),
            anticipative_preferred: true,
            max_sample_rate_hint: Some(96_000),
        })
        .expect("public device supervision recovering handshake should succeed");
    recovering
        .configure(RuntimeConfigRequest::new(48_000, 512))
        .expect("public device supervision recovering configure should succeed");
    recovering
        .start()
        .expect("public device supervision recovering start should succeed");
    recovering.record_watchdog_restart(WatchdogRestartRecord {
        sandbox_id: "public-runtime-device-supervision-watchdog".into(),
        trigger: RuntimeWatchdogTrigger::HeartbeatMisses,
        processing_epoch: 2,
    });

    let recovering_observation =
        RuntimeObservationReport::capture(&recovering, &RuntimeEventRecorder::default());
    assert_eq!(
        recovering_observation.device_supervision_snapshot.state,
        RuntimeDeviceSupervisionState::Stable
    );
    assert_eq!(
        recovering_observation
            .device_supervision_snapshot
            .restart_state,
        RuntimeDeviceRestartState::Recovered
    );
    assert_eq!(
        recovering_observation
            .device_supervision_snapshot
            .fault_boundary,
        RuntimeDeviceFaultBoundaryState::Clear
    );
    assert_eq!(
        recovering_observation
            .device_supervision_snapshot
            .interruption_class,
        RuntimeInterruptionClass::Steady
    );
    assert_eq!(
        recovering_observation
            .device_supervision_snapshot
            .recovery_state,
        RuntimeRecoveryState::Steady
    );
    assert_eq!(
        recovering_observation
            .device_supervision_snapshot
            .watchdog_restart_count,
        1
    );

    let mut faulted = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
    faulted
        .handshake(HandshakeRequest {
            client_version: "public-runtime-device-supervision-faulted".into(),
            anticipative_preferred: true,
            max_sample_rate_hint: Some(96_000),
        })
        .expect("public device supervision faulted handshake should succeed");
    faulted
        .configure(RuntimeConfigRequest::new(48_000, 512))
        .expect("public device supervision faulted configure should succeed");
    faulted
        .start()
        .expect("public device supervision faulted start should succeed");
    let readiness = faulted.fail_runtime(RuntimeError::new(
        RuntimeErrorKind::HardwareFailure,
        "public runtime device supervision fault boundary",
    ));
    assert!(matches!(
        readiness,
        signal_runtime::RuntimeReadiness::Failed { .. }
    ));

    let faulted_observation =
        RuntimeObservationReport::capture(&faulted, &RuntimeEventRecorder::default());
    let _faulted_supervisor = signal_runtime::RuntimeSupervisorReport::capture(
        &faulted,
        &RuntimeEventRecorder::default(),
    );
    assert_eq!(
        faulted_observation.device_supervision_snapshot.state,
        RuntimeDeviceSupervisionState::Faulted
    );
    assert_eq!(
        faulted_observation
            .device_supervision_snapshot
            .restart_state,
        RuntimeDeviceRestartState::Faulted
    );
    assert_eq!(
        faulted_observation
            .device_supervision_snapshot
            .fault_boundary,
        RuntimeDeviceFaultBoundaryState::Faulted
    );
    assert_eq!(
        faulted_observation
            .device_supervision_snapshot
            .recovery_state,
        RuntimeRecoveryState::Faulted
    );
    assert_eq!(
        faulted_observation
            .device_supervision_snapshot
            .primary_fault_cause,
        Some(signal_runtime::RuntimeFaultCause::RuntimeError)
    );
}
