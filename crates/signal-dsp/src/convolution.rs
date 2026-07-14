use crate::{flush_denormal, DspKernel};
use signal_primitives::Sample;

/// Single-channel direct-form FIR convolution kernel.
///
/// Convolves an input stream with a fixed finite impulse response (FIR):
/// `y[n] = Σ_k h[k] · x[n − k]`. This is the primitive behind head-related
/// transfer functions (each ear is a short FIR) and short/medium impulse
/// responses; long reverb tails are better served by a partitioned frequency-
/// domain convolver.
///
/// Real-time safe: the coefficient and history buffers are allocated once at
/// `capacity`, and neither [`process_sample`](Self::process_sample) nor
/// [`set_impulse_response`](Self::set_impulse_response) allocates. The kernel is
/// causal and adds **no latency** — `y[n]` depends only on the current and past
/// inputs, so a convolver's output aligns with its input (the impulse
/// response's own leading delay, if any, is part of the response).
///
/// ```
/// use signal_dsp::{DspKernel, FirConvolver};
///
/// // Convolving with an impulse (a single unit tap) is identity.
/// let mut conv = FirConvolver::new(&[1.0]);
/// let mut block = [0.5, -0.25, 0.125];
/// conv.process_in_place(&mut block);
/// assert_eq!(block, [0.5, -0.25, 0.125]);
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct FirConvolver {
    /// Impulse response taps, `taps[0]` aligned with the current input sample.
    /// Sized to `capacity`; only the first `active_len` entries are live.
    taps: Vec<Sample>,
    /// Ring buffer of past inputs, sized to `capacity`.
    history: Vec<Sample>,
    /// Index in `history` where the next input sample is written.
    write_index: usize,
    /// Number of live taps (`<= capacity`).
    active_len: usize,
    bypassed: bool,
}

impl FirConvolver {
    /// Create a convolver whose impulse response is `taps` (and whose capacity
    /// equals its length). `taps[0]` is applied to the current input sample,
    /// `taps[1]` to the previous one, and so on.
    ///
    /// An empty `taps` slice yields a convolver that outputs silence until an
    /// impulse response is installed.
    pub fn new(taps: &[Sample]) -> Self {
        let mut convolver = Self::with_capacity(taps.len());
        convolver.set_impulse_response(taps);
        convolver
    }

    /// Create a convolver that can hold up to `max_taps` coefficients, with no
    /// impulse response installed yet (it outputs silence until one is set via
    /// [`set_impulse_response`](Self::set_impulse_response)).
    ///
    /// Use this to preallocate for the longest impulse response a consumer will
    /// swap in, so later swaps stay real-time safe.
    pub fn with_capacity(max_taps: usize) -> Self {
        Self {
            taps: vec![0.0; max_taps],
            history: vec![0.0; max_taps],
            write_index: 0,
            active_len: 0,
            bypassed: false,
        }
    }

    /// The maximum number of taps this convolver can hold.
    pub fn capacity(&self) -> usize {
        self.taps.len()
    }

    /// The number of live taps in the current impulse response.
    pub fn len(&self) -> usize {
        self.active_len
    }

    /// Whether no impulse response is installed (the convolver outputs silence).
    pub fn is_empty(&self) -> bool {
        self.active_len == 0
    }

    /// The impulse-response tail length: how many output samples continue after
    /// the input goes silent (`len().saturating_sub(1)`). Useful for sizing
    /// flush/ring-out regions when a source stops.
    pub fn tail_len(&self) -> usize {
        self.active_len.saturating_sub(1)
    }

    /// Install a new impulse response, copying up to `capacity` taps.
    ///
    /// Real-time safe: no allocation, no history clear (the running convolution
    /// stays continuous across a swap — a consumer that needs a click-free
    /// change between very different responses should crossfade two convolvers).
    /// Returns `true` if `taps` fit within `capacity`; when it is longer the
    /// response is truncated to `capacity` taps and `false` is returned.
    pub fn set_impulse_response(&mut self, taps: &[Sample]) -> bool {
        let fit = taps.len() <= self.capacity();
        let count = taps.len().min(self.capacity());
        self.taps[..count].copy_from_slice(&taps[..count]);
        for tap in &mut self.taps[count..] {
            *tap = 0.0;
        }
        self.active_len = count;
        fit
    }

    /// Convolve one input sample and return the output.
    ///
    /// When bypassed the sample passes through unchanged while the history
    /// buffer keeps advancing, so state stays coherent across bypass changes.
    pub fn process_sample(&mut self, input: Sample) -> Sample {
        let capacity = self.history.len();
        if capacity == 0 {
            return if self.bypassed { input } else { 0.0 };
        }

        self.history[self.write_index] = input;

        let output = if self.bypassed || self.active_len == 0 {
            if self.bypassed {
                input
            } else {
                0.0
            }
        } else {
            // Walk backwards from the newest sample: taps[0]·x[n], taps[1]·x[n-1]…
            // Split at the ring boundary so the inner loops carry no wrap test.
            let mut acc = 0.0;
            let mut tap = 0usize;
            let mut index = self.write_index;
            loop {
                acc += self.taps[tap] * self.history[index];
                tap += 1;
                if tap >= self.active_len || index == 0 {
                    break;
                }
                index -= 1;
            }
            if tap < self.active_len {
                // Continue from the top of the ring (guaranteed in range because
                // active_len <= capacity).
                index = capacity - 1;
                while tap < self.active_len {
                    acc += self.taps[tap] * self.history[index];
                    tap += 1;
                    index -= 1;
                }
            }
            flush_denormal(acc)
        };

        self.write_index += 1;
        if self.write_index == capacity {
            self.write_index = 0;
        }
        output
    }
}

