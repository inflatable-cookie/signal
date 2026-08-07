use rustfft::num_complex::Complex32;
use rustfft::Fft;
use std::sync::Arc;

use signal_primitives::Sample;

use crate::realtime_preview::RealtimePreviewStreamConfig;

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
    pub(crate) config: RealtimePreviewStreamConfig,
    pub(crate) bins: usize,

    pub(crate) source_ring: Vec<Sample>,
    pub(crate) source_ring_frames: usize,
    pub(crate) source_write_frame: u64,

    pub(crate) output_ring: Vec<Sample>,
    pub(crate) normalization_ring: Vec<Sample>,
    pub(crate) output_ring_frames: usize,
    pub(crate) output_read_frame: u64,

    pub(crate) window: Vec<Sample>,
    pub(crate) omega: Vec<Sample>,
    pub(crate) analysis_buffer: Vec<Complex32>,
    pub(crate) synthesis_spectrum: Vec<Complex32>,
    pub(crate) forward_fft_scratch: Vec<Complex32>,
    pub(crate) inverse_fft_scratch: Vec<Complex32>,
    pub(crate) previous_phase: Vec<Sample>,
    pub(crate) synthesis_phase: Vec<Sample>,
    pub(crate) current_magnitudes: Vec<Sample>,
    pub(crate) current_phases: Vec<Sample>,
    pub(crate) previous_magnitudes: Vec<Sample>,
    pub(crate) current_peak_bins: Vec<usize>,
    pub(crate) current_energy: Vec<f64>,
    pub(crate) previous_energy: Vec<f64>,
    pub(crate) forward: Arc<dyn Fft<Sample>>,
    pub(crate) inverse: Arc<dyn Fft<Sample>>,

    // The single ratio scheduler. Batch 40.2 deleted the output-side duplicate
    // and kept this one: it tracks the source cursor, which is what drives
    // demand. The `g10.027` projection was never wrong, nothing consumed it.
    pub(crate) current_ratio: f64,
    pub(crate) active_ratio: f64,
    pub(crate) pending_ratio: f64,
    pub(crate) pending_request_frame: u64,
    pub(crate) pending_apply_frame: u64,
    pub(crate) pending_change: bool,
    pub(crate) ratio_change_count: u64,
    pub(crate) last_alignment_error_frames: u64,

    pub(crate) next_analysis_frame: u64,
    pub(crate) next_synthesis_frame: f64,
    pub(crate) spectral_frame_index: u64,
    pub(crate) total_source_frames_consumed: u64,
}
