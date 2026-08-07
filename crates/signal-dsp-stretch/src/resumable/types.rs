//! Shared types for the resumable offline stretch renderer.

use rustfft::num_complex::Complex32;
use signal_primitives::SampleRate;

use crate::StretchRatioPoint;

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

pub(crate) struct ChannelState {
    pub(crate) previous_phase: Vec<f32>,
    pub(crate) synthesis_phase: Vec<f32>,
    pub(crate) previous_magnitudes: Vec<f32>,
    pub(crate) previous_energy: f64,
    pub(crate) current_energy_scratch: f64,
    pub(crate) analysis: Vec<Complex32>,
    pub(crate) spectrum: Vec<Complex32>,
    pub(crate) current_magnitudes: Vec<f32>,
    pub(crate) current_phases: Vec<f32>,
    pub(crate) peaks: Vec<usize>,
    pub(crate) output_ring: Vec<f32>,
    pub(crate) normalization_ring: Vec<f32>,
}
