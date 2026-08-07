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

    // Warm the bridge before handing it back: prove the child answered once,
    // so callers do not spend their first assertion discovering it was still
    // being scheduled.
    //
    // The wait is the offline one. Nothing in this function is a realtime
    // callback, so the child missing a 333 us budget while the OS gets round
    // to its audio thread is not information -- it is noise that used to fail
    // this binary three different-looking ways under load.
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
    let mut scratch = vec![0.0f32; lease.max_frames as usize * lease.channels as usize];
    let channels = lease.channels as usize;
    process_offline(&handle, &mut scratch, 64, channels);

    (client, processor)
}

/// CLAP-fixture convenience wrapper around [`spawn_processing_session_for`].
fn spawn_processing_session(
    library_path: &std::path::Path,
) -> (SandboxBrokerClientSession, Arc<ShmPluginProcessor>) {
    spawn_processing_session_for(library_path, FIXTURE_PLUGIN_ID)
}

/// Drive one block the way an offline render does: wait for the child rather
/// than bypassing on the realtime budget.
///
/// None of these tests is an audio callback. The realtime budget exists so a
/// callback returns before its output buffer drains, and bypassing a slow
/// block is the right answer there. Here there is no buffer: a miss is simply
/// a wrong assertion, and under machine load it lands at an unpredictable
/// block, which is what made this binary fail `5` of `10` runs under
/// deliberate load on 2026-08-05 in three different-looking ways.
///
/// Retrying is NOT the alternative. A missed request is still published, and
/// the child may serve it after the parent gave up; for a block carrying note
/// events the retry then delivers a second `NoteOn` and the voice is already
/// sounding before its own offset. Measured: retrying broke
/// `real_child_instrument_accepts_zero_input_and_generates_audio_from_note_events`
/// under load exactly that way. Waiting is idempotent; retrying is not.
///
/// Scoped per call rather than set once at attach, because four tests in this
/// binary assert the *realtime* bound against a dead child and must keep it.
fn process_offline(
    handle: &RenderPluginProcessor,
    scratch: &mut [f32],
    frames: usize,
    channels: usize,
) {
    let previous = handle.set_offline_waiting(true);
    let processed = handle.process(scratch, frames, channels);
    handle.set_offline_waiting(previous);
    assert!(
        processed,
        "child never answered a process request within the offline wait budget",
    );
}

/// [`process_offline`] with a per-block event slice.
fn process_offline_with_events(
    handle: &RenderPluginProcessor,
    scratch: &mut [f32],
    frames: usize,
    channels: usize,
    events: &[RenderBlockPluginEvent],
) {
    let previous = handle.set_offline_waiting(true);
    let processed = handle.process_with_events(scratch, frames, channels, events);
    handle.set_offline_waiting(previous);
    assert!(
        processed,
        "child never answered an event process request within the offline wait budget",
    );
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

mod au;
mod broker_rejects;
mod chain_insert;
mod editor;
mod instruments;
mod kill_budget;
mod lv2;
mod params;
mod shm_processing;
mod vst3;
