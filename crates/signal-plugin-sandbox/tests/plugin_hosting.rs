//! Full plugin-hosting round trip against a REAL sandbox child process
//! (g11.012): rustc-compiled CLAP fixture → broker load/activate/
//! start-processing → shared-memory blocks → fixture `process()` (fixed
//! gain) → parent reads the processed output. Plus the e2e proof: the
//! fixture audibly processes a chain insert through the real engine's
//! offline render (render-differencing — the gain plugin halves the mix),
//! and crash isolation: a killed child reads as bypass, never as a hang.
//!
//! Skips gracefully when `rustc` is unavailable (the existing fixture
//! pattern).

use std::sync::Arc;
use std::time::{Duration, Instant};

use signal_plugin_bridge::ShmPluginProcessor;
use signal_plugin_clap::fixture::{
    compile_clap_fixture, compile_clap_instrument_fixture, rustc_available, CLAP_FIXTURE_GAIN,
    CLAP_FIXTURE_GAIN_PARAM_ID, CLAP_FIXTURE_GUI_INITIAL_SIZE, CLAP_FIXTURE_GUI_PARAM_OUT_VALUE,
};
use signal_plugin_lv2::fixture::{
    compile_lv2_fixture, LV2_FIXTURE_GAIN, LV2_FIXTURE_GAIN_PORT_INDEX,
};
use signal_plugin_vst3::fixture::{
    compile_vst3_fixture, VST3_FIXTURE_CLASS_ID_HEX, VST3_FIXTURE_GAIN,
};
use signal_render_plane::{
    render_plan_to_pcm, ChannelFormat, OfflineRenderOptions, RenderBlockPluginEvent,
    RenderClipSpec, RenderEdgeSpec, RenderPlanSpec, RenderPluginEventKind, RenderPluginProcessor,
    RenderSampleBuffer, RenderSource, RenderStageKind, RenderStageSpec,
};
use signal_runtime::{
    SandboxBrokerClientSession, SandboxBrokerSpawnConfig, SandboxPluginActivateOutcome,
};

const FIXTURE_PLUGIN_ID: &str = "com.signal.sandbox-hosting-fixture";
const SAMPLE_RATE_HZ: u32 = 48_000;
const MAX_FRAMES: u32 = 256;

struct FixtureDir {
    path: std::path::PathBuf,
}

impl Drop for FixtureDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.path);
    }
}

fn unique_fixture_dir() -> FixtureDir {
    static SEQUENCE: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let sequence = SEQUENCE.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    FixtureDir {
        path: std::env::temp_dir().join(format!(
            "signal-sandbox-hosting-{}-{}-{sequence}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos(),
        )),
    }
}

