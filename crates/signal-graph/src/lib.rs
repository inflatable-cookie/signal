//! Graph model and execution semantics for Signal.

use signal_primitives::AudioBuffer;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct NodeId(pub u64);

pub trait AudioNode {
    fn process(&mut self, buffer: &mut AudioBuffer);
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GraphConfig {
    pub block_size: usize,
}

impl Default for GraphConfig {
    fn default() -> Self {
        Self { block_size: 512 }
    }
}
