//! RBJ Audio EQ Cookbook biquad filters.
//!
//! Coefficient math ([`BiquadCoefficients`]) is stateless and separated from
//! the per-channel processing state ([`BiquadState`]) so graph code can share
//! one coefficient set across any number of channels. Channel count is driven
//! entirely by callers: allocate one [`BiquadState`] per channel of the edge
//! format. Formulas follow Robert Bristow-Johnson's Audio EQ Cookbook, with
//! coefficients normalized by `a0`.

use crate::flush_denormal;
use signal_primitives::{FrequencyHz, Sample, SampleRate};

/// Minimum corner frequency used by the coefficient calculators, in Hz.
const MIN_FREQUENCY_HZ: f64 = 1.0e-3;

/// Minimum quality factor accepted by the coefficient calculators.
const MIN_Q: f64 = 1.0e-3;

/// Normalized biquad coefficients for the transfer function
/// `H(z) = (b0 + b1*z^-1 + b2*z^-2) / (1 + a1*z^-1 + a2*z^-2)`.
///
/// All constructors normalize by the cookbook `a0` term, so `a0` is implied
/// as `1.0`. The struct is pure data with no processing state; pair it with
/// one [`BiquadState`] per channel.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BiquadCoefficients {
    /// Feedforward coefficient for the current input sample.
    pub b0: Sample,
    /// Feedforward coefficient for the input delayed by one sample.
    pub b1: Sample,
    /// Feedforward coefficient for the input delayed by two samples.
    pub b2: Sample,
    /// Feedback coefficient for the output delayed by one sample.
    pub a1: Sample,
    /// Feedback coefficient for the output delayed by two samples.
    pub a2: Sample,
}

impl BiquadCoefficients {
    /// Identity (pass-through) coefficients.
    pub const IDENTITY: Self = Self {
        b0: 1.0,
        b1: 0.0,
        b2: 0.0,
        a1: 0.0,
        a2: 0.0,
    };

    /// Second-order low-pass filter.
    ///
    /// `q = 1/sqrt(2)` yields the maximally flat Butterworth response with
    /// -3.01 dB at `frequency_hz`.
    pub fn low_pass(sample_rate_hz: SampleRate, frequency_hz: FrequencyHz, q: f32) -> Self {
        let Some(trig) = CookbookTrig::new(sample_rate_hz, frequency_hz, q) else {
            return Self::IDENTITY;
        };
        let b1 = 1.0 - trig.cos_w0;
        let b0 = b1 * 0.5;
        Self::normalize(b0, b1, b0, trig.a0(), trig.a1(), trig.a2())
    }

    /// Second-order high-pass filter.
    pub fn high_pass(sample_rate_hz: SampleRate, frequency_hz: FrequencyHz, q: f32) -> Self {
        let Some(trig) = CookbookTrig::new(sample_rate_hz, frequency_hz, q) else {
            return Self::IDENTITY;
        };
        let b1 = -(1.0 + trig.cos_w0);
        let b0 = -b1 * 0.5;
        Self::normalize(b0, b1, b0, trig.a0(), trig.a1(), trig.a2())
    }

    /// Second-order band-pass filter with constant 0 dB peak gain.
    ///
    /// This is the cookbook's `b0 = alpha` variant: the response peaks at
    /// exactly 0 dB at `frequency_hz` regardless of `q` (the alternative
    /// constant-skirt-gain variant peaks at `q` instead).
    pub fn band_pass(sample_rate_hz: SampleRate, frequency_hz: FrequencyHz, q: f32) -> Self {
        let Some(trig) = CookbookTrig::new(sample_rate_hz, frequency_hz, q) else {
            return Self::IDENTITY;
        };
        Self::normalize(
            trig.alpha,
            0.0,
            -trig.alpha,
            trig.a0(),
            trig.a1(),
            trig.a2(),
        )
    }

    /// Second-order notch filter with a transfer-function zero at `frequency_hz`.
    pub fn notch(sample_rate_hz: SampleRate, frequency_hz: FrequencyHz, q: f32) -> Self {
        let Some(trig) = CookbookTrig::new(sample_rate_hz, frequency_hz, q) else {
            return Self::IDENTITY;
        };
        let b1 = -2.0 * trig.cos_w0;
        Self::normalize(1.0, b1, 1.0, trig.a0(), trig.a1(), trig.a2())
    }

