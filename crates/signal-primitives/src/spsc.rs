//! Lock-free single-producer single-consumer sample ring.
//!
//! Mechanism primitive shared by the capture path (input callback → writer
//! thread, see `signal-hardware`) and the live-input monitor path (input
//! callback → render executor, see `signal-render-plane`). Both `push_slice`
//! and `pop_slice` are alloc-free, lock-free, and never block, so either end
//! may sit on an audio thread.

use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};

/// Fixed-capacity lock-free single-producer single-consumer f32 ring.
///
/// Exactly one thread may call [`SpscRing::push_slice`] (the producer) and
/// exactly one thread may call [`SpscRing::pop_slice`] (the consumer). Both
/// are alloc-free and never block: a full ring drops the excess on push and
/// counts it in [`SpscRing::overrun_samples`].
pub struct SpscRing {
    storage: Box<[std::cell::UnsafeCell<f32>]>,
    mask: usize,
    /// Total samples ever pushed (producer-owned, consumer reads).
    head: AtomicUsize,
    /// Total samples ever popped (consumer-owned, producer reads).
    tail: AtomicUsize,
    overrun_samples: AtomicU64,
}

// SAFETY: the SPSC discipline partitions the storage — the producer only
// writes slots in `[tail + capacity, head)` it has claimed before publishing
// `head` with Release, and the consumer only reads slots in `[tail, head)`
// after observing `head` with Acquire. No slot is ever accessed by both
// threads at once.
unsafe impl Sync for SpscRing {}
// SAFETY: f32 has no thread affinity; moving the ring moves plain data.
unsafe impl Send for SpscRing {}

impl std::fmt::Debug for SpscRing {
    /// Reports shape and counters only. The storage is deliberately not
    /// formatted: reading a slot outside the SPSC discipline would race the
    /// other side, and `capacity`, `len`, and `overrun_samples` already read
    /// through the published atomics.
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("SpscRing")
            .field("capacity", &self.capacity())
            .field("len", &self.len())
            .field("overrun_samples", &self.overrun_samples())
            .finish_non_exhaustive()
    }
}

impl SpscRing {
    /// Build a ring holding at least `min_capacity` samples (rounded up to a
    /// power of two).
    pub fn with_capacity(min_capacity: usize) -> Self {
        let capacity = min_capacity.max(2).next_power_of_two();
        let storage: Box<[std::cell::UnsafeCell<f32>]> = (0..capacity)
            .map(|_| std::cell::UnsafeCell::new(0.0f32))
            .collect();
        Self {
            storage,
            mask: capacity - 1,
            head: AtomicUsize::new(0),
            tail: AtomicUsize::new(0),
            overrun_samples: AtomicU64::new(0),
        }
    }

    /// Sample capacity of the ring.
    pub fn capacity(&self) -> usize {
        self.storage.len()
    }

    /// Samples currently buffered.
    pub fn len(&self) -> usize {
        self.head
            .load(Ordering::Acquire)
            .wrapping_sub(self.tail.load(Ordering::Acquire))
    }

    /// Whether the ring currently holds no samples.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Total samples dropped because the ring was full at push time.
    pub fn overrun_samples(&self) -> u64 {
        self.overrun_samples.load(Ordering::Relaxed)
    }

    /// Producer side: copy as many of `samples` as fit, drop and count the
    /// rest. Returns the number of samples actually written. Alloc-free,
    /// lock-free, never blocks — safe on the audio thread.
    pub fn push_slice(&self, samples: &[f32]) -> usize {
        let head = self.head.load(Ordering::Relaxed);
        let tail = self.tail.load(Ordering::Acquire);
        let free = self.capacity() - head.wrapping_sub(tail);
        let writable = samples.len().min(free);
        for (offset, &sample) in samples[..writable].iter().enumerate() {
            let slot = &self.storage[head.wrapping_add(offset) & self.mask];
            // SAFETY: slots in [head, head + writable) are free (consumer is
            // at or before `tail`); only the single producer writes them, and
            // they are published to the consumer by the Release store below.
            unsafe { *slot.get() = sample };
        }
        self.head
            .store(head.wrapping_add(writable), Ordering::Release);
        let dropped = samples.len() - writable;
        if dropped > 0 {
            self.overrun_samples
                .fetch_add(dropped as u64, Ordering::Relaxed);
        }
        writable
    }

