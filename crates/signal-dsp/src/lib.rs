//! Reusable DSP kernels and transforms for the Signal workspace.

use signal_primitives::Sample;

pub trait DspKernel {
    fn reset(&mut self);
    fn process_in_place(&mut self, block: &mut [Sample]);
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Gain {
    gain: Sample,
}

impl Gain {
    pub fn new(gain: Sample) -> Self {
        Self { gain }
    }

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
