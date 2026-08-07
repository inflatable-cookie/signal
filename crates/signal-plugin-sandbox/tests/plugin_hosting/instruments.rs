use super::*;

#[test]
fn real_child_instrument_accepts_zero_input_and_generates_audio_from_note_events() {
    let _slot = sandbox_child_slot();
    if !rustc_available() {
        eprintln!("skipping: rustc unavailable for the CLAP fixture");
        return;
    }
    let directory = unique_fixture_dir();
    let plugin_id = "com.signal.sandbox-instrument-fixture";
    let library =
        compile_clap_instrument_fixture(&directory.path, plugin_id, "Sandbox Instrument Fixture")
            .expect("instrument fixture should compile");
    let (mut client, processor) = spawn_processing_session_for(&library, plugin_id);
    let handle = RenderPluginProcessor::new(Arc::clone(&processor) as Arc<_>);

    let frames = 128usize;
    let mut scratch = vec![0.0; frames * 2];
    process_offline_with_events(
        &handle,
        &mut scratch,
        frames,
        2,
        &[RenderBlockPluginEvent {
            offset_frames: 7,
            channel: 0,
            kind: RenderPluginEventKind::NoteOn {
                key: 60,
                velocity: 0.75,
            },
        }],
    );
    assert!(scratch[..7 * 2].iter().all(|sample| sample.abs() < 1e-6));
    assert!(scratch[7 * 2..].iter().any(|sample| sample.abs() > 0.1));

    client.stop_processing().expect("processing should stop");
    client
        .deactivate_plugin()
        .expect("plugin should deactivate");
    client.unload_plugin().expect("plugin should unload");
    let _ = client.shutdown();
}

#[cfg(target_os = "macos")]
#[test]
fn real_child_system_midi_synth_generates_audio_from_note_events() {
    let _slot = sandbox_child_slot();
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
        .load_plugin("au-registry.component", "aumu:msyn:appl")
        .expect("system MIDI synth should load");
    let lease = match client
        .activate_plugin(SAMPLE_RATE_HZ, 1, MAX_FRAMES)
        .expect("activate should answer")
    {
        SandboxPluginActivateOutcome::Activated(lease) => lease,
        SandboxPluginActivateOutcome::LayoutUnsupported { detail } => {
            panic!("system MIDI synth rejected: {detail}")
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
    let frames = 256usize;
    let mut scratch = vec![0.0; frames * 2];
    process_offline_with_events(
        &handle,
        &mut scratch,
        frames,
        2,
        &[RenderBlockPluginEvent {
            offset_frames: 7,
            channel: 0,
            kind: RenderPluginEventKind::NoteOn {
                key: 60,
                velocity: 0.75,
            },
        }],
    );
    assert!(scratch[7 * 2..].iter().any(|sample| sample.abs() > 1e-5));

    client.stop_processing().expect("processing should stop");
    client
        .deactivate_plugin()
        .expect("plugin should deactivate");
    client.unload_plugin().expect("plugin should unload");
    let _ = client.shutdown();
}
