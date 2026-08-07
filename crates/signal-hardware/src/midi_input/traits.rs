use std::sync::Arc;

use super::ring::MidiEventRing;
use super::types::{MidiInputError, MidiPortDescription, MidiSubscriptionState};

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

    /// Connect `port_id` and start pushing resolved [`super::MidiInputEvent`]s into
    /// the producer side of the caller-owned `ring`. The caller is the
    /// single consumer; the backend is the single producer. Dropping the
    /// returned handle disconnects the port.
    fn subscribe(
        &self,
        port_id: &str,
        ring: Arc<MidiEventRing>,
    ) -> Result<Box<dyn MidiSubscription>, MidiInputError>;
}
