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
#[allow(dead_code)]
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

fn run_phase_vocoder(
    input: &[Sample],
    target_len: usize,
    ratio: f64,
    window_size: usize,
    analysis_hop: usize,
    mode: PhasePropagationMode,
) -> Vec<Sample> {
    let config = PhaseVocoderConfig::new(input, target_len, ratio, window_size, analysis_hop);
    let mut engine = DraftPhaseVocoder::new(config, mode);
    engine.process(input);
    engine.finish()
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PhasePropagationMode {
    IndependentBins,
    IdentityLocked,
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
    omega: Vec<f32>,
    forward: Arc<dyn Fft<f32>>,
    inverse: Arc<dyn Fft<f32>>,
    previous_phase: Vec<f32>,
    synthesis_phase: Vec<f32>,
    current_magnitudes: Vec<f32>,
    current_phases: Vec<f32>,
    current_peaks: Vec<SpectralPeak>,
    analysis_buffer: Vec<Complex32>,
    synthesis_spectrum: Vec<Complex32>,
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
        let ola_len = ((config.frame_count.saturating_sub(1)) as f64 * config.synthesis_hop).ceil()
            as usize
            + config.window_size
            + 1;
        let output_len = ola_len.max(config.target_len);

        Self {
            previous_phase: vec![0.0; config.bins],
            synthesis_phase: vec![0.0; config.bins],
            current_magnitudes: vec![0.0; config.bins],
            current_phases: vec![0.0; config.bins],
            current_peaks: Vec::with_capacity(config.bins / 4),
            analysis_buffer: vec![Complex32::new(0.0, 0.0); config.window_size],
            synthesis_spectrum: vec![Complex32::new(0.0, 0.0); config.window_size],
            output: vec![0.0; output_len],
            normalization: vec![0.0; output_len],
            config,
            mode,
            window,
            omega,
            forward,
            inverse,
        }
    }

    fn process(&mut self, input: &[Sample]) {
        for frame_index in 0..self.config.frame_count {
            self.analyze_frame(input, frame_index);
            self.track_spectral_peaks();
            self.propagate_phase(frame_index);
            self.synthesize_frame(frame_index);
        }
    }

    fn analyze_frame(&mut self, input: &[Sample], frame_index: usize) {
        let analysis_start = frame_index * self.config.analysis_hop;
        for (slot, (sample, weight)) in self.analysis_buffer.iter_mut().zip(
            input[analysis_start..analysis_start + self.config.window_size]
                .iter()
                .zip(self.window.iter()),
        ) {
            *slot = Complex32::new(sample * weight, 0.0);
        }
        self.forward.process(&mut self.analysis_buffer);
    }

    fn track_spectral_peaks(&mut self) {
        self.current_peaks.clear();
        for (bin, magnitude) in self.current_magnitudes.iter_mut().enumerate() {
            *magnitude = self.analysis_buffer[bin].norm();
        }

        if self.config.bins < 3 {
            return;
        }

        for bin in 1..self.config.bins - 1 {
            let magnitude = self.current_magnitudes[bin];
            if magnitude <= 1.0e-6 {
                continue;
            }
            if magnitude > self.current_magnitudes[bin - 1]
                && magnitude >= self.current_magnitudes[bin + 1]
            {
                self.current_peaks.push(SpectralPeak { bin, magnitude });
            }
        }
    }

    fn propagate_phase(&mut self, frame_index: usize) {
        for bin in 0..self.config.bins {
            let phase = self.analysis_buffer[bin].arg();
            self.current_phases[bin] = phase;
            if frame_index == 0 {
                self.synthesis_phase[bin] = phase;
            } else {
                let deviation = wrap_phase(phase - self.previous_phase[bin] - self.omega[bin]);
                let advance = (self.omega[bin] + deviation)
                    * (self.config.synthesis_hop / self.config.analysis_hop as f64) as f32;
                self.synthesis_phase[bin] = wrap_phase(self.synthesis_phase[bin] + advance);
            }
            self.previous_phase[bin] = phase;
        }

        if self.mode == PhasePropagationMode::IdentityLocked {
            self.lock_phase_to_peaks();
        }

        for bin in 0..self.config.bins {
            let magnitude = self.current_magnitudes[bin];
            self.synthesis_spectrum[bin] =
                Complex32::from_polar(magnitude, self.synthesis_phase[bin]);
        }
        for bin in 1..self.config.window_size.div_ceil(2) {
            self.synthesis_spectrum[self.config.window_size - bin] =
                self.synthesis_spectrum[bin].conj();
        }
    }

    fn lock_phase_to_peaks(&mut self) {
        if self.current_peaks.is_empty() {
            return;
        }

        for peak_index in 0..self.current_peaks.len() {
            let peak = self.current_peaks[peak_index];
            let peak_phase = self.synthesis_phase[peak.bin];
            let analysis_peak_phase = self.current_phases[peak.bin];
            let left = if peak_index == 0 {
                0
            } else {
                (self.current_peaks[peak_index - 1].bin + peak.bin) / 2 + 1
            };
            let right = self
                .current_peaks
                .get(peak_index + 1)
                .map(|next| (peak.bin + next.bin) / 2 + 1)
                .unwrap_or(self.config.bins);

            for bin in left..right {
                let relative_phase = wrap_phase(self.current_phases[bin] - analysis_peak_phase);
                self.synthesis_phase[bin] = wrap_phase(peak_phase + relative_phase);
            }
        }
    }

    fn synthesize_frame(&mut self, frame_index: usize) {
        self.inverse.process(&mut self.synthesis_spectrum);
        let synthesis_start = (frame_index as f64 * self.config.synthesis_hop).round() as usize;
        let scale = 1.0 / self.config.window_size as f32;
        for (index, weight) in self.window.iter().enumerate() {
            let out_index = synthesis_start + index;
            if out_index >= self.output.len() {
                break;
            }
            self.output[out_index] += self.synthesis_spectrum[index].re * scale * weight;
            self.normalization[out_index] += weight * weight;
        }
    }

    fn finish(mut self) -> Vec<Sample> {
        for (sample, weight) in self.output.iter_mut().zip(self.normalization.iter()) {
            if *weight > 1.0e-3 {
                *sample /= *weight;
            }
        }
        self.output.resize(self.config.target_len, 0.0);
        self.output
    }
}

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

    #[test]
    fn tracks_local_spectral_peaks_for_current_frame() {
        let window_size = 256;
        let target_bin = 17;
        let input = bin_centered_sine(target_bin, window_size);
        let config =
            PhaseVocoderConfig::new(&input, window_size, 1.0, window_size, window_size / 4);
        let mut engine = DraftPhaseVocoder::new(config, PhasePropagationMode::IndependentBins);

        engine.analyze_frame(&input, 0);
        engine.track_spectral_peaks();

        assert!(
            engine
                .current_peaks
                .iter()
                .any(|peak| peak.bin.abs_diff(target_bin) <= 1),
            "expected a peak near bin {target_bin}, got {:?}",
            engine.current_peaks
        );
    }

    #[test]
    fn identity_locking_preserves_peak_neighborhood_phase_offsets() {
        let input = vec![0.0; 512];
        let config = PhaseVocoderConfig::new(&input, input.len(), 1.0, 512, 128);
        let mut engine = DraftPhaseVocoder::new(config, PhasePropagationMode::IdentityLocked);
        engine.current_peaks.push(SpectralPeak {
            bin: 10,
            magnitude: 1.0,
        });
        engine.current_phases[9] = 0.20;
        engine.current_phases[10] = 0.50;
        engine.current_phases[11] = 0.90;
        engine.synthesis_phase[10] = 1.25;

        engine.lock_phase_to_peaks();

        assert!((wrap_phase(engine.synthesis_phase[9] - 0.95)).abs() < 1.0e-6);
        assert!((wrap_phase(engine.synthesis_phase[10] - 1.25)).abs() < 1.0e-6);
        assert!((wrap_phase(engine.synthesis_phase[11] - 1.65)).abs() < 1.0e-6);
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
    fn phase_vocoder_output_is_deterministic_with_peak_tracking() {
        let input = bin_centered_sine(9, 4096);
        let first = phase_vocoder(&input, 6144, 1.5, 512, 128);
        let repeated = phase_vocoder(&input, 6144, 1.5, 512, 128);

        assert_eq!(first, repeated);
    }
}