/// Spawn the real broker binary, walk it through load → activate →
/// start-processing for a fixture (`load_key` is the format-native key:
/// CLAP plugin id or VST3 class CID hex; the broker picks the format from
/// the library path extension), and return the session plus the attached
/// parent-side processor.
fn spawn_processing_session_for(
    library_path: &std::path::Path,
    load_key: &str,
) -> (SandboxBrokerClientSession, Arc<ShmPluginProcessor>) {
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
        .load_plugin(&library_path.display().to_string(), load_key)
        .expect("fixture should load in the child");
    assert_eq!(inventory.parameters.len(), 2, "fixture exposes two params");
    let gain = inventory
        .parameters
        .iter()
        .find(|parameter| parameter.name == "Gain")
        .expect("fixture Gain param in the inventory");
    assert_eq!(gain.parameter_id, 4096);
    assert!((gain.min_value - 0.0).abs() < 1e-6);
    assert!((gain.max_value - 1.0).abs() < 1e-6);
    assert!((0.0..=1.0).contains(&gain.default_value));
    // Descriptor tokens round-trip the wire (g12.013): the fixture Gain is
    // continuous and automatable; the fixture Bypass is a one-step
    // automatable bypass toggle (identical in the CLAP and VST3 fixtures).
    assert_eq!(gain.step_count, None);
    assert!(gain.is_automatable);
    assert!(!gain.is_bypass);
    // Unit strings are per-format truth: the VST3 fixture labels Gain
    // "dB" via ParameterInfo.units; CLAP param info has no unit field.
    let is_vst3 = library_path.extension().and_then(|e| e.to_str()) == Some("vst3");
    if is_vst3 {
        assert_eq!(gain.unit.as_deref(), Some("dB"));
    } else {
        assert_eq!(gain.unit, None);
    }
    let bypass = inventory
        .parameters
        .iter()
        .find(|parameter| parameter.name == "Bypass")
        .expect("fixture Bypass param in the inventory");
    assert_eq!(bypass.step_count, Some(1));
    assert!(bypass.is_automatable);
    assert!(bypass.is_bypass);
    assert_eq!(bypass.unit, None);

    let lease = match client
        .activate_plugin(SAMPLE_RATE_HZ, 1, MAX_FRAMES)
        .expect("activate should answer")
    {
        SandboxPluginActivateOutcome::Activated(lease) => lease,
        SandboxPluginActivateOutcome::LayoutUnsupported { detail } => {
            panic!("stereo fixture rejected: {detail}")
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
    (client, processor)
}

/// CLAP-fixture convenience wrapper around [`spawn_processing_session_for`].
fn spawn_processing_session(
    library_path: &std::path::Path,
) -> (SandboxBrokerClientSession, Arc<ShmPluginProcessor>) {
    spawn_processing_session_for(library_path, FIXTURE_PLUGIN_ID)
}

/// Drive one block through the handle, retrying misses (the child audio
/// thread may need a scheduler quantum to see the first request).
///
/// The deadline is generous because the first request waits on a real process
/// spawn plus a plugin `dlopen`, which on cold shared CI infrastructure
/// legitimately takes seconds. It exists to catch a child that never answers,
/// not to bound how quickly it answers. Per-block latency is asserted
/// separately by the bypass-budget tests, and those bounds stay tight.
///
/// KNOWN LIMIT: retrying cannot recover from a retired epoch. After
/// `PLUGIN_PROCESS_CONSECUTIVE_TIMEOUT_LIMIT` consecutive misses the processor
/// clears its `alive` flag and every later `process` returns false
/// immediately, so once that happens this loop is futile and only burns the
/// deadline. Serialising the child-spawning tests is what keeps the misses
/// from stacking up; the deadline is short enough that a retired epoch fails
/// in tens of seconds rather than a minute.
///
/// MEASURED 2026-08-05, and the condition matters more than the rate. Under
/// concurrent load — a release gate running in another process — this binary
/// failed `2` runs in `20`, across
/// `fixture_plugin_processes_a_chain_insert_through_the_real_engine_offline_render`
/// and `real_child_instrument_accepts_zero_input_and_generates_audio_from_note_events`.
/// On an idle machine it passed `20` consecutive runs.
///
/// So it is load-dependent, in the `A20`/`A22` family, rather than a fixed rate.
/// Serialising the child-spawning tests earlier the same day cut it without
/// removing it. Two distinct failure modes were seen: a session that fails
/// during spawn, and a render where wet equals dry because a missed response
/// bypassed — the latter being what epoch retirement looks like from outside.
///
/// If this recurs, the fix is the one applied to the `shm` round-trip test:
/// re-attach the processor when `is_alive()` goes false and give the round
/// trip a fresh epoch. It is not done here because the twelve call sites each
/// hold their own handle and the lease needed to re-attach is not threaded
/// through them.
const CHILD_FIRST_RESPONSE_DEADLINE: Duration = Duration::from_secs(20);

fn process_with_retries(handle: &RenderPluginProcessor, scratch: &mut [f32], frames: usize) {
    let deadline = Instant::now() + CHILD_FIRST_RESPONSE_DEADLINE;
    loop {
        if handle.process(scratch, frames, 2) {
            return;
        }
        assert!(
            Instant::now() < deadline,
            "child never answered a process request within \
             {CHILD_FIRST_RESPONSE_DEADLINE:?} (a retired epoch cannot recover \
             by retrying -- see CHILD_FIRST_RESPONSE_DEADLINE)",
        );
        std::thread::yield_now();
    }
}

/// Serialises the tests that spawn a sandbox child process.
///
/// Each child runs a hot-spinning audio thread — `broker.rs` spins with a
/// periodic `yield_now` — and this binary holds twelve such tests. Run in
/// parallel on a machine with few cores, twelve spinning children plus twelve
/// spinning parents starve each other, and the children miss their response
/// budget. Locally that showed up as timing flakes; on a GitHub runner with
/// three or four cores it failed outright, including a child that never
/// answered within `60s`.
///
/// One at a time. These tests take a second or two each, so the wall-clock
/// cost is small next to the failure mode it removes.
fn sandbox_child_slot() -> std::sync::MutexGuard<'static, ()> {
    static SLOT: std::sync::Mutex<()> = std::sync::Mutex::new(());
    // A panicking test poisons the mutex; the next test should still run
    // rather than cascade into a second failure with an unrelated message.
    SLOT.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
}

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
        process_with_retries(&handle, &mut scratch, frames);
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
    assert!(handle.process_with_events(
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
    ));
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
    assert!(handle.process_with_events(
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
    ));
    assert!(scratch[7 * 2..].iter().any(|sample| sample.abs() > 1e-5));

    client.stop_processing().expect("processing should stop");
    client
        .deactivate_plugin()
        .expect("plugin should deactivate");
    client.unload_plugin().expect("plugin should unload");
    let _ = client.shutdown();
}

