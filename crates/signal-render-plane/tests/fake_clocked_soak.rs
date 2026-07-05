//! Device-less clocked soak: the render plane runs under
//! `signal_hardware::FakeClockedBackend` for a couple of simulated seconds
//! of callbacks — no audio hardware, CI-safe.
//!
//! The counting-allocator zero-alloc proof cannot run here (a global
//! allocator cannot be installed from a test target shared with the
//! harness); `examples/render_soak.rs` remains the alloc proof against real
//! cpal hardware. This test asserts the observability surface instead:
//! callback-health counters advance, meters move, synthetic starvation
//! registers as xruns, and nothing panics under sustained clocked load.

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;

use signal_hardware::{FakeClockedBackend, OutputStreamBackend, OutputStreamSpec};
use signal_render_plane::{
    render_plane, ChannelFormat, RenderClipSpec, RenderEdgeSpec, RenderNote, RenderNoteBuffer,
    RenderPlanSpec, RenderSampleBuffer, RenderSource, RenderStageKind, RenderStageSpec,
};

const SAMPLE_RATE_HZ: u32 = 48_000;
const BLOCK_FRAMES: u32 = 256;
const LANE_ID: u64 = 1;
const NOTES_LANE_ID: u64 = 2;
const WARPED_LANE_ID: u64 = 3;
const MASTER_ID: u64 = 100;

fn tone_plan() -> RenderPlanSpec {
    // Notes lane (g11.011): overlapping stateless voices sound for the whole
    // soak, so the note overlap scan and per-voice synthesis run under
    // sustained clocked load alongside the tone.
    let notes: Vec<RenderNote> = (0..240)
        .map(|index| RenderNote {
            start_frame: index * 12_000,
            duration_frames: 18_000,
            degree: 57 + [0i32, 4, 7, 12][index as usize % 4],
            pitch_intent: None,
            velocity: 0.5,
        })
        .collect();
    // Warped lane (g12.027): a looping in-memory sine buffer rate-warped to
    // 1.5 (non-trivial sinc interpolation every block), so warp playback
    // shares the sustained clocked load with the tone and note lanes.
    let warped_data: Vec<f32> = (0..SAMPLE_RATE_HZ as usize)
        .flat_map(|n| {
            let value =
                (std::f32::consts::TAU * 220.0 * n as f32 / SAMPLE_RATE_HZ as f32).sin() * 0.4;
            [value, value]
        })
        .collect();
    RenderPlanSpec {
        sample_rate_hz: SAMPLE_RATE_HZ,
        master_gain: 0.5,
        master_limiter: None,
        stages: vec![
            RenderStageSpec {
                processor: None,
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
                    }],
                },
                inputs: Vec::new(),
            },
            RenderStageSpec {
                processor: None,
                stage_id: NOTES_LANE_ID,
                format: ChannelFormat::stereo(),
                gain: 0.3,
                gain_automation: None,
                kind: RenderStageKind::Source {
                    clips: vec![RenderClipSpec {
                        clip_id: 12,
                        start_frames: 0,
                        end_frames: u64::MAX,
                        source: RenderSource::Notes(RenderNoteBuffer {
                            notes: notes.into(),
                        }),
                        loop_source: false,
                    }],
                },
                inputs: Vec::new(),
            },
            RenderStageSpec {
                processor: None,
                stage_id: WARPED_LANE_ID,
                format: ChannelFormat::stereo(),
                gain: 0.3,
                gain_automation: None,
                kind: RenderStageKind::Source {
                    clips: vec![RenderClipSpec {
                        clip_id: 13,
                        start_frames: 0,
                        end_frames: u64::MAX,
                        source: RenderSource::Warped {
                            source: Box::new(RenderSource::Samples(RenderSampleBuffer {
                                sample_rate_hz: SAMPLE_RATE_HZ,
                                frames: warped_data.into(),
                            })),
                            rate: 1.5,
                        },
                        loop_source: true,
                    }],
                },
                inputs: Vec::new(),
            },
            RenderStageSpec {
                processor: None,
                stage_id: MASTER_ID,
                format: ChannelFormat::stereo(),
                gain: 1.0,
                gain_automation: None,
                kind: RenderStageKind::Output,
                inputs: vec![
                    RenderEdgeSpec {
                        source_stage_id: LANE_ID,
                        gain: 1.0,
                        matrix: None,
                    },
                    RenderEdgeSpec {
                        source_stage_id: NOTES_LANE_ID,
                        gain: 1.0,
                        matrix: None,
                    },
                    RenderEdgeSpec {
                        source_stage_id: WARPED_LANE_ID,
                        gain: 1.0,
                        matrix: None,
                    },
                ],
            },
        ],
    }
}

