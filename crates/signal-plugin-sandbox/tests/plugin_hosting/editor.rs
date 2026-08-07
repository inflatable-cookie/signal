use super::*;

#[cfg(target_os = "macos")]
#[test]
fn sandboxed_fixture_editor_opens_over_the_wire_while_audio_stays_byte_exact() {
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
    let frames = 128usize;
    let reference: Vec<f32> = (0..frames * 2).map(|index| index as f32 / 512.0).collect();
    let assert_gain = |handle: &RenderPluginProcessor, gain: f32, label: &str| {
        let mut scratch = reference.clone();
        process_offline(handle, &mut scratch, frames, 2);
        for (index, (output, input)) in scratch.iter().zip(reference.iter()).enumerate() {
            assert!(
                (output - input * gain).abs() < 1e-7,
                "{label} sample {index}: {output} vs {input} * {gain}",
            );
        }
    };

    // Audio first: default fixture gain, byte-exact.
    assert_gain(&handle, CLAP_FIXTURE_GAIN, "pre-open");

    let editor_instance = "instance:sandbox:editor-proof";
    let opened = match client.open_editor(editor_instance) {
        Ok(opened) => opened,
        Err(error) => {
            // No window server (headless run): the wire answered with a
            // typed receipt instead of hanging or killing the child.
            eprintln!("skipping child-window editor proof (no window server?): {error}");
            let _ = client.stop_processing();
            let _ = client.unload_plugin();
            let _ = client.shutdown();
            return;
        }
    };
    assert_eq!(
        (opened.width, opened.height),
        CLAP_FIXTURE_GUI_INITIAL_SIZE,
        "open receipt carries the plugin's initial content size",
    );

    // Audio uninterrupted during open — and the fixture's on-show editor
    // tweak audibly retunes the gain (the editor really showed).
    let shown_gain = CLAP_FIXTURE_GUI_PARAM_OUT_VALUE as f32;
    for block in 0..4 {
        assert_gain(&handle, shown_gain, &format!("open block {block}"));
    }

    // A second open on the same instance refuses with the typed token.
    let error = client
        .open_editor(editor_instance)
        .expect_err("double open must fail typed");
    assert!(
        format!("{error:?}").contains("editor_already_open"),
        "typed token expected: {error:?}",
    );

    // Host-requested close destroys the window; audio keeps flowing.
    let closed = client.close_editor(editor_instance).expect("close receipt");
    assert!(
        closed.closed,
        "close should report host_requested: {closed:?}"
    );
    assert_gain(&handle, shown_gain, "post-close");

    // Reclose is tolerant (`reason=not_open`), and nothing was reported
    // as user-closed.
    let reclosed = client
        .close_editor(editor_instance)
        .expect("reclose receipt");
    assert!(
        !reclosed.closed,
        "reclose should report not_open: {reclosed:?}"
    );
    assert!(client.take_editor_closed_notifications().is_empty());

    // Kill mid-open: the child dies cleanly with its window; the parent
    // reads dead within the bounded budget (crash isolation unchanged).
    client
        .open_editor(editor_instance)
        .expect("editor should reopen before the kill");
    client.kill();
    assert!(!client.is_alive(), "killed child must read as dead");
    let mut scratch = vec![0.25f32; 256];
    let miss_reference = scratch.clone();
    let start = Instant::now();
    assert!(
        !handle.process(&mut scratch, 128, 2),
        "dead child must bypass"
    );
    assert_eq!(
        scratch, miss_reference,
        "bypass must leave scratch untouched"
    );
    assert!(
        start.elapsed() < Duration::from_millis(20),
        "bounded wait overran against a dead child",
    );

    // Respawn: a fresh child processes again and does NOT auto-reopen the
    // editor (explicit reopen is the park-notification idiom).
    let (mut respawned, respawned_processor) = spawn_processing_session(&library);
    let respawned_handle = RenderPluginProcessor::new(Arc::clone(&respawned_processor) as Arc<_>);
    let mut scratch = reference.clone();
    process_offline(&respawned_handle, &mut scratch, frames, 2);
    for (output, input) in scratch.iter().zip(reference.iter()) {
        assert!((output - input * CLAP_FIXTURE_GAIN).abs() < 1e-7);
    }
    assert!(respawned.take_editor_closed_notifications().is_empty());
    let not_reopened = respawned
        .close_editor(editor_instance)
        .expect("close on the respawned child answers");
    assert!(
        !not_reopened.closed,
        "respawn must not auto-reopen the editor: {not_reopened:?}",
    );

    respawned.stop_processing().expect("stop-processing");
    respawned.unload_plugin().expect("unload-plugin");
    respawned.shutdown().expect("shutdown");
}
