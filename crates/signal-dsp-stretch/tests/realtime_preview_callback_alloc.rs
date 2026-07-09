//! Counting-allocator proof for the RealtimePreview callback state contract.
//!
//! This test proves the mono callback-facing DSP path processes repeated
//! quanta without allocating or deallocating while the callback flag is raised.

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
    let mut state = RealtimePreviewCallbackState::new(RealtimePreviewStreamConfig::new(
        SampleRate(48_000),
        1,
        128,
    ))
    .expect("callback state config should validate");
    let input = (0..128)
        .map(|index| (std::f32::consts::TAU * 440.0 * index as f32 / 48_000.0).sin())
        .collect::<Vec<_>>();
    let mut output = vec![0.0; 128];

    let mut last_result = Ok(());
    IN_CALLBACK.store(true, Ordering::SeqCst);
    for _ in 0..64 {
        last_result = state
            .process(&input, &mut output, 128, 1.0)
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
