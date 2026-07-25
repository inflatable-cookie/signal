use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Mutex;

static CURRENT_LIVE_BYTES: AtomicUsize = AtomicUsize::new(0);
static PEAK_LIVE_BYTES: AtomicUsize = AtomicUsize::new(0);
static MEASURING: AtomicBool = AtomicBool::new(false);
static MEASUREMENT_LOCK: Mutex<()> = Mutex::new(());

pub(crate) struct TrackingAllocator;

unsafe impl GlobalAlloc for TrackingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let pointer = unsafe { System.alloc(layout) };
        if !pointer.is_null() {
            record_allocation(layout.size());
        }
        pointer
    }

    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        let pointer = unsafe { System.alloc_zeroed(layout) };
        if !pointer.is_null() {
            record_allocation(layout.size());
        }
        pointer
    }

    unsafe fn dealloc(&self, pointer: *mut u8, layout: Layout) {
        record_deallocation(layout.size());
        unsafe { System.dealloc(pointer, layout) };
    }

    unsafe fn realloc(&self, pointer: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        let new_pointer = unsafe { System.realloc(pointer, layout, new_size) };
        if !new_pointer.is_null() {
            if new_size >= layout.size() {
                record_allocation(new_size - layout.size());
            } else {
                record_deallocation(layout.size() - new_size);
            }
        }
        new_pointer
    }
}

#[global_allocator]
static ALLOCATOR: TrackingAllocator = TrackingAllocator;

/// Measured peak live-heap growth while an operation ran.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct PeakLiveHeapMeasurement {
    pub(crate) baseline_live_bytes: usize,
    pub(crate) peak_live_bytes: usize,
    pub(crate) peak_growth_bytes: usize,
}

/// Run one control-plane operation while measuring live heap use.
pub(crate) fn measure_peak_live_heap<T>(
    operation: impl FnOnce() -> T,
) -> (T, PeakLiveHeapMeasurement) {
    let _lock = MEASUREMENT_LOCK
        .lock()
        .expect("peak live-heap measurement lock poisoned");
    let baseline_live_bytes = CURRENT_LIVE_BYTES.load(Ordering::SeqCst);
    PEAK_LIVE_BYTES.store(baseline_live_bytes, Ordering::SeqCst);
    MEASURING.store(true, Ordering::SeqCst);
    let guard = MeasurementGuard;
    let result = operation();
    let peak_live_bytes = PEAK_LIVE_BYTES.load(Ordering::SeqCst);
    drop(guard);
    (
        result,
        PeakLiveHeapMeasurement {
            baseline_live_bytes,
            peak_live_bytes,
            peak_growth_bytes: peak_live_bytes.saturating_sub(baseline_live_bytes),
        },
    )
}

struct MeasurementGuard;

impl Drop for MeasurementGuard {
    fn drop(&mut self) {
        MEASURING.store(false, Ordering::SeqCst);
    }
}

fn record_allocation(bytes: usize) {
    let current = CURRENT_LIVE_BYTES
        .fetch_add(bytes, Ordering::Relaxed)
        .saturating_add(bytes);
    if !MEASURING.load(Ordering::Relaxed) {
        return;
    }
    let mut peak = PEAK_LIVE_BYTES.load(Ordering::Relaxed);
    while current > peak {
        match PEAK_LIVE_BYTES.compare_exchange_weak(
            peak,
            current,
            Ordering::Relaxed,
            Ordering::Relaxed,
        ) {
            Ok(_) => break,
            Err(observed) => peak = observed,
        }
    }
}

fn record_deallocation(bytes: usize) {
    CURRENT_LIVE_BYTES.fetch_sub(bytes, Ordering::Relaxed);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn peak_live_heap_measurement_counts_retained_render_working_memory() {
        let (buffer, measurement) = measure_peak_live_heap(|| vec![0_u8; 16_384]);

        assert_eq!(buffer.len(), 16_384);
        assert!(measurement.peak_growth_bytes >= buffer.len());
        assert!(measurement.peak_live_bytes >= measurement.baseline_live_bytes);
    }
}
