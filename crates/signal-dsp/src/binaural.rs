//! Binaural (dual-ear) FIR convolution with crossfading impulse-response
//! swaps — the HRTF rendering kernel.
//!
//! A binaural voice convolves one mono stream against a *pair* of short FIRs
//! (left ear, right ear) drawn from an HRIR measurement set. When the source
//! direction changes, the ear pair must change too — and a hard swap clicks
//! (zipper noise), because the two responses disagree mid-stream. The fix is
//! structural: **two ear pairs, double-buffered**. Both convolve every input
//! sample so the idle pair keeps a warm history; a swap loads the incoming
//! response into the idle pair and linearly crossfades the *outputs* over a
//! fixed window. Linear (constant-sum), not equal-power: the two signals are
//! the *same input* through *neighbouring HRIRs* — highly correlated — so a
//! constant-sum blend is flat for near-identical responses and never
//! overshoots, where equal-power would bump ~+0.1 dB mid-fade and risk the
//! ceiling on hot material. No discontinuity, no history reset, no
//! allocation.
//!
//! Which HRIR to select for a given azimuth/elevation is deliberately not
//! this type's business — that is dataset policy (grid lookup, mirroring,
//! nearest-cell dedup) owned by the caller. This kernel is the real-time
//! half: give it tap pairs, feed it samples.
//!
//! Real-time safety matches [`FirConvolver`]: all buffers are allocated at
//! construction for the longest response the caller will install;
//! [`crossfade_to`](BinauralConvolver::crossfade_to) and
//! [`process_sample`](BinauralConvolver::process_sample) never allocate.

use crate::{DspKernel as _, FirConvolver};
use signal_primitives::Sample;

/// Default crossfade window for HRIR swaps, in samples (~2.7 ms at 48 kHz —
/// long enough to hide the discontinuity, short enough to track fast head
/// motion).
pub const DEFAULT_HRIR_CROSSFADE_SAMPLES: usize = 128;

/// One ear pair: left + right FIR sharing the same mono input.
#[derive(Clone, Debug, PartialEq)]
struct EarPair {
    left: FirConvolver,
    right: FirConvolver,
}

impl EarPair {
    fn with_capacity(max_taps: usize) -> Self {
        Self {
            left: FirConvolver::with_capacity(max_taps),
            right: FirConvolver::with_capacity(max_taps),
        }
    }

    fn set_response(&mut self, left_taps: &[Sample], right_taps: &[Sample]) -> bool {
        let l = self.left.set_impulse_response(left_taps);
        let r = self.right.set_impulse_response(right_taps);
        l && r
    }

    fn process(&mut self, input: Sample) -> (Sample, Sample) {
        (
            self.left.process_sample(input),
            self.right.process_sample(input),
        )
    }

    fn reset(&mut self) {
        self.left.reset();
        self.right.reset();
    }
}

/// Dual-ear FIR convolver with double-buffered, crossfading impulse-response
/// swaps.
///
/// ```
/// use signal_dsp::BinauralConvolver;
///
/// // Unit impulses on both ears: identity until a real HRIR is installed.
/// let mut conv = BinauralConvolver::with_capacity(64, 32);
/// conv.set_response(&[1.0], &[1.0]);
/// assert_eq!(conv.process_sample(0.5), (0.5, 0.5));
///
/// // A swap crossfades instead of clicking.
/// conv.crossfade_to(&[0.5], &[0.25]);
/// assert!(conv.is_crossfading());
/// ```
#[derive(Clone, Debug)]
pub struct BinauralConvolver {
    pairs: [EarPair; 2],
    /// Index of the pair the output is (fading) toward.
    active: usize,
    fade_remaining: usize,
    fade_length: usize,
}

impl BinauralConvolver {
    /// Create a convolver that can hold ear responses up to `max_taps` long,
    /// crossfading swaps over `crossfade_samples`. Outputs silence until a
    /// response is installed.
    pub fn with_capacity(max_taps: usize, crossfade_samples: usize) -> Self {
        Self {
            pairs: [
                EarPair::with_capacity(max_taps),
                EarPair::with_capacity(max_taps),
            ],
            active: 0,
            fade_remaining: 0,
            fade_length: crossfade_samples.max(1),
        }
    }

    /// Install an ear-pair response immediately on **both** buffers (no
    /// crossfade) and cancel any fade in progress. Use for a voice's first
    /// direction — it should start already in place, not fade in from the
    /// previous voice's leftovers. Returns `false` if either response was
    /// truncated to capacity.
    pub fn set_response(&mut self, left_taps: &[Sample], right_taps: &[Sample]) -> bool {
        let a = self.pairs[0].set_response(left_taps, right_taps);
        let b = self.pairs[1].set_response(left_taps, right_taps);
        self.fade_remaining = 0;
        a && b
    }

