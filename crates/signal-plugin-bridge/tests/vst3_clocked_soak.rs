//! Device-less clocked soak with the VST3 in-process processor on a Sum
//! stage (g11.031): the render plane runs under
//! `signal_hardware::FakeClockedBackend` for a sustained stretch of
//! callbacks with the rustc-compiled VST3 fixture processing every block.
//!
//! Like `signal-render-plane/tests/fake_clocked_soak.rs`, the
//! counting-allocator zero-alloc proof cannot run from a shared test target;
//! this soak asserts the observable contract instead: sustained clocked
//! load with plugin processing never panics, never misses a block
//! (`miss_count == 0` — the alloc-free session kept every deadline), health
//! counters advance, and the fixture's gain is audible in the stage meters.

use std::sync::Arc;
use std::time::Duration;

use signal_hardware::{FakeClockedBackend, OutputStreamBackend, OutputStreamSpec};
use signal_plugin_bridge::InProcessVst3Processor;
use signal_plugin_vst3::fixture::{
    compile_vst3_fixture, rustc_available, VST3_FIXTURE_CLASS_ID_HEX,
};
use signal_render_plane::{
    render_plane, ChannelFormat, RenderClipSpec, RenderEdgeSpec, RenderPlanSpec,
    RenderPluginProcessor, RenderSource, RenderStageKind, RenderStageSpec,
};

const SAMPLE_RATE_HZ: u32 = 48_000;
const BLOCK_FRAMES: u32 = 256;
const LANE_ID: u64 = 1;
const INSERT_ID: u64 = 2;
const MASTER_ID: u64 = 100;

fn plan_with_processor(processor: RenderPluginProcessor) -> RenderPlanSpec {
    RenderPlanSpec {
        sample_rate_hz: SAMPLE_RATE_HZ,
        master_gain: 1.0,
        master_limiter: None,
        stages: vec![
            RenderStageSpec {
                parameter_envelopes: Vec::new(),
                accepts_live_events: false,
                processor: None,
                events: None,
                stage_id: LANE_ID,
                format: ChannelFormat::stereo(),
                gain: 0.5,
                gain_automation: None,
                kind: RenderStageKind::Source {
                    clips: vec![RenderClipSpec {
                        clip_id: 11,
                        start_frames: 0,
                        end_frames: u64::MAX,
                        source: RenderSource::TestTone {
                            frequency_hz: 440.0,
                        },
                        loop_source: false,
                        fade_in_frames: 0,
                        fade_out_frames: 0,
                    }],
                },
                inputs: Vec::new(),
            },
            // The VST3 fixture processes every block of this Sum stage.
            RenderStageSpec {
                parameter_envelopes: Vec::new(),
                accepts_live_events: false,
                processor: Some(processor),
                events: None,
                stage_id: INSERT_ID,
                format: ChannelFormat::stereo(),
                gain: 1.0,
                gain_automation: None,
                kind: RenderStageKind::Sum,
                inputs: vec![RenderEdgeSpec {
                    source_stage_id: LANE_ID,
                    gain: 1.0,
                    matrix: None,
                }],
            },
            RenderStageSpec {
                parameter_envelopes: Vec::new(),
                accepts_live_events: false,
                processor: None,
                events: None,
                stage_id: MASTER_ID,
                format: ChannelFormat::stereo(),
                gain: 1.0,
                gain_automation: None,
                kind: RenderStageKind::Output,
                inputs: vec![RenderEdgeSpec {
                    source_stage_id: INSERT_ID,
                    gain: 1.0,
                    matrix: None,
                }],
            },
        ],
    }
}

