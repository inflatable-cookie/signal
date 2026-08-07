use std::sync::{mpsc, Arc};

use signal_hardware::{
    MidiEventRing, MidiInputBackend, MidiInputError, MidiPortDescription, MidiSubscription,
};

use crate::ffi;

use super::cf::{integer_property, parse_port_id, string_property, PORT_ID_PREFIX};
use super::subscription::{owner_thread, CoreMidiSubscription, SubscriptionShared};

/// MIDI input backend over the machine's CoreMIDI sources.
#[derive(Debug, Default)]
pub struct CoreMidiInputBackend;

impl CoreMidiInputBackend {
    /// Construct a backend over the machine's CoreMIDI sources.
    pub fn new() -> Self {
        Self
    }
}

impl MidiInputBackend for CoreMidiInputBackend {
    fn enumerate_ports(&self) -> Result<Vec<MidiPortDescription>, MidiInputError> {
        let mut ports = Vec::new();
        let source_count = unsafe { ffi::MIDIGetNumberOfSources() };
        for index in 0..source_count {
            let source = unsafe { ffi::MIDIGetSource(index) };
            if source == 0 {
                continue;
            }
            // Unique id is the identity anchor; a source without one cannot
            // be re-found later, so it is not offered.
            let Some(unique_id) = integer_property(source, unsafe { ffi::kMIDIPropertyUniqueID })
            else {
                continue;
            };
            let name = string_property(source, unsafe { ffi::kMIDIPropertyDisplayName })
                .or_else(|| string_property(source, unsafe { ffi::kMIDIPropertyName }))
                .unwrap_or_else(|| format!("MIDI Source {}", index + 1));
            let manufacturer = string_property(source, unsafe { ffi::kMIDIPropertyManufacturer })
                .unwrap_or_default();
            ports.push(MidiPortDescription {
                port_id: format!("{PORT_ID_PREFIX}{unique_id}"),
                name,
                manufacturer,
                // CoreMIDI has no default-source concept; the first
                // enumerated source is flagged, per the contract docs.
                is_default: ports.is_empty(),
            });
        }
        Ok(ports)
    }

    fn subscribe(
        &self,
        port_id: &str,
        ring: Arc<MidiEventRing>,
    ) -> Result<Box<dyn MidiSubscription>, MidiInputError> {
        let Some(unique_id) = parse_port_id(port_id) else {
            return Err(MidiInputError::port_not_found(port_id));
        };
        let shared = Arc::new(SubscriptionShared::new_active());
        let owner_shared = Arc::clone(&shared);
        let owner_ring = Arc::clone(&ring);
        let (ready_tx, ready_rx) = mpsc::channel::<Result<(), MidiInputError>>();

        // Client, port, and notification run loop live and die on this
        // thread — the cpal stream-thread ownership pattern.
        let owner = std::thread::Builder::new()
            .name("signal-midi-input".to_string())
            .spawn(move || owner_thread(unique_id, owner_ring, owner_shared, ready_tx))
            .map_err(|error| MidiInputError::backend(format!("spawn midi thread: {error}")))?;

        match ready_rx.recv() {
            Ok(Ok(())) => Ok(Box::new(CoreMidiSubscription::new(
                port_id.to_string(),
                ring,
                shared,
                owner,
            ))),
            Ok(Err(error)) => {
                let _ = owner.join();
                Err(error)
            }
            Err(_) => {
                let _ = owner.join();
                Err(MidiInputError::backend(
                    "midi thread exited before reporting",
                ))
            }
        }
    }
}
