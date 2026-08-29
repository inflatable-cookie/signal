//! CoreMIDI-backed implementation of Signal's MIDI input contract.
//!
//! Owner-thread pattern lifted from `signal-hardware-cpal`: the MIDI client
//! and input port live and die on one dedicated thread per subscription.
//! That thread also owns the CFRunLoop CoreMIDI delivers device add/remove
//! notifications on (`MIDIClientCreate` binds notifications to the run loop
//! current at creation), so device loss is detected without any extra
//! machinery. The read callback is the real-time path: packet-list walk plus
//! the pure parser in [`crate::parse`], pushing into the caller-owned
//! `MidiEventRing` — no allocation, no locks, drop-and-count on overrun.

#[allow(clippy::module_inception)]
mod backend;
mod cf;
mod subscription;

pub use backend::CoreMidiInputBackend;

#[cfg(test)]
mod tests;
