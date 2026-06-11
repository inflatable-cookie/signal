//! Per-sample soft-knee limiter for master-stage protection.
//!
//! The limiter applies one gain per frame across all channels (linked
//! detection via max-abs), so multi-channel images never shift. Channel count
//! is whatever the caller passes per frame; nothing assumes stereo.
//!
//! The static transfer curve mapping detected level `x` to output level is
//! piecewise and tanh-free:
//!
//! - below the knee start: unity (`f(x) = x`)
//! - inside the knee: a quadratic blend that bends the slope from `1` at the
//!   knee start down to [`KNEE_EXIT_SLOPE`] at the knee end
//! - above the knee: a rational soft-saturation segment that continues with
//!   slope [`KNEE_EXIT_SLOPE`] and asymptotically approaches the ceiling
//!   (0 dBFS) without ever reaching it
//!
//! The curve is continuous in value and slope at both joins, monotonic, and
//! strictly below the ceiling, so output can never exceed 0 dBFS. Attack is
//! instant (gain drops immediately when the detector rises); recovery is
//! smoothed by a one-pole release so gain rises without zipper noise.

use signal_primitives::{Sample, SampleRate, Seconds};

/// Hard ceiling of the limiter curve in linear amplitude (0 dBFS).
pub const LIMITER_CEILING: Sample = 1.0;

/// Slope of the transfer curve where the quadratic knee hands over to the
/// rational saturation segment.
const KNEE_EXIT_SLOPE: f32 = 0.5;

/// Minimum knee width in linear amplitude; configured widths are clamped up
/// to this so the quadratic blend never degenerates.
const MIN_KNEE_WIDTH: f32 = 1.0e-3;

/// Soft-knee limiter state with linked-channel detection.
///
/// Construct once per limiter instance (not per channel) and call
/// [`LimiterState::process_frame`] with one interleaved frame at a time.
/// Processing is alloc-free and suitable for the audio thread.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LimiterState {
    /// Detected level at which the curve departs from unity.
    knee_start: f32,
    /// Detected level at which the quadratic knee hands over to the
    /// rational saturation segment.
    knee_end: f32,
    /// Effective knee width after clamping (`knee_end - knee_start`).
    knee_width: f32,
    /// Output level of the curve at `knee_end`.
    knee_end_output: f32,
    /// Ceiling headroom above `knee_end_output` (`LIMITER_CEILING - knee_end_output`).
    saturation_range: f32,
    /// One-pole coefficient applied per frame while gain recovers.
    release_coefficient: f32,
    /// Smoothed gain currently applied to the signal.
    gain: f32,
}

impl LimiterState {
    /// Create a limiter.
    ///
    /// - `threshold` is the linear level where limiting centers; it is
    ///   clamped to `(0, LIMITER_CEILING)` exclusive of the ceiling.
    /// - `knee_width` is the linear width of the soft knee, centered on the
    ///   threshold; it is clamped to keep the whole knee strictly below the
    ///   ceiling.
    /// - `release` is the one-pole gain-recovery time constant; after one
    ///   `release` of low-level input, the gain has recovered roughly 63% of
    ///   the way back to unity.
    pub fn new(
        sample_rate: SampleRate,
        threshold: Sample,
        knee_width: Sample,
        release: Seconds,
    ) -> Self {
        let threshold = threshold.clamp(1.0e-3, LIMITER_CEILING - 1.0e-3);
        // Keep the knee inside (0, ceiling): with the exit slope below one,
        // the curve maximum stays under the ceiling whenever
        // knee_end <= ceiling.
        let max_knee = 2.0 * (LIMITER_CEILING - threshold);
        let knee_width = knee_width.clamp(MIN_KNEE_WIDTH, max_knee);
        let knee_start = threshold - knee_width * 0.5;
        let knee_end = threshold + knee_width * 0.5;
        let knee_end_output = knee_end - (1.0 - KNEE_EXIT_SLOPE) * knee_width * 0.5;
        Self {
            knee_start,
            knee_end,
            knee_width,
            knee_end_output,
            saturation_range: LIMITER_CEILING - knee_end_output,
            release_coefficient: release_coefficient(sample_rate, release),
            gain: 1.0,
        }
    }

    /// Gain currently applied to the signal (1.0 when idle).
    pub fn gain(&self) -> Sample {
        self.gain
    }

    /// Reset the smoothed gain back to unity.
    pub fn reset(&mut self) {
        self.gain = 1.0;
    }

    /// Restore a previously captured smoothed gain (state hand-off across
    /// recompiles: the new limiter continues recovering from where the old
    /// one was instead of snapping to unity).
    pub fn set_gain(&mut self, gain: Sample) {
        self.gain = gain.clamp(0.0, 1.0);
    }

    /// Process one frame of channel samples in place, applying a single
    /// linked gain across all channels.
    ///
    /// Detection is the maximum absolute value across the frame's channels,
    /// so equal inputs always produce equal outputs and the multi-channel
    /// image never shifts. Alloc-free; call once per frame on the audio
    /// thread with however many channels the edge format carries.
    #[inline]
    pub fn process_frame(&mut self, samples: &mut [Sample]) {
        let mut peak: f32 = 0.0;
        for sample in samples.iter() {
            peak = peak.max(sample.abs());
        }

        let target = self.target_gain(peak);
        if target < self.gain {
            // Instant attack: never let the smoothed gain lag above target.
            self.gain = target;
        } else {
            // One-pole recovery toward the (higher) target gain.
            self.gain = target + (self.gain - target) * self.release_coefficient;
        }

        for sample in samples.iter_mut() {
            *sample *= self.gain;
        }
    }

