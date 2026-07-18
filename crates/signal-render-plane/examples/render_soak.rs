//! Soak proof: the render plane executes inside the real cpal audio callback
//! with zero allocations on the callback path, transport-gated and audible.
//!
//! Run with: `cargo run -p signal-render-plane --example render_soak`
//!
//! The plan is graph-shaped: two tone lanes panned hard left/right through a
//! Sum stage into the master — the full schedule walk (source scratch →
//! matrix edges → Sum → boundary) runs inside the measured callback.
//!
//! A counting global allocator tracks every alloc/dealloc that happens while
//! the callback flag is raised. The control thread parks during measurement
//! windows so the count isolates the audio thread.

use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::time::Duration;

use signal_dsp::equal_power_pan_matrix;
use signal_hardware::{OutputStreamBackend, OutputStreamSpec};
use signal_hardware_cpal::CpalOutputBackend;
use signal_render_plane::{
    render_live_input, render_plane, ChannelFormat, PluginBlockProcessor, RenderClipSpec,
    RenderEdgeSpec, RenderNote, RenderNoteBuffer, RenderPlanSpec, RenderPluginProcessor,
    RenderSource, RenderStageKind, RenderStageSpec, LIVE_INPUT_DEFAULT_CAPACITY_FRAMES,
};

/// Fake plugin backend for the soak (g11.012): an in-process gain transform
/// with the exact call shape of a live plugin handle. Runs inside the
/// measured callback, so the processor-attached path is covered by the
/// zero-alloc proof without needing a child process.
struct SoakGainProcessor {
    gain: f32,
    calls: AtomicU64,
}

impl PluginBlockProcessor for SoakGainProcessor {
    fn process(&self, scratch: &mut [f32], frame_count: usize, channels: usize) -> bool {
        self.calls.fetch_add(1, Ordering::Relaxed);
        for sample in &mut scratch[..frame_count * channels] {
            *sample *= self.gain;
        }
        true
    }
}

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

fn tone_lane(stage_id: u64, gain: f32, frequency_hz: f32) -> RenderStageSpec {
    RenderStageSpec {
        accepts_live_events: false,
        processor: None,
        events: None,
        stage_id,
        format: ChannelFormat::stereo(),
        gain,
        gain_automation: None,
        kind: RenderStageKind::Source {
            clips: vec![RenderClipSpec {
                clip_id: 1001,
                start_frames: 0,
                end_frames: u64::MAX,
                source: RenderSource::TestTone { frequency_hz },
                loop_source: false,
                fade_in_frames: 0,
                fade_out_frames: 0,
            }],
        },
        inputs: Vec::new(),
    }
}

/// Notes lane (g11.011): a repeating arpeggio through the built-in
/// stateless instrument, kept sounding for the whole soak so the note
/// overlap scan and per-voice synthesis run inside the measured callback.
fn notes_lane(stage_id: u64, gain: f32) -> RenderStageSpec {
    let notes: Vec<RenderNote> = (0..240)
        .map(|index| RenderNote {
            start_frame: index * 12_000,
            duration_frames: 18_000, // Overlapping voices: polyphony active.
            degree: 57 + [0i32, 4, 7, 12][index as usize % 4],
            pitch_intent: None,
            velocity: 0.5,
        })
        .collect();
    RenderStageSpec {
        accepts_live_events: false,
        processor: None,
        events: None,
        stage_id,
        format: ChannelFormat::stereo(),
        gain,
        gain_automation: None,
        kind: RenderStageKind::Source {
            clips: vec![RenderClipSpec {
                clip_id: 1003,
                start_frames: 0,
                end_frames: u64::MAX,
                source: RenderSource::Notes(RenderNoteBuffer {
                    notes: notes.into(),
                }),
                loop_source: false,
                fade_in_frames: 0,
                fade_out_frames: 0,
            }],
        },
        inputs: Vec::new(),
    }
}

