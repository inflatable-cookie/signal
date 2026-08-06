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
//! - [`StretchBackendTier::OfflineHighQuality`][]: deterministic
//!   export/cache/freeze stretch prototype, correctness- and promotion-gated.
//!
//! The current [`PhaseVocoderStretcher`] remains [`StretchQuality::Draft`]: a
//! plain Hann-windowed phase vocoder with NO phase locking and NO transient
//! preservation. [`OfflineHighQualityStretcher`] uses identity phase locking
//! and transient phase resets as the first clean-room Signal-owned foundation.
//! Product-facing use remains blocked unless an accepted
//! [`StretchPromotionReceipt`] is attached to the render/export/freeze
//! artifact plan. Rubber Band-class quality is the target for the planned
//! Signal-native tiers. Pinned public source may inform report-only architecture
//! research under Contract 082's provenance rule; external engines remain
//! comparators, not production dependencies.
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
//!
//! [`render_creative_stretch`] is a separate offline whole-buffer surface for
//! the admitted continuous-range [`CreativeStretchCharacter::Dream`] and
//! [`CreativeStretchCharacter::Cyclic`] effects. It is not a backend tier,
//! transparent fallback, cache route, or audio-thread API.

#![warn(missing_docs)]

mod artifact_plan;
#[cfg(any(test, feature = "evidence"))]
mod benchmark;
mod cache_identity;
#[cfg(any(test, feature = "evidence"))]
mod corpus_report;
mod creative;
mod creative_cyclic;
#[allow(clippy::manual_div_ceil, clippy::manual_is_multiple_of)]
#[cfg_attr(test, macro_use)]
mod creative_direct_renewal_dream;

#[cfg(test)]
direct_renewal_dream_tests!();
#[cfg(any(test, feature = "evidence"))]
mod formant_boundary;
mod phase_vocoder;
/// `g10.041` `A18` fix candidate, exposed for listening-pack rendering only.
///
/// Unadopted: no production path constructs it. Admission needs Contract `084`
/// Rule 5 listening.
#[cfg(any(test, feature = "evidence"))]
pub fn a18_candidate_stretch_mono(
    input: &[Sample],
    ratio: f64,
    crossover_fraction: f64,
) -> Vec<Sample> {
    let target_len = (input.len() as f64 * ratio).round() as usize;
    phase_vocoder::high_band_transient_reset_phase_vocoder(
        input,
        target_len,
        ratio,
        DEFAULT_WINDOW_SIZE,
        DEFAULT_ANALYSIS_HOP,
        crossover_fraction,
    )
}

/// The shipped transient-reset path at the same geometry, for A/B rendering.
#[cfg(any(test, feature = "evidence"))]
pub fn a18_shipped_stretch_mono(input: &[Sample], ratio: f64) -> Vec<Sample> {
    let target_len = (input.len() as f64 * ratio).round() as usize;
    phase_vocoder::transient_reset_phase_vocoder(
        input,
        target_len,
        ratio,
        DEFAULT_WINDOW_SIZE,
        DEFAULT_ANALYSIS_HOP,
    )
}
mod promotion;
mod realtime_preview;
mod realtime_preview_stream;
#[cfg(any(test, feature = "evidence"))]
mod render_integrity;
mod resumable;
#[cfg(any(test, feature = "evidence"))]
mod spectral_support;
#[cfg(any(test, feature = "evidence"))]
mod tonal_texture;
#[cfg(any(test, feature = "evidence"))]
mod transient_detail;
mod transient_smear;

pub use artifact_plan::{
    plan_offline_stretch_chunks, StretchOfflineChunk, StretchOfflineChunkConfig,
    StretchOfflineChunkPlan, DEFAULT_OFFLINE_STRETCH_CHUNK_OVERLAP_FRAMES,
    DEFAULT_OFFLINE_STRETCH_CHUNK_SOURCE_FRAMES,
};
#[cfg(any(test, feature = "evidence"))]
pub use benchmark::{
    assess_stretch_metrics, compare_sustained_material_coherence,
    compare_synthetic_realtime_preview_backends, compare_synthetic_stretch_backends,
    format_stretch_acceptance_report, format_stretch_quality_priority_report,
    format_synthetic_stretch_comparison_report, generate_synthetic_stretch_audio,
    measure_draft_loop_boundary_click, measure_draft_stereo_image_delta,
    measure_draft_transient_smear, measure_dynamic_segment_seam_click, measure_loop_boundary_click,
    measure_pitch_shift_error_cents, measure_stereo_image_delta,
    measure_transient_reset_loop_boundary_click, measure_transient_reset_stereo_image_delta,
    measure_transient_reset_transient_smear, output_length_drift_samples,
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
    STRETCH_BENCHMARK_CORPUS, STRETCH_CORPUS_MANIFEST, STRETCH_CORPUS_MANIFEST_ENTRIES,
    STRETCH_CORPUS_SOURCE_POLICY,
};
pub use cache_identity::{
    StretchCacheIdentity, StretchCacheIdentityError, StretchCacheIdentityInput,
    StretchChannelLayout, StretchPitchPoint, StretchRatioPoint, StretchRenderGeometry,
    StretchWarpMarker, SIGNAL_STRETCH_BEHAVIOR_VERSION, SIGNAL_STRETCH_ENGINE_VERSION,
    STRETCH_CACHE_IDENTITY_SCHEMA_VERSION,
};
#[cfg(any(test, feature = "evidence"))]
pub use corpus_report::{
    build_stretch_corpus_comparison_report, build_stretch_corpus_comparison_report_with_external,
    build_stretch_corpus_comparison_report_with_sources, format_stretch_corpus_comparison_report,
    StretchCorpusComparisonReport, StretchCorpusListeningNoteSlot, StretchCorpusListeningSource,
    StretchCorpusListeningSourceRecord, StretchCorpusSkippedAsset,
    StretchExternalBenchmarkComparison, StretchExternalBenchmarkRender,
};
pub use creative::{
    render_creative_stretch, CreativeStretchCharacter, CreativeStretchError,
    CreativeStretchRatioDomain, CreativeStretchRequest, CREATIVE_STRETCH_CYCLIC_MAX_RATIO,
    CREATIVE_STRETCH_CYCLIC_MIN_RATIO, CREATIVE_STRETCH_DEFAULT_CYCLE,
    CREATIVE_STRETCH_DEFAULT_SPACE, CREATIVE_STRETCH_DREAM_MAX_RATIO,
    CREATIVE_STRETCH_DREAM_MIN_RATIO, CREATIVE_STRETCH_ENGINE_VERSION, CREATIVE_STRETCH_MAX_CYCLE,
    CREATIVE_STRETCH_MIN_CYCLE,
};
#[cfg(any(test, feature = "evidence"))]
pub use formant_boundary::{measure_formant_boundary, StretchFormantBoundaryMeasurement};
#[cfg(any(test, feature = "evidence"))]
pub use promotion::{
    current_synthetic_offline_high_quality_promotion_receipt, StretchSyntheticPromotionPolicy,
};
pub use promotion::{
    StretchProductQualityEvidence, StretchPromotionReceipt, StretchPromotionStatus,
    REQUIRED_STRETCH_LISTENING_FAMILY_COUNT,
};
pub use realtime_preview::{
    plan_realtime_preview_stream, project_realtime_preview_fixed_ratio_source_advance,
    RealtimePreviewCallbackProcessError, RealtimePreviewCallbackProcessReport,
    RealtimePreviewCallbackState, RealtimePreviewCallbackTimelineMode,
    RealtimePreviewDynamicSourceProjectionReport, RealtimePreviewIntegrationMode,
    RealtimePreviewPlanError, RealtimePreviewSourceProjectionReport, RealtimePreviewStreamConfig,
    RealtimePreviewStreamingContract, RealtimePreviewUnsupportedMode,
};
pub use realtime_preview_stream::{
    RealtimePreviewStreamError, RealtimePreviewStreamRenderReport, RealtimePreviewStreamState,
    REALTIME_PREVIEW_STREAM_MAX_RATIO, REALTIME_PREVIEW_STREAM_MAX_WORKING_BYTES,
    REALTIME_PREVIEW_STREAM_MIN_RATIO,
};
#[cfg(any(test, feature = "evidence"))]
pub use render_integrity::{
    assess_stretch_render_integrity, measure_stretch_render_integrity,
    StretchRenderIntegrityAssessment, StretchRenderIntegrityLimits,
    StretchRenderIntegrityMeasurement,
};
pub use resumable::{
    ResumableOfflineStretch, ResumableRenderReport, ResumableStretchConfig,
    MAX_RESUMABLE_WINDOW_SIZE, MAX_RESUMABLE_WORKING_BYTES,
};
#[cfg(any(test, feature = "evidence"))]
pub use tonal_texture::{measure_tonal_texture, StretchTonalTextureMeasurement};
#[cfg(any(test, feature = "evidence"))]
pub use transient_detail::{
    measure_transient_detail, measure_transient_event_detail, StretchTransientDetailMeasurement,
    StretchTransientEventDetail,
};
#[cfg(any(test, feature = "evidence"))]
pub use transient_smear::{
    detect_stretch_transients, detect_stretch_transients_with_policy, measure_transient_smear,
    StretchTransientDetectorPolicy, StretchTransientEvent, StretchTransientSmearMeasurement,
    StretchTransientSmearPolicies,
};

