use rustfft::FftPlanner;

use super::super::contract::{
    build_realtime_preview_dynamic_source_projection_report, plan_realtime_preview_stream,
    project_realtime_preview_fixed_ratio_source_advance, DynamicSourceProjectionRatios,
    RealtimePreviewCallbackState, RealtimePreviewDynamicSourceProjectionReport,
    RealtimePreviewPlanError, RealtimePreviewSourceProjectionReport, RealtimePreviewStreamConfig,
    RealtimePreviewStreamingContract,
};
use crate::{ceil_frame_to_usize, sanitize_ratio};
use rustfft::num_complex::Complex32;

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
}
