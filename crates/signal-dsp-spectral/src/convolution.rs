//! Uniform-partitioned frequency-domain convolution.
//!
//! [`PartitionedConvolver`] convolves an input stream with a long impulse
//! response by partitioning the response into fixed-size blocks, transforming
//! each to the frequency domain once, and running a frequency-domain delay line
//! with a complex multiply-accumulate per processed block. This is the standard
//! technique for reverb-length impulse responses, where direct-form
//! time-domain convolution (see `signal_dsp::FirConvolver`) costs one
//! multiply-add per tap per sample.
//!
//! The convolver processes fixed blocks of `block_size` samples via
//! [`process_block`](PartitionedConvolver::process_block) and adds no latency
//! beyond that block granularity — with an identity impulse response the output
//! block equals the input block. Cost per block is `O(partitions × fft_size)`,
//! i.e. `O(ir_len / block_size)` per sample rather than `O(ir_len)`.
//!
//! Real-time safe: every buffer, FFT plan, and scratch region is allocated once
//! in [`PartitionedConvolver::new`]; [`process_block`](PartitionedConvolver::process_block)
//! never allocates.

use rustfft::{num_complex::Complex32, Fft, FftPlanner};
use signal_primitives::Sample;
use std::sync::Arc;

/// Magnitude below which convolver output is flushed to zero, mirroring
/// `signal_dsp::DENORMAL_THRESHOLD` (kept local to avoid a crate dependency).
const DENORMAL_THRESHOLD: Sample = 1.0e-20;

/// A uniform-partitioned overlap-save frequency-domain convolver.
///
/// Convolves a single channel with a fixed impulse response of any length.
/// Construct it with the response and a processing block size, then feed audio
/// one `block_size`-sample block at a time through
/// [`process_block`](Self::process_block).
///
/// ```
/// use signal_dsp_spectral::PartitionedConvolver;
///
/// // An identity impulse response passes blocks through unchanged.
/// let mut conv = PartitionedConvolver::new(&[1.0], 4);
/// let mut block = [0.5, -0.25, 0.125, 1.0];
/// conv.process_block(&mut block);
/// for (got, want) in block.iter().zip([0.5, -0.25, 0.125, 1.0]) {
///     assert!((got - want).abs() < 1e-4);
/// }
/// ```
pub struct PartitionedConvolver {
    block_size: usize,
    fft_size: usize,
    partitions: usize,
    ir_len: usize,
    forward: Arc<dyn Fft<f32>>,
    inverse: Arc<dyn Fft<f32>>,
    /// Precomputed impulse-response partition spectra, `partitions × fft_size`
    /// complex bins laid out contiguously (partition `p` at `p * fft_size`).
    ir_spectra: Vec<Complex32>,
    /// Frequency-domain delay line: a ring of the last `partitions` input
    /// spectra, same layout as `ir_spectra`.
    fdl: Vec<Complex32>,
    /// Ring slot holding the most recent input spectrum.
    fdl_head: usize,
    /// Overlap-save time-domain window (previous block followed by current).
    window: Vec<Complex32>,
    /// The previous block's input samples (the overlap-save carry).
    prev_block: Vec<Sample>,
    /// In-place FFT scratch, sized for the larger of the two plans.
    fft_scratch: Vec<Complex32>,
    /// Frequency-domain accumulator, reused as the inverse-FFT buffer.
    accum: Vec<Complex32>,
    bypassed: bool,
}

impl std::fmt::Debug for PartitionedConvolver {
    /// Reports the partitioning shape. The FFT plans are foreign trait
    /// objects and the spectra, delay line, and scratch are large working
    /// buffers, so neither is formatted.
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("PartitionedConvolver")
            .field("block_size", &self.block_size)
            .field("fft_size", &self.fft_size)
            .field("partitions", &self.partitions)
            .field("ir_len", &self.ir_len)
            .field("bypassed", &self.bypassed)
            .finish_non_exhaustive()
    }
}

