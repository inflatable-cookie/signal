//! RealtimePreview planning types and source-projection helpers.

use std::sync::Arc;

use rustfft::{num_complex::Complex32, Fft};
use signal_primitives::{Sample, SampleRate};

use crate::{
    abs_diff_frames, ceil_frame_to_u64, floor_frame_to_u64, sanitize_ratio, usize_to_u64,
    REALTIME_PREVIEW_ANALYSIS_HOP, REALTIME_PREVIEW_WINDOW_SIZE,
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
    pub(crate) config: RealtimePreviewStreamConfig,
    pub(crate) scratch: Vec<Sample>,
    pub(crate) input_ring: Vec<Sample>,
    pub(crate) output_ring: Vec<Sample>,
    pub(crate) normalization_ring: Vec<f32>,
    pub(crate) window: Vec<f32>,
    pub(crate) omega: Vec<f32>,
    pub(crate) analysis_buffer: Vec<Complex32>,
    pub(crate) synthesis_spectrum: Vec<Complex32>,
    pub(crate) forward_fft_scratch: Vec<Complex32>,
    pub(crate) inverse_fft_scratch: Vec<Complex32>,
    pub(crate) previous_phase: Vec<f32>,
    pub(crate) synthesis_phase: Vec<f32>,
    pub(crate) current_magnitudes: Vec<f32>,
    pub(crate) current_phases: Vec<f32>,
    pub(crate) previous_magnitudes: Vec<f32>,
    pub(crate) current_peak_bins: Vec<usize>,
    pub(crate) forward: Arc<dyn Fft<f32>>,
    pub(crate) inverse: Arc<dyn Fft<f32>>,
    pub(crate) current_ratio: f64,
    pub(crate) active_ratio: f64,
    pub(crate) pending_ratio: f64,
    pub(crate) pending_ratio_request_frame: u64,
    pub(crate) pending_ratio_apply_frame: u64,
    pub(crate) pending_ratio_change: bool,
    pub(crate) last_ratio_change_request_frame: u64,
    pub(crate) last_ratio_change_applied_frame: u64,
    pub(crate) last_ratio_change_output_frame: u64,
    pub(crate) last_ratio_change_alignment_error_frames: usize,
    pub(crate) ratio_change_count: u64,
    pub(crate) input_write_frame: u64,
    pub(crate) output_read_frame: u64,
    pub(crate) source_projection_output_frame: u64,
    pub(crate) source_projection_source_cursor: f64,
    pub(crate) last_source_projection: RealtimePreviewSourceProjectionReport,
    pub(crate) source_projection_current_ratio: f64,
    pub(crate) source_projection_active_ratio: f64,
    pub(crate) source_projection_pending_ratio: f64,
    pub(crate) source_projection_pending_ratio_request_frame: u64,
    pub(crate) source_projection_pending_ratio_apply_frame: u64,
    pub(crate) source_projection_pending_ratio_change: bool,
    pub(crate) last_source_projection_ratio_change_request_frame: u64,
    pub(crate) last_source_projection_ratio_change_output_frame: u64,
    pub(crate) last_source_projection_ratio_change_source_frame: f64,
    pub(crate) last_source_projection_ratio_change_alignment_error_frames: usize,
    pub(crate) source_projection_ratio_change_count: u64,
    pub(crate) last_dynamic_source_projection: RealtimePreviewDynamicSourceProjectionReport,
    pub(crate) next_analysis_frame: u64,
    pub(crate) next_synthesis_frame: f64,
    pub(crate) processed_frames: u64,
    pub(crate) spectral_frame_index: u64,
    pub(crate) current_energy: Vec<f64>,
    pub(crate) previous_energy: Vec<f64>,
}

impl std::fmt::Debug for RealtimePreviewCallbackState {
    /// Reports configuration and ratio-scheduler progress. The FFT plans are
    /// foreign trait objects and the remaining fields are preallocated
    /// spectral working buffers.
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("RealtimePreviewCallbackState")
            .field("config", &self.config)
            .field("current_ratio", &self.current_ratio)
            .field("active_ratio", &self.active_ratio)
            .field("ratio_change_count", &self.ratio_change_count)
            .field("processed_frames", &self.processed_frames)
            .finish_non_exhaustive()
    }
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

pub(crate) fn build_realtime_preview_source_projection_report(
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
pub(crate) struct DynamicSourceProjectionRatios {
    pub(crate) start_ratio: f64,
    pub(crate) end_ratio: f64,
    pub(crate) ratio_change_applied: bool,
    pub(crate) ratio_change_count: u64,
    pub(crate) ratio_change_request_output_frame: u64,
    pub(crate) ratio_change_output_frame: u64,
    pub(crate) ratio_change_source_frame: f64,
    pub(crate) ratio_change_alignment_error_frames: usize,
}

impl DynamicSourceProjectionRatios {
    /// Unity ratio, no change ever scheduled — the reset and construction state.
    pub(crate) const fn idle() -> Self {
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

pub(crate) fn build_realtime_preview_dynamic_source_projection_report(
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