    /// Peaking (parametric) EQ band with `gain_db` boost or cut at `frequency_hz`.
    pub fn peaking(
        sample_rate_hz: SampleRate,
        frequency_hz: FrequencyHz,
        q: f32,
        gain_db: f32,
    ) -> Self {
        let Some(trig) = CookbookTrig::new(sample_rate_hz, frequency_hz, q) else {
            return Self::IDENTITY;
        };
        let a = db_to_cookbook_amplitude(gain_db);
        let b1 = -2.0 * trig.cos_w0;
        Self::normalize(
            1.0 + trig.alpha * a,
            b1,
            1.0 - trig.alpha * a,
            1.0 + trig.alpha / a,
            b1,
            1.0 - trig.alpha / a,
        )
    }

    /// Low shelf with `gain_db` plateau below `frequency_hz`.
    ///
    /// `q` is the cookbook's quality-factor parameterization of the shelf
    /// transition steepness (not the slope parameter `S`); `q = 1/sqrt(2)`
    /// gives the classic monotonic shelf.
    pub fn low_shelf(
        sample_rate_hz: SampleRate,
        frequency_hz: FrequencyHz,
        q: f32,
        gain_db: f32,
    ) -> Self {
        let Some(trig) = CookbookTrig::new(sample_rate_hz, frequency_hz, q) else {
            return Self::IDENTITY;
        };
        let a = db_to_cookbook_amplitude(gain_db);
        let (ap1, am1, cos_w0) = (a + 1.0, a - 1.0, trig.cos_w0);
        let two_sqrt_a_alpha = 2.0 * a.sqrt() * trig.alpha;
        Self::normalize(
            a * (ap1 - am1 * cos_w0 + two_sqrt_a_alpha),
            2.0 * a * (am1 - ap1 * cos_w0),
            a * (ap1 - am1 * cos_w0 - two_sqrt_a_alpha),
            ap1 + am1 * cos_w0 + two_sqrt_a_alpha,
            -2.0 * (am1 + ap1 * cos_w0),
            ap1 + am1 * cos_w0 - two_sqrt_a_alpha,
        )
    }

    /// High shelf with `gain_db` plateau above `frequency_hz`.
    ///
    /// `q` follows the same quality-factor parameterization as [`Self::low_shelf`].
    pub fn high_shelf(
        sample_rate_hz: SampleRate,
        frequency_hz: FrequencyHz,
        q: f32,
        gain_db: f32,
    ) -> Self {
        let Some(trig) = CookbookTrig::new(sample_rate_hz, frequency_hz, q) else {
            return Self::IDENTITY;
        };
        let a = db_to_cookbook_amplitude(gain_db);
        let (ap1, am1, cos_w0) = (a + 1.0, a - 1.0, trig.cos_w0);
        let two_sqrt_a_alpha = 2.0 * a.sqrt() * trig.alpha;
        Self::normalize(
            a * (ap1 + am1 * cos_w0 + two_sqrt_a_alpha),
            -2.0 * a * (am1 + ap1 * cos_w0),
            a * (ap1 + am1 * cos_w0 - two_sqrt_a_alpha),
            ap1 - am1 * cos_w0 + two_sqrt_a_alpha,
            2.0 * (am1 - ap1 * cos_w0),
            ap1 - am1 * cos_w0 - two_sqrt_a_alpha,
        )
    }

    fn normalize(b0: f64, b1: f64, b2: f64, a0: f64, a1: f64, a2: f64) -> Self {
        Self {
            b0: (b0 / a0) as Sample,
            b1: (b1 / a0) as Sample,
            b2: (b2 / a0) as Sample,
            a1: (a1 / a0) as Sample,
            a2: (a2 / a0) as Sample,
        }
    }
}

/// Shared cookbook intermediates computed in `f64` for coefficient accuracy.
struct CookbookTrig {
    cos_w0: f64,
    alpha: f64,
}

impl CookbookTrig {
    /// Returns `None` when the sample rate is zero, signalling the caller to
    /// fall back to identity coefficients. The corner frequency is clamped to
    /// `(0, 0.49 * sample_rate)` and `q` to a small positive minimum so the
    /// coefficient math stays finite for degenerate inputs.
    fn new(sample_rate_hz: SampleRate, frequency_hz: FrequencyHz, q: f32) -> Option<Self> {
        if sample_rate_hz.0 == 0 {
            return None;
        }
        let rate = f64::from(sample_rate_hz.0);
        let frequency = f64::from(frequency_hz.0).clamp(MIN_FREQUENCY_HZ, 0.49 * rate);
        let q = f64::from(q).max(MIN_Q);
        let w0 = core::f64::consts::TAU * frequency / rate;
        let (sin_w0, cos_w0) = w0.sin_cos();
        Some(Self {
            cos_w0,
            alpha: sin_w0 / (2.0 * q),
        })
    }

    fn a0(&self) -> f64 {
        1.0 + self.alpha
    }

