//! Time-stretching backends for the Signal workspace.
//!
//! The crate defines the abstract [`TimeStretcher`] contract — stretch audio
//! in time without shifting pitch — and ships two offline backends plus one
//! preview prototype this round:
//! [`PhaseVocoderStretcher`], a dependency-light draft-quality phase vocoder,
//! [`RealtimePreviewStretcher`], a lower-latency pitch-preserving preview
//! prototype, and [`OfflineHighQualityStretcher`], Signal's offline-quality
//! foundation for corpus-gated export, freeze, and cache artifacts.
//!
//! ## Signal-owned backend tiers
//!
//! Signal owns three execution tiers:
//!
//! - [`StretchBackendTier::Repitch`]: render-plane rate conversion, pitch
//!   changes with tempo, realtime-safe today.
//! - [`StretchBackendTier::RealtimePreview`]: bounded-latency pitch-preserving
//!   preview stretch, prototype.
//! - [`StretchBackendTier::OfflineHighQuality`]: deterministic
//!   export/cache/freeze stretch, implemented and promotion-gated as the
//!   quality reference tier.
//!
//! The current [`PhaseVocoderStretcher`] remains [`StretchQuality::Draft`]: a
//! plain Hann-windowed phase vocoder with NO phase locking and NO transient
//! preservation. [`OfflineHighQualityStretcher`] uses identity phase locking
//! and transient phase resets as the first clean-room Signal-owned foundation.
//! Product-facing use remains blocked unless an accepted
//! [`StretchPromotionReceipt`] is attached to the render/export/freeze
//! artifact plan. Rubber Band-class quality is the target for the planned
//! Signal-native tiers, but Rubber Band source is not an implementation input.
//!
//! ## Real-time posture
//!
//! This backend is OFFLINE-ONLY: it allocates its analysis/synthesis buffers
//! per call and processes whole buffers. It must never run on the audio
//! thread. Consumers that need stretched playback precompute the stretched
//! buffer control-side (anticipative posture) and hand the render plane an
//! ordinary sample buffer. [`RealtimePreviewStreamingContract`] names the
//! callback-safe boundary that must be satisfied before direct render-plane
//! integration is allowed.

#![warn(missing_docs)]

mod artifact_plan;
mod benchmark;
mod cache_identity;
mod corpus_report;
mod phase_vocoder;
mod promotion;

pub use artifact_plan::{
    plan_offline_stretch_chunks, StretchOfflineChunk, StretchOfflineChunkConfig,
    StretchOfflineChunkPlan, DEFAULT_OFFLINE_STRETCH_CHUNK_OVERLAP_FRAMES,
    DEFAULT_OFFLINE_STRETCH_CHUNK_SOURCE_FRAMES,
};
pub use benchmark::{
    assess_stretch_metrics, compare_sustained_material_coherence,
    compare_synthetic_realtime_preview_backends, compare_synthetic_stretch_backends,
    detect_stretch_transients, detect_stretch_transients_with_policy,
    format_stretch_acceptance_report, format_stretch_quality_priority_report,
    format_synthetic_stretch_comparison_report, generate_synthetic_stretch_audio,
    measure_draft_loop_boundary_click, measure_draft_stereo_image_delta,
    measure_draft_transient_smear, measure_dynamic_segment_seam_click, measure_loop_boundary_click,
    measure_pitch_shift_error_cents, measure_stereo_image_delta,
    measure_transient_reset_loop_boundary_click, measure_transient_reset_stereo_image_delta,
    measure_transient_reset_transient_smear, measure_transient_smear,
    measure_transient_smear_with_output_recovery_policy, measure_transient_smear_with_policies,
    measure_transient_smear_with_policy, output_length_drift_samples,
    prioritize_stretch_quality_work, synthetic_stretch_corpus_cases, StretchAcceptanceReport,
    StretchAcceptanceSeverity, StretchAcceptanceStatus, StretchBenchmarkBackend,
    StretchBenchmarkComparisonOutcome, StretchBenchmarkPath, StretchCoherenceComparison,
    StretchCorpusAssetRequirement, StretchCorpusCase, StretchCorpusFamily, StretchCorpusManifest,
    StretchCorpusManifestEntry, StretchCorpusMissingAssetBehavior, StretchCorpusSource,
    StretchCorpusSourcePolicy, StretchDynamicSegmentSeamMeasurement,
    StretchLoopBoundaryMeasurement, StretchMetric, StretchMetricAssessment, StretchMetricLimit,
    StretchMetricValue, StretchPitchShiftMeasurement, StretchQualityPriority,
    StretchQualityWorkArea, StretchStereoImageMeasurement, StretchSyntheticAudio,
    StretchSyntheticBenchmarkComparison, StretchSyntheticBenchmarkComparisonReport,
    StretchTransientDetectorPolicy, StretchTransientEvent, StretchTransientSmearMeasurement,
    STRETCH_BENCHMARK_CORPUS, STRETCH_CORPUS_MANIFEST, STRETCH_CORPUS_MANIFEST_ENTRIES,
    STRETCH_CORPUS_SOURCE_POLICY,
};
pub use cache_identity::{
    StretchCacheIdentity, StretchCacheIdentityError, StretchCacheIdentityInput,
    StretchChannelLayout, StretchPitchPoint, StretchRatioPoint, StretchWarpMarker,
    SIGNAL_STRETCH_ENGINE_VERSION, STRETCH_CACHE_IDENTITY_SCHEMA_VERSION,
};
pub use corpus_report::{
    build_stretch_corpus_comparison_report, build_stretch_corpus_comparison_report_with_external,
    build_stretch_corpus_comparison_report_with_sources, format_stretch_corpus_comparison_report,
    StretchCorpusComparisonReport, StretchCorpusListeningNoteSlot, StretchCorpusListeningSource,
    StretchCorpusListeningSourceRecord, StretchCorpusSkippedAsset,
    StretchExternalBenchmarkComparison, StretchExternalBenchmarkRender,
};
pub use promotion::{
    current_synthetic_offline_high_quality_promotion_receipt, StretchPromotionReceipt,
    StretchPromotionStatus, StretchSyntheticPromotionPolicy,
};

use phase_vocoder::{
    compression_transient_anchor_phase_vocoder, magnitude_slew_phase_vocoder,
    phase_locked_phase_vocoder, phase_vocoder, stability_adaptive_phase_vocoder,
    tracked_peak_region_phase_vocoder, transient_reset_phase_vocoder,
    transient_reset_phase_vocoder_linked_stereo,
};
use rustfft::{num_complex::Complex32, Fft, FftPlanner};
use signal_dsp_resample::{resample_mono, ResampleConfig, ResampleQuality};
use signal_primitives::{Sample, SampleRate};
use std::sync::Arc;

const DYNAMIC_RATIO_SEAM_SMOOTH_FRAMES: usize = 256;

/// Quality tier of a stretch backend (memo 013 vocabulary). One tier exists
/// today; real-time and offline production tiers land with the library
/// evaluation (P-TS-001).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StretchQuality {
    /// Draft-quality phase vocoder: pitch-preserving, but transients smear
    /// and no formant handling. Offline use only.
    Draft,
    /// Bounded-latency preview quality. Implemented as a control-side
    /// prototype; direct audio-thread processing is still unsupported.
    RealtimePreview,
    /// Highest-quality deterministic offline/export quality. Product-facing
    /// use is still promotion-gated per artifact.
    OfflineHighQuality,
}

/// Signal-owned stretch execution tier.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StretchBackendTier {
    /// Existing render-plane varispeed path. Tempo changes also shift pitch.
    Repitch,
    /// Prototype bounded-latency preview tier for live audition and playback.
    RealtimePreview,
    /// Deterministic high-quality tier for exports, freeze, and cached
    /// post-warp artifacts.
    OfflineHighQuality,
}

/// Implementation status for one tier in the Signal-native stretch program.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StretchBackendStatus {
    /// The tier is implemented in Signal today.
    Implemented,
    /// The tier has an implemented DSP path, but it has not yet satisfied the
    /// full product-facing backend contract or corpus promotion gate.
    Prototype,
    /// The tier is designed but not implemented.
    Planned,
}

/// Clean-room architecture contract for one Signal-owned tier.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StretchBackendPlan {
    /// Signal-owned execution tier.
    pub tier: StretchBackendTier,
    /// Current implementation status.
    pub status: StretchBackendStatus,
    /// Whether tempo and pitch can be controlled independently.
    pub independent_tempo_and_pitch: bool,
    /// Whether stretch ratio may change within one render.
    pub dynamic_ratio: bool,
    /// Whether transient preservation is part of the tier contract.
    pub transient_preservation: bool,
    /// Whether stereo or multichannel vertical coherence is part of the tier
    /// contract.
    pub vertical_phase_coherence: bool,
    /// Whether the tier promises sample-accurate or near-sample-accurate
    /// timeline alignment.
    pub alignment_promised: bool,
    /// Whether processing may run on the realtime audio thread.
    pub audio_thread_safe: bool,
    /// Whether rendered output is deterministic enough for cache identity,
    /// export reuse, and regression comparison.
    pub deterministic_output: bool,
}

/// Signal-owned tier plan. This is a code-level mirror of the roadmap
/// contract so callers can gate behavior without vendor-specific names.
pub const SIGNAL_STRETCH_BACKEND_PLAN: [StretchBackendPlan; 3] = [
    StretchBackendPlan {
        tier: StretchBackendTier::Repitch,
        status: StretchBackendStatus::Implemented,
        independent_tempo_and_pitch: false,
        dynamic_ratio: true,
        transient_preservation: true,
        vertical_phase_coherence: true,
        alignment_promised: true,
        audio_thread_safe: true,
        deterministic_output: true,
    },
    StretchBackendPlan {
        tier: StretchBackendTier::RealtimePreview,
        status: StretchBackendStatus::Prototype,
        independent_tempo_and_pitch: true,
        dynamic_ratio: true,
        transient_preservation: true,
        vertical_phase_coherence: true,
        alignment_promised: true,
        audio_thread_safe: false,
        deterministic_output: true,
    },
    StretchBackendPlan {
        tier: StretchBackendTier::OfflineHighQuality,
        status: StretchBackendStatus::Implemented,
        independent_tempo_and_pitch: true,
        dynamic_ratio: true,
        transient_preservation: true,
        vertical_phase_coherence: true,
        alignment_promised: true,
        audio_thread_safe: false,
        deterministic_output: true,
    },
];

/// Returns the Signal-owned plan for `tier`.
pub fn stretch_backend_plan(tier: StretchBackendTier) -> StretchBackendPlan {
    SIGNAL_STRETCH_BACKEND_PLAN
        .iter()
        .copied()
        .find(|plan| plan.tier == tier)
        .expect("all StretchBackendTier variants are represented")
}
/// Abstract time-stretcher contract (memo 013): stretch audio in time while
/// preserving pitch. `ratio` is the OUTPUT/INPUT duration factor — 2.0 makes
/// the audio twice as long (half speed), 0.5 twice as fast.
///
/// v1 scope is offline/control-side whole-buffer processing; the direct
/// streaming/RT surface (bounded latency, PDC reporting, variable ratio
/// mid-stream) extends this trait when a production callback-safe backend
/// lands.
pub trait TimeStretcher {
    /// Quality tier this backend provides — consumers must be able to make
    /// an honest offline/RT routing decision from this.
    fn quality(&self) -> StretchQuality;

    /// Current output/input duration ratio.
    fn ratio(&self) -> f64;

    /// Set the output/input duration ratio. Non-finite or non-positive
    /// values are clamped to 1.0 (identity).
    fn set_ratio(&mut self, ratio: f64);

    /// Stretch one mono buffer offline. Output length contract:
    /// `round(input.len() as f64 * ratio)` frames (identity ratio returns the
    /// input verbatim).
    fn stretch_mono(&mut self, input: &[Sample]) -> Vec<Sample>;
}

/// Draft-quality phase vocoder time-stretcher.
///
/// Classic STFT phase vocoder: fixed analysis hop, synthesis hop scaled by
/// the stretch ratio, per-bin phase propagation from the measured
/// instantaneous frequency, Hann analysis and synthesis windows with
/// window-power overlap-add normalization. Inputs shorter than one analysis
/// window fall back to linear time-domain interpolation (the honest cheap
/// path — a single window carries no phase-propagation benefit).
pub struct PhaseVocoderStretcher {
    ratio: f64,
    window_size: usize,
    analysis_hop: usize,
}

/// Lower-latency pitch-preserving preview stretcher.
///
/// This is a control-side prototype, not a render-callback object. It uses a
/// shorter STFT window than [`OfflineHighQualityStretcher`] so edits can be
/// previewed with lower algorithmic latency, while keeping the same clean-room
/// transient-reset and linked-stereo foundation.
pub struct RealtimePreviewStretcher {
    ratio: f64,
    window_size: usize,
    analysis_hop: usize,
}

/// Offline high-quality time-stretcher.
///
/// This is the first Signal-owned offline-quality DSP path: a deterministic
/// whole-buffer STFT stretcher with identity phase locking and transient phase
/// resets. It is exposed as [`StretchQuality::OfflineHighQuality`] for
/// export/cache/freeze artifact planning, while product-facing consumption is
/// gated by accepted promotion evidence on each artifact plan.
pub struct OfflineHighQualityStretcher {
    ratio: f64,
    window_size: usize,
    analysis_hop: usize,
    path: OfflineHighQualityPath,
}

/// Offline high-quality renderer path.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OfflineHighQualityPath {
    /// Current production-candidate OfflineHighQuality path.
    Default,
    /// Compression-only selector that switches to a shorter STFT window when
    /// the current path misses transients or exceeds the current-smear gate.
    CompressionShortWindowSelector,
    /// Expansion-only selector that switches to a shorter STFT window when
    /// the current path misses transients or regresses versus the draft
    /// transient-smear baseline.
    ExpansionShortWindowSelector,
}

/// Default STFT window: 2048 samples (~43 ms at 48 kHz).
pub const DEFAULT_WINDOW_SIZE: usize = 2_048;
/// Default analysis hop: window / 4 (75% overlap).
pub const DEFAULT_ANALYSIS_HOP: usize = DEFAULT_WINDOW_SIZE / 4;
/// Short-window selector STFT size for compression material.
pub const COMPRESSION_SHORT_WINDOW_SELECTOR_WINDOW_SIZE: usize = 1_024;
/// Short-window selector analysis hop.
pub const COMPRESSION_SHORT_WINDOW_SELECTOR_ANALYSIS_HOP: usize =
    COMPRESSION_SHORT_WINDOW_SELECTOR_WINDOW_SIZE / 4;
/// RealtimePreview prototype STFT size.
pub const REALTIME_PREVIEW_WINDOW_SIZE: usize = 512;
/// RealtimePreview prototype analysis hop.
pub const REALTIME_PREVIEW_ANALYSIS_HOP: usize = REALTIME_PREVIEW_WINDOW_SIZE / 4;
/// Short-window selector gate: current path must miss at least this many
/// source transients before the selector may switch.
pub const COMPRESSION_SHORT_WINDOW_SELECTOR_MIN_CURRENT_MISSES: usize = 1;
/// Short-window selector gate: current path must exceed this transient-smear
/// value before the selector may switch.
pub const COMPRESSION_SHORT_WINDOW_SELECTOR_MIN_CURRENT_SMEAR_FRAMES: f64 = 64.0;
/// Short-window selector STFT size for expansion material.
pub const EXPANSION_SHORT_WINDOW_SELECTOR_WINDOW_SIZE: usize =
    COMPRESSION_SHORT_WINDOW_SELECTOR_WINDOW_SIZE;
/// Short-window selector analysis hop for expansion material.
pub const EXPANSION_SHORT_WINDOW_SELECTOR_ANALYSIS_HOP: usize =
    COMPRESSION_SHORT_WINDOW_SELECTOR_ANALYSIS_HOP;
/// Expansion short-window selector gate: current path must miss at least this
/// many source transients before the selector may switch.
pub const EXPANSION_SHORT_WINDOW_SELECTOR_MIN_CURRENT_MISSES: usize =
    COMPRESSION_SHORT_WINDOW_SELECTOR_MIN_CURRENT_MISSES;
/// Report-only sustained-coherence review STFT size.
pub const SUSTAINED_COHERENCE_REVIEW_WINDOW_SIZE: usize = DEFAULT_WINDOW_SIZE * 2;
/// Report-only sustained-coherence review analysis hop.
pub const SUSTAINED_COHERENCE_REVIEW_ANALYSIS_HOP: usize =
    SUSTAINED_COHERENCE_REVIEW_WINDOW_SIZE / 4;
