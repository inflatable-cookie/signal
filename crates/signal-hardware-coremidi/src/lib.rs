//! CoreMIDI-backed hardware MIDI input for Signal on macOS.
//!
//! Implements `signal-hardware`'s [`MidiInputBackend`] contract over
//! handwritten CoreMIDI FFI — the same posture as the AU/VST3 host
//! adapters: no binding crate, only the small surface this backend needs
//! (client, input port, source enumeration, packet-list parse,
//! add/remove notifications).
//!
//! The packet-list parser lives in [`parse`] as a pure, platform-independent
//! function so its running-status/SysEx/real-time rules are unit-tested on
//! every platform; the CoreMIDI client itself compiles on macOS only.
//!
//! [`MidiInputBackend`]: signal_hardware::MidiInputBackend

#![warn(missing_docs)]

pub mod parse;

#[cfg(target_os = "macos")]
mod ffi;

#[cfg(target_os = "macos")]
mod backend;

#[cfg(target_os = "macos")]
pub use backend::CoreMidiInputBackend;