    fn a1(&self) -> f64 {
        -2.0 * self.cos_w0
    }

    fn a2(&self) -> f64 {
        1.0 - self.alpha
    }
}

/// Convert decibels to the cookbook's `A` amplitude term (`10^(dB/40)`),
/// computed in `f64`.
fn db_to_cookbook_amplitude(gain_db: f32) -> f64 {
    10.0_f64.powf(f64::from(gain_db) / 40.0)
}

/// Per-channel biquad processing state in transposed direct form II.
///
/// Holds the two delay registers only; coefficients are passed into
/// [`BiquadState::process`] so one coefficient set can drive many channels.
/// Processing is alloc-free and branch-free, suitable for the audio thread.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct BiquadState {
    z1: Sample,
    z2: Sample,
}

impl BiquadState {
    /// Create a state with cleared delay registers.
    pub fn new() -> Self {
        Self::default()
    }

    /// Clear the delay registers.
    pub fn reset(&mut self) {
        *self = Self::default();
    }

    /// Process one sample through the filter described by `coefficients`.
    ///
    /// State registers are defensively flushed via [`flush_denormal`] so
    /// decaying tails cannot park the filter memory in the subnormal range
    /// even without an FTZ guard active.
    #[inline]
    pub fn process(&mut self, coefficients: &BiquadCoefficients, input: Sample) -> Sample {
        let output = coefficients.b0 * input + self.z1;
        self.z1 = flush_denormal(coefficients.b1 * input - coefficients.a1 * output + self.z2);
        self.z2 = flush_denormal(coefficients.b2 * input - coefficients.a2 * output);
        output
    }

