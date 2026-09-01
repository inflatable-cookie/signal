//! Cross-thread parameter-change queue for hosted plugin instances
//! (g12.023 wire param-set phase 2).
//!
//! One bounded lock-free ring per hosted instance carries pending
//! `(parameter_id, value)` writes from the control/lifecycle thread to the
//! audio thread, which drains them at the top of each processed block
//! (block-boundary application — sample-accuracy posture v1; intra-block
//! offsets arrive with automation depth). The value's domain is the
//! FORMAT's set domain: plain for CLAP param events / AU / LV2 control
//! ports, normalized 0..1 for VST3 — the format adapters convert before
//! pushing, so the drain side applies verbatim.
//!
//! Threading contract: exactly one producer at a time (the owning
//! instance/processor serializes `push` behind its lifecycle lock) and
//! exactly one consumer (the audio thread's process session). Both sides
//! are alloc-free; the consumer's drain writes into a caller-preallocated
//! scratch and coalesces repeated writes to the same parameter (last write
//! wins — a block applies each parameter's newest value once).

use std::sync::atomic::{AtomicU32, AtomicU64, AtomicUsize, Ordering};

/// Ring capacity (power of two). Also the largest number of DISTINCT
/// parameters one drain can yield, so drain scratch preallocated at this
/// capacity never grows on the audio thread.
pub const PLUGIN_PARAM_CHANGE_CAPACITY: usize = 256;

/// One pending parameter change.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PluginParamChange {
    /// Format-native parameter id (CLAP/VST3 param id, AU parameter id,
    /// LV2 control-port index).
    pub parameter_id: u32,
    /// Value in the format's set domain (see module docs).
    pub value: f64,
}

struct Slot {
    parameter_id: AtomicU32,
    value_bits: AtomicU64,
}

/// Bounded single-producer/single-consumer parameter-change ring.
pub struct PluginParamChangeQueue {
    slots: Box<[Slot]>,
    /// Consumer cursor (count of entries consumed).
    head: AtomicUsize,
    /// Producer cursor (count of entries published).
    tail: AtomicUsize,
}

impl std::fmt::Debug for PluginParamChangeQueue {
    /// Reports capacity and the published cursors only. The slots are
    /// deliberately not formatted: a slot's parameter id and value are two
    /// separate atomics that are only consistent for the consumer between the
    /// cursors, so formatting them would report a torn pair.
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("PluginParamChangeQueue")
            .field("capacity", &self.slots.len())
            .field("head", &self.head.load(Ordering::Acquire))
            .field("tail", &self.tail.load(Ordering::Acquire))
            .finish_non_exhaustive()
    }
}

impl Default for PluginParamChangeQueue {
    fn default() -> Self {
        Self::new()
    }
}

impl PluginParamChangeQueue {
    /// Build an empty queue at [`PLUGIN_PARAM_CHANGE_CAPACITY`].
    pub fn new() -> Self {
        let slots = (0..PLUGIN_PARAM_CHANGE_CAPACITY)
            .map(|_| Slot {
                parameter_id: AtomicU32::new(0),
                value_bits: AtomicU64::new(0),
            })
            .collect::<Vec<_>>()
            .into_boxed_slice();
        Self {
            slots,
            head: AtomicUsize::new(0),
            tail: AtomicUsize::new(0),
        }
    }

    /// Publish one change. Returns `false` when the ring is full (the
    /// caller surfaces a typed error instead of blocking or dropping
    /// silently).
    pub fn push(&self, parameter_id: u32, value: f64) -> bool {
        let tail = self.tail.load(Ordering::Relaxed);
        let head = self.head.load(Ordering::Acquire);
        if tail.wrapping_sub(head) >= self.slots.len() {
            return false;
        }
        let slot = &self.slots[tail % self.slots.len()];
        slot.parameter_id.store(parameter_id, Ordering::Relaxed);
        slot.value_bits.store(value.to_bits(), Ordering::Relaxed);
        self.tail.store(tail.wrapping_add(1), Ordering::Release);
        true
    }