#[test]
fn vst3_in_process_soak_processes_every_clocked_block_without_misses() {
    if !soak_tests_enabled() {
        return;
    }
    if !rustc_available() {
        eprintln!("skipping: rustc unavailable for the VST3 fixture");
        return;
    }
    let directory = std::env::temp_dir().join(format!(
        "signal-vst3-clocked-soak-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos(),
    ));
    let bundle = compile_vst3_fixture(
        &directory,
        "plugin:vst3:clocked-soak",
        "Signal VST3 Clocked Soak",
    )
    .expect("vst3 fixture should compile");

    let backend = Arc::new(
        InProcessVst3Processor::load_and_activate(
            &bundle,
            VST3_FIXTURE_CLASS_ID_HEX,
            SAMPLE_RATE_HZ,
            signal_render_plane::MAX_BLOCK_FRAMES as u32,
        )
        .expect("backend should load and activate"),
    );
    let handle = RenderPluginProcessor::new(Arc::clone(&backend) as Arc<_>);

    let (mut controller, mut executor) = render_plane();
    let hardware = FakeClockedBackend::new();
    let stream = hardware
        .open_output_stream(
            OutputStreamSpec {
                sample_rate_hz: SAMPLE_RATE_HZ,
                channels: 2,
                buffer_frames: Some(BLOCK_FRAMES),
            },
            Box::new(move |frames| {
                executor.render_block(frames);
            }),
        )
        .expect("open fake clocked stream");
    assert_eq!(stream.last_error(), None);

    controller
        .set_stream_channels(stream.channels())
        .expect("record stream channels");
    controller
        .install_plan(&plan_with_processor(handle))
        .expect("install plan");
    controller.set_playing(true).expect("play");

    // ~1.2 s of simulated callbacks (256 frames ≈ 5.3 ms per block),
    // with a continuous fader-style param sweep hammering the g12.023 set
    // path from a control thread the whole time: the queue-backed
    // IParameterChanges delivery must never cost the audio thread a
    // deadline (miss_count stays 0 below).
    use signal_plugin_vst3::fixture::VST3_FIXTURE_GAIN_PARAM_ID;
    let sweep_backend = Arc::clone(&backend);
    let sweep = std::thread::spawn(move || {
        for step in 0..1_200u32 {
            // 0.25..=0.75 triangle sweep; the exact values are irrelevant,
            // sustained concurrent pressure is the point.
            let phase = (step % 100) as f32 / 100.0;
            let normalized = 0.25
                + 0.5
                    * if phase < 0.5 {
                        phase * 2.0
                    } else {
                        2.0 - phase * 2.0
                    };
            let _ = sweep_backend.set_parameter_normalized(VST3_FIXTURE_GAIN_PARAM_ID, normalized);
            std::thread::sleep(Duration::from_millis(1));
        }
    });
    std::thread::sleep(Duration::from_millis(1_200));
    sweep.join().expect("sweep thread joins");

    let callbacks = controller.callback_count();
    assert!(
        callbacks >= 100,
        "expected sustained clocked callbacks, saw {callbacks}",
    );
    assert!(controller.position_frames() > SAMPLE_RATE_HZ as u64 / 2);
    assert_eq!(
        backend.miss_count(),
        0,
        "the alloc-free VST3 session must never bypass under clocked load",
    );

    // The insert is audibly live: fixture gain halves the lane feed, so the
    // insert stage meters read roughly half the lane's peak.
    let meters = controller.meters();
    let lane = meters
        .iter()
        .find(|(id, _, _)| *id == LANE_ID)
        .expect("lane meter");
    let insert = meters
        .iter()
        .find(|(id, _, _)| *id == INSERT_ID)
        .expect("insert meter");
    assert!(lane.1 > 0.01, "lane peak should move, saw {}", lane.1);
    assert!(
        insert.1 > 0.005,
        "insert peak should move, saw {}",
        insert.1
    );
    assert!(
        insert.1 < lane.1,
        "fixture gain must attenuate the insert stage: {} vs {}",
        insert.1,
        lane.1,
    );

    controller.set_playing(false).expect("stop");
    std::thread::sleep(Duration::from_millis(120));
    drop(stream);
    backend.shutdown();
    drop(backend);
    let _ = std::fs::remove_dir_all(&directory);
}

/// Wall-clock soak gate.
///
/// These tests sleep for a fixed wall-clock span and then assert a minimum
/// callback count, which is a claim about sustained real-time throughput on the
/// machine running them. That claim is only meaningful on a host that is not
/// otherwise loaded. Shared CI runners cannot satisfy it, and a throughput
/// assertion that can be retried until it passes is not a proof of anything.
///
/// They run when `SIGNAL_SOAK_TESTS=1`, and say why when they do not. The
/// `test:soak` effigy task sets it and runs them single-threaded.
fn soak_tests_enabled() -> bool {
    if std::env::var("SIGNAL_SOAK_TESTS").as_deref() == Ok("1") {
        return true;
    }
    eprintln!("SKIPPED: wall-clock soak test; set SIGNAL_SOAK_TESTS=1 (or run `effigy test:soak`)");
    false
}
