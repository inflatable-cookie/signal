use std::sync::Arc;

use signal_hardware::MidiEventRing;

use super::backend::CoreMidiInputBackend;
use super::cf::{parse_port_id, PORT_ID_PREFIX};
use signal_hardware::MidiInputBackend;

/// Smoke test against the real CoreMIDI service; skips quietly when the
/// machine has no MIDI sources (CI) — the cpal enumeration-test posture.
#[test]
fn enumerates_midi_sources_when_present() {
    let backend = CoreMidiInputBackend::new();
    let ports = backend.enumerate_ports().expect("enumerate midi sources");
    if ports.is_empty() {
        eprintln!("no midi sources; skipping");
        return;
    }
    assert!(ports[0].is_default);
    assert_eq!(ports.iter().filter(|port| port.is_default).count(), 1);
    for port in &ports {
        assert!(port.port_id.starts_with(PORT_ID_PREFIX), "{}", port.port_id);
        assert!(
            parse_port_id(&port.port_id).is_some(),
            "port id round-trips: {}",
            port.port_id
        );
        assert!(!port.name.is_empty());
    }
}

#[test]
fn subscribing_a_malformed_port_id_is_port_not_found() {
    let backend = CoreMidiInputBackend::new();
    let ring = Arc::new(MidiEventRing::with_capacity(16));
    let error = backend
        .subscribe("not-a-coremidi-id", ring)
        .err()
        .expect("malformed id must not subscribe");
    assert_eq!(
        error.kind,
        signal_hardware::MidiInputErrorKind::PortNotFound
    );
}

#[test]
fn subscribing_an_absent_unique_id_is_port_not_found() {
    // Unique ids are i32; this one is overwhelmingly unlikely to exist.
    let backend = CoreMidiInputBackend::new();
    let ring = Arc::new(MidiEventRing::with_capacity(16));
    let error = backend
        .subscribe("coremidi:2147480001", ring)
        .err()
        .expect("absent source must not subscribe");
    assert_eq!(
        error.kind,
        signal_hardware::MidiInputErrorKind::PortNotFound
    );
}
