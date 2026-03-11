use crate::{flush_denormal, DspKernel};
use signal_primitives::{FrequencyHz, Sample, SampleRate};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct OnePoleLowPass {
    sample_rate: SampleRate,
    cutoff_hz: FrequencyHz,
    alpha: Sample,
    state: Sample,
    bypassed: bool,
}

impl OnePoleLowPass {
    pub fn new(sample_rate: SampleRate, cutoff_hz: FrequencyHz) -> Self {
        let mut filter = Self {
            sample_rate,
            cutoff_hz,
            alpha: 1.0,
            state: 0.0,
            bypassed: false,
        };
        filter.update_alpha();
        filter
    }

    pub fn cutoff_hz(&self) -> FrequencyHz {
        self.cutoff_hz
    }

    pub fn set_cutoff_hz(&mut self, cutoff_hz: FrequencyHz) {
        self.cutoff_hz = cutoff_hz;
        self.update_alpha();
    }

    pub fn process_sample(&mut self, input: Sample) -> Sample {
        if self.bypassed {
            self.state = input;
            return input;
        }

        self.state = flush_denormal(self.state + self.alpha * (input - self.state));
        self.state
    }

    fn update_alpha(&mut self) {
        if self.sample_rate.0 == 0 {
            self.alpha = 1.0;
            return;
        }

        let normalized = self.cutoff_hz.normalized(self.sample_rate);
        let coefficient = 1.0 - (-2.0 * core::f32::consts::PI * normalized).exp();
        self.alpha = coefficient.clamp(0.0, 1.0);
    }
}

impl DspKernel for OnePoleLowPass {
    fn reset(&mut self) {
        self.state = 0.0;
    }

    fn set_bypassed(&mut self, bypassed: bool) {
        self.bypassed = bypassed;
    }

    fn is_bypassed(&self) -> bool {
        self.bypassed
    }

    fn process_in_place(&mut self, block: &mut [Sample]) {
        for sample in block {
            *sample = self.process_sample(*sample);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::OnePoleLowPass;
    use crate::DspKernel;
    use signal_primitives::{FrequencyHz, SampleRate};

    #[test]
    fn one_pole_low_pass_has_monotonic_step_response() {
        let mut filter = OnePoleLowPass::new(SampleRate(48_000), FrequencyHz(500.0));
        let mut block = [1.0; 8];
        filter.process_in_place(&mut block);

        assert!(block[0] > 0.0);
        assert!(block[1] > block[0]);
        assert!(block[7] < 1.0);
    }

    #[test]
    fn one_pole_low_pass_bypass_tracks_input_continuity() {
        let mut filter = OnePoleLowPass::new(SampleRate(48_000), FrequencyHz(500.0));
        let mut active = [1.0; 4];
        filter.process_in_place(&mut active);

        filter.set_bypassed(true);
        let mut bypassed = [0.25; 2];
        filter.process_in_place(&mut bypassed);

        assert_eq!(bypassed, [0.25, 0.25]);

        filter.set_bypassed(false);
        let mut resumed = [0.25; 2];
        filter.process_in_place(&mut resumed);

        assert!(resumed[0] >= 0.25);
    }
}
