use signal_host_server::ServerRuntimeHost;
use signal_runtime::{
    RuntimeConfig, RuntimePipeWireAlsaDeviceClaimParity, RuntimePipeWireAlsaGuardedParityState,
    RuntimePipeWireAlsaSessionRoleParity, RuntimePipeWireAlsaStreamPolicyParity, SignalRuntime,
};

#[test]
fn server_shared_host_edge_exports_runtime_pipewire_alsa_parity_truth() {
    let runtime = SignalRuntime::new(RuntimeConfig::server(48_000, 512));
    let host = ServerRuntimeHost::new(runtime);
    let report = host.supervisor_report();

    assert_eq!(
        report
            .observation
            .pipewire_alsa_parity_snapshot
            .session_role_parity,
        RuntimePipeWireAlsaSessionRoleParity::PrimaryAudioIo
    );
    assert_eq!(
        report
            .observation
            .pipewire_alsa_parity_snapshot
            .device_claim_parity,
        RuntimePipeWireAlsaDeviceClaimParity::SharedGraph
    );
    assert_eq!(
        report
            .observation
            .pipewire_alsa_parity_snapshot
            .stream_policy_parity,
        RuntimePipeWireAlsaStreamPolicyParity::BackendManagedGraph
    );
    assert_eq!(
        report
            .observation
            .pipewire_alsa_parity_snapshot
            .guarded_state,
        RuntimePipeWireAlsaGuardedParityState::ClockGuarded
    );

    let rendered = report.render_json();
    assert!(rendered.contains("\"pipewire_alsa_parity_snapshot\":{"));
    assert!(rendered.contains("\"session_role_parity\":\"PrimaryAudioIo\""));
    assert!(rendered.contains("\"device_claim_parity\":\"SharedGraph\""));
    assert!(rendered.contains("\"stream_policy_parity\":\"BackendManagedGraph\""));
}
