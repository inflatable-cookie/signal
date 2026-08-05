//! Source-owning RealtimePreview streaming kernel (`g10.040` Batch 40.3).
//!
//! Isolated candidate per Contract `084` Rule 2: nothing in the workspace
//! constructs this yet, and [`crate::realtime_preview`] is untouched.
//!
//! The difference from the shipped callback state is the whole point of the
//! lane. That one is quantum-locked — it takes `n` input frames and returns `n`
//! output frames whatever the ratio, so the analysis and synthesis cursors
//! diverge until a ring guard silently discards unanalysed source and returns
//! `Ok`. This one has no input parameter at all. The caller pushes source
//! frames ahead of time from a non-realtime thread, and [`RealtimePreviewStreamState::render`]
//! pulls however many source frames the active ratio demands.
//!
//! Frozen by `g10.040` Batch 40.2: ratio range, memory ceiling, one scheduler,
//! the underrun contract, and the latency report.

use rustfft::num_complex::Complex32;
use rustfft::{Fft, FftPlanner};
use std::sync::Arc;

use signal_primitives::Sample;

use crate::realtime_preview::{RealtimePreviewPlanError, RealtimePreviewStreamConfig};
use crate::{align_to_next_grid, wrap_phase};

/// Slowest playback the preview kernel accepts.
///
/// Work per callback scales as `1/ratio`, so a floor is what makes Contract
/// `046`'s bounded-work requirement satisfiable at all. At `0.25` — four times
/// faster than source — a stereo `128`-frame callback measured `2.36%` of its
/// budget in `g10.040` Batch 40.1.
pub const REALTIME_PREVIEW_STREAM_MIN_RATIO: f64 = 0.25;

/// Largest ratio the frozen geometry covers.
///
/// Contract `046`'s overlap law requires `analysis_hop * ratio <= 0.75 *
/// window_size`, which at the frozen `128`/`512` geometry is exactly `3.0`.
/// Higher ratios are cheap — `0.20%` of budget — so this is a spectral coverage
/// limit, not a cost one. Exceeding it needs the contract's hop reduction,
/// which changes the geometry and is out of this lane's scope.
pub const REALTIME_PREVIEW_STREAM_MAX_RATIO: f64 = 3.0;

/// Working-set ceiling, stereo at `MAX_BLOCK_FRAMES`.
///
/// Batch 40.2 computed `804.3 KiB` from a measured state plus a sized source
/// ring. Derived after the design, deliberately: `g10.039` froze a ceiling
/// before its design existed and moved it three times.
pub const REALTIME_PREVIEW_STREAM_MAX_WORKING_BYTES: usize = 1024 * 1024;

/// Why a preview render could not run.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum RealtimePreviewStreamError {
    /// The requested ratio is outside the frozen range.
    RatioOutOfRange {
        /// Ratio the caller asked for.
        requested: f64,
        /// Frozen minimum.
        min: f64,
        /// Frozen maximum.
        max: f64,
    },
    /// The block is larger than the configured maximum.
    FrameCountExceedsConfig {
        /// Frames requested.
        requested: usize,
        /// Configured maximum.
        max: usize,
    },
    /// The caller's output slice cannot hold the requested block.
    OutputTooSmall {
        /// Samples the block needs.
        required_samples: usize,
        /// Samples provided.
        output_samples: usize,
    },
}

/// What one preview render actually did.
///
/// `underrun_frames` is the field that distinguishes this kernel from the one
/// it replaces. The shipped callback state reports `input_frames ==
/// output_frames` while discarding source, so a starved block is
/// indistinguishable from a healthy one — which is how the defect survived
/// three roadmaps. A report that cannot express failure hides it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RealtimePreviewStreamRenderReport {
    /// Output frames written this block.
    pub output_frames: usize,
    /// Output frames left silent because source was not filled far enough.
    pub underrun_frames: usize,
    /// Source frames the kernel consumed this block.
    pub source_frames_consumed: u64,
    /// Cumulative source frames consumed.
    pub total_source_frames_consumed: u64,
    /// Spectral frames processed this block; the callback's work measure.
    pub spectral_frames: usize,
    /// Ratio requested by the caller.
    pub requested_ratio: f64,
    /// Ratio actually in force at the end of the block.
    pub active_ratio: f64,
    /// Ratio changes applied by this state.
    pub ratio_change_count: u64,
    /// Output-frame distance between the latest ratio request and its
    /// application. Bounded by `analysis_hop` by construction.
    pub ratio_change_alignment_error_frames: u64,
    /// Absolute source frame the producer must fill to.
    pub source_demand_frame: u64,
}

