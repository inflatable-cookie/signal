use super::super::*;

#[test]
fn hardware_config_updates_runtime_and_backend_policy() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
    handshake_and_configure(&mut runtime);
    runtime
        .apply_hardware_config(HardwareConfigRequest::new(
            96_000,
            256,
            BackendPolicyTier::Tier1Brokered,
        ))
        .unwrap();

    let config = runtime.get_effective_config();
    assert_eq!(config.sample_rate.0, 96_000);
    assert_eq!(config.block_size, 256);
    assert_eq!(
        runtime.get_diagnostics_snapshot().backend_policy_tier,
        BackendPolicyTier::Tier1Brokered
    );
}
