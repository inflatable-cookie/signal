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
use signal_dsp_resample::StreamingResampler;
use signal_primitives::{Sample, SampleRate};

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
    /// Session sample rate. Only consulted when `pitch_shift_semitones` is
    /// non-zero.
    pub sample_rate: SampleRate,
    /// Pitch shift in semitones. Zero renders the unpitched path unchanged.
    pub pitch_shift_semitones: f64,
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

/// `2^(semitones/12)`. Resampling divides the frame count by this, so a source
/// position divides by it and a ratio multiplies by it.
fn pitch_shift_factor(semitones: f64) -> f64 {
    if !semitones.is_finite() || semitones.abs() < 1.0e-9 {
        return 1.0;
    }
    let factor = 2.0f64.powf(semitones / 12.0);
    if factor.is_finite() && factor > 0.0 {
        factor
    } else {
        1.0
    }
}

fn build_pitch_stage(config: &ResumableStretchConfig, factor: f64) -> Option<PitchStage> {
    if (factor - 1.0).abs() < 1.0e-12 || config.sample_rate.0 == 0 {
        return None;
    }
    // Same virtual-rate construction the whole-buffer path uses, so the pitched
    // material is identical: resample from `rate * factor` down to `rate`.
    let virtual_rate =
        ((config.sample_rate.0 as f64 * factor).round()).clamp(1.0, u32::MAX as f64) as u32;
    let resample_config = signal_dsp_resample::ResampleConfig::new(
        SampleRate(virtual_rate),
        config.sample_rate,
        signal_dsp_resample::ResampleQuality::BandLimited,
    );
    Some(PitchStage {
        mid: StreamingResampler::new(resample_config),
        side: (config.channels == 2).then(|| StreamingResampler::new(resample_config)),
        mid_scratch: Vec::new(),
        side_scratch: Vec::new(),
        pitched: Vec::new(),
        carry: Vec::new(),
    })
}

/// Resample stage that carries its state across chunk boundaries.
///
/// `signal-dsp-resample` already provides the carry: `StreamingResampler` holds
/// a pending history buffer and a fractional source cursor, which is exactly
/// what a chunk boundary destroys. `resample_mono`, which the whole-buffer pitch
/// path calls, is a thin wrapper over it — so this writes no resampling.
struct PitchStage {
    /// Mid for stereo, or the single channel for mono.
    mid: StreamingResampler,
    /// Side for stereo only.
    side: Option<StreamingResampler>,
    mid_scratch: Vec<Sample>,
    side_scratch: Vec<Sample>,
    pitched: Vec<Sample>,
    /// Pitched frames produced but not yet accepted by the stretch stage.
    ///
    /// The ring-feed loop can exit with frames outstanding when a drain cannot
    /// progress. Without this they would be dropped, because the pitched buffer
    /// is rebuilt from fresh source on the next call — a source drop whose size
    /// depends on the caller's chunking, which is exactly what chunk-count
    /// independence forbids.
    carry: Vec<Sample>,
}

impl PitchStage {
    /// Resample one interleaved chunk into `self.pitched`.
    ///
    /// `finish` drains the resamplers' tails instead of feeding them more.
    fn process(&mut self, source: &[Sample], channels: usize, finish: bool) -> &[Sample] {
        self.pitched.clear();
        self.pitched.extend_from_slice(&self.carry);
        self.carry.clear();
        if channels == 2 {
            let frames = source.len() / 2;
            self.mid_scratch.clear();
            self.side_scratch.clear();
            for frame in source[..frames * 2].chunks_exact(2) {
                self.mid_scratch.push((frame[0] + frame[1]) * 0.5);
                self.side_scratch.push((frame[0] - frame[1]) * 0.5);
            }
            let mid = if finish {
                self.mid.finish()
            } else {
                self.mid.process_chunk(&self.mid_scratch)
            };
            let side = match self.side.as_mut() {
                Some(side) if finish => side.finish(),
                Some(side) => side.process_chunk(&self.side_scratch),
                None => Vec::new(),
            };
            let count = mid.len().min(side.len());
            for index in 0..count {
                self.pitched.push(mid[index] + side[index]);
                self.pitched.push(mid[index] - side[index]);
            }
        } else {
            let produced = if finish {
                self.mid.finish()
            } else {
                self.mid.process_chunk(source)
            };
            self.pitched.extend_from_slice(&produced);
        }
        &self.pitched
    }
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
    /// Resample stage, upstream of the stretch stage. `g10.042` Batch 42.2
    /// froze the order: resample then stretch, mid/side rather than left/right,
    /// matching the whole-buffer pitch path.
    pitch: Option<PitchStage>,
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