/// Source-owning preview streaming state.
///
/// One caller at a time on the audio thread. `push_source` is the non-realtime
/// producer's entry point and `render` is the callback's; they are separate so
/// that no I/O, allocation, or lock can reach the callback.
pub struct RealtimePreviewStreamState {
    config: RealtimePreviewStreamConfig,
    bins: usize,

    source_ring: Vec<Sample>,
    source_ring_frames: usize,
    source_write_frame: u64,

    output_ring: Vec<Sample>,
    normalization_ring: Vec<Sample>,
    output_ring_frames: usize,
    output_read_frame: u64,

    window: Vec<Sample>,
    omega: Vec<Sample>,
    analysis_buffer: Vec<Complex32>,
    synthesis_spectrum: Vec<Complex32>,
    forward_fft_scratch: Vec<Complex32>,
    inverse_fft_scratch: Vec<Complex32>,
    previous_phase: Vec<Sample>,
    synthesis_phase: Vec<Sample>,
    current_magnitudes: Vec<Sample>,
    current_phases: Vec<Sample>,
    previous_magnitudes: Vec<Sample>,
    current_peak_bins: Vec<usize>,
    current_energy: Vec<f64>,
    previous_energy: Vec<f64>,
    forward: Arc<dyn Fft<Sample>>,
    inverse: Arc<dyn Fft<Sample>>,

    // The single ratio scheduler. Batch 40.2 deleted the output-side duplicate
    // and kept this one: it tracks the source cursor, which is what drives
    // demand. The `g10.027` projection was never wrong, nothing consumed it.
    current_ratio: f64,
    active_ratio: f64,
    pending_ratio: f64,
    pending_request_frame: u64,
    pending_apply_frame: u64,
    pending_change: bool,
    ratio_change_count: u64,
    last_alignment_error_frames: u64,

    next_analysis_frame: u64,
    next_synthesis_frame: f64,
    spectral_frame_index: u64,
    total_source_frames_consumed: u64,
}

impl RealtimePreviewStreamState {
    /// Plan and allocate. Every buffer the callback touches is allocated here.
    pub fn new(config: RealtimePreviewStreamConfig) -> Result<Self, RealtimePreviewPlanError> {
        let contract = crate::realtime_preview::plan_realtime_preview_stream(config)?;
        let config = contract.config;
        let channel_count = config.channel_count;
        let window_size = config.window_size;
        let bins = window_size / 2 + 1;

        let source_ring_frames = Self::prefill_frames_for(&config);
        let output_ring_frames = config.max_block_frames + window_size * 4;

        let mut planner = FftPlanner::<Sample>::new();
        let forward = planner.plan_fft_forward(window_size);
        let inverse = planner.plan_fft_inverse(window_size);
        let forward_fft_scratch = vec![Complex32::new(0.0, 0.0); forward.get_inplace_scratch_len()];
        let inverse_fft_scratch = vec![Complex32::new(0.0, 0.0); inverse.get_inplace_scratch_len()];

        Ok(Self {
            config,
            bins,
            source_ring: vec![0.0; source_ring_frames * channel_count],
            source_ring_frames,
            source_write_frame: 0,
            output_ring: vec![0.0; output_ring_frames * channel_count],
            normalization_ring: vec![0.0; output_ring_frames * channel_count],
            output_ring_frames,
            output_read_frame: 0,
            window: (0..window_size)
                .map(|index| {
                    let phase =
                        std::f32::consts::TAU * index as f32 / (window_size as f32 - 1.0).max(1.0);
                    0.5 - 0.5 * phase.cos()
                })
                .collect(),
            omega: (0..bins)
                .map(|bin| {
                    std::f32::consts::TAU * config.analysis_hop as f32 * bin as f32
                        / window_size as f32
                })
                .collect(),
            analysis_buffer: vec![Complex32::new(0.0, 0.0); window_size * channel_count],
            synthesis_spectrum: vec![Complex32::new(0.0, 0.0); window_size * channel_count],
            forward_fft_scratch,
            inverse_fft_scratch,
            previous_phase: vec![0.0; bins * channel_count],
            synthesis_phase: vec![0.0; bins * channel_count],
            current_magnitudes: vec![0.0; bins * channel_count],
            current_phases: vec![0.0; bins * channel_count],
            previous_magnitudes: vec![0.0; bins * channel_count],
            current_peak_bins: Vec::with_capacity(bins),
            current_energy: vec![0.0; channel_count],
            previous_energy: vec![0.0; channel_count],
            forward,
            inverse,
            current_ratio: 1.0,
            active_ratio: 1.0,
            pending_ratio: 1.0,
            pending_request_frame: 0,
            pending_apply_frame: 0,
            pending_change: false,
            ratio_change_count: 0,
            last_alignment_error_frames: 0,
            next_analysis_frame: 0,
            next_synthesis_frame: 0.0,
            spectral_frame_index: 0,
            total_source_frames_consumed: 0,
        })
    }