/// g12.023: the sandboxed param-set round trip — a `set-param` stdio
/// command reaches the child's audio thread and changes the processed
/// output at the next shared-memory block, byte-exact; a batched
/// `set-params` (fader-style sweep, last write wins) lands its final
/// value the same way.
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
    process_with_retries(&handle, &mut scratch, frames);
    for (output, input) in scratch.iter().zip(reference.iter()) {
        assert!((output - input * CLAP_FIXTURE_GAIN).abs() < 1e-7);
    }

    // Single wire set: applied by the child's audio thread next block.
    let detail = client
        .set_parameter(CLAP_FIXTURE_GAIN_PARAM_ID, 0.25)
        .expect("set-param receipt");
    assert!(detail.contains("param_set"), "typed receipt: {detail}");
    let mut scratch = reference.clone();
    process_with_retries(&handle, &mut scratch, frames);
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
    process_with_retries(&handle, &mut scratch, frames);
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
    process_with_retries(&handle, &mut scratch, frames);
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
    process_with_retries(&handle, &mut scratch, frames);
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

/// g13.027 Batch 1: the sandboxed CLAP fixture opens its editor in a
/// CHILD-OWNED floating window over the new `open-editor` wire verb while
/// shared-memory audio round-trips stay byte-exact (the RT isolation
/// invariant — the child's audio thread never touches AppKit); the
/// fixture's on-show "editor tweak" audibly retunes the processed gain
/// (proof the editor really showed); `close-editor` destroys the window
/// (tolerant of a reclose); kill mid-open reads as dead within the
/// bounded budget; a respawned child does NOT auto-reopen the editor.
///
/// Skips (loudly) when the child cannot reach a window server — child
/// window creation needs a GUI login session; the wire still answers with
/// typed receipts in that case, which is what the skip asserts.
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
        process_with_retries(handle, &mut scratch, frames);
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
    process_with_retries(&respawned_handle, &mut scratch, frames);
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
    process_with_retries(&handle, &mut scratch, 128);

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

/// The VST3 mirror of the CLAP e2e (g11.031): rustc-compiled VST3 bundle →
/// broker load (format picked from the `.vst3` extension, load key = class
/// CID hex) → parameter receipt via IEditController → activate/shm lease →
/// start-processing → wet = dry × gain byte-verified → kill child → bypass
/// within the bounded budget.
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
        process_with_retries(&handle, &mut scratch, frames);
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

/// The LV2 mirror of the CLAP/VST3 e2e (g11.033): rustc-compiled `.lv2`
/// bundle (real manifest.ttl + plugin TTL + cdylib) → broker load (format
/// picked from the `.lv2` DIRECTORY extension, load key = the bare plugin
/// URI; the child re-parses the TTL for the port model) → parameter
/// receipt from the TTL control ports → activate/shm lease → start →
/// wet = dry × the Gain port's non-unity TTL default, byte-verified
/// through shm (no param set exists phase 1) → kill child → bypass within
/// the bounded budget, scratch untouched.
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
        process_with_retries(&handle, &mut scratch, frames);
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

/// The AU mirror of the VST3 broker e2e (g11.032), against the stock Apple
/// AUDelay — no compiled fixture (the AudioComponent registrar cannot see
/// temp bundles): broker load (format picked from the `.component`
/// extension, sentinel path never opened, load key = fourcc triple) →
/// parameter receipt via the AudioUnit property API → activate/shm lease →
/// start-processing → deterministic processing proof through shm → kill
/// child → bypass within the bounded budget.
///
/// Processing proof: the v1 wire has no set-parameter command, so the child
/// runs AUDelay at its defaults (WetDryMix 50%, DelayTime 1 s). For the
/// first second the delay line is silent, so every output sample is the
/// input scaled by one constant dry-mix gain k with k well below identity —
/// which proves the unit really rendered the block (a passthrough bridge
/// bug would read k = 1).
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
        process_with_retries(&handle, &mut scratch, frames);
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

