use rustfft::num_complex::Complex32;

use crate::wrap_phase;

use super::types::RealtimePreviewStreamState;

impl RealtimePreviewStreamState {
    pub(super) fn analyze(&mut self, channel: usize) {
        let channel_count = self.config.channel_count;
        let fft_offset = channel * self.config.window_size;
        self.current_energy[channel] = 0.0;
        for index in 0..self.config.window_size {
            let source_index =
                (self.next_analysis_frame as usize + index) % self.source_ring_frames;
            let windowed =
                self.source_ring[source_index * channel_count + channel] * self.window[index];
            self.current_energy[channel] += (windowed * windowed) as f64;
            self.analysis_buffer[fft_offset + index] = Complex32::new(windowed, 0.0);
        }
        self.current_energy[channel] /= self.config.window_size as f64;
        self.forward.process_with_scratch(
            &mut self.analysis_buffer[fft_offset..fft_offset + self.config.window_size],
            &mut self.forward_fft_scratch,
        );
    }

    pub(super) fn propagate_phase(&mut self, channel: usize, ratio: f64) {
        let bins = self.bins;
        let fft_offset = channel * self.config.window_size;
        let bin_offset = channel * bins;
        let is_first_frame = self.spectral_frame_index == 0;
        let reset_at_transient = self.should_reset_at_transient(channel, ratio);
        self.current_peak_bins.clear();

        for bin in 0..bins {
            let spectrum = self.analysis_buffer[fft_offset + bin];
            self.current_magnitudes[bin_offset + bin] = spectrum.norm();
            self.current_phases[bin_offset + bin] = spectrum.arg();
        }
        for bin in 1..bins.saturating_sub(1) {
            let magnitude = self.current_magnitudes[bin_offset + bin];
            if magnitude > 1.0e-6
                && magnitude > self.current_magnitudes[bin_offset + bin - 1]
                && magnitude >= self.current_magnitudes[bin_offset + bin + 1]
            {
                self.current_peak_bins.push(bin);
            }
        }
        for bin in 0..bins {
            let index = bin_offset + bin;
            let phase = self.current_phases[index];
            if is_first_frame || reset_at_transient {
                self.synthesis_phase[index] = phase;
            } else {
                let deviation = wrap_phase(phase - self.previous_phase[index] - self.omega[bin]);
                let advance = (self.omega[bin] + deviation) * (ratio as f32);
                self.synthesis_phase[index] = wrap_phase(self.synthesis_phase[index] + advance);
            }
            self.previous_phase[index] = phase;
        }
        self.lock_phase_to_peaks(channel);
        for bin in 0..bins {
            let index = bin_offset + bin;
            self.synthesis_spectrum[fft_offset + bin] =
                Complex32::from_polar(self.current_magnitudes[index], self.synthesis_phase[index]);
            self.previous_magnitudes[index] = self.current_magnitudes[index];
        }
        self.previous_energy[channel] = self.current_energy[channel];
        for bin in 1..self.config.window_size.div_ceil(2) {
            self.synthesis_spectrum[fft_offset + self.config.window_size - bin] =
                self.synthesis_spectrum[fft_offset + bin].conj();
        }
    }

    pub(super) fn should_reset_at_transient(&self, channel: usize, ratio: f64) -> bool {
        if self.spectral_frame_index == 0 || ratio < 1.0 {
            return false;
        }
        let fft_offset = channel * self.config.window_size;
        let bin_offset = channel * self.bins;
        let mut flux = 0.0f32;
        let mut magnitude_sum = 0.0f32;
        for bin in 0..self.bins {
            let magnitude = self.analysis_buffer[fft_offset + bin].norm();
            flux += (magnitude - self.previous_magnitudes[bin_offset + bin]).max(0.0);
            magnitude_sum += magnitude;
        }
        let flux_ratio = flux as f64 / (magnitude_sum as f64 + 1.0e-12);
        let energy_ratio = self.current_energy[channel] / (self.previous_energy[channel] + 1.0e-12);
        flux_ratio >= 0.30 && energy_ratio >= 1.20
    }

    pub(super) fn lock_phase_to_peaks(&mut self, channel: usize) {
        if self.current_peak_bins.is_empty() {
            return;
        }
        let bin_offset = channel * self.bins;
        for peak_index in 0..self.current_peak_bins.len() {
            let peak_bin = self.current_peak_bins[peak_index];
            let peak_phase = self.synthesis_phase[bin_offset + peak_bin];
            let analysis_peak_phase = self.current_phases[bin_offset + peak_bin];
            let (left, right) = self.peak_region_bounds(peak_index);
            for bin in left..right {
                let index = bin_offset + bin;
                let relative_phase = wrap_phase(self.current_phases[index] - analysis_peak_phase);
                self.synthesis_phase[index] = wrap_phase(peak_phase + relative_phase);
            }
        }
    }

    pub(super) fn peak_region_bounds(&self, peak_index: usize) -> (usize, usize) {
        let peak = self.current_peak_bins[peak_index];
        let left = if peak_index == 0 {
            0
        } else {
            (self.current_peak_bins[peak_index - 1] + peak) / 2 + 1
        };
        let right = self
            .current_peak_bins
            .get(peak_index + 1)
            .map(|next| (peak + *next) / 2 + 1)
            .unwrap_or(self.bins);
        (left, right)
    }

    pub(super) fn synthesize(&mut self, channel: usize, synthesis_start: u64) {
        let fft_offset = channel * self.config.window_size;
        self.inverse.process_with_scratch(
            &mut self.synthesis_spectrum[fft_offset..fft_offset + self.config.window_size],
            &mut self.inverse_fft_scratch,
        );
        let channel_count = self.config.channel_count;
        let scale = 1.0 / self.config.window_size as f32;
        for index in 0..self.config.window_size {
            let output_index = (synthesis_start as usize + index) % self.output_ring_frames;
            let ring_index = output_index * channel_count + channel;
            let sample =
                self.synthesis_spectrum[fft_offset + index].re * scale * self.window[index];
            self.output_ring[ring_index] += sample;
            self.normalization_ring[ring_index] += self.window[index] * self.window[index];
        }
    }
}
