use super::*;

#[test]
fn real_child_processes_blocks_through_the_shm_bridge() {
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
                (output - input * CLAP_FIXTURE_GAIN).abs() < 1e-7,
                "block {block} sample {index}: {output} vs {input} * {CLAP_FIXTURE_GAIN}",
            );
        }
    }
    assert!(client.is_alive(), "child should still be alive");

    // Orderly teardown: stop, deactivate (destroys the region), unload.
    client.stop_processing().expect("stop-processing");
    client.deactivate_plugin().expect("deactivate");
    client.unload_plugin().expect("unload-plugin");
    client.shutdown().expect("shutdown");
}