    /// Process a block of samples in place with fixed coefficients.
    pub fn process_in_place(&mut self, coefficients: &BiquadCoefficients, block: &mut [Sample]) {
        for sample in block {
            *sample = self.process(coefficients, *sample);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{BiquadCoefficients, BiquadState};
    use signal_primitives::{FrequencyHz, SampleRate};

    const RATE: SampleRate = SampleRate(48_000);
    const FC: FrequencyHz = FrequencyHz(1_000.0);
    const BUTTERWORTH_Q: f32 = core::f32::consts::FRAC_1_SQRT_2;

    /// Analytic magnitude response in dB: evaluates |H(e^jw)| from the
    /// normalized coefficients at `frequency_hz`, in f64.
    fn magnitude_db(coefficients: &BiquadCoefficients, frequency_hz: f32) -> f64 {
        let w = core::f64::consts::TAU * f64::from(frequency_hz) / f64::from(RATE.0);
        let (b0, b1, b2) = (
            f64::from(coefficients.b0),
            f64::from(coefficients.b1),
            f64::from(coefficients.b2),
        );
        let (a1, a2) = (f64::from(coefficients.a1), f64::from(coefficients.a2));
        // H(e^jw) = (b0 + b1 e^-jw + b2 e^-2jw) / (1 + a1 e^-jw + a2 e^-2jw)
        let num_re = b0 + b1 * w.cos() + b2 * (2.0 * w).cos();
        let num_im = -(b1 * w.sin() + b2 * (2.0 * w).sin());
        let den_re = 1.0 + a1 * w.cos() + a2 * (2.0 * w).cos();
        let den_im = -(a1 * w.sin() + a2 * (2.0 * w).sin());
        let magnitude =
            (num_re.hypot(num_im) / den_re.hypot(den_im)).max(f64::from(f32::MIN_POSITIVE));
        20.0 * magnitude.log10()
    }

    fn assert_db(actual: f64, expected: f64, label: &str) {
        assert!(
            (actual - expected).abs() < 0.1,
            "{label}: expected {expected} dB, measured {actual} dB"
        );
    }

    #[test]
    fn low_pass_matches_cookbook_magnitudes() {
        let coefficients = BiquadCoefficients::low_pass(RATE, FC, BUTTERWORTH_Q);
        assert_db(magnitude_db(&coefficients, 1.0), 0.0, "low pass near DC");
        assert_db(magnitude_db(&coefficients, FC.0), -3.0103, "low pass at fc");
        assert!(
            magnitude_db(&coefficients, 23_999.0) < -40.0,
            "low pass near Nyquist should be deeply attenuated"
        );
    }

    #[test]
    fn high_pass_matches_cookbook_magnitudes() {
        let coefficients = BiquadCoefficients::high_pass(RATE, FC, BUTTERWORTH_Q);
        assert_db(
            magnitude_db(&coefficients, 23_999.0),
            0.0,
            "high pass near Nyquist",
        );
        assert_db(
            magnitude_db(&coefficients, FC.0),
            -3.0103,
            "high pass at fc",
        );
        assert!(
            magnitude_db(&coefficients, 1.0) < -40.0,
            "high pass near DC should be deeply attenuated"
        );
    }

    #[test]
    fn band_pass_peaks_at_zero_db() {
        let coefficients = BiquadCoefficients::band_pass(RATE, FC, 4.0);
        assert_db(magnitude_db(&coefficients, FC.0), 0.0, "band pass at fc");
        assert!(
            magnitude_db(&coefficients, 1.0) < -40.0,
            "band pass near DC should be deeply attenuated"
        );
        assert!(
            magnitude_db(&coefficients, 23_999.0) < -40.0,
            "band pass near Nyquist should be deeply attenuated"
        );
    }

    #[test]
    fn notch_attenuates_deeply_at_center() {
        let coefficients = BiquadCoefficients::notch(RATE, FC, 4.0);
        assert!(
            magnitude_db(&coefficients, FC.0) < -60.0,
            "notch at fc should be deeply attenuated"
        );
        assert_db(magnitude_db(&coefficients, 1.0), 0.0, "notch near DC");
        assert_db(
            magnitude_db(&coefficients, 23_999.0),
            0.0,
            "notch near Nyquist",
        );
    }

    #[test]
    fn peaking_reads_gain_db_at_center() {
        for gain_db in [-9.0, 6.0] {
            let coefficients = BiquadCoefficients::peaking(RATE, FC, 1.5, gain_db);
            assert_db(
                magnitude_db(&coefficients, FC.0),
                f64::from(gain_db),
                "peaking at fc",
            );
            assert_db(magnitude_db(&coefficients, 1.0), 0.0, "peaking near DC");
            assert_db(
                magnitude_db(&coefficients, 23_999.0),
                0.0,
                "peaking near Nyquist",
            );
        }
    }

    #[test]
    fn low_shelf_plateau_matches_gain_db() {
        let coefficients = BiquadCoefficients::low_shelf(RATE, FC, BUTTERWORTH_Q, 6.0);
        assert_db(magnitude_db(&coefficients, 1.0), 6.0, "low shelf plateau");
        assert_db(
            magnitude_db(&coefficients, FC.0),
            3.0,
            "low shelf midpoint at fc",
        );
        assert_db(
            magnitude_db(&coefficients, 23_999.0),
            0.0,
            "low shelf above fc",
        );
    }

    #[test]
    fn high_shelf_plateau_matches_gain_db() {
        let coefficients = BiquadCoefficients::high_shelf(RATE, FC, BUTTERWORTH_Q, -6.0);
        assert_db(
            magnitude_db(&coefficients, 23_999.0),
            -6.0,
            "high shelf plateau",
        );
        assert_db(
            magnitude_db(&coefficients, FC.0),
            -3.0,
            "high shelf midpoint at fc",
        );
        assert_db(magnitude_db(&coefficients, 1.0), 0.0, "high shelf below fc");
    }

    #[test]
    fn dc_converges_through_low_pass_and_dies_through_high_pass() {
        let low = BiquadCoefficients::low_pass(RATE, FC, BUTTERWORTH_Q);
        let high = BiquadCoefficients::high_pass(RATE, FC, BUTTERWORTH_Q);
        let mut low_state = BiquadState::new();
        let mut high_state = BiquadState::new();

        let mut low_out = 0.0;
        let mut high_out = 1.0;
        for _ in 0..48_000 {
            low_out = low_state.process(&low, 1.0);
            high_out = high_state.process(&high, 1.0);
        }

        assert!(
            (low_out - 1.0).abs() < 1.0e-4,
            "DC through low pass should converge to DC, got {low_out}"
        );
        assert!(
            high_out.abs() < 1.0e-4,
            "DC through high pass should converge to zero, got {high_out}"
        );
    }

    #[test]
    fn zero_sample_rate_yields_identity_coefficients() {
        let coefficients = BiquadCoefficients::low_pass(SampleRate(0), FC, BUTTERWORTH_Q);
        assert_eq!(coefficients, BiquadCoefficients::IDENTITY);
    }

    #[test]
    fn independent_states_share_coefficients_across_channels() {
        let coefficients = BiquadCoefficients::low_pass(RATE, FC, BUTTERWORTH_Q);
        let mut channels = [BiquadState::new(), BiquadState::new(), BiquadState::new()];
        let outputs: Vec<f32> = channels
            .iter_mut()
            .map(|state| state.process(&coefficients, 0.5))
            .collect();
        assert_eq!(outputs[0], outputs[1]);
        assert_eq!(outputs[1], outputs[2]);
    }
}
