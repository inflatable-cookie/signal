//! Time-stretching backends for the Signal workspace.
//!
//! The crate defines the abstract [`TimeStretcher`] contract — stretch audio
//! in time without shifting pitch — and ships two offline backends this round:
//! [`PhaseVocoderStretcher`], a dependency-light draft-quality phase vocoder,
//! and [`OfflineHighQualityStretcher`], Signal's offline-quality foundation
//! for corpus-gated export, freeze, and cache artifacts.
//!
//! ## Signal-owned backend tiers
//!
//! Signal owns three execution tiers:
//!
//! - [`StretchBackendTier::Repitch`]: render-plane rate conversion, pitch
//!   changes with tempo, realtime-safe today.
//! - [`StretchBackendTier::RealtimePreview`]: bounded-latency pitch-preserving
//!   preview stretch, planned.
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
//! ordinary sample buffer; a bounded-latency streaming tier is future work
//! behind the same trait.

#![warn(missing_docs)]

mod benchmark;
mod cache_identity;
mod corpus_report;
mod phase_vocoder;
mod promotion;

pub use benchmark::{
    assess_stretch_metrics, compare_sustained_material_coherence,
    compare_synthetic_stretch_backends, detect_stretch_transients,
    format_stretch_acceptance_report, format_stretch_quality_priority_report,
    format_synthetic_stretch_comparison_report, generate_synthetic_stretch_audio,
    measure_draft_loop_boundary_click, measure_draft_stereo_image_delta,
    measure_draft_transient_smear, measure_dynamic_segment_seam_click, measure_loop_boundary_click,
    measure_pitch_shift_error_cents, measure_stereo_image_delta,
    measure_transient_reset_loop_boundary_click, measure_transient_reset_stereo_image_delta,
    measure_transient_reset_transient_smear, measure_transient_smear, output_length_drift_samples,
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
    StretchTransientEvent, StretchTransientSmearMeasurement, STRETCH_BENCHMARK_CORPUS,
    STRETCH_CORPUS_MANIFEST, STRETCH_CORPUS_MANIFEST_ENTRIES, STRETCH_CORPUS_SOURCE_POLICY,
};
pub use cache_identity::{
    StretchCacheIdentity, StretchCacheIdentityError, StretchCacheIdentityInput,
    StretchChannelLayout, StretchPitchPoint, StretchRatioPoint, StretchWarpMarker,
    SIGNAL_STRETCH_ENGINE_VERSION, STRETCH_CACHE_IDENTITY_SCHEMA_VERSION,
};
pub use corpus_report::{
    build_stretch_corpus_comparison_report, format_stretch_corpus_comparison_report,
    StretchCorpusComparisonReport, StretchCorpusListeningNoteSlot, StretchCorpusSkippedAsset,
};
pub use promotion::{
    current_synthetic_offline_high_quality_promotion_receipt, StretchPromotionReceipt,
    StretchPromotionStatus, StretchSyntheticPromotionPolicy,
};

use phase_vocoder::{
    phase_vocoder, transient_reset_phase_vocoder, transient_reset_phase_vocoder_linked_stereo,
};
use signal_dsp_resample::{resample_mono, ResampleConfig, ResampleQuality};
use signal_primitives::{Sample, SampleRate};

/// Quality tier of a stretch backend (memo 013 vocabulary). One tier exists
/// today; real-time and offline production tiers land with the library
/// evaluation (P-TS-001).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StretchQuality {
    /// Draft-quality phase vocoder: pitch-preserving, but transients smear
    /// and no formant handling. Offline use only.
    Draft,
    /// Bounded-latency preview quality. Planned; not implemented by the
    /// current backend.
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
    /// Planned bounded-latency preview tier for live audition and playback.
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
        status: StretchBackendStatus::Planned,
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
/// v1 scope is offline whole-buffer processing; the streaming/RT surface
/// (bounded latency, PDC reporting, variable ratio mid-stream) extends this
/// trait when a production backend lands.
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
}

/// Default STFT window: 2048 samples (~43 ms at 48 kHz).
pub const DEFAULT_WINDOW_SIZE: usize = 2_048;
/// Default analysis hop: window / 4 (75% overlap).
pub const DEFAULT_ANALYSIS_HOP: usize = DEFAULT_WINDOW_SIZE / 4;

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
        };
        stretcher.set_ratio(ratio);
        stretcher
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

        transient_reset_phase_vocoder_linked_stereo(
            even_frames,
            target_frames,
            self.ratio,
            self.window_size,
            self.analysis_hop,
        )
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
        stretch_mono_with_engine(
            input,
            self.ratio,
            self.window_size,
            self.analysis_hop,
            transient_reset_phase_vocoder,
        )
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
    let mut boundaries = Vec::with_capacity(segments.len().saturating_sub(1));
    let mut output_frame = 0usize;
    let total_frames: usize = segments.iter().map(|segment| segment.target_frames).sum();
    for segment in segments.iter().take(segments.len().saturating_sub(1)) {
        output_frame += segment.target_frames;
        if output_frame > 0 && output_frame < total_frames {
            boundaries.push(output_frame);
        }
    }
    boundaries
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
    }
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
        assert_eq!(preview.status, StretchBackendStatus::Planned);
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
        assert!(formatted.contains("offline_hq="));
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
        assert!(formatted.contains("summary comparisons=27 missing_assets=5"));
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
    fn transient_detector_stays_quiet_on_plain_sustain() {
        let input = sine(440.0, 48_000.0, 48_000);
        let events = detect_stretch_transients(&input, 1024, 256);

        assert!(
            events.len() <= 1,
            "plain sustain should not generate repeated transient events: {events:?}"
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
