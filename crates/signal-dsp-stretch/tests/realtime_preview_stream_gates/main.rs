//! `g10.040` Batch 40.3 evidence gates, in the blocking order Batch 40.2 froze.
//!
//! `G1` allocation-free callback execution
//! `G2` bounded work across the frozen ratio range
//! `G3` sustained ratios either side of `1.0` produce continuous output with no
//!      dropped source
//! `G4` reported source consumption equals actual kernel consumption
//! `G5` underrun reports silence rather than a normal-looking block
//! `G6` dynamic-ratio changes land inside the frozen alignment tolerance

use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::Cell;
use std::sync::atomic::{AtomicBool, Ordering};

use signal_dsp_stretch::{
    RealtimePreviewStreamConfig, RealtimePreviewStreamError, RealtimePreviewStreamState,
    REALTIME_PREVIEW_STREAM_MAX_RATIO, REALTIME_PREVIEW_STREAM_MIN_RATIO,
};
use signal_primitives::SampleRate;

thread_local! {
    static IN_CALLBACK: Cell<bool> = const { Cell::new(false) };
    static CALLBACK_ALLOCS: Cell<usize> = const { Cell::new(0) };
}
static ARMED: AtomicBool = AtomicBool::new(false);

struct CountingAllocator;
unsafe impl GlobalAlloc for CountingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        if ARMED.load(Ordering::Relaxed) {
            IN_CALLBACK.with(|flag| {
                if flag.get() {
                    CALLBACK_ALLOCS.with(|count| count.set(count.get() + 1));
                }
            });
        }
        unsafe { System.alloc(layout) }
    }
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        unsafe { System.dealloc(ptr, layout) }
    }
}
#[global_allocator]
static ALLOC: CountingAllocator = CountingAllocator;

const RATE: u32 = 48_000;
const BLOCK: usize = 128;
const CHANNELS: usize = 2;

fn state(block: usize) -> RealtimePreviewStreamState {
    RealtimePreviewStreamState::new(RealtimePreviewStreamConfig::new(
        SampleRate(RATE),
        CHANNELS,
        block,
    ))
    .expect("preview stream should plan")
}

/// A 220 Hz tone with a click every 250 ms, interleaved stereo.
fn source(frames: usize) -> Vec<f32> {
    (0..frames)
        .flat_map(|index| {
            let phase = std::f32::consts::TAU * 220.0 * index as f32 / RATE as f32;
            let tone = 0.4 * phase.sin();
            let click = if index % (RATE as usize / 4) < 32 {
                0.5
            } else {
                0.0
            };
            [tone + click, tone + click]
        })
        .collect()
}

/// Keep the producer ahead of demand, the way a non-realtime filler would.
fn top_up(state: &mut RealtimePreviewStreamState, source: &[f32], cursor: &mut usize) {
    let demand = state.source_demand_frame() as usize;
    if demand <= *cursor {
        return;
    }
    let wanted = (demand - *cursor).min(source.len() / CHANNELS - *cursor);
    if wanted == 0 {
        return;
    }
    let slice = &source[*cursor * CHANNELS..(*cursor + wanted) * CHANNELS];
    let accepted = state.push_source(slice);
    *cursor += accepted;
}
fn sweep(frames: usize) -> Vec<f32> {
    let mut phase = 0.0f32;
    (0..frames)
        .flat_map(|index| {
            let progress = index as f32 / frames as f32;
            phase += std::f32::consts::TAU * (200.0 + 2800.0 * progress) / RATE as f32;
            let sample = 0.4 * phase.sin();
            [sample, sample]
        })
        .collect()
}

/// Zero-crossing frequency estimate per chunk of rendered output.
fn frequency_per_chunk(rendered: &[f32], chunk: usize) -> Vec<f32> {
    let frames = rendered.len() / CHANNELS;
    let mut estimates = Vec::new();
    let mut start = 0usize;
    while start + chunk <= frames {
        let mut crossings = 0usize;
        for frame in start..start + chunk - 1 {
            let current = rendered[frame * CHANNELS];
            let next = rendered[(frame + 1) * CHANNELS];
            if (current >= 0.0) != (next >= 0.0) {
                crossings += 1;
            }
        }
        estimates.push(crossings as f32 * RATE as f32 / (2.0 * chunk as f32));
        start += chunk;
    }
    estimates
}

fn rms(samples: &[f32]) -> f64 {
    (samples
        .iter()
        .map(|s| (*s as f64) * (*s as f64))
        .sum::<f64>()
        / samples.len().max(1) as f64)
        .sqrt()
}

/// Best correlation over a symmetric lag search, so a constant offset between
/// two renders is not mistaken for divergence.
fn correlation(a: &[f32], b: &[f32]) -> f64 {
    let frames = (a.len() / CHANNELS).min(b.len() / CHANNELS);
    let mut best = -1.0f64;
    for signed in -6000i64..6000 {
        let (offset_a, offset_b) = if signed >= 0 {
            (0usize, signed as usize)
        } else {
            ((-signed) as usize, 0usize)
        };
        if offset_a.max(offset_b) + 4096 >= frames {
            continue;
        }
        let len = frames - offset_a.max(offset_b);
        let (mut num, mut da, mut db) = (0.0f64, 0.0f64, 0.0f64);
        for index in 0..len {
            let x = a[(index + offset_a) * CHANNELS] as f64;
            let y = b[(index + offset_b) * CHANNELS] as f64;
            num += x * y;
            da += x * x;
            db += y * y;
        }
        let denom = (da * db).sqrt();
        if denom > 0.0 {
            best = best.max(num / denom);
        }
    }
    best
}

mod gates;
