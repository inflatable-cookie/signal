//! RealtimePreview callback state machine.

use rustfft::{num_complex::Complex32, FftPlanner};
use signal_primitives::Sample;

use super::contract::{
    build_realtime_preview_dynamic_source_projection_report,
    build_realtime_preview_source_projection_report, plan_realtime_preview_stream,
    project_realtime_preview_fixed_ratio_source_advance, DynamicSourceProjectionRatios,
    RealtimePreviewCallbackProcessError, RealtimePreviewCallbackProcessReport,
    RealtimePreviewCallbackState, RealtimePreviewDynamicSourceProjectionReport,
    RealtimePreviewPlanError, RealtimePreviewSourceProjectionReport, RealtimePreviewStreamConfig,
    RealtimePreviewStreamingContract,
};
use crate::{
    abs_diff_frames, align_to_next_grid, ceil_frame_to_usize, sanitize_ratio, usize_to_u64,
    wrap_phase,
};

impl RealtimePreviewCallbackState {
    /// Construct callback state and allocate all state-owned scratch outside
    /// the audio callback.
    pub fn new(config: RealtimePreviewStreamConfig) -> Result<Self, RealtimePreviewPlanError> {
        let contract = plan_realtime_preview_stream(config)?;
        let config = contract.config;
        let channel_count = config.channel_count;
        let bins = config.window_size / 2 + 1;
        let spectral_values = bins * channel_count;
        let spectral_samples = config.window_size * channel_count;
        let ring_frames =
            (config.window_size * 4 + config.max_block_frames * 4).max(config.window_size * 2);
        let mut planner = FftPlanner::<f32>::new();
        let forward = planner.plan_fft_forward(config.window_size);
        let inverse = planner.plan_fft_inverse(config.window_size);
        let forward_fft_scratch = vec![Complex32::new(0.0, 0.0); forward.get_inplace_scratch_len()];
        let inverse_fft_scratch = vec![Complex32::new(0.0, 0.0); inverse.get_inplace_scratch_len()];
        Ok(Self {
            config,
            scratch: vec![0.0; config.max_block_frames * channel_count],
            input_ring: vec![0.0; ring_frames * channel_count],
            output_ring: vec![0.0; ring_frames * channel_count],
            normalization_ring: vec![0.0; ring_frames * channel_count],
            window: (0..config.window_size)
                .map(|index| {
                    0.5 - 0.5
                        * (std::f32::consts::TAU * index as f32 / config.window_size as f32).cos()
                })
                .collect(),
            omega: (0..bins)
                .map(|bin| {
                    std::f32::consts::TAU * bin as f32 * config.analysis_hop as f32
                        / config.window_size as f32
                })
                .collect(),
            analysis_buffer: vec![Complex32::new(0.0, 0.0); spectral_samples],
            synthesis_spectrum: vec![Complex32::new(0.0, 0.0); spectral_samples],
            forward_fft_scratch,
            inverse_fft_scratch,
            previous_phase: vec![0.0; spectral_values],
            synthesis_phase: vec![0.0; spectral_values],
            current_magnitudes: vec![0.0; spectral_values],
            current_phases: vec![0.0; spectral_values],
            previous_magnitudes: vec![0.0; spectral_values],
            current_peak_bins: Vec::with_capacity(bins),
            forward,
            inverse,
            current_ratio: 1.0,
            active_ratio: 1.0,
            pending_ratio: 1.0,
            pending_ratio_request_frame: 0,
            pending_ratio_apply_frame: 0,
            pending_ratio_change: false,
            last_ratio_change_request_frame: 0,
            last_ratio_change_applied_frame: 0,
            last_ratio_change_output_frame: 0,
            last_ratio_change_alignment_error_frames: 0,
            ratio_change_count: 0,
            input_write_frame: 0,
            output_read_frame: 0,
            source_projection_output_frame: 0,
            source_projection_source_cursor: 0.0,
            last_source_projection: project_realtime_preview_fixed_ratio_source_advance(0, 0, 1.0),
            source_projection_current_ratio: 1.0,
            source_projection_active_ratio: 1.0,
            source_projection_pending_ratio: 1.0,
            source_projection_pending_ratio_request_frame: 0,
            source_projection_pending_ratio_apply_frame: 0,
            source_projection_pending_ratio_change: false,
            last_source_projection_ratio_change_request_frame: 0,
            last_source_projection_ratio_change_output_frame: 0,
            last_source_projection_ratio_change_source_frame: 0.0,
            last_source_projection_ratio_change_alignment_error_frames: 0,
            source_projection_ratio_change_count: 0,
            last_dynamic_source_projection: build_realtime_preview_dynamic_source_projection_report(
                0,
                0,
                0,
                0.0,
                0.0,
                DynamicSourceProjectionRatios::idle(),
            ),
            next_analysis_frame: 0,
            next_synthesis_frame: config.window_size as f64,
            processed_frames: 0,
            spectral_frame_index: 0,
            current_energy: vec![0.0; channel_count],
            previous_energy: vec![0.0; channel_count],
        })
    }