use phase_vocoder::{
    phase_vocoder, transient_reset_phase_vocoder, transient_reset_phase_vocoder_linked_stereo,
};
use signal_dsp_resample::{resample_mono, ResampleConfig, ResampleQuality};
use signal_primitives::{Sample, SampleRate};

const DYNAMIC_RATIO_SEAM_SMOOTH_FRAMES: usize = 256;

/// Analysis hops of source, beyond one window, that every dynamic-ratio
/// segment must carry so the phase vocoder has overlapping frames to track.
///
/// Contract `046` freezes one window as the floor. This is stricter for two
/// measured reasons.
///
/// Pitch: a single-window segment gives the phase vocoder one analysis frame
/// and tracks the source poorly. On a `440 Hz` tone through a curve sampled
/// every `1024` frames, three extra hops leave `19.6` cents of error, eight
/// leave `2.8`.
///
/// Seam-rate modulation: segments render independently, so every join leaves an
/// envelope dip and the render modulates at the segment rate. Concealed
/// listening heard it as a secondary rhythmic pulse. Measured envelope
/// modulation at the segment period against a `0.04 dB` whole-render floor:
/// `0.545 dB` at eight extra hops, `0.268` at sixteen, `0.115` at
/// thirty-two, `0.039` at sixty-four.
///
/// Thirty-two is the balance point. Sixty-four reaches the floor but its
/// `725 ms` minimum swallows realistic tempo-ramp spans. At the retained
/// `2048/512` geometry this is `18432` source frames, `384 ms` at 48 kHz.
///
/// The modulation is inherent to independently rendered segments. `g10.039`
/// removes it by carrying renderer state across the join instead of lengthening
/// segments.
const MIN_DYNAMIC_RATIO_SEGMENT_EXTRA_HOPS: usize = 32;

/// Largest whole-buffer render, in output samples across all channels.
///
/// One gibibyte of [`Sample`]: roughly 93 minutes mono or 46 minutes stereo at
/// 48 kHz in a single call. Longer material is the offline chunk plan's
/// responsibility (see [`plan_offline_stretch_chunks`]). Frozen by Contract
/// `046`, 2026-07-27 addendum.
pub const MAX_OFFLINE_STRETCH_OUTPUT_SAMPLES: usize = 268_435_456;

/// Whole-buffer stretch render failure.
///
/// A backend that cannot serve a request says so instead of attempting the
/// allocation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StretchRenderError {
    /// The resumable renderer was configured outside its supported geometry.
    UnsupportedResumableConfiguration,
    /// The requested output exceeds [`MAX_OFFLINE_STRETCH_OUTPUT_SAMPLES`].
    OutputTooLarge {
        /// Output samples the request would have produced, saturated.
        requested_samples: u128,
        /// Frozen ceiling in output samples.
        maximum_samples: usize,
    },
}

/// Validate the output size one whole-buffer render would produce.
///
/// Returns the target frame count when the render fits inside
/// [`MAX_OFFLINE_STRETCH_OUTPUT_SAMPLES`].
fn checked_target_frames(
    source_frames: usize,
    ratio: f64,
    channels: usize,
) -> Result<usize, StretchRenderError> {
    let target_frames = (source_frames as f64 * ratio).round();
    checked_output_frames(target_frames, channels)
}

fn checked_output_frames(target_frames: f64, channels: usize) -> Result<usize, StretchRenderError> {
    let samples = target_frames * channels as f64;
    if !samples.is_finite() || samples < 0.0 || samples > MAX_OFFLINE_STRETCH_OUTPUT_SAMPLES as f64
    {
        return Err(StretchRenderError::OutputTooLarge {
            requested_samples: saturating_u128(samples),
            maximum_samples: MAX_OFFLINE_STRETCH_OUTPUT_SAMPLES,
        });
    }
    Ok(target_frames as usize)
}