    /// Load a new ear-pair response into the idle buffer and begin a linear
    /// (constant-sum) crossfade toward it. Calling again mid-fade retargets:
    /// the current blended output keeps evolving toward the newest response
    /// (the previous target becomes the fade-out side). Real-time safe.
    /// Returns `false` if either response was truncated to capacity.
    pub fn crossfade_to(&mut self, left_taps: &[Sample], right_taps: &[Sample]) -> bool {
        let incoming = 1 - self.active;
        let fit = self.pairs[incoming].set_response(left_taps, right_taps);
        self.active = incoming;
        self.fade_remaining = self.fade_length;
        fit
    }

    /// Whether a response crossfade is currently in progress.
    pub fn is_crossfading(&self) -> bool {
        self.fade_remaining > 0
    }

    /// Convolve one mono sample into a binaural `(left, right)` pair.
    ///
    /// Both ear pairs run every sample — the idle pair keeps a warm input
    /// history so it can become the fade target without a discontinuity.
    pub fn process_sample(&mut self, input: Sample) -> (Sample, Sample) {
        let (head, tail) = self.pairs.split_at_mut(1);
        let (active, other) = if self.active == 0 {
            (&mut head[0], &mut tail[0])
        } else {
            (&mut tail[0], &mut head[0])
        };
        let (al, ar) = active.process(input);
        let (bl, br) = other.process(input);

        if self.fade_remaining == 0 {
            return (al, ar);
        }
        self.fade_remaining -= 1;
        // Linear constant-sum blend: incoming weight rises 0→1. Correct for
        // correlated material (same input, neighbouring responses).
        let active_gain = 1.0 - self.fade_remaining as f32 / self.fade_length as f32;
        let other_gain = 1.0 - active_gain;
        (
            al * active_gain + bl * other_gain,
            ar * active_gain + br * other_gain,
        )
    }

    /// Convolve a mono block into separate left/right output blocks. All
    /// three slices must share a length.
    ///
    /// # Panics
    ///
    /// Panics if `input`, `left_out`, and `right_out` do not all have the
    /// same length. Block sizes are fixed by the render plan, so a mismatch
    /// is a caller wiring error rather than a runtime condition.
    pub fn process_block(
        &mut self,
        input: &[Sample],
        left_out: &mut [Sample],
        right_out: &mut [Sample],
    ) {
        assert_eq!(input.len(), left_out.len());
        assert_eq!(input.len(), right_out.len());
        for ((&sample, left), right) in input
            .iter()
            .zip(left_out.iter_mut())
            .zip(right_out.iter_mut())
        {
            let (l, r) = self.process_sample(sample);
            *left = l;
            *right = r;
        }
    }
}

