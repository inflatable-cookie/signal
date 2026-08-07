use rustfft::num_complex::Complex32;
use rustfft::FftPlanner;

use signal_primitives::Sample;

use crate::realtime_preview::{RealtimePreviewPlanError, RealtimePreviewStreamConfig};

use super::constants::REALTIME_PREVIEW_STREAM_MIN_RATIO;
use super::types::RealtimePreviewStreamState;

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
    pub(super) fn prefill_frames_for(config: &RealtimePreviewStreamConfig) -> usize {
        let per_callback =
            (config.max_block_frames as f64 / REALTIME_PREVIEW_STREAM_MIN_RATIO).ceil() as usize;
        per_callback * 2 + config.window_size
    }
}