fn saturating_u128(value: f64) -> u128 {
    if !value.is_finite() || value <= 0.0 {
        0
    } else if value >= u128::MAX as f64 {
        u128::MAX
    } else {
        value as u128
    }
}

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
    ///
    /// Renders larger than [`MAX_OFFLINE_STRETCH_OUTPUT_SAMPLES`] are refused
    /// rather than attempted.
    fn stretch_mono(&mut self, input: &[Sample]) -> Result<Vec<Sample>, StretchRenderError>;
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

    fn stretch_mono(&mut self, input: &[Sample]) -> Result<Vec<Sample>, StretchRenderError> {
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
    pub fn stretch_interleaved_stereo(
        &mut self,
        frames: &[Sample],
    ) -> Result<Vec<Sample>, StretchRenderError> {
        let frame_count = frames.len() / 2;
        let target_frames = checked_target_frames(frame_count, self.ratio, 2)?;
        if frame_count == 0 || target_frames == 0 {
            return Ok(Vec::new());
        }
        let even_frames = &frames[..frame_count * 2];
        if (self.ratio - 1.0).abs() < 1.0e-9 {
            return Ok(even_frames.to_vec());
        }
        if frame_count < self.window_size {
            return Ok(linear_time_scale_interleaved_stereo(
                even_frames,
                target_frames,
            ));
        }
        Ok(transient_reset_phase_vocoder_linked_stereo(
            even_frames,
            target_frames,
            self.ratio,
            self.window_size,
            self.analysis_hop,
        ))
    }

    /// Apply independent pitch shift and tempo stretch to one mono preview
    /// buffer.
    pub fn stretch_pitch_mono(
        &mut self,
        input: &[Sample],
        sample_rate: SampleRate,
        pitch_shift_semitones: f64,
    ) -> Result<Vec<Sample>, StretchRenderError> {
        let target_len = checked_target_frames(input.len(), self.ratio, 1)?;
        if input.is_empty() || target_len == 0 {
            return Ok(Vec::new());
        }
        if pitch_shift_semitones.abs() < 1.0e-9 || sample_rate.0 == 0 {
            return self.stretch_mono(input);
        }

        let pitched = pitch_shift_mono_to_nominal_rate(input, sample_rate, pitch_shift_semitones);
        Ok(stretch_to_exact_mono(
            &pitched,
            target_len,
            self.window_size,
            self.analysis_hop,
            transient_reset_phase_vocoder,
        ))
    }

    /// Apply independent pitch shift and tempo stretch to interleaved stereo
    /// preview material.
    pub fn stretch_pitch_interleaved_stereo(
        &mut self,
        frames: &[Sample],
        sample_rate: SampleRate,
        pitch_shift_semitones: f64,
    ) -> Result<Vec<Sample>, StretchRenderError> {
        let frame_count = frames.len() / 2;
        let target_frames = checked_target_frames(frame_count, self.ratio, 2)?;
        if frame_count == 0 || target_frames == 0 {
            return Ok(Vec::new());
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
        Ok(stretch_to_exact_linked_stereo(
            &pitched,
            target_frames,
            self.window_size,
            self.analysis_hop,
        ))
    }

    /// Stretch one mono buffer with a stepwise dynamic ratio curve.
    pub fn stretch_dynamic_ratio_mono(
        &mut self,
        input: &[Sample],
        ratio_curve: &[StretchRatioPoint],
    ) -> Result<Vec<Sample>, StretchRenderError> {
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
    ) -> Result<Vec<Sample>, StretchRenderError> {
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

    fn stretch_mono(&mut self, input: &[Sample]) -> Result<Vec<Sample>, StretchRenderError> {
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
    /// This path uses a mid/side linked analysis surface so stereo image
    /// metrics can be measured against a candidate that preserves channel
    /// relationships directly. A trailing odd sample is ignored.
    pub fn stretch_interleaved_stereo(
        &mut self,
        frames: &[Sample],
    ) -> Result<Vec<Sample>, StretchRenderError> {
        let frame_count = frames.len() / 2;
        let target_frames = checked_target_frames(frame_count, self.ratio, 2)?;
        if frame_count == 0 || target_frames == 0 {
            return Ok(Vec::new());
        }
        let even_frames = &frames[..frame_count * 2];
        if (self.ratio - 1.0).abs() < 1.0e-9 {
            return Ok(even_frames.to_vec());
        }
        if frame_count < self.window_size {
            return Ok(linear_time_scale_interleaved_stereo(
                even_frames,
                target_frames,
            ));
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
            Ok(transient_reset_phase_vocoder_linked_stereo(
                even_frames,
                target_frames,
                self.ratio,
                short_window_size_for_path(self.path),
                short_window_analysis_hop_for_path(self.path),
            ))
        } else {
            Ok(default_output)
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
    ) -> Result<Vec<Sample>, StretchRenderError> {
        let target_len = checked_target_frames(input.len(), self.ratio, 1)?;
        if input.is_empty() || target_len == 0 {
            return Ok(Vec::new());
        }
        if pitch_shift_semitones.abs() < 1.0e-9 || sample_rate.0 == 0 {
            return self.stretch_mono(input);
        }

        let pitched = pitch_shift_mono_to_nominal_rate(input, sample_rate, pitch_shift_semitones);
        Ok(stretch_to_exact_mono(
            &pitched,
            target_len,
            self.window_size,
            self.analysis_hop,
            transient_reset_phase_vocoder,
        ))
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
    ) -> Result<Vec<Sample>, StretchRenderError> {
        let frame_count = frames.len() / 2;
        let target_frames = checked_target_frames(frame_count, self.ratio, 2)?;
        if frame_count == 0 || target_frames == 0 {
            return Ok(Vec::new());
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
        Ok(stretch_to_exact_linked_stereo(
            &pitched,
            target_frames,
            self.window_size,
            self.analysis_hop,
        ))
    }

    /// Apply one static pitch shift while following a stepwise dynamic ratio
    /// curve over interleaved stereo.
    ///
    /// Segment boundaries use the same source-frame vocabulary as
    /// [`Self::stretch_dynamic_ratio_interleaved_stereo`]. Resampling runs
    /// ahead of the stretch over the whole stream, so the stretch plan is in
    /// pitched coordinates — the same order the offline artifact renderer
    /// uses, and the reason there is no per-segment resampler restart to
    /// smooth.
    pub fn stretch_dynamic_ratio_pitch_interleaved_stereo(
        &mut self,
        frames: &[Sample],
        ratio_curve: &[StretchRatioPoint],
        sample_rate: SampleRate,
        pitch_shift_semitones: f64,
    ) -> Result<Vec<Sample>, StretchRenderError> {
        if pitch_shift_semitones.abs() < 1.0e-9 || sample_rate.0 == 0 {
            return self.stretch_dynamic_ratio_interleaved_stereo(frames, ratio_curve);
        }
        self.stretch_dynamic_ratio_resumable(
            frames,
            2,
            ratio_curve,
            sample_rate,
            pitch_shift_semitones,
        )
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
    ) -> Result<Vec<Sample>, StretchRenderError> {
        self.stretch_dynamic_ratio_resumable(input, 1, ratio_curve, SampleRate(48_000), 0.0)
    }

    /// Render a dynamic ratio curve through the resumable renderer in one call.
    ///
    /// The segmented predecessor rendered each ratio segment independently and
    /// concatenated them, which restarts the phase vocoder at every segment join.
    /// Measured on a sustained `110 Hz` tone across a `1.6 -> 0.8` boundary, that
    /// left a first-difference step of `0.204` against a median step of `0.0051`.
    /// [`smooth_dynamic_segment_boundaries_interleaved`] attenuated it to `0.0174`
    /// but did not remove it — still above the render's own `p99.9` of `0.0138`.
    ///
    /// The resumable renderer carries phase, detector, and overlap-add state across
    /// the boundary, so there is no join to smooth: `0.0068`, below that same
    /// `p99.9`. Its whole-render `p99.9` is also half the segmented path's, because
    /// every segment restart was contributing, not only the ones at a ratio change.
    fn stretch_dynamic_ratio_resumable(
        &self,
        input: &[Sample],
        channels: usize,
        ratio_curve: &[StretchRatioPoint],
        sample_rate: SampleRate,
        pitch_shift_semitones: f64,
    ) -> Result<Vec<Sample>, StretchRenderError> {
        let frame_count = input.len() / channels;
        let even_input = &input[..frame_count * channels];
        let mut renderer = crate::resumable::ResumableOfflineStretch::new(
            crate::resumable::ResumableStretchConfig {
                channels,
                window_size: self.window_size,
                analysis_hop: self.analysis_hop,
                source_frames: frame_count,
                ratio_curve: ratio_curve.to_vec(),
                fallback_ratio: sanitize_ratio(self.ratio),
                sample_rate,
                pitch_shift_semitones,
            },
        )?;
        checked_output_frames(renderer.target_output_frames() as f64, channels)?;
        let mut output = Vec::with_capacity(renderer.target_output_frames() * channels);
        renderer.render(even_input, &mut output)?;
        renderer.flush(&mut output)?;
        Ok(output)
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
    ) -> Result<Vec<Sample>, StretchRenderError> {
        self.stretch_dynamic_ratio_resumable(frames, 2, ratio_curve, SampleRate(48_000), 0.0)
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

    fn stretch_mono(&mut self, input: &[Sample]) -> Result<Vec<Sample>, StretchRenderError> {
        let default_output = stretch_mono_with_engine(
            input,
            self.ratio,
            self.window_size,
            self.analysis_hop,
            transient_reset_phase_vocoder,
        )?;
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
            Ok(default_output)
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

fn usize_to_u64(value: usize) -> u64 {
    value.try_into().unwrap_or(u64::MAX)
}

fn abs_diff_frames(left: u64, right: u64) -> usize {
    left.abs_diff(right).try_into().unwrap_or(usize::MAX)
}

fn floor_frame_to_u64(frame: f64) -> u64 {
    if !frame.is_finite() || frame <= 0.0 {
        0
    } else if frame >= u64::MAX as f64 {
        u64::MAX
    } else {
        frame.floor() as u64
    }
}

fn ceil_frame_to_u64(frame: f64) -> u64 {
    if !frame.is_finite() || frame <= 0.0 {
        0
    } else if frame >= u64::MAX as f64 {
        u64::MAX
    } else {
        frame.ceil() as u64
    }
}

fn ceil_frame_to_usize(frame: f64) -> usize {
    if !frame.is_finite() || frame <= 0.0 {
        0
    } else if frame >= usize::MAX as f64 {
        usize::MAX
    } else {
        frame.ceil() as usize
    }
}

/// Wrap a phase into `-PI..PI` by remainder.
///
/// `phase_vocoder` carries a second implementation using
/// `phase - TAU * (phase / TAU).round()`. The two are **not** interchangeable:
/// over a `-50..50` sweep at `1e-4` steps, `945158` of `1005319` values differ
/// in bits, worst delta `2.6e-6`, and at exactly `-PI` they disagree in sign,
/// this one returning `-PI` and the round form `+PI`.
///
/// Unifying them is therefore an output change, not a refactor, so `g10.038`
/// left both in place. It needs a batch that can carry a re-baseline with
/// evidence. Audit finding `A10` is refined rather than closed.
fn wrap_phase(phase: f32) -> f32 {
    (phase + std::f32::consts::PI).rem_euclid(std::f32::consts::TAU) - std::f32::consts::PI
}

fn stretch_mono_with_engine(
    input: &[Sample],
    ratio: f64,
    window_size: usize,
    analysis_hop: usize,
    engine: fn(&[Sample], usize, f64, usize, usize) -> Vec<Sample>,
) -> Result<Vec<Sample>, StretchRenderError> {
    let target_len = checked_target_frames(input.len(), ratio, 1)?;
    if input.is_empty() || target_len == 0 {
        return Ok(Vec::new());
    }
    if (ratio - 1.0).abs() < 1.0e-9 {
        return Ok(input.to_vec());
    }
    if input.len() < window_size {
        return Ok(linear_time_scale(input, target_len));
    }
    Ok(engine(input, target_len, ratio, window_size, analysis_hop))
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

/// Output-frame positions of the seams a dynamic-ratio render actually
/// produces, after short segments are coalesced.
#[cfg(any(test, feature = "evidence"))]
pub(crate) fn dynamic_ratio_output_boundaries(
    input_frames: usize,
    ratio_curve: &[StretchRatioPoint],
    fallback_ratio: f64,
) -> Vec<usize> {
    let segments = coalesce_short_dynamic_ratio_segments(
        dynamic_ratio_segments(input_frames, ratio_curve, sanitize_ratio(fallback_ratio)),
        min_dynamic_ratio_segment_frames(DEFAULT_WINDOW_SIZE, DEFAULT_ANALYSIS_HOP),
    );
    dynamic_ratio_segment_boundaries(&segments)
}

pub(crate) fn stretch_dynamic_ratio_mono_with_engine(
    input: &[Sample],
    ratio_curve: &[StretchRatioPoint],
    fallback_ratio: f64,
    window_size: usize,
    analysis_hop: usize,
    engine: fn(&[Sample], usize, f64, usize, usize) -> Vec<Sample>,
) -> Result<Vec<Sample>, StretchRenderError> {
    let segments = coalesce_short_dynamic_ratio_segments(
        dynamic_ratio_segments(input.len(), ratio_curve, sanitize_ratio(fallback_ratio)),
        min_dynamic_ratio_segment_frames(window_size, analysis_hop),
    );
    let boundaries = dynamic_ratio_segment_boundaries(&segments);
    let target_len: usize = segments.iter().map(|segment| segment.target_frames).sum();
    checked_output_frames(target_len as f64, 1)?;
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
    smooth_dynamic_segment_boundaries_interleaved(
        &mut output,
        1,
        &boundaries,
        DYNAMIC_RATIO_SEAM_SMOOTH_FRAMES,
    );
    Ok(output)
}

fn stretch_dynamic_ratio_linked_stereo_with_engine(
    input: &[Sample],
    ratio_curve: &[StretchRatioPoint],
    fallback_ratio: f64,
    window_size: usize,
    analysis_hop: usize,
) -> Result<Vec<Sample>, StretchRenderError> {
    let frame_count = input.len() / 2;
    let even_input = &input[..frame_count * 2];
    let segments = coalesce_short_dynamic_ratio_segments(
        dynamic_ratio_segments(frame_count, ratio_curve, sanitize_ratio(fallback_ratio)),
        min_dynamic_ratio_segment_frames(window_size, analysis_hop),
    );
    let boundaries = dynamic_ratio_segment_boundaries(&segments);
    let target_frames: usize = segments.iter().map(|segment| segment.target_frames).sum();
    checked_output_frames(target_frames as f64, 2)?;
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
    Ok(output)
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

/// Merge adjacent segments until every one carries at least
/// `min_segment_frames` source frames.
///
/// A segment shorter than one analysis window cannot be rendered by the STFT
/// engine and would fall through to time-domain interpolation, which
/// pitch-shifts. Merging keeps the render pitch-preserving.
///
/// The merged target frame count is the sum of the counts its constituent
/// spans would have produced, so total output length and the average tempo
/// over the merged span are preserved exactly and the segment renders at the
/// mean ratio of the spans it covers. Frozen by Contract `046`, 2026-07-27
/// addendum.
fn coalesce_short_dynamic_ratio_segments(
    segments: Vec<DynamicRatioSegment>,
    min_segment_frames: usize,
) -> Vec<DynamicRatioSegment> {
    if min_segment_frames <= 1 || segments.len() < 2 {
        return segments;
    }

    let mut coalesced: Vec<DynamicRatioSegment> = Vec::with_capacity(segments.len());
    for segment in segments {
        match coalesced.last_mut() {
            Some(previous) if previous.end_frame - previous.start_frame < min_segment_frames => {
                previous.end_frame = segment.end_frame;
                previous.target_frames += segment.target_frames;
                previous.ratio = mean_segment_ratio(previous);
            }
            _ => coalesced.push(segment),
        }
    }

    // The final segment can still be short when the source ends mid-span. Fold
    // it backwards rather than leaving one sub-window render at the tail.
    while coalesced.len() >= 2 {
        let last = coalesced[coalesced.len() - 1];
        if last.end_frame - last.start_frame >= min_segment_frames {
            break;
        }
        coalesced.pop();
        let previous = coalesced
            .last_mut()
            .expect("length checked before the pop above");
        previous.end_frame = last.end_frame;
        previous.target_frames += last.target_frames;
        previous.ratio = mean_segment_ratio(previous);
    }

    coalesced
}

/// Shortest source span a dynamic-ratio segment may render.
///
/// One window yields a single analysis frame, which is enough to avoid the
/// interpolation fallback but not enough for the phase vocoder to track the
/// source. The extra hops give every segment several overlapping frames.
pub(crate) fn min_dynamic_ratio_segment_frames(window_size: usize, analysis_hop: usize) -> usize {
    window_size + analysis_hop.saturating_mul(MIN_DYNAMIC_RATIO_SEGMENT_EXTRA_HOPS)
}

fn mean_segment_ratio(segment: &DynamicRatioSegment) -> f64 {
    let source_frames = segment.end_frame - segment.start_frame;
    if source_frames == 0 {
        return segment.ratio;
    }
    segment.target_frames as f64 / source_frames as f64
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

    let current_smear = transient_smear::measure_selector_transient_smear(
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

    // Source transients are detected once and reused for both comparisons.
    // The current-output and draft-baseline measurements previously each
    // re-detected them from the same input with the same policy and geometry.
    let input_events = transient_smear::detect_stretch_transients_with_policy(
        input,
        EXPANSION_SHORT_WINDOW_SELECTOR_WINDOW_SIZE,
        EXPANSION_SHORT_WINDOW_SELECTOR_ANALYSIS_HOP,
        transient_smear::StretchTransientDetectorPolicy::production(),
    );
    let current_smear = transient_smear::measure_selector_transient_smear_with_input_events(
        input,
        &input_events,
        current_output,
        ratio,
        EXPANSION_SHORT_WINDOW_SELECTOR_WINDOW_SIZE,
        EXPANSION_SHORT_WINDOW_SELECTOR_ANALYSIS_HOP,
    );
    if current_smear.missed_transients >= EXPANSION_SHORT_WINDOW_SELECTOR_MIN_CURRENT_MISSES {
        return true;
    }

    let mut draft = PhaseVocoderStretcher::new(ratio);
    let Ok(draft_output) = draft.stretch_mono(input) else {
        // The default render already succeeded at this size, so a draft render
        // of the same input cannot exceed the bound. Stay on the current path
        // rather than switching on missing evidence.
        return false;
    };
    let draft_smear = transient_smear::measure_selector_transient_smear_with_input_events(
        input,
        &input_events,
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

    fn boundary_content_probe(len: usize, edge_frames: usize) -> Vec<Sample> {
        let mut input = vec![0.0; len];
        input[..edge_frames].fill(0.5);
        input[len - edge_frames..].fill(-0.5);
        input
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
        assert_eq!(
            stretcher
                .stretch_mono(&input)
                .expect("render fits the offline output bound"),
            input
        );
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
            let output = stretcher
                .stretch_mono(&input)
                .expect("render fits the offline output bound");
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
            let first_output = first
                .stretch_mono(&input)
                .expect("render fits the offline output bound");
            let repeated_output = repeated
                .stretch_mono(&input)
                .expect("render fits the offline output bound");

            assert_eq!(
                first_output.len(),
                (input.len() as f64 * ratio).round() as usize,
                "ratio {ratio}"
            );
            assert_eq!(first_output, repeated_output, "ratio {ratio}");
        }
    }

    #[test]
    fn offline_high_quality_boundary_preserves_endpoint_content() {
        let input = boundary_content_probe(48_000, 384);
        for ratio in [0.5, 2.0] {
            let mut stretcher = OfflineHighQualityStretcher::new(ratio);
            let output = stretcher
                .stretch_mono(&input)
                .expect("render fits the offline output bound");
            let edge_span = 2_048.min(output.len());

            assert_eq!(output.len(), (input.len() as f64 * ratio).round() as usize);
            assert!(
                rms(&output[..edge_span]) > 0.01,
                "ratio {ratio}: silent head"
            );
            assert!(
                rms(&output[output.len() - edge_span..]) > 0.01,
                "ratio {ratio}: silent tail"
            );
        }
    }

    #[test]
    fn compression_short_window_selector_matches_gate_decision() {
        let input = masked_soft_attack_probe(0.35);
        let ratio = 0.75;
        let mut default = OfflineHighQualityStretcher::new(ratio);
        let default_output = default
            .stretch_mono(&input)
            .expect("render fits the offline output bound");
        let mut short_window = OfflineHighQualityStretcher::with_window(
            ratio,
            COMPRESSION_SHORT_WINDOW_SELECTOR_WINDOW_SIZE,
            COMPRESSION_SHORT_WINDOW_SELECTOR_ANALYSIS_HOP,
        );
        let short_window_output = short_window
            .stretch_mono(&input)
            .expect("render fits the offline output bound");
        let mut selector = OfflineHighQualityStretcher::with_path(
            ratio,
            OfflineHighQualityPath::CompressionShortWindowSelector,
        );
        let selector_output = selector
            .stretch_mono(&input)
            .expect("render fits the offline output bound");
        let default_smear = measure_transient_smear(
            &input,
            &default_output,
            ratio,
            COMPRESSION_SHORT_WINDOW_SELECTOR_WINDOW_SIZE,
            COMPRESSION_SHORT_WINDOW_SELECTOR_ANALYSIS_HOP,
            StretchTransientSmearPolicies::production(),
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
        let default_output = default
            .stretch_mono(&input)
            .expect("render fits the offline output bound");
        let mut selector = OfflineHighQualityStretcher::with_path(
            ratio,
            OfflineHighQualityPath::CompressionShortWindowSelector,
        );

        assert_eq!(
            selector
                .stretch_mono(&input)
                .expect("render fits the offline output bound"),
            default_output
        );
    }

    #[test]
    fn expansion_short_window_selector_matches_gate_decision() {
        let input = masked_soft_attack_probe(0.35);
        let ratio = 1.25;
        let mut default = OfflineHighQualityStretcher::new(ratio);
        let default_output = default
            .stretch_mono(&input)
            .expect("render fits the offline output bound");
        let mut short_window = OfflineHighQualityStretcher::with_window(
            ratio,
            EXPANSION_SHORT_WINDOW_SELECTOR_WINDOW_SIZE,
            EXPANSION_SHORT_WINDOW_SELECTOR_ANALYSIS_HOP,
        );
        let short_window_output = short_window
            .stretch_mono(&input)
            .expect("render fits the offline output bound");
        let mut selector = OfflineHighQualityStretcher::with_path(
            ratio,
            OfflineHighQualityPath::ExpansionShortWindowSelector,
        );
        let selector_output = selector
            .stretch_mono(&input)
            .expect("render fits the offline output bound");
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
        let default_output = default
            .stretch_mono(&input)
            .expect("render fits the offline output bound");
        let mut selector = OfflineHighQualityStretcher::with_path(
            ratio,
            OfflineHighQualityPath::ExpansionShortWindowSelector,
        );

        assert_eq!(
            selector
                .stretch_mono(&input)
                .expect("render fits the offline output bound"),
            default_output
        );
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

        assert_eq!(
            stretcher
                .stretch_mono(&input)
                .expect("render fits the offline output bound"),
            input
        );
    }

    #[test]
    fn stretch_preserves_pitch_within_tolerance() {
        let sample_rate = 48_000.0;
        let input = sine(440.0, sample_rate, 48_000);
        for ratio in [0.75, 1.5, 2.0] {
            let mut stretcher = PhaseVocoderStretcher::new(ratio);
            let output = stretcher
                .stretch_mono(&input)
                .expect("render fits the offline output bound");
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
        let output = stretcher
            .stretch_mono(&input)
            .expect("render fits the offline output bound");
        assert_eq!(output.len(), 200);
        // Monotone ramp stays monotone under linear scaling.
        assert!(output.windows(2).all(|pair| pair[1] >= pair[0] - 1.0e-6));
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
            let output = stretcher
                .stretch_interleaved_stereo(&frames)
                .expect("render fits the offline output bound");

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

        assert_eq!(
            stretcher
                .stretch_interleaved_stereo(&frames)
                .expect("render fits the offline output bound"),
            frames[..4]
        );
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
            first
                .stretch_interleaved_stereo(&frames)
                .expect("render fits the offline output bound"),
            repeated
                .stretch_interleaved_stereo(&frames)
                .expect("render fits the offline output bound")
        );
    }

    #[test]
    fn offline_high_quality_pitch_shift_preserves_tempo_length_contract() {
        let input = sine(440.0, 48_000.0, 48_000);
        for (ratio, semitones) in [(1.0, 12.0), (1.5, -7.0), (0.75, 5.0)] {
            let mut stretcher = OfflineHighQualityStretcher::new(ratio);
            let output = stretcher
                .stretch_pitch_mono(&input, SampleRate(48_000), semitones)
                .expect("render fits the offline output bound");

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

        let output = stretcher
            .stretch_pitch_mono(&input, SampleRate(48_000), 12.0)
            .expect("render fits the offline output bound");
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
        let first_output = first
            .stretch_pitch_interleaved_stereo(&frames, SampleRate(48_000), -5.0)
            .expect("render fits the offline output bound");
        let repeated_output = repeated
            .stretch_pitch_interleaved_stereo(&frames, SampleRate(48_000), -5.0)
            .expect("render fits the offline output bound");

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
        let output = stretcher
            .stretch_dynamic_ratio_mono(&input, &ratio_curve)
            .expect("render fits the offline output bound");

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

        // Invalid points are ignored, so this must render as the stretcher's
        // own static ratio. Compared through the same renderer with an empty
        // curve, because the invariant is about the curve, not about which
        // renderer runs.
        let curved = dynamic
            .stretch_dynamic_ratio_mono(&input, &ratio_curve)
            .expect("render fits the offline output bound");
        let empty_curve = dynamic
            .stretch_dynamic_ratio_mono(&input, &[])
            .expect("render fits the offline output bound");
        assert_eq!(curved, empty_curve);

        // `stretch_dynamic_ratio_mono` renders resumably and `stretch_mono` in
        // one shot, so they are close rather than identical at a static ratio.
        // Recorded as a bound because it is a real consequence of the dynamic
        // API moving to the resumable renderer: same length, same algorithm,
        // different state handling.
        let flat = fixed
            .stretch_mono(&input)
            .expect("render fits the offline output bound");
        assert_eq!(curved.len(), flat.len());
        let worst = curved
            .iter()
            .zip(flat.iter())
            .map(|(left, right)| (left - right).abs())
            .fold(0.0f32, f32::max);
        assert!(
            worst < 1.0e-4,
            "resumable and one-shot static renders drifted apart: {worst}",
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
        let first_output = first
            .stretch_dynamic_ratio_interleaved_stereo(&frames, &ratio_curve)
            .expect("render fits the offline output bound");
        let repeated_output = repeated
            .stretch_dynamic_ratio_interleaved_stereo(&frames, &ratio_curve)
            .expect("render fits the offline output bound");

        assert_eq!(
            first_output.len(),
            dynamic_ratio_output_frames(left.len(), &ratio_curve, 1.0) * 2
        );
        assert_eq!(first_output, repeated_output);
    }

    #[test]
    fn dynamic_segment_seam_smoothing_is_not_neutral_on_continuous_material() {
        let sample_rate = 48_000.0;
        let left = sine(220.0, sample_rate, 48_000);
        let right = sine(440.0, sample_rate, 48_000);
        let mut frames = Vec::with_capacity(left.len() * 2);
        for (l, r) in left.iter().zip(right.iter()) {
            frames.push(*l);
            frames.push(*r);
        }
        // Explicit boundaries: this owner tests the smoother, not the
        // segmentation law. Deriving them from a ratio curve coupled it to the
        // Contract `046` minimum segment length, and it broke when that
        // minimum grew past the curve's span length.
        let boundaries = vec![12_000, 28_000];
        let mut raw = frames.clone();
        let before = measure_dynamic_segment_seam_click(&raw, 2, &boundaries, 1.0);
        smooth_dynamic_segment_boundaries_interleaved(&mut raw, 2, &boundaries, 64);
        let after = measure_dynamic_segment_seam_click(&raw, 2, &boundaries, 1.0);

        // Continuous material with no join: the smoother has nothing to fix,
        // and drags 64 frames either side of each nominated frame toward the
        // midpoint of the pair it straddles. That is a discontinuity it
        // introduces, not one it removes. Measured -240 dBFS (nothing) before,
        // -70.9 dBFS after.
        assert!(
            before.click_dbfs <= -240.0,
            "clean sines should show no seam, got {:.2} dBFS",
            before.click_dbfs,
        );
        assert!(
            after.click_dbfs > before.click_dbfs + 100.0,
            "smoothing continuous material should introduce a measurable \
             discontinuity, got {:.2} dBFS",
            after.click_dbfs,
        );
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
        let first_output = first
            .stretch_dynamic_ratio_pitch_interleaved_stereo(
                &frames,
                &ratio_curve,
                SampleRate(48_000),
                2.0,
            )
            .expect("render fits the offline output bound");
        let repeated_output = repeated
            .stretch_dynamic_ratio_pitch_interleaved_stereo(
                &frames,
                &ratio_curve,
                SampleRate(48_000),
                2.0,
            )
            .expect("render fits the offline output bound");

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
    fn dynamic_segment_seam_metric_reports_excess_over_the_renders_own_floor() {
        // Too short to hold any frame outside a seam window: there is no way to
        // tell a seam from the waveform, so the answer is "unmeasurable", not
        // "clean". The predecessor of this measurement answered "clean".
        let tiny = [0.0, 0.0, 0.1, 0.2, 0.9, -0.4, 1.0, -0.3];
        assert!(measure_dynamic_segment_seam_click(&tiny, 2, &[2], 1.0)
            .click_dbfs
            .is_nan());

        // A long, smooth ramp with one injected step. The step is 0.5 against a
        // per-frame background of 0.0001, so it must read close to 0.5 rather
        // than to the raw first difference.
        let frame_count = 8_000usize;
        let mut frames = Vec::with_capacity(frame_count * 2);
        for index in 0..frame_count {
            let value = index as f32 * 0.0001;
            frames.push(value);
            frames.push(value);
        }
        for sample in frames[4_000 * 2..].iter_mut() {
            *sample += 0.5;
        }
        let measurement = measure_dynamic_segment_seam_click(&frames, 2, &[4_000], 1.0);
        assert_eq!(measurement.ratio, 1.0);
        assert_eq!(measurement.channels, 2);
        assert_eq!(measurement.seam_frames, vec![4_000]);
        assert!(
            (measurement.peak_seam_delta - 0.5).abs() < 1.0e-3,
            "expected the injected step less the floor, got {}",
            measurement.peak_seam_delta,
        );
        assert_eq!(
            measurement.metric.metric,
            StretchMetric::DynamicSegmentSeamClickDbfs
        );
        assert_eq!(measurement.metric.value, measurement.click_dbfs);

        // And it stays visible through the smoother, which is the whole point:
        // the smoother sets the straddling pair equal, so a measurement that
        // read only that pair scored this -240 dBFS, the silence sentinel.
        // A linear ramp is the smoother's best case -- it really does spread
        // the 0.5 step over its 256-frame fade -- and even here the residue
        // reads -60.2 dBFS rather than silence.
        let mut smoothed = frames.clone();
        smooth_dynamic_segment_boundaries_interleaved(&mut smoothed, 2, &[4_000], 256);
        let after = measure_dynamic_segment_seam_click(&smoothed, 2, &[4_000], 1.0);
        assert!(
            after.click_dbfs > -120.0,
            "the smoother must not be able to hide the step, got {:.2} dBFS",
            after.click_dbfs,
        );
    }

    #[test]
    fn pitch_shift_metric_reports_dominant_frequency_error() {
        let sample_rate_hz = 48_000;
        let sample_rate = sample_rate_hz as f32;
        let input = sine(440.0, sample_rate, sample_rate_hz as usize);
        let mut stretcher = OfflineHighQualityStretcher::new(1.0);
        let output = stretcher
            .stretch_pitch_mono(&input, SampleRate(sample_rate_hz), 12.0)
            .expect("render fits the offline output bound");
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
        let expanded_transient = report
            .comparisons
            .iter()
            .find(|comparison| {
                comparison.case_id == "stretch:extreme_ratio"
                    && comparison.metric == StretchMetric::TransientSmearFrames
                    && comparison.path == StretchBenchmarkPath::FixedRatio
                    && comparison.ratio == 2.0
            })
            .expect("2x transient-smear comparison should remain covered");
        assert!(expanded_transient.baseline_value.is_finite());
        assert!(expanded_transient.candidate_value.is_finite());
        assert!(expanded_transient.delta.is_finite());
        assert_ne!(
            expanded_transient.outcome,
            StretchBenchmarkComparisonOutcome::Inconclusive
        );
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
        // Assert against the constant, not a literal. The engine version
        // advances whenever renderer output changes, and this owner should
        // prove the report carries it, not pin a particular value.
        assert!(formatted.contains(&format!("engine={SIGNAL_STRETCH_ENGINE_VERSION}")));
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
        let measurement = measure_transient_smear(
            &input,
            &output,
            1.0,
            16,
            4,
            StretchTransientSmearPolicies::production(),
        );

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
        let promoted = measure_transient_smear(
            &input,
            &output,
            1.0,
            1024,
            256,
            StretchTransientSmearPolicies::production(),
        );
        let strict = measure_transient_smear(
            &input,
            &output,
            1.0,
            1024,
            256,
            StretchTransientSmearPolicies::symmetric(StretchTransientDetectorPolicy::production()),
        );
        let recovery = measure_transient_smear(
            &input,
            &output,
            1.0,
            1024,
            256,
            StretchTransientSmearPolicies {
                input: StretchTransientDetectorPolicy::production(),
                output: StretchTransientDetectorPolicy::production(),
                output_recovery: Some(StretchTransientDetectorPolicy::candidate_review()),
            },
        );

        assert_eq!(promoted, recovery);
        assert!(promoted.matched_transients > strict.matched_transients);
        assert!(promoted.missed_transients < strict.missed_transients);
    }

    #[test]
    fn candidate_transient_smear_counts_masked_soft_attack() {
        let input = masked_soft_attack_probe(0.25);
        let production = measure_transient_smear(
            &input,
            &input,
            1.0,
            1024,
            256,
            StretchTransientSmearPolicies::symmetric(StretchTransientDetectorPolicy::production()),
        );
        let candidate = measure_transient_smear(
            &input,
            &input,
            1.0,
            1024,
            256,
            StretchTransientSmearPolicies::symmetric(
                StretchTransientDetectorPolicy::candidate_review(),
            ),
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
        let production = measure_transient_smear(
            &input,
            &output,
            1.0,
            1024,
            256,
            StretchTransientSmearPolicies {
                input: StretchTransientDetectorPolicy::production(),
                output: StretchTransientDetectorPolicy::production(),
                output_recovery: None,
            },
        );
        let candidate_output = measure_transient_smear(
            &input,
            &output,
            1.0,
            1024,
            256,
            StretchTransientSmearPolicies {
                input: StretchTransientDetectorPolicy::production(),
                output: StretchTransientDetectorPolicy::candidate_review(),
                output_recovery: None,
            },
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
        let production = measure_transient_smear(
            &input,
            &output,
            1.0,
            1024,
            256,
            StretchTransientSmearPolicies {
                input: StretchTransientDetectorPolicy::production(),
                output: StretchTransientDetectorPolicy::production(),
                output_recovery: None,
            },
        );
        let recovery = measure_transient_smear(
            &input,
            &output,
            1.0,
            1024,
            256,
            StretchTransientSmearPolicies {
                input: StretchTransientDetectorPolicy::production(),
                output: StretchTransientDetectorPolicy::production(),
                output_recovery: Some(StretchTransientDetectorPolicy::candidate_review()),
            },
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

        benchmark::smooth_loop_boundary_interleaved(&mut frames, 2, 1);

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

#[cfg(test)]
mod dynamic_segment_seam_evidence {
    use super::*;
    use crate::benchmark::measure_dynamic_segment_seam_click;

    /// The independently-rendered segment path leaves a seam at a hard ratio
    /// change; the resumable renderer does not.
    ///
    /// The corpus case `stretch:tempo_ramp` changes ratio by `8%` and shows no
    /// seam on either path, so it cannot support this claim. This curve steps
    /// `1.6 -> 0.8` across a sustained `110 Hz` tone, which is where a phase
    /// vocoder restart is audible, and it is the case the fixed
    /// `DynamicSegmentSeamClickDbfs` measurement was built against.
    #[test]
    fn resumable_dynamic_ratio_has_no_seam_where_segmented_rendering_does() {
        let frame_count = 96_000usize;
        let mut frames = Vec::with_capacity(frame_count * 2);
        for index in 0..frame_count {
            let seconds = index as f32 / 48_000.0;
            let sample = (2.0 * std::f32::consts::PI * 110.0 * seconds).sin() * 0.5;
            frames.push(sample);
            frames.push(sample);
        }
        let curve = vec![
            StretchRatioPoint::new(0, 1.0),
            StretchRatioPoint::new(32_000, 1.6),
            StretchRatioPoint::new(64_000, 0.8),
        ];
        let seams = dynamic_ratio_output_boundaries(frame_count, &curve, 1.0);
        assert!(!seams.is_empty(), "the curve must produce segment joins");

        // Three renders: segments concatenated raw, the same with the seam
        // smoother, and the resumable renderer that has no join at all.
        let segments = coalesce_short_dynamic_ratio_segments(
            dynamic_ratio_segments(frame_count, &curve, 1.0),
            min_dynamic_ratio_segment_frames(DEFAULT_WINDOW_SIZE, DEFAULT_ANALYSIS_HOP),
        );
        let mut unsmoothed = Vec::new();
        for segment in segments {
            unsmoothed.extend(stretch_to_exact_linked_stereo(
                &frames[segment.start_frame * 2..segment.end_frame * 2],
                segment.target_frames,
                DEFAULT_WINDOW_SIZE,
                DEFAULT_ANALYSIS_HOP,
            ));
        }
        let smoothed = stretch_dynamic_ratio_linked_stereo_with_engine(
            &frames,
            &curve,
            1.0,
            DEFAULT_WINDOW_SIZE,
            DEFAULT_ANALYSIS_HOP,
        )
        .expect("smoothed segmented render");
        let resumable = OfflineHighQualityStretcher::new(1.0)
            .stretch_dynamic_ratio_interleaved_stereo(&frames, &curve)
            .expect("resumable render");
        assert_eq!(smoothed.len(), resumable.len());
        assert_eq!(unsmoothed.len(), smoothed.len());

        let click =
            |data: &[Sample]| measure_dynamic_segment_seam_click(data, 2, &seams, 1.0).click_dbfs;
        let unsmoothed_click = click(&unsmoothed);
        let smoothed_click = click(&smoothed);
        let resumable_click = click(&resumable);
        println!(
            "unsmoothed {unsmoothed_click:.2} smoothed {smoothed_click:.2} \
             resumable {resumable_click:.2} dBFS"
        );

        // Both segmented renders leave a seam the measurement can see. This is
        // the half of the assertion that shows the measurement works: an
        // earlier version of this metric scored the smoothed render -240 dBFS
        // because the smoother sets the two samples it reads to their midpoint.
        assert!(
            unsmoothed_click > -40.0,
            "raw segment joins should be plainly visible, got {unsmoothed_click:.2} dBFS",
        );
        assert!(
            smoothed_click > -40.0,
            "the smoother spreads the join across its fade rather than removing \
             it, so it should still be visible, got {smoothed_click:.2} dBFS",
        );

        // The resumable renderer carries phase, detector, and overlap-add state
        // across the join, so there is no restart to hear.
        assert!(
            resumable_click < smoothed_click - 40.0,
            "resumable should be far below the smoothed segmented render: \
             {resumable_click:.2} vs {smoothed_click:.2} dBFS",
        );
    }
}
