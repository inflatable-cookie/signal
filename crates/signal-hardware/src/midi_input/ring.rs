use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};

use super::types::MidiInputEvent;

/// Fixed-capacity lock-free single-producer single-consumer event ring — the
/// MIDI twin of [`signal_primitives::SpscRing`], holding whole
/// [`MidiInputEvent`]s instead of samples.
///
/// The caller owns the ring (typically in an `Arc`) and is the single
/// consumer; the backend's receive thread is the single producer. Both sides
/// are alloc-free and never block: a full ring drops the event on push and
/// counts it in [`MidiEventRing::overrun_events`].
pub struct MidiEventRing {
    storage: Box<[std::cell::UnsafeCell<MidiInputEvent>]>,
    mask: usize,
    /// Total events ever pushed (producer-owned, consumer reads).
    head: AtomicUsize,
    /// Total events ever popped (consumer-owned, producer reads).
    tail: AtomicUsize,
    overrun_events: AtomicU64,
}

// SAFETY: identical SPSC discipline to `signal_primitives::SpscRing` — the
// producer only writes free slots before publishing `head` with Release, the
// consumer only reads published slots observed via Acquire, and no slot is
// ever accessed by both threads at once. `MidiInputEvent` is plain `Copy`
// data with no thread affinity.
unsafe impl Sync for MidiEventRing {}
// SAFETY: see above; moving the ring moves plain data.
unsafe impl Send for MidiEventRing {}

impl MidiEventRing {
    /// Build a ring holding at least `min_capacity` events (rounded up to a
    /// power of two).
    pub fn with_capacity(min_capacity: usize) -> Self {
        let capacity = min_capacity.max(2).next_power_of_two();
        let storage: Box<[std::cell::UnsafeCell<MidiInputEvent>]> = (0..capacity)
            .map(|_| std::cell::UnsafeCell::new(MidiInputEvent::default()))
            .collect();
        Self {
            storage,
            mask: capacity - 1,
            head: AtomicUsize::new(0),
            tail: AtomicUsize::new(0),
            overrun_events: AtomicU64::new(0),
        }
    }

    /// Event capacity of the ring.
    pub fn capacity(&self) -> usize {
        self.storage.len()
    }

    /// Events currently buffered.
    pub fn len(&self) -> usize {
        self.head
            .load(Ordering::Acquire)
            .wrapping_sub(self.tail.load(Ordering::Acquire))
    }

    /// Whether the ring currently holds no events.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Total events dropped because the ring was full at push time.
    pub fn overrun_events(&self) -> u64 {
        self.overrun_events.load(Ordering::Relaxed)
    }

    /// Producer side: push one event, or drop and count it when the ring is
    /// full. Returns whether the event was written. Alloc-free, lock-free,
    /// never blocks — safe on the backend's receive thread.
    pub fn push(&self, event: MidiInputEvent) -> bool {
        let head = self.head.load(Ordering::Relaxed);
        let tail = self.tail.load(Ordering::Acquire);
        if head.wrapping_sub(tail) == self.capacity() {
            self.overrun_events.fetch_add(1, Ordering::Relaxed);
            return false;
        }
        let slot = &self.storage[head & self.mask];
        // SAFETY: the slot at `head` is free (the consumer is at or before
        // `tail`); only the single producer writes it, and it is published to
        // the consumer by the Release store below.
        unsafe { *slot.get() = event };
        self.head.store(head.wrapping_add(1), Ordering::Release);
        true
    }

    /// Consumer side: pop the oldest buffered event, when one exists.
    pub fn pop(&self) -> Option<MidiInputEvent> {
        let tail = self.tail.load(Ordering::Relaxed);
        let head = self.head.load(Ordering::Acquire);
        if head == tail {
            return None;
        }
        let slot = &self.storage[tail & self.mask];
        // SAFETY: the slot at `tail` was published by the producer's Release
        // store observed via the Acquire load above; only the single consumer
        // reads it before freeing it with the Release store below.
        let event = unsafe { *slot.get() };
        self.tail.store(tail.wrapping_add(1), Ordering::Release);
        Some(event)
    }
}