    /// Validated stream configuration.
    pub fn config(&self) -> RealtimePreviewStreamConfig {
        self.config
    }

    /// Current callback contract. This intentionally remains unsupported for
    /// direct audio-thread processing until streaming DSP lands.
    pub fn contract(&self) -> RealtimePreviewStreamingContract {
        plan_realtime_preview_stream(self.config)
            .expect("callback state stores a validated RealtimePreview config")
    }

    /// Preallocated scratch capacity in interleaved samples.
    pub fn scratch_capacity_samples(&self) -> usize {
        self.scratch.len()
    }

    /// Preallocated input ring capacity in interleaved samples.
    pub fn input_ring_capacity_samples(&self) -> usize {
        self.input_ring.len()
    }

    /// Preallocated output ring capacity in interleaved samples.
    pub fn output_ring_capacity_samples(&self) -> usize {
        self.output_ring.len()
    }

    /// Preallocated normalization ring capacity in interleaved samples.
    pub fn normalization_ring_capacity_samples(&self) -> usize {
        self.normalization_ring.len()
    }

    /// Preallocated analysis window length in sample frames.
    pub fn window_size(&self) -> usize {
        self.window.len()
    }

    /// Preallocated complex analysis/synthesis buffer size in samples.
    pub fn spectral_scratch_samples(&self) -> usize {
        self.analysis_buffer
            .len()
            .min(self.synthesis_spectrum.len())
    }

    /// Preallocated per-bin phase-state size.
    pub fn phase_state_values(&self) -> usize {
        self.previous_phase
            .len()
            .min(self.synthesis_phase.len())
            .min(self.current_phases.len())
            .min(self.current_magnitudes.len())
            .min(self.previous_magnitudes.len())
    }

    /// Current sanitized ratio remembered by the state.
    pub fn current_ratio(&self) -> f64 {
        self.current_ratio
    }

    /// Ratio currently applied to streaming spectral frames.
    pub fn active_ratio(&self) -> f64 {
        self.active_ratio
    }

    /// Number of scheduled ratio changes applied by this state.
    pub fn ratio_change_count(&self) -> u64 {
        self.ratio_change_count
    }

    /// Source frame where the latest applied ratio change was requested.
    pub fn last_ratio_change_request_frame(&self) -> u64 {
        self.last_ratio_change_request_frame
    }

    /// Source frame where the latest ratio change reached the analysis grid.
    pub fn last_ratio_change_applied_frame(&self) -> u64 {
        self.last_ratio_change_applied_frame
    }

    /// Output frame where the latest applied ratio change first contributes.
    pub fn last_ratio_change_output_frame(&self) -> u64 {
        self.last_ratio_change_output_frame
    }

    /// Source-frame error between the latest ratio request and its application.
    pub fn last_ratio_change_alignment_error_frames(&self) -> usize {
        self.last_ratio_change_alignment_error_frames
    }

    /// Contracted source-frame tolerance for scheduled ratio changes.
    pub fn ratio_change_alignment_tolerance_frames(&self) -> usize {
        self.config.analysis_hop + self.config.max_block_frames
    }

