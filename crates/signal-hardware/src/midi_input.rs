//! MIDI input contract: the boundary where the operating system hands live
//! MIDI events to Signal.
//!
//! Mirror of the audio input contract in [`crate::input_stream`]: backends
//! that can read hardware MIDI ports implement [`MidiInputBackend`], the
//! subscription handle is RAII (dropping it closes the port connection), and
//! device loss surfaces by polling [`MidiSubscription::state`] — the exact
//! shape of [`crate::input_stream::InputStreamState::Faulted`] on the audio
//! side.
//!
//! # Real-time contract
//!
//! The backend's receive path runs on an OS-scheduled MIDI thread.
//! Implementations MUST resolve running status, pass real-time messages, and
//! skip SysEx there without allocating, locking, blocking, or performing
//! I/O. Cross-thread hand-off is the caller-owned [`MidiEventRing`] — the
//! MIDI twin of [`signal_primitives::SpscRing`], typed for events instead of
//! samples: bounded copies plus atomics, drop-and-count when full, never
//! block.
//!
//! # Event vocabulary
//!
//! [`MidiInputEvent`] carries a complete, running-status-resolved MIDI 1.0
//! message of at most three bytes: channel voice, system common, and system
//! real-time messages. SysEx is skipped at the backend boundary (recorded
//! runway; nothing downstream consumes it yet). Timestamps are host-clock
//! nanoseconds; the consumer maps them onto the stream clock.

use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

/// One enumerated hardware MIDI input port.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MidiPortDescription {
    /// Stable machine-scoped port identity. Survives unplug/replug of the
    /// same device on the same machine; never portable across machines.
    pub port_id: String,
    /// Port name as reported by the OS.
    pub name: String,
    /// Manufacturer as reported by the OS; empty when the OS reports none.
    pub manufacturer: String,
    /// Whether the backend considers this the default port (backends without
    /// a native default concept flag their first enumerated port).
    pub is_default: bool,
}

/// One complete MIDI 1.0 message, running status already resolved by the
/// backend. `bytes[..len]` is the full wire message (status byte first);
/// trailing bytes are zero. SysEx never appears here.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct MidiInputEvent {
    /// Host-clock timestamp of the message in nanoseconds. The consumer maps
    /// host time onto the audio stream clock; backends never guess frames.
    pub timestamp_host_nanos: u64,
    /// Message bytes, status first; `bytes[len..]` is zero.
    pub bytes: [u8; 3],
    /// Number of valid bytes in `bytes` (1..=3).
    pub len: u8,
}

impl MidiInputEvent {
    /// Build an event from a complete message of 1..=3 bytes.
    ///
    /// # Panics
    ///
    /// Panics when `message` is empty or longer than three bytes; backends
    /// produce only complete resolved messages.
    pub fn new(timestamp_host_nanos: u64, message: &[u8]) -> Self {
        assert!(
            !message.is_empty() && message.len() <= 3,
            "MIDI message must be 1..=3 bytes, got {}",
            message.len()
        );
        let mut bytes = [0u8; 3];
        bytes[..message.len()].copy_from_slice(message);
        Self {
            timestamp_host_nanos,
            bytes,
            len: message.len() as u8,
        }
    }

    /// The valid message bytes (status first).
    pub fn data(&self) -> &[u8] {
        &self.bytes[..usize::from(self.len)]
    }
}

/// Lifecycle state of a MIDI port subscription. Mirror of
/// [`crate::input_stream::InputStreamState`]: device loss is polled off the
/// handle, exactly how the audio side surfaces stream faults.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MidiSubscriptionState {
    /// Subscription is connected and delivering events into the ring.
    Active,
    /// Subscription has been closed (explicitly or by drop).
    Closed,
    /// The subscribed port disappeared mid-session (unplug, OS removal).
    PortLost,
}

/// Classified failure opening or operating a MIDI input backend.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MidiInputErrorKind {
    /// The requested `port_id` does not exist on this machine right now.
    PortNotFound,
    /// The OS MIDI service failed (client/port creation, enumeration).
    Backend,
}

/// Error from a MIDI input backend, mirroring
/// [`crate::input_stream::InputStreamError`] with a classification the
/// consumer's posture mapping needs (missing port vs backend failure).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MidiInputError {
    /// What class of failure this is.
    pub kind: MidiInputErrorKind,
    /// Human-readable description of the failure.
    pub message: String,
}

impl MidiInputError {
    /// Build a [`MidiInputErrorKind::PortNotFound`] error.
    pub fn port_not_found(port_id: &str) -> Self {
        Self {
            kind: MidiInputErrorKind::PortNotFound,
            message: format!("midi input port not found: {port_id}"),
        }
    }