impl PartitionedConvolver {
    /// Build a convolver for `impulse_response`, processing `block_size` samples
    /// per call. The FFT size is `2 × block_size`; the response is split into
    /// `ceil(ir_len / block_size)` partitions whose spectra are precomputed here.
    ///
    /// `block_size` is clamped to at least 1. An empty impulse response yields a
    /// convolver that outputs silence.
    pub fn new(impulse_response: &[Sample], block_size: usize) -> Self {
        let block_size = block_size.max(1);
        let fft_size = block_size * 2;
        let partitions = impulse_response.len().div_ceil(block_size);

        let mut planner = FftPlanner::<f32>::new();
        let forward = planner.plan_fft_forward(fft_size);
        let inverse = planner.plan_fft_inverse(fft_size);
        let scratch_len = forward
            .get_inplace_scratch_len()
            .max(inverse.get_inplace_scratch_len());
        let mut fft_scratch = vec![Complex32::default(); scratch_len];

        // Precompute each IR partition's spectrum: block_size taps zero-padded to
        // fft_size, forward-transformed once.
        let mut ir_spectra = vec![Complex32::default(); partitions * fft_size];
        for partition in 0..partitions {
            let slot = &mut ir_spectra[partition * fft_size..partition * fft_size + fft_size];
            for (offset, bin) in slot.iter_mut().take(block_size).enumerate() {
                let index = partition * block_size + offset;
                let sample = impulse_response.get(index).copied().unwrap_or(0.0);
                *bin = Complex32::new(sample, 0.0);
            }
            forward.process_with_scratch(slot, &mut fft_scratch);
        }

        Self {
            block_size,
            fft_size,
            partitions,
            ir_len: impulse_response.len(),
            forward,
            inverse,
            ir_spectra,
            fdl: vec![Complex32::default(); partitions * fft_size],
            // First process_block advances to slot 0.
            fdl_head: partitions.saturating_sub(1),
            window: vec![Complex32::default(); fft_size],
            prev_block: vec![0.0; block_size],
            fft_scratch,
            accum: vec![Complex32::default(); fft_size],
            bypassed: false,
        }
    }

    /// The processing block size (samples per [`process_block`](Self::process_block) call).
    pub fn block_size(&self) -> usize {
        self.block_size
    }

    /// The number of impulse-response partitions.
    pub fn partitions(&self) -> usize {
        self.partitions
    }

    /// The impulse-response length in samples.
    pub fn ir_len(&self) -> usize {
        self.ir_len
    }

    /// Whether the impulse response is empty (the convolver outputs silence).
    pub fn is_empty(&self) -> bool {
        self.partitions == 0
    }

    /// Set the bypass flag. While bypassed, [`process_block`](Self::process_block)
    /// passes input through and skips the transform work; the frequency-domain
    /// state is not advanced, so un-bypassing restarts the convolution tail.
    pub fn set_bypassed(&mut self, bypassed: bool) {
        self.bypassed = bypassed;
    }

    /// Report the current bypass flag.
    pub fn is_bypassed(&self) -> bool {
        self.bypassed
    }

    /// Clear all running state: the overlap carry and the frequency-domain delay
    /// line. The precomputed impulse-response spectra are retained.
    pub fn reset(&mut self) {
        self.prev_block.fill(0.0);
        self.fdl.fill(Complex32::default());
        self.fdl_head = self.partitions.saturating_sub(1);
    }

    /// Convolve one block of exactly `block_size` samples in place.
    ///
    /// # Panics
    /// Panics if `block.len() != block_size`.
    pub fn process_block(&mut self, block: &mut [Sample]) {
        assert_eq!(
            block.len(),
            self.block_size,
            "process_block requires exactly block_size samples"
        );

        if self.bypassed {
            return;
        }
        if self.partitions == 0 {
            block.fill(0.0);
            return;
        }

        let block_size = self.block_size;
        let fft_size = self.fft_size;

        // Overlap-save input window: [previous block | current block].
        for (slot, &sample) in self.window[..block_size]
            .iter_mut()
            .zip(self.prev_block.iter())
        {
            *slot = Complex32::new(sample, 0.0);
        }
        for (slot, &sample) in self.window[block_size..].iter_mut().zip(block.iter()) {
            *slot = Complex32::new(sample, 0.0);
        }
        self.prev_block.copy_from_slice(block);

        // Transform the window into the newest frequency-domain delay-line slot.
        self.fdl_head = (self.fdl_head + 1) % self.partitions;
        let head = self.fdl_head;
        {
            let slot = &mut self.fdl[head * fft_size..head * fft_size + fft_size];
            slot.copy_from_slice(&self.window);
            self.forward
                .process_with_scratch(slot, &mut self.fft_scratch);
        }

        // Y = Σ_p H[p] · X[head − p]: partition p multiplies the input spectrum
        // from p blocks ago.
        for bin in &mut self.accum {
            *bin = Complex32::default();
        }
        for partition in 0..self.partitions {
            let delayed = (head + self.partitions - partition) % self.partitions;
            let ir = &self.ir_spectra[partition * fft_size..partition * fft_size + fft_size];
            let x = &self.fdl[delayed * fft_size..delayed * fft_size + fft_size];
            for ((acc, &h), &xv) in self.accum.iter_mut().zip(ir.iter()).zip(x.iter()) {
                *acc += h * xv;
            }
        }

        // Inverse transform; the last block_size samples are the valid linear
        // convolution (overlap-save discards the aliased first half). rustfft's
        // inverse is unnormalized, so scale by 1/fft_size.
        self.inverse
            .process_with_scratch(&mut self.accum, &mut self.fft_scratch);
        let scale = 1.0 / fft_size as f32;
        for (out, bin) in block.iter_mut().zip(self.accum[block_size..].iter()) {
            let value = bin.re * scale;
            *out = if value.abs() < DENORMAL_THRESHOLD {
                0.0
            } else {
                value
            };
        }
    }
}