#[test]
fn clocked_soak_advances_health_counters_and_meters() {
    let (mut controller, mut executor) = render_plane();
    let block_duration = Duration::from_secs_f64(BLOCK_FRAMES as f64 / SAMPLE_RATE_HZ as f64);

    // Synthetic starvation: during a bounded early stretch, every 32nd
    // callback sleeps for ~3 block durations before rendering, so the next
    // callback arrives late and the executor's interval inference must count
    // an xrun. Callbacks after the stretch run at clean cadence so the test
    // can assert recovery.
    const STARVED_LAST_CALLBACK: u64 = 160;
    let callback_index = Arc::new(AtomicU64::new(0));
    let render_callback_index = Arc::clone(&callback_index);
    let backend = FakeClockedBackend::new();
    let stream = backend
        .open_output_stream(
            OutputStreamSpec {
                sample_rate_hz: SAMPLE_RATE_HZ,
                channels: 2,
                buffer_frames: Some(BLOCK_FRAMES),
            },
            Box::new(move |frames| {
                let index = render_callback_index.fetch_add(1, Ordering::Relaxed);
                if index > 0 && index <= STARVED_LAST_CALLBACK && index.is_multiple_of(32) {
                    std::thread::sleep(block_duration * 3);
                }
                executor.render_block(frames);
            }),
        )
        .expect("open fake clocked stream");
    assert_eq!(stream.sample_rate_hz(), SAMPLE_RATE_HZ);
    assert_eq!(stream.channels(), 2);
    assert_eq!(stream.last_error(), None);

    controller
        .set_stream_channels(stream.channels())
        .expect("record stream channels");
    controller.install_plan(&tone_plan()).expect("install plan");
    controller.set_playing(true).expect("play");

    // ~1.5 s of simulated callbacks (256 frames ≈ 5.3 ms per block).
    std::thread::sleep(Duration::from_millis(1_500));

    let callbacks = controller.callback_count();
    assert!(
        callbacks >= 100,
        "expected sustained clocked callbacks, saw {callbacks}",
    );
    assert!(controller.position_frames() > SAMPLE_RATE_HZ as u64 / 2);
    assert!(controller.last_callback_duration_micros() > 0 || callbacks > 0);
    assert!(
        controller.max_callback_duration_micros() >= controller.last_callback_duration_micros()
    );
    // Duration counters measure render_block itself (the starvation sleep
    // sits outside it, showing up as interval/xruns instead).
    assert!(
        controller.max_callback_duration_micros() > 0,
        "max callback duration should be recorded",
    );
    assert!(
        controller.xrun_count() >= 1,
        "synthetic starvation must register as xruns",
    );

    // Starvation recovery: after the starved stretch (callbacks ≤160; ~1.5 s
    // covers well past it) playback must keep advancing and the xrun counter
    // must stop growing once cadence is clean again. Allow ≤1 straggler from
    // the first clean interval after the final injected stall.
    assert!(
        callback_index.load(Ordering::Relaxed) > STARVED_LAST_CALLBACK,
        "starved stretch should be over before sampling recovery",
    );
    let recovered_position = controller.position_frames();
    let recovered_xruns = controller.xrun_count();
    std::thread::sleep(Duration::from_millis(500));
    assert!(
        controller.position_frames() > recovered_position,
        "playback must keep advancing after starvation: {} -> {}",
        recovered_position,
        controller.position_frames(),
    );
    let xrun_delta = controller.xrun_count() - recovered_xruns;
    assert!(
        xrun_delta <= 1,
        "xruns must stop accruing once cadence recovers, saw +{xrun_delta}",
    );

    let meters = controller.meters();
    assert_eq!(
        meters.len(),
        4,
        "tone lane + notes lane + warped lane + master metered"
    );
    let warped = meters
        .iter()
        .find(|(id, _, _)| *id == WARPED_LANE_ID)
        .unwrap();
    assert!(warped.1 > 0.01, "warped peak should move, saw {}", warped.1);
    let lane = meters.iter().find(|(id, _, _)| *id == LANE_ID).unwrap();
    let notes = meters
        .iter()
        .find(|(id, _, _)| *id == NOTES_LANE_ID)
        .unwrap();
    let master = meters.iter().find(|(id, _, _)| *id == MASTER_ID).unwrap();
    assert!(lane.1 > 0.01, "lane peak should move, saw {}", lane.1);
    assert!(lane.2 > 0.001, "lane rms should move, saw {}", lane.2);
    assert!(notes.1 > 0.01, "notes peak should move, saw {}", notes.1);
    assert!(
        master.1 > 0.001,
        "master peak should move, saw {}",
        master.1
    );

    // Stop: meters publish zeros once the edge ramp closes.
    controller.set_playing(false).expect("stop");
    std::thread::sleep(Duration::from_millis(120));
    let meters = controller.meters();
    assert!(
        meters
            .iter()
            .all(|(_, peak, rms)| *peak == 0.0 && *rms == 0.0),
        "stopped meters should read zero: {meters:?}",
    );

    drop(stream);
}
