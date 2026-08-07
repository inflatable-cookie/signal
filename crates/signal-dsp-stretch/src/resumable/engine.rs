use std::sync::Arc;

use rustfft::{num_complex::Complex32, Fft, FftPlanner};
use signal_primitives::Sample;

use crate::{StretchRatioPoint, StretchRenderError};

use super::pitch::{build_pitch_stage, pitch_shift_factor, PitchStage};
use super::types::{
    ChannelState, ResumableRenderReport, ResumableStretchConfig, MAX_RESUMABLE_WINDOW_SIZE,
};

/// Offline stretch renderer that survives chunk boundaries.
pub struct ResumableOfflineStretch {
    pub(in crate::resumable) config: ResumableStretchConfig,
    pub(in crate::resumable) window_size: usize,
    pub(in crate::resumable) analysis_hop: usize,
    pub(in crate::resumable) bins: usize,
    pub(in crate::resumable) ring_frames: usize,
    pub(in crate::resumable) output_ring_frames: usize,
    pub(in crate::resumable) window: Vec<f32>,
    pub(in crate::resumable) omega: Vec<f32>,
    pub(in crate::resumable) forward: Arc<dyn Fft<f32>>,
    pub(in crate::resumable) inverse: Arc<dyn Fft<f32>>,
    pub(in crate::resumable) forward_scratch: Vec<Complex32>,
    pub(in crate::resumable) inverse_scratch: Vec<Complex32>,
    pub(in crate::resumable) channels: Vec<ChannelState>,
    pub(in crate::resumable) input_ring: Vec<f32>,
    /// Padded-source frames written so far.
    pub(in crate::resumable) input_write_frame: usize,
    /// Next analysis frame start, in padded-source coordinates.
    pub(in crate::resumable) next_analysis_frame: usize,
    /// Fractional synthesis cursor, in padded-output coordinates.
    pub(in crate::resumable) next_synthesis_frame: f64,
    /// Padded-output frames already emitted.
    pub(in crate::resumable) output_read_frame: usize,
    /// Source frames accepted from the caller.
    pub(in crate::resumable) accepted_source_frames: usize,
    /// Resample stage, upstream of the stretch stage. `g10.042` Batch 42.2
    /// froze the order: resample then stretch, mid/side rather than left/right,
    /// matching the whole-buffer pitch path.
    pub(in crate::resumable) pitch: Option<PitchStage>,
    /// Output frames delivered to the caller after cropping.
    pub(in crate::resumable) delivered_output_frames: usize,
    /// Frames of leading pad still to be discarded from the output.
    pub(in crate::resumable) pending_crop_frames: usize,
    pub(in crate::resumable) target_output_frames: usize,
    pub(in crate::resumable) frame_index: usize,
    pub(in crate::resumable) flushed: bool,
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
}
