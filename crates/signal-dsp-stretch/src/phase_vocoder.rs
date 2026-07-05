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
    let config = PhaseVocoderConfig::new(input, target_len, ratio, window_size, analysis_hop);
    let mut engine = DraftPhaseVocoder::new(config);
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
    window: Vec<f32>,
    omega: Vec<f32>,
    forward: Arc<dyn Fft<f32>>,
    inverse: Arc<dyn Fft<f32>>,
    previous_phase: Vec<f32>,
    synthesis_phase: Vec<f32>,
    current_magnitudes: Vec<f32>,
    current_peaks: Vec<SpectralPeak>,
    analysis_buffer: Vec<Complex32>,
    synthesis_spectrum: Vec<Complex32>,
    output: Vec<f32>,
    normalization: Vec<f32>,
}

impl DraftPhaseVocoder {
    fn new(config: PhaseVocoderConfig) -> Self {
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
            current_peaks: Vec::with_capacity(config.bins / 4),
            analysis_buffer: vec![Complex32::new(0.0, 0.0); config.window_size],
            synthesis_spectrum: vec![Complex32::new(0.0, 0.0); config.window_size],
            output: vec![0.0; output_len],
            normalization: vec![0.0; output_len],
            config,
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
            let magnitude = self.current_magnitudes[bin];
            let phase = self.analysis_buffer[bin].arg();
            if frame_index == 0 {
                self.synthesis_phase[bin] = phase;
            } else {
                let deviation = wrap_phase(phase - self.previous_phase[bin] - self.omega[bin]);
                let advance = (self.omega[bin] + deviation)
                    * (self.config.synthesis_hop / self.config.analysis_hop as f64) as f32;
                self.synthesis_phase[bin] = wrap_phase(self.synthesis_phase[bin] + advance);
            }
            self.previous_phase[bin] = phase;
            self.synthesis_spectrum[bin] =
                Complex32::from_polar(magnitude, self.synthesis_phase[bin]);
        }
        for bin in 1..self.config.window_size.div_ceil(2) {
            self.synthesis_spectrum[self.config.window_size - bin] =
                self.synthesis_spectrum[bin].conj();
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

    #[test]
    fn tracks_local_spectral_peaks_for_current_frame() {
        let window_size = 256;
        let target_bin = 17;
        let input = bin_centered_sine(target_bin, window_size);
        let config =
            PhaseVocoderConfig::new(&input, window_size, 1.0, window_size, window_size / 4);
        let mut engine = DraftPhaseVocoder::new(config);

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
    fn phase_vocoder_output_is_deterministic_with_peak_tracking() {
        let input = bin_centered_sine(9, 4096);
        let first = phase_vocoder(&input, 6144, 1.5, 512, 128);
        let repeated = phase_vocoder(&input, 6144, 1.5, 512, 128);

        assert_eq!(first, repeated);
    }
}
