//! Compiled render plans (preallocated at compile time).

mod compile;
mod inherit;
mod types;

pub use compile::RenderPlan;
pub(crate) use types::{CompiledClip, CompiledNode, CompiledSource};