    /// Source frames the producer should keep filled ahead of the read cursor.
    ///
    /// `ceil(max_block / ratio_min) * 2 + window_size`: two callbacks of
    /// headroom at the fastest supported playback, plus one analysis window.
    fn prefill_frames_for(config: &RealtimePreviewStreamConfig) -> usize {
        let per_callback =
            (config.max_block_frames as f64 / REALTIME_PREVIEW_STREAM_MIN_RATIO).ceil() as usize;
        per_callback * 2 + config.window_size
    }

    /// Validated stream configuration.
    pub fn config(&self) -> RealtimePreviewStreamConfig {
        self.config
    }

    /// Source frames the producer must fill before playback starts, and keep
    /// ahead of the read cursor afterwards.
    pub fn prefill_frames(&self) -> usize {
        self.source_ring_frames
    }

    /// Reported algorithmic latency: one analysis window plus the prefill.
    ///
    /// Constant for a configuration. This is a start-up delay before preview
    /// playback begins rather than a round-trip cost, because preview plays
    /// back a stored asset rather than monitoring a live signal.
    pub fn latency_frames(&self) -> u64 {
        self.config.window_size as u64 + self.source_ring_frames as u64
    }

    /// Absolute source frame index the producer must fill to.
    pub fn source_demand_frame(&self) -> u64 {
        self.next_analysis_frame
            .saturating_add(self.source_ring_frames as u64)
    }

    /// Source frames accepted so far.
    pub fn source_write_frame(&self) -> u64 {
        self.source_write_frame
    }

    /// Whether enough source is buffered for playback to start.
    pub fn ready(&self) -> bool {
        self.source_write_frame >= self.config.window_size as u64
    }

    /// Non-realtime producer entry point. Returns frames accepted, which is
    /// fewer than offered when the ring is full.
    ///
    /// Never called from the audio thread.
    pub fn push_source(&mut self, interleaved: &[Sample]) -> usize {
        let channel_count = self.config.channel_count;
        let offered = interleaved.len() / channel_count;
        let in_flight = self
            .source_write_frame
            .saturating_sub(self.next_analysis_frame) as usize;
        let free = self.source_ring_frames.saturating_sub(in_flight);
        let accepted = offered.min(free);
        for frame in 0..accepted {
            let ring_frame = (self.source_write_frame as usize + frame) % self.source_ring_frames;
            for channel in 0..channel_count {
                self.source_ring[ring_frame * channel_count + channel] =
                    interleaved[frame * channel_count + channel];
            }
        }
        self.source_write_frame = self.source_write_frame.saturating_add(accepted as u64);
        accepted
    }