/// A [`PartitionedConvolver`] wrapper that accepts arbitrary block sizes.
///
/// Real-time hosts deliver audio in callback blocks whose length rarely matches
/// a convolver's internal partition size. `StreamingConvolver` buffers input to
/// the fixed block boundary, so consumers can call
/// [`process_in_place`](Self::process_in_place) with any slice length.
///
/// The cost is a fixed processing latency of one block: an output sample is only
/// available once its whole block has been collected. Query it with
/// [`latency`](Self::latency) to align against dry/parallel paths.
///
/// Real-time safe: the wrapped convolver and both block buffers allocate once in
/// [`new`](Self::new); [`process_in_place`](Self::process_in_place) never
/// allocates.
///
/// ```
/// use signal_dsp_spectral::StreamingConvolver;
///
/// let mut conv = StreamingConvolver::new(&[1.0], 4);
/// assert_eq!(conv.latency(), 4);
/// // The first `latency` samples are the priming delay; input then emerges.
/// let mut audio = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0];
/// conv.process_in_place(&mut audio);
/// for (got, want) in audio[4..].iter().zip([1.0, 2.0, 3.0, 4.0]) {
///     assert!((got - want).abs() < 1e-4);
/// }
/// ```
#[derive(Debug)]
pub struct StreamingConvolver {
    convolver: PartitionedConvolver,
    block_size: usize,
    /// Input samples collected toward the next full block.
    collect: Vec<Sample>,
    /// The most recently processed output block, drained as new input arrives.
    ready: Vec<Sample>,
    /// Fill position shared by `collect` (writing) and `ready` (reading).
    position: usize,
    bypassed: bool,
}

impl StreamingConvolver {
    /// Build a streaming convolver for `impulse_response` with an internal
    /// partition size of `block_size` (see [`PartitionedConvolver::new`]).
    pub fn new(impulse_response: &[Sample], block_size: usize) -> Self {
        let convolver = PartitionedConvolver::new(impulse_response, block_size);
        let block_size = convolver.block_size();
        Self {
            convolver,
            block_size,
            collect: vec![0.0; block_size],
            ready: vec![0.0; block_size],
            position: 0,
            bypassed: false,
        }
    }

    /// The fixed processing latency in samples (equal to the internal block size).
    pub fn latency(&self) -> usize {
        self.block_size
    }

    /// The impulse-response length in samples.
    pub fn ir_len(&self) -> usize {
        self.convolver.ir_len()
    }

    /// Whether the impulse response is empty.
    pub fn is_empty(&self) -> bool {
        self.convolver.is_empty()
    }

    /// Set the bypass flag. While bypassed, [`process_in_place`](Self::process_in_place)
    /// passes input through immediately (dropping the latency alignment); the
    /// internal buffers are not advanced.
    pub fn set_bypassed(&mut self, bypassed: bool) {
        self.bypassed = bypassed;
    }

    /// Report the current bypass flag.
    pub fn is_bypassed(&self) -> bool {
        self.bypassed
    }

    /// Clear all running state, including the priming delay and the wrapped
    /// convolver's frequency-domain state.
    pub fn reset(&mut self) {
        self.convolver.reset();
        self.collect.fill(0.0);
        self.ready.fill(0.0);
        self.position = 0;
    }

