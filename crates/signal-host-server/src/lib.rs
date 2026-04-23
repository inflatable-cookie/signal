//! Headless runtime host.
//!
//! Assembles `signal-runtime` and the plugin stack without audio hardware for
//! server-side or test use. The top-level type is [`ServerRuntimeHost`], which
//! owns the runtime and all plugin adapters. Unlike the local host there is no
//! CoreAudio backend or audio pump — processing is driven programmatically
//! through the [`RuntimeSupervisorApi`] trait.

#![warn(missing_docs)]


/// Internal host implementation module.
pub mod host;

pub use host::{ensure_default_demo_plugin_override, ServerRuntimeHost, ServerRuntimeHostSummary};
pub use signal_runtime::RecoveryRestartIntent;
