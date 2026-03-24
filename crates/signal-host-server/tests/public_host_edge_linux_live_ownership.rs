use signal_host_server::ServerRuntimeHost;
use signal_runtime::{
    RuntimeConfig, RuntimeLinuxAudioBackendIdentity, RuntimeLinuxBackendDeviceClaimPosture,
    RuntimeLinuxBackendOwnershipFallbackState, RuntimeLinuxBackendSessionLifecycleState,
    RuntimeLinuxBackendSessionOwnership, RuntimeLinuxBackendSessionRole, SignalRuntime,
};

#[test]
fn server_shared_host_edge_exports_runtime_linux_live_ownership_truth() {
    let runtime = SignalRuntime::new(RuntimeConfig::server(48_000, 512));
    let host = ServerRuntimeHost::new(runtime);
    let report = host.supervisor_report();

    assert_eq!(
        report
            .observation
            .linux_backend_session_snapshot
            .backend_identity,
        RuntimeLinuxAudioBackendIdentity::PipeWire
    );
    assert_eq!(
        report.observation.linux_backend_session_snapshot.ownership,
        RuntimeLinuxBackendSessionOwnership::BackendManagedGraph
    );
    assert_eq!(
        report
            .observation
            .linux_backend_session_snapshot
            .lifecycle_state,
        RuntimeLinuxBackendSessionLifecycleState::Running
    );
    assert_eq!(
        report
            .observation
            .linux_backend_session_snapshot
            .device_claim_posture,
        RuntimeLinuxBackendDeviceClaimPosture::SharedGraph
    );
    assert_eq!(
        report
            .observation
            .linux_backend_session_snapshot
            .session_role,
        RuntimeLinuxBackendSessionRole::PrimaryAudioIo
    );
    assert_eq!(
        report
            .observation
            .linux_backend_session_snapshot
            .ownership_fallback,
        RuntimeLinuxBackendOwnershipFallbackState::BackendManagedGuarded
    );

    let rendered = report.render_json();
    assert!(rendered.contains("\"linux_backend_session_snapshot\":{"));
    assert!(rendered.contains("\"backend_identity\":\"PipeWire\""));
    assert!(rendered.contains("\"ownership\":\"BackendManagedGraph\""));
    assert!(rendered.contains("\"session_role\":\"PrimaryAudioIo\""));
}
