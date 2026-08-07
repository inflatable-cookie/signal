use super::*;

#[test]
fn lv2_child_processes_blocks_and_killed_child_bypasses_within_budget() {
    let _slot = sandbox_child_slot();
    if !rustc_available() {
        eprintln!("skipping: rustc unavailable for the LV2 fixture");
        return;
    }
    let directory = unique_fixture_dir();
    let plugin_uri = "https://signal.dev/fixtures/lv2/sandbox-hosting";
    let bundle = compile_lv2_fixture(&directory.path, plugin_uri, "Signal Sandbox LV2 Fixture")
        .expect("lv2 fixture should compile");

    let mut client = SandboxBrokerClientSession::spawn_command(
        env!("CARGO_BIN_EXE_signal-plugin-sandbox"),
        &[],
        &SandboxBrokerSpawnConfig::default(),
    )
    .expect("broker child should spawn");
    client
        .read_startup_receipts()
        .expect("startup receipts should arrive");

    let inventory = client
        .load_plugin(&bundle.display().to_string(), plugin_uri)
        .expect("lv2 fixture should load in the child");
    assert_eq!(
        inventory.parameters.len(),
        2,
        "TTL control ports arrive as parameters",
    );
    let gain = inventory
        .parameters
        .iter()
        .find(|parameter| parameter.name == "Gain")
        .expect("fixture Gain param in the inventory");
    assert_eq!(
        gain.parameter_id, LV2_FIXTURE_GAIN_PORT_INDEX,
        "LV2 parameter_id = control port index",
    );
    assert!((gain.min_value - 0.0).abs() < 1e-6);
    assert!((gain.max_value - 1.0).abs() < 1e-6);
    assert!((gain.default_value - LV2_FIXTURE_GAIN).abs() < 1e-6);
    // Descriptor tokens round-trip the wire (g12.013): units:unit from the
    // TTL; toggled + designation lv2:enabled mark the bypass toggle.
    assert_eq!(gain.unit.as_deref(), Some("coef"));
    assert_eq!(gain.step_count, None);
    assert!(gain.is_automatable);
    assert!(!gain.is_bypass);
    let bypass = inventory
        .parameters
        .iter()
        .find(|parameter| parameter.name == "Bypass")
        .expect("fixture Bypass param in the inventory");
    assert_eq!(bypass.step_count, Some(1));
    assert!(bypass.is_bypass);
    assert_eq!(bypass.unit, None);

    let lease = match client
        .activate_plugin(SAMPLE_RATE_HZ, 1, MAX_FRAMES)
        .expect("activate should answer")
    {
        SandboxPluginActivateOutcome::Activated(lease) => lease,
        SandboxPluginActivateOutcome::LayoutUnsupported { detail } => {
            panic!("stereo lv2 fixture rejected: {detail}")
        }
    };
    assert_eq!(lease.max_frames, MAX_FRAMES);
    assert_eq!(lease.channels, 2);
    client
        .start_processing()
        .expect("child audio thread should start");
    let processor = Arc::new(
        ShmPluginProcessor::attach(
            &lease.region_id,
            &lease.shm_path,
            lease.shm_bytes,
            lease.max_frames,
            lease.channels,
            SAMPLE_RATE_HZ,
        )
        .expect("parent should attach the audio block region"),
    );
    let handle = RenderPluginProcessor::new(Arc::clone(&processor) as Arc<_>);

    // Round-trip several blocks: output = input × the TTL default gain,
    // exactly (our own fixture math).
    for block in 0..8u32 {
        let frames = 128usize;
        let mut scratch: Vec<f32> = (0..frames * 2)
            .map(|index| (index as f32 + block as f32) / 512.0)
            .collect();
        let reference = scratch.clone();
        process_offline(&handle, &mut scratch, frames, 2);
        for (index, (output, input)) in scratch.iter().zip(reference.iter()).enumerate() {
            assert!(
                (output - input * LV2_FIXTURE_GAIN).abs() < 1e-7,
                "block {block} sample {index}: {output} vs {input} * {LV2_FIXTURE_GAIN}",
            );
        }
    }
    assert!(client.is_alive(), "child should still be alive");

    // Kill the child mid-session (the crash the sandbox tier isolates).
    client.kill();
    assert!(!client.is_alive(), "killed child must read as dead");

    // Un-served requests miss within the bounded budget and leave the
    // scratch untouched — the engine callback would bypass, not block.
    let mut scratch = vec![0.25f32; 256];
    let reference = scratch.clone();
    let misses_before = processor.miss_count();
    let start = Instant::now();
    let processed = handle.process(&mut scratch, 128, 2);
    let elapsed = start.elapsed();
    assert!(!processed, "dead child must bypass");
    assert_eq!(scratch, reference, "bypass must leave scratch untouched");
    assert!(processor.miss_count() > misses_before);
    assert!(
        elapsed < Duration::from_millis(20),
        "bounded wait overran against a dead child: {elapsed:?}",
    );
}
