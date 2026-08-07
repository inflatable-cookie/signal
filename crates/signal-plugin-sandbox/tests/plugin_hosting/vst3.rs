use super::*;

#[test]
fn vst3_child_processes_blocks_and_killed_child_bypasses_within_budget() {
    let _slot = sandbox_child_slot();
    if !rustc_available() {
        eprintln!("skipping: rustc unavailable for the VST3 fixture");
        return;
    }
    let directory = unique_fixture_dir();
    let bundle = compile_vst3_fixture(
        &directory.path,
        "plugin:vst3:sandbox-hosting-fixture",
        "Signal Sandbox VST3 Fixture",
    )
    .expect("vst3 fixture should compile");

    let (mut client, processor) = spawn_processing_session_for(&bundle, VST3_FIXTURE_CLASS_ID_HEX);
    let handle = RenderPluginProcessor::new(Arc::clone(&processor) as Arc<_>);

    // Round-trip several blocks: output = input × fixture gain, exactly.
    for block in 0..8u32 {
        let frames = 128usize;
        let mut scratch: Vec<f32> = (0..frames * 2)
            .map(|index| (index as f32 + block as f32) / 512.0)
            .collect();
        let reference = scratch.clone();
        process_offline(&handle, &mut scratch, frames, 2);
        for (index, (output, input)) in scratch.iter().zip(reference.iter()).enumerate() {
            assert!(
                (output - input * VST3_FIXTURE_GAIN).abs() < 1e-7,
                "block {block} sample {index}: {output} vs {input} * {VST3_FIXTURE_GAIN}",
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