impl DspKernel for FirConvolver {
    fn reset(&mut self) {
        self.history.fill(0.0);
        self.write_index = 0;
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
    use super::FirConvolver;
    use crate::DspKernel;

    /// Straightforward reference convolution over the full input history.
    fn naive_convolution(taps: &[f32], input: &[f32]) -> Vec<f32> {
        (0..input.len())
            .map(|n| {
                let mut acc = 0.0;
                for (k, &tap) in taps.iter().enumerate() {
                    if n >= k {
                        acc += tap * input[n - k];
                    }
                }
                acc
            })
            .collect()
    }

    #[test]
    fn unit_impulse_response_is_identity() {
        let mut conv = FirConvolver::new(&[1.0]);
        let mut block = [0.5, -0.25, 0.125, 1.0];
        conv.process_in_place(&mut block);
        assert_eq!(block, [0.5, -0.25, 0.125, 1.0]);
    }

    #[test]
    fn impulse_input_returns_the_response() {
        // Feeding a unit impulse through a convolver emits its impulse response.
        let taps = [0.5, 0.25, -0.125, 0.0625];
        let mut conv = FirConvolver::new(&taps);
        let mut block = [1.0, 0.0, 0.0, 0.0, 0.0, 0.0];
        conv.process_in_place(&mut block);
        assert_eq!(&block[..4], &taps);
        assert_eq!(&block[4..], &[0.0, 0.0]);
    }

    #[test]
    fn matches_naive_convolution_over_ring_wrap() {
        let taps = [0.3, -0.6, 0.9, 0.2, -0.4];
        let input: Vec<f32> = (0..64)
            .map(|i| ((i as f32) * 0.37).sin() * 0.5 + if i % 7 == 0 { 0.8 } else { -0.1 })
            .collect();
        let expected = naive_convolution(&taps, &input);

        let mut conv = FirConvolver::new(&taps);
        let mut block = input.clone();
        conv.process_in_place(&mut block);

        for (got, want) in block.iter().zip(expected.iter()) {
            assert!((got - want).abs() < 1e-6, "got {got}, want {want}");
        }
    }

    #[test]
    fn tail_rings_out_after_input_stops() {
        let taps = [1.0, 0.5, 0.25];
        let mut conv = FirConvolver::new(&taps);
        assert_eq!(conv.tail_len(), 2);
        let mut block = [1.0, 0.0, 0.0];
        conv.process_in_place(&mut block);
        assert_eq!(block, taps);
    }

    #[test]
    fn with_capacity_before_response_outputs_silence() {
        let mut conv = FirConvolver::with_capacity(8);
        assert!(conv.is_empty());
        let mut block = [1.0, 0.5, -0.3];
        conv.process_in_place(&mut block);
        assert_eq!(block, [0.0, 0.0, 0.0]);

        // Installing a response later reuses the preallocated buffers. Reset first
        // to isolate from the history captured during the silent phase.
        assert!(conv.set_impulse_response(&[2.0, 1.0]));
        assert_eq!(conv.len(), 2);
        conv.reset();
        let mut block = [1.0, 0.0, 0.0];
        conv.process_in_place(&mut block);
        assert_eq!(block, [2.0, 1.0, 0.0]);
    }

    #[test]
    fn set_impulse_response_truncates_beyond_capacity() {
        let mut conv = FirConvolver::with_capacity(2);
        assert!(!conv.set_impulse_response(&[1.0, 2.0, 3.0]));
        assert_eq!(conv.len(), 2);
        let mut block = [1.0, 0.0, 0.0];
        conv.process_in_place(&mut block);
        // Only the first two taps survive.
        assert_eq!(block, [1.0, 2.0, 0.0]);
    }

    #[test]
    fn set_impulse_response_keeps_history_continuous() {
        // A swap must not clear the running convolution's history.
        let mut conv = FirConvolver::new(&[1.0, 1.0]);
        assert_eq!(conv.process_sample(1.0), 1.0); // taps[0]*1
        // Swap to a response that reads the previous sample; history retained.
        conv.set_impulse_response(&[0.0, 1.0]);
        // y = 0*x[n] + 1*x[n-1] = previous input (1.0).
        assert_eq!(conv.process_sample(0.0), 1.0);
    }

    #[test]
    fn bypass_passes_through_and_preserves_history() {
        let mut conv = FirConvolver::new(&[0.0, 1.0]);
        conv.set_bypassed(true);
        let mut block = [0.1, 0.2, 0.3];
        conv.process_in_place(&mut block);
        assert_eq!(block, [0.1, 0.2, 0.3]);

        // Un-bypass: the delayed tap now reads the history captured while bypassed.
        conv.set_bypassed(false);
        assert_eq!(conv.process_sample(0.0), 0.3);
    }

    #[test]
    fn reset_clears_history() {
        let mut conv = FirConvolver::new(&[0.0, 1.0]);
        assert_eq!(conv.process_sample(0.9), 0.0);
        conv.reset();
        // History cleared: the delayed tap reads zero, not 0.9.
        assert_eq!(conv.process_sample(0.0), 0.0);
    }

    #[test]
    fn empty_response_outputs_silence_unless_bypassed() {
        let mut conv = FirConvolver::new(&[]);
        assert!(conv.is_empty());
        assert_eq!(conv.process_sample(1.0), 0.0);
        conv.set_bypassed(true);
        assert_eq!(conv.process_sample(1.0), 1.0);
    }
}
