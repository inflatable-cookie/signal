use std::sync::Arc;

use rustfft::{num_complex::Complex32, Fft, FftPlanner};
use signal_primitives::Sample;

/// Run the draft phase-vocoder backend.
pub(crate) fn phase_vocoder(
    input: &[Sample],
    target_len: usize,
    ratio: f64,
    window_size: usize,
    analysis_hop: usize,
) -> Vec<Sample> {
    run_phase_vocoder(
        input,
        target_len,
        ratio,
        window_size,
        analysis_hop,
        PhasePropagationMode::IndependentBins,
    )
}

/// Run the identity phase-locked phase-vocoder prototype.
#[cfg(any(test, feature = "evidence"))]
pub(crate) fn phase_locked_phase_vocoder(
    input: &[Sample],
    target_len: usize,
    ratio: f64,
    window_size: usize,
    analysis_hop: usize,
) -> Vec<Sample> {
    run_phase_vocoder(
        input,
        target_len,
        ratio,
        window_size,
        analysis_hop,
        PhasePropagationMode::IdentityLocked,
    )
}

/// `g10.041` Batch 41.3 candidate: transient reset above a crossover only.
///
/// `crossover_fraction` is a fraction of Nyquist. Bins below it propagate
/// continuously through a transient instead of being reset.
///
/// Evidence-only until listening admits it. Contract `084` Rule 2 keeps a
/// candidate isolated, and Rule 5 makes listening the promotion authority, so
/// an unadopted candidate having no production caller is the intended state
/// rather than an oversight. It is reachable under the `evidence` feature so a
/// listening pack can render it.
#[cfg(any(test, feature = "evidence"))]
pub(crate) fn high_band_transient_reset_phase_vocoder(
    input: &[Sample],
    target_len: usize,
    ratio: f64,
    window_size: usize,
    analysis_hop: usize,
    crossover_fraction: f64,
) -> Vec<Sample> {
    let bins = window_size / 2 + 1;
    let crossover_bin = (crossover_fraction * bins as f64).ceil().max(0.0) as usize;
    let mode = if ratio < 1.0 {
        PhasePropagationMode::IdentityLocked
    } else {
        PhasePropagationMode::IdentityLockedTransientResetHighBand { crossover_bin }
    };
    run_phase_vocoder(input, target_len, ratio, window_size, analysis_hop, mode)
}

/// Crossover for the transient phase reset, as a fraction of Nyquist.
///
/// `240 Hz` at `48 kHz`. Bins below it propagate continuously through a
/// transient; bins above it reset.
///
/// Admitted by listening on 2026-08-05 (`g10.041`). Low-frequency content is
/// sustained *through* a transient — a bass note rings on while the attack
/// happens — so resetting its phase destroys continuity in something that never
/// restarted, which is finding `A18`. High-frequency content *is* the transient,
/// and resetting it is what stops smearing.
///
/// Bounded on both sides by measurement: below about `120 Hz` the artifact
/// returns, and at `504 Hz` transient smear regresses because the reset stops
/// reaching content that should restart.
pub const TRANSIENT_RESET_CROSSOVER_FRACTION: f64 = 0.010;

/// Run the identity phase-locked prototype with transient phase resets.
pub(crate) fn transient_reset_phase_vocoder(
    input: &[Sample],
    target_len: usize,
    ratio: f64,
    window_size: usize,
    analysis_hop: usize,
) -> Vec<Sample> {
    let bins = window_size / 2 + 1;
    let crossover_bin = (TRANSIENT_RESET_CROSSOVER_FRACTION * bins as f64).ceil() as usize;
    let mode = if ratio < 1.0 {
        PhasePropagationMode::IdentityLocked
    } else {
        PhasePropagationMode::IdentityLockedTransientResetHighBand { crossover_bin }
    };
    run_phase_vocoder(input, target_len, ratio, window_size, analysis_hop, mode)
}

