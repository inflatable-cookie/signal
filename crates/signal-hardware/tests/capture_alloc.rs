//! Counting-allocator proof that the capture callback path is alloc-free.
//!
//! A global counting allocator tracks every alloc/dealloc that happens while
//! the in-callback flag is raised. The fake input backend drives the exact
//! callback body `CaptureSession` installs — a single `SpscRing::push_slice`
//! — with the flag raised around it, while a consumer thread drains the ring
//! concurrently (flag down). Zero counted allocations proves the RT path.

use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;

use signal_hardware::{FakeInputBackend, InputStreamBackend, InputStreamSpec, SpscRing};

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
fn capture_callback_path_allocates_nothing() {
    let ring = Arc::new(SpscRing::with_capacity(48_000));
    let callback_ring = Arc::clone(&ring);
    let blocks = Arc::new(AtomicU64::new(0));
    let callback_blocks = Arc::clone(&blocks);

    // Consumer thread drains like the capture writer (flag stays down here;
    // the writer is explicitly non-RT and allowed to allocate).
    let drain_ring = Arc::clone(&ring);
    let drain_stop = Arc::new(AtomicBool::new(false));
    let drain_stop_flag = Arc::clone(&drain_stop);
    let drainer = std::thread::spawn(move || {
        let mut chunk = vec![0.0f32; 4_096];
        while !drain_stop_flag.load(Ordering::Relaxed) {
            if drain_ring.pop_slice(&mut chunk) == 0 {
                std::thread::sleep(Duration::from_millis(1));
            }
        }
    });

    let backend = FakeInputBackend::new();
    let stream = backend
        .open_input_stream(
            InputStreamSpec {
                sample_rate_hz: 48_000,
                channels: 2,
                buffer_frames: Some(256),
            },
            Box::new(move |frames| {
                // Exactly the CaptureSession callback body, measured.
                IN_CALLBACK.store(true, Ordering::SeqCst);
                callback_ring.push_slice(frames);
                IN_CALLBACK.store(false, Ordering::SeqCst);
                callback_blocks.fetch_add(1, Ordering::Relaxed);
            }),
        )
        .expect("open fake input stream");

    std::thread::sleep(Duration::from_millis(500));
    drop(stream);
    drain_stop.store(true, Ordering::Relaxed);
    drainer.join().expect("drainer joins");

    let observed_blocks = blocks.load(Ordering::Relaxed);
    assert!(
        observed_blocks > 10,
        "callback barely ran: {observed_blocks}"
    );
    assert_eq!(
        CALLBACK_ALLOCS.load(Ordering::Relaxed),
        0,
        "capture callback allocated"
    );
    assert_eq!(
        CALLBACK_DEALLOCS.load(Ordering::Relaxed),
        0,
        "capture callback deallocated"
    );
    assert_eq!(ring.overrun_samples(), 0);
}
