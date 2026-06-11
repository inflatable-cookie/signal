use super::super::*;

#[test]
fn runtime_starts_and_reports_ready() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
    handshake_and_configure(&mut runtime);
    runtime.start().unwrap();

    assert_eq!(runtime.get_readiness(), RuntimeReadiness::Ready);
    assert_eq!(runtime.config().profile, RuntimeProfile::Local);
}

#[test]
fn configure_updates_effective_config() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::server(48_000, 512));
    runtime
        .handshake(HandshakeRequest {
            client_version: "runtime-test".into(),
            anticipative_preferred: true,
            max_sample_rate_hint: Some(96_000),
        })
        .unwrap();
    runtime
        .configure(RuntimeConfigRequest::new(96_000, 256))
        .unwrap();

    let config = runtime.get_effective_config();
    assert_eq!(config.sample_rate.0, 96_000);
    assert_eq!(config.block_size, 256);
}

#[test]
fn configure_resets_runtime_block_timeline() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
    runtime
        .handshake(HandshakeRequest {
            client_version: "runtime-test".into(),
            anticipative_preferred: true,
            max_sample_rate_hint: Some(96_000),
        })
        .unwrap();
    let first_sequence = runtime.allocate_block_sequence();
    runtime.record_block_sequence("sandbox-a", 1, "lease-a", first_sequence);

    runtime
        .configure(RuntimeConfigRequest::new(48_000, 256))
        .unwrap();

    let timeline = runtime.get_timeline_snapshot();
    assert_eq!(timeline.next_block_sequence, 0);
}

#[test]
fn runtime_timeline_tracks_sequences_across_leases() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
    let first = runtime.allocate_block_sequence();
    runtime.record_block_sequence("sandbox-a", 1, "lease-a", first);
    let second = runtime.allocate_block_sequence();
    runtime.record_block_sequence("sandbox-a", 1, "lease-a", second);
    let third = runtime.allocate_block_sequence();
    runtime.record_block_sequence("sandbox-a", 2, "lease-b", third);

    let timeline = runtime.get_timeline_snapshot();
    assert_eq!(timeline.next_block_sequence, 3);
}

