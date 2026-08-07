use rustfft::num_complex::Complex32;

use crate::wrap_phase;

use super::engine::ResumableOfflineStretch;

impl ResumableOfflineStretch {
    pub(in crate::resumable) fn analyze(&mut self, channel: usize) {
        let channel_count = self.config.channels;
        let mut energy = 0.0f64;
        for index in 0..self.window_size {
            let source_frame = (self.next_analysis_frame + index) % self.ring_frames;
            let windowed =
                self.input_ring[source_frame * channel_count + channel] * self.window[index];
            energy += (windowed * windowed) as f64;
            self.channels[channel].analysis[index] = Complex32::new(windowed, 0.0);
        }
        energy /= self.window_size as f64;
        self.forward.process_with_scratch(
            &mut self.channels[channel].analysis,
            &mut self.forward_scratch,
        );
        let state = &mut self.channels[channel];
        for bin in 0..self.bins {
            state.current_magnitudes[bin] = state.analysis[bin].norm();
            state.current_phases[bin] = state.analysis[bin].arg();
        }
        state.current_energy_scratch = energy;
    }

    pub(in crate::resumable) fn propagate(&mut self, channel: usize, ratio: f64) {
        let bins = self.bins;
        let reset = self.should_reset(channel, ratio);
        let first = self.frame_index == 0;
        let state = &mut self.channels[channel];

        state.peaks.clear();
        for bin in 1..bins.saturating_sub(1) {
            let magnitude = state.current_magnitudes[bin];
            if magnitude > 1.0e-6
                && magnitude > state.current_magnitudes[bin - 1]
                && magnitude >= state.current_magnitudes[bin + 1]
            {
                state.peaks.push(bin);
            }
        }

        for bin in 0..bins {
            let phase = state.current_phases[bin];
            if first || reset {
                state.synthesis_phase[bin] = phase;
            } else {
                let deviation = wrap_phase(phase - state.previous_phase[bin] - self.omega[bin]);
                let advance = (self.omega[bin] + deviation) * (ratio as f32);
                state.synthesis_phase[bin] = wrap_phase(state.synthesis_phase[bin] + advance);
            }
            state.previous_phase[bin] = phase;
        }

        for index in 0..state.peaks.len() {
            let peak = state.peaks[index];
            let peak_phase = state.synthesis_phase[peak];
            let analysis_peak_phase = state.current_phases[peak];
            let left = if index == 0 {
                0
            } else {
                (state.peaks[index - 1] + peak) / 2 + 1
            };
            let right = state
                .peaks
                .get(index + 1)
                .map(|next| (peak + *next) / 2 + 1)
                .unwrap_or(bins);
            for bin in left..right {
                let relative = wrap_phase(state.current_phases[bin] - analysis_peak_phase);
                state.synthesis_phase[bin] = wrap_phase(peak_phase + relative);
            }
        }

        for bin in 0..bins {
            state.spectrum[bin] =
                Complex32::from_polar(state.current_magnitudes[bin], state.synthesis_phase[bin]);
            state.previous_magnitudes[bin] = state.current_magnitudes[bin];
        }
        state.previous_energy = state.current_energy_scratch;
        for bin in 1..self.window_size.div_ceil(2) {
            state.spectrum[self.window_size - bin] = state.spectrum[bin].conj();
        }
    }

    fn should_reset(&self, channel: usize, ratio: f64) -> bool {
        if self.frame_index == 0 || ratio < 1.0 {
            return false;
        }
        let state = &self.channels[channel];
        let mut flux = 0.0f32;
        let mut magnitude_sum = 0.0f32;
        for bin in 0..self.bins {
            let magnitude = state.current_magnitudes[bin];
            flux += (magnitude - state.previous_magnitudes[bin]).max(0.0);
            magnitude_sum += magnitude;
        }
        let flux_ratio = flux as f64 / (magnitude_sum as f64 + 1.0e-12);
        let energy_ratio = state.current_energy_scratch / (state.previous_energy + 1.0e-12);
        flux_ratio >= 0.30 && energy_ratio >= 1.20
    }

    pub(in crate::resumable) fn synthesize(&mut self, channel: usize, synthesis_start: usize) {
        self.inverse.process_with_scratch(
            &mut self.channels[channel].spectrum,
            &mut self.inverse_scratch,
        );
        let scale = 1.0 / self.window_size as f32;
        let ring_frames = self.output_ring_frames;
        let state = &mut self.channels[channel];
        for index in 0..self.window_size {
            let ring_frame = (synthesis_start + index) % ring_frames;
            let weight = self.window[index];
            state.output_ring[ring_frame] += state.spectrum[index].re * scale * weight;
            state.normalization_ring[ring_frame] += weight * weight;
        }
    }
}