/// Run the OfflineHighQuality prototype over interleaved stereo with a linked
/// mid/side analysis surface instead of independent left/right stretching.
pub(crate) fn transient_reset_phase_vocoder_linked_stereo(
    input_interleaved: &[Sample],
    target_frames: usize,
    ratio: f64,
    window_size: usize,
    analysis_hop: usize,
) -> Vec<Sample> {
    let frame_count = input_interleaved.len() / 2;
    if frame_count == 0 || target_frames == 0 {
        return Vec::new();
    }

    let mut mid = Vec::with_capacity(frame_count);
    let mut side = Vec::with_capacity(frame_count);
    for frame in input_interleaved.chunks_exact(2) {
        let left = frame[0];
        let right = frame[1];
        mid.push((left + right) * 0.5);
        side.push((left - right) * 0.5);
    }

    let stretched_mid =
        transient_reset_phase_vocoder(&mid, target_frames, ratio, window_size, analysis_hop);
    let stretched_side =
        transient_reset_phase_vocoder(&side, target_frames, ratio, window_size, analysis_hop);

    let out_frames = stretched_mid
        .len()
        .min(stretched_side.len())
        .min(target_frames);
    let mut output = Vec::with_capacity(target_frames * 2);
    for index in 0..out_frames {
        let mid = stretched_mid[index];
        let side = stretched_side[index];
        output.push(mid + side);
        output.push(mid - side);
    }
    output.resize(target_frames * 2, 0.0);
    output
}

/// Largest synthesis hop, as a fraction of the window, that keeps overlap-add
/// coverage intact. Above this the window-power normalization gate zeroes
/// output samples. Frozen by Contract `046`, 2026-07-27 addendum.
pub(crate) const MAX_SYNTHESIS_HOP_WINDOW_FRACTION: f64 = 0.75;

/// Analysis hop that keeps `analysis_hop * ratio` inside the overlap coverage
/// bound. Returns `analysis_hop` unchanged whenever the configured geometry
/// already satisfies it, so ratios inside the bound stay byte-exact.
pub(crate) fn overlap_safe_analysis_hop(
    analysis_hop: usize,
    ratio: f64,
    window_size: usize,
) -> usize {
    if !ratio.is_finite() || ratio <= 0.0 {
        return analysis_hop;
    }
    let max_synthesis_hop = MAX_SYNTHESIS_HOP_WINDOW_FRACTION * window_size as f64;
    if analysis_hop as f64 * ratio <= max_synthesis_hop {
        return analysis_hop;
    }
    ((max_synthesis_hop / ratio).floor() as usize).clamp(1, analysis_hop)
}

fn run_phase_vocoder(
    input: &[Sample],
    target_len: usize,
    ratio: f64,
    window_size: usize,
    analysis_hop: usize,
    mode: PhasePropagationMode,
) -> Vec<Sample> {
    let analysis_hop = overlap_safe_analysis_hop(analysis_hop, ratio, window_size);
    // Give the first and last source samples complete overlapping analysis
    // windows. The extra post-roll hop guarantees that the cropped target
    // remains inside synthesized coverage for both compression and expansion.
    let prefix_frames = window_size / 2;
    let suffix_frames = window_size + analysis_hop;
    let mut padded_input = vec![0.0; prefix_frames + input.len() + suffix_frames];
    padded_input[prefix_frames..prefix_frames + input.len()].copy_from_slice(input);

    // Synthesis hops scale while the samples inside each synthesis window do
    // not, so the crop starts at the analysis window centre. The prefix is
    // exactly half a window, so the ratio-scaled centre offset is always zero
    // and the start is simply that half window; the earlier expression spelled
    // this as `((prefix - half_window) * ratio + half_window)`, which reads as
    // ratio-dependent but never was.
    let output_start = prefix_frames;
    let output_end = output_start + target_len;
    let config =
        PhaseVocoderConfig::new(&padded_input, output_end, ratio, window_size, analysis_hop);
    let mut engine = DraftPhaseVocoder::new(config, mode);
    engine.process(&padded_input);
    engine.finish()[output_start..output_end].to_vec()
}

