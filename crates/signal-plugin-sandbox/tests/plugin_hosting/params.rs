use super::*;

#[test]
fn param_set_over_the_wire_changes_the_sandboxed_output_next_block() {
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

    // Default gain first.
    let frames = 128usize;
    let reference: Vec<f32> = (0..frames * 2).map(|index| index as f32 / 512.0).collect();
    let mut scratch = reference.clone();
    process_offline(&handle, &mut scratch, frames, 2);
    for (output, input) in scratch.iter().zip(reference.iter()) {
        assert!((output - input * CLAP_FIXTURE_GAIN).abs() < 1e-7);
    }

    // Single wire set: applied by the child's audio thread next block.
    let detail = client
        .set_parameter(CLAP_FIXTURE_GAIN_PARAM_ID, 0.25)
        .expect("set-param receipt");
    assert!(detail.contains("param_set"), "typed receipt: {detail}");
    let mut scratch = reference.clone();
    process_offline(&handle, &mut scratch, frames, 2);
    for (index, (output, input)) in scratch.iter().zip(reference.iter()).enumerate() {
        assert!(
            (output - input * 0.25).abs() < 1e-7,
            "sample {index}: {output} vs {input} * 0.25",
        );
    }

    // Batched sweep: 100 coalescing writes, one receipt, final value wins.
    let sweep: Vec<(u32, f32)> = (0..100)
        .map(|step| (CLAP_FIXTURE_GAIN_PARAM_ID, step as f32 / 99.0))
        .collect();
    let detail = client.set_parameters(&sweep).expect("set-params receipt");
    assert!(detail.contains("count=100"), "batched receipt: {detail}");
    let mut scratch = reference.clone();
    process_offline(&handle, &mut scratch, frames, 2);
    for (index, (output, input)) in scratch.iter().zip(reference.iter()).enumerate() {
        assert!(
            (output - input).abs() < 1e-7,
            "sample {index}: {output} vs {input} (sweep ends at unity)",
        );
    }

    // Unknown parameter ids fail typed without killing the session.
    let error = client
        .set_parameter(9999, 0.5)
        .expect_err("unknown parameter must fail");
    assert!(
        format!("{error:?}").contains("unknown_parameter"),
        "typed token expected: {error:?}",
    );

    client.stop_processing().expect("stop-processing");
    client.unload_plugin().expect("unload-plugin");
    client.shutdown().expect("shutdown");
}

/// g12.023: the owed AU TRUE-IDENTITY proof (deferred from g11.032),
/// through the real wire — driving the stock AUDelay's WetDryMix to fully
/// dry over `set-param` makes the sandboxed unit render identity, where
/// its defaults audibly attenuated. Closes the loop the g11.032 broker
/// test could only approximate (no set path existed).
#[cfg(target_os = "macos")]
#[test]
fn au_wire_param_set_drives_audelay_to_true_identity() {
    let _slot = sandbox_child_slot();
    const AUDELAY_WET_DRY_MIX: u32 = 0;
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
    client
        .load_plugin(&sentinel.display().to_string(), "aufx:dely:appl")
        .expect("stock AUDelay should load in the child");
    let lease = match client
        .activate_plugin(SAMPLE_RATE_HZ, 1, MAX_FRAMES)
        .expect("activate should answer")
    {
        SandboxPluginActivateOutcome::Activated(lease) => lease,
        SandboxPluginActivateOutcome::LayoutUnsupported { detail } => {
            panic!("stereo AUDelay rejected: {detail}")
        }
    };
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

    // Defaults attenuate (WetDryMix 50%, silent first-second delay line).
    let frames = 128usize;
    let input_level = 0.25f32;
    let mut scratch = vec![input_level; frames * 2];
    process_offline(&handle, &mut scratch, frames, 2);
    let default_gain = scratch[0] / input_level;
    assert!(
        default_gain < 0.95,
        "AUDelay defaults must attenuate below identity, saw {default_gain}",
    );

    // Fully dry over the wire: WetDryMix plain range is 0..100, so
    // normalized 0.0 = plain 0 % wet.
    client
        .set_parameter(AUDELAY_WET_DRY_MIX, 0.0)
        .expect("set-param receipt");

    // The unit renders identity from the next pulled block on.
    let mut scratch = vec![input_level; frames * 2];
    process_offline(&handle, &mut scratch, frames, 2);
    for (index, sample) in scratch.iter().enumerate() {
        assert!(
            (sample - input_level).abs() <= 1e-3,
            "sample {index}: {sample} vs {input_level} (identity after full-dry set)",
        );
    }

    client.stop_processing().expect("stop-processing");
    client.unload_plugin().expect("unload-plugin");
    client.shutdown().expect("shutdown");
}