    /// Audio-callback entry point: produce `frame_count` output frames,
    /// consuming however much source `ratio` demands.
    ///
    /// Allocation-free, lock-free, and I/O-free. Unlike the kernel it replaces
    /// it takes no input slice: source arrives through [`Self::push_source`].
    pub fn render(
        &mut self,
        output: &mut [Sample],
        frame_count: usize,
        ratio: f64,
    ) -> Result<RealtimePreviewStreamRenderReport, RealtimePreviewStreamError> {
        if frame_count > self.config.max_block_frames {
            return Err(RealtimePreviewStreamError::FrameCountExceedsConfig {
                requested: frame_count,
                max: self.config.max_block_frames,
            });
        }
        let required_samples = frame_count * self.config.channel_count;
        if output.len() < required_samples {
            return Err(RealtimePreviewStreamError::OutputTooSmall {
                required_samples,
                output_samples: output.len(),
            });
        }
        // Rejected, not clamped. A silently clamped ratio would make the
        // reported and actual source advance disagree, which is the class of
        // defect this lane exists to remove.
        if !ratio.is_finite()
            || !(REALTIME_PREVIEW_STREAM_MIN_RATIO..=REALTIME_PREVIEW_STREAM_MAX_RATIO)
                .contains(&ratio)
        {
            return Err(RealtimePreviewStreamError::RatioOutOfRange {
                requested: ratio,
                min: REALTIME_PREVIEW_STREAM_MIN_RATIO,
                max: REALTIME_PREVIEW_STREAM_MAX_RATIO,
            });
        }

        self.schedule_ratio_change(ratio);

        let analysis_start = self.next_analysis_frame;
        let target_output_frame = self.output_read_frame + frame_count as u64;
        let mut spectral_frames = 0usize;

        while self.next_synthesis_frame < target_output_frame as f64 {
            let window_end = self.next_analysis_frame + self.config.window_size as u64;
            if self.source_write_frame < window_end {
                // Underrun. Stop rather than advancing past source the producer
                // has not delivered: skipping is the defect this replaces.
                break;
            }
            let synthesis_start = self.next_synthesis_frame.round() as u64;
            self.apply_ratio_change_if_due(synthesis_start);
            let active = self.active_ratio;
            for channel in 0..self.config.channel_count {
                self.analyze(channel);
                self.propagate_phase(channel, active);
                self.synthesize(channel, synthesis_start);
            }
            self.next_analysis_frame = self
                .next_analysis_frame
                .saturating_add(self.config.analysis_hop as u64);
            self.next_synthesis_frame += self.config.analysis_hop as f64 * active;
            self.spectral_frame_index = self.spectral_frame_index.saturating_add(1);
            spectral_frames += 1;
        }

        // Frames past the synthesis frontier were never accumulated, so their
        // normalization weight is zero and `read_output` emits silence. The
        // count is arithmetic, not a second source of truth.
        let covered = (self.next_synthesis_frame.floor() as u64).min(target_output_frame);
        let underrun_frames = target_output_frame.saturating_sub(covered) as usize;

        self.read_output(output, frame_count);

        let consumed = self.next_analysis_frame.saturating_sub(analysis_start);
        self.total_source_frames_consumed =
            self.total_source_frames_consumed.saturating_add(consumed);

        Ok(RealtimePreviewStreamRenderReport {
            output_frames: frame_count,
            underrun_frames,
            source_frames_consumed: consumed,
            total_source_frames_consumed: self.total_source_frames_consumed,
            spectral_frames,
            requested_ratio: ratio,
            active_ratio: self.active_ratio,
            ratio_change_count: self.ratio_change_count,
            ratio_change_alignment_error_frames: self.last_alignment_error_frames,
            source_demand_frame: self.source_demand_frame(),
        })
    }

    fn schedule_ratio_change(&mut self, ratio: f64) {
        if (ratio - self.current_ratio).abs() <= f64::EPSILON {
            return;
        }
        self.current_ratio = ratio;
        self.pending_ratio = ratio;
        self.pending_request_frame = self.output_read_frame;
        self.pending_apply_frame =
            align_to_next_grid(self.output_read_frame, self.config.analysis_hop as u64);
        self.pending_change = true;
    }

    fn apply_ratio_change_if_due(&mut self, synthesis_start: u64) {
        if !self.pending_change || synthesis_start < self.pending_apply_frame {
            return;
        }
        self.active_ratio = self.pending_ratio;
        // Bounded by `analysis_hop` by construction: changes align to the hop
        // grid, so the request and application cannot be further apart.
        self.last_alignment_error_frames = synthesis_start.abs_diff(self.pending_request_frame);
        self.pending_change = false;
        self.ratio_change_count = self.ratio_change_count.saturating_add(1);
    }

    fn analyze(&mut self, channel: usize) {
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

    fn propagate_phase(&mut self, channel: usize, ratio: f64) {
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

    fn should_reset_at_transient(&self, channel: usize, ratio: f64) -> bool {
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

    fn lock_phase_to_peaks(&mut self, channel: usize) {
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

    fn peak_region_bounds(&self, peak_index: usize) -> (usize, usize) {
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

    fn synthesize(&mut self, channel: usize, synthesis_start: u64) {
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

    fn read_output(&mut self, output: &mut [Sample], frame_count: usize) {
        let channel_count = self.config.channel_count;
        for frame_offset in 0..frame_count {
            let ring_frame =
                (self.output_read_frame as usize + frame_offset) % self.output_ring_frames;
            for channel in 0..channel_count {
                let ring_index = ring_frame * channel_count + channel;
                let output_index = frame_offset * channel_count + channel;
                let weight = self.normalization_ring[ring_index];
                output[output_index] = if weight > 1.0e-3 {
                    self.output_ring[ring_index] / weight
                } else {
                    0.0
                };
                self.output_ring[ring_index] = 0.0;
                self.normalization_ring[ring_index] = 0.0;
            }
        }
        self.output_read_frame = self.output_read_frame.saturating_add(frame_count as u64);
    }
}
