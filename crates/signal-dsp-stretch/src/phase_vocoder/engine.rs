use std::sync::Arc;

use rustfft::{num_complex::Complex32, Fft, FftPlanner};
use signal_primitives::Sample;

use super::config::{PhasePropagationMode, PhaseVocoderConfig, SpectralPeak};
use super::wrap_phase::wrap_phase;

pub(crate) struct DraftPhaseVocoder {
    pub(crate) config: PhaseVocoderConfig,
    pub(crate) mode: PhasePropagationMode,
    pub(crate) window: Vec<f32>,
    pub(crate) analysis: PhaseVocoderAnalysisState,
    pub(crate) propagation: PhaseVocoderPropagationState,
    pub(crate) synthesis: PhaseVocoderSynthesisState,
}

pub(crate) struct PhaseVocoderAnalysisState {
    pub(crate) forward: Arc<dyn Fft<f32>>,
    /// Caller-owned FFT scratch. `Fft::process` allocates its own scratch on
    /// every call, which is two heap allocations per STFT frame in the hot
    /// loop; the RealtimePreview kernel already avoided that.
    pub(crate) scratch: Vec<Complex32>,
    pub(crate) current_magnitudes: Vec<f32>,
    pub(crate) current_phases: Vec<f32>,
    pub(crate) current_peaks: Vec<SpectralPeak>,
    pub(crate) previous_magnitudes: Vec<f32>,
    pub(crate) current_energy: f64,
    pub(crate) previous_energy: f64,
    pub(crate) transient_reset_current_frame: bool,
    pub(crate) buffer: Vec<Complex32>,
}

pub(crate) struct PhaseVocoderPropagationState {
    pub(crate) omega: Vec<f32>,
    pub(crate) previous_phase: Vec<f32>,
    pub(crate) synthesis_phase: Vec<f32>,
}

pub(crate) struct PhaseVocoderSynthesisState {
    pub(crate) inverse: Arc<dyn Fft<f32>>,
    pub(crate) scratch: Vec<Complex32>,
    pub(crate) spectrum: Vec<Complex32>,
    pub(crate) output: Vec<f32>,
    pub(crate) normalization: Vec<f32>,
}

impl DraftPhaseVocoder {
    pub(crate) fn new(config: PhaseVocoderConfig, mode: PhasePropagationMode) -> Self {
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

    pub(crate) fn process(&mut self, input: &[Sample]) {
        for frame_index in 0..self.config.frame_count {
            self.analyze_frame(input, frame_index);
            self.track_spectral_peaks(frame_index);
            self.propagate_phase(frame_index);
            self.synthesize_frame(frame_index);
        }
    }

    pub(crate) fn analyze_frame(&mut self, input: &[Sample], frame_index: usize) {
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

    pub(crate) fn track_spectral_peaks(&mut self, frame_index: usize) {
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

    pub(crate) fn lock_phase_to_peaks(&mut self) {
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

    pub(crate) fn finish(mut self) -> Vec<Sample> {
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
