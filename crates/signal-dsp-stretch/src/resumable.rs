//! Resumable offline stretch renderer.
//!
//! Frozen by `g10.039` Batch 39.2. The renderer carries phase, detector, and
//! overlap-add state across calls, so a source rendered in any number of chunks
//! produces bit-identical output to the same source rendered in one call.
//!
//! Frame scheduling is the reason that holds: analysis frames sit on a fixed
//! grid measured from the source origin, never from a chunk boundary. A chunk
//! edge changes only *when* a frame is computed, never *which* frames exist or
//! what they see.

use std::sync::Arc;

use rustfft::{num_complex::Complex32, Fft, FftPlanner};
use signal_primitives::Sample;

use crate::{sanitize_ratio, wrap_phase, StretchRatioPoint, StretchRenderError};

/// Largest window the resumable renderer supports.
///
/// `with_window` clamps only to a power of two at or above `64`, with no upper
/// limit, so the memory bound needs its own maximum to be a number.
pub const MAX_RESUMABLE_WINDOW_SIZE: usize = 65_536;

/// Frozen working-state ceiling in bytes.
///
/// Covers `MAX_RESUMABLE_WINDOW_SIZE` in stereo, which measures `10616892` B.
///
/// This figure moved twice. The Batch 39.2 brief put it at `8 MiB` from an
/// inventory that omitted the input ring. The corrected `9 MiB` assumed output
/// rings of twice the window, which deadlocks: the write frontier meets the
/// emission limit exactly and the render stalls. Output rings are four times
/// the window, so the real cost is `12 MiB`.
pub const MAX_RESUMABLE_WORKING_BYTES: usize = 12 * 1024 * 1024;

/// Configuration for one resumable render.
#[derive(Clone, Debug, PartialEq)]
pub struct ResumableStretchConfig {
    /// Source and output channel count.
    pub channels: usize,
    /// STFT window size in frames.
    pub window_size: usize,
    /// Analysis hop in frames, before the overlap coverage law adapts it.
    pub analysis_hop: usize,
    /// Total source frames the render will consume.
    pub source_frames: usize,
    /// Ratio curve in source-frame coordinates. Empty uses `fallback_ratio`.
    pub ratio_curve: Vec<StretchRatioPoint>,
    /// Ratio for spans the curve does not cover.
    pub fallback_ratio: f64,
}

/// Frames consumed and produced by one call.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ResumableRenderReport {
    /// Source frames accepted by this call.
    pub source_frames: usize,
    /// Output frames written by this call.
    pub output_frames: usize,
    /// Cumulative source frames accepted.
    pub total_source_frames: usize,
    /// Cumulative output frames written.
    pub total_output_frames: usize,
}

struct ChannelState {
    previous_phase: Vec<f32>,
    synthesis_phase: Vec<f32>,
    previous_magnitudes: Vec<f32>,
    previous_energy: f64,
    current_energy_scratch: f64,
    analysis: Vec<Complex32>,
    spectrum: Vec<Complex32>,
    current_magnitudes: Vec<f32>,
    current_phases: Vec<f32>,
    peaks: Vec<usize>,
    output_ring: Vec<f32>,
    normalization_ring: Vec<f32>,
}

/// Offline stretch renderer that survives chunk boundaries.
pub struct ResumableOfflineStretch {
    config: ResumableStretchConfig,
    window_size: usize,
    analysis_hop: usize,
    bins: usize,
    ring_frames: usize,
    output_ring_frames: usize,
    window: Vec<f32>,
    omega: Vec<f32>,
    forward: Arc<dyn Fft<f32>>,
    inverse: Arc<dyn Fft<f32>>,
    forward_scratch: Vec<Complex32>,
    inverse_scratch: Vec<Complex32>,
    channels: Vec<ChannelState>,
    input_ring: Vec<f32>,
    /// Padded-source frames written so far.
    input_write_frame: usize,
    /// Next analysis frame start, in padded-source coordinates.
    next_analysis_frame: usize,
    /// Fractional synthesis cursor, in padded-output coordinates.
    next_synthesis_frame: f64,
    /// Padded-output frames already emitted.
    output_read_frame: usize,
    /// Source frames accepted from the caller.
    accepted_source_frames: usize,
    /// Output frames delivered to the caller after cropping.
    delivered_output_frames: usize,
    /// Frames of leading pad still to be discarded from the output.
    pending_crop_frames: usize,
    target_output_frames: usize,
    frame_index: usize,
    flushed: bool,
}

