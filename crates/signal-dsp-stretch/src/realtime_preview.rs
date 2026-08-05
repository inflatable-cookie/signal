//! Callback-facing RealtimePreview tier.
//!
//! This tier has no consumer outside the crate and its callback path is not
//! render-plane usable: `process` is quantum-locked, so at any ratio other
//! than `1.0` it stalls analysis or drops source frames while returning `Ok`.
//! `g10.040` decides whether the tier is completed or closed; `g10.038`
//! deliberately left it intact and only moved it out of `lib.rs`.

use std::sync::Arc;

use rustfft::{num_complex::Complex32, Fft, FftPlanner};
use signal_primitives::{Sample, SampleRate};

use crate::{
    abs_diff_frames, align_to_next_grid, ceil_frame_to_u64, ceil_frame_to_usize,
    floor_frame_to_u64, sanitize_ratio, usize_to_u64, wrap_phase, REALTIME_PREVIEW_ANALYSIS_HOP,
    REALTIME_PREVIEW_WINDOW_SIZE,
};

/// Integration posture for a RealtimePreview stream.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RealtimePreviewIntegrationMode {
    /// Preview renders are built control-side and handed to the render plane
    /// as normal sample buffers.
    AnticipativePreRender,
    /// Direct render-callback processing by a proven allocation-free state
    /// object. This mode is not implemented yet.
    CallbackSafeStreaming,
}

/// Source/output timeline mode for a RealtimePreview callback stream.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RealtimePreviewCallbackTimelineMode {
    /// The caller supplies one input quantum for one output quantum. This can
    /// prove callback-local DSP safety, but it is not a render-plane
    /// time-stretch source-advance contract.
    QuantumLocked,
    /// The callback state owns ratio-projected source advancement and reports
    /// consumed source position against produced output position.
    SourceProjected,
}

/// Configuration used to plan a RealtimePreview stream.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RealtimePreviewStreamConfig {
    /// Session sample rate.
    pub sample_rate: SampleRate,
    /// Number of linked channels in the preview stream.
    pub channel_count: usize,
    /// Maximum render quantum or preview block size in sample frames.
    pub max_block_frames: usize,
    /// STFT window size in sample frames.
    pub window_size: usize,
    /// Analysis hop in sample frames.
    pub analysis_hop: usize,
}

/// Planned latency and routing contract for a RealtimePreview stream.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RealtimePreviewStreamingContract {
    /// Validated stream configuration.
    pub config: RealtimePreviewStreamConfig,
    /// Current integration posture.
    pub integration_mode: RealtimePreviewIntegrationMode,
    /// Source/output timeline contract for callback processing.
    pub callback_timeline_mode: RealtimePreviewCallbackTimelineMode,
    /// Input-side latency in sample frames.
    pub input_latency_frames: usize,
    /// Output-side latency in sample frames.
    pub output_latency_frames: usize,
    /// Maximum source-frame alignment tolerance for an immediate ratio change.
    pub ratio_change_alignment_tolerance_frames: usize,
    /// Whether the planned path may run directly on the realtime callback.
    pub audio_thread_processing_supported: bool,
    /// Unsupported mode that keeps this contract out of direct callback use.
    pub unsupported_mode: Option<RealtimePreviewUnsupportedMode>,
}

/// Fixed-ratio source projection for a RealtimePreview output span.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RealtimePreviewSourceProjectionReport {
    /// Sanitized output/input duration ratio.
    pub ratio: f64,
    /// Output-domain start frame for the projected span.
    pub output_start_frame: u64,
    /// Output frames in this projected span.
    pub output_frames: usize,
    /// Exclusive output-domain end frame for the projected span.
    pub output_end_frame: u64,
    /// Fractional source-domain start frame.
    pub source_start_frame: f64,
    /// Fractional source-domain end frame.
    pub source_end_frame: f64,
    /// Fractional source-domain advance required for this output span.
    pub source_advance_frames: f64,
    /// First integer source frame needed by the projected span.
    pub source_frame_floor: u64,
    /// Exclusive integer source frame bound needed by the projected span.
    pub source_frame_ceil: u64,
    /// Integer source frame count covering the projected fractional span.
    pub source_frames_required: usize,
}

/// Dynamic-ratio source projection for a RealtimePreview output span.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RealtimePreviewDynamicSourceProjectionReport {
    /// Output-domain start frame for the projected span.
    pub output_start_frame: u64,
    /// Output frames in this projected span.
    pub output_frames: usize,
    /// Exclusive output-domain end frame for the projected span.
    pub output_end_frame: u64,
    /// Fractional source-domain start frame.
    pub source_start_frame: f64,
    /// Fractional source-domain end frame.
    pub source_end_frame: f64,
    /// Fractional source-domain advance required for this output span.
    pub source_advance_frames: f64,
    /// First integer source frame needed by the projected span.
    pub source_frame_floor: u64,
    /// Exclusive integer source frame bound needed by the projected span.
    pub source_frame_ceil: u64,
    /// Integer source frame count covering the projected fractional span.
    pub source_frames_required: usize,
    /// Active ratio at the start of this projection span.
    pub start_ratio: f64,
    /// Active ratio at the end of this projection span.
    pub end_ratio: f64,
    /// Whether a scheduled ratio change was applied inside this span.
    pub ratio_change_applied: bool,
    /// Number of scheduled source-projection ratio changes applied by this state.
    pub ratio_change_count: u64,
    /// Output frame where the latest source-projection ratio change was requested.
    pub ratio_change_request_output_frame: u64,
    /// Output frame where the latest source-projection ratio change first contributes.
    pub ratio_change_output_frame: u64,
    /// Fractional source frame at the latest source-projection ratio change seam.
    pub ratio_change_source_frame: f64,
    /// Output-frame error between latest ratio request and application.
    pub ratio_change_alignment_error_frames: usize,
}