    /// Build a [`MidiInputErrorKind::Backend`] error from any displayable
    /// message.
    pub fn backend(message: impl Into<String>) -> Self {
        Self {
            kind: MidiInputErrorKind::Backend,
            message: message.into(),
        }
    }
}

impl std::fmt::Display for MidiInputError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "midi input error: {}", self.message)
    }
}

impl std::error::Error for MidiInputError {}

/// Fixed-capacity lock-free single-producer single-consumer event ring — the
/// MIDI twin of [`signal_primitives::SpscRing`], holding whole
/// [`MidiInputEvent`]s instead of samples.
///
/// The caller owns the ring (typically in an [`Arc`]) and is the single
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

/// Handle to an open MIDI port subscription. Dropping the handle closes the
/// port connection (RAII, mirroring
/// [`crate::input_stream::InputStreamHandle`]).
pub trait MidiSubscription: Send {
    /// Current lifecycle state; hosts poll this for device loss exactly as
    /// they poll the audio input stream for faults.
    fn state(&self) -> MidiSubscriptionState;
    /// The port this subscription is (or was) connected to.
    fn port_id(&self) -> &str;
    /// Events dropped because the caller's ring was full at delivery time.
    fn overrun_events(&self) -> u64;
    /// Human-readable detail of the most recent backend-reported failure,
    /// when the backend captures one (typically alongside a
    /// [`MidiSubscriptionState::PortLost`] transition). Default: `None` for
    /// backends without error capture.
    fn last_error(&self) -> Option<String> {
        None
    }
}

/// A backend capable of enumerating and subscribing hardware MIDI input
/// ports. Mechanism only: route/selection policy lives in consumers.
pub trait MidiInputBackend {
    /// Enumerate the machine's MIDI input ports right now (hot rescan is
    /// calling this again).
    fn enumerate_ports(&self) -> Result<Vec<MidiPortDescription>, MidiInputError>;

    /// Connect `port_id` and start pushing resolved [`MidiInputEvent`]s into
    /// the producer side of the caller-owned `ring`. The caller is the
    /// single consumer; the backend is the single producer. Dropping the
    /// returned handle disconnects the port.
    fn subscribe(
        &self,
        port_id: &str,
        ring: Arc<MidiEventRing>,
    ) -> Result<Box<dyn MidiSubscription>, MidiInputError>;
}

// ---------------------------------------------------------------------------
// Fake backend for CI — the FakeInputBackend pattern, event-tape flavoured.
// ---------------------------------------------------------------------------

/// One scripted tape: events delivered into the ring when the port is
/// subscribed.
type FakeTape = Vec<MidiInputEvent>;

#[derive(Default)]
struct FakeMidiState {
    ports: Vec<MidiPortDescription>,
    tapes: std::collections::HashMap<String, FakeTape>,
    /// Live subscriptions, so port removal can flip them to `PortLost`.
    subscriptions: Vec<Arc<FakeSubscriptionShared>>,
}

struct FakeSubscriptionShared {
    port_id: String,
    state: std::sync::atomic::AtomicU8,
    ring: Arc<MidiEventRing>,
}

const FAKE_STATE_ACTIVE: u8 = 0;
const FAKE_STATE_CLOSED: u8 = 1;
const FAKE_STATE_PORT_LOST: u8 = 2;

/// Device-less MIDI input backend for CI: scripted event tapes drain into
/// the subscriber's ring, and the port list is mutable at runtime so
/// device-lost paths are testable without hardware. Mirror of
/// [`crate::fake_input::FakeInputBackend`] for the MIDI direction.
#[derive(Default)]
pub struct FakeMidiInputBackend {
    state: Mutex<FakeMidiState>,
}

impl FakeMidiInputBackend {
    /// Construct a fake backend with no ports; add them with
    /// [`FakeMidiInputBackend::add_port`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Construct a fake backend over an initial port list.
    pub fn with_ports(ports: Vec<MidiPortDescription>) -> Self {
        Self {
            state: Mutex::new(FakeMidiState {
                ports,
                ..FakeMidiState::default()
            }),
        }
    }

    /// Add (or re-add, for device-return tests) a port to the inventory.
    pub fn add_port(&self, port: MidiPortDescription) {
        let mut state = self.state.lock().expect("fake midi state");
        state
            .ports
            .retain(|existing| existing.port_id != port.port_id);
        state.ports.push(port);
    }

    /// Remove a port from the inventory and flip its live subscriptions to
    /// [`MidiSubscriptionState::PortLost`] — the scripted unplug.
    pub fn remove_port(&self, port_id: &str) {
        let mut state = self.state.lock().expect("fake midi state");
        state.ports.retain(|port| port.port_id != port_id);
        for subscription in &state.subscriptions {
            if subscription.port_id == port_id
                && subscription.state.load(Ordering::Relaxed) == FAKE_STATE_ACTIVE
            {
                subscription
                    .state
                    .store(FAKE_STATE_PORT_LOST, Ordering::Relaxed);
            }
        }
    }