struct PhaseVocoderConfig {
    target_len: usize,
    window_size: usize,
    analysis_hop: usize,
    synthesis_hop: f64,
    bins: usize,
    frame_count: usize,
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct SpectralPeak {
    bin: usize,
    magnitude: f32,
}

// `IdentityLockedTransientReset` resets every bin. It is no longer the shipped
// path — `g10.041` replaced it with the high-band variant after listening — but
// it is retained as the control the `A18` evidence compares against.
#[allow(dead_code)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PhasePropagationMode {
    IndependentBins,
    IdentityLocked,
    IdentityLockedTransientReset,
    /// `g10.041` Batch 41.3 candidate for `A18`.
    ///
    /// Resets transient phase only above a crossover, leaving low bins to
    /// propagate continuously. Low-frequency content is *sustained through* a
    /// transient — a bass note rings on while the attack happens — so resetting
    /// its phase destroys continuity in something that never restarted. High
    /// bins are the transient, and resetting them is what stops smearing.
    ///
    /// The crossover is a fraction of Nyquist rather than a frequency, because
    /// the stretch API is sample-rate agnostic all the way down.
    IdentityLockedTransientResetHighBand {
        crossover_bin: usize,
    },
}

impl PhaseVocoderConfig {
    fn new(
        input: &[Sample],
        target_len: usize,
        ratio: f64,
        window_size: usize,
        analysis_hop: usize,
    ) -> Self {
        Self {
            target_len,
            window_size,
            analysis_hop,
            synthesis_hop: analysis_hop as f64 * ratio,
            bins: window_size / 2 + 1,
            frame_count: (input.len().saturating_sub(window_size)) / analysis_hop + 1,
        }
    }
}

struct DraftPhaseVocoder {
    config: PhaseVocoderConfig,
    mode: PhasePropagationMode,
    window: Vec<f32>,
    analysis: PhaseVocoderAnalysisState,
    propagation: PhaseVocoderPropagationState,
    synthesis: PhaseVocoderSynthesisState,
}

struct PhaseVocoderAnalysisState {
    forward: Arc<dyn Fft<f32>>,
    /// Caller-owned FFT scratch. `Fft::process` allocates its own scratch on
    /// every call, which is two heap allocations per STFT frame in the hot
    /// loop; the RealtimePreview kernel already avoided that.
    scratch: Vec<Complex32>,
    current_magnitudes: Vec<f32>,
    current_phases: Vec<f32>,
    current_peaks: Vec<SpectralPeak>,
    previous_magnitudes: Vec<f32>,
    current_energy: f64,
    previous_energy: f64,
    transient_reset_current_frame: bool,
    buffer: Vec<Complex32>,
}

struct PhaseVocoderPropagationState {
    omega: Vec<f32>,
    previous_phase: Vec<f32>,
    synthesis_phase: Vec<f32>,
}

struct PhaseVocoderSynthesisState {
    inverse: Arc<dyn Fft<f32>>,
    scratch: Vec<Complex32>,
    spectrum: Vec<Complex32>,
    output: Vec<f32>,
    normalization: Vec<f32>,
}

impl DraftPhaseVocoder {
    fn new(config: PhaseVocoderConfig, mode: PhasePropagationMode) -> Self {
        let window: Vec<f32> = (0..config.window_size)
            .map(|index| {
                0.5 - 0.5 * (std::f32::consts::TAU * index as f32 / config.window_size as f32).cos()
            })
            .collect();
        let omega: Vec<f32> = (0..config.bins)
            .map(|bin| {
                std::f32::consts::TAU * bin as f32 * config.analysis_hop as f32
                    / config.window_size as f32
            })
            .collect();

        let mut planner = FftPlanner::<f32>::new();
        let forward = planner.plan_fft_forward(config.window_size);
        let inverse = planner.plan_fft_inverse(config.window_size);
        let final_synthesis_start =
            ((config.frame_count.saturating_sub(1)) as f64 * config.synthesis_hop).ceil() as usize;
        let ola_len = final_synthesis_start + config.window_size + 1;
        let output_len = ola_len.max(config.target_len);

        let analysis = PhaseVocoderAnalysisState {
            scratch: vec![Complex32::new(0.0, 0.0); forward.get_inplace_scratch_len()],
            forward,
            current_magnitudes: vec![0.0; config.bins],
            current_phases: vec![0.0; config.bins],
            current_peaks: Vec::with_capacity(config.bins / 4),
            previous_magnitudes: vec![0.0; config.bins],
            current_energy: 0.0,
            previous_energy: 0.0,
            transient_reset_current_frame: false,
            buffer: vec![Complex32::new(0.0, 0.0); config.window_size],
        };
        let propagation = PhaseVocoderPropagationState {
            omega,
            previous_phase: vec![0.0; config.bins],
            synthesis_phase: vec![0.0; config.bins],
        };
        let synthesis = PhaseVocoderSynthesisState {
            scratch: vec![Complex32::new(0.0, 0.0); inverse.get_inplace_scratch_len()],
            inverse,
            spectrum: vec![Complex32::new(0.0, 0.0); config.window_size],
            output: vec![0.0; output_len],
            normalization: vec![0.0; output_len],
        };

        Self {
            config,
            mode,
            window,
            analysis,
            propagation,
            synthesis,
        }
    }