    /// Cumulative source-domain frames accepted by this state.
    pub fn processed_frames(&self) -> u64 {
        self.processed_frames
    }

    /// Output-domain cursor used by source-projection planning.
    pub fn source_projection_output_frame(&self) -> u64 {
        self.source_projection_output_frame
    }

    /// Fractional source-domain cursor used by source-projection planning.
    pub fn source_projection_source_cursor(&self) -> f64 {
        self.source_projection_source_cursor
    }

    /// Last source projection advanced by this callback state.
    pub fn last_source_projection(&self) -> RealtimePreviewSourceProjectionReport {
        self.last_source_projection
    }

    /// Ratio currently requested by source-projection planning.
    pub fn source_projection_current_ratio(&self) -> f64 {
        self.source_projection_current_ratio
    }

    /// Ratio currently applied to scheduled source-projection advancement.
    pub fn source_projection_active_ratio(&self) -> f64 {
        self.source_projection_active_ratio
    }

    /// Number of scheduled source-projection ratio changes applied by this state.
    pub fn source_projection_ratio_change_count(&self) -> u64 {
        self.source_projection_ratio_change_count
    }

    /// Last dynamic source projection advanced by this callback state.
    pub fn last_dynamic_source_projection(&self) -> RealtimePreviewDynamicSourceProjectionReport {
        self.last_dynamic_source_projection
    }

    /// Output frame where the latest projected ratio change was requested.
    pub fn last_source_projection_ratio_change_request_frame(&self) -> u64 {
        self.last_source_projection_ratio_change_request_frame
    }

    /// Output frame where the latest projected ratio change first contributes.
    pub fn last_source_projection_ratio_change_output_frame(&self) -> u64 {
        self.last_source_projection_ratio_change_output_frame
    }

    /// Fractional source frame where the latest projected ratio change applies.
    pub fn last_source_projection_ratio_change_source_frame(&self) -> f64 {
        self.last_source_projection_ratio_change_source_frame
    }

    /// Output-frame error between the latest projected ratio request and application.
    pub fn last_source_projection_ratio_change_alignment_error_frames(&self) -> usize {
        self.last_source_projection_ratio_change_alignment_error_frames
    }

    /// Conservative input-frame demand bound for one configured output block.
    pub fn source_projection_input_demand_limit_frames(&self, ratio: f64) -> usize {
        let ratio = sanitize_ratio(ratio);
        ceil_frame_to_usize(self.config.max_block_frames as f64 / ratio).saturating_add(1)
    }

    /// Advance callback-owned source projection state for one output quantum.
    pub fn advance_source_projection(
        &mut self,
        output_frames: usize,
        ratio: f64,
    ) -> Result<RealtimePreviewSourceProjectionReport, RealtimePreviewCallbackProcessError> {
        if output_frames > self.config.max_block_frames {
            return Err(
                RealtimePreviewCallbackProcessError::FrameCountExceedsConfig {
                    requested: output_frames,
                    max: self.config.max_block_frames,
                },
            );
        }

        let ratio = sanitize_ratio(ratio);
        let output_start_frame = self.source_projection_output_frame;
        let output_end_frame = output_start_frame.saturating_add(usize_to_u64(output_frames));
        let source_start_frame = self.source_projection_source_cursor;
        let source_end_frame = source_start_frame + output_frames as f64 / ratio;
        let projection = build_realtime_preview_source_projection_report(
            ratio,
            output_start_frame,
            output_frames,
            output_end_frame,
            source_start_frame,
            source_end_frame,
        );
        self.source_projection_output_frame = output_end_frame;
        self.source_projection_source_cursor = source_end_frame;
        self.last_source_projection = projection;
        Ok(projection)
    }

