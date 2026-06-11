//! Precomputed polyphase windowed-sinc interpolation table.
//!
//! Built control-side (allocates), consumed sample-by-sample on the audio
//! thread (pure table reads + a tap dot product — no allocation, no
//! transcendentals). The render plane uses this to replace linear
//! interpolation for rate-converted media playback.
//!
//! Design: Kaiser-windowed sinc, unity-DC-normalised per phase. Cutoff is
//! expressed as a fraction of the *output* Nyquist; downsampling callers pass
//! `1.0 / ratio` to keep images below the source band, upsampling callers
//! pass `1.0`.

use std::sync::Arc;

use signal_primitives::Sample;

/// Kaiser window shape parameter. β ≈ 9 gives ~90 dB sidelobe attenuation,
/// comfortably beyond audibility for 16-tap interpolation.
const KAISER_BETA: f64 = 9.0;
/// Margin keeping the transition band inside the cutoff.
const CUTOFF_MARGIN: f64 = 0.91;

/// Zeroth-order modified Bessel function of the first kind (power series).
fn bessel_i0(x: f64) -> f64 {
    let mut sum = 1.0;
    let mut term = 1.0;
    let half_x = x / 2.0;
    for k in 1..32 {
        term *= (half_x / k as f64) * (half_x / k as f64);
        sum += term;
        if term < sum * 1e-18 {
            break;
        }
    }
    sum
}

fn kaiser(position: f64, half_width: f64) -> f64 {
    let ratio = position / half_width;
    if ratio.abs() > 1.0 {
        return 0.0;
    }
    bessel_i0(KAISER_BETA * (1.0 - ratio * ratio).sqrt()) / bessel_i0(KAISER_BETA)
}

fn sinc(x: f64) -> f64 {
    if x.abs() < 1e-12 {
        1.0
    } else {
        (std::f64::consts::PI * x).sin() / (std::f64::consts::PI * x)
    }
}

/// Precomputed interpolation table: `phases` fractional offsets ×
/// `taps_per_phase` coefficients, flattened phase-major.
#[derive(Debug, Clone)]
pub struct PolyphaseInterpolationTable {
    taps_per_phase: usize,
    phases: usize,
    /// Flattened `[phase][tap]` coefficients, each phase unity-DC.
    coefficients: Arc<[Sample]>,
}

impl PolyphaseInterpolationTable {
    /// Build a table. `cutoff` is the lowpass cutoff as a fraction of
    /// Nyquist (`0 < cutoff <= 1`); pass `1.0` for upsampling and
    /// `1.0 / ratio` when reading the source faster than 1:1.
    pub fn new(taps_per_phase: usize, phases: usize, cutoff: f64) -> Self {
        let taps = taps_per_phase.max(2);
        let phase_count = phases.max(1);
        let cutoff = (cutoff * CUTOFF_MARGIN).clamp(1e-3, 1.0);
        let half_width = taps as f64 / 2.0;
        // Tap t covers source offset (t - center); each phase shifts the
        // kernel by its fractional offset.
        let center = (taps / 2 - 1) as f64;
        let mut coefficients = vec![0.0 as Sample; taps * phase_count];
        for phase in 0..phase_count {
            let fraction = phase as f64 / phase_count as f64;
            let row = &mut coefficients[phase * taps..(phase + 1) * taps];
            let mut sum = 0.0f64;
            for (tap, slot) in row.iter_mut().enumerate() {
                let offset = tap as f64 - center - fraction;
                let value = cutoff * sinc(cutoff * offset) * kaiser(offset, half_width);
                *slot = value as Sample;
                sum += value;
            }
            // Unity DC gain per phase: flat passband regardless of cutoff.
            if sum.abs() > 1e-12 {
                let scale = (1.0 / sum) as Sample;
                for slot in row.iter_mut() {
                    *slot *= scale;
                }
            }
        }
        Self {
            taps_per_phase: taps,
            phases: phase_count,
            coefficients: coefficients.into(),
        }
    }

    /// Number of taps in each phase row.
    pub fn taps_per_phase(&self) -> usize {
        self.taps_per_phase
    }

    /// Number of fractional phases.
    pub fn phases(&self) -> usize {
        self.phases
    }

    /// Source-index offset of the first tap relative to the integer read
    /// position (negative: taps reach back into history).
    pub fn first_tap_offset(&self) -> isize {
        -((self.taps_per_phase / 2 - 1) as isize)
    }

    /// Shared flattened coefficients (phase-major).
    pub fn coefficients(&self) -> &Arc<[Sample]> {
        &self.coefficients
    }

    /// Coefficient row for a fractional position `0.0 <= fraction < 1.0`.
    #[inline]
    pub fn phase_row(&self, fraction: f64) -> &[Sample] {
        let phase = ((fraction * self.phases as f64) as usize).min(self.phases - 1);
        &self.coefficients[phase * self.taps_per_phase..(phase + 1) * self.taps_per_phase]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_phase_has_unity_dc_gain() {
        let table = PolyphaseInterpolationTable::new(16, 512, 1.0);
        for phase in 0..table.phases() {
            let row = &table.coefficients()[phase * 16..(phase + 1) * 16];
            let sum: f32 = row.iter().sum();
            assert!((sum - 1.0).abs() < 1e-4, "phase {phase} DC gain {sum}");
        }
    }

    #[test]
    fn interpolates_a_sine_more_accurately_than_linear() {
        // 1 kHz sine at 44.1k read at 48k output positions; compare both
        // interpolators against the analytic signal.
        let source_rate = 44_100.0f64;
        let output_rate = 48_000.0f64;
        let frequency = 1_000.0f64;
        let source: Vec<f32> = (0..44_100)
            .map(|n| (2.0 * std::f64::consts::PI * frequency * n as f64 / source_rate).sin() as f32)
            .collect();
        let table = PolyphaseInterpolationTable::new(16, 512, 1.0);
        let step = source_rate / output_rate;
        let taps = table.taps_per_phase();
        let first = table.first_tap_offset();

        let mut sinc_error = 0.0f64;
        let mut linear_error = 0.0f64;
        let mut signal_power = 0.0f64;
        // Stay away from buffer edges.
        for output_index in 1_000..40_000usize {
            let position = output_index as f64 * step;
            let index = position as usize;
            let fraction = position - index as f64;
            let expected = (2.0 * std::f64::consts::PI * frequency * position / source_rate).sin();
            signal_power += expected * expected;

            let row = table.phase_row(fraction);
            let mut acc = 0.0f64;
            for (tap, coefficient) in row.iter().enumerate().take(taps) {
                let source_index = index as isize + first + tap as isize;
                if source_index >= 0 && (source_index as usize) < source.len() {
                    acc += source[source_index as usize] as f64 * *coefficient as f64;
                }
            }
            sinc_error += (acc - expected) * (acc - expected);

            let a = source[index] as f64;
            let b = source[index + 1] as f64;
            let linear = a + (b - a) * fraction;
            linear_error += (linear - expected) * (linear - expected);
        }

        let sinc_snr = 10.0 * (signal_power / sinc_error.max(1e-30)).log10();
        let linear_snr = 10.0 * (signal_power / linear_error.max(1e-30)).log10();
        assert!(sinc_snr > 60.0, "sinc SNR {sinc_snr:.1} dB");
        assert!(
            sinc_snr > linear_snr + 20.0,
            "sinc {sinc_snr:.1} dB should beat linear {linear_snr:.1} dB by 20 dB",
        );
    }
}
