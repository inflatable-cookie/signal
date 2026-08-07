use super::*;

#[cfg(target_os = "macos")]
#[test]
fn au_child_processes_blocks_and_killed_child_bypasses_within_budget() {
    let _slot = sandbox_child_slot();
    let sentinel = std::path::Path::new(signal_plugin_au::AU_REGISTRY_COMPONENT_PATH);
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
        .load_plugin(&sentinel.display().to_string(), "aufx:dely:appl")
        .expect("stock AUDelay should load in the child");
    assert!(
        !inventory.parameters.is_empty(),
        "AUDelay inventory arrives in the receipt",
    );
    for id in [0u32, 1, 2, 3] {
        assert!(
            inventory
                .parameters
                .iter()
                .any(|parameter| parameter.parameter_id == id),
            "AUDelay parameter id {id} missing from the receipt inventory",
        );
    }
    // Descriptor tokens round-trip the wire (g12.013): AUDelay's real
    // ranges and unit labels arrive from the AudioUnit property API.
    let wet_dry = inventory
        .parameters
        .iter()
        .find(|parameter| parameter.parameter_id == 0)
        .expect("wet/dry mix in the receipt inventory");
    assert_eq!(wet_dry.unit.as_deref(), Some("%"));
    assert!((wet_dry.min_value - 0.0).abs() < 1e-6);
    assert!((wet_dry.max_value - 100.0).abs() < 1e-6);
    assert_eq!(wet_dry.step_count, None);
    assert!(wet_dry.is_automatable);
    assert!(!wet_dry.is_bypass);
    let cutoff = inventory
        .parameters
        .iter()
        .find(|parameter| parameter.parameter_id == 3)
        .expect("lowpass cutoff in the receipt inventory");
    assert_eq!(cutoff.unit.as_deref(), Some("Hz"));

    let lease = match client
        .activate_plugin(SAMPLE_RATE_HZ, 1, MAX_FRAMES)
        .expect("activate should answer")
    {
        SandboxPluginActivateOutcome::Activated(lease) => lease,
        SandboxPluginActivateOutcome::LayoutUnsupported { detail } => {
            panic!("stereo AUDelay rejected: {detail}")
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

    // Deterministic dry-mix proof: constant input, so every output sample
    // must equal input × k for one stable k across all samples and blocks.
    let frames = 128usize;
    let input_level = 0.25f32;
    let mut dry_mix_gain: Option<f32> = None;
    for block in 0..8u32 {
        let mut scratch = vec![input_level; frames * 2];
        process_offline(&handle, &mut scratch, frames, 2);
        let k = dry_mix_gain.get_or_insert(scratch[0] / input_level);
        for (index, sample) in scratch.iter().enumerate() {
            assert!(
                (sample - input_level * *k).abs() < 1e-3,
                "block {block} sample {index}: {sample} vs {input_level} × {k}",
            );
        }
    }
    let k = dry_mix_gain.expect("dry-mix gain measured");
    assert!(
        (0.05..=0.95).contains(&k),
        "AUDelay's default dry mix must attenuate well below identity, saw k = {k}",
    );
    assert!(client.is_alive(), "child should still be alive");

    // Kill the child mid-session (the crash the sandbox tier isolates).
    client.kill();
    assert!(!client.is_alive(), "killed child must read as dead");

    // Un-served requests miss within the bounded budget and leave the
    // scratch untouched — the engine callback would bypass, not block.
    let mut scratch = vec![input_level; frames * 2];
    let reference = scratch.clone();
    let misses_before = processor.miss_count();
    let start = Instant::now();
    let processed = handle.process(&mut scratch, frames, 2);
    let elapsed = start.elapsed();
    assert!(!processed, "dead child must bypass");
    assert_eq!(scratch, reference, "bypass must leave scratch untouched");
    assert!(processor.miss_count() > misses_before);
    assert!(
        elapsed < Duration::from_millis(20),
        "bounded wait overran against a dead child: {elapsed:?}",
    );
}