    /// Drain every pending change into `scratch`, coalescing repeated
    /// writes to the same parameter (last write wins). `scratch` is
    /// cleared first and never grows past [`PLUGIN_PARAM_CHANGE_CAPACITY`]
    /// entries, so a scratch preallocated at that capacity keeps this
    /// call alloc-free. Consumer side only.
    pub fn drain_coalesced(&self, scratch: &mut Vec<PluginParamChange>) {
        scratch.clear();
        let head = self.head.load(Ordering::Relaxed);
        let tail = self.tail.load(Ordering::Acquire);
        let mut cursor = head;
        while cursor != tail {
            let slot = &self.slots[cursor % self.slots.len()];
            let change = PluginParamChange {
                parameter_id: slot.parameter_id.load(Ordering::Relaxed),
                value: f64::from_bits(slot.value_bits.load(Ordering::Relaxed)),
            };
            match scratch
                .iter_mut()
                .find(|pending| pending.parameter_id == change.parameter_id)
            {
                Some(pending) => pending.value = change.value,
                None => scratch.push(change),
            }
            cursor = cursor.wrapping_add(1);
        }
        self.head.store(tail, Ordering::Release);
    }

    /// Whether any change is pending (cheap peek for the audio thread's
    /// fast path).
    pub fn is_empty(&self) -> bool {
        self.head.load(Ordering::Relaxed) == self.tail.load(Ordering::Relaxed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn drains_pushed_changes_in_order() {
        let queue = PluginParamChangeQueue::new();
        assert!(queue.is_empty());
        assert!(queue.push(1, 0.25));
        assert!(queue.push(2, 0.75));
        let mut scratch = Vec::with_capacity(PLUGIN_PARAM_CHANGE_CAPACITY);
        queue.drain_coalesced(&mut scratch);
        assert_eq!(
            scratch,
            vec![
                PluginParamChange {
                    parameter_id: 1,
                    value: 0.25
                },
                PluginParamChange {
                    parameter_id: 2,
                    value: 0.75
                },
            ]
        );
        assert!(queue.is_empty());
    }

    #[test]
    fn coalesces_repeated_writes_last_wins() {
        let queue = PluginParamChangeQueue::new();
        for step in 0..100 {
            assert!(queue.push(7, step as f64 / 100.0));
        }
        assert!(queue.push(9, 0.5));
        let mut scratch = Vec::with_capacity(PLUGIN_PARAM_CHANGE_CAPACITY);
        queue.drain_coalesced(&mut scratch);
        assert_eq!(scratch.len(), 2, "100 writes to one id coalesce to one");
        assert_eq!(scratch[0].parameter_id, 7);
        assert!((scratch[0].value - 0.99).abs() < 1e-12, "last write wins");
        assert_eq!(scratch[1].parameter_id, 9);
    }

    #[test]
    fn push_reports_full_instead_of_overwriting() {
        let queue = PluginParamChangeQueue::new();
        for index in 0..PLUGIN_PARAM_CHANGE_CAPACITY {
            assert!(queue.push(index as u32, 0.0));
        }
        assert!(!queue.push(9999, 1.0), "full ring must refuse");
        let mut scratch = Vec::with_capacity(PLUGIN_PARAM_CHANGE_CAPACITY);
        queue.drain_coalesced(&mut scratch);
        assert_eq!(scratch.len(), PLUGIN_PARAM_CHANGE_CAPACITY);
        // Space reclaimed after the drain.
        assert!(queue.push(9999, 1.0));
    }

    #[test]
    fn drain_survives_interleaved_producer_thread() {
        use std::sync::Arc;
        let queue = Arc::new(PluginParamChangeQueue::new());
        let producer_queue = Arc::clone(&queue);
        let producer = std::thread::spawn(move || {
            for step in 0..10_000u32 {
                while !producer_queue.push(step % 8, step as f64) {
                    std::thread::yield_now();
                }
            }
        });
        let mut scratch = Vec::with_capacity(PLUGIN_PARAM_CHANGE_CAPACITY);
        let mut last_seen = [-1.0f64; 8];
        while !producer.is_finished() || !queue.is_empty() {
            queue.drain_coalesced(&mut scratch);
            for change in &scratch {
                let slot = change.parameter_id as usize;
                assert!(
                    change.value > last_seen[slot],
                    "per-id values must arrive monotonically (id {slot}: {} then {})",
                    last_seen[slot],
                    change.value,
                );
                last_seen[slot] = change.value;
            }
        }
        producer.join().expect("producer joins");
    }
}
