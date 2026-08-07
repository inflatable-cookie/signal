use super::*;

#[test]
fn broker_rejects_unknown_library_extensions_with_typed_detail() {
    let mut client = SandboxBrokerClientSession::spawn_command(
        env!("CARGO_BIN_EXE_signal-plugin-sandbox"),
        &[],
        &SandboxBrokerSpawnConfig::default(),
    )
    .expect("broker child should spawn");
    client
        .read_startup_receipts()
        .expect("startup receipts should arrive");
    let result = client.load_plugin("/tmp/some-plugin.dll", "any-key");
    let error = format!("{:?}", result.expect_err("unknown extension must fail"));
    assert!(
        error.contains("unsupported_library_extension"),
        "typed token expected, got: {error}",
    );
    let _ = client.shutdown();
}