    /// Consumer side: pop up to `out.len()` samples. Returns the number of
    /// samples actually read.
    pub fn pop_slice(&self, out: &mut [f32]) -> usize {
        let tail = self.tail.load(Ordering::Relaxed);
        let head = self.head.load(Ordering::Acquire);
        let available = head.wrapping_sub(tail);
        let readable = out.len().min(available);
        for (offset, sample) in out[..readable].iter_mut().enumerate() {
            let slot = &self.storage[tail.wrapping_add(offset) & self.mask];
            // SAFETY: slots in [tail, tail + readable) were published by the
            // producer's Release store observed via the Acquire load above;
            // only the single consumer reads them before freeing them with
            // the Release store below.
            *sample = unsafe { *slot.get() };
        }
        self.tail
            .store(tail.wrapping_add(readable), Ordering::Release);
        readable
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    #[test]
    fn ring_round_trips_in_order() {
        let ring = SpscRing::with_capacity(8);
        assert_eq!(ring.capacity(), 8);
        assert!(ring.is_empty());
        assert_eq!(ring.push_slice(&[1.0, 2.0, 3.0]), 3);
        assert_eq!(ring.len(), 3);
        let mut out = [0.0f32; 2];
        assert_eq!(ring.pop_slice(&mut out), 2);
        assert_eq!(out, [1.0, 2.0]);
        let mut rest = [0.0f32; 4];
        assert_eq!(ring.pop_slice(&mut rest), 1);
        assert_eq!(rest[0], 3.0);
        assert!(ring.is_empty());
        assert_eq!(ring.overrun_samples(), 0);
    }

    #[test]
    fn ring_wraps_around_its_storage() {
        let ring = SpscRing::with_capacity(4);
        let mut out = [0.0f32; 4];
        // Push/pop repeatedly so the indices lap the storage several times.
        for lap in 0..10 {
            let base = lap as f32 * 3.0;
            assert_eq!(ring.push_slice(&[base, base + 1.0, base + 2.0]), 3);
            assert_eq!(ring.pop_slice(&mut out[..3]), 3);
            assert_eq!(&out[..3], &[base, base + 1.0, base + 2.0]);
        }
        assert_eq!(ring.overrun_samples(), 0);
    }

    #[test]
    fn ring_drops_and_counts_overruns_when_full() {
        let ring = SpscRing::with_capacity(4);
        assert_eq!(ring.push_slice(&[1.0, 2.0, 3.0, 4.0]), 4);
        // Full: everything dropped, counted, push never blocks.
        assert_eq!(ring.push_slice(&[5.0, 6.0]), 0);
        assert_eq!(ring.overrun_samples(), 2);
        // Partial fit: one in, one dropped.
        let mut out = [0.0f32; 1];
        assert_eq!(ring.pop_slice(&mut out), 1);
        assert_eq!(ring.push_slice(&[7.0, 8.0]), 1);
        assert_eq!(ring.overrun_samples(), 3);
        let mut drained = [0.0f32; 4];
        assert_eq!(ring.pop_slice(&mut drained), 4);
        assert_eq!(drained, [2.0, 3.0, 4.0, 7.0]);
    }

    #[test]
    fn ring_survives_concurrent_producer_and_consumer() {
        let ring = Arc::new(SpscRing::with_capacity(1024));
        let producer_ring = Arc::clone(&ring);
        const TOTAL: usize = 100_000;
        let producer = std::thread::spawn(move || {
            let mut next = 0usize;
            while next < TOTAL {
                let batch_end = (next + 64).min(TOTAL);
                let batch: Vec<f32> = (next..batch_end).map(|value| value as f32).collect();
                let mut written = 0;
                while written < batch.len() {
                    written += producer_ring.push_slice(&batch[written..]);
                    std::hint::spin_loop();
                }
                next = batch_end;
            }
        });
        let mut received = Vec::with_capacity(TOTAL);
        let mut chunk = [0.0f32; 97];
        while received.len() < TOTAL {
            let popped = ring.pop_slice(&mut chunk);
            received.extend_from_slice(&chunk[..popped]);
            if popped == 0 {
                std::hint::spin_loop();
            }
        }
        producer.join().expect("producer joins");
        // No sample lost, none reordered. (Overruns DO accrue here: the
        // producer deliberately retries against a full ring, and every
        // rejected sample is counted — that is the drop-and-count contract.)
        for (index, &value) in received.iter().enumerate() {
            assert_eq!(value, index as f32, "sample {index} out of order");
        }
    }
}