/// Unsupported RealtimePreview routing mode.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RealtimePreviewUnsupportedMode {
    /// Source projection is reported, but the callback path does not yet own
    /// bounded source fill, underrun, or input-demand behavior.
    SourceBufferingContract,
}

/// RealtimePreview stream planning failure.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RealtimePreviewPlanError {
    /// The sample rate is zero.
    InvalidSampleRate,
    /// The channel count is zero or not currently supported.
    UnsupportedChannelCount(usize),
    /// The maximum block size is zero.
    InvalidBlockSize,
}

/// Callback-facing RealtimePreview state.
///
/// This state owns the preallocated scratch required for the callback-facing
/// RealtimePreview kernel. Batch 26.2 supports mono and linked-stereo
/// streaming DSP; render-plane routing remains gated.
pub struct RealtimePreviewCallbackState {
    config: RealtimePreviewStreamConfig,
    scratch: Vec<Sample>,
    input_ring: Vec<Sample>,
    output_ring: Vec<Sample>,
    normalization_ring: Vec<f32>,
    window: Vec<f32>,
    omega: Vec<f32>,
    analysis_buffer: Vec<Complex32>,
    synthesis_spectrum: Vec<Complex32>,
    forward_fft_scratch: Vec<Complex32>,
    inverse_fft_scratch: Vec<Complex32>,
    previous_phase: Vec<f32>,
    synthesis_phase: Vec<f32>,
    current_magnitudes: Vec<f32>,
    current_phases: Vec<f32>,
    previous_magnitudes: Vec<f32>,
    current_peak_bins: Vec<usize>,
    forward: Arc<dyn Fft<f32>>,
    inverse: Arc<dyn Fft<f32>>,
    current_ratio: f64,
    active_ratio: f64,
    pending_ratio: f64,
    pending_ratio_request_frame: u64,
    pending_ratio_apply_frame: u64,
    pending_ratio_change: bool,
    last_ratio_change_request_frame: u64,
    last_ratio_change_applied_frame: u64,
    last_ratio_change_output_frame: u64,
    last_ratio_change_alignment_error_frames: usize,
    ratio_change_count: u64,
    input_write_frame: u64,
    output_read_frame: u64,
    source_projection_output_frame: u64,
    source_projection_source_cursor: f64,
    last_source_projection: RealtimePreviewSourceProjectionReport,
    source_projection_current_ratio: f64,
    source_projection_active_ratio: f64,
    source_projection_pending_ratio: f64,
    source_projection_pending_ratio_request_frame: u64,
    source_projection_pending_ratio_apply_frame: u64,
    source_projection_pending_ratio_change: bool,
    last_source_projection_ratio_change_request_frame: u64,
    last_source_projection_ratio_change_output_frame: u64,
    last_source_projection_ratio_change_source_frame: f64,
    last_source_projection_ratio_change_alignment_error_frames: usize,
    source_projection_ratio_change_count: u64,
    last_dynamic_source_projection: RealtimePreviewDynamicSourceProjectionReport,
    next_analysis_frame: u64,
    next_synthesis_frame: f64,
    processed_frames: u64,
    spectral_frame_index: u64,
    current_energy: Vec<f64>,
    previous_energy: Vec<f64>,
}

/// Report returned by a successful RealtimePreview callback process call.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RealtimePreviewCallbackProcessReport {
    /// Sanitized ratio requested by this block.
    pub ratio: f64,
    /// Active ratio at the end of this process call.
    pub active_ratio: f64,
    /// Number of scheduled ratio changes applied by this state.
    pub ratio_change_count: u64,
    /// Alignment error, in source frames, for the last applied ratio change.
    pub ratio_change_alignment_error_frames: usize,
    /// Output frame where the last applied ratio change first contributes.
    pub ratio_change_output_frame: u64,
    /// Frames consumed from the input block.
    pub input_frames: usize,
    /// Frames produced into the output block.
    pub output_frames: usize,
    /// Cumulative source-domain frames accepted by this state.
    pub processed_frames: u64,
}

/// RealtimePreview callback process failure.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RealtimePreviewCallbackProcessError {
    /// The requested frame count exceeds the state's configured maximum block.
    FrameCountExceedsConfig {
        /// Requested process frame count.
        requested: usize,
        /// Configured maximum frame count.
        max: usize,
    },
    /// Input or output buffer is shorter than `frame_count * channel_count`.
    BufferTooSmall {
        /// Buffer samples required for this block.
        required_samples: usize,
        /// Available input samples.
        input_samples: usize,
        /// Available output samples.
        output_samples: usize,
    },
}

impl RealtimePreviewStreamConfig {
    /// Default RealtimePreview stream configuration for a session.
    pub fn new(sample_rate: SampleRate, channel_count: usize, max_block_frames: usize) -> Self {
        Self {
            sample_rate,
            channel_count,
            max_block_frames,
            window_size: REALTIME_PREVIEW_WINDOW_SIZE,
            analysis_hop: REALTIME_PREVIEW_ANALYSIS_HOP,
        }
    }

    /// Clamp window and hop sizes to the supported STFT range.
    pub fn normalized(self) -> Self {
        let window_size = self.window_size.next_power_of_two().max(64);
        let analysis_hop = self.analysis_hop.clamp(1, window_size / 2);
        Self {
            window_size,
            analysis_hop,
            ..self
        }
    }
}