    fn process(&mut self, input: &[Sample]) {
        for frame_index in 0..self.config.frame_count {
            self.analyze_frame(input, frame_index);
            self.track_spectral_peaks(frame_index);
            self.propagate_phase(frame_index);
            self.synthesize_frame(frame_index);
        }
    }

    fn analyze_frame(&mut self, input: &[Sample], frame_index: usize) {
        let analysis_start = frame_index * self.config.analysis_hop;
        self.analysis.current_energy = 0.0;
        for (slot, (sample, weight)) in self.analysis.buffer.iter_mut().zip(
            input[analysis_start..analysis_start + self.config.window_size]
                .iter()
                .zip(self.window.iter()),
        ) {
            let windowed = sample * weight;
            self.analysis.current_energy += (windowed * windowed) as f64;
            *slot = Complex32::new(windowed, 0.0);
        }
        self.analysis.current_energy /= self.config.window_size as f64;
        self.analysis
            .forward
            .process_with_scratch(&mut self.analysis.buffer, &mut self.analysis.scratch);
    }

    fn track_spectral_peaks(&mut self, frame_index: usize) {
        self.analysis.current_peaks.clear();
        for (bin, magnitude) in self.analysis.current_magnitudes.iter_mut().enumerate() {
            *magnitude = self.analysis.buffer[bin].norm();
        }
        self.analysis.transient_reset_current_frame =
            self.should_reset_phase_at_transient(frame_index);

        if self.config.bins < 3 {
            self.analysis
                .previous_magnitudes
                .copy_from_slice(&self.analysis.current_magnitudes);
            self.analysis.previous_energy = self.analysis.current_energy;
            return;
        }

        for bin in 1..self.config.bins - 1 {
            let magnitude = self.analysis.current_magnitudes[bin];
            if magnitude <= 1.0e-6 {
                continue;
            }
            if magnitude > self.analysis.current_magnitudes[bin - 1]
                && magnitude >= self.analysis.current_magnitudes[bin + 1]
            {
                self.analysis
                    .current_peaks
                    .push(SpectralPeak { bin, magnitude });
            }
        }

        self.analysis
            .previous_magnitudes
            .copy_from_slice(&self.analysis.current_magnitudes);
        self.analysis.previous_energy = self.analysis.current_energy;
    }

    fn propagate_phase(&mut self, frame_index: usize) {
        for bin in 0..self.config.bins {
            let phase = self.analysis.buffer[bin].arg();
            self.analysis.current_phases[bin] = phase;
            let reset_this_bin = self.analysis.transient_reset_current_frame
                && match self.mode {
                    PhasePropagationMode::IdentityLockedTransientResetHighBand {
                        crossover_bin,
                    } => bin >= crossover_bin,
                    _ => true,
                };
            if frame_index == 0 || reset_this_bin {
                self.propagation.synthesis_phase[bin] = phase;
            } else {
                let deviation = wrap_phase(
                    phase - self.propagation.previous_phase[bin] - self.propagation.omega[bin],
                );
                let advance = (self.propagation.omega[bin] + deviation)
                    * (self.config.synthesis_hop / self.config.analysis_hop as f64) as f32;
                self.propagation.synthesis_phase[bin] =
                    wrap_phase(self.propagation.synthesis_phase[bin] + advance);
            }
            self.propagation.previous_phase[bin] = phase;
        }

        if self.should_lock_phase_to_peaks() {
            self.lock_phase_to_peaks();
        }

        for bin in 0..self.config.bins {
            self.synthesis.spectrum[bin] = Complex32::from_polar(
                self.analysis.current_magnitudes[bin],
                self.propagation.synthesis_phase[bin],
            );
        }
        for bin in 1..self.config.window_size.div_ceil(2) {
            self.synthesis.spectrum[self.config.window_size - bin] =
                self.synthesis.spectrum[bin].conj();
        }
    }