impl ResumableOfflineStretch {
    /// Construct a renderer and allocate all carried state.
    pub fn new(config: ResumableStretchConfig) -> Result<Self, StretchRenderError> {
        let window_size = config.window_size.next_power_of_two().max(64);
        if window_size > MAX_RESUMABLE_WINDOW_SIZE || !matches!(config.channels, 1 | 2) {
            return Err(StretchRenderError::UnsupportedResumableConfiguration);
        }
        let analysis_hop = config.analysis_hop.clamp(1, window_size / 2);
        let bins = window_size / 2 + 1;
        // Input holds at most one window plus a hop of unanalysed source.
        let ring_frames = window_size * 2;
        // Output must hold [output_read, synthesis_start + window) with room to
        // spare. At 2 * window the write frontier meets the emission limit
        // exactly and neither side can advance: the render then stalls.
        let output_ring_frames = window_size * 4;

        let mut planner = FftPlanner::<f32>::new();
        let forward = planner.plan_fft_forward(window_size);
        let inverse = planner.plan_fft_inverse(window_size);
        let forward_scratch = vec![Complex32::new(0.0, 0.0); forward.get_inplace_scratch_len()];
        let inverse_scratch = vec![Complex32::new(0.0, 0.0); inverse.get_inplace_scratch_len()];

        let channels = (0..config.channels)
            .map(|_| ChannelState {
                previous_phase: vec![0.0; bins],
                synthesis_phase: vec![0.0; bins],
                previous_magnitudes: vec![0.0; bins],
                previous_energy: 0.0,
                current_energy_scratch: 0.0,
                analysis: vec![Complex32::new(0.0, 0.0); window_size],
                spectrum: vec![Complex32::new(0.0, 0.0); window_size],
                current_magnitudes: vec![0.0; bins],
                current_phases: vec![0.0; bins],
                peaks: Vec::with_capacity(bins),
                output_ring: vec![0.0; output_ring_frames],
                normalization_ring: vec![0.0; output_ring_frames],
            })
            .collect();

        let target_output_frames = crate::dynamic_ratio_output_frames(
            config.source_frames,
            &config.ratio_curve,
            config.fallback_ratio,
        );

        let mut renderer = Self {
            window_size,
            analysis_hop,
            bins,
            ring_frames,
            output_ring_frames,
            window: (0..window_size)
                .map(|index| {
                    0.5 - 0.5 * (std::f32::consts::TAU * index as f32 / window_size as f32).cos()
                })
                .collect(),
            omega: (0..bins)
                .map(|bin| {
                    std::f32::consts::TAU * bin as f32 * analysis_hop as f32 / window_size as f32
                })
                .collect(),
            forward,
            inverse,
            forward_scratch,
            inverse_scratch,
            channels,
            input_ring: vec![0.0; ring_frames * config.channels],
            input_write_frame: 0,
            next_analysis_frame: 0,
            next_synthesis_frame: 0.0,
            output_read_frame: 0,
            accepted_source_frames: 0,
            delivered_output_frames: 0,
            pending_crop_frames: window_size / 2,
            target_output_frames,
            frame_index: 0,
            config,
            flushed: false,
        };
        // Leading pad so the first source sample gets a complete window.
        renderer.push_silence(renderer.window_size / 2);
        Ok(renderer)
    }

    /// Working-state size in bytes, from geometry alone.
    pub fn working_bytes(&self) -> usize {
        let f32_size = std::mem::size_of::<f32>();
        let complex = std::mem::size_of::<Complex32>();
        let shared = self.window.len() * f32_size
            + self.omega.len() * f32_size
            + (self.forward_scratch.len() + self.inverse_scratch.len()) * complex
            + self.input_ring.len() * f32_size;
        let per_channel: usize = self
            .channels
            .iter()
            .map(|state| {
                (state.previous_phase.len()
                    + state.synthesis_phase.len()
                    + state.previous_magnitudes.len()
                    + state.current_magnitudes.len()
                    + state.current_phases.len()
                    + state.output_ring.len()
                    + state.normalization_ring.len())
                    * f32_size
                    + (state.analysis.len() + state.spectrum.len()) * complex
                    + state.peaks.capacity() * std::mem::size_of::<usize>()
            })
            .sum();
        shared + per_channel
    }