    /// Static transfer curve: detected input level to output level.
    fn transfer(&self, level: f32) -> f32 {
        if level <= self.knee_start {
            level
        } else if level <= self.knee_end {
            // Quadratic blend: value and slope continuous at both knee ends,
            // bending slope from 1.0 down to KNEE_EXIT_SLOPE.
            let excess = level - self.knee_start;
            level - (1.0 - KNEE_EXIT_SLOPE) * excess * excess / (2.0 * self.knee_width)
        } else {
            // Rational saturation: starts at (knee_end, knee_end_output) with
            // slope KNEE_EXIT_SLOPE and approaches the ceiling asymptotically.
            let excess = level - self.knee_end;
            LIMITER_CEILING
                - self.saturation_range * self.saturation_range
                    / (KNEE_EXIT_SLOPE * excess + self.saturation_range)
        }
    }

    /// Desired gain for a detected level (1.0 below the knee start).
    fn target_gain(&self, level: f32) -> f32 {
        if level <= self.knee_start {
            1.0
        } else {
            self.transfer(level) / level
        }
    }
}

/// One-pole release coefficient for per-frame gain recovery.
fn release_coefficient(sample_rate: SampleRate, release: Seconds) -> f32 {
    if sample_rate.0 == 0 || release.0 <= 0.0 {
        return 0.0;
    }
    (-1.0 / (release.0 * sample_rate.0 as f32)).exp()
}

#[cfg(test)]
mod tests {
    use super::{LimiterState, LIMITER_CEILING};
    use signal_primitives::{SampleRate, Seconds};

    const RATE: SampleRate = SampleRate(48_000);

    fn limiter() -> LimiterState {
        LimiterState::new(RATE, 0.7, 0.2, Seconds(0.050))
    }

    #[test]
    fn passthrough_below_threshold_is_unity() {
        let mut limiter = limiter();
        let mut frame = [0.25, -0.5, 0.1];
        limiter.process_frame(&mut frame);
        assert_eq!(frame, [0.25, -0.5, 0.1]);
        assert_eq!(limiter.gain(), 1.0);
    }

    #[test]
    fn output_never_exceeds_ceiling_up_to_plus_twelve_dbfs() {
        let mut limiter = limiter();
        // Sweep up to +12 dBFS (~3.981 linear).
        let mut level = 0.01;
        while level <= 3.981 {
            let mut frame = [level, -level];
            limiter.process_frame(&mut frame);
            for sample in frame {
                assert!(
                    sample.abs() < LIMITER_CEILING,
                    "level {level} produced {sample} above the ceiling"
                );
            }
            level += 0.01;
        }
    }

    #[test]
    fn transfer_curve_is_monotonic_and_continuous() {
        let limiter = limiter();
        let mut previous = 0.0;
        let mut level = 0.0;
        while level <= 8.0 {
            let output = limiter.transfer(level);
            assert!(
                output >= previous,
                "transfer curve regressed at level {level}"
            );
            assert!(
                output - previous < 2.0e-3,
                "transfer curve jumped at level {level}"
            );
            previous = output;
            level += 1.0e-3;
        }
    }

    #[test]
    fn gain_recovery_matches_release_time_constant() {
        let release = Seconds(0.010);
        let mut limiter = LimiterState::new(RATE, 0.5, 0.05, release);

        // Slam the limiter, record the depressed gain.
        let mut loud = [4.0];
        limiter.process_frame(&mut loud);
        let depressed = limiter.gain();
        assert!(depressed < 0.5);

        // Feed silence for exactly one release constant of frames.
        let frames = (release.0 * RATE.0 as f32) as usize;
        for _ in 0..frames {
            let mut quiet = [0.0];
            limiter.process_frame(&mut quiet);
        }

        // One-pole: remaining deficit should be ~e^-1 of the initial deficit.
        let remaining = (1.0 - limiter.gain()) / (1.0 - depressed);
        let expected = (-1.0_f32).exp();
        assert!(
            (remaining - expected).abs() < 0.05,
            "expected ~{expected} of the gain deficit to remain, got {remaining}"
        );
    }

    #[test]
    fn channels_stay_linked() {
        let mut limiter = limiter();
        // Equal inputs across four channels produce equal outputs.
        let mut equal = [2.0, 2.0, 2.0, 2.0];
        limiter.process_frame(&mut equal);
        assert!(equal.windows(2).all(|pair| pair[0] == pair[1]));

        // Unequal inputs keep their exact ratio (single linked gain).
        let mut limiter = self::limiter();
        let mut frame = [2.0, 0.5];
        limiter.process_frame(&mut frame);
        assert!(
            (frame[0] / frame[1] - 4.0).abs() < 1.0e-6,
            "linked gain must preserve inter-channel ratio, got {frame:?}"
        );
    }
}
