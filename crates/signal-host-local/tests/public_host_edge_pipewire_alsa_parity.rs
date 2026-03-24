use signal_host_local::LocalRuntimeHost;
use signal_runtime::{
    RuntimeConfig, RuntimePipeWireAlsaDeviceClaimParity, RuntimePipeWireAlsaGuardedParityState,
    RuntimePipeWireAlsaSessionRoleParity, RuntimePipeWireAlsaStreamPolicyParity, SignalRuntime,
};

#[test]
fn local_shared_host_edge_exports_runtime_pipewire_alsa_parity_truth() {
    let runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
    let mut host = LocalRuntimeHost::new(runtime);
    host.boot_default()
        .expect("public local pipewire/alsa parity default boot should succeed");
    let report = host.supervisor_report();

    assert_eq!(
        report
            .observation
            .pipewire_alsa_parity_snapshot
            .session_role_parity,
        RuntimePipeWireAlsaSessionRoleParity::NotPipeWireOrAlsa
    );
    assert_eq!(
        report
            .observation
            .pipewire_alsa_parity_snapshot
            .device_claim_parity,
        RuntimePipeWireAlsaDeviceClaimParity::NotPipeWireOrAlsa
    );
    assert_eq!(
        report
            .observation
            .pipewire_alsa_parity_snapshot
            .stream_policy_parity,
        RuntimePipeWireAlsaStreamPolicyParity::NotPipeWireOrAlsa
    );
    assert_eq!(
        report
            .observation
            .pipewire_alsa_parity_snapshot
            .guarded_state,
        RuntimePipeWireAlsaGuardedParityState::NotPipeWireOrAlsa
    );

    let rendered = report.render_json();
    assert!(rendered.contains("\"pipewire_alsa_parity_snapshot\":{"));
    assert!(rendered.contains("\"session_role_parity\":\"NotPipeWireOrAlsa\""));
    assert!(rendered.contains("\"device_claim_parity\":\"NotPipeWireOrAlsa\""));
}