/// Build a RealtimePreview streaming contract.
///
/// The first Signal-owned preview implementation is intentionally
/// anticipative: it defines latency and ratio-change tolerance, but returns an
/// unsupported callback mode until the state object proves allocation-free
/// bounded work.
pub fn plan_realtime_preview_stream(
    config: RealtimePreviewStreamConfig,
) -> Result<RealtimePreviewStreamingContract, RealtimePreviewPlanError> {
    if config.sample_rate.0 == 0 {
        return Err(RealtimePreviewPlanError::InvalidSampleRate);
    }
    if !(1..=2).contains(&config.channel_count) {
        return Err(RealtimePreviewPlanError::UnsupportedChannelCount(
            config.channel_count,
        ));
    }
    if config.max_block_frames == 0 {
        return Err(RealtimePreviewPlanError::InvalidBlockSize);
    }
    let config = config.normalized();
    Ok(RealtimePreviewStreamingContract {
        input_latency_frames: config.window_size,
        output_latency_frames: config.window_size,
        ratio_change_alignment_tolerance_frames: config.analysis_hop + config.max_block_frames,
        integration_mode: RealtimePreviewIntegrationMode::AnticipativePreRender,
        callback_timeline_mode: RealtimePreviewCallbackTimelineMode::QuantumLocked,
        audio_thread_processing_supported: false,
        unsupported_mode: Some(RealtimePreviewUnsupportedMode::SourceBufferingContract),
        config,
    })
}

/// Project a fixed-ratio RealtimePreview output span into source frames.
///
/// `ratio` uses the crate-wide output/input duration convention: `2.0`
/// produces twice as much output time as source time, so a 256-frame output
/// quantum advances 128 source frames.
pub fn project_realtime_preview_fixed_ratio_source_advance(
    output_start_frame: u64,
    output_frames: usize,
    ratio: f64,
) -> RealtimePreviewSourceProjectionReport {
    let ratio = sanitize_ratio(ratio);
    let output_end_frame = output_start_frame.saturating_add(usize_to_u64(output_frames));
    let source_start_frame = output_start_frame as f64 / ratio;
    let source_end_frame = output_end_frame as f64 / ratio;
    build_realtime_preview_source_projection_report(
        ratio,
        output_start_frame,
        output_frames,
        output_end_frame,
        source_start_frame,
        source_end_frame,
    )
}

fn build_realtime_preview_source_projection_report(
    ratio: f64,
    output_start_frame: u64,
    output_frames: usize,
    output_end_frame: u64,
    source_start_frame: f64,
    source_end_frame: f64,
) -> RealtimePreviewSourceProjectionReport {
    let source_frame_floor = floor_frame_to_u64(source_start_frame);
    let source_frame_ceil = ceil_frame_to_u64(source_end_frame);
    let source_frames_required = abs_diff_frames(source_frame_ceil, source_frame_floor);

    RealtimePreviewSourceProjectionReport {
        ratio,
        output_start_frame,
        output_frames,
        output_end_frame,
        source_start_frame,
        source_end_frame,
        source_advance_frames: source_end_frame - source_start_frame,
        source_frame_floor,
        source_frame_ceil,
        source_frames_required,
    }
}

/// The ratio half of a dynamic source projection: the ratios active across the
/// span, plus the running record of the latest scheduled ratio change. Grouped
/// so the projection builder takes a span and its ratio state rather than
/// thirteen positional arguments.
#[derive(Clone, Copy, Debug)]
struct DynamicSourceProjectionRatios {
    start_ratio: f64,
    end_ratio: f64,
    ratio_change_applied: bool,
    ratio_change_count: u64,
    ratio_change_request_output_frame: u64,
    ratio_change_output_frame: u64,
    ratio_change_source_frame: f64,
    ratio_change_alignment_error_frames: usize,
}

impl DynamicSourceProjectionRatios {
    /// Unity ratio, no change ever scheduled — the reset and construction state.
    const fn idle() -> Self {
        Self {
            start_ratio: 1.0,
            end_ratio: 1.0,
            ratio_change_applied: false,
            ratio_change_count: 0,
            ratio_change_request_output_frame: 0,
            ratio_change_output_frame: 0,
            ratio_change_source_frame: 0.0,
            ratio_change_alignment_error_frames: 0,
        }
    }
}

