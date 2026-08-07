use super::*;

#[test]
fn killed_child_bypasses_within_budget_instead_of_hanging() {
    let _slot = sandbox_child_slot();
    if !rustc_available() {
        eprintln!("skipping: rustc unavailable for the CLAP fixture");
        return;
    }
    let directory = unique_fixture_dir();
    let library = compile_clap_fixture(
        &directory.path,
        FIXTURE_PLUGIN_ID,
        "Signal Sandbox Hosting Fixture",
        0,
    )
    .expect("fixture should compile");

    let (mut client, processor) = spawn_processing_session(&library);
    let handle = RenderPluginProcessor::new(Arc::clone(&processor) as Arc<_>);

    // Prove it processes first.
    let mut scratch = vec![0.25f32; 256];
    process_offline(&handle, &mut scratch, 128, 2);

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

    // Host-side death handling marks the handle dead: bypass turns
    // immediate (no budget burned per block).
    processor.mark_dead();
    let start = Instant::now();
    assert!(!handle.process(&mut scratch, 128, 2));
    assert!(start.elapsed() < Duration::from_millis(2));
}
