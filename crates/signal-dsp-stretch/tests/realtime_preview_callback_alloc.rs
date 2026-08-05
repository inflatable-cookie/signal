//! Counting-allocator proof for the RealtimePreview callback state contract.
//!
//! This test proves the callback-facing DSP path processes repeated quanta
//! without allocating or deallocating while the callback flag is raised.

use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::Cell;

use signal_dsp_stretch::{RealtimePreviewCallbackState, RealtimePreviewStreamConfig};
use signal_primitives::SampleRate;

// Thread-scoped: the allocator hook is process-global, so a second test in
// this binary would otherwise be counted against whichever test is measuring.
thread_local! {
    static IN_CALLBACK: Cell<bool> = const { Cell::new(false) };
    static CALLBACK_ALLOCS: Cell<u64> = const { Cell::new(0) };
    static CALLBACK_DEALLOCS: Cell<u64> = const { Cell::new(0) };
}

struct CountingAllocator;

unsafe impl GlobalAlloc for CountingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        if IN_CALLBACK.with(Cell::get) {
            CALLBACK_ALLOCS.with(|value| value.set(value.get().saturating_add(1)));
        }
        unsafe { System.alloc(layout) }
    }

    unsafe fn dealloc(&self, pointer: *mut u8, layout: Layout) {
        if IN_CALLBACK.with(Cell::get) {
            CALLBACK_DEALLOCS.with(|value| value.set(value.get().saturating_add(1)));
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
    IN_CALLBACK.with(|flag| flag.set(true));
    for iteration in 0..64 {
        let mono_ratio = if iteration < 32 { 1.0 } else { 1.25 };
        let stereo_ratio = if iteration < 32 { 1.0 } else { 0.75 };
        last_result = mono_state
            .advance_scheduled_source_projection(128, mono_ratio)
            .map(|_| ());
        if last_result.is_err() {
            break;
        }
        last_result = mono_state
            .process(&mono_input, &mut mono_output, 128, mono_ratio)
            .map(|_| ());
        if last_result.is_err() {
            break;
        }
        last_result = stereo_state
            .advance_scheduled_source_projection(128, stereo_ratio)
            .map(|_| ());
        if last_result.is_err() {
            break;
        }
        last_result = stereo_state
            .process(&stereo_input, &mut stereo_output, 128, stereo_ratio)
            .map(|_| ());
    }
    IN_CALLBACK.with(|flag| flag.set(false));

    assert_eq!(last_result, Ok(()));
    assert_eq!(
        CALLBACK_ALLOCS.with(Cell::get),
        0,
        "RealtimePreview callback contract path allocated"
    );
    assert_eq!(
        CALLBACK_DEALLOCS.with(Cell::get),
        0,
        "RealtimePreview callback contract path deallocated"
    );
}
