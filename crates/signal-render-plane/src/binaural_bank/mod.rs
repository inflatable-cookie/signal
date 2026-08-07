//! Binaural voice bank: a [`PluginBlockProcessor`] hosting N one-shot voice
//! slots, each rendered through its own crossfading HRTF convolver — the
//! "option B" per-voice model from
//! `docs/research/binaural-render-plane-integration-v1.md`.
//!
//! Voices live *inside* the processor, so spawning a game sound is a live
//! event, not a plan recompile: `VoiceStart { voice, sound, gain }` begins a
//! preloaded mono sound on a slot, `VoiceParam { HrirIndex }` retargets the
//! slot's ear responses (crossfaded by [`signal_dsp::BinauralConvolver`]) as
//! the source moves, `VoiceStop` silences it, and a voice frees itself when
//! its sound ends. Slot allocation and stealing policy stay with the sender
//! (the game engine) — the bank plays what it is told on the slot it is
//! told.
//!
//! The bank **adds** its stereo output into the stage scratch (composes with
//! whatever the Sum stage already carries). Stereo stages only; any other
//! channel count bypasses. Events apply **sample-accurately**: the block is
//! rendered in segments split at each event's `offset_frames`, so a
//! `VoiceStart` at offset 96 begins exactly there.
//!
//! Real-time safety: sounds and HRIR tables are `Arc`-shared and immutable;
//! all per-voice state is preallocated at construction. `process` takes the
//! state through a `try_lock` (never blocks — contention can only come from
//! a control-thread `reset`, which is rare and tolerates one bypassed
//! block).

mod bank;
mod processor;
#[cfg(test)]
mod tests;
mod types;

pub use bank::BinauralVoiceBank;
pub use types::{BankHrir, BankSound};
