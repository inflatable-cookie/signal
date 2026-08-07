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

mod fake;
mod ring;
mod traits;
mod types;

pub use fake::FakeMidiInputBackend;
pub use ring::MidiEventRing;
pub use traits::{MidiInputBackend, MidiSubscription};
pub use types::{
    MidiInputError, MidiInputErrorKind, MidiInputEvent, MidiPortDescription, MidiSubscriptionState,
};

#[cfg(test)]
mod tests;
