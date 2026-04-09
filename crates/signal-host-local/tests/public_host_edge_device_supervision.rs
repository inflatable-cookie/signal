#[path = "support/public_host_edge_plugins.rs"]
mod public_host_edge_plugins_support;

use public_host_edge_plugins_support::{temp_public_local_au_scan_root, DemoPluginEnvGuard};
use signal_host_local::LocalRuntimeHost;
use signal_runtime::{
    RuntimeConfig, RuntimeConfigRequest, RuntimeDeviceFaultBoundaryState,
    RuntimeDeviceRestartState, RuntimeDeviceSupervisionState, RuntimeError, RuntimeErrorKind,
    RuntimeLifecycleApi, RuntimeRecoveryState, SignalRuntime,
};

#[test]
fn local_shared_host_edge_exports_runtime_device_supervision_truth() {
    let scan_root = temp_public_local_au_scan_root();
    let _guard = DemoPluginEnvGuard::enable_au(&scan_root, "plugin:au:instrument");
    let recovering_runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
    let mut recovering_host = LocalRuntimeHost::new(recovering_runtime);
    recovering_host
        .boot_with_device_loss_recovery()
        .expect("public local device supervision recovery should succeed");
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
            .device_loss_count,
        1
    );

    let exhausted_runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
    let mut exhausted_host = LocalRuntimeHost::new(exhausted_runtime);
    let error = exhausted_host
        .boot_with_device_loss_restart_failure()
        .expect_err("public local device supervision restart failure should fail");
    assert_eq!(error.kind, RuntimeErrorKind::HardwareFailure);
    let exhausted = exhausted_host.supervisor_report();
    assert_eq!(
        exhausted.observation.device_supervision_snapshot.state,
        RuntimeDeviceSupervisionState::Exhausted
    );
    assert_eq!(
        exhausted
            .observation
            .device_supervision_snapshot
            .restart_state,
        RuntimeDeviceRestartState::Exhausted
    );
    assert_eq!(
        exhausted
            .observation
            .device_supervision_snapshot
            .fault_boundary,
        RuntimeDeviceFaultBoundaryState::Exhausted
    );
    assert_eq!(
        exhausted
            .observation
            .device_supervision_snapshot
            .restart_failure_count,
        Some(1)
    );

    let mut faulted_runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
    faulted_runtime
        .handshake(signal_runtime::HandshakeRequest {
            client_version: "public-host-local-device-supervision-faulted".into(),
            anticipative_preferred: true,
            max_sample_rate_hint: Some(96_000),
        })
        .expect("public local device supervision handshake should succeed");
    faulted_runtime
        .configure(RuntimeConfigRequest::new(48_000, 512))
        .expect("public local device supervision configure should succeed");
    faulted_runtime
        .start()
        .expect("public local device supervision start should succeed");
    faulted_runtime.fail_runtime(RuntimeError::new(
        RuntimeErrorKind::HardwareFailure,
        "public local host device supervision fault",
    ));
    let faulted_host = LocalRuntimeHost::new(faulted_runtime);
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