    /// Total output frames this render will deliver.
    pub fn target_output_frames(&self) -> usize {
        self.target_output_frames
    }

    /// Render one source chunk. `output` receives whatever is ready.
    pub fn render(
        &mut self,
        source: &[Sample],
        output: &mut Vec<Sample>,
    ) -> Result<ResumableRenderReport, StretchRenderError> {
        let channels = self.config.channels;
        let frames = source.len() / channels;
        let before = self.delivered_output_frames;
        let mut consumed = 0;
        // The input ring holds a bounded amount of unanalysed source, so a
        // caller chunk larger than the ring must be fed in slices with a drain
        // between them. Pushing the whole chunk would overwrite source the
        // analysis cursor has not reached yet, and the output would then depend
        // on the caller's chunk size.
        while consumed < frames {
            let pending = self.input_write_frame - self.next_analysis_frame;
            let capacity = self.ring_frames.saturating_sub(pending);
            if capacity == 0 {
                let progressed = self.drain(output, false);
                if progressed == 0
                    && self.input_write_frame == self.next_analysis_frame + self.ring_frames
                {
                    break;
                }
                continue;
            }
            let take = capacity.min(frames - consumed);
            self.push_input(
                &source[consumed * channels..(consumed + take) * channels],
                take,
            );
            consumed += take;
            self.drain(output, false);
        }
        self.accepted_source_frames += consumed;
        Ok(self.report(consumed, self.delivered_output_frames - before))
    }

    /// Drain the tail once all source has been delivered.
    pub fn flush(
        &mut self,
        output: &mut Vec<Sample>,
    ) -> Result<ResumableRenderReport, StretchRenderError> {
        let before = self.delivered_output_frames;
        if !self.flushed {
            let mut remaining = self.window_size + self.analysis_hop;
            while remaining > 0 {
                let pending = self.input_write_frame - self.next_analysis_frame;
                let capacity = self.ring_frames.saturating_sub(pending).max(1);
                let take = capacity.min(remaining);
                self.push_silence(take);
                remaining -= take;
                self.drain(output, false);
            }
            self.flushed = true;
        }
        self.drain(output, true);
        Ok(self.report(0, self.delivered_output_frames - before))
    }

    /// Return to construction state without reallocating.
    pub fn reset(&mut self) {
        for state in &mut self.channels {
            state.previous_phase.fill(0.0);
            state.synthesis_phase.fill(0.0);
            state.previous_magnitudes.fill(0.0);
            state.previous_energy = 0.0;
            state.current_energy_scratch = 0.0;
            state.output_ring.fill(0.0);
            state.normalization_ring.fill(0.0);
            state.peaks.clear();
        }
        self.input_ring.fill(0.0);
        self.input_write_frame = 0;
        self.next_analysis_frame = 0;
        self.next_synthesis_frame = 0.0;
        self.output_read_frame = 0;
        self.accepted_source_frames = 0;
        self.delivered_output_frames = 0;
        self.pending_crop_frames = self.window_size / 2;
        self.frame_index = 0;
        self.flushed = false;
        self.push_silence(self.window_size / 2);
    }

    fn report(&self, source_frames: usize, output_frames: usize) -> ResumableRenderReport {
        ResumableRenderReport {
            source_frames,
            output_frames,
            total_source_frames: self.accepted_source_frames,
            total_output_frames: self.delivered_output_frames,
        }
    }

    fn push_silence(&mut self, frames: usize) {
        for _ in 0..frames {
            let ring_frame = self.input_write_frame % self.ring_frames;
            for channel in 0..self.config.channels {
                self.input_ring[ring_frame * self.config.channels + channel] = 0.0;
            }
            self.input_write_frame += 1;
        }
    }

