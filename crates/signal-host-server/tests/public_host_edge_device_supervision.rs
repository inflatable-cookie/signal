use signal_host_server::ServerRuntimeHost;
use signal_runtime::{
    RuntimeConfig, RuntimeConfigRequest, RuntimeDeviceFaultBoundaryState,
    RuntimeDeviceRestartState, RuntimeDeviceSupervisionState, RuntimeError, RuntimeErrorKind,
    RuntimeInterruptionClass, RuntimeLifecycleApi, RuntimeRecoveryState, RuntimeWatchdogTrigger,
    SignalRuntime, WatchdogRestartRecord,
};

#[test]
fn server_shared_host_edge_exports_runtime_device_supervision_truth() {
    let mut recovering_runtime = SignalRuntime::new(RuntimeConfig::server(48_000, 512));
    recovering_runtime
        .handshake(signal_runtime::HandshakeRequest {
            client_version: "public-host-server-device-supervision-recovering".into(),
            anticipative_preferred: true,
            max_sample_rate_hint: Some(96_000),
        })
        .expect("public server device supervision recovering handshake should succeed");
    recovering_runtime
        .configure(RuntimeConfigRequest::new(48_000, 512))
        .expect("public server device supervision recovering configure should succeed");
    recovering_runtime
        .start()
        .expect("public server device supervision recovering start should succeed");
    recovering_runtime.record_watchdog_restart(WatchdogRestartRecord {
        sandbox_id: "public-host-server-device-supervision-watchdog".into(),
        trigger: RuntimeWatchdogTrigger::HeartbeatMisses,
        processing_epoch: 3,
    });
    let recovering_host = ServerRuntimeHost::new(recovering_runtime);
    let recovering = recovering_host.supervisor_report();
    assert_eq!(
        recovering.observation.device_supervision_snapshot.state,
        RuntimeDeviceSupervisionState::Stable
    );
    assert_eq!(
        recovering
            .observation
            .device_supervision_snapshot
            .restart_state,
        RuntimeDeviceRestartState::Recovered
    );
    assert_eq!(
        recovering
            .observation
            .device_supervision_snapshot
            .fault_boundary,
        RuntimeDeviceFaultBoundaryState::Clear
    );
    assert_eq!(
        recovering
            .observation
            .device_supervision_snapshot
            .interruption_class,
        RuntimeInterruptionClass::Steady
    );

    let mut faulted_runtime = SignalRuntime::new(RuntimeConfig::server(48_000, 512));
    faulted_runtime
        .handshake(signal_runtime::HandshakeRequest {
            client_version: "public-host-server-device-supervision-faulted".into(),
            anticipative_preferred: true,
            max_sample_rate_hint: Some(96_000),
        })
        .expect("public server device supervision faulted handshake should succeed");
    faulted_runtime
        .configure(RuntimeConfigRequest::new(48_000, 512))
        .expect("public server device supervision faulted configure should succeed");
    faulted_runtime
        .start()
        .expect("public server device supervision faulted start should succeed");
    faulted_runtime.fail_runtime(RuntimeError::new(
        RuntimeErrorKind::HardwareFailure,
        "public server host device supervision fault",
    ));
    let faulted_host = ServerRuntimeHost::new(faulted_runtime);
    let faulted = faulted_host.supervisor_report();
    assert_eq!(
        faulted.observation.device_supervision_snapshot.state,
        RuntimeDeviceSupervisionState::Faulted
    );
    assert_eq!(
        faulted
            .observation
            .device_supervision_snapshot
            .restart_state,
        RuntimeDeviceRestartState::Faulted
    );
    assert_eq!(
        faulted
            .observation
            .device_supervision_snapshot
            .fault_boundary,
        RuntimeDeviceFaultBoundaryState::Faulted
    );
    assert_eq!(
        faulted
            .observation
            .device_supervision_snapshot
            .recovery_state,
        RuntimeRecoveryState::Faulted
    );

    let rendered = faulted.render_json();
    assert!(rendered.contains("\"device_supervision_snapshot\":{"));
    assert!(rendered.contains("\"state\":\"Faulted\""));
    assert!(rendered.contains("\"fault_boundary\":\"Faulted\""));
}