    /// Convolve a block of any length in place. The output lags the input by
    /// [`latency`](Self::latency) samples.
    pub fn process_in_place(&mut self, block: &mut [Sample]) {
        if self.bypassed {
            return;
        }
        for sample in block {
            let output = self.ready[self.position];
            self.collect[self.position] = *sample;
            self.position += 1;
            if self.position == self.block_size {
                self.convolver.process_block(&mut self.collect);
                std::mem::swap(&mut self.ready, &mut self.collect);
                self.position = 0;
            }
            *sample = output;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{PartitionedConvolver, StreamingConvolver};

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

    /// Run a convolver over a whole signal in block_size chunks (zero-padded to
    /// a block multiple) and return the concatenated output.
    fn run_blocked(conv: &mut PartitionedConvolver, input: &[f32]) -> Vec<f32> {
        let block = conv.block_size();
        let mut out = Vec::new();
        let mut buffer = vec![0.0f32; block];
        let mut index = 0;
        while index < input.len() {
            for (slot, src) in buffer
                .iter_mut()
                .zip(input[index..].iter().chain(std::iter::repeat(&0.0)))
            {
                *slot = *src;
            }
            conv.process_block(&mut buffer);
            out.extend_from_slice(&buffer);
            index += block;
        }
        out
    }

    #[test]
    fn identity_response_passes_blocks_through() {
        let mut conv = PartitionedConvolver::new(&[1.0], 4);
        assert_eq!(conv.block_size(), 4);
        assert_eq!(conv.partitions(), 1);
        let mut block = [0.5, -0.25, 0.125, 1.0];
        conv.process_block(&mut block);
        for (got, want) in block.iter().zip([0.5, -0.25, 0.125, 1.0]) {
            assert!((got - want).abs() < 1e-4, "got {got}, want {want}");
        }
    }

    #[test]
    fn single_partition_matches_naive() {
        // IR shorter than one block: exercises the P == 1 path.
        let taps = [0.5, 0.25, -0.125];
        let input: Vec<f32> = (0..32).map(|i| ((i as f32) * 0.3).sin()).collect();
        let expected = naive_convolution(&taps, &input);

        let mut conv = PartitionedConvolver::new(&taps, 8);
        let out = run_blocked(&mut conv, &input);
        for (got, want) in out.iter().zip(expected.iter()) {
            assert!((got - want).abs() < 1e-3, "got {got}, want {want}");
        }
    }

    #[test]
    fn multi_partition_matches_naive_no_latency() {
        // IR spanning several partitions; verify sample-aligned (no added latency).
        let taps: Vec<f32> = (0..37).map(|i| ((i as f32) * 0.21).cos() * 0.4).collect();
        let input: Vec<f32> = (0..96)
            .map(|i| {
                if i % 5 == 0 {
                    0.7
                } else {
                    ((i as f32) * 0.11).sin() * 0.3
                }
            })
            .collect();
        let expected = naive_convolution(&taps, &input);

        let block = 8;
        let mut conv = PartitionedConvolver::new(&taps, block);
        assert_eq!(conv.partitions(), 37usize.div_ceil(block));
        let out = run_blocked(&mut conv, &input);
        for (n, want) in expected.iter().enumerate() {
            assert!(
                (out[n] - want).abs() < 1e-3,
                "sample {n}: got {}, want {want}",
                out[n]
            );
        }
    }

    #[test]
    fn delayed_impulse_response_delays_signal() {
        // IR = unit impulse at tap 10: output is the input delayed by 10 samples.
        let mut taps = vec![0.0; 11];
        taps[10] = 1.0;
        let input: Vec<f32> = (0..64).map(|i| (i as f32 + 1.0) * 0.01).collect();

        let mut conv = PartitionedConvolver::new(&taps, 4);
        let out = run_blocked(&mut conv, &input);
        for n in 10..input.len() {
            assert!((out[n] - input[n - 10]).abs() < 1e-4, "sample {n}");
        }
        for &early in &out[..10] {
            assert!(early.abs() < 1e-4);
        }
    }

    #[test]
    fn reset_clears_running_state() {
        let taps = [0.3, 0.6, 0.9, 0.2, -0.5];
        let mut conv = PartitionedConvolver::new(&taps, 4);
        let mut block = [1.0, 0.0, 0.0, 0.0];
        conv.process_block(&mut block);
        conv.reset();
        // After reset, a fresh impulse produces the head of the IR again.
        let mut block = [1.0, 0.0, 0.0, 0.0];
        conv.process_block(&mut block);
        for (got, want) in block.iter().zip(&taps[..4]) {
            assert!((got - want).abs() < 1e-3, "got {got}, want {want}");
        }
    }

    #[test]
    fn empty_response_outputs_silence() {
        let mut conv = PartitionedConvolver::new(&[], 4);
        assert!(conv.is_empty());
        let mut block = [1.0, -1.0, 0.5, 0.25];
        conv.process_block(&mut block);
        assert_eq!(block, [0.0, 0.0, 0.0, 0.0]);
    }

    #[test]
    fn bypass_passes_through() {
        let mut conv = PartitionedConvolver::new(&[0.5, 0.5], 4);
        conv.set_bypassed(true);
        let mut block = [0.1, 0.2, 0.3, 0.4];
        conv.process_block(&mut block);
        assert_eq!(block, [0.1, 0.2, 0.3, 0.4]);
    }

    #[test]
    #[should_panic(expected = "block_size")]
    fn wrong_block_length_panics() {
        let mut conv = PartitionedConvolver::new(&[1.0], 4);
        let mut block = [0.0; 3];
        conv.process_block(&mut block);
    }

    #[test]
    fn streaming_matches_naive_with_block_latency() {
        // A streaming convolver over arbitrary chunk sizes reproduces the naive
        // linear convolution, delayed by exactly one block (the latency).
        let taps: Vec<f32> = (0..40).map(|i| ((i as f32) * 0.17).sin() * 0.3).collect();
        let block = 8;
        let input: Vec<f32> = (0..200)
            .map(|i| {
                if i % 9 == 0 {
                    0.6
                } else {
                    ((i as f32) * 0.07).cos() * 0.25
                }
            })
            .collect();
        let expected = naive_convolution(&taps, &input);

        let mut conv = StreamingConvolver::new(&taps, block);
        assert_eq!(conv.latency(), block);

        // Feed in irregular chunk sizes to exercise the buffering.
        let mut out = Vec::new();
        let mut buffer = input.clone();
        let mut cursor = 0;
        for &chunk in &[3usize, 8, 1, 16, 5, 32, 7] {
            let mut remaining = chunk;
            while remaining > 0 && cursor < buffer.len() {
                let take = remaining.min(buffer.len() - cursor);
                conv.process_in_place(&mut buffer[cursor..cursor + take]);
                out.extend_from_slice(&buffer[cursor..cursor + take]);
                cursor += take;
                remaining -= take;
            }
        }
        // Feed the tail in one call.
        conv.process_in_place(&mut buffer[cursor..]);
        out.extend_from_slice(&buffer[cursor..]);

        // out[n + latency] == expected[n].
        for n in 0..(input.len() - block) {
            assert!(
                (out[n + block] - expected[n]).abs() < 1e-3,
                "sample {n}: got {}, want {}",
                out[n + block],
                expected[n]
            );
        }
    }

    #[test]
    fn streaming_primes_with_latency_of_silence() {
        let mut conv = StreamingConvolver::new(&[1.0, 0.5], 4);
        let mut block = [1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0];
        conv.process_in_place(&mut block);
        // First `latency` (4) samples are the priming delay.
        assert_eq!(&block[..4], &[0.0, 0.0, 0.0, 0.0]);
        // Then the impulse response emerges: [1.0, 0.5, 0.0, 0.0].
        assert!((block[4] - 1.0).abs() < 1e-4);
        assert!((block[5] - 0.5).abs() < 1e-4);
    }

    #[test]
    fn streaming_bypass_passes_through_without_latency() {
        let mut conv = StreamingConvolver::new(&[0.5, 0.5], 4);
        conv.set_bypassed(true);
        let mut block = [0.1, 0.2, 0.3, 0.4, 0.5];
        conv.process_in_place(&mut block);
        assert_eq!(block, [0.1, 0.2, 0.3, 0.4, 0.5]);
    }

    #[test]
    fn streaming_reset_reprimes() {
        let taps = [0.4, 0.3, 0.2];
        let mut conv = StreamingConvolver::new(&taps, 4);
        let mut warmup = [1.0, 0.5, -0.5, 0.25, 0.0, 0.0, 0.0, 0.0];
        conv.process_in_place(&mut warmup);
        conv.reset();
        // After reset the priming delay returns: first block is silence again.
        let mut block = [1.0, 0.0, 0.0, 0.0];
        conv.process_in_place(&mut block);
        assert_eq!(block, [0.0, 0.0, 0.0, 0.0]);
    }
}
