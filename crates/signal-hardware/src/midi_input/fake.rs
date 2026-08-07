use std::sync::atomic::Ordering;
use std::sync::{Arc, Mutex};

use super::ring::MidiEventRing;
use super::traits::{MidiInputBackend, MidiSubscription};
use super::types::{MidiInputError, MidiInputEvent, MidiPortDescription, MidiSubscriptionState};

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
