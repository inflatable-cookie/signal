//! Local desktop runtime host.
//!
//! Assembles `signal-runtime`, the local hardware backend (cpal-enumerated
//! output devices), and the plugin stack into a runnable host for use in
//! desktop applications. The top-level type is [`LocalRuntimeHost`], which owns
//! the runtime, the hardware backend, and all plugin adapters. The host
//! drives the audio pump, constructs bridge backends for scanned plugin types,
//! and manages plugin sandbox lifecycle.

#![warn(missing_docs)]

mod host;

pub use host::{
    ensure_default_demo_plugin_override, LocalAudioPumpSummary, LocalAudioStreamState,
    LocalHardwareSummary, LocalRuntimeHost, LocalRuntimeHostSummary,
};
