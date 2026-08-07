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
