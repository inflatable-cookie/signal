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
    render_plane, ChannelFormat, RenderClipSpec, RenderEdgeSpec, RenderPlanSpec, RenderSource,
    RenderStageKind, RenderStageSpec,
};

const SAMPLE_RATE_HZ: u32 = 48_000;
const BLOCK_FRAMES: u32 = 256;
const LANE_ID: u64 = 1;
const MASTER_ID: u64 = 100;

fn tone_plan() -> RenderPlanSpec {
    RenderPlanSpec {
        sample_rate_hz: SAMPLE_RATE_HZ,
        master_gain: 0.5,
        master_limiter: None,
        stages: vec![
            RenderStageSpec {
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
                stage_id: MASTER_ID,
                format: ChannelFormat::stereo(),
                gain: 1.0,
                gain_automation: None,
                kind: RenderStageKind::Output,
                inputs: vec![RenderEdgeSpec {
                    source_stage_id: LANE_ID,
                    gain: 1.0,
                    matrix: None,
                }],
            },
        ],
    }
}

#[test]
fn clocked_soak_advances_health_counters_and_meters() {
    let (mut controller, mut executor) = render_plane();
    let block_duration = Duration::from_secs_f64(BLOCK_FRAMES as f64 / SAMPLE_RATE_HZ as f64);

    // Synthetic starvation: every 32nd callback sleeps for ~3 block
    // durations before rendering, so the next callback arrives late and the
    // executor's interval inference must count an xrun.
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
                if index > 0 && index.is_multiple_of(32) {
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

    let meters = controller.meters();
    assert_eq!(meters.len(), 2, "one lane + one master metered");
    let lane = meters.iter().find(|(id, _, _)| *id == LANE_ID).unwrap();
    let master = meters.iter().find(|(id, _, _)| *id == MASTER_ID).unwrap();
    assert!(lane.1 > 0.01, "lane peak should move, saw {}", lane.1);
    assert!(lane.2 > 0.001, "lane rms should move, saw {}", lane.2);
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
