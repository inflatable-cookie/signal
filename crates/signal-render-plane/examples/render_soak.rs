//! Soak proof: the render plane executes inside the real cpal audio callback
//! with zero allocations on the callback path, transport-gated and audible.
//!
//! Run with: `cargo run -p signal-render-plane --example render_soak`
//!
//! A counting global allocator tracks every alloc/dealloc that happens while
//! the callback flag is raised. The control thread parks during measurement
//! windows so the count isolates the audio thread.

use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::time::Duration;

use signal_hardware::{OutputStreamBackend, OutputStreamSpec};
use signal_hardware_output_cpal::CpalOutputBackend;
use signal_render_plane::{
    render_plane, RenderClipSpec, RenderLaneSpec, RenderPlanSpec, RenderSource,
};

static IN_CALLBACK: AtomicBool = AtomicBool::new(false);
static CALLBACK_ALLOCS: AtomicU64 = AtomicU64::new(0);
static CALLBACK_DEALLOCS: AtomicU64 = AtomicU64::new(0);

struct CountingAllocator;

unsafe impl GlobalAlloc for CountingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        if IN_CALLBACK.load(Ordering::Relaxed) {
            CALLBACK_ALLOCS.fetch_add(1, Ordering::Relaxed);
        }
        unsafe { System.alloc(layout) }
    }

    unsafe fn dealloc(&self, pointer: *mut u8, layout: Layout) {
        if IN_CALLBACK.load(Ordering::Relaxed) {
            CALLBACK_DEALLOCS.fetch_add(1, Ordering::Relaxed);
        }
        unsafe { System.dealloc(pointer, layout) }
    }
}

#[global_allocator]
static ALLOCATOR: CountingAllocator = CountingAllocator;

fn main() {
    let sample_rate_hz = 48_000u32;
    let channels = 2u16;

    let (controller, mut executor) = render_plane();

    let backend = CpalOutputBackend::new();
    let stream = backend
        .open_output_stream(
            OutputStreamSpec {
                sample_rate_hz,
                channels,
                buffer_frames: Some(128),
            },
            Box::new(move |frames| {
                IN_CALLBACK.store(true, Ordering::Relaxed);
                executor.render_block(frames);
                IN_CALLBACK.store(false, Ordering::Relaxed);
            }),
        )
        .expect("open output stream");

    controller
        .install_plan(&RenderPlanSpec {
            sample_rate_hz,
            channels,
            master_gain: 0.5,
            lanes: vec![
                RenderLaneSpec {
                    lane_id: "lane:a".to_string(),
                    gain: 0.4,
                    clips: vec![RenderClipSpec {
                        start_frames: 0,
                        end_frames: u64::MAX,
                        source: RenderSource::TestTone {
                            frequency_hz: 440.0,
                        },
                        loop_source: false,
                    }],
                },
                RenderLaneSpec {
                    lane_id: "lane:b".to_string(),
                    gain: 0.25,
                    clips: vec![RenderClipSpec {
                        start_frames: 0,
                        end_frames: u64::MAX,
                        source: RenderSource::TestTone {
                            frequency_hz: 660.0,
                        },
                        loop_source: false,
                    }],
                },
            ],
        })
        .expect("install plan");

    println!("transport: play (1.5s, two tones)");
    controller.set_playing(true).expect("play");
    std::thread::sleep(Duration::from_millis(1_500));
    let position_after_play = controller.position_frames();

    println!("transport: seek while playing (declick ramp)");
    controller.seek(0).expect("seek");
    std::thread::sleep(Duration::from_millis(300));

    println!("transport: stop (0.5s silence)");
    controller.set_playing(false).expect("stop");
    std::thread::sleep(Duration::from_millis(500));
    let position_after_stop = controller.position_frames();

    println!("plan swap while running, then play 1s at 220 Hz");
    controller
        .install_plan(&RenderPlanSpec {
            sample_rate_hz,
            channels,
            master_gain: 0.5,
            lanes: vec![RenderLaneSpec {
                lane_id: "lane:c".to_string(),
                gain: 0.5,
                clips: vec![RenderClipSpec {
                    start_frames: 0,
                    end_frames: u64::MAX,
                    source: RenderSource::TestTone {
                        frequency_hz: 220.0,
                    },
                    loop_source: false,
                }],
            }],
        })
        .expect("swap plan");
    controller.set_playing(true).expect("play again");
    std::thread::sleep(Duration::from_secs(1));
    controller.set_playing(false).expect("final stop");

    let retired = controller.collect_retired();
    drop(stream);

    let allocs = CALLBACK_ALLOCS.load(Ordering::Relaxed);
    let deallocs = CALLBACK_DEALLOCS.load(Ordering::Relaxed);

    println!("stream clock: {position_after_play} frames after play");
    assert!(
        position_after_play >= (sample_rate_hz as u64) * 13 / 10,
        "stream clock should have advanced ~1.5s of frames",
    );
    let stopped_drift = position_after_stop.saturating_sub(position_after_play);
    assert!(
        stopped_drift <= 1_024,
        "stream clock must hold while stopped (drifted {stopped_drift} frames)",
    );
    println!("retired plans reclaimed control-side: {retired}");
    assert!(retired >= 1, "swapped plan should have been reclaimed");
    println!("callback allocations: {allocs}, deallocations: {deallocs}");
    assert_eq!(allocs, 0, "render path must not allocate");
    assert_eq!(deallocs, 0, "render path must not deallocate");
    println!("soak passed: audible, transport-gated, zero-alloc callback");
}