    /// Advance scheduled source projection state for one output quantum.
    pub fn advance_scheduled_source_projection(
        &mut self,
        output_frames: usize,
        ratio: f64,
    ) -> Result<RealtimePreviewDynamicSourceProjectionReport, RealtimePreviewCallbackProcessError>
    {
        if output_frames > self.config.max_block_frames {
            return Err(
                RealtimePreviewCallbackProcessError::FrameCountExceedsConfig {
                    requested: output_frames,
                    max: self.config.max_block_frames,
                },
            );
        }

        let ratio = sanitize_ratio(ratio);
        self.schedule_source_projection_ratio_change(ratio);

        let output_start_frame = self.source_projection_output_frame;
        let output_end_frame = output_start_frame.saturating_add(usize_to_u64(output_frames));
        let source_start_frame = self.source_projection_source_cursor;
        let start_ratio = self.source_projection_active_ratio;
        let mut source_end_frame = source_start_frame;
        let mut active_ratio = self.source_projection_active_ratio;
        let mut ratio_change_applied = false;

        if self.source_projection_pending_ratio_change
            && self.source_projection_pending_ratio_apply_frame <= output_start_frame
        {
            self.apply_source_projection_ratio_change(output_start_frame, source_end_frame);
            active_ratio = self.source_projection_active_ratio;
            ratio_change_applied = true;
        }

        if self.source_projection_pending_ratio_change
            && self.source_projection_pending_ratio_apply_frame < output_end_frame
        {
            let ratio_change_output_frame = self.source_projection_pending_ratio_apply_frame;
            let frames_before_change =
                abs_diff_frames(ratio_change_output_frame, output_start_frame);
            source_end_frame += frames_before_change as f64 / active_ratio;
            self.apply_source_projection_ratio_change(ratio_change_output_frame, source_end_frame);
            active_ratio = self.source_projection_active_ratio;
            ratio_change_applied = true;

            let frames_after_change = abs_diff_frames(output_end_frame, ratio_change_output_frame);
            source_end_frame += frames_after_change as f64 / active_ratio;
        } else {
            source_end_frame += output_frames as f64 / active_ratio;
        }

        let projection = build_realtime_preview_dynamic_source_projection_report(
            output_start_frame,
            output_frames,
            output_end_frame,
            source_start_frame,
            source_end_frame,
            DynamicSourceProjectionRatios {
                start_ratio,
                end_ratio: active_ratio,
                ratio_change_applied,
                ratio_change_count: self.source_projection_ratio_change_count,
                ratio_change_request_output_frame: self
                    .last_source_projection_ratio_change_request_frame,
                ratio_change_output_frame: self.last_source_projection_ratio_change_output_frame,
                ratio_change_source_frame: self.last_source_projection_ratio_change_source_frame,
                ratio_change_alignment_error_frames: self
                    .last_source_projection_ratio_change_alignment_error_frames,
            },
        );

        self.source_projection_output_frame = output_end_frame;
        self.source_projection_source_cursor = source_end_frame;
        self.last_dynamic_source_projection = projection;
        Ok(projection)
    }

