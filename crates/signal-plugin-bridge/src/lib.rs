//! Host-side plugin processing backends (g11.012).
//!
//! The render plane sees one placement-agnostic handle
//! ([`RenderPluginProcessor`] over [`PluginBlockProcessor`]); this crate
//! provides the concrete backends behind it, one per isolation tier
//! ([`signal_plugin::PluginIsolationTier`]):
//!
//! - [`ShmPluginProcessor`] — the **DedicatedSandbox** tier: the plugin runs
//!   in a sandbox child process; each block takes one shared-memory
//!   round-trip with a bounded wait ([`plugin_process_wait_budget`]) and
//!   bypass-on-miss. Full crash isolation: a dead child reads as misses,
//!   never as a blocked callback.
//! - [`InProcessClapProcessor`] / [`InProcessVst3Processor`] — the
//!   **InProcess** tier: the plugin's `process()` is a direct FFI call on
//!   the audio thread. No wait, no round-trip — and honestly NO crash
//!   isolation; that is the documented tradeoff of the tier.
//!
//! The **SharedSandbox** tier (one broker process, many plugins) is modeled
//! in the tier enum but unimplemented in v1; hosts reject it at
//! instantiation with a typed error.

#![warn(missing_docs)]

mod in_process;
mod shm;

pub use in_process::{InProcessClapProcessor, InProcessVst3Processor};
pub use shm::{
    plugin_process_wait_budget, ShmPluginProcessor, PLUGIN_PROCESS_WAIT_BUDGET_MAX_MICROS,
};