    fn should_reset_phase_at_transient(&self, frame_index: usize) -> bool {
        if frame_index == 0 {
            return false;
        }

        let mut flux = 0.0f32;
        let mut magnitude_sum = 0.0f32;
        for (current, previous) in self
            .analysis
            .current_magnitudes
            .iter()
            .zip(self.analysis.previous_magnitudes.iter())
        {
            flux += (current - previous).max(0.0);
            magnitude_sum += *current;
        }

        let flux_ratio = flux as f64 / (magnitude_sum as f64 + 1.0e-12);
        let energy_ratio = self.analysis.current_energy / (self.analysis.previous_energy + 1.0e-12);
        match self.mode {
            PhasePropagationMode::IdentityLockedTransientReset
            | PhasePropagationMode::IdentityLockedTransientResetHighBand { .. } => {
                flux_ratio >= 0.30 && energy_ratio >= 1.20
            }
            PhasePropagationMode::IndependentBins | PhasePropagationMode::IdentityLocked => false,
        }
    }

    fn should_lock_phase_to_peaks(&self) -> bool {
        matches!(
            self.mode,
            PhasePropagationMode::IdentityLocked
                | PhasePropagationMode::IdentityLockedTransientReset
                | PhasePropagationMode::IdentityLockedTransientResetHighBand { .. }
        )
    }

    fn lock_phase_to_peaks(&mut self) {
        if self.analysis.current_peaks.is_empty() {
            return;
        }

        for peak_index in 0..self.analysis.current_peaks.len() {
            let peak = self.analysis.current_peaks[peak_index];
            let peak_phase = self.propagation.synthesis_phase[peak.bin];
            let analysis_peak_phase = self.analysis.current_phases[peak.bin];
            let (left, right) = self.peak_region_bounds(peak_index);

            for bin in left..right {
                let relative_phase =
                    wrap_phase(self.analysis.current_phases[bin] - analysis_peak_phase);
                self.propagation.synthesis_phase[bin] = wrap_phase(peak_phase + relative_phase);
            }
        }
    }

    fn peak_region_bounds(&self, peak_index: usize) -> (usize, usize) {
        let peak = self.analysis.current_peaks[peak_index];
        let left = if peak_index == 0 {
            0
        } else {
            (self.analysis.current_peaks[peak_index - 1].bin + peak.bin) / 2 + 1
        };
        let right = self
            .analysis
            .current_peaks
            .get(peak_index + 1)
            .map(|next| (peak.bin + next.bin) / 2 + 1)
            .unwrap_or(self.config.bins);
        (left, right)
    }

    fn synthesize_frame(&mut self, frame_index: usize) {
        self.synthesis
            .inverse
            .process_with_scratch(&mut self.synthesis.spectrum, &mut self.synthesis.scratch);
        let synthesis_start = (frame_index as f64 * self.config.synthesis_hop).round() as usize;
        let scale = 1.0 / self.config.window_size as f32;
        for (index, weight) in self.window.iter().enumerate() {
            let out_index = synthesis_start + index;
            if out_index >= self.synthesis.output.len() {
                break;
            }
            self.synthesis.output[out_index] += self.synthesis.spectrum[index].re * scale * weight;
            self.synthesis.normalization[out_index] += weight * weight;
        }
    }

    fn finish(mut self) -> Vec<Sample> {
        for (sample, weight) in self
            .synthesis
            .output
            .iter_mut()
            .zip(self.synthesis.normalization.iter())
        {
            if *weight > 1.0e-3 {
                *sample /= *weight;
            }
        }
        self.synthesis.output.resize(self.config.target_len, 0.0);
        self.synthesis.output
    }
}