    /// Reset callback state without reallocating.
    pub fn reset(&mut self) {
        self.scratch.fill(0.0);
        self.input_ring.fill(0.0);
        self.output_ring.fill(0.0);
        self.normalization_ring.fill(0.0);
        self.analysis_buffer.fill(Complex32::new(0.0, 0.0));
        self.synthesis_spectrum.fill(Complex32::new(0.0, 0.0));
        self.forward_fft_scratch.fill(Complex32::new(0.0, 0.0));
        self.inverse_fft_scratch.fill(Complex32::new(0.0, 0.0));
        self.previous_phase.fill(0.0);
        self.synthesis_phase.fill(0.0);
        self.current_magnitudes.fill(0.0);
        self.current_phases.fill(0.0);
        self.previous_magnitudes.fill(0.0);
        self.current_peak_bins.clear();
        self.current_ratio = 1.0;
        self.active_ratio = 1.0;
        self.pending_ratio = 1.0;
        self.pending_ratio_request_frame = 0;
        self.pending_ratio_apply_frame = 0;
        self.pending_ratio_change = false;
        self.last_ratio_change_request_frame = 0;
        self.last_ratio_change_applied_frame = 0;
        self.last_ratio_change_output_frame = 0;
        self.last_ratio_change_alignment_error_frames = 0;
        self.ratio_change_count = 0;
        self.input_write_frame = 0;
        self.output_read_frame = 0;
        self.source_projection_output_frame = 0;
        self.source_projection_source_cursor = 0.0;
        self.last_source_projection =
            project_realtime_preview_fixed_ratio_source_advance(0, 0, 1.0);
        self.source_projection_current_ratio = 1.0;
        self.source_projection_active_ratio = 1.0;
        self.source_projection_pending_ratio = 1.0;
        self.source_projection_pending_ratio_request_frame = 0;
        self.source_projection_pending_ratio_apply_frame = 0;
        self.source_projection_pending_ratio_change = false;
        self.last_source_projection_ratio_change_request_frame = 0;
        self.last_source_projection_ratio_change_output_frame = 0;
        self.last_source_projection_ratio_change_source_frame = 0.0;
        self.last_source_projection_ratio_change_alignment_error_frames = 0;
        self.source_projection_ratio_change_count = 0;
        self.last_dynamic_source_projection =
            build_realtime_preview_dynamic_source_projection_report(
                0,
                0,
                0,
                0.0,
                0.0,
                DynamicSourceProjectionRatios::idle(),
            );
        self.next_analysis_frame = 0;
        self.next_synthesis_frame = self.config.window_size as f64;
        self.processed_frames = 0;
        self.spectral_frame_index = 0;
        self.current_energy.fill(0.0);
        self.previous_energy.fill(0.0);
    }

    /// Process one callback quantum.
    ///
    /// Mono and linked-stereo streams run through the bounded preview kernel.
    /// The callback contract still reports unsupported render-plane routing
    /// until dynamic-ratio scheduling and integration proof land.
    pub fn process(
        &mut self,
        input: &[Sample],
        output: &mut [Sample],
        frame_count: usize,
        ratio: f64,
    ) -> Result<RealtimePreviewCallbackProcessReport, RealtimePreviewCallbackProcessError> {
        if frame_count > self.config.max_block_frames {
            return Err(
                RealtimePreviewCallbackProcessError::FrameCountExceedsConfig {
                    requested: frame_count,
                    max: self.config.max_block_frames,
                },
            );
        }
        let required_samples = frame_count * self.config.channel_count;
        if input.len() < required_samples || output.len() < required_samples {
            return Err(RealtimePreviewCallbackProcessError::BufferTooSmall {
                required_samples,
                input_samples: input.len(),
                output_samples: output.len(),
            });
        }
        let ratio = sanitize_ratio(ratio);
        self.schedule_ratio_change(ratio);
        self.push_interleaved_input(input, frame_count);
        self.process_available_streaming_frames();
        self.read_interleaved_output(output, frame_count);
        self.processed_frames = self.processed_frames.saturating_add(frame_count as u64);
        Ok(RealtimePreviewCallbackProcessReport {
            ratio,
            active_ratio: self.active_ratio,
            ratio_change_count: self.ratio_change_count,
            ratio_change_alignment_error_frames: self.last_ratio_change_alignment_error_frames,
            ratio_change_output_frame: self.last_ratio_change_output_frame,
            input_frames: frame_count,
            output_frames: frame_count,
            processed_frames: self.processed_frames,
        })
    }

    fn schedule_source_projection_ratio_change(&mut self, ratio: f64) {
        if (ratio - self.source_projection_current_ratio).abs() <= f64::EPSILON {
            return;
        }
        self.source_projection_current_ratio = ratio;
        self.source_projection_pending_ratio = ratio;
        self.source_projection_pending_ratio_request_frame = self.source_projection_output_frame;
        self.source_projection_pending_ratio_apply_frame = align_to_next_grid(
            self.source_projection_output_frame,
            self.config.analysis_hop as u64,
        );
        self.source_projection_pending_ratio_change = true;
    }