    /// Script the event tape delivered when `port_id` is next subscribed.
    pub fn set_tape(&self, port_id: &str, tape: Vec<MidiInputEvent>) {
        self.state
            .lock()
            .expect("fake midi state")
            .tapes
            .insert(port_id.to_string(), tape);
    }

    /// Number of subscriptions currently held open (RAII proof: drops
    /// decrement this).
    pub fn active_subscription_count(&self) -> usize {
        let mut state = self.state.lock().expect("fake midi state");
        state
            .subscriptions
            .retain(|subscription| Arc::strong_count(subscription) > 1);
        state
            .subscriptions
            .iter()
            .filter(|subscription| subscription.state.load(Ordering::Relaxed) == FAKE_STATE_ACTIVE)
            .count()
    }
}

impl MidiInputBackend for FakeMidiInputBackend {
    fn enumerate_ports(&self) -> Result<Vec<MidiPortDescription>, MidiInputError> {
        Ok(self.state.lock().expect("fake midi state").ports.clone())
    }

    fn subscribe(
        &self,
        port_id: &str,
        ring: Arc<MidiEventRing>,
    ) -> Result<Box<dyn MidiSubscription>, MidiInputError> {
        let mut state = self.state.lock().expect("fake midi state");
        if !state.ports.iter().any(|port| port.port_id == port_id) {
            return Err(MidiInputError::port_not_found(port_id));
        }
        let shared = Arc::new(FakeSubscriptionShared {
            port_id: port_id.to_string(),
            state: std::sync::atomic::AtomicU8::new(FAKE_STATE_ACTIVE),
            ring: Arc::clone(&ring),
        });
        // Deliver the scripted tape through the real producer path: full
        // rings drop-and-count exactly as a hardware backend would.
        if let Some(tape) = state.tapes.get(port_id) {
            for event in tape {
                shared.ring.push(*event);
            }
        }
        state.subscriptions.push(Arc::clone(&shared));
        Ok(Box::new(FakeMidiSubscription { shared }))
    }
}

struct FakeMidiSubscription {
    shared: Arc<FakeSubscriptionShared>,
}

impl MidiSubscription for FakeMidiSubscription {
    fn state(&self) -> MidiSubscriptionState {
        match self.shared.state.load(Ordering::Relaxed) {
            FAKE_STATE_ACTIVE => MidiSubscriptionState::Active,
            FAKE_STATE_PORT_LOST => MidiSubscriptionState::PortLost,
            _ => MidiSubscriptionState::Closed,
        }
    }

    fn port_id(&self) -> &str {
        &self.shared.port_id
    }

    fn overrun_events(&self) -> u64 {
        self.shared.ring.overrun_events()
    }
}

