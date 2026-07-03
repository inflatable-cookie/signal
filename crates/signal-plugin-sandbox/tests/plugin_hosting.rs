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
use signal_plugin_clap::fixture::{compile_clap_fixture, rustc_available, CLAP_FIXTURE_GAIN};
use signal_plugin_lv2::fixture::{
    compile_lv2_fixture, LV2_FIXTURE_GAIN, LV2_FIXTURE_GAIN_PORT_INDEX,
};
use signal_plugin_vst3::fixture::{
    compile_vst3_fixture, VST3_FIXTURE_CLASS_ID_HEX, VST3_FIXTURE_GAIN,
};
use signal_render_plane::{
    render_plan_to_pcm, ChannelFormat, OfflineRenderOptions, RenderClipSpec, RenderEdgeSpec,
    RenderPlanSpec, RenderPluginProcessor, RenderSampleBuffer, RenderSource, RenderStageKind,
    RenderStageSpec,
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
    assert!((gain.default_value - 0.5).abs() < 1e-6);

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
fn process_with_retries(handle: &RenderPluginProcessor, scratch: &mut [f32], frames: usize) {
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        if handle.process(scratch, frames, 2) {
            return;
        }
        assert!(
            Instant::now() < deadline,
            "child never answered a process request",
        );
        std::thread::yield_now();
    }
}

#[test]
fn real_child_processes_blocks_through_the_shm_bridge() {
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
fn killed_child_bypasses_within_budget_instead_of_hanging() {
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
    let buffer = RenderSampleBuffer {
        sample_rate_hz: SAMPLE_RATE_HZ,
        frames: data.into(),
    };
    let plan = |processor: Option<RenderPluginProcessor>| RenderPlanSpec {
        sample_rate_hz: SAMPLE_RATE_HZ,
        master_gain: 1.0,
        master_limiter: None,
        stages: vec![
            RenderStageSpec {
                processor: None,
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
                    }],
                },
                inputs: Vec::new(),
            },
            RenderStageSpec {
                processor,
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
                processor: None,
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