    fn apply_source_projection_ratio_change(&mut self, output_frame: u64, source_frame: f64) {
        self.source_projection_active_ratio = self.source_projection_pending_ratio;
        self.last_source_projection_ratio_change_request_frame =
            self.source_projection_pending_ratio_request_frame;
        self.last_source_projection_ratio_change_output_frame = output_frame;
        self.last_source_projection_ratio_change_source_frame = source_frame;
        self.last_source_projection_ratio_change_alignment_error_frames = abs_diff_frames(
            output_frame,
            self.source_projection_pending_ratio_request_frame,
        );
        self.source_projection_pending_ratio_change = false;
        self.source_projection_ratio_change_count =
            self.source_projection_ratio_change_count.saturating_add(1);
    }

    fn schedule_ratio_change(&mut self, ratio: f64) {
        if (ratio - self.current_ratio).abs() <= f64::EPSILON {
            return;
        }
        self.current_ratio = ratio;
        self.pending_ratio = ratio;
        self.pending_ratio_request_frame = self.input_write_frame;
        self.pending_ratio_apply_frame =
            align_to_next_grid(self.input_write_frame, self.config.analysis_hop as u64);
        self.pending_ratio_change = true;
    }

    fn ratio_for_next_analysis_frame(&mut self, synthesis_start: u64) -> f64 {
        if self.pending_ratio_change && self.next_analysis_frame >= self.pending_ratio_apply_frame {
            self.active_ratio = self.pending_ratio;
            self.last_ratio_change_request_frame = self.pending_ratio_request_frame;
            self.last_ratio_change_applied_frame = self.next_analysis_frame;
            self.last_ratio_change_output_frame = synthesis_start;
            self.last_ratio_change_alignment_error_frames =
                abs_diff_frames(self.next_analysis_frame, self.pending_ratio_request_frame);
            self.pending_ratio_change = false;
            self.ratio_change_count = self.ratio_change_count.saturating_add(1);
        }
        self.active_ratio
    }

    fn ring_frame_capacity(&self) -> usize {
        self.input_ring.len() / self.config.channel_count
    }

    fn push_interleaved_input(&mut self, input: &[Sample], frame_count: usize) {
        let ring_frames = self.ring_frame_capacity();
        let channel_count = self.config.channel_count;
        for frame_offset in 0..frame_count {
            let ring_frame = (self.input_write_frame as usize + frame_offset) % ring_frames;
            for channel in 0..channel_count {
                self.input_ring[ring_frame * channel_count + channel] =
                    input[frame_offset * channel_count + channel];
            }
        }
        self.input_write_frame = self.input_write_frame.saturating_add(frame_count as u64);
    }

    fn process_available_streaming_frames(&mut self) {
        let ring_frames = self.ring_frame_capacity() as u64;
        while self.next_analysis_frame + self.config.window_size as u64 <= self.input_write_frame {
            if self
                .input_write_frame
                .saturating_sub(self.next_analysis_frame)
                > ring_frames
            {
                self.next_analysis_frame = self.input_write_frame.saturating_sub(ring_frames);
            }
            let synthesis_start = self.next_synthesis_frame.round() as u64;
            if synthesis_start + self.config.window_size as u64
                >= self.output_read_frame.saturating_add(ring_frames)
            {
                break;
            }
            let ratio = self.ratio_for_next_analysis_frame(synthesis_start);
            for channel in 0..self.config.channel_count {
                self.analyze_streaming_frame(channel);
                self.propagate_streaming_phase(channel, ratio);
                self.synthesize_streaming_frame(channel, synthesis_start);
            }
            self.next_analysis_frame = self
                .next_analysis_frame
                .saturating_add(self.config.analysis_hop as u64);
            self.next_synthesis_frame += self.config.analysis_hop as f64 * ratio;
            self.spectral_frame_index = self.spectral_frame_index.saturating_add(1);
        }
    }

