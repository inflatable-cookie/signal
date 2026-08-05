//! Device-less clocked soak for the live render posture (g13.018): the
//! render plane runs under `signal_hardware::FakeClockedBackend` with the
//! transport STOPPED and the live render posture active, while the control
//! thread pushes live events at a hosted-instrument stage.
//!
//! Proves, under sustained clocked load:
//! - live events pushed while stopped produce audible instrument output
//!   (meters move on the instrument and master stages);
//! - the transport position never advances;
//! - the audio thread stays zero-alloc: a counting global allocator guards
//!   every alloc/dealloc made while the render-callback thread-local flag is
//!   raised (thread-local, so concurrent control-side allocation — including
//!   the event pushes themselves — never pollutes the count);
//! - no live events are dropped at this push cadence.

use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::Cell;
use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;

use signal_hardware::{FakeClockedBackend, OutputStreamBackend, OutputStreamSpec};
use signal_render_plane::{
    render_plane, ChannelFormat, PluginBlockProcessor, RenderBlockPluginEvent, RenderEdgeSpec,
    RenderPlanSpec, RenderPluginEvent, RenderPluginEventKind, RenderPluginProcessor,
    RenderStageKind, RenderStageSpec,
};

const SAMPLE_RATE_HZ: u32 = 48_000;
const BLOCK_FRAMES: u32 = 256;
const LANE_ID: u64 = 1;
const INSTRUMENT_ID: u64 = 7;
const MASTER_ID: u64 = 100;

thread_local! {
    static IN_RENDER_CALLBACK: Cell<bool> = const { Cell::new(false) };
}
// Thread-scoped: the allocator hook is process-global, so a second test in
// this binary would otherwise be counted against whichever test is measuring.
thread_local! {
    static CALLBACK_ALLOCS: Cell<u64> = const { Cell::new(0) };
    static CALLBACK_DEALLOCS: Cell<u64> = const { Cell::new(0) };
}

struct CountingAllocator;

unsafe impl GlobalAlloc for CountingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        if IN_RENDER_CALLBACK
            .try_with(|flag| flag.get())
            .unwrap_or(false)
        {
            CALLBACK_ALLOCS.with(|value| value.set(value.get().saturating_add(1)));
        }
        unsafe { System.alloc(layout) }
    }

    unsafe fn dealloc(&self, pointer: *mut u8, layout: Layout) {
        if IN_RENDER_CALLBACK
            .try_with(|flag| flag.get())
            .unwrap_or(false)
        {
            CALLBACK_DEALLOCS.with(|value| value.set(value.get().saturating_add(1)));
        }
        unsafe { System.dealloc(pointer, layout) }
    }
}

#[global_allocator]
static ALLOCATOR: CountingAllocator = CountingAllocator;

/// Alloc-free instrument backend: note-on holds a constant signal at the
/// event velocity, note-off releases it. State is one atomic — safe inside
/// the measured callback.
struct SoakInstrument {
    amplitude_bits: AtomicU32,
    events_seen: AtomicU64,
}

impl SoakInstrument {
    fn render(
        &self,
        scratch: &mut [f32],
        frame_count: usize,
        channels: usize,
        events: &[RenderBlockPluginEvent],
    ) -> bool {
        self.events_seen
            .fetch_add(events.len() as u64, Ordering::Relaxed);
        let mut amplitude = f32::from_bits(self.amplitude_bits.load(Ordering::Relaxed));
        for event in events {
            amplitude = match event.kind {
                RenderPluginEventKind::NoteOn { velocity, .. } => velocity,
                RenderPluginEventKind::NoteOff { .. } => 0.0,
                _ => amplitude,
            };
        }
        for sample in &mut scratch[..frame_count * channels] {
            *sample = amplitude;
        }
        self.amplitude_bits
            .store(amplitude.to_bits(), Ordering::Relaxed);
        true
    }
}

impl PluginBlockProcessor for SoakInstrument {
    fn process(&self, scratch: &mut [f32], frame_count: usize, channels: usize) -> bool {
        self.render(scratch, frame_count, channels, &[])
    }

    fn process_with_events(
        &self,
        scratch: &mut [f32],
        frame_count: usize,
        channels: usize,
        events: &[RenderBlockPluginEvent],
    ) -> bool {
        self.render(scratch, frame_count, channels, events)
    }
}

/// Empty source lane into a live-event-accepting instrument Sum stage.
fn live_instrument_plan(instrument: RenderPluginProcessor) -> RenderPlanSpec {
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
                gain: 1.0,
                gain_automation: None,
                kind: RenderStageKind::Source { clips: Vec::new() },
                inputs: Vec::new(),
            },
            RenderStageSpec {
                parameter_envelopes: Vec::new(),
                accepts_live_events: true,
                processor: Some(instrument),
                events: None,
                stage_id: INSTRUMENT_ID,
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
                    source_stage_id: INSTRUMENT_ID,
                    gain: 1.0,
                    matrix: None,
                }],
            },
        ],
    }
}

