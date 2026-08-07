use std::sync::Arc;

use super::{
    FakeMidiInputBackend, MidiEventRing, MidiInputBackend, MidiInputErrorKind, MidiInputEvent,
    MidiPortDescription, MidiSubscriptionState,
};

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
