use signal_host_local::LocalRuntimeHost;
use signal_runtime::{
    RuntimeConfig, RuntimeLinuxAudioBackendIdentity, RuntimeLinuxBackendDeviceClaimPosture,
    RuntimeLinuxBackendOwnershipFallbackState, RuntimeLinuxBackendSessionLifecycleState,
    RuntimeLinuxBackendSessionOwnership, RuntimeLinuxBackendSessionRole, SignalRuntime,
};

#[test]
fn local_shared_host_edge_exports_runtime_linux_live_ownership_truth() {
    let runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
    let mut host = LocalRuntimeHost::new(runtime);
    host.boot_default()
        .expect("public local linux live ownership default boot should succeed");
    let report = host.host_supervisor_report();

    assert_eq!(
        report
            .observation
            .observation
            .linux_backend_session_snapshot
            .backend_identity,
        RuntimeLinuxAudioBackendIdentity::NotLinux
    );
    assert_eq!(
        report
            .observation
            .observation
            .linux_backend_session_snapshot
            .ownership,
        RuntimeLinuxBackendSessionOwnership::NotLinux
    );
    assert_eq!(
        report
            .observation
            .observation
            .linux_backend_session_snapshot
            .lifecycle_state,
        RuntimeLinuxBackendSessionLifecycleState::NotLinux
    );
    assert_eq!(
        report
            .observation
            .observation
            .linux_backend_session_snapshot
            .device_claim_posture,
        RuntimeLinuxBackendDeviceClaimPosture::NotLinux
    );
    assert_eq!(
        report
            .observation
            .observation
            .linux_backend_session_snapshot
            .session_role,
        RuntimeLinuxBackendSessionRole::NotLinux
    );
    assert_eq!(
        report
            .observation
            .observation
            .linux_backend_session_snapshot
            .ownership_fallback,
        RuntimeLinuxBackendOwnershipFallbackState::NotLinux
    );

    let rendered = report.render_json();
    assert!(rendered.contains("\"linux_backend_session_snapshot\":{"));
    assert!(rendered.contains("\"backend_identity\":\"NotLinux\""));
    assert!(rendered.contains("\"ownership\":\"NotLinux\""));
    assert!(rendered.contains("\"device_claim_posture\":\"NotLinux\""));
}