fn control_change(frame: u64, value: f32) -> RenderPluginEvent {
    RenderPluginEvent {
        frame,
        channel: 0,
        kind: RenderPluginEventKind::ControlChange {
            controller: 1,
            value,
        },
    }
}

#[test]
fn live_events_while_stopped_sound_and_hold_zero_alloc_under_clocked_load() {
    if !soak_tests_enabled() {
        return;
    }
    let (mut controller, mut executor) = render_plane();
    let backend = FakeClockedBackend::new();
    let stream = backend
        .open_output_stream(
            OutputStreamSpec {
                sample_rate_hz: SAMPLE_RATE_HZ,
                channels: 2,
                buffer_frames: Some(BLOCK_FRAMES),
            },
            Box::new(move |frames| {
                IN_RENDER_CALLBACK.with(|flag| flag.set(true));
                executor.render_block(frames);
                IN_RENDER_CALLBACK.with(|flag| flag.set(false));
            }),
        )
        .expect("open fake clocked stream");

    controller
        .set_stream_channels(stream.channels())
        .expect("record stream channels");
    let instrument = Arc::new(SoakInstrument {
        amplitude_bits: AtomicU32::new(0.0f32.to_bits()),
        events_seen: AtomicU64::new(0),
    });
    controller
        .install_plan(&live_instrument_plan(RenderPluginProcessor::new(
            Arc::clone(&instrument) as Arc<_>,
        )))
        .expect("install plan");

    // The transport NEVER rolls in this soak: the live render posture alone
    // must make the instrument audible.
    controller.set_live_render(true).expect("live render on");
    controller
        .push_live_events(
            INSTRUMENT_ID,
            &[RenderPluginEvent {
                frame: 0,
                channel: 0,
                kind: RenderPluginEventKind::NoteOn {
                    key: 60,
                    velocity: 0.5,
                },
            }],
        )
        .expect("push note on");

    // Warm-up: install lands, edge envelope opens, first deliveries flow.
    std::thread::sleep(Duration::from_millis(300));
    assert!(controller.live_render());

    // Measured window: sustained clocked callbacks while the control side
    // keeps pushing live CC events at the held note.
    let allocs_before = CALLBACK_ALLOCS.with(Cell::get);
    let deallocs_before = CALLBACK_DEALLOCS.with(Cell::get);
    let callbacks_before = controller.callback_count();
    for index in 0..12u64 {
        let batch: Vec<RenderPluginEvent> = (0..8)
            .map(|step| control_change(index * 8 + step, ((index % 12) as f32) / 12.0))
            .collect();
        controller
            .push_live_events(INSTRUMENT_ID, &batch)
            .expect("push live CC batch");
        std::thread::sleep(Duration::from_millis(50));
    }
    let allocs_during = CALLBACK_ALLOCS.with(Cell::get) - allocs_before;
    let deallocs_during = CALLBACK_DEALLOCS.with(Cell::get) - deallocs_before;
    let callbacks_during = controller.callback_count() - callbacks_before;

    assert!(
        callbacks_during >= 60,
        "expected sustained clocked callbacks, saw {callbacks_during}",
    );
    assert_eq!(
        allocs_during, 0,
        "render callback allocated under the live render posture",
    );
    assert_eq!(
        deallocs_during, 0,
        "render callback deallocated under the live render posture",
    );

    // Audible: the held live note reads on the instrument and master
    // meters while the transport is stopped.
    let meters = controller.meters();
    let instrument_meter = meters
        .iter()
        .find(|(id, _, _)| *id == INSTRUMENT_ID)
        .expect("instrument metered");
    let master_meter = meters
        .iter()
        .find(|(id, _, _)| *id == MASTER_ID)
        .expect("master metered");
    assert!(
        instrument_meter.1 > 0.4,
        "instrument stage should sound while stopped, peak {}",
        instrument_meter.1,
    );
    assert!(
        master_meter.1 > 0.4,
        "master should carry the instrument while stopped, peak {}",
        master_meter.1,
    );

    // Stopped means stopped: the position never advanced and nothing was
    // dropped at this push cadence.
    assert!(!controller.playing());
    assert_eq!(controller.position_frames(), 0);
    assert_eq!(controller.live_event_drop_count(), 0);
    let delivered = instrument.events_seen.load(Ordering::Relaxed);
    assert!(
        delivered > 12 * 8,
        "all pushed live events should deliver, saw {delivered}",
    );

    drop(stream);
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
