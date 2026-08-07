//! Sandboxed (DedicatedSandbox tier) backend: the parent half of the
//! shared-memory audio block bridge.
//!
//! The child (sandbox broker) created the region at plugin activation and
//! runs its audio thread against it; this side attaches the same mapping,
//! posts input blocks, and bounded-spin-waits for the child's response. A
//! miss (budget exhausted) or a dead child bypasses: the caller's scratch is
//! left untouched and a miss counter increments — the engine callback never
//! blocks past the budget.
//!
//! That budget is a realtime one. Offline drivers switch it via
//! [`signal_render_plane::PluginBlockProcessor::set_offline_waiting`], because an offline render has
//! no output buffer to protect and a bypass there is a wrong render rather than
//! a late block.

mod budget;
mod processor;

#[cfg(test)]
mod tests;

#[allow(unused_imports)] // pre-split public API preserved for crate::shm consumers
pub use budget::{
    plugin_process_wait_budget, PLUGIN_PROCESS_CONSECUTIVE_TIMEOUT_LIMIT,
    PLUGIN_PROCESS_OFFLINE_WAIT_BUDGET, PLUGIN_PROCESS_WAIT_BUDGET_MAX_MICROS,
};
pub use processor::ShmPluginProcessor;
