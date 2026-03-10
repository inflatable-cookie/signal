//! Reusable DSP kernels and transforms for the Signal workspace.
//!
//! The crate is intentionally small today. It exposes a shared trait for
//! in-place block processors plus a basic gain kernel.
//!
//! ```no_run
//! use signal_dsp::{DspKernel, Gain};
//!
//! let mut gain = Gain::new(0.5);
//! let mut block = [0.8, -0.4, 0.2];
//! gain.process_in_place(&mut block);
//!
//! assert_eq!(block, [0.4, -0.2, 0.1]);
//! ```

use signal_primitives::Sample;

/// Common trait for reusable in-place DSP processors.
pub trait DspKernel {
    /// Reset any internal state carried across blocks.
    fn reset(&mut self);

    /// Process a mutable block of samples in place.
    fn process_in_place(&mut self, block: &mut [Sample]);
}

/// A stateless gain kernel that scales every sample in a block.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Gain {
    gain: Sample,
}

impl Gain {
    /// Create a gain kernel from a linear scalar.
    pub fn new(gain: Sample) -> Self {
        Self { gain }
    }

    /// Return the linear gain scalar applied by this kernel.
    pub fn gain(&self) -> Sample {
        self.gain
    }
}

impl DspKernel for Gain {
    fn reset(&mut self) {}

    fn process_in_place(&mut self, block: &mut [Sample]) {
        for sample in block {
            *sample *= self.gain;
        }
    }
}
