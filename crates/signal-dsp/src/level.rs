use crate::{flush_denormal, DspKernel};
use signal_primitives::{Sample, SampleRate, Seconds};

/// Stateful peak meter that tracks the maximum absolute sample value seen since the last reset.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PeakMeter {
    peak: Sample,
    bypassed: bool,
}

impl PeakMeter {
    /// Create a new peak meter initialised to zero.
    pub fn new() -> Self {
        Self {
            peak: 0.0,
            bypassed: false,
        }
    }

    /// Return the highest absolute sample value observed since the last reset.
    pub fn peak(&self) -> Sample {
        self.peak
    }

    fn process_sample(&mut self, sample: Sample) {
        self.peak = self.peak.max(sample.abs());
    }
}

impl Default for PeakMeter {
    fn default() -> Self {
        Self::new()
    }
}

impl DspKernel for PeakMeter {
    fn reset(&mut self) {
        self.peak = 0.0;
    }

    fn set_bypassed(&mut self, bypassed: bool) {
        self.bypassed = bypassed;
    }

    fn is_bypassed(&self) -> bool {
        self.bypassed
    }

    fn process_in_place(&mut self, block: &mut [Sample]) {
        for sample in block.iter().copied() {
            self.process_sample(sample);
        }
    }
}

/// Sliding-window RMS meter that computes the root-mean-square level over a fixed sample window.
#[derive(Clone, Debug, PartialEq)]
pub struct RmsMeter {
    window: Vec<Sample>,
    write_index: usize,
    filled: usize,
    sum_squares: Sample,
    rms: Sample,
    bypassed: bool,
}

impl RmsMeter {
    /// Create a new RMS meter with a window of `window_samples` samples.
    ///
    /// `window_samples` is clamped to a minimum of 1.
    pub fn new(window_samples: usize) -> Self {
        Self {
            window: vec![0.0; window_samples.max(1)],
            write_index: 0,
            filled: 0,
            sum_squares: 0.0,
            rms: 0.0,
            bypassed: false,
        }
    }

    /// Return the current RMS level.
    pub fn rms(&self) -> Sample {
        self.rms
    }

    fn process_sample(&mut self, sample: Sample) {
        let squared = sample * sample;
        let old = self.window[self.write_index];
        self.window[self.write_index] = squared;
        self.write_index = (self.write_index + 1) % self.window.len();
        if self.filled < self.window.len() {
            self.filled += 1;
        }

        self.sum_squares = flush_denormal((self.sum_squares + squared - old).max(0.0));
        self.rms = (self.sum_squares / self.filled.max(1) as Sample).sqrt();
    }
}

impl DspKernel for RmsMeter {
    fn reset(&mut self) {
        self.window.fill(0.0);
        self.write_index = 0;
        self.filled = 0;
        self.sum_squares = 0.0;
        self.rms = 0.0;
    }

    fn set_bypassed(&mut self, bypassed: bool) {
        self.bypassed = bypassed;
    }

    fn is_bypassed(&self) -> bool {
        self.bypassed
    }

    fn process_in_place(&mut self, block: &mut [Sample]) {
        for sample in block.iter().copied() {
            self.process_sample(sample);
        }
    }
}

/// Exponential attack/release envelope follower that tracks the amplitude of an audio signal.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct EnvelopeFollower {
    sample_rate: SampleRate,
    attack: Seconds,
    release: Seconds,
    attack_coeff: Sample,
    release_coeff: Sample,
    envelope: Sample,
    bypassed: bool,
}

impl EnvelopeFollower {
    /// Create a new envelope follower with the given attack and release times.
    pub fn new(sample_rate: SampleRate, attack: Seconds, release: Seconds) -> Self {
        let mut follower = Self {
            sample_rate,
            attack,
            release,
            attack_coeff: 0.0,
            release_coeff: 0.0,
            envelope: 0.0,
            bypassed: false,
        };
        follower.update_coefficients();
        follower
    }

    /// Return the current envelope level.
    pub fn envelope(&self) -> Sample {
        self.envelope
    }

    /// Set a new attack time and recompute the coefficient.
    pub fn set_attack(&mut self, attack: Seconds) {
        self.attack = attack;
        self.update_coefficients();
    }

    /// Set a new release time and recompute the coefficient.
    pub fn set_release(&mut self, release: Seconds) {
        self.release = release;
        self.update_coefficients();
    }

    /// Process one input sample and return the updated envelope level.
    pub fn process_sample(&mut self, input: Sample) -> Sample {
        let input = input.abs();
        let coeff = if input >= self.envelope {
            self.attack_coeff
        } else {
            self.release_coeff
        };
        self.envelope = flush_denormal((coeff * self.envelope) + ((1.0 - coeff) * input));
        self.envelope
    }

    fn update_coefficients(&mut self) {
        self.attack_coeff = time_to_coefficient(self.sample_rate, self.attack);
        self.release_coeff = time_to_coefficient(self.sample_rate, self.release);
    }
}

impl DspKernel for EnvelopeFollower {
    fn reset(&mut self) {
        self.envelope = 0.0;
    }

    fn set_bypassed(&mut self, bypassed: bool) {
        self.bypassed = bypassed;
    }

    fn is_bypassed(&self) -> bool {
        self.bypassed
    }

    fn process_in_place(&mut self, block: &mut [Sample]) {
        for sample in block.iter().copied() {
            self.process_sample(sample);
        }
    }
}

fn time_to_coefficient(sample_rate: SampleRate, seconds: Seconds) -> Sample {
    if sample_rate.0 == 0 || seconds.0 <= 0.0 {
        return 0.0;
    }

    (-1.0 / (seconds.0 * sample_rate.0 as Sample)).exp()
}

#[cfg(test)]
mod tests {
    use super::{EnvelopeFollower, PeakMeter, RmsMeter};
    use crate::DspKernel;
    use signal_primitives::{SampleRate, Seconds};

    #[test]
    fn peak_meter_tracks_maximum_absolute_sample() {
        let mut meter = PeakMeter::new();
        let mut block = [0.25, -0.75, 0.5];
        meter.process_in_place(&mut block);

        assert_eq!(meter.peak(), 0.75);
    }

    #[test]
    fn rms_meter_tracks_windowed_energy() {
        let mut meter = RmsMeter::new(4);
        let mut block = [1.0, 1.0, 1.0, 1.0];
        meter.process_in_place(&mut block);

        assert!((meter.rms() - 1.0).abs() < 1.0e-6);
    }

    #[test]
    fn envelope_follower_attacks_and_releases_smoothly() {
        let mut follower =
            EnvelopeFollower::new(SampleRate(48_000), Seconds(0.001), Seconds(0.050));
        let mut attack = [1.0; 8];
        follower.process_in_place(&mut attack);
        let attacked = follower.envelope();

        let mut release = [0.0; 8];
        follower.process_in_place(&mut release);
        let released = follower.envelope();

        assert!(attacked > 0.0);
        assert!(released < attacked);
        assert!(released > 0.0);
    }
}