/// The broker rejects libraries whose extension names no hosted format.
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

/// The e2e proof: the fixture audibly processes a chain insert through the
/// REAL engine offline render. Render-differencing: the same plan without
/// the processor renders the dry mix; with it, every in-window sample is
/// halved by the fixture's gain.
#[test]
fn fixture_plugin_processes_a_chain_insert_through_the_real_engine_offline_render() {
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

    // Constant-content source lane so the differencing is exact.
    let mut data = Vec::new();
    for _ in 0..SAMPLE_RATE_HZ / 2 {
        data.push(0.5f32);
        data.push(0.5f32);
    }
    let buffer = RenderSampleBuffer::stereo(SAMPLE_RATE_HZ, data.into());
    let plan = |processor: Option<RenderPluginProcessor>| RenderPlanSpec {
        sample_rate_hz: SAMPLE_RATE_HZ,
        master_gain: 1.0,
        master_limiter: None,
        stages: vec![
            RenderStageSpec {
                parameter_envelopes: Vec::new(),
                accepts_live_events: false,
                processor: None,
                events: None,
                stage_id: 1,
                format: ChannelFormat::stereo(),
                gain: 1.0,
                gain_automation: None,
                kind: RenderStageKind::Source {
                    clips: vec![RenderClipSpec {
                        clip_id: 11,
                        start_frames: 0,
                        end_frames: u64::MAX,
                        source: RenderSource::Samples(buffer.clone()),
                        loop_source: true,
                        fade_in_frames: 0,
                        fade_out_frames: 0,
                    }],
                },
                inputs: Vec::new(),
            },
            RenderStageSpec {
                parameter_envelopes: Vec::new(),
                accepts_live_events: false,
                processor,
                events: None,
                stage_id: 2,
                format: ChannelFormat::stereo(),
                gain: 1.0,
                gain_automation: None,
                kind: RenderStageKind::Sum,
                inputs: vec![RenderEdgeSpec {
                    source_stage_id: 1,
                    gain: 1.0,
                    matrix: None,
                }],
            },
            RenderStageSpec {
                parameter_envelopes: Vec::new(),
                accepts_live_events: false,
                processor: None,
                events: None,
                stage_id: 100,
                format: ChannelFormat::stereo(),
                gain: 1.0,
                gain_automation: None,
                kind: RenderStageKind::Output,
                inputs: vec![RenderEdgeSpec {
                    source_stage_id: 2,
                    gain: 1.0,
                    matrix: None,
                }],
            },
        ],
    };
    let options = OfflineRenderOptions {
        start_frame: 0,
        frame_count: 4_800,
        block_frames: 128,
        capture_stage_ids: Vec::new(),
    };

    // Warm the child's audio thread so the offline render (faster than
    // realtime, no retries) never races its first block.
    let mut warm = vec![0.0f32; 256];
    process_with_retries(&handle, &mut warm, 128);

    let dry = render_plan_to_pcm(&plan(None), &options).expect("dry render");
    let wet = render_plan_to_pcm(&plan(Some(handle)), &options).expect("wet render");
    assert_eq!(dry.master.len(), wet.master.len());

    // Render-differencing: skip the clip edge fade, then every sample must
    // be dry × fixture gain — the insert audibly halves the mix.
    let fade_guard = 64 * 2;
    let mut checked = 0usize;
    for (index, (dry_sample, wet_sample)) in dry
        .master
        .iter()
        .zip(wet.master.iter())
        .enumerate()
        .skip(fade_guard)
    {
        assert!(
            (wet_sample - dry_sample * CLAP_FIXTURE_GAIN).abs() < 1e-6,
            "sample {index}: wet {wet_sample} vs dry {dry_sample} * {CLAP_FIXTURE_GAIN}",
        );
        checked += 1;
    }
    assert!(checked > 8_000, "differencing covered the render");
    // The dry mix itself was audible (the test has teeth).
    assert!(dry.master[fade_guard].abs() > 0.4);

    client.stop_processing().expect("stop-processing");
    client.unload_plugin().expect("unload-plugin");
    client.shutdown().expect("shutdown");
}