/// Report-only blend weight for the sustained-coherence review path.
pub const SUSTAINED_COHERENCE_BLEND_REVIEW_WEIGHT: f32 = 0.5;
/// Report-only sustained-coherence envelope-match review window.
pub const SUSTAINED_COHERENCE_ENVELOPE_REVIEW_WINDOW_SIZE: usize = DEFAULT_WINDOW_SIZE * 2;
/// Report-only sustained-coherence envelope-match review hop.
pub const SUSTAINED_COHERENCE_ENVELOPE_REVIEW_HOP_SIZE: usize =
    SUSTAINED_COHERENCE_ENVELOPE_REVIEW_WINDOW_SIZE / 2;

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

/// Unsupported RealtimePreview routing mode.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RealtimePreviewUnsupportedMode {
    /// The current prototype allocates scratch buffers and processes whole
    /// preview buffers, so it must remain outside the audio callback.
    AudioThreadProcessing,
    /// The requested channel layout is not part of the current linked preview
    /// contract.
    ChannelLayout,
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
    /// The callback-facing state exists, but streaming DSP is not implemented.
    CallbackProcessingUnsupported,
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
        audio_thread_processing_supported: false,
        unsupported_mode: Some(RealtimePreviewUnsupportedMode::AudioThreadProcessing),
        config,
    })
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

    /// Whether FFT plans are already constructed for the callback state.
    pub fn fft_plans_ready(&self) -> bool {
        Arc::strong_count(&self.forward) >= 1 && Arc::strong_count(&self.inverse) >= 1
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

impl PhaseVocoderStretcher {
    /// Stretcher with the default window/hop configuration.
    pub fn new(ratio: f64) -> Self {
        Self::with_window(ratio, DEFAULT_WINDOW_SIZE, DEFAULT_ANALYSIS_HOP)
    }

    /// Stretcher with an explicit window size and analysis hop. The window
    /// is clamped to a power of two ≥ 64; the hop to `1..=window/2`.
    pub fn with_window(ratio: f64, window_size: usize, analysis_hop: usize) -> Self {
        let window_size = window_size.next_power_of_two().max(64);
        let analysis_hop = analysis_hop.clamp(1, window_size / 2);
        let mut stretcher = Self {
            ratio: 1.0,
            window_size,
            analysis_hop,
        };
        stretcher.set_ratio(ratio);
        stretcher
    }
}

impl TimeStretcher for PhaseVocoderStretcher {
    fn quality(&self) -> StretchQuality {
        StretchQuality::Draft
    }

    fn ratio(&self) -> f64 {
        self.ratio
    }

    fn set_ratio(&mut self, ratio: f64) {
        self.ratio = sanitize_ratio(ratio);
    }

    fn stretch_mono(&mut self, input: &[Sample]) -> Vec<Sample> {
        stretch_mono_with_engine(
            input,
            self.ratio,
            self.window_size,
            self.analysis_hop,
            phase_vocoder,
        )
    }
}

impl RealtimePreviewStretcher {
    /// Stretcher with the preview window/hop configuration.
    pub fn new(ratio: f64) -> Self {
        Self::with_window(
            ratio,
            REALTIME_PREVIEW_WINDOW_SIZE,
            REALTIME_PREVIEW_ANALYSIS_HOP,
        )
    }

    /// Stretcher with an explicit window size and analysis hop. The window
    /// is clamped to a power of two ≥ 64; the hop to `1..=window/2`.
    pub fn with_window(ratio: f64, window_size: usize, analysis_hop: usize) -> Self {
        let window_size = window_size.next_power_of_two().max(64);
        let analysis_hop = analysis_hop.clamp(1, window_size / 2);
        let mut stretcher = Self {
            ratio: 1.0,
            window_size,
            analysis_hop,
        };
        stretcher.set_ratio(ratio);
        stretcher
    }

    /// Build the stream contract for this preview stretcher.
    pub fn streaming_contract(
        &self,
        sample_rate: SampleRate,
        channel_count: usize,
        max_block_frames: usize,
    ) -> Result<RealtimePreviewStreamingContract, RealtimePreviewPlanError> {
        plan_realtime_preview_stream(RealtimePreviewStreamConfig {
            sample_rate,
            channel_count,
            max_block_frames,
            window_size: self.window_size,
            analysis_hop: self.analysis_hop,
        })
    }

    /// Stretch an interleaved stereo buffer through the linked preview path.
    ///
    /// A trailing odd sample is ignored. This allocates and processes a whole
    /// control-side preview buffer, so callers must not use it on the audio
    /// callback.
    pub fn stretch_interleaved_stereo(&mut self, frames: &[Sample]) -> Vec<Sample> {
        let frame_count = frames.len() / 2;
        let target_frames = (frame_count as f64 * self.ratio).round() as usize;
        if frame_count == 0 || target_frames == 0 {
            return Vec::new();
        }
        let even_frames = &frames[..frame_count * 2];
        if (self.ratio - 1.0).abs() < 1.0e-9 {
            return even_frames.to_vec();
        }
        if frame_count < self.window_size {
            return linear_time_scale_interleaved_stereo(even_frames, target_frames);
        }
        transient_reset_phase_vocoder_linked_stereo(
            even_frames,
            target_frames,
            self.ratio,
            self.window_size,
            self.analysis_hop,
        )
    }

    /// Apply independent pitch shift and tempo stretch to one mono preview
    /// buffer.
    pub fn stretch_pitch_mono(
        &mut self,
        input: &[Sample],
        sample_rate: SampleRate,
        pitch_shift_semitones: f64,
    ) -> Vec<Sample> {
        let target_len = (input.len() as f64 * self.ratio).round() as usize;
        if input.is_empty() || target_len == 0 {
            return Vec::new();
        }
        if pitch_shift_semitones.abs() < 1.0e-9 || sample_rate.0 == 0 {
            return self.stretch_mono(input);
        }

        let pitched = pitch_shift_mono_to_nominal_rate(input, sample_rate, pitch_shift_semitones);
        stretch_to_exact_mono(
            &pitched,
            target_len,
            self.window_size,
            self.analysis_hop,
            transient_reset_phase_vocoder,
        )
    }

    /// Apply independent pitch shift and tempo stretch to interleaved stereo
    /// preview material.
    pub fn stretch_pitch_interleaved_stereo(
        &mut self,
        frames: &[Sample],
        sample_rate: SampleRate,
        pitch_shift_semitones: f64,
    ) -> Vec<Sample> {
        let frame_count = frames.len() / 2;
        let target_frames = (frame_count as f64 * self.ratio).round() as usize;
        if frame_count == 0 || target_frames == 0 {
            return Vec::new();
        }
        let even_frames = &frames[..frame_count * 2];
        if pitch_shift_semitones.abs() < 1.0e-9 || sample_rate.0 == 0 {
            return self.stretch_interleaved_stereo(even_frames);
        }

        let pitched = pitch_shift_interleaved_stereo_to_nominal_rate(
            even_frames,
            sample_rate,
            pitch_shift_semitones,
        );
        stretch_to_exact_linked_stereo(&pitched, target_frames, self.window_size, self.analysis_hop)
    }

    /// Stretch one mono buffer with a stepwise dynamic ratio curve.
    pub fn stretch_dynamic_ratio_mono(
        &mut self,
        input: &[Sample],
        ratio_curve: &[StretchRatioPoint],
    ) -> Vec<Sample> {
        stretch_dynamic_ratio_mono_with_engine(
            input,
            ratio_curve,
            self.ratio,
            self.window_size,
            self.analysis_hop,
            transient_reset_phase_vocoder,
        )
    }

    /// Stretch an interleaved stereo buffer with a stepwise dynamic ratio
    /// curve through the linked preview path.
    pub fn stretch_dynamic_ratio_interleaved_stereo(
        &mut self,
        frames: &[Sample],
        ratio_curve: &[StretchRatioPoint],
    ) -> Vec<Sample> {
        stretch_dynamic_ratio_linked_stereo_with_engine(
            frames,
            ratio_curve,
            self.ratio,
            self.window_size,
            self.analysis_hop,
        )
    }
}

impl TimeStretcher for RealtimePreviewStretcher {
    fn quality(&self) -> StretchQuality {
        StretchQuality::RealtimePreview
    }

    fn ratio(&self) -> f64 {
        self.ratio
    }

    fn set_ratio(&mut self, ratio: f64) {
        self.ratio = sanitize_ratio(ratio);
    }

    fn stretch_mono(&mut self, input: &[Sample]) -> Vec<Sample> {
        stretch_mono_with_engine(
            input,
            self.ratio,
            self.window_size,
            self.analysis_hop,
            transient_reset_phase_vocoder,
        )
    }
}

impl OfflineHighQualityStretcher {
    /// Stretcher with the default window/hop configuration.
    pub fn new(ratio: f64) -> Self {
        Self::with_window(ratio, DEFAULT_WINDOW_SIZE, DEFAULT_ANALYSIS_HOP)
    }

    /// Stretcher with an explicit window size and analysis hop. The window
    /// is clamped to a power of two ≥ 64; the hop to `1..=window/2`.
    pub fn with_window(ratio: f64, window_size: usize, analysis_hop: usize) -> Self {
        let window_size = window_size.next_power_of_two().max(64);
        let analysis_hop = analysis_hop.clamp(1, window_size / 2);
        let mut stretcher = Self {
            ratio: 1.0,
            window_size,
            analysis_hop,
            path: OfflineHighQualityPath::Default,
        };
        stretcher.set_ratio(ratio);
        stretcher
    }

    /// Stretcher with the default window/hop and an explicit offline path.
    pub fn with_path(ratio: f64, path: OfflineHighQualityPath) -> Self {
        let mut stretcher = Self::new(ratio);
        stretcher.path = path;
        stretcher
    }

    /// Current offline high-quality renderer path.
    pub fn path(&self) -> OfflineHighQualityPath {
        self.path
    }

    /// Set the offline high-quality renderer path.
    pub fn set_path(&mut self, path: OfflineHighQualityPath) {
        self.path = path;
    }

    /// Stretch an interleaved stereo buffer through the linked
    /// OfflineHighQuality prototype path.
    ///
    /// Unlike [`stretch_interleaved_stereo`], this path does not process left
    /// and right independently. It uses a mid/side linked analysis surface so
    /// stereo image metrics can be measured against a candidate that preserves
    /// channel relationships more directly. A trailing odd sample is ignored.
    pub fn stretch_interleaved_stereo(&mut self, frames: &[Sample]) -> Vec<Sample> {
        let frame_count = frames.len() / 2;
        let target_frames = (frame_count as f64 * self.ratio).round() as usize;
        if frame_count == 0 || target_frames == 0 {
            return Vec::new();
        }
        let even_frames = &frames[..frame_count * 2];
        if (self.ratio - 1.0).abs() < 1.0e-9 {
            return even_frames.to_vec();
        }
        if frame_count < self.window_size {
            return linear_time_scale_interleaved_stereo(even_frames, target_frames);
        }

        let default_output = transient_reset_phase_vocoder_linked_stereo(
            even_frames,
            target_frames,
            self.ratio,
            self.window_size,
            self.analysis_hop,
        );
        let selected_short_window = match self.path {
            OfflineHighQualityPath::Default => false,
            OfflineHighQualityPath::CompressionShortWindowSelector => {
                should_select_compression_short_window_interleaved(
                    even_frames,
                    &default_output,
                    self.ratio,
                )
            }
            OfflineHighQualityPath::ExpansionShortWindowSelector => {
                should_select_expansion_short_window_interleaved(
                    even_frames,
                    &default_output,
                    self.ratio,
                )
            }
        };
        if selected_short_window {
            transient_reset_phase_vocoder_linked_stereo(
                even_frames,
                target_frames,
                self.ratio,
                short_window_size_for_path(self.path),
                short_window_analysis_hop_for_path(self.path),
            )
        } else {
            default_output
        }
    }

    /// Apply independent pitch shift and tempo stretch to one mono buffer.
    ///
    /// `pitch_shift_semitones` changes pitch without changing the final
    /// duration target. The current [`Self::ratio`] remains the tempo/output
    /// duration contract, so output length is
    /// `round(input.len() as f64 * self.ratio)`.
    pub fn stretch_pitch_mono(
        &mut self,
        input: &[Sample],
        sample_rate: SampleRate,
        pitch_shift_semitones: f64,
    ) -> Vec<Sample> {
        let target_len = (input.len() as f64 * self.ratio).round() as usize;
        if input.is_empty() || target_len == 0 {
            return Vec::new();
        }
        if pitch_shift_semitones.abs() < 1.0e-9 || sample_rate.0 == 0 {
            return self.stretch_mono(input);
        }

        let pitched = pitch_shift_mono_to_nominal_rate(input, sample_rate, pitch_shift_semitones);
        stretch_to_exact_mono(
            &pitched,
            target_len,
            self.window_size,
            self.analysis_hop,
            transient_reset_phase_vocoder,
        )
    }

    /// Apply independent pitch shift and tempo stretch to interleaved stereo.
    ///
    /// Pitch shift is composed through linked mid/side resampling, then the
    /// linked OfflineHighQuality stereo stretcher restores the requested tempo
    /// duration. A trailing odd sample is ignored.
    pub fn stretch_pitch_interleaved_stereo(
        &mut self,
        frames: &[Sample],
        sample_rate: SampleRate,
        pitch_shift_semitones: f64,
    ) -> Vec<Sample> {
        let frame_count = frames.len() / 2;
        let target_frames = (frame_count as f64 * self.ratio).round() as usize;
        if frame_count == 0 || target_frames == 0 {
            return Vec::new();
        }
        let even_frames = &frames[..frame_count * 2];
        if pitch_shift_semitones.abs() < 1.0e-9 || sample_rate.0 == 0 {
            return self.stretch_interleaved_stereo(even_frames);
        }

        let pitched = pitch_shift_interleaved_stereo_to_nominal_rate(
            even_frames,
            sample_rate,
            pitch_shift_semitones,
        );
        stretch_to_exact_linked_stereo(&pitched, target_frames, self.window_size, self.analysis_hop)
    }

    /// Apply one static pitch shift while following a stepwise dynamic ratio
    /// curve over interleaved stereo.
    ///
    /// Segment boundaries use the same source-frame vocabulary as
    /// [`Self::stretch_dynamic_ratio_interleaved_stereo`]. Pitch shifting is
    /// applied per source segment before that segment is stretched to its
    /// target duration, so cache identity marker frames stay anchored to the
    /// original decoded source.
    pub fn stretch_dynamic_ratio_pitch_interleaved_stereo(
        &mut self,
        frames: &[Sample],
        ratio_curve: &[StretchRatioPoint],
        sample_rate: SampleRate,
        pitch_shift_semitones: f64,
    ) -> Vec<Sample> {
        if pitch_shift_semitones.abs() < 1.0e-9 || sample_rate.0 == 0 {
            return self.stretch_dynamic_ratio_interleaved_stereo(frames, ratio_curve);
        }

        let frame_count = frames.len() / 2;
        let even_frames = &frames[..frame_count * 2];
        let segments = dynamic_ratio_segments(frame_count, ratio_curve, sanitize_ratio(self.ratio));
        let boundaries = dynamic_ratio_segment_boundaries(&segments);
        let target_frames: usize = segments.iter().map(|segment| segment.target_frames).sum();
        let mut output = Vec::with_capacity(target_frames * 2);
        for segment in segments {
            let start = segment.start_frame * 2;
            let end = segment.end_frame * 2;
            let pitched = pitch_shift_interleaved_stereo_to_nominal_rate(
                &even_frames[start..end],
                sample_rate,
                pitch_shift_semitones,
            );
            let rendered = stretch_to_exact_linked_stereo(
                &pitched,
                segment.target_frames,
                self.window_size,
                self.analysis_hop,
            );
            output.extend(rendered);
        }
        smooth_dynamic_segment_boundaries_interleaved(
            &mut output,
            2,
            &boundaries,
            DYNAMIC_RATIO_SEAM_SMOOTH_FRAMES,
        );
        output
    }

    /// Stretch one mono buffer with a stepwise dynamic ratio curve.
    ///
    /// `ratio_curve` uses the same sample-frame vocabulary as cache identity:
    /// each [`StretchRatioPoint::timeline_frame`] is interpreted as a
    /// source-frame offset in this buffer where the point's ratio becomes
    /// active. Invalid points are ignored. Gaps before the first valid point
    /// use the stretcher's current [`Self::ratio`].
    pub fn stretch_dynamic_ratio_mono(
        &mut self,
        input: &[Sample],
        ratio_curve: &[StretchRatioPoint],
    ) -> Vec<Sample> {
        stretch_dynamic_ratio_mono_with_engine(
            input,
            ratio_curve,
            self.ratio,
            self.window_size,
            self.analysis_hop,
            transient_reset_phase_vocoder,
        )
    }

    /// Stretch an interleaved stereo buffer with a stepwise dynamic ratio
    /// curve through the linked OfflineHighQuality prototype path.
    ///
    /// A trailing odd sample is ignored. Segment boundaries are deterministic
    /// and sample-domain; smoothing/crossfade policy remains promotion work.
    pub fn stretch_dynamic_ratio_interleaved_stereo(
        &mut self,
        frames: &[Sample],
        ratio_curve: &[StretchRatioPoint],
    ) -> Vec<Sample> {
        stretch_dynamic_ratio_linked_stereo_with_engine(
            frames,
            ratio_curve,
            self.ratio,
            self.window_size,
            self.analysis_hop,
        )
    }

    /// Report-only compression transient-anchor review path.
    ///
    /// This is not the promoted OfflineHighQuality renderer. It exists so the
    /// decoded corpus report can compare a stricter compression transient-reset
    /// candidate against the current OfflineHighQuality path before any
    /// production routing changes.
    #[doc(hidden)]
    pub fn stretch_compression_transient_anchor_review_mono(
        &mut self,
        input: &[Sample],
    ) -> Vec<Sample> {
        stretch_mono_with_engine(
            input,
            self.ratio,
            self.window_size,
            self.analysis_hop,
            compression_transient_anchor_phase_vocoder,
        )
    }

    /// Report-only sustained/coherence review path.
    ///
    /// This candidate deliberately uses a longer window and identity phase
    /// locking without transient resets. It tests whether tonal and
    /// polyphonic residuals improve when the offline path favors stable
    /// vertical phase/magnitude behavior over attack preservation. It is not a
    /// promoted OfflineHighQuality renderer.
    #[doc(hidden)]
    pub fn stretch_sustained_coherence_review_mono(&mut self, input: &[Sample]) -> Vec<Sample> {
        stretch_mono_with_engine(
            input,
            self.ratio,
            SUSTAINED_COHERENCE_REVIEW_WINDOW_SIZE,
            SUSTAINED_COHERENCE_REVIEW_ANALYSIS_HOP,
            phase_locked_phase_vocoder,
        )
    }

    /// Report-only current/long-window sustained-coherence blend path.
    ///
    /// This tests whether the long-window vertical-coherence wins can be
    /// softened by blending with the currently selected OfflineHighQuality
    /// output. It is not a promoted OfflineHighQuality renderer.
    #[doc(hidden)]
    pub fn stretch_sustained_coherence_blend_review_mono(
        &mut self,
        input: &[Sample],
    ) -> Vec<Sample> {
        let current_output = self.stretch_mono(input);
        let long_window_output = stretch_mono_with_engine(
            input,
            self.ratio,
            SUSTAINED_COHERENCE_REVIEW_WINDOW_SIZE,
            SUSTAINED_COHERENCE_REVIEW_ANALYSIS_HOP,
            phase_locked_phase_vocoder,
        );
        blend_review_outputs(
            &current_output,
            &long_window_output,
            SUSTAINED_COHERENCE_BLEND_REVIEW_WEIGHT,
        )
    }

    /// Report-only long-window sustained-coherence path with current-output
    /// envelope matching.
    ///
    /// This tests whether the long-window candidate's regressions are caused
    /// mainly by local gain/envelope drift. It is not a promoted
    /// OfflineHighQuality renderer.
    #[doc(hidden)]
    pub fn stretch_sustained_coherence_envelope_review_mono(
        &mut self,
        input: &[Sample],
    ) -> Vec<Sample> {
        let current_output = self.stretch_mono(input);
        let long_window_output = stretch_mono_with_engine(
            input,
            self.ratio,
            SUSTAINED_COHERENCE_REVIEW_WINDOW_SIZE,
            SUSTAINED_COHERENCE_REVIEW_ANALYSIS_HOP,
            phase_locked_phase_vocoder,
        );
        match_review_envelope(
            &current_output,
            &long_window_output,
            SUSTAINED_COHERENCE_ENVELOPE_REVIEW_WINDOW_SIZE,
            SUSTAINED_COHERENCE_ENVELOPE_REVIEW_HOP_SIZE,
        )
    }

    /// Report-only expansion-focused sustained-coherence path.
    ///
    /// This keeps the current OfflineHighQuality output for compression and
    /// tests long-window transient-reset phase propagation only for expansion.
    /// It is not a promoted OfflineHighQuality renderer.
    #[doc(hidden)]
    pub fn stretch_sustained_coherence_expansion_reset_review_mono(
        &mut self,
        input: &[Sample],
    ) -> Vec<Sample> {
        if self.ratio <= 1.0 {
            return self.stretch_mono(input);
        }
        stretch_mono_with_engine(
            input,
            self.ratio,
            SUSTAINED_COHERENCE_REVIEW_WINDOW_SIZE,
            SUSTAINED_COHERENCE_REVIEW_ANALYSIS_HOP,
            transient_reset_phase_vocoder,
        )
    }

    /// Report-only expansion-focused sustained-coherence path with adaptive
    /// phase locking.
    ///
    /// This keeps current OfflineHighQuality output for compression and tests
    /// long-window identity locking only on spectrally stable expansion frames.
    /// It is not a promoted OfflineHighQuality renderer.
    #[doc(hidden)]
    pub fn stretch_sustained_coherence_stability_adaptive_review_mono(
        &mut self,
        input: &[Sample],
    ) -> Vec<Sample> {
        if self.ratio <= 1.0 {
            return self.stretch_mono(input);
        }
        stretch_mono_with_engine(
            input,
            self.ratio,
            SUSTAINED_COHERENCE_REVIEW_WINDOW_SIZE,
            SUSTAINED_COHERENCE_REVIEW_ANALYSIS_HOP,
            stability_adaptive_phase_vocoder,
        )
    }

    /// Report-only expansion-focused sustained-coherence path with tracked
    /// peak-lock regions.
    ///
    /// This keeps current OfflineHighQuality output for compression and tests
    /// long-window identity locking with narrow local regions for peaks that
    /// are not tracked from the previous frame. It is not a promoted
    /// OfflineHighQuality renderer.
    #[doc(hidden)]
    pub fn stretch_sustained_coherence_tracked_peak_review_mono(
        &mut self,
        input: &[Sample],
    ) -> Vec<Sample> {
        if self.ratio <= 1.0 {
            return self.stretch_mono(input);
        }
        stretch_mono_with_engine(
            input,
            self.ratio,
            SUSTAINED_COHERENCE_REVIEW_WINDOW_SIZE,
            SUSTAINED_COHERENCE_REVIEW_ANALYSIS_HOP,
            tracked_peak_region_phase_vocoder,
        )
    }

    /// Report-only expansion-focused sustained-coherence path with
    /// stable-frame magnitude slew limiting.
    ///
    /// This keeps current OfflineHighQuality output for compression and tests
    /// long-window identity locking with bounded per-bin magnitude movement on
    /// spectrally stable frames. It is not a promoted OfflineHighQuality
    /// renderer.
    #[doc(hidden)]
    pub fn stretch_sustained_coherence_magnitude_slew_review_mono(
        &mut self,
        input: &[Sample],
    ) -> Vec<Sample> {
        if self.ratio <= 1.0 {
            return self.stretch_mono(input);
        }
        stretch_mono_with_engine(
            input,
            self.ratio,
            SUSTAINED_COHERENCE_REVIEW_WINDOW_SIZE,
            SUSTAINED_COHERENCE_REVIEW_ANALYSIS_HOP,
            magnitude_slew_phase_vocoder,
        )
    }
}

impl TimeStretcher for OfflineHighQualityStretcher {
    fn quality(&self) -> StretchQuality {
        StretchQuality::OfflineHighQuality
    }

    fn ratio(&self) -> f64 {
        self.ratio
    }

    fn set_ratio(&mut self, ratio: f64) {
        self.ratio = sanitize_ratio(ratio);
    }

    fn stretch_mono(&mut self, input: &[Sample]) -> Vec<Sample> {
        let default_output = stretch_mono_with_engine(
            input,
            self.ratio,
            self.window_size,
            self.analysis_hop,
            transient_reset_phase_vocoder,
        );
        let selected_short_window = match self.path {
            OfflineHighQualityPath::Default => false,
            OfflineHighQualityPath::CompressionShortWindowSelector => {
                should_select_compression_short_window(input, &default_output, self.ratio)
            }
            OfflineHighQualityPath::ExpansionShortWindowSelector => {
                should_select_expansion_short_window(input, &default_output, self.ratio)
            }
        };
        if selected_short_window {
            stretch_mono_with_engine(
                input,
                self.ratio,
                short_window_size_for_path(self.path),
                short_window_analysis_hop_for_path(self.path),
                transient_reset_phase_vocoder,
            )
        } else {
            default_output
        }
    }
}

/// Stretch an interleaved stereo buffer through `stretcher`, channel by
/// channel. Output frame count follows the mono length contract; both
/// channels are stretched with identical parameters so they stay
/// sample-aligned.
pub fn stretch_interleaved_stereo(
    stretcher: &mut dyn TimeStretcher,
    frames: &[Sample],
) -> Vec<Sample> {
    let frame_count = frames.len() / 2;
    let mut left = Vec::with_capacity(frame_count);
    let mut right = Vec::with_capacity(frame_count);
    for frame in frames.chunks_exact(2) {
        left.push(frame[0]);
        right.push(frame[1]);
    }
    let left = stretcher.stretch_mono(&left);
    let right = stretcher.stretch_mono(&right);
    let out_frames = left.len().min(right.len());
    let mut output = Vec::with_capacity(out_frames * 2);
    for index in 0..out_frames {
        output.push(left[index]);
        output.push(right[index]);
    }
    output
}

/// Smooth an interleaved loop boundary in place.
///
/// This is an offline loop-context helper, not a generic stretch post-process:
/// it distributes the final-to-first-frame discontinuity across `fade_frames`
/// at both ends so explicit loop renders can reduce boundary clicks while
/// preserving the interior audio.
pub fn smooth_loop_boundary_interleaved(
    interleaved_samples: &mut [Sample],
    channels: u16,
    fade_frames: usize,
) {
    let channel_count = channels as usize;
    if channel_count == 0 || fade_frames == 0 {
        return;
    }
    let frames = interleaved_samples.len() / channel_count;
    if frames < 2 {
        return;
    }

    let fade_frames = fade_frames.min(frames / 2).max(1);
    for channel in 0..channel_count {
        let first = interleaved_samples[channel];
        let last = interleaved_samples[(frames - 1) * channel_count + channel];
        let correction = (first - last) * 0.5;
        for frame in 0..fade_frames {
            let weight = (fade_frames - frame) as f32 / fade_frames as f32;
            interleaved_samples[frame * channel_count + channel] -= correction * weight;
            let tail_frame = frames - 1 - frame;
            interleaved_samples[tail_frame * channel_count + channel] += correction * weight;
        }
    }
}

/// Smooth deterministic dynamic-ratio segment joins in place.
///
/// This is an offline render helper for independently rendered segment joins.
/// It does not change output length or boundary positions.
pub(crate) fn smooth_dynamic_segment_boundaries_interleaved(
    interleaved_samples: &mut [Sample],
    channels: u16,
    boundary_frames: &[usize],
    fade_frames: usize,
) {
    let channel_count = channels as usize;
    if channel_count == 0 || fade_frames == 0 {
        return;
    }
    let frames = interleaved_samples.len() / channel_count;
    if frames < 2 {
        return;
    }

    for boundary in boundary_frames {
        if *boundary == 0 || *boundary >= frames {
            continue;
        }
        let fade_frames = fade_frames.min(*boundary).min(frames - *boundary).max(1);
        for channel in 0..channel_count {
            let before_edge_index = (*boundary - 1) * channel_count + channel;
            let after_edge_index = *boundary * channel_count + channel;
            let before_edge = interleaved_samples[before_edge_index];
            let after_edge = interleaved_samples[after_edge_index];
            let midpoint = (before_edge + after_edge) * 0.5;
            for offset in 0..fade_frames {
                let weight = (fade_frames - offset) as f32 / fade_frames as f32;
                let before_frame = *boundary - 1 - offset;
                let after_frame = *boundary + offset;
                let before_index = before_frame * channel_count + channel;
                let after_index = after_frame * channel_count + channel;
                interleaved_samples[before_index] += (midpoint - before_edge) * weight;
                interleaved_samples[after_index] += (midpoint - after_edge) * weight;
            }
        }
    }
}

/// Cheap fallback for sub-window inputs: linear interpolation over time
/// (this pitch-shifts, but a sub-window buffer is too short for the phase
/// vocoder to do better; documented, deterministic).
fn linear_time_scale(input: &[Sample], target_len: usize) -> Vec<Sample> {
    if input.len() == 1 {
        return vec![input[0]; target_len];
    }
    let step = (input.len() - 1) as f64 / (target_len.max(2) - 1) as f64;
    (0..target_len)
        .map(|index| {
            let position = index as f64 * step;
            let left = position.floor() as usize;
            let right = (left + 1).min(input.len() - 1);
            let fraction = (position - left as f64) as f32;
            input[left] + (input[right] - input[left]) * fraction
        })
        .collect()
}

fn linear_time_scale_interleaved_stereo(input: &[Sample], target_frames: usize) -> Vec<Sample> {
    let frame_count = input.len() / 2;
    let mut left = Vec::with_capacity(frame_count);
    let mut right = Vec::with_capacity(frame_count);
    for frame in input.chunks_exact(2) {
        left.push(frame[0]);
        right.push(frame[1]);
    }
    let left = linear_time_scale(&left, target_frames);
    let right = linear_time_scale(&right, target_frames);
    let out_frames = left.len().min(right.len()).min(target_frames);
    let mut output = Vec::with_capacity(target_frames * 2);
    for index in 0..out_frames {
        output.push(left[index]);
        output.push(right[index]);
    }
    output.resize(target_frames * 2, 0.0);
    output
}

fn sanitize_ratio(ratio: f64) -> f64 {
    if ratio.is_finite() && ratio > 0.0 {
        ratio
    } else {
        1.0
    }
}

fn align_to_next_grid(frame: u64, grid: u64) -> u64 {
    if grid == 0 {
        return frame;
    }
    let remainder = frame % grid;
    if remainder == 0 {
        frame
    } else {
        frame.saturating_add(grid - remainder)
    }
}

fn abs_diff_frames(left: u64, right: u64) -> usize {
    left.abs_diff(right).try_into().unwrap_or(usize::MAX)
}

fn wrap_phase(phase: f32) -> f32 {
    (phase + std::f32::consts::PI).rem_euclid(std::f32::consts::TAU) - std::f32::consts::PI
}

fn stretch_mono_with_engine(
    input: &[Sample],
    ratio: f64,
    window_size: usize,
    analysis_hop: usize,
    engine: fn(&[Sample], usize, f64, usize, usize) -> Vec<Sample>,
) -> Vec<Sample> {
    let target_len = (input.len() as f64 * ratio).round() as usize;
    if input.is_empty() || target_len == 0 {
        return Vec::new();
    }
    if (ratio - 1.0).abs() < 1.0e-9 {
        return input.to_vec();
    }
    if input.len() < window_size {
        return linear_time_scale(input, target_len);
    }
    engine(input, target_len, ratio, window_size, analysis_hop)
}

fn stretch_to_exact_mono(
    input: &[Sample],
    target_len: usize,
    window_size: usize,
    analysis_hop: usize,
    engine: fn(&[Sample], usize, f64, usize, usize) -> Vec<Sample>,
) -> Vec<Sample> {
    if input.is_empty() || target_len == 0 {
        return Vec::new();
    }
    let ratio = target_len as f64 / input.len() as f64;
    if (ratio - 1.0).abs() < 1.0e-9 {
        let mut output = input.to_vec();
        output.resize(target_len, 0.0);
        return output;
    }
    if input.len() < window_size {
        return linear_time_scale(input, target_len);
    }
    engine(input, target_len, ratio, window_size, analysis_hop)
}

pub(crate) fn dynamic_ratio_output_frames(
    input_frames: usize,
    ratio_curve: &[StretchRatioPoint],
    fallback_ratio: f64,
) -> usize {
    dynamic_ratio_segments(input_frames, ratio_curve, sanitize_ratio(fallback_ratio))
        .iter()
        .map(|segment| segment.target_frames)
        .sum()
}

pub(crate) fn dynamic_ratio_output_boundaries(
    input_frames: usize,
    ratio_curve: &[StretchRatioPoint],
    fallback_ratio: f64,
) -> Vec<usize> {
    let segments =
        dynamic_ratio_segments(input_frames, ratio_curve, sanitize_ratio(fallback_ratio));
    dynamic_ratio_segment_boundaries(&segments)
}

pub(crate) fn stretch_dynamic_ratio_mono_with_engine(
    input: &[Sample],
    ratio_curve: &[StretchRatioPoint],
    fallback_ratio: f64,
    window_size: usize,
    analysis_hop: usize,
    engine: fn(&[Sample], usize, f64, usize, usize) -> Vec<Sample>,
) -> Vec<Sample> {
    let segments = dynamic_ratio_segments(input.len(), ratio_curve, sanitize_ratio(fallback_ratio));
    let target_len: usize = segments.iter().map(|segment| segment.target_frames).sum();
    let mut output = Vec::with_capacity(target_len);
    for segment in segments {
        let rendered = stretch_to_exact_mono(
            &input[segment.start_frame..segment.end_frame],
            segment.target_frames,
            window_size,
            analysis_hop,
            engine,
        );
        output.extend(rendered);
    }
    output
}

fn stretch_dynamic_ratio_linked_stereo_with_engine(
    input: &[Sample],
    ratio_curve: &[StretchRatioPoint],
    fallback_ratio: f64,
    window_size: usize,
    analysis_hop: usize,
) -> Vec<Sample> {
    let frame_count = input.len() / 2;
    let even_input = &input[..frame_count * 2];
    let segments = dynamic_ratio_segments(frame_count, ratio_curve, sanitize_ratio(fallback_ratio));
    let boundaries = dynamic_ratio_segment_boundaries(&segments);
    let target_frames: usize = segments.iter().map(|segment| segment.target_frames).sum();
    let mut output = Vec::with_capacity(target_frames * 2);
    for segment in segments {
        let start = segment.start_frame * 2;
        let end = segment.end_frame * 2;
        let rendered = stretch_to_exact_linked_stereo(
            &even_input[start..end],
            segment.target_frames,
            window_size,
            analysis_hop,
        );
        output.extend(rendered);
    }
    smooth_dynamic_segment_boundaries_interleaved(
        &mut output,
        2,
        &boundaries,
        DYNAMIC_RATIO_SEAM_SMOOTH_FRAMES,
    );
    output
}

fn stretch_to_exact_linked_stereo(
    input: &[Sample],
    target_frames: usize,
    window_size: usize,
    analysis_hop: usize,
) -> Vec<Sample> {
    let frame_count = input.len() / 2;
    if frame_count == 0 || target_frames == 0 {
        return Vec::new();
    }
    let ratio = target_frames as f64 / frame_count as f64;
    if (ratio - 1.0).abs() < 1.0e-9 {
        let mut output = input[..frame_count * 2].to_vec();
        output.resize(target_frames * 2, 0.0);
        return output;
    }
    if frame_count < window_size {
        return linear_time_scale_interleaved_stereo(&input[..frame_count * 2], target_frames);
    }
    transient_reset_phase_vocoder_linked_stereo(
        &input[..frame_count * 2],
        target_frames,
        ratio,
        window_size,
        analysis_hop,
    )
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct DynamicRatioSegment {
    start_frame: usize,
    end_frame: usize,
    target_frames: usize,
    ratio: f64,
}

fn dynamic_ratio_segments(
    input_frames: usize,
    ratio_curve: &[StretchRatioPoint],
    fallback_ratio: f64,
) -> Vec<DynamicRatioSegment> {
    if input_frames == 0 {
        return Vec::new();
    }

    let mut points = std::collections::BTreeMap::<usize, f64>::new();
    for point in ratio_curve {
        if point.timeline_frame < 0 || !point.ratio.is_finite() || point.ratio <= 0.0 {
            continue;
        }
        points.insert(point.timeline_frame as usize, point.ratio);
    }

    let mut segments = Vec::new();
    let mut start_frame = 0usize;
    let mut ratio = sanitize_ratio(fallback_ratio);
    for (point_frame, point_ratio) in points {
        let point_frame = point_frame.min(input_frames);
        if point_frame > start_frame {
            segments.push(dynamic_ratio_segment(start_frame, point_frame, ratio));
        }
        ratio = point_ratio;
        start_frame = point_frame;
    }

    if start_frame < input_frames {
        segments.push(dynamic_ratio_segment(start_frame, input_frames, ratio));
    }
    segments
}

fn dynamic_ratio_segment(start_frame: usize, end_frame: usize, ratio: f64) -> DynamicRatioSegment {
    DynamicRatioSegment {
        start_frame,
        end_frame,
        target_frames: ((end_frame - start_frame) as f64 * ratio).round() as usize,
        ratio,
    }
}

fn dynamic_ratio_segment_boundaries(segments: &[DynamicRatioSegment]) -> Vec<usize> {
    let mut boundaries = Vec::with_capacity(segments.len().saturating_sub(1));
    let total_frames: usize = segments.iter().map(|segment| segment.target_frames).sum();
    let mut output_frame = 0usize;
    for segment in segments.iter().take(segments.len().saturating_sub(1)) {
        output_frame += segment.target_frames;
        if output_frame > 0 && output_frame < total_frames {
            boundaries.push(output_frame);
        }
    }
    boundaries
}

fn pitch_shift_mono_to_nominal_rate(
    input: &[Sample],
    sample_rate: SampleRate,
    semitones: f64,
) -> Vec<Sample> {
    let Some(config) = pitch_shift_resample_config(sample_rate, semitones) else {
        return input.to_vec();
    };
    resample_mono(config, input)
}

fn pitch_shift_interleaved_stereo_to_nominal_rate(
    input: &[Sample],
    sample_rate: SampleRate,
    semitones: f64,
) -> Vec<Sample> {
    let frame_count = input.len() / 2;
    let Some(config) = pitch_shift_resample_config(sample_rate, semitones) else {
        return input[..frame_count * 2].to_vec();
    };

    let mut mid = Vec::with_capacity(frame_count);
    let mut side = Vec::with_capacity(frame_count);
    for frame in input[..frame_count * 2].chunks_exact(2) {
        let left = frame[0];
        let right = frame[1];
        mid.push((left + right) * 0.5);
        side.push((left - right) * 0.5);
    }
    let mid = resample_mono(config, &mid);
    let side = resample_mono(config, &side);
    let out_frames = mid.len().min(side.len());
    let mut output = Vec::with_capacity(out_frames * 2);
    for index in 0..out_frames {
        output.push(mid[index] + side[index]);
        output.push(mid[index] - side[index]);
    }
    output
}

fn pitch_shift_resample_config(sample_rate: SampleRate, semitones: f64) -> Option<ResampleConfig> {
    if sample_rate.0 == 0 || !semitones.is_finite() || semitones.abs() < 1.0e-9 {
        return None;
    }
    let factor = 2.0f64.powf(semitones / 12.0);
    if !factor.is_finite() || factor <= 0.0 {
        return None;
    }
    let virtual_input_rate =
        ((sample_rate.0 as f64 * factor).round()).clamp(1.0, u32::MAX as f64) as u32;
    Some(ResampleConfig::new(
        SampleRate(virtual_input_rate),
        sample_rate,
        ResampleQuality::BandLimited,
    ))
}

fn should_select_compression_short_window(
    input: &[Sample],
    current_output: &[Sample],
    ratio: f64,
) -> bool {
    if ratio >= 1.0 || input.is_empty() || current_output.is_empty() {
        return false;
    }

    let current_smear = measure_transient_smear(
        input,
        current_output,
        ratio,
        COMPRESSION_SHORT_WINDOW_SELECTOR_WINDOW_SIZE,
        COMPRESSION_SHORT_WINDOW_SELECTOR_ANALYSIS_HOP,
    );
    current_smear.missed_transients >= COMPRESSION_SHORT_WINDOW_SELECTOR_MIN_CURRENT_MISSES
        || current_smear.max_smear_frames
            >= COMPRESSION_SHORT_WINDOW_SELECTOR_MIN_CURRENT_SMEAR_FRAMES
}

fn should_select_compression_short_window_interleaved(
    input: &[Sample],
    current_output: &[Sample],
    ratio: f64,
) -> bool {
    let input_mono = downmix_interleaved_stereo_to_mono(input);
    let output_mono = downmix_interleaved_stereo_to_mono(current_output);
    should_select_compression_short_window(&input_mono, &output_mono, ratio)
}

fn should_select_expansion_short_window(
    input: &[Sample],
    current_output: &[Sample],
    ratio: f64,
) -> bool {
    if ratio <= 1.0 || input.is_empty() || current_output.is_empty() {
        return false;
    }

    let current_smear = measure_transient_smear(
        input,
        current_output,
        ratio,
        EXPANSION_SHORT_WINDOW_SELECTOR_WINDOW_SIZE,
        EXPANSION_SHORT_WINDOW_SELECTOR_ANALYSIS_HOP,
    );
    if current_smear.missed_transients >= EXPANSION_SHORT_WINDOW_SELECTOR_MIN_CURRENT_MISSES {
        return true;
    }

    let mut draft = PhaseVocoderStretcher::new(ratio);
    let draft_output = draft.stretch_mono(input);
    let draft_smear = measure_transient_smear(
        input,
        &draft_output,
        ratio,
        EXPANSION_SHORT_WINDOW_SELECTOR_WINDOW_SIZE,
        EXPANSION_SHORT_WINDOW_SELECTOR_ANALYSIS_HOP,
    );
    metric_worsened(current_smear.max_smear_frames, draft_smear.max_smear_frames)
}

fn should_select_expansion_short_window_interleaved(
    input: &[Sample],
    current_output: &[Sample],
    ratio: f64,
) -> bool {
    let input_mono = downmix_interleaved_stereo_to_mono(input);
    let output_mono = downmix_interleaved_stereo_to_mono(current_output);
    should_select_expansion_short_window(&input_mono, &output_mono, ratio)
}

fn metric_worsened(candidate: f64, production: f64) -> bool {
    if candidate.is_finite() && production.is_finite() {
        candidate > production
    } else {
        !candidate.is_finite() && production.is_finite()
    }
}

fn short_window_size_for_path(path: OfflineHighQualityPath) -> usize {
    match path {
        OfflineHighQualityPath::Default => DEFAULT_WINDOW_SIZE,
        OfflineHighQualityPath::CompressionShortWindowSelector => {
            COMPRESSION_SHORT_WINDOW_SELECTOR_WINDOW_SIZE
        }
        OfflineHighQualityPath::ExpansionShortWindowSelector => {
            EXPANSION_SHORT_WINDOW_SELECTOR_WINDOW_SIZE
        }
    }
}

fn short_window_analysis_hop_for_path(path: OfflineHighQualityPath) -> usize {
    match path {
        OfflineHighQualityPath::Default => DEFAULT_ANALYSIS_HOP,
        OfflineHighQualityPath::CompressionShortWindowSelector => {
            COMPRESSION_SHORT_WINDOW_SELECTOR_ANALYSIS_HOP
        }
        OfflineHighQualityPath::ExpansionShortWindowSelector => {
            EXPANSION_SHORT_WINDOW_SELECTOR_ANALYSIS_HOP
        }
    }
}

fn downmix_interleaved_stereo_to_mono(samples: &[Sample]) -> Vec<Sample> {
    samples
        .chunks_exact(2)
        .map(|frame| (frame[0] + frame[1]) * 0.5)
        .collect()
}

fn blend_review_outputs(
    current: &[Sample],
    candidate: &[Sample],
    candidate_weight: f32,
) -> Vec<Sample> {
    let candidate_weight = candidate_weight.clamp(0.0, 1.0);
    let current_weight = 1.0 - candidate_weight;
    current
        .iter()
        .enumerate()
        .map(|(index, current_sample)| {
            let candidate_sample = candidate.get(index).copied().unwrap_or(0.0);
            current_sample * current_weight + candidate_sample * candidate_weight
        })
        .collect()
}

fn match_review_envelope(
    reference: &[Sample],
    candidate: &[Sample],
    window_size: usize,
    hop_size: usize,
) -> Vec<Sample> {
    let len = reference.len();
    if len == 0 {
        return Vec::new();
    }
    let window_size = window_size.clamp(1, len);
    let hop_size = hop_size.clamp(1, window_size);
    let mut output = vec![0.0; len];
    let mut normalization = vec![0.0; len];
    let mut start = 0;
    loop {
        let end = (start + window_size).min(len);
        let reference_rms = block_rms(&reference[start..end]);
        let candidate_rms = block_rms(&candidate[start..candidate.len().min(end)]);
        let gain = if candidate_rms > 1.0e-9 {
            (reference_rms / candidate_rms).clamp(0.25, 4.0)
        } else {
            1.0
        };
        for index in start..end {
            let sample = candidate.get(index).copied().unwrap_or(0.0);
            output[index] += sample * gain;
            normalization[index] += 1.0;
        }
        if end == len {
            break;
        }
        start += hop_size;
    }
    for (sample, weight) in output.iter_mut().zip(normalization.iter()) {
        if *weight > 0.0 {
            *sample /= *weight;
        }
    }
    output
}

fn block_rms(samples: &[Sample]) -> f32 {
    if samples.is_empty() {
        return 0.0;
    }
    (samples.iter().map(|sample| sample * sample).sum::<f32>() / samples.len() as f32).sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sine(frequency_hz: f32, sample_rate_hz: f32, len: usize) -> Vec<Sample> {
        (0..len)
            .map(|index| {
                (std::f32::consts::TAU * frequency_hz * index as f32 / sample_rate_hz).sin()
            })
            .collect()
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

    fn rms(samples: &[Sample]) -> f32 {
        (samples.iter().map(|s| s * s).sum::<f32>() / samples.len().max(1) as f32).sqrt()
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
        assert!(!contract.audio_thread_processing_supported);
        assert_eq!(
            contract.unsupported_mode,
            Some(RealtimePreviewUnsupportedMode::AudioThreadProcessing)
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
        assert!(state.fft_plans_ready());
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

        let first_output = first.stretch_mono(&input);
        let second_output = second.stretch_mono(&input);

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

        let first_output = first.stretch_interleaved_stereo(&input);
        let second_output = second.stretch_interleaved_stereo(&input);

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

        let output = stretcher.stretch_dynamic_ratio_mono(&input, &ratio_curve);

        assert_eq!(output.len(), 20_000);
    }

    #[test]
    fn realtime_preview_pitch_shift_preserves_tempo_length_contract() {
        let input = sine(440.0, 48_000.0, 12_000);
        let mut stretcher = RealtimePreviewStretcher::new(1.25);

        let output = stretcher.stretch_pitch_mono(&input, SampleRate(48_000), 12.0);

        assert_eq!(output.len(), 15_000);
        assert!((dominant_frequency_hz(&output, 48_000.0) - 880.0).abs() < 35.0);
    }

    fn add_decaying_burst(samples: &mut [Sample], start: usize, frames: usize, amplitude: f32) {
        for offset in 0..frames {
            let Some(sample) = samples.get_mut(start + offset) else {
                break;
            };
            let envelope = 1.0 - offset as f32 / frames as f32;
            let polarity = if offset % 2 == 0 { 1.0 } else { -1.0 };
            *sample += amplitude * envelope * polarity;
        }
    }

    fn masked_soft_attack_probe(soft_attack_amplitude: f32) -> Vec<Sample> {
        let mut input = sine(180.0, 48_000.0, 48_000)
            .into_iter()
            .map(|sample| sample * 0.06)
            .collect::<Vec<_>>();
        add_decaying_burst(&mut input, 8_000, 96, 1.0);
        add_decaying_burst(&mut input, 24_000, 96, soft_attack_amplitude);
        input
    }

    #[test]
    fn identity_ratio_is_passthrough() {
        let input = sine(440.0, 48_000.0, 10_000);
        let mut stretcher = PhaseVocoderStretcher::new(1.0);
        assert_eq!(stretcher.stretch_mono(&input), input);
    }

    #[test]
    fn ratio_clamps_invalid_values_to_identity() {
        let mut stretcher = PhaseVocoderStretcher::new(f64::NAN);
        assert_eq!(stretcher.ratio(), 1.0);
        stretcher.set_ratio(-2.0);
        assert_eq!(stretcher.ratio(), 1.0);
        stretcher.set_ratio(1.5);
        assert_eq!(stretcher.ratio(), 1.5);
    }

    #[test]
    fn stretch_honors_output_length_contract() {
        let input = sine(440.0, 48_000.0, 48_000);
        for ratio in [0.5, 0.75, 1.25, 1.5, 2.0] {
            let mut stretcher = PhaseVocoderStretcher::new(ratio);
            let output = stretcher.stretch_mono(&input);
            assert_eq!(
                output.len(),
                (input.len() as f64 * ratio).round() as usize,
                "ratio {ratio}"
            );
        }
    }

    #[test]
    fn offline_high_quality_reports_target_quality() {
        let stretcher = OfflineHighQualityStretcher::new(1.25);

        assert_eq!(stretcher.quality(), StretchQuality::OfflineHighQuality);
        assert_eq!(stretcher.ratio(), 1.25);
        assert_eq!(stretcher.path(), OfflineHighQualityPath::Default);
    }

    #[test]
    fn offline_high_quality_path_can_be_selected_explicitly() {
        let mut stretcher = OfflineHighQualityStretcher::with_path(
            0.75,
            OfflineHighQualityPath::CompressionShortWindowSelector,
        );

        assert_eq!(
            stretcher.path(),
            OfflineHighQualityPath::CompressionShortWindowSelector
        );
        stretcher.set_path(OfflineHighQualityPath::Default);
        assert_eq!(stretcher.path(), OfflineHighQualityPath::Default);
        stretcher.set_path(OfflineHighQualityPath::ExpansionShortWindowSelector);
        assert_eq!(
            stretcher.path(),
            OfflineHighQualityPath::ExpansionShortWindowSelector
        );
    }

    #[test]
    fn offline_high_quality_is_deterministic_and_honors_output_length() {
        let input = sine(440.0, 48_000.0, 48_000);
        for ratio in [0.5, 0.75, 1.25, 1.5, 2.0] {
            let mut first = OfflineHighQualityStretcher::new(ratio);
            let mut repeated = OfflineHighQualityStretcher::new(ratio);
            let first_output = first.stretch_mono(&input);
            let repeated_output = repeated.stretch_mono(&input);

            assert_eq!(
                first_output.len(),
                (input.len() as f64 * ratio).round() as usize,
                "ratio {ratio}"
            );
            assert_eq!(first_output, repeated_output, "ratio {ratio}");
        }
    }

    #[test]
    fn compression_transient_anchor_review_path_is_deterministic_and_honors_output_length() {
        let input = masked_soft_attack_probe(0.35);
        let ratio = 0.75;
        let mut first = OfflineHighQualityStretcher::new(ratio);
        let mut repeated = OfflineHighQualityStretcher::new(ratio);
        let first_output = first.stretch_compression_transient_anchor_review_mono(&input);
        let repeated_output = repeated.stretch_compression_transient_anchor_review_mono(&input);

        assert_eq!(
            first_output.len(),
            (input.len() as f64 * ratio).round() as usize
        );
        assert_eq!(first_output, repeated_output);
    }

    #[test]
    fn sustained_coherence_review_path_is_deterministic_and_honors_output_length() {
        let input = sine(110.0, 48_000.0, 48_000)
            .into_iter()
            .zip(sine(220.0, 48_000.0, 48_000))
            .map(|(low, high)| low * 0.7 + high * 0.3)
            .collect::<Vec<_>>();
        let ratio = 1.25;
        let mut first = OfflineHighQualityStretcher::new(ratio);
        let mut repeated = OfflineHighQualityStretcher::new(ratio);
        let first_output = first.stretch_sustained_coherence_review_mono(&input);
        let repeated_output = repeated.stretch_sustained_coherence_review_mono(&input);

        assert_eq!(
            first_output.len(),
            (input.len() as f64 * ratio).round() as usize
        );
        assert_eq!(first_output, repeated_output);
    }

    #[test]
    fn sustained_coherence_blend_review_path_is_deterministic_and_honors_output_length() {
        let input = sine(110.0, 48_000.0, 48_000)
            .into_iter()
            .zip(sine(220.0, 48_000.0, 48_000))
            .map(|(low, high)| low * 0.7 + high * 0.3)
            .collect::<Vec<_>>();
        let ratio = 1.25;
        let mut first = OfflineHighQualityStretcher::new(ratio);
        let mut repeated = OfflineHighQualityStretcher::new(ratio);
        let first_output = first.stretch_sustained_coherence_blend_review_mono(&input);
        let repeated_output = repeated.stretch_sustained_coherence_blend_review_mono(&input);

        assert_eq!(
            first_output.len(),
            (input.len() as f64 * ratio).round() as usize
        );
        assert_eq!(first_output, repeated_output);
    }

    #[test]
    fn sustained_coherence_envelope_review_path_is_deterministic_and_honors_output_length() {
        let input = sine(110.0, 48_000.0, 48_000)
            .into_iter()
            .zip(sine(220.0, 48_000.0, 48_000))
            .map(|(low, high)| low * 0.7 + high * 0.3)
            .collect::<Vec<_>>();
        let ratio = 1.25;
        let mut first = OfflineHighQualityStretcher::new(ratio);
        let mut repeated = OfflineHighQualityStretcher::new(ratio);
        let first_output = first.stretch_sustained_coherence_envelope_review_mono(&input);
        let repeated_output = repeated.stretch_sustained_coherence_envelope_review_mono(&input);

        assert_eq!(
            first_output.len(),
            (input.len() as f64 * ratio).round() as usize
        );
        assert_eq!(first_output, repeated_output);
    }

    #[test]
    fn sustained_coherence_expansion_reset_review_path_is_ratio_scoped_and_deterministic() {
        let input = sine(110.0, 48_000.0, 48_000)
            .into_iter()
            .zip(sine(220.0, 48_000.0, 48_000))
            .map(|(low, high)| low * 0.7 + high * 0.3)
            .collect::<Vec<_>>();
        let mut compression_candidate = OfflineHighQualityStretcher::new(0.75);
        let mut compression_current = OfflineHighQualityStretcher::new(0.75);

        assert_eq!(
            compression_candidate.stretch_sustained_coherence_expansion_reset_review_mono(&input),
            compression_current.stretch_mono(&input)
        );

        let ratio = 1.25;
        let mut first = OfflineHighQualityStretcher::new(ratio);
        let mut repeated = OfflineHighQualityStretcher::new(ratio);
        let first_output = first.stretch_sustained_coherence_expansion_reset_review_mono(&input);
        let repeated_output =
            repeated.stretch_sustained_coherence_expansion_reset_review_mono(&input);

        assert_eq!(
            first_output.len(),
            (input.len() as f64 * ratio).round() as usize
        );
        assert_eq!(first_output, repeated_output);
    }

    #[test]
    fn sustained_coherence_stability_adaptive_review_path_is_ratio_scoped_and_deterministic() {
        let input = sine(110.0, 48_000.0, 48_000)
            .into_iter()
            .zip(sine(220.0, 48_000.0, 48_000))
            .map(|(low, high)| low * 0.7 + high * 0.3)
            .collect::<Vec<_>>();
        let mut compression_candidate = OfflineHighQualityStretcher::new(0.75);
        let mut compression_current = OfflineHighQualityStretcher::new(0.75);

        assert_eq!(
            compression_candidate
                .stretch_sustained_coherence_stability_adaptive_review_mono(&input),
            compression_current.stretch_mono(&input)
        );

        let ratio = 1.25;
        let mut first = OfflineHighQualityStretcher::new(ratio);
        let mut repeated = OfflineHighQualityStretcher::new(ratio);
        let first_output = first.stretch_sustained_coherence_stability_adaptive_review_mono(&input);
        let repeated_output =
            repeated.stretch_sustained_coherence_stability_adaptive_review_mono(&input);

        assert_eq!(
            first_output.len(),
            (input.len() as f64 * ratio).round() as usize
        );
        assert_eq!(first_output, repeated_output);
    }

    #[test]
    fn sustained_coherence_tracked_peak_review_path_is_ratio_scoped_and_deterministic() {
        let input = sine(110.0, 48_000.0, 48_000)
            .into_iter()
            .zip(sine(220.0, 48_000.0, 48_000))
            .map(|(low, high)| low * 0.7 + high * 0.3)
            .collect::<Vec<_>>();
        let mut compression_candidate = OfflineHighQualityStretcher::new(0.75);
        let mut compression_current = OfflineHighQualityStretcher::new(0.75);

        assert_eq!(
            compression_candidate.stretch_sustained_coherence_tracked_peak_review_mono(&input),
            compression_current.stretch_mono(&input)
        );

        let ratio = 1.25;
        let mut first = OfflineHighQualityStretcher::new(ratio);
        let mut repeated = OfflineHighQualityStretcher::new(ratio);
        let first_output = first.stretch_sustained_coherence_tracked_peak_review_mono(&input);
        let repeated_output = repeated.stretch_sustained_coherence_tracked_peak_review_mono(&input);

        assert_eq!(
            first_output.len(),
            (input.len() as f64 * ratio).round() as usize
        );
        assert_eq!(first_output, repeated_output);
    }

    #[test]
    fn sustained_coherence_magnitude_slew_review_path_is_ratio_scoped_and_deterministic() {
        let input = sine(110.0, 48_000.0, 48_000)
            .into_iter()
            .zip(sine(220.0, 48_000.0, 48_000))
            .map(|(low, high)| low * 0.7 + high * 0.3)
            .collect::<Vec<_>>();
        let mut compression_candidate = OfflineHighQualityStretcher::new(0.75);
        let mut compression_current = OfflineHighQualityStretcher::new(0.75);

        assert_eq!(
            compression_candidate.stretch_sustained_coherence_magnitude_slew_review_mono(&input),
            compression_current.stretch_mono(&input)
        );

        let ratio = 1.25;
        let mut first = OfflineHighQualityStretcher::new(ratio);
        let mut repeated = OfflineHighQualityStretcher::new(ratio);
        let first_output = first.stretch_sustained_coherence_magnitude_slew_review_mono(&input);
        let repeated_output =
            repeated.stretch_sustained_coherence_magnitude_slew_review_mono(&input);

        assert_eq!(
            first_output.len(),
            (input.len() as f64 * ratio).round() as usize
        );
        assert_eq!(first_output, repeated_output);
    }

    #[test]
    fn compression_short_window_selector_matches_gate_decision() {
        let input = masked_soft_attack_probe(0.35);
        let ratio = 0.75;
        let mut default = OfflineHighQualityStretcher::new(ratio);
        let default_output = default.stretch_mono(&input);
        let mut short_window = OfflineHighQualityStretcher::with_window(
            ratio,
            COMPRESSION_SHORT_WINDOW_SELECTOR_WINDOW_SIZE,
            COMPRESSION_SHORT_WINDOW_SELECTOR_ANALYSIS_HOP,
        );
        let short_window_output = short_window.stretch_mono(&input);
        let mut selector = OfflineHighQualityStretcher::with_path(
            ratio,
            OfflineHighQualityPath::CompressionShortWindowSelector,
        );
        let selector_output = selector.stretch_mono(&input);
        let default_smear = measure_transient_smear(
            &input,
            &default_output,
            ratio,
            COMPRESSION_SHORT_WINDOW_SELECTOR_WINDOW_SIZE,
            COMPRESSION_SHORT_WINDOW_SELECTOR_ANALYSIS_HOP,
        );
        let accepted = default_smear.missed_transients
            >= COMPRESSION_SHORT_WINDOW_SELECTOR_MIN_CURRENT_MISSES
            || default_smear.max_smear_frames
                >= COMPRESSION_SHORT_WINDOW_SELECTOR_MIN_CURRENT_SMEAR_FRAMES;

        let expected = if accepted {
            &short_window_output
        } else {
            &default_output
        };
        assert_eq!(selector_output, *expected);
        assert_eq!(
            selector_output.len(),
            (input.len() as f64 * ratio).round() as usize
        );
    }

    #[test]
    fn compression_short_window_selector_does_not_switch_expansion_ratios() {
        let input = masked_soft_attack_probe(0.35);
        let ratio = 1.25;
        let mut default = OfflineHighQualityStretcher::new(ratio);
        let default_output = default.stretch_mono(&input);
        let mut selector = OfflineHighQualityStretcher::with_path(
            ratio,
            OfflineHighQualityPath::CompressionShortWindowSelector,
        );

        assert_eq!(selector.stretch_mono(&input), default_output);
    }

    #[test]
    fn expansion_short_window_selector_matches_gate_decision() {
        let input = masked_soft_attack_probe(0.35);
        let ratio = 1.25;
        let mut default = OfflineHighQualityStretcher::new(ratio);
        let default_output = default.stretch_mono(&input);
        let mut short_window = OfflineHighQualityStretcher::with_window(
            ratio,
            EXPANSION_SHORT_WINDOW_SELECTOR_WINDOW_SIZE,
            EXPANSION_SHORT_WINDOW_SELECTOR_ANALYSIS_HOP,
        );
        let short_window_output = short_window.stretch_mono(&input);
        let mut selector = OfflineHighQualityStretcher::with_path(
            ratio,
            OfflineHighQualityPath::ExpansionShortWindowSelector,
        );
        let selector_output = selector.stretch_mono(&input);
        let accepted = should_select_expansion_short_window(&input, &default_output, ratio);

        let expected = if accepted {
            &short_window_output
        } else {
            &default_output
        };
        assert_eq!(selector_output, *expected);
        assert_eq!(
            selector_output.len(),
            (input.len() as f64 * ratio).round() as usize
        );
    }

    #[test]
    fn expansion_short_window_selector_rejects_compression_ratios() {
        let input = masked_soft_attack_probe(0.35);
        let ratio = 0.75;
        let mut default = OfflineHighQualityStretcher::new(ratio);
        let default_output = default.stretch_mono(&input);
        let mut selector = OfflineHighQualityStretcher::with_path(
            ratio,
            OfflineHighQualityPath::ExpansionShortWindowSelector,
        );

        assert_eq!(selector.stretch_mono(&input), default_output);
    }

    #[test]
    fn expansion_short_window_gate_accepts_current_misses() {
        let input = masked_soft_attack_probe(0.35);
        let ratio = 1.25;
        let silent_current = vec![0.0; (input.len() as f64 * ratio).round() as usize];

        assert!(should_select_expansion_short_window(
            &input,
            &silent_current,
            ratio
        ));
        assert!(!should_select_expansion_short_window(
            &input,
            &silent_current,
            0.75
        ));
    }

    #[test]
    fn offline_high_quality_identity_ratio_is_passthrough() {
        let input = sine(330.0, 48_000.0, 8_192);
        let mut stretcher = OfflineHighQualityStretcher::new(1.0);

        assert_eq!(stretcher.stretch_mono(&input), input);
    }

    #[test]
    fn stretch_preserves_pitch_within_tolerance() {
        let sample_rate = 48_000.0;
        let input = sine(440.0, sample_rate, 48_000);
        for ratio in [0.75, 1.5, 2.0] {
            let mut stretcher = PhaseVocoderStretcher::new(ratio);
            let output = stretcher.stretch_mono(&input);
            let frequency = dominant_frequency_hz(&output, sample_rate);
            assert!(
                (frequency - 440.0).abs() < 15.0,
                "ratio {ratio}: dominant frequency {frequency} Hz, expected ~440 Hz"
            );
            assert!(
                rms(&output) > 0.3,
                "ratio {ratio}: stretched output lost energy (rms {})",
                rms(&output)
            );
        }
    }

    #[test]
    fn sub_window_input_scales_by_linear_fallback() {
        let input: Vec<f32> = (0..100).map(|index| index as f32 / 100.0).collect();
        let mut stretcher = PhaseVocoderStretcher::new(2.0);
        let output = stretcher.stretch_mono(&input);
        assert_eq!(output.len(), 200);
        // Monotone ramp stays monotone under linear scaling.
        assert!(output.windows(2).all(|pair| pair[1] >= pair[0] - 1.0e-6));
    }

    #[test]
    fn stereo_helper_keeps_channels_aligned_and_interleaved() {
        let sample_rate = 48_000.0;
        let left = sine(440.0, sample_rate, 24_000);
        let right = sine(220.0, sample_rate, 24_000);
        let mut frames = Vec::with_capacity(left.len() * 2);
        for (l, r) in left.iter().zip(right.iter()) {
            frames.push(*l);
            frames.push(*r);
        }
        let mut stretcher = PhaseVocoderStretcher::new(1.5);
        let output = stretch_interleaved_stereo(&mut stretcher, &frames);
        assert_eq!(output.len() % 2, 0);
        assert_eq!(output.len() / 2, (24_000f64 * 1.5).round() as usize);
        let out_left: Vec<f32> = output.iter().step_by(2).copied().collect();
        let out_right: Vec<f32> = output.iter().skip(1).step_by(2).copied().collect();
        assert!((dominant_frequency_hz(&out_left, sample_rate) - 440.0).abs() < 15.0);
        assert!((dominant_frequency_hz(&out_right, sample_rate) - 220.0).abs() < 10.0);
    }

    #[test]
    fn offline_high_quality_linked_stereo_honors_output_length_contract() {
        let sample_rate = 48_000.0;
        let left = sine(440.0, sample_rate, 48_000);
        let right = sine(660.0, sample_rate, 48_000);
        let mut frames = Vec::with_capacity(left.len() * 2);
        for (l, r) in left.iter().zip(right.iter()) {
            frames.push(*l);
            frames.push(*r);
        }

        for ratio in [0.5, 0.75, 1.25, 1.5, 2.0] {
            let mut stretcher = OfflineHighQualityStretcher::new(ratio);
            let output = stretcher.stretch_interleaved_stereo(&frames);

            assert_eq!(
                output.len(),
                ((left.len() as f64 * ratio).round() as usize) * 2,
                "ratio {ratio}"
            );
        }
    }

    #[test]
    fn offline_high_quality_linked_stereo_is_identity_passthrough() {
        let frames = [0.0, 0.1, 0.2, 0.3, 0.4];
        let mut stretcher = OfflineHighQualityStretcher::new(1.0);

        assert_eq!(stretcher.stretch_interleaved_stereo(&frames), frames[..4]);
    }

    #[test]
    fn offline_high_quality_linked_stereo_is_deterministic() {
        let sample_rate = 48_000.0;
        let left = sine(330.0, sample_rate, 48_000);
        let right = sine(550.0, sample_rate, 48_000);
        let mut frames = Vec::with_capacity(left.len() * 2);
        for (l, r) in left.iter().zip(right.iter()) {
            frames.push(*l);
            frames.push(*r);
        }

        let mut first = OfflineHighQualityStretcher::new(1.5);
        let mut repeated = OfflineHighQualityStretcher::new(1.5);

        assert_eq!(
            first.stretch_interleaved_stereo(&frames),
            repeated.stretch_interleaved_stereo(&frames)
        );
    }

    #[test]
    fn offline_high_quality_pitch_shift_preserves_tempo_length_contract() {
        let input = sine(440.0, 48_000.0, 48_000);
        for (ratio, semitones) in [(1.0, 12.0), (1.5, -7.0), (0.75, 5.0)] {
            let mut stretcher = OfflineHighQualityStretcher::new(ratio);
            let output = stretcher.stretch_pitch_mono(&input, SampleRate(48_000), semitones);

            assert_eq!(
                output.len(),
                (input.len() as f64 * ratio).round() as usize,
                "ratio {ratio}, semitones {semitones}"
            );
        }
    }

    #[test]
    fn offline_high_quality_pitch_shift_raises_tonal_pitch() {
        let sample_rate = 48_000.0;
        let input = sine(440.0, sample_rate, 48_000);
        let mut stretcher = OfflineHighQualityStretcher::new(1.0);

        let output = stretcher.stretch_pitch_mono(&input, SampleRate(48_000), 12.0);
        let frequency = dominant_frequency_hz(&output, sample_rate);

        assert_eq!(output.len(), input.len());
        assert!(
            (frequency - 880.0).abs() < 35.0,
            "expected pitch near 880 Hz, got {frequency} Hz"
        );
    }

    #[test]
    fn offline_high_quality_pitch_shift_stereo_is_exact_and_deterministic() {
        let sample_rate = 48_000.0;
        let left = sine(220.0, sample_rate, 48_000);
        let right = sine(440.0, sample_rate, 48_000);
        let mut frames = Vec::with_capacity(left.len() * 2);
        for (l, r) in left.iter().zip(right.iter()) {
            frames.push(*l);
            frames.push(*r);
        }

        let mut first = OfflineHighQualityStretcher::new(1.25);
        let mut repeated = OfflineHighQualityStretcher::new(1.25);
        let first_output =
            first.stretch_pitch_interleaved_stereo(&frames, SampleRate(48_000), -5.0);
        let repeated_output =
            repeated.stretch_pitch_interleaved_stereo(&frames, SampleRate(48_000), -5.0);

        assert_eq!(first_output.len(), (48_000f64 * 1.25).round() as usize * 2);
        assert_eq!(first_output, repeated_output);
    }

    #[test]
    fn offline_high_quality_dynamic_ratio_mono_sums_segment_targets() {
        let input = sine(440.0, 48_000.0, 48_000);
        let ratio_curve = [
            StretchRatioPoint::new(0, 0.75),
            StretchRatioPoint::new(16_000, 1.0),
            StretchRatioPoint::new(32_000, 1.5),
        ];
        let mut stretcher = OfflineHighQualityStretcher::new(1.0);
        let output = stretcher.stretch_dynamic_ratio_mono(&input, &ratio_curve);

        assert_eq!(
            output.len(),
            dynamic_ratio_output_frames(input.len(), &ratio_curve, 1.0)
        );
        assert_eq!(output.len(), 52_000);
    }

    #[test]
    fn offline_high_quality_dynamic_ratio_ignores_invalid_points() {
        let input = sine(440.0, 48_000.0, 8_000);
        let ratio_curve = [
            StretchRatioPoint::new(-128, 0.5),
            StretchRatioPoint::new(2_000, f64::NAN),
            StretchRatioPoint::new(4_000, -2.0),
        ];
        let mut dynamic = OfflineHighQualityStretcher::new(1.25);
        let mut fixed = OfflineHighQualityStretcher::new(1.25);

        assert_eq!(
            dynamic.stretch_dynamic_ratio_mono(&input, &ratio_curve),
            fixed.stretch_mono(&input)
        );
    }

    #[test]
    fn offline_high_quality_dynamic_ratio_stereo_is_exact_and_deterministic() {
        let sample_rate = 48_000.0;
        let left = sine(220.0, sample_rate, 48_000);
        let right = sine(440.0, sample_rate, 48_000);
        let mut frames = Vec::with_capacity(left.len() * 2);
        for (l, r) in left.iter().zip(right.iter()) {
            frames.push(*l);
            frames.push(*r);
        }
        let ratio_curve = [
            StretchRatioPoint::new(0, 0.75),
            StretchRatioPoint::new(16_000, 1.0),
            StretchRatioPoint::new(32_000, 1.5),
        ];
        let mut first = OfflineHighQualityStretcher::new(1.0);
        let mut repeated = OfflineHighQualityStretcher::new(1.0);
        let first_output = first.stretch_dynamic_ratio_interleaved_stereo(&frames, &ratio_curve);
        let repeated_output =
            repeated.stretch_dynamic_ratio_interleaved_stereo(&frames, &ratio_curve);

        assert_eq!(
            first_output.len(),
            dynamic_ratio_output_frames(left.len(), &ratio_curve, 1.0) * 2
        );
        assert_eq!(first_output, repeated_output);
    }

    #[test]
    fn offline_high_quality_dynamic_ratio_smoothing_reduces_segment_seams() {
        let sample_rate = 48_000.0;
        let left = sine(220.0, sample_rate, 48_000);
        let right = sine(440.0, sample_rate, 48_000);
        let mut frames = Vec::with_capacity(left.len() * 2);
        for (l, r) in left.iter().zip(right.iter()) {
            frames.push(*l);
            frames.push(*r);
        }
        let ratio_curve = [
            StretchRatioPoint::new(0, 0.75),
            StretchRatioPoint::new(16_000, 1.0),
            StretchRatioPoint::new(32_000, 1.5),
        ];
        let boundaries = dynamic_ratio_output_boundaries(left.len(), &ratio_curve, 1.0);
        let mut raw = frames.clone();
        let before = measure_dynamic_segment_seam_click(&raw, 2, &boundaries, 1.0);
        smooth_dynamic_segment_boundaries_interleaved(&mut raw, 2, &boundaries, 64);
        let after = measure_dynamic_segment_seam_click(&raw, 2, &boundaries, 1.0);

        assert!(after.peak_seam_delta < before.peak_seam_delta);
        assert_eq!(raw.len(), frames.len());
    }

    #[test]
    fn offline_high_quality_dynamic_ratio_pitch_stereo_is_exact_and_deterministic() {
        let sample_rate = 48_000.0;
        let left = sine(220.0, sample_rate, 48_000);
        let right = sine(440.0, sample_rate, 48_000);
        let mut frames = Vec::with_capacity(left.len() * 2);
        for (l, r) in left.iter().zip(right.iter()) {
            frames.push(*l);
            frames.push(*r);
        }
        let ratio_curve = [
            StretchRatioPoint::new(0, 0.75),
            StretchRatioPoint::new(16_000, 1.0),
            StretchRatioPoint::new(32_000, 1.5),
        ];
        let mut first = OfflineHighQualityStretcher::new(1.0);
        let mut repeated = OfflineHighQualityStretcher::new(1.0);
        let first_output = first.stretch_dynamic_ratio_pitch_interleaved_stereo(
            &frames,
            &ratio_curve,
            SampleRate(48_000),
            2.0,
        );
        let repeated_output = repeated.stretch_dynamic_ratio_pitch_interleaved_stereo(
            &frames,
            &ratio_curve,
            SampleRate(48_000),
            2.0,
        );

        assert_eq!(
            first_output.len(),
            dynamic_ratio_output_frames(left.len(), &ratio_curve, 1.0) * 2
        );
        assert_eq!(first_output, repeated_output);
    }

    #[test]
    fn backend_plan_tracks_signal_owned_tiers() {
        assert_eq!(SIGNAL_STRETCH_BACKEND_PLAN.len(), 3);
        assert_eq!(
            stretch_backend_plan(StretchBackendTier::Repitch).status,
            StretchBackendStatus::Implemented
        );
        let preview = stretch_backend_plan(StretchBackendTier::RealtimePreview);
        assert_eq!(preview.status, StretchBackendStatus::Prototype);
        assert!(preview.independent_tempo_and_pitch);
        assert!(preview.dynamic_ratio);
        assert!(!preview.audio_thread_safe);

        let offline = stretch_backend_plan(StretchBackendTier::OfflineHighQuality);
        assert_eq!(offline.status, StretchBackendStatus::Implemented);
        assert!(offline.transient_preservation);
        assert!(offline.vertical_phase_coherence);
        assert!(offline.deterministic_output);
    }

    #[test]
    fn benchmark_corpus_covers_required_material_families() {
        let required = [
            StretchCorpusFamily::DrumsPercussion,
            StretchCorpusFamily::Bass,
            StretchCorpusFamily::Vocals,
            StretchCorpusFamily::PadsSustains,
            StretchCorpusFamily::FullMix,
            StretchCorpusFamily::TempoRamp,
            StretchCorpusFamily::LoopSeam,
            StretchCorpusFamily::ExtremeRatio,
        ];

        for family in required {
            assert!(
                STRETCH_BENCHMARK_CORPUS
                    .iter()
                    .any(|case| case.family == family),
                "missing corpus family {family:?}"
            );
        }
        assert!(STRETCH_BENCHMARK_CORPUS.iter().all(|case| case
            .ratios
            .iter()
            .all(|ratio| ratio.is_finite() && *ratio > 0.0)));
    }

    #[test]
    fn real_corpus_manifest_covers_required_families_and_source_policy() {
        assert_eq!(STRETCH_CORPUS_MANIFEST.manifest_id, "stretch-corpus-v1");
        assert_eq!(STRETCH_CORPUS_MANIFEST.schema_version, 1);
        assert_eq!(STRETCH_CORPUS_MANIFEST.sample_rate_hz, 48_000);
        assert_eq!(STRETCH_CORPUS_MANIFEST.channels, 2);
        assert_eq!(
            STRETCH_CORPUS_MANIFEST.source_policy,
            STRETCH_CORPUS_SOURCE_POLICY
        );
        assert_eq!(
            STRETCH_CORPUS_MANIFEST.entries.len(),
            STRETCH_BENCHMARK_CORPUS.len()
        );

        for benchmark_case in STRETCH_BENCHMARK_CORPUS {
            let manifest_entry = STRETCH_CORPUS_MANIFEST
                .entries
                .iter()
                .find(|entry| entry.case.case_id == benchmark_case.case_id)
                .expect("benchmark case should have manifest entry");
            assert_eq!(manifest_entry.case.family, benchmark_case.family);
            assert_eq!(manifest_entry.case.ratios, benchmark_case.ratios);
            assert!(!manifest_entry.source_path_hint.is_empty());
            assert!(!manifest_entry.provenance_note.is_empty());
        }
    }

    #[test]
    fn real_corpus_manifest_keeps_licensed_audio_out_of_repo() {
        for entry in STRETCH_CORPUS_MANIFEST.entries {
            match entry.case.source {
                StretchCorpusSource::Synthetic => {
                    assert_eq!(
                        entry.asset_requirement,
                        StretchCorpusAssetRequirement::InlineSynthetic
                    );
                    assert_eq!(
                        entry.missing_asset_behavior,
                        StretchCorpusMissingAssetBehavior::GenerateInlineSynthetic
                    );
                    assert!(entry.source_path_hint.starts_with("inline:"));
                    assert!(generate_synthetic_stretch_audio(entry.case.family).is_some());
                }
                StretchCorpusSource::LicensedListening => {
                    assert_eq!(
                        entry.asset_requirement,
                        StretchCorpusAssetRequirement::OperatorProvidedAudio
                    );
                    assert_eq!(
                        entry.missing_asset_behavior,
                        StretchCorpusMissingAssetBehavior::ReportMissingAndSkipCase
                    );
                    assert!(entry
                        .source_path_hint
                        .starts_with("fixtures/stretch-corpus/licensed-listening/"));
                    assert!(entry.provenance_note.contains("licensed"));
                }
                StretchCorpusSource::ExternalBenchmark => {
                    assert_eq!(
                        entry.asset_requirement,
                        StretchCorpusAssetRequirement::OptionalExternalBenchmark
                    );
                    assert_eq!(
                        entry.missing_asset_behavior,
                        StretchCorpusMissingAssetBehavior::SkipOptionalBenchmark
                    );
                }
                StretchCorpusSource::LocalFixture => {
                    panic!("stretch corpus v1 must not rely on checked-in licensed fixtures");
                }
            }
        }
        assert!(STRETCH_CORPUS_SOURCE_POLICY
            .licensed_audio_policy
            .contains("do not commit source audio"));
    }

    #[test]
    fn output_length_drift_tracks_fixed_ratio_contract() {
        assert_eq!(output_length_drift_samples(1_000, 1_500, 1.5), 0.0);
        assert_eq!(output_length_drift_samples(1_001, 1_502, 1.5), 0.0);
        assert_eq!(output_length_drift_samples(1_001, 1_503, 1.5), 1.0);
        assert!(output_length_drift_samples(1_000, 1_000, f64::NAN).is_nan());
    }

    #[test]
    fn metric_assessment_aggregates_warnings_and_failures() {
        let measurements = [
            StretchMetricValue::new(StretchMetric::TimingDriftSamples, 0.0),
            StretchMetricValue::new(StretchMetric::StereoImageDelta, 0.2),
            StretchMetricValue::new(StretchMetric::LoopBoundaryClickDbfs, -24.0),
        ];
        let limits = [
            StretchMetricLimit::max(
                StretchMetric::TimingDriftSamples,
                1.0,
                StretchAcceptanceSeverity::Fail,
            ),
            StretchMetricLimit::max(
                StretchMetric::StereoImageDelta,
                0.1,
                StretchAcceptanceSeverity::Warn,
            ),
            StretchMetricLimit::max(
                StretchMetric::LoopBoundaryClickDbfs,
                -60.0,
                StretchAcceptanceSeverity::Fail,
            ),
        ];

        let report = assess_stretch_metrics(&measurements, &limits);

        assert_eq!(report.status, StretchAcceptanceStatus::Fail);
        assert_eq!(report.metrics[0].status, StretchAcceptanceStatus::Pass);
        assert_eq!(report.metrics[1].status, StretchAcceptanceStatus::Warn);
        assert_eq!(report.metrics[2].status, StretchAcceptanceStatus::Fail);
    }

    #[test]
    fn dynamic_segment_seam_metric_reports_direct_discontinuity() {
        let frames = [0.0, 0.0, 0.1, 0.2, 0.9, -0.4, 1.0, -0.3];
        let measurement = measure_dynamic_segment_seam_click(&frames, 2, &[2], 1.0);

        assert_eq!(measurement.ratio, 1.0);
        assert_eq!(measurement.channels, 2);
        assert_eq!(measurement.seam_frames, vec![2]);
        assert!((measurement.peak_seam_delta - 0.8).abs() < 1.0e-6);
        assert!((measurement.click_dbfs - (20.0f64 * 0.8f64.log10())).abs() < 1.0e-6);
        assert_eq!(
            measurement.metric.metric,
            StretchMetric::DynamicSegmentSeamClickDbfs
        );
        assert_eq!(measurement.metric.value, measurement.click_dbfs);
    }

    #[test]
    fn pitch_shift_metric_reports_dominant_frequency_error() {
        let sample_rate_hz = 48_000;
        let sample_rate = sample_rate_hz as f32;
        let input = sine(440.0, sample_rate, sample_rate_hz as usize);
        let mut stretcher = OfflineHighQualityStretcher::new(1.0);
        let output = stretcher.stretch_pitch_mono(&input, SampleRate(sample_rate_hz), 12.0);
        let measurement =
            measure_pitch_shift_error_cents(&output, sample_rate_hz, 440.0, 12.0, 1.0);

        assert_eq!(measurement.ratio, 1.0);
        assert_eq!(measurement.pitch_shift_semitones, 12.0);
        assert!((measurement.expected_frequency_hz - 880.0).abs() < 1.0e-6);
        assert!(measurement.measured_frequency_hz > 850.0);
        assert!(measurement.measured_frequency_hz < 910.0);
        assert!(measurement.pitch_error_cents < 75.0);
        assert_eq!(measurement.metric.metric, StretchMetric::PitchErrorCents);
        assert_eq!(measurement.metric.value, measurement.pitch_error_cents);
    }

    #[test]
    fn synthetic_corpus_cases_run_without_file_io() {
        let cases = synthetic_stretch_corpus_cases();
        assert_eq!(cases.len(), 3);
        for (case, audio) in cases {
            assert_eq!(case.source, StretchCorpusSource::Synthetic);
            assert!(audio.sample_rate_hz > 0);
            assert!(audio.channels > 0);
            assert_eq!(audio.samples.len() % audio.channels as usize, 0);
            assert!(audio.samples.iter().any(|sample| sample.abs() > 0.01));
        }
    }

    #[test]
    fn synthetic_backend_comparison_covers_all_synthetic_cases() {
        let report = compare_synthetic_stretch_backends();

        assert_eq!(report.comparisons.len(), 27);
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
                StretchBenchmarkBackend::OfflineHighQualityPrototype
            );
            assert!(comparison.ratio.is_finite());
            assert!(comparison.ratio > 0.0);
            assert!(matches!(
                comparison.case_id,
                "stretch:tempo_ramp"
                    | "stretch:loop_seam"
                    | "stretch:extreme_ratio"
                    | "stretch:pitch_shift"
                    | "stretch:sustained_coherence"
            ));
        }
        assert!(report.comparisons.iter().any(|comparison| {
            comparison.case_id == "stretch:tempo_ramp"
                && comparison.metric == StretchMetric::TimingDriftSamples
                && comparison.ratio > 1.0
                && comparison.path == StretchBenchmarkPath::DynamicRatio
        }));
        assert!(report.comparisons.iter().any(|comparison| {
            comparison.case_id == "stretch:tempo_ramp"
                && comparison.metric == StretchMetric::DynamicSegmentSeamClickDbfs
                && comparison.ratio > 1.0
                && comparison.path == StretchBenchmarkPath::DynamicRatio
        }));
        assert!(report.comparisons.iter().any(|comparison| {
            comparison.case_id == "stretch:loop_seam"
                && comparison.metric == StretchMetric::LoopBoundaryClickDbfs
                && comparison.path == StretchBenchmarkPath::FixedRatio
        }));
        assert!(report.comparisons.iter().any(|comparison| {
            comparison.case_id == "stretch:loop_seam"
                && comparison.metric == StretchMetric::StereoImageDelta
                && comparison.path == StretchBenchmarkPath::LinkedStereo
        }));
        assert!(report.comparisons.iter().any(|comparison| {
            comparison.case_id == "stretch:extreme_ratio"
                && comparison.metric == StretchMetric::TransientSmearFrames
                && comparison.path == StretchBenchmarkPath::FixedRatio
        }));
        assert!(report.comparisons.iter().any(|comparison| {
            comparison.case_id == "stretch:extreme_ratio"
                && comparison.metric == StretchMetric::TransientSmearFrames
                && comparison.path == StretchBenchmarkPath::FixedRatio
                && comparison.ratio == 2.0
                && comparison.delta == 1.0
                && comparison.outcome == StretchBenchmarkComparisonOutcome::Unchanged
        }));
        assert!(report.comparisons.iter().any(|comparison| {
            comparison.case_id == "stretch:pitch_shift"
                && comparison.metric == StretchMetric::PitchErrorCents
                && comparison.path == StretchBenchmarkPath::PitchShift
                && comparison.pitch_shift_semitones == Some(12.0)
        }));
        assert!(report.comparisons.iter().any(|comparison| {
            comparison.case_id == "stretch:sustained_coherence"
                && comparison.metric == StretchMetric::VerticalCoherenceDelta
                && comparison.path == StretchBenchmarkPath::PhaseLocked
        }));
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

    #[test]
    fn synthetic_backend_comparison_report_formats_deterministically() {
        let report = compare_synthetic_stretch_backends();
        let formatted = format_synthetic_stretch_comparison_report(&report);
        let repeated = format_synthetic_stretch_comparison_report(&report);

        assert_eq!(formatted, repeated);
        assert!(formatted.starts_with("synthetic_stretch_comparison improved="));
        assert!(formatted.contains("case=stretch:tempo_ramp"));
        assert!(formatted.contains("path=DynamicRatio"));
        assert!(formatted.contains("path=PhaseLocked"));
        assert!(formatted.contains("path=LinkedStereo"));
        assert!(formatted.contains("path=PitchShift"));
        assert!(formatted.contains("pitch_shift=12.000000"));
        assert!(formatted.contains("metric=TimingDriftSamples"));
        assert!(formatted.contains("metric=DynamicSegmentSeamClickDbfs"));
        assert!(formatted.contains("metric=VerticalCoherenceDelta"));
        assert!(formatted.contains("metric=PitchErrorCents"));
        assert!(formatted.contains("candidate_backend=OfflineHighQualityPrototype"));
        assert!(formatted.contains("candidate="));
        assert!(formatted.contains("outcome="));
    }

    #[test]
    fn stretch_corpus_comparison_report_covers_manifest_and_note_slots() {
        let report =
            build_stretch_corpus_comparison_report("stretch-corpus-v1-local", "projection:unit");

        assert_eq!(report.report_name, "stretch-corpus-v1-local");
        assert_eq!(report.projection_epoch, "projection:unit");
        assert_eq!(report.manifest.manifest_id, "stretch-corpus-v1");
        assert_eq!(report.engine_version, SIGNAL_STRETCH_ENGINE_VERSION);
        assert_eq!(report.missing_assets.len(), 5);
        assert_eq!(report.optional_benchmark_skips.len(), 0);
        assert_eq!(report.synthetic_report.comparisons.len(), 27);
        assert_eq!(
            report.listening_note_slots.len(),
            report.missing_assets.len() + report.synthetic_report.comparisons.len()
        );
        assert!(report
            .missing_assets
            .iter()
            .all(|asset| asset.missing_asset_behavior
                == StretchCorpusMissingAssetBehavior::ReportMissingAndSkipCase));
        assert!(report.listening_note_slots.iter().any(|slot| slot.case_id
            == "stretch:drums_percussion"
            && slot.ratio.is_none()
            && slot
                .source_path_hint
                .starts_with("fixtures/stretch-corpus/licensed-listening/")));
        assert!(report.listening_note_slots.iter().any(|slot| {
            slot.case_id == "stretch:pitch_shift"
                && slot.pitch_shift_semitones == Some(12.0)
                && slot.source_path_hint == "inline:pitch-shift-tone"
        }));
    }

    #[test]
    fn stretch_corpus_comparison_report_formats_deterministically() {
        let report =
            build_stretch_corpus_comparison_report("stretch-corpus-v1-local", "projection:unit");
        let formatted = format_stretch_corpus_comparison_report(&report);
        let repeated = format_stretch_corpus_comparison_report(&report);

        assert_eq!(formatted, repeated);
        assert!(formatted.starts_with(
            "stretch_corpus_report name=\"stretch-corpus-v1-local\" corpus=stretch-corpus-v1"
        ));
        assert!(formatted.contains("engine=signal-native-stretch-v1"));
        assert!(formatted.contains("projection_epoch=\"projection:unit\""));
        assert!(formatted.contains("source_policy synthetic="));
        assert!(formatted.contains(
            "summary comparisons=27 external_benchmark_comparisons=0 operator_listening_sources=0 missing_assets=5"
        ));
        assert!(formatted.contains("asset case=stretch:drums_percussion status=missing_required"));
        assert!(formatted.contains("comparison case=stretch:tempo_ramp"));
        assert!(formatted.contains("ratio_curve=synthetic_tempo_ramp:"));
        assert!(formatted.contains("pitch_curve=constant:12.000000"));
        assert!(formatted.contains("metric=DynamicSegmentSeamClickDbfs"));
        assert!(formatted.contains("listening_note case=stretch:pitch_shift"));
        assert!(formatted.contains(
            "prompt=\"operator-note: record audible artifacts beside objective metrics\""
        ));
    }

    #[test]
    fn stretch_corpus_report_accepts_operator_listening_sources() {
        let report = build_stretch_corpus_comparison_report_with_sources(
            "stretch-corpus-v1-local",
            "projection:unit",
            &[],
            &[StretchCorpusListeningSource {
                case_id: "stretch:vocals".to_string(),
                source_path: "/Users/tom/Downloads/FMA/fma_large/000/000010.mp3".to_string(),
                source_label: "Kurt Vile - Freeway".to_string(),
                license_title: "Attribution-NonCommercial-NoDerivatives".to_string(),
                license_url: "https://example.test/license".to_string(),
                provenance_url: "https://example.test/track".to_string(),
            }],
        );

        assert_eq!(report.operator_listening_sources.len(), 1);
        assert_eq!(report.missing_assets.len(), 4);
        assert!(report
            .missing_assets
            .iter()
            .all(|asset| asset.case.case_id != "stretch:vocals"));
        assert!(report
            .listening_note_slots
            .iter()
            .any(|slot| slot.case_id == "stretch:vocals"
                && slot.source_path_hint == "/Users/tom/Downloads/FMA/fma_large/000/000010.mp3"
                && slot.prompt
                    == "operator-note: record real-source listening artifacts before promotion"));

        let formatted = format_stretch_corpus_comparison_report(&report);

        assert!(formatted.contains("operator_listening_sources=1 missing_assets=4"));
        assert!(formatted.contains("operator_listening_source case=stretch:vocals"));
        assert!(formatted.contains("label=\"Kurt Vile - Freeway\""));
        assert!(formatted.contains(
            "source_boundary=\"operator-provided licensed local audio; no source audio committed\""
        ));
    }

    #[test]
    fn stretch_corpus_report_accepts_optional_external_benchmark_render() {
        let loop_frames = generate_synthetic_stretch_audio(StretchCorpusFamily::LoopSeam)
            .expect("loop seam synthetic exists")
            .frame_count();
        let report = build_stretch_corpus_comparison_report_with_external(
            "stretch-corpus-v1-local",
            "projection:unit",
            &[StretchExternalBenchmarkRender {
                case_id: "stretch:loop_seam".to_string(),
                ratio: 1.0,
                pitch_shift_semitones: None,
                tool_name: "rubberband-cli".to_string(),
                rendered_path: "fixtures/stretch-corpus/external-benchmark/loop.wav".to_string(),
                rendered_frames: loop_frames + 2,
                sample_rate_hz: 48_000,
                channels: 2,
            }],
        );
        let comparison = &report.external_benchmark_comparisons[0];

        assert_eq!(comparison.case_id, "stretch:loop_seam");
        assert_eq!(comparison.tool_name, "rubberband-cli");
        assert_eq!(comparison.expected_frames, Some(loop_frames));
        assert_eq!(comparison.timing_drift_samples, Some(2.0));
        assert_eq!(
            comparison.source_boundary,
            "rendered-output-only; no external source or library dependency"
        );

        let formatted = format_stretch_corpus_comparison_report(&report);
        assert!(formatted.contains("external_benchmark case=stretch:loop_seam"));
        assert!(formatted.contains("tool=\"rubberband-cli\""));
        assert!(formatted.contains(
            "source_boundary=\"rendered-output-only; no external source or library dependency\""
        ));
        assert!(formatted.contains("timing_drift_samples=2.000000"));
    }

    #[test]
    fn stretch_corpus_report_keeps_unknown_external_benchmark_metadata_only() {
        let report = build_stretch_corpus_comparison_report_with_external(
            "stretch-corpus-v1-local",
            "projection:unit",
            &[StretchExternalBenchmarkRender {
                case_id: "stretch:licensed-only".to_string(),
                ratio: 1.25,
                pitch_shift_semitones: None,
                tool_name: "rubberband-cli".to_string(),
                rendered_path: "fixtures/stretch-corpus/external-benchmark/licensed.wav"
                    .to_string(),
                rendered_frames: 60_000,
                sample_rate_hz: 48_000,
                channels: 2,
            }],
        );
        let comparison = &report.external_benchmark_comparisons[0];

        assert_eq!(comparison.expected_frames, None);
        assert_eq!(comparison.timing_drift_samples, None);
        assert_eq!(comparison.rendered_frames, 60_000);
        assert_eq!(comparison.sample_rate_hz, 48_000);
        assert_eq!(comparison.channels, 2);
    }

    #[test]
    fn stretch_quality_priorities_are_regression_only_and_sorted() {
        let report = compare_synthetic_stretch_backends();
        let priorities = prioritize_stretch_quality_work(&report, 8);
        let formatted = format_stretch_quality_priority_report(&priorities);

        assert!(priorities.is_empty());
        for priority in &priorities {
            assert!(matches!(
                priority.outcome,
                StretchBenchmarkComparisonOutcome::Regressed
                    | StretchBenchmarkComparisonOutcome::Inconclusive
            ));
            assert!(priority.priority_score.is_finite());
            assert!(priority.priority_score > 0.0);
        }
        for pair in priorities.windows(2) {
            assert!(pair[0].priority_score >= pair[1].priority_score);
        }
        assert!(formatted.starts_with("stretch_quality_priorities count="));
        assert_eq!(formatted, "stretch_quality_priorities count=0");
    }

    #[test]
    fn acceptance_report_format_is_deterministic() {
        let report = assess_stretch_metrics(
            &[StretchMetricValue::new(
                StretchMetric::TimingDriftSamples,
                0.0,
            )],
            &[StretchMetricLimit::max(
                StretchMetric::TimingDriftSamples,
                1.0,
                StretchAcceptanceSeverity::Fail,
            )],
        );

        assert_eq!(
            format_stretch_acceptance_report("stretch:tempo_ramp", &report),
            "case=stretch:tempo_ramp status=Pass\nmetric=TimingDriftSamples value=0.000000 max=1.000000 status=Pass"
        );
    }

    #[test]
    fn sustained_material_coherence_comparison_logs_measured_gap() {
        let comparison = compare_sustained_material_coherence(1.5);

        assert_eq!(comparison.ratio, 1.5);
        assert!(comparison.draft_vertical_coherence_score.is_finite());
        assert!(comparison.phase_locked_vertical_coherence_score.is_finite());
        assert_eq!(
            comparison.metric.metric,
            StretchMetric::VerticalCoherenceDelta
        );
        assert!(
            (comparison.metric.value
                - (comparison.phase_locked_vertical_coherence_score
                    - comparison.draft_vertical_coherence_score))
                .abs()
                < 1.0e-12
        );
    }

    #[test]
    fn sustained_material_coherence_gap_formats_as_acceptance_metric() {
        let comparison = compare_sustained_material_coherence(1.25);
        let report = assess_stretch_metrics(
            &[comparison.metric],
            &[StretchMetricLimit::max(
                StretchMetric::VerticalCoherenceDelta,
                f64::INFINITY,
                StretchAcceptanceSeverity::Warn,
            )],
        );
        let formatted = format_stretch_acceptance_report("stretch:pads_sustains", &report);

        assert_eq!(report.status, StretchAcceptanceStatus::Pass);
        assert!(formatted.contains("metric=VerticalCoherenceDelta"));
        assert!(formatted.contains("status=Pass"));
    }

    #[test]
    fn transient_detector_finds_synthetic_attack_frames() {
        let audio = generate_synthetic_stretch_audio(StretchCorpusFamily::ExtremeRatio)
            .expect("extreme-ratio synthetic audio exists");
        let events = detect_stretch_transients(&audio.samples, 1024, 256);

        assert!(
            events.len() >= 10,
            "expected repeated synthetic attacks, got {events:?}"
        );
        for expected in [8_000usize, 16_000, 24_000, 32_000, 40_000] {
            assert!(
                events
                    .iter()
                    .any(|event| event.frame_index.abs_diff(expected) <= 768),
                "missing transient near frame {expected}, got {events:?}"
            );
        }
        assert!(events.iter().all(|event| event.energy_score.is_finite()
            && event.spectral_flux_score.is_finite()
            && event.combined_score.is_finite()));
    }

    #[test]
    fn transient_detector_default_policy_matches_production_entry_point() {
        let audio = generate_synthetic_stretch_audio(StretchCorpusFamily::ExtremeRatio)
            .expect("extreme-ratio synthetic audio exists");

        assert_eq!(
            detect_stretch_transients(&audio.samples, 1024, 256),
            detect_stretch_transients_with_policy(
                &audio.samples,
                1024,
                256,
                StretchTransientDetectorPolicy::production()
            )
        );
    }

    #[test]
    fn candidate_transient_detector_recovers_masked_soft_attack() {
        let input = masked_soft_attack_probe(0.25);
        let production = detect_stretch_transients_with_policy(
            &input,
            1024,
            256,
            StretchTransientDetectorPolicy::production(),
        );
        let candidate = detect_stretch_transients_with_policy(
            &input,
            1024,
            256,
            StretchTransientDetectorPolicy::candidate_review(),
        );

        assert!(
            production
                .iter()
                .all(|event| event.frame_index.abs_diff(24_000) > 768),
            "production policy should miss the softened probe attack: {production:?}"
        );
        assert!(
            candidate
                .iter()
                .any(|event| event.frame_index.abs_diff(24_000) <= 768),
            "candidate policy should recover the softened probe attack: {candidate:?}"
        );
    }

    #[test]
    fn transient_detector_stays_quiet_on_plain_sustain() {
        let input = sine(440.0, 48_000.0, 48_000);
        let events = detect_stretch_transients(&input, 1024, 256);

        assert!(
            events.len() <= 1,
            "plain sustain should not generate repeated transient events: {events:?}"
        );
    }

    #[test]
    fn candidate_transient_detector_stays_quiet_on_plain_sustain() {
        let input = sine(440.0, 48_000.0, 48_000);
        let events = detect_stretch_transients_with_policy(
            &input,
            1024,
            256,
            StretchTransientDetectorPolicy::candidate_review(),
        );

        assert!(
            events.len() <= 1,
            "candidate policy should not generate repeated sustain events: {events:?}"
        );
    }

    #[test]
    fn transient_smear_metric_reports_synthetic_draft_case() {
        let measurement = measure_draft_transient_smear(1.5);

        assert_eq!(measurement.ratio, 1.5);
        assert!(measurement.input_transients >= 10);
        assert!(measurement.output_transients > 0);
        assert!(measurement.matched_transients > 0);
        assert_eq!(
            measurement.input_transients,
            measurement.matched_transients + measurement.missed_transients
        );
        assert!(measurement.mean_smear_frames.is_finite());
        assert!(measurement.max_smear_frames.is_finite());
        assert_eq!(
            measurement.metric.metric,
            StretchMetric::TransientSmearFrames
        );
        assert_eq!(measurement.metric.value, measurement.max_smear_frames);
    }

    #[test]
    fn transient_reset_smear_metric_reports_synthetic_case() {
        let draft = measure_draft_transient_smear(1.5);
        let reset = measure_transient_reset_transient_smear(1.5);

        assert_eq!(reset.ratio, 1.5);
        assert_eq!(reset.input_transients, draft.input_transients);
        assert!(reset.output_transients > 0);
        assert!(reset.matched_transients > 0);
        assert_eq!(
            reset.input_transients,
            reset.matched_transients + reset.missed_transients
        );
        assert!(reset.max_smear_frames.is_finite());
        assert_eq!(reset.metric.metric, StretchMetric::TransientSmearFrames);
    }

    #[test]
    fn transient_smear_metric_penalizes_missing_matches() {
        let mut input = vec![0.0; 64];
        input[20] = 1.0;
        input[21] = 0.5;
        input[22] = 0.25;
        let output = vec![0.0; 64];
        let measurement = measure_transient_smear(&input, &output, 1.0, 16, 4);

        assert!(measurement.input_transients > 0);
        assert_eq!(measurement.output_transients, 0);
        assert_eq!(measurement.matched_transients, 0);
        assert_eq!(measurement.missed_transients, measurement.input_transients);
        assert_eq!(measurement.mean_smear_frames, 16.0);
        assert_eq!(measurement.max_smear_frames, 16.0);
        assert_eq!(
            measurement.metric.metric,
            StretchMetric::TransientSmearFrames
        );
        assert_eq!(measurement.metric.value, 16.0);
    }

    #[test]
    fn transient_smear_entry_point_uses_promoted_output_recovery_policy() {
        let input = masked_soft_attack_probe(1.0);
        let output = masked_soft_attack_probe(0.25);
        let promoted = measure_transient_smear(&input, &output, 1.0, 1024, 256);
        let strict = measure_transient_smear_with_policy(
            &input,
            &output,
            1.0,
            1024,
            256,
            StretchTransientDetectorPolicy::production(),
        );
        let recovery = measure_transient_smear_with_output_recovery_policy(
            &input,
            &output,
            1.0,
            1024,
            256,
            StretchTransientDetectorPolicy::production(),
            StretchTransientDetectorPolicy::production(),
            StretchTransientDetectorPolicy::candidate_review(),
        );

        assert_eq!(promoted, recovery);
        assert!(promoted.matched_transients > strict.matched_transients);
        assert!(promoted.missed_transients < strict.missed_transients);
    }

    #[test]
    fn candidate_transient_smear_counts_masked_soft_attack() {
        let input = masked_soft_attack_probe(0.25);
        let production = measure_transient_smear_with_policy(
            &input,
            &input,
            1.0,
            1024,
            256,
            StretchTransientDetectorPolicy::production(),
        );
        let candidate = measure_transient_smear_with_policy(
            &input,
            &input,
            1.0,
            1024,
            256,
            StretchTransientDetectorPolicy::candidate_review(),
        );

        assert!(candidate.input_transients > production.input_transients);
        assert!(candidate.matched_transients > production.matched_transients);
        assert_eq!(candidate.missed_transients, 0);
        assert_eq!(candidate.max_smear_frames, 0.0);
    }

    #[test]
    fn candidate_output_policy_recovers_production_input_match() {
        let input = masked_soft_attack_probe(1.0);
        let output = masked_soft_attack_probe(0.25);
        let production = measure_transient_smear_with_policies(
            &input,
            &output,
            1.0,
            1024,
            256,
            StretchTransientDetectorPolicy::production(),
            StretchTransientDetectorPolicy::production(),
        );
        let candidate_output = measure_transient_smear_with_policies(
            &input,
            &output,
            1.0,
            1024,
            256,
            StretchTransientDetectorPolicy::production(),
            StretchTransientDetectorPolicy::candidate_review(),
        );

        assert_eq!(
            candidate_output.input_transients,
            production.input_transients
        );
        assert!(candidate_output.matched_transients > production.matched_transients);
        assert!(candidate_output.missed_transients < production.missed_transients);
    }

    #[test]
    fn output_recovery_policy_keeps_primary_matches_before_candidate_recovery() {
        let input = masked_soft_attack_probe(1.0);
        let output = masked_soft_attack_probe(0.25);
        let production = measure_transient_smear_with_policies(
            &input,
            &output,
            1.0,
            1024,
            256,
            StretchTransientDetectorPolicy::production(),
            StretchTransientDetectorPolicy::production(),
        );
        let recovery = measure_transient_smear_with_output_recovery_policy(
            &input,
            &output,
            1.0,
            1024,
            256,
            StretchTransientDetectorPolicy::production(),
            StretchTransientDetectorPolicy::production(),
            StretchTransientDetectorPolicy::candidate_review(),
        );

        assert_eq!(recovery.input_transients, production.input_transients);
        assert_eq!(recovery.output_transients, production.output_transients);
        assert!(recovery.matched_transients > production.matched_transients);
        assert!(recovery.missed_transients < production.missed_transients);
        assert!(recovery.max_smear_frames <= production.max_smear_frames);
    }

    #[test]
    fn transient_smear_metric_formats_as_acceptance_metric() {
        let measurement = measure_draft_transient_smear(1.25);
        let report = assess_stretch_metrics(
            &[measurement.metric],
            &[StretchMetricLimit::max(
                StretchMetric::TransientSmearFrames,
                f64::INFINITY,
                StretchAcceptanceSeverity::Warn,
            )],
        );
        let formatted = format_stretch_acceptance_report("stretch:extreme_ratio", &report);

        assert_eq!(report.status, StretchAcceptanceStatus::Pass);
        assert!(formatted.contains("metric=TransientSmearFrames"));
        assert!(formatted.contains("status=Pass"));
    }

    #[test]
    fn loop_boundary_metric_reports_direct_discontinuity() {
        let frames = [0.1, -0.2, 0.3, 0.1];
        let measurement = measure_loop_boundary_click(&frames, 2, 1.0);

        assert_eq!(measurement.ratio, 1.0);
        assert_eq!(measurement.channels, 2);
        assert!((measurement.peak_boundary_delta - 0.3).abs() < 1.0e-6);
        assert!((measurement.click_dbfs - (20.0f64 * 0.3f64.log10())).abs() < 1.0e-6);
        assert_eq!(
            measurement.metric.metric,
            StretchMetric::LoopBoundaryClickDbfs
        );
        assert_eq!(measurement.metric.value, measurement.click_dbfs);
    }

    #[test]
    fn loop_boundary_smoothing_equalizes_endpoints() {
        let mut frames = [1.0, -0.5, 0.25, 0.25, -1.0, 0.75];

        smooth_loop_boundary_interleaved(&mut frames, 2, 1);

        assert!((frames[0] - frames[4]).abs() < 1.0e-6);
        assert!((frames[1] - frames[5]).abs() < 1.0e-6);
        assert!((frames[2] - 0.25).abs() < 1.0e-6);
        assert!((frames[3] - 0.25).abs() < 1.0e-6);
    }

    #[test]
    fn dynamic_segment_boundary_smoothing_equalizes_join_edges() {
        let mut frames = [0.0, 0.0, 1.0, -1.0, -1.0, 1.0, 0.0, 0.0];

        smooth_dynamic_segment_boundaries_interleaved(&mut frames, 2, &[2], 1);

        assert!((frames[2] - frames[4]).abs() < 1.0e-6);
        assert!((frames[3] - frames[5]).abs() < 1.0e-6);
        assert_eq!(frames[0], 0.0);
        assert_eq!(frames[1], 0.0);
        assert_eq!(frames[6], 0.0);
        assert_eq!(frames[7], 0.0);
    }

    #[test]
    fn loop_boundary_metric_reports_synthetic_draft_case() {
        let measurement = measure_draft_loop_boundary_click(1.25);

        assert_eq!(measurement.ratio, 1.25);
        assert_eq!(measurement.channels, 2);
        assert!(measurement.peak_boundary_delta.is_finite());
        assert!(measurement.click_dbfs.is_finite());
        assert_eq!(
            measurement.metric.metric,
            StretchMetric::LoopBoundaryClickDbfs
        );
    }

    #[test]
    fn transient_reset_loop_boundary_metric_reports_synthetic_case() {
        let measurement = measure_transient_reset_loop_boundary_click(1.25);

        assert_eq!(measurement.ratio, 1.25);
        assert_eq!(measurement.channels, 2);
        assert!(measurement.peak_boundary_delta.is_finite());
        assert!(measurement.click_dbfs.is_finite());
        assert_eq!(
            measurement.metric.metric,
            StretchMetric::LoopBoundaryClickDbfs
        );
    }

    #[test]
    fn loop_boundary_metric_formats_as_acceptance_metric() {
        let measurement = measure_draft_loop_boundary_click(1.5);
        let report = assess_stretch_metrics(
            &[measurement.metric],
            &[StretchMetricLimit::max(
                StretchMetric::LoopBoundaryClickDbfs,
                f64::INFINITY,
                StretchAcceptanceSeverity::Warn,
            )],
        );
        let formatted = format_stretch_acceptance_report("stretch:loop_seam", &report);

        assert_eq!(report.status, StretchAcceptanceStatus::Pass);
        assert!(formatted.contains("metric=LoopBoundaryClickDbfs"));
        assert!(formatted.contains("status=Pass"));
    }

    #[test]
    fn stereo_image_metric_reports_direct_movement() {
        let input = [0.5, 0.5, 0.25, 0.25, -0.25, -0.25, -0.5, -0.5];
        let output = [0.5, -0.5, 0.25, -0.25, -0.25, 0.25, -0.5, 0.5];
        let measurement = measure_stereo_image_delta(&input, &output, 1.0);

        assert_eq!(measurement.ratio, 1.0);
        assert!(measurement.input_correlation > 0.99);
        assert!(measurement.output_correlation < -0.99);
        assert!(measurement.image_delta > 1.0);
        assert_eq!(measurement.metric.metric, StretchMetric::StereoImageDelta);
        assert_eq!(measurement.metric.value, measurement.image_delta);
    }

    #[test]
    fn stereo_image_metric_reports_synthetic_draft_case() {
        let measurement = measure_draft_stereo_image_delta(1.25);

        assert_eq!(measurement.ratio, 1.25);
        assert!(measurement.input_correlation.is_finite());
        assert!(measurement.output_correlation.is_finite());
        assert!(measurement.input_side_mid_ratio.is_finite());
        assert!(measurement.output_side_mid_ratio.is_finite());
        assert!(measurement.image_delta.is_finite());
        assert_eq!(measurement.metric.metric, StretchMetric::StereoImageDelta);
    }

    #[test]
    fn transient_reset_stereo_image_metric_reports_synthetic_case() {
        let measurement = measure_transient_reset_stereo_image_delta(1.25);

        assert_eq!(measurement.ratio, 1.25);
        assert!(measurement.input_correlation.is_finite());
        assert!(measurement.output_correlation.is_finite());
        assert!(measurement.image_delta.is_finite());
        assert_eq!(measurement.metric.metric, StretchMetric::StereoImageDelta);
    }

    #[test]
    fn stereo_image_metric_formats_as_acceptance_metric() {
        let measurement = measure_draft_stereo_image_delta(1.5);
        let report = assess_stretch_metrics(
            &[measurement.metric],
            &[StretchMetricLimit::max(
                StretchMetric::StereoImageDelta,
                f64::INFINITY,
                StretchAcceptanceSeverity::Warn,
            )],
        );
        let formatted = format_stretch_acceptance_report("stretch:full_mix", &report);

        assert_eq!(report.status, StretchAcceptanceStatus::Pass);
        assert!(formatted.contains("metric=StereoImageDelta"));
        assert!(formatted.contains("status=Pass"));
    }
}