impl BinauralConvolver {
    /// Clear both ear pairs' histories and cancel any fade in progress.
    /// (`DspKernel` is deliberately not implemented — that trait models
    /// in-place mono processors, and this kernel is mono→stereo.)
    pub fn reset(&mut self) {
        self.pairs[0].reset();
        self.pairs[1].reset();
        self.fade_remaining = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identity_response_passes_through_both_ears() {
        let mut conv = BinauralConvolver::with_capacity(8, 4);
        conv.set_response(&[1.0], &[1.0]);
        for &x in &[0.5, -0.25, 1.0, 0.0] {
            assert_eq!(conv.process_sample(x), (x, x));
        }
    }

    #[test]
    fn per_ear_responses_differ() {
        let mut conv = BinauralConvolver::with_capacity(8, 4);
        conv.set_response(&[0.5], &[0.25]);
        assert_eq!(conv.process_sample(1.0), (0.5, 0.25));
    }

    #[test]
    fn set_response_snaps_without_fade() {
        let mut conv = BinauralConvolver::with_capacity(8, 64);
        conv.set_response(&[1.0], &[1.0]);
        conv.set_response(&[0.5], &[0.5]);
        assert!(!conv.is_crossfading());
        assert_eq!(conv.process_sample(1.0), (0.5, 0.5));
    }

    #[test]
    fn crossfade_is_smooth_and_completes() {
        let fade = 64usize;
        let mut conv = BinauralConvolver::with_capacity(8, fade);
        conv.set_response(&[1.0], &[1.0]);

        // Warm both histories with DC so the swap has steady state on each side.
        for _ in 0..8 {
            conv.process_sample(1.0);
        }

        conv.crossfade_to(&[0.5], &[0.5]);
        assert!(conv.is_crossfading());

        // Under DC input, output must move monotonically 1.0 → 0.5 with no
        // step bigger than the fade slope allows (no zipper click).
        let mut last = 1.0f32;
        for _ in 0..fade {
            let (l, r) = conv.process_sample(1.0);
            assert!((l - r).abs() < 1e-6);
            assert!(
                l <= last + 1e-4,
                "output rose during fade-down: {l} > {last}"
            );
            assert!(last - l < 0.05, "step too large: {last} -> {l}");
            last = l;
        }
        assert!(!conv.is_crossfading());
        let (l, _) = conv.process_sample(1.0);
        assert!(
            (l - 0.5).abs() < 1e-4,
            "fade should settle on the new response, got {l}"
        );
    }

    #[test]
    fn same_response_fade_is_exactly_flat() {
        // The constant-sum property: fading A→A is output-invariant — no dip,
        // no bump. (This is why the blend is linear, not equal-power: the two
        // branches carry the same input and near-identical responses.)
        let fade = 32usize;
        let mut conv = BinauralConvolver::with_capacity(8, fade);
        conv.set_response(&[1.0], &[1.0]);
        for _ in 0..4 {
            conv.process_sample(1.0);
        }
        conv.crossfade_to(&[1.0], &[1.0]);
        for _ in 0..fade {
            let (l, r) = conv.process_sample(1.0);
            assert!(
                (l - 1.0).abs() < 1e-5 && (r - 1.0).abs() < 1e-5,
                "flat fade broke: {l}"
            );
        }
    }

    #[test]
    fn retarget_mid_fade_keeps_moving_toward_newest() {
        let fade = 32usize;
        let mut conv = BinauralConvolver::with_capacity(8, fade);
        conv.set_response(&[1.0], &[1.0]);
        for _ in 0..4 {
            conv.process_sample(1.0);
        }
        conv.crossfade_to(&[0.5], &[0.5]);
        for _ in 0..8 {
            conv.process_sample(1.0);
        }
        // Retarget before the first fade finishes.
        conv.crossfade_to(&[0.0], &[0.0]);
        for _ in 0..fade {
            conv.process_sample(1.0);
        }
        let (l, _) = conv.process_sample(1.0);
        assert!(
            l.abs() < 0.05,
            "should settle near the retargeted response, got {l}"
        );
    }

    #[test]
    fn interaural_delay_is_preserved() {
        // Right ear delayed by two samples relative to left: the classic ITD
        // shape an HRIR encodes. An impulse must come out at different times.
        let mut conv = BinauralConvolver::with_capacity(8, 4);
        conv.set_response(&[1.0, 0.0, 0.0], &[0.0, 0.0, 1.0]);
        let outs: Vec<(Sample, Sample)> = (0..4)
            .map(|i| conv.process_sample(if i == 0 { 1.0 } else { 0.0 }))
            .collect();
        assert_eq!(outs[0], (1.0, 0.0));
        assert_eq!(outs[2], (0.0, 1.0));
    }

    #[test]
    fn process_block_matches_per_sample() {
        let mut a = BinauralConvolver::with_capacity(16, 8);
        let mut b = BinauralConvolver::with_capacity(16, 8);
        let taps_l = [0.6, 0.3, 0.1];
        let taps_r = [0.1, 0.3, 0.6];
        a.set_response(&taps_l, &taps_r);
        b.set_response(&taps_l, &taps_r);

        let input: Vec<Sample> = (0..32).map(|i| ((i * 7) % 5) as Sample - 2.0).collect();
        let mut left = vec![0.0; input.len()];
        let mut right = vec![0.0; input.len()];
        a.process_block(&input, &mut left, &mut right);
        for (i, &x) in input.iter().enumerate() {
            let (l, r) = b.process_sample(x);
            assert_eq!((left[i], right[i]), (l, r));
        }
    }

    #[test]
    fn reset_clears_history_and_fade() {
        let mut conv = BinauralConvolver::with_capacity(8, 16);
        conv.set_response(&[0.0, 1.0], &[0.0, 1.0]);
        conv.process_sample(1.0);
        conv.crossfade_to(&[1.0], &[1.0]);
        conv.reset();
        assert!(!conv.is_crossfading());
        // History cleared: the delayed tap sees silence.
        assert_eq!(conv.process_sample(0.0), (0.0, 0.0));
    }

    #[test]
    fn truncation_reports_false() {
        let mut conv = BinauralConvolver::with_capacity(2, 4);
        assert!(!conv.set_response(&[1.0, 0.5, 0.25], &[1.0]));
        assert!(!conv.crossfade_to(&[1.0], &[1.0, 0.5, 0.25]));
    }
}
