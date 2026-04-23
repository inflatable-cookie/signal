use signal_primitives::{FrameCount, FrequencyHz, Sample, SampleRate};

/// Deterministic in-memory signal buffer for testing DSP kernels.
#[derive(Clone, Debug, PartialEq)]
pub struct SignalFixture {
    sample_rate: SampleRate,
    samples: Vec<Sample>,
}

impl SignalFixture {
    /// Create a silent (all-zero) fixture of the given length.
    pub fn silence(sample_rate: SampleRate, frames: FrameCount) -> Self {
        Self {
            sample_rate,
            samples: vec![0.0; frames.0],
        }
    }

    /// Create a fixture with a single non-zero sample at `impulse_frame`.
    ///
    /// Frames outside `[0, frames)` are silent. If `impulse_frame` is out of
    /// range the entire fixture is silent.
    pub fn impulse(
        sample_rate: SampleRate,
        frames: FrameCount,
        impulse_frame: usize,
        amplitude: Sample,
    ) -> Self {
        let mut fixture = Self::silence(sample_rate, frames);
        if let Some(sample) = fixture.samples.get_mut(impulse_frame) {
            *sample = amplitude;
        }
        fixture
    }

    /// Create a fixture that is zero before `step_frame` and `amplitude` from `step_frame` onward.
    pub fn step(
        sample_rate: SampleRate,
        frames: FrameCount,
        step_frame: usize,
        amplitude: Sample,
    ) -> Self {
        let mut fixture = Self::silence(sample_rate, frames);
        for sample in fixture.samples.iter_mut().skip(step_frame) {
            *sample = amplitude;
        }
        fixture
    }

    /// Create a fixture containing a sine wave at the given frequency, amplitude, and phase.
    pub fn sine(
        sample_rate: SampleRate,
        frames: FrameCount,
        frequency: FrequencyHz,
        amplitude: Sample,
        phase_radians: Sample,
    ) -> Self {
        let sample_rate_f32 = sample_rate.as_f32().max(1.0);
        let angular = 2.0 * core::f32::consts::PI * frequency.as_f32() / sample_rate_f32;
        let samples = (0..frames.0)
            .map(|frame| ((angular * frame as Sample) + phase_radians).sin() * amplitude)
            .collect();

        Self {
            sample_rate,
            samples,
        }
    }

    /// Return the sample rate of this fixture.
    pub fn sample_rate(&self) -> SampleRate {
        self.sample_rate
    }

    /// Return the number of frames in this fixture.
    pub fn frames(&self) -> FrameCount {
        FrameCount(self.samples.len())
    }

    /// Return a slice of all samples.
    pub fn samples(&self) -> &[Sample] {
        &self.samples
    }

    /// Consume the fixture and return the underlying sample buffer.
    pub fn into_samples(self) -> Vec<Sample> {
        self.samples
    }
}

#[cfg(test)]
mod tests {
    use super::SignalFixture;
    use signal_primitives::{FrameCount, FrequencyHz, SampleRate};

    #[test]
    fn silence_fixture_is_all_zeroes() {
        let fixture = SignalFixture::silence(SampleRate(48_000), FrameCount(8));
        assert_eq!(fixture.samples(), &[0.0; 8]);
    }

    #[test]
    fn impulse_fixture_sets_single_sample() {
        let fixture = SignalFixture::impulse(SampleRate(48_000), FrameCount(8), 3, 0.75);
        assert_eq!(fixture.samples()[3], 0.75);
        assert_eq!(
            fixture
                .samples()
                .iter()
                .filter(|sample| **sample != 0.0)
                .count(),
            1
        );
    }

    #[test]
    fn step_fixture_switches_at_requested_frame() {
        let fixture = SignalFixture::step(SampleRate(48_000), FrameCount(8), 3, 0.5);
        assert_eq!(&fixture.samples()[0..3], &[0.0, 0.0, 0.0]);
        assert_eq!(&fixture.samples()[3..8], &[0.5, 0.5, 0.5, 0.5, 0.5]);
    }

    #[test]
    fn sine_fixture_matches_expected_quadrature_points() {
        let fixture = SignalFixture::sine(SampleRate(8), FrameCount(8), FrequencyHz(1.0), 1.0, 0.0);
        let expected = [
            0.0,
            0.70710677,
            1.0,
            0.70710677,
            0.0,
            -0.70710677,
            -1.0,
            -0.70710677,
        ];

        for (actual, expected) in fixture.samples().iter().zip(expected) {
            assert!((actual - expected).abs() < 1.0e-5);
        }
    }
}