impl Drop for FakeMidiSubscription {
    fn drop(&mut self) {
        if self.shared.state.load(Ordering::Relaxed) == FAKE_STATE_ACTIVE {
            self.shared
                .state
                .store(FAKE_STATE_CLOSED, Ordering::Relaxed);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn port(id: &str, name: &str, is_default: bool) -> MidiPortDescription {
        MidiPortDescription {
            port_id: id.to_string(),
            name: name.to_string(),
            manufacturer: "Fake Instruments".to_string(),
            is_default,
        }
    }

    #[test]
    fn event_ring_round_trips_in_order() {
        let ring = MidiEventRing::with_capacity(8);
        assert_eq!(ring.capacity(), 8);
        assert!(ring.is_empty());
        let note_on = MidiInputEvent::new(100, &[0x90, 60, 100]);
        let note_off = MidiInputEvent::new(200, &[0x80, 60, 0]);
        assert!(ring.push(note_on));
        assert!(ring.push(note_off));
        assert_eq!(ring.len(), 2);
        assert_eq!(ring.pop(), Some(note_on));
        assert_eq!(ring.pop(), Some(note_off));
        assert_eq!(ring.pop(), None);
        assert_eq!(ring.overrun_events(), 0);
    }

    #[test]
    fn event_ring_drops_and_counts_overruns_when_full() {
        let ring = MidiEventRing::with_capacity(2);
        let event = MidiInputEvent::new(1, &[0x90, 60, 100]);
        assert!(ring.push(event));
        assert!(ring.push(event));
        assert!(!ring.push(event), "full ring drops, never blocks");
        assert_eq!(ring.overrun_events(), 1);
        assert!(ring.pop().is_some());
        assert!(ring.push(event), "freed slot accepts again");
        assert_eq!(ring.overrun_events(), 1);
    }

    #[test]
    fn event_ring_wraps_around_its_storage() {
        let ring = MidiEventRing::with_capacity(4);
        for lap in 0u64..40 {
            let event = MidiInputEvent::new(lap, &[0x90, (lap % 128) as u8, 100]);
            assert!(ring.push(event));
            assert_eq!(ring.pop(), Some(event));
        }
        assert_eq!(ring.overrun_events(), 0);
    }

    #[test]
    fn fake_backend_enumerates_its_scripted_ports() {
        let backend = FakeMidiInputBackend::with_ports(vec![
            port("fake:1", "Fake Keys", true),
            port("fake:2", "Fake Pads", false),
        ]);
        let ports = backend.enumerate_ports().expect("enumerate");
        assert_eq!(ports.len(), 2);
        assert!(ports[0].is_default);
        assert_eq!(ports[1].port_id, "fake:2");
    }

    #[test]
    fn subscribe_drains_the_scripted_tape_through_the_ring() {
        let backend = FakeMidiInputBackend::with_ports(vec![port("fake:1", "Fake Keys", true)]);
        let tape = vec![
            MidiInputEvent::new(1_000, &[0x90, 60, 100]),
            MidiInputEvent::new(2_000, &[0x80, 60, 0]),
            MidiInputEvent::new(3_000, &[0xC0, 5]),
        ];
        backend.set_tape("fake:1", tape.clone());
        let ring = Arc::new(MidiEventRing::with_capacity(16));
        let subscription = backend
            .subscribe("fake:1", Arc::clone(&ring))
            .expect("subscribe");
        assert_eq!(subscription.state(), MidiSubscriptionState::Active);
        assert_eq!(subscription.port_id(), "fake:1");
        let drained: Vec<MidiInputEvent> = std::iter::from_fn(|| ring.pop()).collect();
        assert_eq!(drained, tape, "tape arrives whole and in order");
        assert_eq!(subscription.overrun_events(), 0);
    }

    #[test]
    fn tape_longer_than_the_ring_drops_and_counts() {
        let backend = FakeMidiInputBackend::with_ports(vec![port("fake:1", "Fake Keys", true)]);
        let tape: Vec<MidiInputEvent> = (0..8)
            .map(|index| MidiInputEvent::new(index, &[0x90, index as u8, 1]))
            .collect();
        backend.set_tape("fake:1", tape);
        let ring = Arc::new(MidiEventRing::with_capacity(4));
        let subscription = backend
            .subscribe("fake:1", Arc::clone(&ring))
            .expect("subscribe");
        assert_eq!(ring.len(), 4);
        assert_eq!(subscription.overrun_events(), 4);
    }

    #[test]
    fn subscribing_a_missing_port_is_a_typed_error() {
        let backend = FakeMidiInputBackend::new();
        let ring = Arc::new(MidiEventRing::with_capacity(4));
        let error = backend
            .subscribe("fake:absent", ring)
            .err()
            .expect("missing port must not subscribe");
        assert_eq!(error.kind, MidiInputErrorKind::PortNotFound);
        assert!(error.message.contains("fake:absent"));
    }

    #[test]
    fn dropping_the_subscription_closes_it() {
        let backend = FakeMidiInputBackend::with_ports(vec![port("fake:1", "Fake Keys", true)]);
        let ring = Arc::new(MidiEventRing::with_capacity(4));
        let subscription = backend.subscribe("fake:1", ring).expect("subscribe");
        assert_eq!(backend.active_subscription_count(), 1);
        drop(subscription);
        assert_eq!(backend.active_subscription_count(), 0, "RAII close");
    }

    #[test]
    fn removing_the_port_surfaces_as_port_lost_then_return_is_resubscribable() {
        let backend = FakeMidiInputBackend::with_ports(vec![port("fake:1", "Fake Keys", true)]);
        let ring = Arc::new(MidiEventRing::with_capacity(4));
        let subscription = backend
            .subscribe("fake:1", Arc::clone(&ring))
            .expect("subscribe");
        backend.remove_port("fake:1");
        assert_eq!(subscription.state(), MidiSubscriptionState::PortLost);
        assert!(backend.enumerate_ports().expect("enumerate").is_empty());
        // Device returns: inventory lists it again and a fresh subscription
        // opens (the auto-reopen host pattern builds on exactly this).
        backend.add_port(port("fake:1", "Fake Keys", true));
        let reopened = backend.subscribe("fake:1", ring).expect("resubscribe");
        assert_eq!(reopened.state(), MidiSubscriptionState::Active);
        drop(subscription);
    }

    #[test]
    fn midi_input_event_exposes_only_its_valid_bytes() {
        let program_change = MidiInputEvent::new(0, &[0xC0, 12]);
        assert_eq!(program_change.data(), &[0xC0, 12]);
        assert_eq!(program_change.bytes[2], 0);
        let clock = MidiInputEvent::new(0, &[0xF8]);
        assert_eq!(clock.data(), &[0xF8]);
    }
}