        // The stretch stage runs *after* the resampler, so everything it is
        // configured with must be in pitched-frame coordinates. `target_frames`
        // in the whole-buffer path is computed from the original count before
        // resampling, which is why the effective ratio is not the nominal one.
        //
        // Resampling changes the frame count by `1 / factor`, so a source
        // position divides by `factor` and a ratio multiplies by it. Getting
        // this wrong yields a render of exactly the right length with its ratio
        // automation in the wrong places, which no length or chunk-independence
        // check can see.
        let pitch_factor = pitch_shift_factor(config.pitch_shift_semitones);
        let pitch = build_pitch_stage(&config, pitch_factor);
        let (plan_source_frames, plan_curve, plan_fallback) = if pitch.is_some() {
            (
                ((config.source_frames as f64) / pitch_factor).round() as usize,
                config
                    .ratio_curve
                    .iter()
                    .map(|point| StretchRatioPoint {
                        timeline_frame: ((point.timeline_frame as f64) / pitch_factor).round()
                            as i64,
                        ratio: point.ratio * pitch_factor,
                    })
                    .collect::<Vec<_>>(),
                config.fallback_ratio * pitch_factor,
            )
        } else {
            (
                config.source_frames,
                config.ratio_curve.clone(),
                config.fallback_ratio,
            )
        };

        let target_output_frames =
            crate::dynamic_ratio_output_frames(plan_source_frames, &plan_curve, plan_fallback);

        let mut config = config;
        config.ratio_curve = plan_curve;
        config.fallback_ratio = plan_fallback;
        let mut renderer = Self {
            pitch,
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
        let caller_frames = source.len() / channels;
        let before = self.delivered_output_frames;

        // Resample first, then stretch. The stretch stage below never sees the
        // caller's source under pitch — it sees the pitched material, which is
        // why its plan is in pitched coordinates.
        let pitched_owned;
        let pitched = self.pitch.is_some();
        let source = if let Some(pitch) = self.pitch.as_mut() {
            pitched_owned = pitch.process(source, channels, false).to_vec();
            &pitched_owned[..]
        } else {
            source
        };
        let frames = source.len() / channels;
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
        // Anything the stretch stage did not take is carried, not dropped.
        if pitched && consumed < frames {
            let leftover = source[consumed * channels..].to_vec();
            if let Some(pitch) = self.pitch.as_mut() {
                pitch.carry = leftover;
            }
        }

        // Report the caller's frames, not the pitched ones: the caller sizes
        // its chunks in source coordinates and a mismatch here would make the
        // reported and actual source advance disagree.
        let reported = if pitched { caller_frames } else { consumed };
        self.accepted_source_frames += reported;
        Ok(self.report(reported, self.delivered_output_frames - before))
    }

    /// Drain the tail once all source has been delivered.
    pub fn flush(
        &mut self,
        output: &mut Vec<Sample>,
    ) -> Result<ResumableRenderReport, StretchRenderError> {
        let before = self.delivered_output_frames;

        // Finish the resamplers before the stretcher. The other order discards
        // the resampler tail, which is a source drop.
        if !self.flushed {
            if let Some(pitch) = self.pitch.as_mut() {
                let channels = self.config.channels;
                let tail = pitch.process(&[], channels, true).to_vec();
                if !tail.is_empty() {
                    let mut consumed = 0;
                    let frames = tail.len() / channels;
                    while consumed < frames {
                        let pending = self.input_write_frame - self.next_analysis_frame;
                        let capacity = self.ring_frames.saturating_sub(pending);
                        if capacity == 0 {
                            if self.drain(output, false) == 0 {
                                break;
                            }
                            continue;
                        }
                        let take = capacity.min(frames - consumed);
                        self.push_input(
                            &tail[consumed * channels..(consumed + take) * channels],
                            take,
                        );
                        consumed += take;
                        self.drain(output, false);
                    }
                }
            }
        }

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
