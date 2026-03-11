use crate::DspKernel;
use signal_primitives::Sample;

/// A stateless gain kernel that scales every sample in a block.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Gain {
    gain: Sample,
    bypassed: bool,
}

impl Gain {
    pub fn new(gain: Sample) -> Self {
        Self {
            gain,
            bypassed: false,
        }
    }

    pub fn gain(&self) -> Sample {
        self.gain
    }

    pub fn set_gain(&mut self, gain: Sample) {
        self.gain = gain;
    }
}

impl DspKernel for Gain {
    fn reset(&mut self) {}

    fn set_bypassed(&mut self, bypassed: bool) {
        self.bypassed = bypassed;
    }

    fn is_bypassed(&self) -> bool {
        self.bypassed
    }

    fn process_in_place(&mut self, block: &mut [Sample]) {
        if self.bypassed {
            return;
        }

        apply_gain_in_place(block, self.gain);
    }
}

pub fn clear_block(block: &mut [Sample]) {
    block.fill(0.0);
}

pub fn apply_gain_in_place(block: &mut [Sample], gain: Sample) {
    for sample in block {
        *sample *= gain;
    }
}

pub fn sum_in_place(destination: &mut [Sample], source: &[Sample]) {
    for (dst, src) in destination.iter_mut().zip(source.iter().copied()) {
        *dst += src;
    }
}

pub fn mix_in_place(destination: &mut [Sample], source: &[Sample], source_gain: Sample) {
    for (dst, src) in destination.iter_mut().zip(source.iter().copied()) {
        *dst += src * source_gain;
    }
}

#[cfg(test)]
mod tests {
    use super::{apply_gain_in_place, clear_block, mix_in_place, sum_in_place, Gain};
    use crate::DspKernel;

    #[test]
    fn gain_scales_audio_when_active() {
        let mut gain = Gain::new(0.5);
        let mut block = [0.8, -0.4, 0.2];
        gain.process_in_place(&mut block);

        assert_eq!(block, [0.4, -0.2, 0.1]);
    }

    #[test]
    fn gain_respects_bypass() {
        let mut gain = Gain::new(0.5);
        let mut block = [0.8, -0.4, 0.2];
        gain.set_bypassed(true);
        gain.process_in_place(&mut block);

        assert_eq!(block, [0.8, -0.4, 0.2]);
    }

    #[test]
    fn block_mix_helpers_are_deterministic() {
        let mut destination = [1.0, 0.5, -0.5];
        let source = [0.5, -0.5, 0.25];

        sum_in_place(&mut destination, &source);
        assert_eq!(destination, [1.5, 0.0, -0.25]);

        mix_in_place(&mut destination, &source, 0.5);
        assert_eq!(destination, [1.75, -0.25, -0.125]);

        apply_gain_in_place(&mut destination, 0.5);
        assert_eq!(destination, [0.875, -0.125, -0.0625]);

        clear_block(&mut destination);
        assert_eq!(destination, [0.0, 0.0, 0.0]);
    }
}
