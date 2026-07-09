//! Counting-allocator proof for the RealtimePreview callback state contract.
//!
//! This test proves the callback-facing DSP path processes repeated quanta
//! without allocating or deallocating while the callback flag is raised.

use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};

use signal_dsp_stretch::{RealtimePreviewCallbackState, RealtimePreviewStreamConfig};
use signal_primitives::SampleRate;

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

#[test]
fn realtime_preview_callback_contract_path_allocates_nothing() {
    let mut mono_state = RealtimePreviewCallbackState::new(RealtimePreviewStreamConfig::new(
        SampleRate(48_000),
        1,
        128,
    ))
    .expect("callback state config should validate");
    let mut stereo_state = RealtimePreviewCallbackState::new(RealtimePreviewStreamConfig::new(
        SampleRate(48_000),
        2,
        128,
    ))
    .expect("callback state config should validate");
    let mono_input = (0..128)
        .map(|index| (std::f32::consts::TAU * 440.0 * index as f32 / 48_000.0).sin())
        .collect::<Vec<_>>();
    let stereo_input = (0..128)
        .flat_map(|index| {
            let left = (std::f32::consts::TAU * 330.0 * index as f32 / 48_000.0).sin();
            let right = (std::f32::consts::TAU * 660.0 * index as f32 / 48_000.0).sin();
            [left, right]
        })
        .collect::<Vec<_>>();
    let mut mono_output = vec![0.0; 128];
    let mut stereo_output = vec![0.0; 128 * 2];

    let mut last_result = Ok(());
    IN_CALLBACK.store(true, Ordering::SeqCst);
    for _ in 0..64 {
        last_result = mono_state
            .process(&mono_input, &mut mono_output, 128, 1.0)
            .map(|_| ())
            .map_err(|error| error);
        if last_result.is_err() {
            break;
        }
        last_result = stereo_state
            .process(&stereo_input, &mut stereo_output, 128, 1.0)
            .map(|_| ())
            .map_err(|error| error);
    }
    IN_CALLBACK.store(false, Ordering::SeqCst);

    assert_eq!(last_result, Ok(()));
    assert_eq!(
        CALLBACK_ALLOCS.load(Ordering::Relaxed),
        0,
        "RealtimePreview callback contract path allocated"
    );
    assert_eq!(
        CALLBACK_DEALLOCS.load(Ordering::Relaxed),
        0,
        "RealtimePreview callback contract path deallocated"
    );
}