    fn push_input(&mut self, source: &[Sample], frames: usize) {
        for frame in 0..frames {
            let ring_frame = self.input_write_frame % self.ring_frames;
            for channel in 0..self.config.channels {
                self.input_ring[ring_frame * self.config.channels + channel] =
                    source[frame * self.config.channels + channel];
            }
            self.input_write_frame += 1;
        }
    }

    /// Active ratio at one padded-source position.
    fn ratio_at(&self, padded_frame: usize) -> f64 {
        let pad = self.window_size / 2;
        let source_frame = padded_frame.saturating_sub(pad);
        let mut ratio = sanitize_ratio(self.config.fallback_ratio);
        let mut best: Option<i64> = None;
        for point in &self.config.ratio_curve {
            if point.timeline_frame < 0 || !point.ratio.is_finite() || point.ratio <= 0.0 {
                continue;
            }
            if (point.timeline_frame as usize) <= source_frame
                && best.is_none_or(|b| point.timeline_frame >= b)
            {
                best = Some(point.timeline_frame);
                ratio = point.ratio;
            }
        }
        ratio
    }

    fn drain(&mut self, output: &mut Vec<Sample>, final_pass: bool) -> usize {
        let before = self.delivered_output_frames;
        loop {
            // A frame is computable once its whole window has arrived.
            if self.next_analysis_frame + self.window_size > self.input_write_frame {
                break;
            }
            let synthesis_start = self.next_synthesis_frame.round() as usize;
            // Do not overrun the ring: emit resolved output first.
            if synthesis_start + self.window_size >= self.output_read_frame + self.ring_frames {
                self.emit(output, synthesis_start, final_pass);
                if self.output_read_frame + self.ring_frames <= synthesis_start + self.window_size {
                    break;
                }
                continue;
            }
            let ratio = self.ratio_at(self.next_analysis_frame);
            for channel in 0..self.config.channels {
                self.analyze(channel);
                self.propagate(channel, ratio);
                self.synthesize(channel, synthesis_start);
            }
            self.next_analysis_frame += self.analysis_hop;
            self.next_synthesis_frame += self.analysis_hop as f64 * ratio;
            self.frame_index += 1;
        }
        let resolved = self.next_synthesis_frame.round() as usize;
        self.emit(output, resolved, final_pass);
        self.delivered_output_frames - before
    }

    /// Emit output frames that no future analysis frame can still touch.
    fn emit(&mut self, output: &mut Vec<Sample>, synthesis_start: usize, final_pass: bool) {
        // The frame about to be written covers [synthesis_start, +window), so
        // everything below synthesis_start is final and can be released.
        let safe_until = if final_pass {
            synthesis_start + self.window_size
        } else {
            synthesis_start
        };
        while self.output_read_frame < safe_until {
            if self.delivered_output_frames >= self.target_output_frames
                && self.pending_crop_frames == 0
            {
                // Target reached: keep draining the ring so it stays clean.
                self.clear_output_frame(self.output_read_frame);
                self.output_read_frame += 1;
                continue;
            }
            let ring_frame = self.output_read_frame % self.output_ring_frames;
            if self.pending_crop_frames > 0 {
                self.pending_crop_frames -= 1;
            } else {
                for channel in 0..self.config.channels {
                    let state = &self.channels[channel];
                    let weight = state.normalization_ring[ring_frame];
                    let sample = if weight > 1.0e-3 {
                        state.output_ring[ring_frame] / weight
                    } else {
                        0.0
                    };
                    output.push(sample);
                }
                self.delivered_output_frames += 1;
            }
            self.clear_output_frame(self.output_read_frame);
            self.output_read_frame += 1;
        }
        if final_pass {
            while self.delivered_output_frames < self.target_output_frames {
                for _ in 0..self.config.channels {
                    output.push(0.0);
                }
                self.delivered_output_frames += 1;
            }
        }
    }

    fn clear_output_frame(&mut self, frame: usize) {
        let ring_frame = frame % self.output_ring_frames;
        for state in &mut self.channels {
            state.output_ring[ring_frame] = 0.0;
            state.normalization_ring[ring_frame] = 0.0;
        }
    }

    fn analyze(&mut self, channel: usize) {
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

    fn propagate(&mut self, channel: usize, ratio: f64) {
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

    fn synthesize(&mut self, channel: usize, synthesis_start: usize) {
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
