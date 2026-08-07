//! In-process (InProcess tier) backend: direct FFI processing in the host.
//!
//! The plugin's library is dlopen'd IN THE HOST PROCESS and `process()` is
//! called directly on the audio thread — no shared-memory round trip, no
//! wait budget, and honestly NO crash isolation: a crashing plugin takes
//! the host down. That is the documented tradeoff of choosing this tier.

mod au;
mod clap;
mod common;
mod lv2;
mod vst3;

pub use au::InProcessAuProcessor;
pub use clap::InProcessClapProcessor;
pub(crate) use common::convert_block_event;
pub use common::PluginGuiEvent;
pub use lv2::InProcessLv2Processor;
pub use vst3::{InProcessVst3Editor, InProcessVst3Processor};

#[cfg(test)]
mod tests;