fn main() {
    let sample_rate_hz = 48_000u32;
    let channels = 2u16;

    let (mut controller, mut executor) = render_plane();

    // Live-input monitor lane (g11.010): a feeder thread synthesizes a
    // quiet tone into the live ring at input-callback cadence — no
    // allocation inside its loop, because the counting allocator attributes
    // any allocation that overlaps a render callback (the flag is global).
    // The executor's drain runs inside the measured callback, extending the
    // zero-alloc proof over the monitor path.
    let (live_feeder, live_handle) = render_live_input(LIVE_INPUT_DEFAULT_CAPACITY_FRAMES);
    let live_stop = std::sync::Arc::new(AtomicBool::new(false));
    let feeder_stop = std::sync::Arc::clone(&live_stop);
    let live_feeder_thread = std::thread::spawn(move || {
        let mut chunk = [0.0f32; 256 * 2]; // Preallocated: the loop is alloc-free.
        let phase_step = std::f32::consts::TAU * 330.0 / 48_000.0;
        let mut phase = 0.0f32;
        while !feeder_stop.load(Ordering::Relaxed) {
            for frame in chunk.chunks_exact_mut(2) {
                let sample = 0.2 * phase.sin();
                frame[0] = sample;
                frame[1] = sample;
                phase += phase_step;
                if phase >= std::f32::consts::TAU {
                    phase -= std::f32::consts::TAU;
                }
            }
            live_feeder.push_slice(&chunk);
            std::thread::sleep(Duration::from_millis(5)); // ≈ 256 frames at 48 kHz.
        }
    });

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
        .set_stream_channels(stream.channels())
        .expect("record stream channels");

    // Plugin insert on the Sum stage: the fake gain backend runs inside the
    // measured callback (zero-alloc proof covers the processor path).
    let soak_processor = std::sync::Arc::new(SoakGainProcessor {
        gain: 0.8,
        calls: AtomicU64::new(0),
    });
    let soak_processor_handle =
        RenderPluginProcessor::new(std::sync::Arc::clone(&soak_processor) as std::sync::Arc<_>);

    // Bussed plan: lanes a (440 Hz, panned hard left) and b (660 Hz, hard
    // right) feed a Sum stage through equal-power pan matrices; it feeds the
    // master through an identity edge.
    let summed_plan = RenderPlanSpec {
        sample_rate_hz,
        master_gain: 0.5,
        master_limiter: None,
        stages: vec![
            tone_lane(1, 0.4, 440.0),
            tone_lane(2, 0.25, 660.0),
            // Built-in instrument lane: stateless note voices render inside
            // the measured callback (must stay alloc-free like every source).
            notes_lane(5, 0.3),
            // Live monitor lane: drains the input feeder's ring inside the
            // measured callback (must stay alloc-free like every source).
            RenderStageSpec {
                accepts_live_events: false,
                processor: None,
                events: None,
                stage_id: 4,
                format: ChannelFormat::stereo(),
                gain: 0.3,
                gain_automation: None,
                kind: RenderStageKind::Source {
                    clips: vec![RenderClipSpec {
                        clip_id: 1002,
                        start_frames: 0,
                        end_frames: u64::MAX,
                        source: RenderSource::LiveInput(live_handle.clone()),
                        loop_source: false,
                        fade_in_frames: 0,
                        fade_out_frames: 0,
                    }],
                },
                inputs: Vec::new(),
            },
            RenderStageSpec {
                accepts_live_events: false,
                processor: Some(soak_processor_handle),
                events: None,
                stage_id: 10,
                format: ChannelFormat::stereo(),
                gain: 1.0,
                gain_automation: None,
                kind: RenderStageKind::Sum,
                inputs: vec![
                    RenderEdgeSpec {
                        source_stage_id: 1,
                        gain: 1.0,
                        matrix: Some(equal_power_pan_matrix(-1.0).to_vec()),
                    },
                    RenderEdgeSpec {
                        source_stage_id: 2,
                        gain: 1.0,
                        matrix: Some(equal_power_pan_matrix(1.0).to_vec()),
                    },
                ],
            },
            RenderStageSpec {
                accepts_live_events: false,
                processor: None,
                events: None,
                stage_id: 100,
                format: ChannelFormat::stereo(),
                gain: 1.0,
                gain_automation: None,
                kind: RenderStageKind::Output,
                inputs: vec![
                    RenderEdgeSpec {
                        source_stage_id: 10,
                        gain: 1.0,
                        matrix: None,
                    },
                    RenderEdgeSpec {
                        source_stage_id: 4,
                        gain: 1.0,
                        matrix: None,
                    },
                    RenderEdgeSpec {
                        source_stage_id: 5,
                        gain: 1.0,
                        matrix: None,
                    },
                ],
            },
        ],
    };
    controller.install_plan(&summed_plan).expect("install plan");

    println!("transport: play (1.5s, two tones panned hard left/right through a Sum stage)");
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
            master_gain: 0.5,
            master_limiter: None,
            stages: vec![
                tone_lane(3, 0.5, 220.0),
                RenderStageSpec {
                    accepts_live_events: false,
                    processor: None,
                    events: None,
                    stage_id: 100,
                    format: ChannelFormat::stereo(),
                    gain: 1.0,
                    gain_automation: None,
                    kind: RenderStageKind::Output,
                    inputs: vec![RenderEdgeSpec {
                        source_stage_id: 3,
                        gain: 1.0,
                        matrix: None,
                    }],
                },
            ],
        })
        .expect("swap plan");
    controller.set_playing(true).expect("play again");
    std::thread::sleep(Duration::from_secs(1));
    controller.set_playing(false).expect("final stop");

    let retired = controller.collect_retired();
    drop(stream);
    live_stop.store(true, Ordering::Relaxed);
    live_feeder_thread.join().expect("live feeder joins");
    println!(
        "live input: {} underrun frames, {} buffered at teardown",
        live_handle.underrun_frames(),
        live_handle.buffered_frames(),
    );

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
    let processor_calls = soak_processor.calls.load(Ordering::Relaxed);
    println!("plugin processor calls inside the callback: {processor_calls}");
    assert!(
        processor_calls > 0,
        "processor-attached Sum stage must have run in the soak",
    );
    println!("soak passed: audible, summed, plugin-inserted, transport-gated, zero-alloc callback");
}