fn build_realtime_preview_dynamic_source_projection_report(
    output_start_frame: u64,
    output_frames: usize,
    output_end_frame: u64,
    source_start_frame: f64,
    source_end_frame: f64,
    ratios: DynamicSourceProjectionRatios,
) -> RealtimePreviewDynamicSourceProjectionReport {
    let DynamicSourceProjectionRatios {
        start_ratio,
        end_ratio,
        ratio_change_applied,
        ratio_change_count,
        ratio_change_request_output_frame,
        ratio_change_output_frame,
        ratio_change_source_frame,
        ratio_change_alignment_error_frames,
    } = ratios;

    let source_frame_floor = floor_frame_to_u64(source_start_frame);
    let source_frame_ceil = ceil_frame_to_u64(source_end_frame);
    let source_frames_required = abs_diff_frames(source_frame_ceil, source_frame_floor);

    RealtimePreviewDynamicSourceProjectionReport {
        output_start_frame,
        output_frames,
        output_end_frame,
        source_start_frame,
        source_end_frame,
        source_advance_frames: source_end_frame - source_start_frame,
        source_frame_floor,
        source_frame_ceil,
        source_frames_required,
        start_ratio,
        end_ratio,
        ratio_change_applied,
        ratio_change_count,
        ratio_change_request_output_frame,
        ratio_change_output_frame,
        ratio_change_source_frame,
        ratio_change_alignment_error_frames,
    }
}

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::benchmark::{
        compare_synthetic_realtime_preview_backends, generate_synthetic_stretch_audio,
        measure_dynamic_segment_seam_click, StretchBenchmarkBackend, StretchBenchmarkPath,
        StretchCorpusFamily, StretchMetric,
    };
    use crate::{RealtimePreviewStretcher, StretchQuality, StretchRatioPoint, TimeStretcher};

    fn sine(frequency_hz: f32, sample_rate_hz: f32, len: usize) -> Vec<Sample> {
        (0..len)
            .map(|index| {
                (std::f32::consts::TAU * frequency_hz * index as f32 / sample_rate_hz).sin()
            })
            .collect()
    }

    fn rms(samples: &[Sample]) -> f32 {
        (samples.iter().map(|s| s * s).sum::<f32>() / samples.len().max(1) as f32).sqrt()
    }

    /// Dominant frequency estimate by zero-crossing count over a trimmed
    /// interior span (skips windup/tail edges).
    fn dominant_frequency_hz(samples: &[Sample], sample_rate_hz: f32) -> f32 {
        let margin = samples.len() / 8;
        let interior = &samples[margin..samples.len() - margin];
        let crossings = interior
            .windows(2)
            .filter(|pair| (pair[0] >= 0.0) != (pair[1] >= 0.0))
            .count();
        crossings as f32 * sample_rate_hz / (2.0 * interior.len() as f32)
    }

    #[test]
    fn realtime_preview_contract_reports_latency_and_callback_blocker() {
        let contract = plan_realtime_preview_stream(RealtimePreviewStreamConfig::new(
            SampleRate(48_000),
            2,
            128,
        ))
        .expect("default preview contract should plan");

        assert_eq!(contract.config.window_size, REALTIME_PREVIEW_WINDOW_SIZE);
        assert_eq!(contract.config.analysis_hop, REALTIME_PREVIEW_ANALYSIS_HOP);
        assert_eq!(contract.input_latency_frames, REALTIME_PREVIEW_WINDOW_SIZE);
        assert_eq!(contract.output_latency_frames, REALTIME_PREVIEW_WINDOW_SIZE);
        assert_eq!(
            contract.ratio_change_alignment_tolerance_frames,
            REALTIME_PREVIEW_ANALYSIS_HOP + 128
        );
        assert_eq!(
            contract.integration_mode,
            RealtimePreviewIntegrationMode::AnticipativePreRender
        );
        assert_eq!(
            contract.callback_timeline_mode,
            RealtimePreviewCallbackTimelineMode::QuantumLocked
        );
        assert!(!contract.audio_thread_processing_supported);
        assert_eq!(
            contract.unsupported_mode,
            Some(RealtimePreviewUnsupportedMode::SourceBufferingContract)
        );
    }

    #[test]
    fn realtime_preview_contract_rejects_invalid_streams() {
        assert_eq!(
            plan_realtime_preview_stream(RealtimePreviewStreamConfig::new(SampleRate(0), 2, 128,)),
            Err(RealtimePreviewPlanError::InvalidSampleRate)
        );
        assert_eq!(
            plan_realtime_preview_stream(RealtimePreviewStreamConfig::new(
                SampleRate(48_000),
                6,
                128,
            )),
            Err(RealtimePreviewPlanError::UnsupportedChannelCount(6))
        );
        assert_eq!(
            plan_realtime_preview_stream(RealtimePreviewStreamConfig::new(
                SampleRate(48_000),
                2,
                0,
            )),
            Err(RealtimePreviewPlanError::InvalidBlockSize)
        );
    }

    #[test]
    fn realtime_preview_fixed_ratio_source_projection_reports_required_source_span() {
        let slow = project_realtime_preview_fixed_ratio_source_advance(480, 96, 2.0);
        assert_eq!(slow.ratio, 2.0);
        assert_eq!(slow.output_start_frame, 480);
        assert_eq!(slow.output_frames, 96);
        assert_eq!(slow.output_end_frame, 576);
        assert_eq!(slow.source_start_frame, 240.0);
        assert_eq!(slow.source_end_frame, 288.0);
        assert_eq!(slow.source_advance_frames, 48.0);
        assert_eq!(slow.source_frame_floor, 240);
        assert_eq!(slow.source_frame_ceil, 288);
        assert_eq!(slow.source_frames_required, 48);

        let fast = project_realtime_preview_fixed_ratio_source_advance(480, 96, 0.5);
        assert_eq!(fast.source_start_frame, 960.0);
        assert_eq!(fast.source_end_frame, 1152.0);
        assert_eq!(fast.source_advance_frames, 192.0);
        assert_eq!(fast.source_frames_required, 192);

        let identity = project_realtime_preview_fixed_ratio_source_advance(480, 96, 1.0);
        assert_eq!(identity.source_start_frame, 480.0);
        assert_eq!(identity.source_end_frame, 576.0);
        assert_eq!(identity.source_frames_required, 96);
    }

    #[test]
    fn realtime_preview_fixed_ratio_source_projection_covers_fractional_source_bounds() {
        let projection = project_realtime_preview_fixed_ratio_source_advance(0, 256, 1.5);

        assert_eq!(projection.source_frame_floor, 0);
        assert_eq!(projection.source_frame_ceil, 171);
        assert_eq!(projection.source_frames_required, 171);
        assert!((projection.source_advance_frames - (256.0 / 1.5)).abs() < 1.0e-9);

        let sanitized = project_realtime_preview_fixed_ratio_source_advance(32, 64, f64::NAN);
        assert_eq!(sanitized.ratio, 1.0);
        assert_eq!(sanitized.source_start_frame, 32.0);
        assert_eq!(sanitized.source_end_frame, 96.0);
    }

    #[test]
    fn realtime_preview_source_projection_state_advances_fractional_cursor() {
        let mut state = RealtimePreviewCallbackState::new(RealtimePreviewStreamConfig::new(
            SampleRate(48_000),
            2,
            128,
        ))
        .expect("callback state config should validate");

        let first = state
            .advance_source_projection(96, 1.5)
            .expect("projection should stay within the configured block size");
        let second = state
            .advance_source_projection(96, 1.5)
            .expect("projection should stay within the configured block size");

        assert_eq!(first.output_start_frame, 0);
        assert_eq!(first.output_end_frame, 96);
        assert_eq!(first.source_start_frame, 0.0);
        assert_eq!(first.source_end_frame, 64.0);
        assert_eq!(first.source_frames_required, 64);
        assert_eq!(second.output_start_frame, 96);
        assert_eq!(second.output_end_frame, 192);
        assert_eq!(second.source_start_frame, 64.0);
        assert_eq!(second.source_end_frame, 128.0);
        assert_eq!(second.source_frames_required, 64);
        assert_eq!(state.source_projection_output_frame(), 192);
        assert_eq!(state.source_projection_source_cursor(), 128.0);
        assert_eq!(state.last_source_projection(), second);

        state.reset();
        assert_eq!(state.source_projection_output_frame(), 0);
        assert_eq!(state.source_projection_source_cursor(), 0.0);
        assert_eq!(
            state.last_source_projection(),
            project_realtime_preview_fixed_ratio_source_advance(0, 0, 1.0)
        );
    }

    #[test]
    fn realtime_preview_source_projection_state_bounds_input_demand() {
        let mut state = RealtimePreviewCallbackState::new(RealtimePreviewStreamConfig::new(
            SampleRate(48_000),
            1,
            128,
        ))
        .expect("callback state config should validate");

        let fast_limit = state.source_projection_input_demand_limit_frames(0.5);
        let fast = state
            .advance_source_projection(128, 0.5)
            .expect("projection should stay within the configured block size");
        assert_eq!(fast.source_advance_frames, 256.0);
        assert_eq!(fast.source_frames_required, 256);
        assert!(fast.source_frames_required <= fast_limit);

        let fractional_limit = state.source_projection_input_demand_limit_frames(3.0);
        let fractional = state
            .advance_source_projection(100, 3.0)
            .expect("projection should stay within the configured block size");
        assert!((fractional.source_advance_frames - (100.0 / 3.0)).abs() < 1.0e-9);
        assert_eq!(fractional.source_frame_floor, 256);
        assert_eq!(fractional.source_frame_ceil, 290);
        assert_eq!(fractional.source_frames_required, 34);
        assert!(fractional.source_frames_required <= fractional_limit);

        assert_eq!(
            state.advance_source_projection(129, 1.0),
            Err(
                RealtimePreviewCallbackProcessError::FrameCountExceedsConfig {
                    requested: 129,
                    max: 128,
                }
            )
        );
    }

    #[test]
    fn realtime_preview_source_projection_state_is_deterministic_for_fixed_ratio() {
        let mut first = RealtimePreviewCallbackState::new(RealtimePreviewStreamConfig::new(
            SampleRate(48_000),
            1,
            128,
        ))
        .expect("callback state config should validate");
        let mut second = RealtimePreviewCallbackState::new(RealtimePreviewStreamConfig::new(
            SampleRate(48_000),
            1,
            128,
        ))
        .expect("callback state config should validate");

        for _ in 0..16 {
            let first_report = first
                .advance_source_projection(100, 3.0)
                .expect("projection should stay within the configured block size");
            let second_report = second
                .advance_source_projection(100, 3.0)
                .expect("projection should stay within the configured block size");
            assert_eq!(first_report, second_report);
            assert!(first_report.source_frames_required <= 35);
        }

        assert_eq!(
            first.source_projection_output_frame(),
            second.source_projection_output_frame()
        );
        assert!(
            (first.source_projection_source_cursor() - second.source_projection_source_cursor())
                .abs()
                < 1.0e-9
        );
    }

    #[test]
    fn realtime_preview_scheduled_source_projection_applies_ratio_change_on_grid() {
        let mut state = RealtimePreviewCallbackState::new(RealtimePreviewStreamConfig::new(
            SampleRate(48_000),
            1,
            96,
        ))
        .expect("callback state config should validate");

        for _ in 0..5 {
            let report = state
                .advance_scheduled_source_projection(96, 1.0)
                .expect("projection should stay within the configured block size");
            assert!(!report.ratio_change_applied);
            assert_eq!(report.start_ratio, 1.0);
            assert_eq!(report.end_ratio, 1.0);
        }

        let changed = state
            .advance_scheduled_source_projection(96, 1.5)
            .expect("projection should stay within the configured block size");

        assert!(changed.ratio_change_applied);
        assert_eq!(changed.output_start_frame, 480);
        assert_eq!(changed.output_end_frame, 576);
        assert_eq!(changed.source_start_frame, 480.0);
        assert_eq!(changed.ratio_change_request_output_frame, 480);
        assert_eq!(changed.ratio_change_output_frame, 512);
        assert_eq!(changed.ratio_change_source_frame, 512.0);
        assert_eq!(changed.ratio_change_alignment_error_frames, 32);
        assert_eq!(changed.start_ratio, 1.0);
        assert_eq!(changed.end_ratio, 1.5);
        assert!((changed.source_end_frame - (512.0 + 64.0 / 1.5)).abs() < 1.0e-9);
        assert_eq!(state.source_projection_active_ratio(), 1.5);
        assert_eq!(state.source_projection_ratio_change_count(), 1);
        assert_eq!(
            state.last_source_projection_ratio_change_output_frame(),
            512
        );
        assert_eq!(
            state.last_source_projection_ratio_change_source_frame(),
            512.0
        );
        assert!(
            state.last_source_projection_ratio_change_alignment_error_frames()
                <= state.ratio_change_alignment_tolerance_frames()
        );

        let next = state
            .advance_scheduled_source_projection(96, 1.5)
            .expect("projection should stay within the configured block size");
        assert!(!next.ratio_change_applied);
        assert_eq!(next.start_ratio, 1.5);
        assert_eq!(next.end_ratio, 1.5);
        assert!((next.source_start_frame - changed.source_end_frame).abs() < 1.0e-9);
        assert_eq!(next.output_start_frame, changed.output_end_frame);
    }

    #[test]
    fn realtime_preview_scheduled_source_projection_is_continuous_across_tempo_ramp() {
        let mut state = RealtimePreviewCallbackState::new(RealtimePreviewStreamConfig::new(
            SampleRate(48_000),
            2,
            96,
        ))
        .expect("callback state config should validate");
        let mut previous_output_end = 0;
        let mut previous_source_end = 0.0;
        let mut changes = Vec::new();

        for block_index in 0..18 {
            let ratio = if block_index < 5 {
                0.75
            } else if block_index < 10 {
                1.0
            } else {
                1.5
            };
            let report = state
                .advance_scheduled_source_projection(96, ratio)
                .expect("projection should stay within the configured block size");

            assert_eq!(report.output_start_frame, previous_output_end);
            assert!((report.source_start_frame - previous_source_end).abs() < 1.0e-9);
            assert!(report.source_end_frame >= report.source_start_frame);
            assert!(report.source_frames_required <= 129);
            if report.ratio_change_applied {
                assert!(
                    report.ratio_change_alignment_error_frames
                        <= state.ratio_change_alignment_tolerance_frames()
                );
                assert!(
                    report.ratio_change_source_frame >= report.source_start_frame
                        && report.ratio_change_source_frame <= report.source_end_frame
                );
                changes.push((
                    report.ratio_change_output_frame,
                    report.ratio_change_source_frame,
                ));
            }

            previous_output_end = report.output_end_frame;
            previous_source_end = report.source_end_frame;
        }

        assert_eq!(changes.len(), 3);
        assert_eq!(changes[0].0, 0);
        assert_eq!(changes[1].0, 512);
        assert_eq!(changes[2].0, 1024);
        assert!(changes.windows(2).all(|pair| pair[0].1 <= pair[1].1));
        assert_eq!(state.source_projection_ratio_change_count(), 3);
        assert_eq!(state.source_projection_current_ratio(), 1.5);
        assert_eq!(state.source_projection_active_ratio(), 1.5);
        assert_eq!(
            state.last_dynamic_source_projection().output_end_frame,
            previous_output_end
        );
    }

    #[test]
    fn realtime_preview_callback_state_validates_stereo_geometry_without_enabling_contract() {
        let mut state = RealtimePreviewCallbackState::new(RealtimePreviewStreamConfig::new(
            SampleRate(48_000),
            2,
            128,
        ))
        .expect("callback state config should validate");
        let input = vec![0.0; 128 * 2];
        let mut output = vec![0.25; 128 * 2];

        assert_eq!(state.config().channel_count, 2);
        assert_eq!(state.scratch_capacity_samples(), 128 * 2);
        assert!(state.input_ring_capacity_samples() >= REALTIME_PREVIEW_WINDOW_SIZE * 2);
        assert_eq!(
            state.input_ring_capacity_samples(),
            state.output_ring_capacity_samples()
        );
        assert_eq!(
            state.output_ring_capacity_samples(),
            state.normalization_ring_capacity_samples()
        );
        assert_eq!(state.window_size(), REALTIME_PREVIEW_WINDOW_SIZE);
        assert_eq!(
            state.spectral_scratch_samples(),
            REALTIME_PREVIEW_WINDOW_SIZE * 2
        );
        assert_eq!(
            state.phase_state_values(),
            (REALTIME_PREVIEW_WINDOW_SIZE / 2 + 1) * 2
        );
        assert!(!state.contract().audio_thread_processing_supported);
        let report = state
            .process(&input, &mut output, 128, 1.25)
            .expect("linked-stereo callback kernel should process");
        assert_eq!(report.input_frames, 128);
        assert_eq!(report.output_frames, 128);
        assert_eq!(report.processed_frames, 128);
        assert_eq!(state.current_ratio(), 1.25);
        assert!(output.iter().all(|sample| *sample == 0.0));

        state.reset();
        assert_eq!(state.current_ratio(), 1.0);
        assert_eq!(state.processed_frames(), 0);
    }

    #[test]
    fn realtime_preview_callback_state_rejects_bad_callback_blocks() {
        let mut state = RealtimePreviewCallbackState::new(RealtimePreviewStreamConfig::new(
            SampleRate(48_000),
            2,
            128,
        ))
        .expect("callback state config should validate");
        let input = vec![0.0; 128 * 2];
        let mut output = vec![0.0; 128 * 2];

        assert_eq!(
            state.process(&input, &mut output, 129, 1.0),
            Err(
                RealtimePreviewCallbackProcessError::FrameCountExceedsConfig {
                    requested: 129,
                    max: 128,
                }
            )
        );
        assert_eq!(
            state.process(&input[..64], &mut output, 128, 1.0),
            Err(RealtimePreviewCallbackProcessError::BufferTooSmall {
                required_samples: 256,
                input_samples: 64,
                output_samples: 256,
            })
        );
    }

    #[test]
    fn realtime_preview_callback_state_processes_mono_stream_without_allocation_contract_claim() {
        let mut state = RealtimePreviewCallbackState::new(RealtimePreviewStreamConfig::new(
            SampleRate(48_000),
            1,
            128,
        ))
        .expect("callback state config should validate");
        let input = sine(440.0, 48_000.0, 128 * 48);
        let mut output = vec![0.0; input.len()];

        for block_index in 0..48 {
            let start = block_index * 128;
            let report = state
                .process(
                    &input[start..start + 128],
                    &mut output[start..start + 128],
                    128,
                    1.0,
                )
                .expect("mono callback kernel should process");
            assert_eq!(report.input_frames, 128);
            assert_eq!(report.output_frames, 128);
            assert_eq!(report.processed_frames, ((block_index + 1) * 128) as u64);
        }

        assert!(!state.contract().audio_thread_processing_supported);
        assert!(rms(&output[1024..]) > 0.05);
        assert!((dominant_frequency_hz(&output[1024..], 48_000.0) - 440.0).abs() < 20.0);
    }

    #[test]
    fn realtime_preview_callback_state_is_deterministic_for_fixed_ratio() {
        let input = sine(330.0, 48_000.0, 128 * 48);
        let mut first = RealtimePreviewCallbackState::new(RealtimePreviewStreamConfig::new(
            SampleRate(48_000),
            1,
            128,
        ))
        .expect("callback state config should validate");
        let mut second = RealtimePreviewCallbackState::new(RealtimePreviewStreamConfig::new(
            SampleRate(48_000),
            1,
            128,
        ))
        .expect("callback state config should validate");
        let mut first_output = vec![0.0; input.len()];
        let mut second_output = vec![0.0; input.len()];

        for block_index in 0..48 {
            let start = block_index * 128;
            first
                .process(
                    &input[start..start + 128],
                    &mut first_output[start..start + 128],
                    128,
                    1.25,
                )
                .expect("first mono callback kernel should process");
            second
                .process(
                    &input[start..start + 128],
                    &mut second_output[start..start + 128],
                    128,
                    1.25,
                )
                .expect("second mono callback kernel should process");
        }

        assert_eq!(first_output, second_output);
        assert!(rms(&first_output[1024..]) > 0.02);
    }

    #[test]
    fn realtime_preview_callback_state_processes_linked_stereo_stream() {
        let left = sine(330.0, 48_000.0, 128 * 64);
        let right = sine(660.0, 48_000.0, 128 * 64);
        let input = left
            .iter()
            .zip(right.iter())
            .flat_map(|(left, right)| [*left, *right])
            .collect::<Vec<_>>();
        let mut first = RealtimePreviewCallbackState::new(RealtimePreviewStreamConfig::new(
            SampleRate(48_000),
            2,
            128,
        ))
        .expect("callback state config should validate");
        let mut second = RealtimePreviewCallbackState::new(RealtimePreviewStreamConfig::new(
            SampleRate(48_000),
            2,
            128,
        ))
        .expect("callback state config should validate");
        let mut first_output = vec![0.0; input.len()];
        let mut second_output = vec![0.0; input.len()];

        for block_index in 0..64 {
            let start = block_index * 128 * 2;
            first
                .process(
                    &input[start..start + 128 * 2],
                    &mut first_output[start..start + 128 * 2],
                    128,
                    1.0,
                )
                .expect("first linked-stereo callback kernel should process");
            second
                .process(
                    &input[start..start + 128 * 2],
                    &mut second_output[start..start + 128 * 2],
                    128,
                    1.0,
                )
                .expect("second linked-stereo callback kernel should process");
        }

        let out_left = first_output
            .chunks_exact(2)
            .map(|frame| frame[0])
            .collect::<Vec<_>>();
        let out_right = first_output
            .chunks_exact(2)
            .map(|frame| frame[1])
            .collect::<Vec<_>>();

        assert_eq!(first_output, second_output);
        assert!(rms(&out_left[1024..]) > 0.05);
        assert!(rms(&out_right[1024..]) > 0.05);
        assert!((dominant_frequency_hz(&out_left[1024..], 48_000.0) - 330.0).abs() < 20.0);
        assert!((dominant_frequency_hz(&out_right[1024..], 48_000.0) - 660.0).abs() < 25.0);
    }

    #[test]
    fn realtime_preview_callback_state_schedules_ratio_changes_on_analysis_grid() {
        let mut state = RealtimePreviewCallbackState::new(RealtimePreviewStreamConfig::new(
            SampleRate(48_000),
            1,
            96,
        ))
        .expect("callback state config should validate");
        let input = sine(440.0, 48_000.0, 96 * 16);
        let mut output = vec![0.0; input.len()];

        for block_index in 0..16 {
            let start = block_index * 96;
            let ratio = if block_index < 5 { 1.0 } else { 1.5 };
            let report = state
                .process(
                    &input[start..start + 96],
                    &mut output[start..start + 96],
                    96,
                    ratio,
                )
                .expect("callback kernel should process dynamic ratio");
            assert_eq!(report.ratio, ratio);
            assert!(
                report.ratio_change_alignment_error_frames
                    <= state.ratio_change_alignment_tolerance_frames()
            );
        }

        assert_eq!(state.current_ratio(), 1.5);
        assert_eq!(state.active_ratio(), 1.5);
        assert_eq!(state.ratio_change_count(), 1);
        assert_eq!(state.last_ratio_change_request_frame(), 480);
        assert_eq!(state.last_ratio_change_applied_frame(), 512);
        assert_eq!(state.last_ratio_change_output_frame(), 1024);
        assert_eq!(state.last_ratio_change_alignment_error_frames(), 32);
        assert!(
            state.last_ratio_change_alignment_error_frames()
                <= state.ratio_change_alignment_tolerance_frames()
        );
    }

    #[test]
    fn realtime_preview_callback_state_bounds_dynamic_ratio_seams_on_tempo_ramp() {
        let input = generate_synthetic_stretch_audio(StretchCorpusFamily::TempoRamp)
            .expect("tempo ramp synthetic case should exist");
        let ratio_change_frames = [input.frame_count() / 3, input.frame_count() * 2 / 3];
        let mut state = RealtimePreviewCallbackState::new(RealtimePreviewStreamConfig::new(
            SampleRate(input.sample_rate_hz),
            input.channels as usize,
            96,
        ))
        .expect("callback state config should validate");
        let mut output = vec![0.0; input.samples.len()];
        let mut seam_frames = Vec::new();
        let mut last_ratio_change_count = 0;

        for block_start in (0..input.frame_count()).step_by(96) {
            let frame_count = (input.frame_count() - block_start).min(96);
            let sample_start = block_start * input.channels as usize;
            let sample_end = sample_start + frame_count * input.channels as usize;
            let ratio = if block_start < ratio_change_frames[0] {
                0.75
            } else if block_start < ratio_change_frames[1] {
                1.0
            } else {
                1.5
            };
            let report = state
                .process(
                    &input.samples[sample_start..sample_end],
                    &mut output[sample_start..sample_end],
                    frame_count,
                    ratio,
                )
                .expect("callback kernel should process tempo ramp");
            if report.ratio_change_count > last_ratio_change_count
                && state.last_ratio_change_request_frame() > 0
            {
                seam_frames.push(report.ratio_change_output_frame as usize);
            }
            last_ratio_change_count = report.ratio_change_count;
        }

        let seam = measure_dynamic_segment_seam_click(&output, input.channels, &seam_frames, 1.0);

        assert_eq!(seam_frames.len(), 2);
        assert_eq!(seam.seam_frames, seam_frames);
        assert!(
            seam.peak_seam_delta < 0.35,
            "peak seam delta {}",
            seam.peak_seam_delta
        );
        assert!(
            seam.click_dbfs < -9.0,
            "seam click dBFS {}",
            seam.click_dbfs
        );
    }

    #[test]
    fn realtime_preview_mono_is_deterministic_and_pitch_preserving() {
        let input = sine(440.0, 48_000.0, 12_000);
        let mut first = RealtimePreviewStretcher::new(1.25);
        let mut second = RealtimePreviewStretcher::new(1.25);

        let first_output = first
            .stretch_mono(&input)
            .expect("render fits the offline output bound");
        let second_output = second
            .stretch_mono(&input)
            .expect("render fits the offline output bound");

        assert_eq!(first.quality(), StretchQuality::RealtimePreview);
        assert_eq!(
            first_output.len(),
            (input.len() as f64 * 1.25).round() as usize
        );
        assert_eq!(first_output, second_output);
        assert!((dominant_frequency_hz(&first_output, 48_000.0) - 440.0).abs() < 20.0);
    }

    #[test]
    fn realtime_preview_linked_stereo_is_deterministic_and_exact_length() {
        let left = sine(330.0, 48_000.0, 16_000);
        let right = sine(660.0, 48_000.0, 16_000);
        let input = left
            .iter()
            .zip(right.iter())
            .flat_map(|(left, right)| [*left, *right])
            .collect::<Vec<_>>();
        let mut first = RealtimePreviewStretcher::new(0.75);
        let mut second = RealtimePreviewStretcher::new(0.75);

        let first_output = first
            .stretch_interleaved_stereo(&input)
            .expect("render fits the offline output bound");
        let second_output = second
            .stretch_interleaved_stereo(&input)
            .expect("render fits the offline output bound");

        assert_eq!(
            first_output.len(),
            (16_000.0_f64 * 0.75).round() as usize * 2
        );
        assert_eq!(first_output, second_output);
    }

    #[test]
    fn realtime_preview_dynamic_ratio_curve_keeps_sample_domain_length() {
        let input = sine(220.0, 48_000.0, 16_000);
        let ratio_curve = [
            StretchRatioPoint {
                timeline_frame: 0,
                ratio: 1.0,
            },
            StretchRatioPoint {
                timeline_frame: 8_000,
                ratio: 1.5,
            },
        ];
        let mut stretcher = RealtimePreviewStretcher::new(1.0);

        let output = stretcher
            .stretch_dynamic_ratio_mono(&input, &ratio_curve)
            .expect("render fits the offline output bound");

        assert_eq!(output.len(), 20_000);
    }

    #[test]
    fn realtime_preview_pitch_shift_preserves_tempo_length_contract() {
        let input = sine(440.0, 48_000.0, 12_000);
        let mut stretcher = RealtimePreviewStretcher::new(1.25);

        let output = stretcher
            .stretch_pitch_mono(&input, SampleRate(48_000), 12.0)
            .expect("render fits the offline output bound");

        assert_eq!(output.len(), 15_000);
        assert!((dominant_frequency_hz(&output, 48_000.0) - 880.0).abs() < 35.0);
    }

    #[test]
    fn realtime_preview_backend_comparison_covers_preview_subset() {
        let report = compare_synthetic_realtime_preview_backends();

        assert_eq!(report.comparisons.len(), 24);
        assert_eq!(
            report.improved_count
                + report.regressed_count
                + report.unchanged_count
                + report.inconclusive_count,
            report.comparisons.len()
        );
        for comparison in &report.comparisons {
            assert_eq!(comparison.baseline_backend, StretchBenchmarkBackend::Draft);
            assert_eq!(
                comparison.candidate_backend,
                StretchBenchmarkBackend::RealtimePreviewPrototype
            );
            assert!(comparison.ratio.is_finite());
            assert!(comparison.ratio > 0.0);
        }
        assert!(report.comparisons.iter().any(|comparison| {
            comparison.case_id == "stretch:tempo_ramp"
                && comparison.metric == StretchMetric::DynamicSegmentSeamClickDbfs
                && comparison.path == StretchBenchmarkPath::DynamicRatio
        }));
        assert!(report.comparisons.iter().any(|comparison| {
            comparison.case_id == "stretch:loop_seam"
                && comparison.metric == StretchMetric::StereoImageDelta
                && comparison.path == StretchBenchmarkPath::LinkedStereo
        }));
        assert!(report.comparisons.iter().any(|comparison| {
            comparison.case_id == "stretch:pitch_shift"
                && comparison.metric == StretchMetric::PitchErrorCents
                && comparison.path == StretchBenchmarkPath::PitchShift
                && comparison.pitch_shift_semitones == Some(12.0)
        }));
    }
}