/// Wrap a phase into `-PI..PI` by rounding.
///
/// See the crate-root `wrap_phase` for why this second implementation is
/// retained: the two forms are not bit-equivalent, so unifying them changes
/// rendered output and needs its own evidence.
fn wrap_phase(phase: f32) -> f32 {
    let tau = std::f32::consts::TAU;
    phase - tau * (phase / tau).round()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bin_centered_sine(bin: usize, window_size: usize) -> Vec<Sample> {
        (0..window_size)
            .map(|index| {
                (std::f32::consts::TAU * bin as f32 * index as f32 / window_size as f32).sin()
            })
            .collect()
    }

    fn dominant_frequency_hz(samples: &[Sample], sample_rate_hz: f32) -> f32 {
        let margin = samples.len() / 8;
        let interior = &samples[margin..samples.len() - margin];
        let crossings = interior
            .windows(2)
            .filter(|pair| (pair[0] >= 0.0) != (pair[1] >= 0.0))
            .count();
        crossings as f32 * sample_rate_hz / (2.0 * interior.len() as f32)
    }

    fn rms(samples: &[Sample]) -> f32 {
        (samples.iter().map(|sample| sample * sample).sum::<f32>() / samples.len().max(1) as f32)
            .sqrt()
    }

    fn boundary_content_probe(len: usize, edge_frames: usize) -> Vec<Sample> {
        let mut input = vec![0.0; len];
        input[..edge_frames].fill(0.5);
        input[len - edge_frames..].fill(-0.5);
        input
    }

    fn sample_bit_hash(samples: &[Sample]) -> u64 {
        samples.iter().fold(0xcbf2_9ce4_8422_2325, |hash, sample| {
            sample
                .to_bits()
                .to_le_bytes()
                .into_iter()
                .fold(hash, |hash, byte| {
                    (hash ^ u64::from(byte)).wrapping_mul(0x0000_0100_0000_01b3)
                })
        })
    }

    /// `A1` byte-exactness control. The overlap law must be a no-op wherever
    /// the configured geometry already satisfies it, so every ratio through
    /// `3.0` at the retained `2048/512` geometry keeps the pre-correction
    /// analysis hop and therefore the pre-correction output.
    ///
    /// This is asserted structurally rather than by output hash because f32
    /// render output differs between optimization profiles, so an absolute
    /// hash is only valid in the profile that captured it.
    #[test]
    fn overlap_safe_analysis_hop_is_a_no_op_through_ratio_three() {
        for ratio in [0.25, 0.5, 0.75, 1.0, 1.25, 1.5, 2.0, 2.5, 3.0] {
            assert_eq!(
                overlap_safe_analysis_hop(512, ratio, 2_048),
                512,
                "ratio {ratio} must keep the configured hop"
            );
        }
    }

    /// `A1`. Above ratio `3.0` the law reduces the hop so the synthesis hop
    /// stays inside `0.75 * window_size`.
    #[test]
    fn overlap_safe_analysis_hop_bounds_the_synthesis_hop() {
        let bound = MAX_SYNTHESIS_HOP_WINDOW_FRACTION * 2_048.0;
        for (ratio, expected) in [(3.5, 438), (4.0, 384), (6.0, 256), (8.0, 192), (16.0, 96)] {
            let hop = overlap_safe_analysis_hop(512, ratio, 2_048);
            assert_eq!(hop, expected, "ratio {ratio}");
            assert!(
                hop as f64 * ratio <= bound,
                "ratio {ratio}: synthesis hop {} passes the bound {bound}",
                hop as f64 * ratio
            );
        }
    }

    /// The law never enlarges a caller's hop and never returns zero.
    #[test]
    fn overlap_safe_analysis_hop_stays_within_caller_bounds() {
        assert_eq!(overlap_safe_analysis_hop(128, 64.0, 512), 6);
        assert_eq!(overlap_safe_analysis_hop(1, 1_000.0, 64), 1);
        assert_eq!(overlap_safe_analysis_hop(512, f64::NAN, 2_048), 512);
        assert_eq!(overlap_safe_analysis_hop(512, 0.0, 2_048), 512);
        assert_eq!(overlap_safe_analysis_hop(64, 0.5, 2_048), 64);
    }

    #[test]
    fn phase_vocoder_bit_exact_baseline() {
        let input = (0..8_192)
            .map(|index| {
                let fundamental =
                    (std::f32::consts::TAU * 11.0 * index as f32 / 2_048.0).sin() * 0.6;
                let partial = (std::f32::consts::TAU * 37.0 * index as f32 / 2_048.0).sin() * 0.2;
                let impulse = if index % 997 == 0 { 0.75 } else { 0.0 };
                fundamental + partial + impulse
            })
            .collect::<Vec<_>>();
        let output = transient_reset_phase_vocoder(&input, 12_288, 1.5, 2_048, 512);

        assert_eq!(sample_bit_hash(&output), 0x8255_b183_11f7_78f9);
    }

    #[test]
    fn phase_vocoder_boundary_expansion_preserves_head_and_tail_content() {
        let input = boundary_content_probe(48_000, 384);
        let output = transient_reset_phase_vocoder(&input, 96_000, 2.0, 2_048, 512);
        let edge_span = 2_048;

        assert!(rms(&output[..edge_span]) > 0.01);
        assert!(rms(&output[output.len() - edge_span..]) > 0.01);
    }

    #[test]
    fn phase_vocoder_boundary_compression_preserves_head_and_tail_content() {
        let input = boundary_content_probe(48_000, 384);
        let output = transient_reset_phase_vocoder(&input, 24_000, 0.5, 2_048, 512);
        let edge_span = 1_024;

        assert!(rms(&output[..edge_span]) > 0.01);
        assert!(rms(&output[output.len() - edge_span..]) > 0.01);
    }

    #[test]
    fn tracks_local_spectral_peaks_for_current_frame() {
        let window_size = 256;
        let target_bin = 17;
        let input = bin_centered_sine(target_bin, window_size);
        let config =
            PhaseVocoderConfig::new(&input, window_size, 1.0, window_size, window_size / 4);
        let mut engine = DraftPhaseVocoder::new(config, PhasePropagationMode::IndependentBins);

        engine.analyze_frame(&input, 0);
        engine.track_spectral_peaks(0);

        assert!(
            engine
                .analysis
                .current_peaks
                .iter()
                .any(|peak| peak.bin.abs_diff(target_bin) <= 1),
            "expected a peak near bin {target_bin}, got {:?}",
            engine.analysis.current_peaks
        );
    }

    #[test]
    fn transient_reset_detector_flags_energy_and_flux_jump() {
        let window_size = 256;
        let mut input = vec![0.0; window_size * 2];
        for sample in &mut input[window_size..] {
            *sample = 1.0;
        }
        let config = PhaseVocoderConfig::new(&input, input.len(), 1.0, window_size, window_size);
        let mut engine =
            DraftPhaseVocoder::new(config, PhasePropagationMode::IdentityLockedTransientReset);

        engine.analyze_frame(&input, 0);
        engine.track_spectral_peaks(0);
        assert!(!engine.analysis.transient_reset_current_frame);

        engine.analyze_frame(&input, 1);
        engine.track_spectral_peaks(1);
        assert!(engine.analysis.transient_reset_current_frame);
    }

    #[test]
    fn identity_locking_preserves_peak_neighborhood_phase_offsets() {
        let input = vec![0.0; 512];
        let config = PhaseVocoderConfig::new(&input, input.len(), 1.0, 512, 128);
        let mut engine = DraftPhaseVocoder::new(config, PhasePropagationMode::IdentityLocked);
        engine.analysis.current_peaks.push(SpectralPeak {
            bin: 10,
            magnitude: 1.0,
        });
        engine.analysis.current_phases[9] = 0.20;
        engine.analysis.current_phases[10] = 0.50;
        engine.analysis.current_phases[11] = 0.90;
        engine.propagation.synthesis_phase[10] = 1.25;

        engine.lock_phase_to_peaks();

        assert!((wrap_phase(engine.propagation.synthesis_phase[9] - 0.95)).abs() < 1.0e-6);
        assert!((wrap_phase(engine.propagation.synthesis_phase[10] - 1.25)).abs() < 1.0e-6);
        assert!((wrap_phase(engine.propagation.synthesis_phase[11] - 1.65)).abs() < 1.0e-6);
    }

    #[test]
    fn draft_phase_vocoder_keeps_independent_bin_baseline() {
        let input = bin_centered_sine(9, 4096);
        let draft = phase_vocoder(&input, 6144, 1.5, 512, 128);
        let baseline = run_phase_vocoder(
            &input,
            6144,
            1.5,
            512,
            128,
            PhasePropagationMode::IndependentBins,
        );

        assert_eq!(draft, baseline);
    }

    #[test]
    fn phase_locked_prototype_honors_output_length_contract() {
        let input = bin_centered_sine(11, 8192);
        for ratio in [0.5, 0.75, 1.25, 1.5, 2.0] {
            let target_len = (input.len() as f64 * ratio).round() as usize;
            let output = phase_locked_phase_vocoder(&input, target_len, ratio, 1024, 256);
            assert_eq!(output.len(), target_len, "ratio {ratio}");
        }
    }

    #[test]
    fn transient_reset_prototype_honors_output_length_contract() {
        let input = bin_centered_sine(11, 8192);
        for ratio in [0.5, 0.75, 1.25, 1.5, 2.0] {
            let target_len = (input.len() as f64 * ratio).round() as usize;
            let output = transient_reset_phase_vocoder(&input, target_len, ratio, 1024, 256);
            assert_eq!(output.len(), target_len, "ratio {ratio}");
        }
    }

    #[test]
    fn transient_reset_uses_phase_locking_for_time_compression() {
        let ratio = 0.75;
        let input = bin_centered_sine(11, 8192);
        let target_len = (input.len() as f64 * ratio).round() as usize;

        assert_eq!(
            transient_reset_phase_vocoder(&input, target_len, ratio, 1024, 256),
            phase_locked_phase_vocoder(&input, target_len, ratio, 1024, 256)
        );
    }

    #[test]
    fn phase_locked_prototype_preserves_tonal_pitch_near_draft_baseline() {
        let sample_rate = 48_000.0;
        let frequency_hz = 468.75;
        let input = (0..48_000)
            .map(|index| (std::f32::consts::TAU * frequency_hz * index as f32 / sample_rate).sin())
            .collect::<Vec<_>>();

        for ratio in [0.75, 1.5, 2.0] {
            let target_len = (input.len() as f64 * ratio).round() as usize;
            let draft = phase_vocoder(&input, target_len, ratio, 2048, 512);
            let locked = phase_locked_phase_vocoder(&input, target_len, ratio, 2048, 512);
            let draft_frequency = dominant_frequency_hz(&draft, sample_rate);
            let locked_frequency = dominant_frequency_hz(&locked, sample_rate);

            assert!(
                (locked_frequency - frequency_hz).abs() <= (draft_frequency - frequency_hz).abs() + 3.0,
                "ratio {ratio}: locked frequency {locked_frequency} Hz regressed from draft {draft_frequency} Hz"
            );
        }
    }

    #[test]
    fn transient_reset_prototype_preserves_tonal_pitch_near_draft_baseline() {
        let sample_rate = 48_000.0;
        let frequency_hz = 468.75;
        let input = (0..48_000)
            .map(|index| (std::f32::consts::TAU * frequency_hz * index as f32 / sample_rate).sin())
            .collect::<Vec<_>>();

        for ratio in [0.75, 1.5, 2.0] {
            let target_len = (input.len() as f64 * ratio).round() as usize;
            let draft = phase_vocoder(&input, target_len, ratio, 2048, 512);
            let reset = transient_reset_phase_vocoder(&input, target_len, ratio, 2048, 512);
            let draft_frequency = dominant_frequency_hz(&draft, sample_rate);
            let reset_frequency = dominant_frequency_hz(&reset, sample_rate);

            assert!(
                (reset_frequency - frequency_hz).abs()
                    <= (draft_frequency - frequency_hz).abs() + 3.0,
                "ratio {ratio}: reset frequency {reset_frequency} Hz regressed from draft {draft_frequency} Hz"
            );
        }
    }

    #[test]
    fn phase_vocoder_output_is_deterministic_with_peak_tracking() {
        let input = bin_centered_sine(9, 4096);
        let first = phase_vocoder(&input, 6144, 1.5, 512, 128);
        let repeated = phase_vocoder(&input, 6144, 1.5, 512, 128);

        assert_eq!(first, repeated);
    }
}
