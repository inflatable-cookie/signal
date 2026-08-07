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

#[allow(unused_imports)] // unit tests import `Sample` via `use super::*`
pub(crate) use signal_primitives::Sample;

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

mod stretch_backend;
mod stretch_engine;

pub use stretch_backend::{
    stretch_backend_plan, OfflineHighQualityPath, OfflineHighQualityStretcher,
    PhaseVocoderStretcher, RealtimePreviewStretcher, StretchBackendPlan, StretchBackendStatus,
    StretchBackendTier, StretchQuality, TimeStretcher,
    COMPRESSION_SHORT_WINDOW_SELECTOR_ANALYSIS_HOP,
    COMPRESSION_SHORT_WINDOW_SELECTOR_MIN_CURRENT_MISSES,
    COMPRESSION_SHORT_WINDOW_SELECTOR_MIN_CURRENT_SMEAR_FRAMES,
    COMPRESSION_SHORT_WINDOW_SELECTOR_WINDOW_SIZE, DEFAULT_ANALYSIS_HOP, DEFAULT_WINDOW_SIZE,
    EXPANSION_SHORT_WINDOW_SELECTOR_ANALYSIS_HOP,
    EXPANSION_SHORT_WINDOW_SELECTOR_MIN_CURRENT_MISSES,
    EXPANSION_SHORT_WINDOW_SELECTOR_WINDOW_SIZE, REALTIME_PREVIEW_ANALYSIS_HOP,
    REALTIME_PREVIEW_WINDOW_SIZE, SIGNAL_STRETCH_BACKEND_PLAN,
};
pub use stretch_engine::{StretchRenderError, MAX_OFFLINE_STRETCH_OUTPUT_SAMPLES};

// Crate-private helpers previously defined in lib.rs; sibling modules import via `crate::`.
#[cfg(any(test, feature = "evidence"))]
pub(crate) use stretch_engine::dynamic_ratio_output_boundaries;
#[allow(unused_imports)] // re-exported for sibling modules and unit tests via `crate::`
pub(crate) use stretch_engine::{
    abs_diff_frames, align_to_next_grid, ceil_frame_to_u64, ceil_frame_to_usize,
    coalesce_short_dynamic_ratio_segments, dynamic_ratio_output_frames, dynamic_ratio_segments,
    floor_frame_to_u64, min_dynamic_ratio_segment_frames, sanitize_ratio,
    should_select_expansion_short_window, smooth_dynamic_segment_boundaries_interleaved,
    stretch_dynamic_ratio_linked_stereo_with_engine, stretch_dynamic_ratio_mono_with_engine,
    stretch_to_exact_linked_stereo, usize_to_u64, wrap_phase,
};

#[cfg(test)]
mod dynamic_segment_seam_evidence;
#[cfg(test)]
mod tests;