    fn analyze_streaming_frame(&mut self, channel: usize) {
        let ring_frames = self.ring_frame_capacity();
        let channel_count = self.config.channel_count;
        let fft_offset = channel * self.config.window_size;
        self.current_energy[channel] = 0.0;
        for index in 0..self.config.window_size {
            let source_index = (self.next_analysis_frame as usize + index) % ring_frames;
            let windowed =
                self.input_ring[source_index * channel_count + channel] * self.window[index];
            self.current_energy[channel] += (windowed * windowed) as f64;
            self.analysis_buffer[fft_offset + index] = Complex32::new(windowed, 0.0);
        }
        self.current_energy[channel] /= self.config.window_size as f64;
        self.forward.process_with_scratch(
            &mut self.analysis_buffer[fft_offset..fft_offset + self.config.window_size],
            &mut self.forward_fft_scratch,
        );
    }

    fn propagate_streaming_phase(&mut self, channel: usize, ratio: f64) {
        let bins = self.config.window_size / 2 + 1;
        let fft_offset = channel * self.config.window_size;
        let bin_offset = channel * bins;
        let is_first_frame = self.spectral_frame_index == 0;
        let reset_at_transient =
            self.should_reset_streaming_phase_at_transient(channel, bins, ratio);
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

        self.lock_streaming_phase_to_peaks(channel, bins);
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

    fn should_reset_streaming_phase_at_transient(
        &self,
        channel: usize,
        bins: usize,
        ratio: f64,
    ) -> bool {
        if self.spectral_frame_index == 0 || ratio < 1.0 {
            return false;
        }
        let fft_offset = channel * self.config.window_size;
        let bin_offset = channel * bins;
        let mut flux = 0.0f32;
        let mut magnitude_sum = 0.0f32;
        for bin in 0..bins {
            let magnitude = self.analysis_buffer[fft_offset + bin].norm();
            flux += (magnitude - self.previous_magnitudes[bin_offset + bin]).max(0.0);
            magnitude_sum += magnitude;
        }
        let flux_ratio = flux as f64 / (magnitude_sum as f64 + 1.0e-12);
        let energy_ratio = self.current_energy[channel] / (self.previous_energy[channel] + 1.0e-12);
        flux_ratio >= 0.30 && energy_ratio >= 1.20
    }

    fn lock_streaming_phase_to_peaks(&mut self, channel: usize, bins: usize) {
        if self.current_peak_bins.is_empty() {
            return;
        }
        let bin_offset = channel * bins;
        for peak_index in 0..self.current_peak_bins.len() {
            let peak_bin = self.current_peak_bins[peak_index];
            let peak_phase = self.synthesis_phase[bin_offset + peak_bin];
            let analysis_peak_phase = self.current_phases[bin_offset + peak_bin];
            let (left, right) = self.streaming_peak_region_bounds(peak_index, bins);
            for bin in left..right {
                let index = bin_offset + bin;
                let relative_phase = wrap_phase(self.current_phases[index] - analysis_peak_phase);
                self.synthesis_phase[index] = wrap_phase(peak_phase + relative_phase);
            }
        }
    }

    fn streaming_peak_region_bounds(&self, peak_index: usize, bins: usize) -> (usize, usize) {
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
            .unwrap_or(bins);
        (left, right)
    }

    fn synthesize_streaming_frame(&mut self, channel: usize, synthesis_start: u64) {
        let fft_offset = channel * self.config.window_size;
        self.inverse.process_with_scratch(
            &mut self.synthesis_spectrum[fft_offset..fft_offset + self.config.window_size],
            &mut self.inverse_fft_scratch,
        );
        let ring_frames = self.ring_frame_capacity();
        let channel_count = self.config.channel_count;
        let scale = 1.0 / self.config.window_size as f32;
        for index in 0..self.config.window_size {
            let output_index = (synthesis_start as usize + index) % ring_frames;
            let ring_index = output_index * channel_count + channel;
            let sample =
                self.synthesis_spectrum[fft_offset + index].re * scale * self.window[index];
            self.output_ring[ring_index] += sample;
            self.normalization_ring[ring_index] += self.window[index] * self.window[index];
        }
    }

    fn read_interleaved_output(&mut self, output: &mut [Sample], frame_count: usize) {
        let ring_frames = self.ring_frame_capacity();
        let channel_count = self.config.channel_count;
        for frame_offset in 0..frame_count {
            let ring_frame = (self.output_read_frame as usize + frame_offset) % ring_frames;
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
