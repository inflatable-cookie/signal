//! Rhythm analysis surfaces for Signal.
//!
//! The crate currently exposes offline beat, tempo, and limited meter analysis
//! built on a multifeature onset envelope and STFT-derived rhythm cues.
//!
//! ```no_run
//! use signal_analysis::AnalysisStage;
//! use signal_analysis_rhythm::{BeatTracker, BeatTrackerConfig};
//! use signal_primitives::{AudioBuffer, ChannelLayout, SampleRate};
//!
//! let audio = AudioBuffer::from_interleaved(
//!     SampleRate(48_000),
//!     ChannelLayout::Mono,
//!     vec![0.0; 96_000],
//! );
//! let mut tracker = BeatTracker::new(BeatTrackerConfig::default());
//! let result = tracker.analyze(&audio);
//!
//! assert_eq!(tracker.mode(), signal_analysis::AnalysisMode::Offline);
//! assert!(result.beat_positions_seconds.is_empty() || result.bpm >= 0.0);
//! ```
mod beat_tempo_core;
mod onset_features;
mod rhythm_policy;
mod tempo_interpretation_runtime;
mod tempo_policy;
mod tempo_state_continuity_basics;
mod tempo_state_continuity_refresh;
mod tempo_state_continuity_transition;

use beat_tempo_core::{
    beat_frames_to_seconds, beat_frames_to_seconds_refined, combined_confidence, estimate_tempo,
    refine_beat_frames, refine_bpm_from_beats, track_beats,
};
use onset_features::{band_profile_change, low_band_flux, multifeature_onset_envelope};
pub use rhythm_policy::*;
use rhythm_policy::{
    meter_confidence_breakdown, meter_hypotheses, meter_hypothesis_confidence,
    meter_recommendation, meter_recovery_context, meter_support_profile, meter_trust_level,
    rhythm_structure_ambiguity_summary, select_segment_meter_candidate,
    trailing_meter_window_candidate, MeterHypothesis,
};
use signal_analysis::{
    prepare_audio_analysis, prepare_mono_analysis, AnalysisInputConfig, AnalysisMode,
    AnalysisStage, Confidence,
};
use signal_dsp_spectral::{Stft, StftConfig};
use signal_primitives::{AudioBuffer, Sample, SampleRate, Seconds};
use tempo_interpretation_runtime::interpret_tempo;
pub use tempo_policy::*;
use tempo_policy::{analyze_local_tempo, tempo_summary};
use tempo_state_continuity_basics::{
    continuity_cause_stack, continuity_history, continuity_provenance, continuity_severity,
    continuity_trigger, has_tempo_cause, unresolved_span,
};
use tempo_state_continuity_refresh::continuity_refresh_strength;
use tempo_state_continuity_transition::{continuity_expiry, continuity_transition};

/// Controls the trade-off between speed and accuracy in rhythm analysis.
///
/// Each tier configures the FFT size, onset-feature set, segment duration,
/// and meter inference to match a different use case.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AnalysisProfile {
    /// 30-second centre segment, 1024-point FFT, no phase computation,
    /// three onset features, no meter inference.  Suitable for rapid
    /// library scanning.  ~20× faster than [`High`](AnalysisProfile::High)
    /// on a 4-minute track.
    Low,
    /// 60-second centre segment, 1024-point FFT, no phase computation,
    /// three onset features, with meter inference.  Balanced accuracy and
    /// performance for interactive use.  ~5× faster than
    /// [`High`](AnalysisProfile::High).
    Medium,
    /// Full track, 2048-point FFT with phases, all five onset features,
    /// full meter inference and diagnostics.  Maximum accuracy.
    High,
}

/// Configuration for the offline beat tracker.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BeatTrackerConfig {
    pub stft: StftConfig,
    pub min_bpm: f32,
    pub max_bpm: f32,
    pub beat_tolerance: f32,
    /// Sample rate used by the rhythm analysis path after input prep.
    ///
    /// Freezing the analysis rate keeps onset framing and tempo heuristics on
    /// one stable domain across source material with different native rates.
    pub analysis_sample_rate: SampleRate,
    /// When set, only analyze this many seconds from the centre of the track.
    /// Dramatically reduces processing time for long audio files.
    pub analysis_duration_seconds: Option<f32>,
    /// Controls the speed/accuracy trade-off.  See [`AnalysisProfile`].
    pub profile: AnalysisProfile,
}

impl Default for BeatTrackerConfig {
    fn default() -> Self {
        Self::high()
    }
}

impl BeatTrackerConfig {
    /// Fastest preset — 30-second centre segment, small FFT, reduced
    /// onset features, no meter.  ~20× faster than [`high`](Self::high).
    pub fn low() -> Self {
        Self {
            stft: StftConfig {
                window_size: signal_primitives::FrameCount(1024),
                hop_size: signal_primitives::FrameCount(512),
                compute_phases: false,
            },
            min_bpm: 70.0,
            max_bpm: 180.0,
            beat_tolerance: 0.2,
            analysis_sample_rate: SampleRate(48_000),
            analysis_duration_seconds: Some(30.0),
            profile: AnalysisProfile::Low,
        }
    }

    /// Balanced preset — 60-second centre segment, small FFT, reduced
    /// onset features, with meter.  ~5× faster than [`high`](Self::high).
    pub fn medium() -> Self {
        Self {
            stft: StftConfig {
                window_size: signal_primitives::FrameCount(1024),
                hop_size: signal_primitives::FrameCount(512),
                compute_phases: false,
            },
            min_bpm: 70.0,
            max_bpm: 180.0,
            beat_tolerance: 0.2,
            analysis_sample_rate: SampleRate(48_000),
            analysis_duration_seconds: Some(60.0),
            profile: AnalysisProfile::Medium,
        }
    }

    /// Full-accuracy preset — entire track, large FFT with phases, all
    /// five onset features, full meter and diagnostics.
    pub fn high() -> Self {
        Self {
            stft: StftConfig::new(2048, 512),
            min_bpm: 70.0,
            max_bpm: 180.0,
            beat_tolerance: 0.2,
            analysis_sample_rate: SampleRate(48_000),
            analysis_duration_seconds: None,
            profile: AnalysisProfile::High,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct TempoHypothesis {
    bpm: f32,
    lag_frames: usize,
    refined_lag_frames: f32,
    phase_offset_frames: usize,
    phase_score: f32,
    score: f32,
    confidence: Confidence,
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct TempoEstimate {
    bpm: f32,
    confidence: Confidence,
    lag_frames: usize,
    phase_offset_frames: usize,
    candidates: [Option<TempoHypothesis>; 3],
    ambiguity: Confidence,
}

#[derive(Clone, Copy, Debug)]
struct MeterSuppressionProfile {
    best_confidence: Confidence,
    best_support: f32,
    best_regularity: f32,
    trailing_confidence: Confidence,
    trailing_recent_stability: f32,
}

struct MeterDecision {
    estimate: Option<MeterEstimate>,
    suppression_profile: MeterSuppressionProfile,
    ambiguity: RhythmStructureAmbiguitySummary,
}

/// Offline beat, tempo, and meter tracker for mono audio.
#[derive(Debug, Default)]
pub struct BeatTracker {
    config: BeatTrackerConfig,
}

impl BeatTracker {
    /// Create a beat tracker with the provided config.
    pub fn new(config: BeatTrackerConfig) -> Self {
        Self { config }
    }

    /// Return the current tracker config.
    pub fn config(&self) -> BeatTrackerConfig {
        self.config
    }

    /// Analyze a mono sample slice directly.
    pub fn analyze_mono(
        &mut self,
        sample_rate: SampleRate,
        mono_samples: &[Sample],
    ) -> BeatAnalysisResult {
        let prepared =
            prepare_mono_analysis(sample_rate, mono_samples, self.analysis_input_config());
        self.analyze_prepared(prepared.sample_rate, &prepared.samples)
    }

    fn analysis_input_config(&self) -> AnalysisInputConfig {
        AnalysisInputConfig {
            max_duration: self.config.analysis_duration_seconds.map(Seconds),
            target_sample_rate: Some(self.config.analysis_sample_rate),
            ..AnalysisInputConfig::default()
        }
    }

    fn analyze_prepared(
        &self,
        sample_rate: SampleRate,
        mono_samples: &[Sample],
    ) -> BeatAnalysisResult {
        let hop_size = self.config.stft.hop_size.0.max(1);
        let profile = self.config.profile;
        let reduced_features = !matches!(profile, AnalysisProfile::High);

        let stft = Stft::new(self.config.stft);
        let spectrogram = stft.analyze_mono(sample_rate, mono_samples);
        let onset_envelope = multifeature_onset_envelope(
            &spectrogram,
            mono_samples,
            sample_rate,
            hop_size,
            reduced_features,
        );
        let tempo = estimate_tempo(
            &onset_envelope,
            sample_rate,
            hop_size,
            self.config.min_bpm,
            self.config.max_bpm,
        );

        let beat_frames = track_beats(
            &onset_envelope,
            tempo.lag_frames,
            tempo.phase_offset_frames,
            self.config.beat_tolerance,
        );
        let refined_beat_frames = refine_beat_frames(&onset_envelope, &beat_frames);
        let refined_bpm =
            refine_bpm_from_beats(tempo.bpm, &refined_beat_frames, sample_rate, hop_size);
        let beat_positions_seconds =
            beat_frames_to_seconds_refined(&refined_beat_frames, sample_rate, hop_size);

        // Meter inference is skipped at the Low tier — it adds cost without
        // value when the caller only needs a BPM estimate.
        let meter_decision = if matches!(profile, AnalysisProfile::Low) {
            MeterDecision {
                estimate: None,
                suppression_profile: MeterSuppressionProfile {
                    best_confidence: Confidence::new(0.0),
                    best_support: 0.0,
                    best_regularity: 0.0,
                    trailing_confidence: Confidence::new(0.0),
                    trailing_recent_stability: 0.0,
                },
                ambiguity: RhythmStructureAmbiguitySummary {
                    kind: RhythmStructureAmbiguityKind::InsufficientEvidence,
                    confidence: Confidence::new(0.0),
                    primary: None,
                    runner_up: None,
                    trailing_recovery_confidence: Confidence::new(0.0),
                },
            }
        } else {
            let low_band_cue = low_band_flux(&spectrogram, 180.0);
            let profile_change_cue = band_profile_change(&spectrogram, 5);
            let meter_cue = combine_meter_cues(&low_band_cue, &profile_change_cue);
            infer_meter(
                &onset_envelope,
                &meter_cue,
                &beat_frames,
                sample_rate,
                hop_size,
            )
        };

        let confidence = combined_confidence(
            &onset_envelope,
            tempo.confidence,
            &beat_positions_seconds,
            refined_bpm,
        );
        let tempo_diagnostics = analyze_local_tempo(&beat_positions_seconds);
        let tempo_interpretation =
            interpret_tempo(refined_bpm, confidence, tempo.ambiguity, &tempo_diagnostics);
        let tempo_state = tempo_state_recommendation_with_scope(
            tempo_interpretation,
            confidence,
            tempo.ambiguity,
            tempo_diagnostics.stability_scope,
        );
        let meter_state = meter_state_recommendation(
            meter_decision.estimate.as_ref(),
            meter_decision.suppression_profile,
            confidence,
            tempo.ambiguity,
            refined_bpm,
            &beat_positions_seconds,
        );
        let output_bpm = match profile {
            // Low and Medium use a simple integer snap whose tolerance is
            // appropriate for the reduced segment lengths.  The full
            // interpretation pipeline is tuned for whole-track statistics and
            // may fail to trigger on shorter segments.
            AnalysisProfile::Low | AnalysisProfile::Medium => {
                let nearest = refined_bpm.round();
                if (refined_bpm - nearest).abs() < 0.15 && confidence.0 >= 0.4 {
                    nearest
                } else {
                    refined_bpm
                }
            }
            // High uses the full interpretation pipeline with its carefully
            // calibrated snap thresholds.
            AnalysisProfile::High => tempo_interpretation
                .snapped_bpm
                .filter(|_| {
                    matches!(
                        tempo_interpretation.recommendation,
                        TempoRecommendation::SnapInteger
                    )
                })
                .unwrap_or(refined_bpm),
        };
        let mut tempo_candidates: Vec<TempoCandidate> = tempo
            .candidates
            .into_iter()
            .flatten()
            .map(|candidate| TempoCandidate {
                bpm: candidate.bpm,
                confidence: candidate.confidence,
            })
            .collect();
        if let Some(primary_candidate) = tempo_candidates.first_mut() {
            primary_candidate.bpm = output_bpm;
        }

        BeatAnalysisResult {
            bpm: output_bpm,
            confidence,
            beat_positions_seconds,
            onset_envelope,
            tempo_candidates,
            tempo_diagnostics,
            tempo_interpretation,
            tempo_state,
            tempo_ambiguity: tempo.ambiguity,
            meter_state,
            meter: meter_decision.estimate,
            structure_ambiguity: meter_decision.ambiguity,
        }
    }
}

impl AnalysisStage<BeatAnalysisResult> for BeatTracker {
    fn mode(&self) -> AnalysisMode {
        AnalysisMode::Offline
    }

    fn analyze(&mut self, audio: &AudioBuffer) -> BeatAnalysisResult {
        let prepared = prepare_audio_analysis(audio, self.analysis_input_config());
        self.analyze_prepared(prepared.sample_rate, &prepared.samples)
    }
}

#[cfg(test)]
fn default_tempo_stability_scope() -> TempoStabilityScopeSummary {
    TempoStabilityScopeSummary {
        scope: TempoStabilityScope::WholeTrackStable,
        support: TempoStabilityScopeSupport {
            edge_trimmed_coverage: Confidence::new(1.0),
            contiguous_core_coverage: Confidence::new(1.0),
            interior_stability: Confidence::new(1.0),
            edge_locality: Confidence::new(0.0),
        },
    }
}

#[cfg(test)]
fn tempo_state_recommendation(
    interpretation: TempoInterpretation,
    confidence: Confidence,
    tempo_ambiguity: Confidence,
) -> TempoStateRecommendation {
    tempo_state_recommendation_with_scope(
        interpretation,
        confidence,
        tempo_ambiguity,
        default_tempo_stability_scope(),
    )
}

fn tempo_state_recommendation_with_scope(
    interpretation: TempoInterpretation,
    confidence: Confidence,
    tempo_ambiguity: Confidence,
    stability_scope: TempoStabilityScopeSummary,
) -> TempoStateRecommendation {
    fn continuity_arc_support(
        unresolved: TempoContinuityUnresolvedSpan,
        causes: TempoContinuityCauseStack,
        current: TempoContinuityHistory,
        refresh: TempoContinuityTransition,
        first_decay: TempoContinuityTransition,
        final_decay: TempoContinuityTransition,
    ) -> TempoContinuityArcSupport {
        let refresh_bonus = match refresh.history {
            TempoContinuityHistory::Reinforcing => 0.26,
            TempoContinuityHistory::Preserving => 0.12,
            TempoContinuityHistory::Degrading => 0.0,
        };
        let current_bonus = match current {
            TempoContinuityHistory::Reinforcing => 0.18,
            TempoContinuityHistory::Preserving => 0.08,
            TempoContinuityHistory::Degrading => 0.0,
        };
        let decay_penalty = match first_decay.history {
            TempoContinuityHistory::Degrading => 0.08,
            _ => 0.0,
        } + match final_decay.history {
            TempoContinuityHistory::Degrading => 0.12,
            _ => 0.0,
        };
        let refresh_strength = Confidence::new(
            (refresh.refresh_strength.0 + refresh_bonus + current_bonus - decay_penalty)
                .clamp(0.0, 1.0),
        );

        let drift_pressure = Confidence::new(
            ((unresolved.failed_revalidations as f32 * 0.20)
                + match current {
                    TempoContinuityHistory::Degrading => 0.18,
                    TempoContinuityHistory::Preserving => 0.08,
                    TempoContinuityHistory::Reinforcing => 0.0,
                }
                + match first_decay.history {
                    TempoContinuityHistory::Degrading => 0.14,
                    _ => 0.0,
                }
                + match final_decay.history {
                    TempoContinuityHistory::Degrading => 0.18,
                    _ => 0.0,
                })
            .clamp(0.0, 1.0),
        );

        let instability_pressure = Confidence::new(
            ((if has_tempo_cause(causes, TempoContinuityCause::BoundaryDrift) {
                0.28_f32
            } else {
                0.0
            }) + (if has_tempo_cause(causes, TempoContinuityCause::TempoAmbiguity) {
                0.18
            } else {
                0.0
            }) + (if has_tempo_cause(causes, TempoContinuityCause::PriorTempoCarry) {
                0.16
            } else {
                0.0
            }) + (if has_tempo_cause(causes, TempoContinuityCause::CoreWindowCarry) {
                0.10
            } else {
                0.0
            }) + (if has_tempo_cause(causes, TempoContinuityCause::EvidenceLoss) {
                0.40
            } else {
                0.0
            }))
            .clamp(0.0, 1.0),
        );

        TempoContinuityArcSupport {
            refresh_strength,
            drift_pressure,
            instability_pressure,
        }
    }

    pub(crate) fn continuity_arc_assessment(
        source: TempoContinuitySource,
        confidence: Confidence,
        unresolved: TempoContinuityUnresolvedSpan,
        causes: TempoContinuityCauseStack,
        current: TempoContinuityHistory,
        refresh: TempoContinuityTransition,
        first_decay: TempoContinuityTransition,
        final_decay: TempoContinuityTransition,
    ) -> (
        TempoContinuityArc,
        TempoContinuityArcRationale,
        TempoContinuityArcSupport,
    ) {
        let has_evidence_loss = has_tempo_cause(causes, TempoContinuityCause::EvidenceLoss);
        let has_boundary = has_tempo_cause(causes, TempoContinuityCause::BoundaryDrift);
        let has_prior_carry = has_tempo_cause(causes, TempoContinuityCause::PriorTempoCarry);
        let persistent_decay = matches!(first_decay.history, TempoContinuityHistory::Degrading)
            && matches!(final_decay.history, TempoContinuityHistory::Degrading);
        let support = continuity_arc_support(
            unresolved,
            causes,
            current,
            refresh,
            first_decay,
            final_decay,
        );

        if matches!(current, TempoContinuityHistory::Degrading)
            && (persistent_decay || has_evidence_loss)
        {
            return (
                TempoContinuityArc::Collapsing,
                if has_evidence_loss {
                    TempoContinuityArcRationale::EvidenceLoss
                } else {
                    TempoContinuityArcRationale::UnresolvedDrift
                },
                support,
            );
        }

        if matches!(refresh.history, TempoContinuityHistory::Reinforcing) && !has_evidence_loss {
            if matches!(current, TempoContinuityHistory::Reinforcing) {
                return (
                    TempoContinuityArc::Recovering,
                    TempoContinuityArcRationale::RefreshStrength,
                    support,
                );
            }

            if matches!(current, TempoContinuityHistory::Preserving)
                && confidence.0 >= 0.56
                && unresolved.failed_revalidations <= 1
                && !has_prior_carry
            {
                return (
                    TempoContinuityArc::Recovering,
                    TempoContinuityArcRationale::RefreshStrength,
                    support,
                );
            }
        }

        if has_evidence_loss
            || (persistent_decay && confidence.0 < 0.24)
            || (matches!(current, TempoContinuityHistory::Degrading)
                && !matches!(refresh.history, TempoContinuityHistory::Reinforcing))
        {
            return (
                TempoContinuityArc::Collapsing,
                if has_evidence_loss {
                    TempoContinuityArcRationale::EvidenceLoss
                } else if has_boundary {
                    TempoContinuityArcRationale::BoundaryDrift
                } else {
                    TempoContinuityArcRationale::UnresolvedDrift
                },
                support,
            );
        }

        (
            TempoContinuityArc::Stalling,
            if has_boundary || matches!(source, TempoContinuitySource::CoreWindow) {
                TempoContinuityArcRationale::BoundaryDrift
            } else if unresolved.failed_revalidations >= 2 || has_prior_carry {
                TempoContinuityArcRationale::UnresolvedDrift
            } else {
                TempoContinuityArcRationale::StableCarry
            },
            support,
        )
    }

    pub(crate) fn continuity_arc_decision(
        arc: TempoContinuityArc,
        rationale: TempoContinuityArcRationale,
        support: TempoContinuityArcSupport,
        severity: TempoContinuitySeverity,
        history: TempoContinuityHistory,
        trigger: TempoContinuityTrigger,
        causes: TempoContinuityCauseStack,
        provenance: TempoContinuityProvenance,
        expiry: TempoContinuityExpiry,
        trusted_beats: usize,
        revalidate_after_beats: usize,
        confidence: Confidence,
        unresolved: TempoContinuityUnresolvedSpan,
        refresh: TempoContinuityTransition,
        first_decay: TempoContinuityTransition,
        final_decay: TempoContinuityTransition,
    ) -> TempoContinuityArcDecision {
        let cause_stack = causes;
        let action_expiry = |action: TempoContinuityArcAction| -> TempoContinuityArcActionExpiry {
            let guaranteed_until_beats = match action {
                TempoContinuityArcAction::LockCurrentTempo => trusted_beats,
                TempoContinuityArcAction::PreferCoreWindowTempo => trusted_beats
                    .min(revalidate_after_beats.saturating_mul(2))
                    .max(1),
                TempoContinuityArcAction::PreservePriorTempo => {
                    trusted_beats.min(revalidate_after_beats).max(1)
                }
                TempoContinuityArcAction::ReacquireCurrentTempo => trusted_beats.max(1),
                TempoContinuityArcAction::ClearTempo => 0,
            };
            let fallback_after_beats = match action {
                TempoContinuityArcAction::LockCurrentTempo => expiry.downgrade_after_beats,
                TempoContinuityArcAction::PreferCoreWindowTempo => {
                    expiry.downgrade_after_beats.min(expiry.clear_after_beats)
                }
                TempoContinuityArcAction::PreservePriorTempo
                | TempoContinuityArcAction::ReacquireCurrentTempo => expiry.clear_after_beats,
                TempoContinuityArcAction::ClearTempo => 0,
            };
            let max_failed_revalidations = match action {
                TempoContinuityArcAction::LockCurrentTempo => expiry.max_failed_revalidations,
                TempoContinuityArcAction::PreferCoreWindowTempo
                | TempoContinuityArcAction::PreservePriorTempo => {
                    expiry.max_failed_revalidations.min(2).max(1)
                }
                TempoContinuityArcAction::ReacquireCurrentTempo => {
                    expiry.max_failed_revalidations.min(3).max(1)
                }
                TempoContinuityArcAction::ClearTempo => 0,
            };

            TempoContinuityArcActionExpiry {
                guaranteed_until_beats,
                fallback_after_beats,
                clear_after_beats: expiry.clear_after_beats,
                max_failed_revalidations,
            }
        };

        let decision_fields = |action: TempoContinuityArcAction| {
            let action_severity = match action {
                TempoContinuityArcAction::LockCurrentTempo => TempoContinuitySeverity::Confirmed,
                TempoContinuityArcAction::PreferCoreWindowTempo => TempoContinuitySeverity::Guarded,
                TempoContinuityArcAction::PreservePriorTempo => TempoContinuitySeverity::Fragile,
                TempoContinuityArcAction::ReacquireCurrentTempo => {
                    if matches!(history, TempoContinuityHistory::Reinforcing)
                        && support.refresh_strength.0 >= 0.72
                    {
                        TempoContinuitySeverity::Guarded
                    } else {
                        TempoContinuitySeverity::Fragile
                    }
                }
                TempoContinuityArcAction::ClearTempo => TempoContinuitySeverity::Cleared,
            };
            let fallback_action = match action {
                TempoContinuityArcAction::LockCurrentTempo => {
                    TempoContinuityArcAction::ReacquireCurrentTempo
                }
                TempoContinuityArcAction::PreferCoreWindowTempo => {
                    TempoContinuityArcAction::PreservePriorTempo
                }
                TempoContinuityArcAction::PreservePriorTempo
                | TempoContinuityArcAction::ReacquireCurrentTempo => {
                    TempoContinuityArcAction::ClearTempo
                }
                TempoContinuityArcAction::ClearTempo => TempoContinuityArcAction::ClearTempo,
            };
            let action_provenance = match action {
                TempoContinuityArcAction::LockCurrentTempo
                | TempoContinuityArcAction::ReacquireCurrentTempo => provenance,
                TempoContinuityArcAction::PreferCoreWindowTempo => {
                    TempoContinuityProvenance::CoreWindowEstimate
                }
                TempoContinuityArcAction::PreservePriorTempo => {
                    TempoContinuityProvenance::PriorTempoCarry
                }
                TempoContinuityArcAction::ClearTempo => TempoContinuityProvenance::NoTempo,
            };
            let downgrade_support = TempoContinuityArcDowngradeSupport {
                stability_window_pressure: Confidence::new(
                    if matches!(trigger, TempoContinuityTrigger::StableRevalidation) {
                        (0.55
                            + 0.25 * support.refresh_strength.0
                            + 0.20 * (1.0 - support.drift_pressure.0))
                            .clamp(0.0, 1.0)
                    } else {
                        0.0
                    },
                ),
                boundary_drift_pressure: Confidence::new(
                    ((if matches!(trigger, TempoContinuityTrigger::BoundaryDrift) {
                        0.45_f32
                    } else {
                        0.0
                    }) + if has_tempo_cause(cause_stack, TempoContinuityCause::BoundaryDrift) {
                        0.35
                    } else {
                        0.0
                    } + if has_tempo_cause(cause_stack, TempoContinuityCause::CoreWindowCarry) {
                        0.15
                    } else {
                        0.0
                    } + 0.10 * support.drift_pressure.0)
                        .clamp(0.0, 1.0),
                ),
                ambiguity_pressure: Confidence::new(
                    ((if matches!(trigger, TempoContinuityTrigger::AmbiguityCarry) {
                        0.55_f32
                    } else {
                        0.0
                    }) + if has_tempo_cause(cause_stack, TempoContinuityCause::TempoAmbiguity) {
                        0.35
                    } else {
                        0.0
                    } + 0.10 * support.instability_pressure.0)
                        .clamp(0.0, 1.0),
                ),
                failed_revalidation_pressure: Confidence::new(
                    ((unresolved.failed_revalidations as f32 / 3.0) * 0.75
                        + if unresolved.failed_revalidations >= 2 {
                            0.20
                        } else {
                            0.0
                        })
                    .clamp(0.0, 1.0),
                ),
                evidence_loss_pressure: Confidence::new(
                    ((if matches!(trigger, TempoContinuityTrigger::EvidenceLoss) {
                        0.55_f32
                    } else {
                        0.0
                    }) + if has_tempo_cause(cause_stack, TempoContinuityCause::EvidenceLoss) {
                        0.35
                    } else {
                        0.0
                    } + if matches!(action, TempoContinuityArcAction::ClearTempo) {
                        0.10
                    } else {
                        0.0
                    })
                    .clamp(0.0, 1.0),
                ),
            };
            let downgrade_rationale = if matches!(action, TempoContinuityArcAction::ClearTempo)
                || matches!(trigger, TempoContinuityTrigger::EvidenceLoss)
            {
                TempoContinuityArcDowngradeRationale::EvidenceLoss
            } else if unresolved.failed_revalidations >= 3
                || (unresolved.failed_revalidations >= 2
                    && matches!(action, TempoContinuityArcAction::PreservePriorTempo))
            {
                TempoContinuityArcDowngradeRationale::RepeatedFailedRevalidation
            } else {
                match trigger {
                    TempoContinuityTrigger::StableRevalidation => {
                        TempoContinuityArcDowngradeRationale::StabilityWindowEnd
                    }
                    TempoContinuityTrigger::BoundaryDrift => {
                        TempoContinuityArcDowngradeRationale::BoundaryDrift
                    }
                    TempoContinuityTrigger::AmbiguityCarry => {
                        TempoContinuityArcDowngradeRationale::AmbiguityCarry
                    }
                    TempoContinuityTrigger::PriorTempoDrift => {
                        TempoContinuityArcDowngradeRationale::PriorTempoDrift
                    }
                    TempoContinuityTrigger::EvidenceLoss => {
                        TempoContinuityArcDowngradeRationale::EvidenceLoss
                    }
                }
            };
            let downgrade_trend_support = {
                let current_pressure = Confidence::new((1.0 - confidence.0).clamp(0.0, 1.0));
                let next_stage_pressure = match arc {
                    TempoContinuityArc::Recovering => {
                        Confidence::new((1.0 - refresh.refresh_strength.0).clamp(0.0, 1.0))
                    }
                    TempoContinuityArc::Stalling => {
                        Confidence::new((1.0 - first_decay.refresh_strength.0).clamp(0.0, 1.0))
                    }
                    TempoContinuityArc::Collapsing => {
                        Confidence::new((1.0 - final_decay.refresh_strength.0).clamp(0.0, 1.0))
                    }
                };
                let terminal_pressure =
                    Confidence::new((1.0 - final_decay.refresh_strength.0).clamp(0.0, 1.0));

                TempoContinuityArcDowngradeTrendSupport {
                    current_pressure,
                    next_stage_pressure,
                    terminal_pressure,
                }
            };
            let downgrade_trend = if matches!(action, TempoContinuityArcAction::ClearTempo) {
                TempoContinuityArcDowngradeTrend::Stable
            } else if downgrade_trend_support.next_stage_pressure.0
                > downgrade_trend_support.current_pressure.0 + 0.08
            {
                TempoContinuityArcDowngradeTrend::Rising
            } else if downgrade_trend_support.next_stage_pressure.0 + 0.12
                < downgrade_trend_support.current_pressure.0
            {
                TempoContinuityArcDowngradeTrend::Easing
            } else {
                TempoContinuityArcDowngradeTrend::Stable
            };
            let downgrade_trend_rationale = match downgrade_trend {
                TempoContinuityArcDowngradeTrend::Rising
                    if matches!(trigger, TempoContinuityTrigger::BoundaryDrift) =>
                {
                    TempoContinuityArcDowngradeTrendRationale::BoundaryEscalation
                }
                TempoContinuityArcDowngradeTrend::Rising => {
                    TempoContinuityArcDowngradeTrendRationale::RevalidationDecay
                }
                TempoContinuityArcDowngradeTrend::Easing
                    if matches!(trigger, TempoContinuityTrigger::AmbiguityCarry) =>
                {
                    TempoContinuityArcDowngradeTrendRationale::AmbiguityCarry
                }
                TempoContinuityArcDowngradeTrend::Easing => {
                    TempoContinuityArcDowngradeTrendRationale::StabilityWindowCarry
                }
                TempoContinuityArcDowngradeTrend::Stable
                    if matches!(action, TempoContinuityArcAction::ClearTempo) =>
                {
                    TempoContinuityArcDowngradeTrendRationale::FlatCollapse
                }
                TempoContinuityArcDowngradeTrend::Stable
                    if downgrade_trend_support.terminal_pressure.0
                        > downgrade_trend_support.current_pressure.0 + 0.12 =>
                {
                    TempoContinuityArcDowngradeTrendRationale::TerminalClearPressure
                }
                TempoContinuityArcDowngradeTrend::Stable
                    if matches!(trigger, TempoContinuityTrigger::AmbiguityCarry) =>
                {
                    TempoContinuityArcDowngradeTrendRationale::AmbiguityCarry
                }
                TempoContinuityArcDowngradeTrend::Stable
                    if matches!(trigger, TempoContinuityTrigger::BoundaryDrift) =>
                {
                    TempoContinuityArcDowngradeTrendRationale::BoundaryEscalation
                }
                TempoContinuityArcDowngradeTrend::Stable => {
                    TempoContinuityArcDowngradeTrendRationale::StabilityWindowCarry
                }
            };
            let downgrade_inflection = {
                let next_stage_after_beats = match arc {
                    TempoContinuityArc::Recovering => refresh.after_beats,
                    TempoContinuityArc::Stalling => first_decay.after_beats,
                    TempoContinuityArc::Collapsing => final_decay.after_beats,
                };
                let next_stage_delta = Confidence::new(
                    (downgrade_trend_support.next_stage_pressure.0
                        - downgrade_trend_support.current_pressure.0)
                        .abs()
                        .clamp(0.0, 1.0),
                );
                let terminal_delta = Confidence::new(
                    (downgrade_trend_support.terminal_pressure.0
                        - downgrade_trend_support.current_pressure.0)
                        .abs()
                        .clamp(0.0, 1.0),
                );
                let stage = if matches!(action, TempoContinuityArcAction::ClearTempo)
                    || (matches!(downgrade_trend, TempoContinuityArcDowngradeTrend::Stable)
                        && next_stage_delta.0 < 0.06
                        && terminal_delta.0 < 0.06)
                {
                    TempoContinuityArcDowngradeInflectionStage::FlatWindow
                } else if matches!(
                    downgrade_trend,
                    TempoContinuityArcDowngradeTrend::Rising
                        | TempoContinuityArcDowngradeTrend::Easing
                ) {
                    TempoContinuityArcDowngradeInflectionStage::NextStage
                } else if terminal_delta.0 > next_stage_delta.0 + 0.06 {
                    TempoContinuityArcDowngradeInflectionStage::TerminalClear
                } else if next_stage_delta.0 >= 0.06 {
                    TempoContinuityArcDowngradeInflectionStage::NextStage
                } else {
                    TempoContinuityArcDowngradeInflectionStage::FlatWindow
                };
                let after_beats = match stage {
                    TempoContinuityArcDowngradeInflectionStage::FlatWindow => 0,
                    TempoContinuityArcDowngradeInflectionStage::NextStage => next_stage_after_beats,
                    TempoContinuityArcDowngradeInflectionStage::TerminalClear => {
                        final_decay.after_beats
                    }
                };
                let primary_delta = match stage {
                    TempoContinuityArcDowngradeInflectionStage::FlatWindow => Confidence::new(0.0),
                    TempoContinuityArcDowngradeInflectionStage::NextStage => next_stage_delta,
                    TempoContinuityArcDowngradeInflectionStage::TerminalClear => terminal_delta,
                };
                let (competing_stage, competing_after_beats, competing_delta) = match stage {
                    TempoContinuityArcDowngradeInflectionStage::NextStage
                        if terminal_delta.0 >= 0.06
                            && terminal_delta.0 >= (primary_delta.0 * 0.55) =>
                    {
                        (
                            Some(TempoContinuityArcDowngradeInflectionStage::TerminalClear),
                            final_decay.after_beats,
                            terminal_delta,
                        )
                    }
                    TempoContinuityArcDowngradeInflectionStage::TerminalClear
                        if next_stage_delta.0 >= 0.06
                            && next_stage_delta.0 >= (primary_delta.0 * 0.55) =>
                    {
                        (
                            Some(TempoContinuityArcDowngradeInflectionStage::NextStage),
                            next_stage_after_beats,
                            next_stage_delta,
                        )
                    }
                    _ => (None, 0, Confidence::new(0.0)),
                };
                let competing_support = if primary_delta.0 > 0.0 {
                    Confidence::new((competing_delta.0 / primary_delta.0).clamp(0.0, 1.0))
                } else {
                    Confidence::new(0.0)
                };
                let balance = {
                    let modeled_total = (primary_delta.0 + competing_delta.0).clamp(0.0, 1.0);
                    let primary_weight = if modeled_total > 0.0 {
                        Confidence::new(primary_delta.0 / modeled_total)
                    } else {
                        Confidence::new(0.0)
                    };
                    let competing_weight = if modeled_total > 0.0 {
                        Confidence::new(competing_delta.0 / modeled_total)
                    } else {
                        Confidence::new(0.0)
                    };
                    let unattributed_weight = Confidence::new(1.0 - modeled_total);
                    let dominance =
                        Confidence::new((primary_weight.0 - competing_weight.0).max(0.0));

                    TempoContinuityArcDowngradeInflectionBalance {
                        primary_weight,
                        competing_weight,
                        unattributed_weight,
                        dominance,
                    }
                };
                let stage_rationale_weights =
                    |stage: TempoContinuityArcDowngradeInflectionStage,
                     stage_delta: Confidence|
                     -> TempoContinuityArcDowngradeStageRationaleWeights {
                        let has_prior_carry =
                            has_tempo_cause(cause_stack, TempoContinuityCause::PriorTempoCarry);
                        let trigger_is_stable =
                            matches!(trigger, TempoContinuityTrigger::StableRevalidation);
                        let trigger_is_boundary =
                            matches!(trigger, TempoContinuityTrigger::BoundaryDrift);
                        let trigger_is_ambiguity =
                            matches!(trigger, TempoContinuityTrigger::AmbiguityCarry);
                        let trigger_is_evidence =
                            matches!(trigger, TempoContinuityTrigger::EvidenceLoss);

                        let base = stage_delta.0.clamp(0.0, 1.0);
                        let stage_bias = match stage {
                            TempoContinuityArcDowngradeInflectionStage::FlatWindow => 0.0,
                            TempoContinuityArcDowngradeInflectionStage::NextStage => 0.18,
                            TempoContinuityArcDowngradeInflectionStage::TerminalClear => 0.12,
                        };
                        let stability_window = (if trigger_is_stable {
                            0.18 + 0.82 * downgrade_support.stability_window_pressure.0
                        } else {
                            0.35 * downgrade_support.stability_window_pressure.0
                        }) * match stage {
                            TempoContinuityArcDowngradeInflectionStage::FlatWindow => {
                                if trigger_is_stable {
                                    0.15
                                } else {
                                    0.0
                                }
                            }
                            TempoContinuityArcDowngradeInflectionStage::NextStage => 1.0,
                            TempoContinuityArcDowngradeInflectionStage::TerminalClear => 0.40,
                        };
                        let boundary_drift = (if trigger_is_boundary {
                            0.18 + 0.82 * downgrade_support.boundary_drift_pressure.0
                        } else {
                            0.55 * downgrade_support.boundary_drift_pressure.0
                        }) * match stage {
                            TempoContinuityArcDowngradeInflectionStage::FlatWindow => {
                                if trigger_is_boundary {
                                    0.20
                                } else {
                                    0.0
                                }
                            }
                            TempoContinuityArcDowngradeInflectionStage::NextStage => 1.0,
                            TempoContinuityArcDowngradeInflectionStage::TerminalClear => 0.70,
                        };
                        let ambiguity_carry = (if trigger_is_ambiguity {
                            0.18 + 0.82 * downgrade_support.ambiguity_pressure.0
                        } else {
                            0.55 * downgrade_support.ambiguity_pressure.0
                        }) * match stage {
                            TempoContinuityArcDowngradeInflectionStage::FlatWindow => {
                                if trigger_is_ambiguity {
                                    0.20
                                } else {
                                    0.0
                                }
                            }
                            TempoContinuityArcDowngradeInflectionStage::NextStage => 1.0,
                            TempoContinuityArcDowngradeInflectionStage::TerminalClear => 0.68,
                        };
                        let prior_tempo_drift = ((if has_prior_carry { 0.22 } else { 0.0 })
                            + 0.55 * downgrade_support.failed_revalidation_pressure.0)
                            * match stage {
                                TempoContinuityArcDowngradeInflectionStage::FlatWindow => {
                                    if has_prior_carry {
                                        0.25
                                    } else {
                                        0.0
                                    }
                                }
                                TempoContinuityArcDowngradeInflectionStage::NextStage => {
                                    if has_prior_carry {
                                        0.70
                                    } else {
                                        0.20
                                    }
                                }
                                TempoContinuityArcDowngradeInflectionStage::TerminalClear => {
                                    if has_prior_carry {
                                        0.82
                                    } else {
                                        0.30
                                    }
                                }
                            };
                        let revalidation_decay = (0.70
                            * downgrade_support.failed_revalidation_pressure.0)
                            * match stage {
                                TempoContinuityArcDowngradeInflectionStage::FlatWindow => {
                                    if unresolved.failed_revalidations > 0 {
                                        0.25
                                    } else {
                                        0.0
                                    }
                                }
                                TempoContinuityArcDowngradeInflectionStage::NextStage => 0.78,
                                TempoContinuityArcDowngradeInflectionStage::TerminalClear => 0.88,
                            };
                        let evidence_loss = ((if trigger_is_evidence
                            || matches!(action, TempoContinuityArcAction::ClearTempo)
                        {
                            0.18
                        } else {
                            0.0
                        }) + 0.82
                            * downgrade_support.evidence_loss_pressure.0
                            + if matches!(
                                stage,
                                TempoContinuityArcDowngradeInflectionStage::TerminalClear
                            ) {
                                0.22
                            } else {
                                0.0
                            })
                            * match stage {
                                TempoContinuityArcDowngradeInflectionStage::FlatWindow => {
                                    if matches!(action, TempoContinuityArcAction::ClearTempo) {
                                        1.0
                                    } else {
                                        0.0
                                    }
                                }
                                TempoContinuityArcDowngradeInflectionStage::NextStage => 0.62,
                                TempoContinuityArcDowngradeInflectionStage::TerminalClear => 1.0,
                            };

                        let raw_stability_window = (stability_window
                            + stage_bias * if trigger_is_stable { 1.0 } else { 0.0 })
                        .clamp(0.0, 1.0);
                        let raw_boundary_drift = (boundary_drift
                            + stage_bias * if trigger_is_boundary { 1.0 } else { 0.0 })
                        .clamp(0.0, 1.0);
                        let raw_ambiguity_carry = (ambiguity_carry
                            + stage_bias * if trigger_is_ambiguity { 1.0 } else { 0.0 })
                        .clamp(0.0, 1.0);
                        let raw_prior_tempo_drift = prior_tempo_drift.clamp(0.0, 1.0);
                        let raw_revalidation_decay = revalidation_decay.clamp(0.0, 1.0);
                        let raw_evidence_loss = evidence_loss.clamp(0.0, 1.0);

                        let total = raw_stability_window
                            + raw_boundary_drift
                            + raw_ambiguity_carry
                            + raw_prior_tempo_drift
                            + raw_revalidation_decay
                            + raw_evidence_loss;
                        if total < 0.001
                            || (matches!(
                                stage,
                                TempoContinuityArcDowngradeInflectionStage::FlatWindow
                            ) && base <= 0.0
                                && !matches!(action, TempoContinuityArcAction::ClearTempo))
                        {
                            return TempoContinuityArcDowngradeStageRationaleWeights {
                                dominant: TempoContinuityArcDowngradeStageRationale::NoPressure,
                                stability_window: Confidence::new(0.0),
                                boundary_drift: Confidence::new(0.0),
                                ambiguity_carry: Confidence::new(0.0),
                                prior_tempo_drift: Confidence::new(0.0),
                                revalidation_decay: Confidence::new(0.0),
                                evidence_loss: Confidence::new(0.0),
                            };
                        }

                        let stability_window = Confidence::new(raw_stability_window / total);
                        let boundary_drift = Confidence::new(raw_boundary_drift / total);
                        let ambiguity_carry = Confidence::new(raw_ambiguity_carry / total);
                        let prior_tempo_drift = Confidence::new(raw_prior_tempo_drift / total);
                        let revalidation_decay = Confidence::new(raw_revalidation_decay / total);
                        let evidence_loss = Confidence::new(raw_evidence_loss / total);
                        let dominant = [
                            (
                                TempoContinuityArcDowngradeStageRationale::StabilityWindow,
                                stability_window.0,
                            ),
                            (
                                TempoContinuityArcDowngradeStageRationale::BoundaryDrift,
                                boundary_drift.0,
                            ),
                            (
                                TempoContinuityArcDowngradeStageRationale::AmbiguityCarry,
                                ambiguity_carry.0,
                            ),
                            (
                                TempoContinuityArcDowngradeStageRationale::PriorTempoDrift,
                                prior_tempo_drift.0,
                            ),
                            (
                                TempoContinuityArcDowngradeStageRationale::RevalidationDecay,
                                revalidation_decay.0,
                            ),
                            (
                                TempoContinuityArcDowngradeStageRationale::EvidenceLoss,
                                evidence_loss.0,
                            ),
                        ]
                        .into_iter()
                        .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
                        .map(|entry| entry.0)
                        .unwrap_or(TempoContinuityArcDowngradeStageRationale::NoPressure);

                        TempoContinuityArcDowngradeStageRationaleWeights {
                            dominant,
                            stability_window,
                            boundary_drift,
                            ambiguity_carry,
                            prior_tempo_drift,
                            revalidation_decay,
                            evidence_loss,
                        }
                    };
                let rationale_balance = TempoContinuityArcDowngradeInflectionRationaleBalance {
                    primary: stage_rationale_weights(stage, primary_delta),
                    competing: competing_stage
                        .map(|stage| stage_rationale_weights(stage, competing_delta)),
                };

                TempoContinuityArcDowngradeInflection {
                    stage,
                    after_beats,
                    next_stage_delta,
                    terminal_delta,
                    competing_stage,
                    competing_after_beats,
                    competing_delta,
                    competing_support,
                    balance,
                    rationale_balance,
                }
            };
            let expiry = action_expiry(action);

            (
                action_severity,
                fallback_action,
                downgrade_rationale,
                downgrade_support,
                downgrade_trend,
                downgrade_trend_rationale,
                downgrade_trend_support,
                downgrade_inflection,
                action_provenance,
                expiry,
            )
        };

        match arc {
            TempoContinuityArc::Recovering
                if matches!(severity, TempoContinuitySeverity::Confirmed)
                    && matches!(history, TempoContinuityHistory::Reinforcing)
                    && unresolved.failed_revalidations == 0
                    && matches!(rationale, TempoContinuityArcRationale::RefreshStrength) =>
            {
                let action = TempoContinuityArcAction::LockCurrentTempo;
                let (
                    severity,
                    fallback_action,
                    downgrade_rationale,
                    downgrade_support,
                    downgrade_trend,
                    downgrade_trend_rationale,
                    downgrade_trend_support,
                    downgrade_inflection,
                    provenance,
                    expiry,
                ) = decision_fields(action);
                TempoContinuityArcDecision {
                    recommendation: TempoContinuityArcRecommendation::KeepLock,
                    action,
                    severity,
                    fallback_action,
                    downgrade_rationale,
                    downgrade_support,
                    downgrade_trend,
                    downgrade_trend_rationale,
                    downgrade_trend_support,
                    downgrade_inflection,
                    provenance,
                    expiry,
                    confidence: Confidence::new(
                        (0.55 * support.refresh_strength.0
                            + 0.25 * confidence.0
                            + 0.20 * (1.0 - support.instability_pressure.0))
                            .clamp(0.0, 1.0),
                    ),
                }
            }
            TempoContinuityArc::Recovering | TempoContinuityArc::Stalling => {
                let action = match arc {
                    TempoContinuityArc::Recovering => {
                        TempoContinuityArcAction::ReacquireCurrentTempo
                    }
                    TempoContinuityArc::Stalling
                        if matches!(rationale, TempoContinuityArcRationale::BoundaryDrift) =>
                    {
                        TempoContinuityArcAction::PreferCoreWindowTempo
                    }
                    TempoContinuityArc::Stalling => TempoContinuityArcAction::PreservePriorTempo,
                    TempoContinuityArc::Collapsing => TempoContinuityArcAction::ClearTempo,
                };
                let (
                    severity,
                    fallback_action,
                    downgrade_rationale,
                    downgrade_support,
                    downgrade_trend,
                    downgrade_trend_rationale,
                    downgrade_trend_support,
                    downgrade_inflection,
                    provenance,
                    expiry,
                ) = decision_fields(action);
                TempoContinuityArcDecision {
                    recommendation: TempoContinuityArcRecommendation::MonitorRecovery,
                    action,
                    severity,
                    fallback_action,
                    downgrade_rationale,
                    downgrade_support,
                    downgrade_trend,
                    downgrade_trend_rationale,
                    downgrade_trend_support,
                    downgrade_inflection,
                    provenance,
                    expiry,
                    confidence: Confidence::new(
                        (0.45 * support.refresh_strength.0
                            + 0.20 * confidence.0
                            + 0.20 * (1.0 - support.drift_pressure.0)
                            + 0.15 * (1.0 - support.instability_pressure.0))
                            .clamp(0.0, 1.0),
                    ),
                }
            }
            TempoContinuityArc::Collapsing => {
                let action = TempoContinuityArcAction::ClearTempo;
                let (
                    severity,
                    fallback_action,
                    downgrade_rationale,
                    downgrade_support,
                    downgrade_trend,
                    downgrade_trend_rationale,
                    downgrade_trend_support,
                    downgrade_inflection,
                    provenance,
                    expiry,
                ) = decision_fields(action);
                TempoContinuityArcDecision {
                    recommendation: TempoContinuityArcRecommendation::Clear,
                    action,
                    severity,
                    fallback_action,
                    downgrade_rationale,
                    downgrade_support,
                    downgrade_trend,
                    downgrade_trend_rationale,
                    downgrade_trend_support,
                    downgrade_inflection,
                    provenance,
                    expiry,
                    confidence: Confidence::new(
                        (0.50 * support.instability_pressure.0
                            + 0.30 * support.drift_pressure.0
                            + 0.20
                                * if matches!(rationale, TempoContinuityArcRationale::EvidenceLoss)
                                {
                                    1.0
                                } else {
                                    0.65
                                })
                        .clamp(0.0, 1.0),
                    ),
                }
            }
        }
    }

    fn continuity_plan(
        action: TempoContinuityAction,
        source: TempoContinuitySource,
        reason: TempoContinuityReason,
        boundary_pressure: Confidence,
        tempo_ambiguity: Confidence,
        confidence: Confidence,
        trusted_beats: usize,
        revalidate_after_beats: usize,
        refresh: TempoContinuityTransition,
        first_decay: TempoContinuityTransition,
        final_decay: TempoContinuityTransition,
    ) -> TempoContinuityPlan {
        let trigger =
            continuity_trigger(action, source, reason, boundary_pressure, tempo_ambiguity);
        let unresolved = unresolved_span(trigger, trusted_beats, revalidate_after_beats, 0);
        let causes =
            continuity_cause_stack(action, source, reason, boundary_pressure, tempo_ambiguity);
        let severity = continuity_severity(action, source);
        let history = continuity_history(action, source, reason, trigger, unresolved, causes, 0);
        let provenance = continuity_provenance(action, source, reason);
        let expiry = continuity_expiry(
            trusted_beats,
            revalidate_after_beats,
            first_decay,
            final_decay,
        );
        let (arc, arc_rationale, arc_support) = continuity_arc_assessment(
            source,
            confidence,
            unresolved,
            causes,
            history,
            refresh,
            first_decay,
            final_decay,
        );
        let arc_decision = continuity_arc_decision(
            arc,
            arc_rationale,
            arc_support,
            severity,
            history,
            trigger,
            causes,
            provenance,
            expiry,
            trusted_beats,
            revalidate_after_beats,
            confidence,
            unresolved,
            refresh,
            first_decay,
            final_decay,
        );
        TempoContinuityPlan {
            action,
            source,
            severity,
            history,
            arc,
            arc_rationale,
            arc_support,
            arc_decision,
            reason,
            trigger,
            unresolved,
            causes,
            provenance,
            confidence,
            refresh_strength: continuity_refresh_strength(
                action,
                source,
                confidence,
                history,
                unresolved,
                causes,
                trusted_beats.max(revalidate_after_beats),
            ),
            trusted_beats,
            revalidate_after_beats,
            expiry,
            lifecycle: TempoContinuityLifecycle {
                refresh,
                decay: [first_decay, final_decay],
            },
        }
    }

    let base_confidence = (0.45 * interpretation.profile.stability_score.0
        + 0.25 * confidence.0
        + 0.15 * (1.0 - tempo_ambiguity.0)
        + 0.15 * interpretation.support.grid_stability.0)
        .clamp(0.0, 1.0);
    let localized_edge_horizons = || {
        if interpretation.support.boundary_pressure.0 >= 0.20 {
            (10, 6, 12, 18, 0.60)
        } else {
            (12, 8, 14, 20, 0.64)
        }
    };
    let whole_track_scope = matches!(stability_scope.scope, TempoStabilityScope::WholeTrackStable);
    let localized_edge_scope = matches!(
        stability_scope.scope,
        TempoStabilityScope::StableWithLocalizedEdgeDamage
    );
    let core_stable_scope = matches!(stability_scope.scope, TempoStabilityScope::CoreStableOnly);
    let mid_track_unstable_scope =
        matches!(stability_scope.scope, TempoStabilityScope::MidTrackUnstable);
    let strong_integer_anchor = matches!(
        interpretation.recommendation,
        TempoRecommendation::SnapInteger
    ) && interpretation.support.integer_closeness.0 > 0.85
        && interpretation.support.core_consensus.0 > 0.8
        && interpretation.support.drift_stability.0 > 0.5
        && interpretation.support.grid_stability.0 > 0.35
        && interpretation.support.boundary_pressure.0 < 0.6;
    let ambiguity_guard = tempo_ambiguity.0 < 0.4 || strong_integer_anchor;

    match interpretation.recommendation {
        TempoRecommendation::SnapInteger
            if interpretation.trust != TempoTrustLevel::Tentative
                && (interpretation.profile.stability_score.0 >= 0.78 || strong_integer_anchor)
                && (interpretation.profile.snap_error_bpm >= 0.04
                    || interpretation.support.integer_closeness.0 > 0.9)
                && ambiguity_guard =>
        {
            if core_stable_scope || mid_track_unstable_scope {
                let state_confidence = Confidence::new(base_confidence.max(if core_stable_scope {
                    0.58
                } else {
                    0.48
                }));
                return TempoStateRecommendation {
                    action: if core_stable_scope {
                        TempoStateAction::Monitor
                    } else {
                        TempoStateAction::Defer
                    },
                    reason: if core_stable_scope {
                        TempoStateReason::CoreStableTempo
                    } else {
                        TempoStateReason::TempoDeferred
                    },
                    confidence: state_confidence,
                    continuity: continuity_plan(
                        if core_stable_scope {
                            TempoContinuityAction::Reacquire
                        } else {
                            TempoContinuityAction::Clear
                        },
                        if core_stable_scope {
                            TempoContinuitySource::CurrentTempo
                        } else {
                            TempoContinuitySource::Cleared
                        },
                        if core_stable_scope {
                            TempoContinuityReason::RevalidationDecay
                        } else {
                            TempoContinuityReason::InsufficientEvidence
                        },
                        interpretation.support.boundary_pressure,
                        tempo_ambiguity,
                        state_confidence,
                        if core_stable_scope { 4 } else { 0 },
                        if core_stable_scope { 4 } else { 0 },
                        continuity_transition(
                            if core_stable_scope { 4 } else { 0 },
                            if core_stable_scope {
                                TempoContinuityAction::Lock
                            } else {
                                TempoContinuityAction::Clear
                            },
                            if core_stable_scope {
                                TempoContinuitySource::CurrentTempo
                            } else {
                                TempoContinuitySource::Cleared
                            },
                            if core_stable_scope {
                                TempoContinuityReason::StableTempo
                            } else {
                                TempoContinuityReason::InsufficientEvidence
                            },
                            interpretation.support.boundary_pressure,
                            tempo_ambiguity,
                            if core_stable_scope { 4 } else { 0 },
                            0,
                            if core_stable_scope {
                                Confidence::new((state_confidence.0 * 0.92).clamp(0.0, 1.0))
                            } else {
                                Confidence::new(0.0)
                            },
                        ),
                        continuity_transition(
                            if core_stable_scope { 8 } else { 0 },
                            if core_stable_scope {
                                TempoContinuityAction::Reacquire
                            } else {
                                TempoContinuityAction::Clear
                            },
                            if core_stable_scope {
                                TempoContinuitySource::CurrentTempo
                            } else {
                                TempoContinuitySource::Cleared
                            },
                            if core_stable_scope {
                                TempoContinuityReason::RevalidationDecay
                            } else {
                                TempoContinuityReason::InsufficientEvidence
                            },
                            interpretation.support.boundary_pressure,
                            tempo_ambiguity,
                            if core_stable_scope { 4 } else { 0 },
                            1,
                            if core_stable_scope {
                                Confidence::new((state_confidence.0 * 0.64).clamp(0.0, 1.0))
                            } else {
                                Confidence::new(0.0)
                            },
                        ),
                        continuity_transition(
                            if core_stable_scope { 12 } else { 0 },
                            TempoContinuityAction::Clear,
                            TempoContinuitySource::Cleared,
                            TempoContinuityReason::InsufficientEvidence,
                            interpretation.support.boundary_pressure,
                            tempo_ambiguity,
                            if core_stable_scope { 4 } else { 0 },
                            2,
                            Confidence::new(0.0),
                        ),
                    ),
                };
            }
            let state_confidence = Confidence::new(base_confidence.max(if localized_edge_scope {
                0.76
            } else if strong_integer_anchor {
                0.80
            } else {
                0.82
            }));
            let (
                localized_trusted_beats,
                localized_revalidate_after_beats,
                localized_downgrade_after_beats,
                localized_clear_after_beats,
                localized_decay_confidence_scale,
            ) = localized_edge_horizons();
            TempoStateRecommendation {
                action: TempoStateAction::Lock,
                reason: if localized_edge_scope {
                    TempoStateReason::StableTempoWithEdgeDamage
                } else {
                    TempoStateReason::StableIntegerTempo
                },
                confidence: state_confidence,
                continuity: continuity_plan(
                    TempoContinuityAction::Lock,
                    TempoContinuitySource::CurrentTempo,
                    TempoContinuityReason::IntegerTempoSnap,
                    interpretation.support.boundary_pressure,
                    tempo_ambiguity,
                    state_confidence,
                    if localized_edge_scope {
                        localized_trusted_beats
                    } else {
                        16
                    },
                    if localized_edge_scope {
                        localized_revalidate_after_beats
                    } else {
                        12
                    },
                    continuity_transition(
                        if localized_edge_scope {
                            localized_revalidate_after_beats
                        } else {
                            12
                        },
                        TempoContinuityAction::Lock,
                        TempoContinuitySource::CurrentTempo,
                        TempoContinuityReason::IntegerTempoSnap,
                        interpretation.support.boundary_pressure,
                        tempo_ambiguity,
                        if localized_edge_scope {
                            localized_revalidate_after_beats
                        } else {
                            12
                        },
                        0,
                        state_confidence,
                    ),
                    continuity_transition(
                        if localized_edge_scope {
                            localized_downgrade_after_beats
                        } else {
                            20
                        },
                        TempoContinuityAction::Retain,
                        TempoContinuitySource::CurrentTempo,
                        TempoContinuityReason::RevalidationDecay,
                        interpretation.support.boundary_pressure,
                        tempo_ambiguity,
                        if localized_edge_scope {
                            localized_revalidate_after_beats
                        } else {
                            12
                        },
                        1,
                        Confidence::new(
                            (state_confidence.0
                                * if localized_edge_scope {
                                    localized_decay_confidence_scale
                                } else {
                                    0.72
                                })
                            .clamp(0.0, 1.0),
                        ),
                    ),
                    continuity_transition(
                        if localized_edge_scope {
                            localized_clear_after_beats
                        } else {
                            28
                        },
                        TempoContinuityAction::Clear,
                        TempoContinuitySource::Cleared,
                        TempoContinuityReason::InsufficientEvidence,
                        interpretation.support.boundary_pressure,
                        tempo_ambiguity,
                        if localized_edge_scope {
                            localized_revalidate_after_beats
                        } else {
                            12
                        },
                        2,
                        Confidence::new(0.0),
                    ),
                ),
            }
        }
        TempoRecommendation::UseRefined
            if interpretation.trust == TempoTrustLevel::Stable
                && interpretation.profile.stability_score.0 >= 0.72
                && interpretation.support.boundary_pressure.0 < 0.55
                && ambiguity_guard =>
        {
            if core_stable_scope || mid_track_unstable_scope {
                let state_confidence = Confidence::new(base_confidence.max(if core_stable_scope {
                    0.56
                } else {
                    0.46
                }));
                return TempoStateRecommendation {
                    action: if core_stable_scope {
                        TempoStateAction::Monitor
                    } else {
                        TempoStateAction::Defer
                    },
                    reason: if core_stable_scope {
                        TempoStateReason::CoreStableTempo
                    } else {
                        TempoStateReason::TempoDeferred
                    },
                    confidence: state_confidence,
                    continuity: continuity_plan(
                        if core_stable_scope {
                            TempoContinuityAction::Reacquire
                        } else {
                            TempoContinuityAction::Clear
                        },
                        if core_stable_scope {
                            TempoContinuitySource::CurrentTempo
                        } else {
                            TempoContinuitySource::Cleared
                        },
                        if core_stable_scope {
                            TempoContinuityReason::RevalidationDecay
                        } else {
                            TempoContinuityReason::InsufficientEvidence
                        },
                        interpretation.support.boundary_pressure,
                        tempo_ambiguity,
                        state_confidence,
                        if core_stable_scope { 4 } else { 0 },
                        if core_stable_scope { 4 } else { 0 },
                        continuity_transition(
                            if core_stable_scope { 4 } else { 0 },
                            if core_stable_scope {
                                TempoContinuityAction::Lock
                            } else {
                                TempoContinuityAction::Clear
                            },
                            if core_stable_scope {
                                TempoContinuitySource::CurrentTempo
                            } else {
                                TempoContinuitySource::Cleared
                            },
                            if core_stable_scope {
                                TempoContinuityReason::StableTempo
                            } else {
                                TempoContinuityReason::InsufficientEvidence
                            },
                            interpretation.support.boundary_pressure,
                            tempo_ambiguity,
                            if core_stable_scope { 4 } else { 0 },
                            0,
                            if core_stable_scope {
                                Confidence::new((state_confidence.0 * 0.94).clamp(0.0, 1.0))
                            } else {
                                Confidence::new(0.0)
                            },
                        ),
                        continuity_transition(
                            if core_stable_scope { 8 } else { 0 },
                            if core_stable_scope {
                                TempoContinuityAction::Reacquire
                            } else {
                                TempoContinuityAction::Clear
                            },
                            if core_stable_scope {
                                TempoContinuitySource::CurrentTempo
                            } else {
                                TempoContinuitySource::Cleared
                            },
                            if core_stable_scope {
                                TempoContinuityReason::RevalidationDecay
                            } else {
                                TempoContinuityReason::InsufficientEvidence
                            },
                            interpretation.support.boundary_pressure,
                            tempo_ambiguity,
                            if core_stable_scope { 4 } else { 0 },
                            1,
                            if core_stable_scope {
                                Confidence::new((state_confidence.0 * 0.66).clamp(0.0, 1.0))
                            } else {
                                Confidence::new(0.0)
                            },
                        ),
                        continuity_transition(
                            if core_stable_scope { 12 } else { 0 },
                            TempoContinuityAction::Clear,
                            TempoContinuitySource::Cleared,
                            TempoContinuityReason::InsufficientEvidence,
                            interpretation.support.boundary_pressure,
                            tempo_ambiguity,
                            if core_stable_scope { 4 } else { 0 },
                            2,
                            Confidence::new(0.0),
                        ),
                    ),
                };
            }
            let state_confidence = Confidence::new(base_confidence.max(if localized_edge_scope {
                0.72
            } else {
                0.76
            }));
            let (
                localized_trusted_beats,
                localized_revalidate_after_beats,
                localized_downgrade_after_beats,
                localized_clear_after_beats,
                localized_decay_confidence_scale,
            ) = localized_edge_horizons();
            TempoStateRecommendation {
                action: TempoStateAction::Lock,
                reason: if localized_edge_scope {
                    TempoStateReason::StableTempoWithEdgeDamage
                } else if whole_track_scope {
                    TempoStateReason::StableRefinedTempo
                } else {
                    TempoStateReason::StableRefinedTempo
                },
                confidence: state_confidence,
                continuity: continuity_plan(
                    TempoContinuityAction::Lock,
                    TempoContinuitySource::CurrentTempo,
                    TempoContinuityReason::StableTempo,
                    interpretation.support.boundary_pressure,
                    tempo_ambiguity,
                    state_confidence,
                    if localized_edge_scope {
                        localized_trusted_beats
                    } else {
                        16
                    },
                    if localized_edge_scope {
                        localized_revalidate_after_beats
                    } else {
                        12
                    },
                    continuity_transition(
                        if localized_edge_scope {
                            localized_revalidate_after_beats
                        } else {
                            12
                        },
                        TempoContinuityAction::Lock,
                        TempoContinuitySource::CurrentTempo,
                        TempoContinuityReason::StableTempo,
                        interpretation.support.boundary_pressure,
                        tempo_ambiguity,
                        if localized_edge_scope {
                            localized_revalidate_after_beats
                        } else {
                            12
                        },
                        0,
                        state_confidence,
                    ),
                    continuity_transition(
                        if localized_edge_scope {
                            localized_downgrade_after_beats
                        } else {
                            20
                        },
                        TempoContinuityAction::Retain,
                        TempoContinuitySource::CurrentTempo,
                        TempoContinuityReason::RevalidationDecay,
                        interpretation.support.boundary_pressure,
                        tempo_ambiguity,
                        if localized_edge_scope {
                            localized_revalidate_after_beats
                        } else {
                            12
                        },
                        1,
                        Confidence::new(
                            (state_confidence.0
                                * if localized_edge_scope {
                                    localized_decay_confidence_scale
                                } else {
                                    0.72
                                })
                            .clamp(0.0, 1.0),
                        ),
                    ),
                    continuity_transition(
                        if localized_edge_scope {
                            localized_clear_after_beats
                        } else {
                            28
                        },
                        TempoContinuityAction::Clear,
                        TempoContinuitySource::Cleared,
                        TempoContinuityReason::InsufficientEvidence,
                        interpretation.support.boundary_pressure,
                        tempo_ambiguity,
                        if localized_edge_scope {
                            localized_revalidate_after_beats
                        } else {
                            12
                        },
                        2,
                        Confidence::new(0.0),
                    ),
                ),
            }
        }
        TempoRecommendation::UseCoreWindow
            if interpretation.profile.stability_score.0 >= 0.55
                && interpretation.support.boundary_pressure.0 >= 0.45 =>
        {
            let state_confidence = Confidence::new(base_confidence.max(0.58));
            TempoStateRecommendation {
                action: TempoStateAction::Monitor,
                reason: TempoStateReason::CoreWindowFallback,
                confidence: state_confidence,
                continuity: continuity_plan(
                    TempoContinuityAction::Retain,
                    TempoContinuitySource::CoreWindow,
                    TempoContinuityReason::CoreWindowCarry,
                    interpretation.support.boundary_pressure,
                    tempo_ambiguity,
                    state_confidence,
                    8,
                    4,
                    continuity_transition(
                        4,
                        TempoContinuityAction::Retain,
                        TempoContinuitySource::CoreWindow,
                        TempoContinuityReason::CoreWindowCarry,
                        interpretation.support.boundary_pressure,
                        tempo_ambiguity,
                        4,
                        0,
                        state_confidence,
                    ),
                    continuity_transition(
                        8,
                        TempoContinuityAction::Reacquire,
                        TempoContinuitySource::PriorTempo,
                        TempoContinuityReason::RevalidationDecay,
                        interpretation.support.boundary_pressure,
                        tempo_ambiguity,
                        4,
                        1,
                        Confidence::new((state_confidence.0 * 0.68).clamp(0.0, 1.0)),
                    ),
                    continuity_transition(
                        12,
                        TempoContinuityAction::Clear,
                        TempoContinuitySource::Cleared,
                        TempoContinuityReason::InsufficientEvidence,
                        interpretation.support.boundary_pressure,
                        tempo_ambiguity,
                        4,
                        2,
                        Confidence::new(0.0),
                    ),
                ),
            }
        }
        TempoRecommendation::UseRefined
            if interpretation.trust == TempoTrustLevel::Guarded
                && interpretation.profile.stability_score.0 >= 0.58 =>
        {
            let state_confidence = Confidence::new(base_confidence.max(0.56));
            TempoStateRecommendation {
                action: TempoStateAction::Monitor,
                reason: TempoStateReason::StableRefinedTempo,
                confidence: state_confidence,
                continuity: continuity_plan(
                    TempoContinuityAction::Reacquire,
                    TempoContinuitySource::CurrentTempo,
                    TempoContinuityReason::RevalidationDecay,
                    interpretation.support.boundary_pressure,
                    tempo_ambiguity,
                    state_confidence,
                    4,
                    4,
                    continuity_transition(
                        4,
                        TempoContinuityAction::Lock,
                        TempoContinuitySource::CurrentTempo,
                        TempoContinuityReason::StableTempo,
                        interpretation.support.boundary_pressure,
                        tempo_ambiguity,
                        4,
                        0,
                        Confidence::new((state_confidence.0 * 0.96).clamp(0.0, 1.0)),
                    ),
                    continuity_transition(
                        8,
                        TempoContinuityAction::Reacquire,
                        TempoContinuitySource::CurrentTempo,
                        TempoContinuityReason::RevalidationDecay,
                        interpretation.support.boundary_pressure,
                        tempo_ambiguity,
                        4,
                        1,
                        Confidence::new((state_confidence.0 * 0.66).clamp(0.0, 1.0)),
                    ),
                    continuity_transition(
                        12,
                        TempoContinuityAction::Clear,
                        TempoContinuitySource::Cleared,
                        TempoContinuityReason::InsufficientEvidence,
                        interpretation.support.boundary_pressure,
                        tempo_ambiguity,
                        4,
                        2,
                        Confidence::new(0.0),
                    ),
                ),
            }
        }
        _ => {
            let state_confidence = Confidence::new(
                (0.55 * (1.0 - interpretation.profile.stability_score.0)
                    + 0.45 * tempo_ambiguity.0)
                    .clamp(0.0, 1.0),
            );
            TempoStateRecommendation {
                action: TempoStateAction::Defer,
                reason: TempoStateReason::TempoDeferred,
                confidence: state_confidence,
                continuity: continuity_plan(
                    TempoContinuityAction::Clear,
                    TempoContinuitySource::Cleared,
                    TempoContinuityReason::InsufficientEvidence,
                    interpretation.support.boundary_pressure,
                    tempo_ambiguity,
                    state_confidence,
                    0,
                    0,
                    continuity_transition(
                        0,
                        TempoContinuityAction::Clear,
                        TempoContinuitySource::Cleared,
                        TempoContinuityReason::InsufficientEvidence,
                        interpretation.support.boundary_pressure,
                        tempo_ambiguity,
                        0,
                        0,
                        Confidence::new(0.0),
                    ),
                    continuity_transition(
                        0,
                        TempoContinuityAction::Clear,
                        TempoContinuitySource::Cleared,
                        TempoContinuityReason::InsufficientEvidence,
                        interpretation.support.boundary_pressure,
                        tempo_ambiguity,
                        0,
                        1,
                        Confidence::new(0.0),
                    ),
                    continuity_transition(
                        0,
                        TempoContinuityAction::Clear,
                        TempoContinuitySource::Cleared,
                        TempoContinuityReason::InsufficientEvidence,
                        interpretation.support.boundary_pressure,
                        tempo_ambiguity,
                        0,
                        2,
                        Confidence::new(0.0),
                    ),
                ),
            }
        }
    }
}

fn combine_meter_cues(low_band_cue: &[f32], profile_change_cue: &[f32]) -> Vec<f32> {
    let len = low_band_cue.len().max(profile_change_cue.len());
    let mut combined = vec![0.0; len];

    for index in 0..len {
        let low = low_band_cue.get(index).copied().unwrap_or(0.0);
        let profile = profile_change_cue.get(index).copied().unwrap_or(0.0);
        combined[index] = 0.55 * low + 0.45 * profile;
    }

    normalize(&mut combined);
    combined
}

fn downbeat_frames_for_hypothesis(
    beat_frames: &[usize],
    beat_offset: usize,
    hypothesis: MeterHypothesis,
) -> Vec<usize> {
    beat_frames
        .iter()
        .skip(beat_offset + hypothesis.phase_offset_beats)
        .step_by(hypothesis.beats_per_bar)
        .copied()
        .collect()
}

fn beat_index_to_seconds(
    beat_frames: &[usize],
    beat_index: usize,
    sample_rate: SampleRate,
    hop_size: usize,
) -> f32 {
    if sample_rate.0 == 0 || hop_size == 0 {
        return 0.0;
    }

    beat_frames
        .get(beat_index)
        .copied()
        .unwrap_or_else(|| *beat_frames.last().unwrap_or(&0)) as f32
        * hop_size as f32
        / sample_rate.0 as f32
}

fn meter_state_recommendation(
    estimate: Option<&MeterEstimate>,
    suppression_profile: MeterSuppressionProfile,
    rhythm_confidence: Confidence,
    tempo_ambiguity: Confidence,
    bpm: f32,
    beat_positions_seconds: &[f32],
) -> MeterStateRecommendation {
    fn push_cause(
        slots: &mut [Option<MeterContinuityCause>; 3],
        count: &mut usize,
        cause: MeterContinuityCause,
    ) {
        if slots.iter().flatten().any(|existing| *existing == cause) {
            return;
        }
        if *count < slots.len() {
            slots[*count] = Some(cause);
            *count += 1;
        }
    }

    fn cause_stack(
        action: MeterContinuityAction,
        source: MeterContinuitySource,
        reason: MeterContinuityReason,
        trigger: MeterContinuityTrigger,
        suppression_profile: MeterSuppressionProfile,
        tempo_ambiguity: Confidence,
        phase_displaced: bool,
        stage_index: usize,
    ) -> MeterContinuityCauseStack {
        let mut causes = [None; 3];
        let mut count = 0usize;

        match reason {
            MeterContinuityReason::StableEvidence => {
                push_cause(
                    &mut causes,
                    &mut count,
                    MeterContinuityCause::StableMeterEvidence,
                );
            }
            MeterContinuityReason::PriorStateCarry => {
                push_cause(
                    &mut causes,
                    &mut count,
                    MeterContinuityCause::PriorContinuityCarry,
                );
            }
            MeterContinuityReason::RecoveryWindowSupport => {
                push_cause(
                    &mut causes,
                    &mut count,
                    MeterContinuityCause::RecoveryWindowInstability,
                );
            }
            MeterContinuityReason::PhaseDisplacement => {
                push_cause(
                    &mut causes,
                    &mut count,
                    MeterContinuityCause::PhaseDisplacement,
                );
            }
            MeterContinuityReason::InsufficientEvidence => {
                push_cause(&mut causes, &mut count, MeterContinuityCause::EvidenceLoss);
            }
            MeterContinuityReason::TentativeEvidence | MeterContinuityReason::RevalidationDecay => {
            }
        }

        match trigger {
            MeterContinuityTrigger::StableRevalidation => {
                push_cause(
                    &mut causes,
                    &mut count,
                    MeterContinuityCause::StableMeterEvidence,
                );
            }
            MeterContinuityTrigger::TentativeCarry => {
                push_cause(
                    &mut causes,
                    &mut count,
                    MeterContinuityCause::SparseMeterSupport,
                );
            }
            MeterContinuityTrigger::PhaseRecovery => {
                push_cause(
                    &mut causes,
                    &mut count,
                    MeterContinuityCause::PhaseDisplacement,
                );
            }
            MeterContinuityTrigger::PriorStateDrift => {
                push_cause(
                    &mut causes,
                    &mut count,
                    MeterContinuityCause::PriorContinuityCarry,
                );
            }
            MeterContinuityTrigger::RecoveryWindowDrift => {
                push_cause(
                    &mut causes,
                    &mut count,
                    MeterContinuityCause::RecoveryWindowInstability,
                );
            }
            MeterContinuityTrigger::EvidenceLoss => {
                push_cause(&mut causes, &mut count, MeterContinuityCause::EvidenceLoss);
            }
        }

        if phase_displaced {
            push_cause(
                &mut causes,
                &mut count,
                MeterContinuityCause::PhaseDisplacement,
            );
        }

        if tempo_ambiguity.0 >= 0.28 {
            push_cause(
                &mut causes,
                &mut count,
                MeterContinuityCause::TempoAmbiguity,
            );
        }

        if suppression_profile.best_support < 0.58 || suppression_profile.best_confidence.0 < 0.24 {
            push_cause(
                &mut causes,
                &mut count,
                MeterContinuityCause::SparseMeterSupport,
            );
        }

        if suppression_profile.best_regularity < 0.32
            || (stage_index > 0 && suppression_profile.trailing_recent_stability < 0.30)
        {
            push_cause(
                &mut causes,
                &mut count,
                MeterContinuityCause::IrregularBarStructure,
            );
        }

        if matches!(source, MeterContinuitySource::Cleared)
            || matches!(action, MeterContinuityAction::Clear)
        {
            push_cause(&mut causes, &mut count, MeterContinuityCause::EvidenceLoss);
        }

        let primary = causes[0].unwrap_or_else(|| match action {
            MeterContinuityAction::Lock => MeterContinuityCause::StableMeterEvidence,
            MeterContinuityAction::Retain | MeterContinuityAction::Reacquire => {
                MeterContinuityCause::SparseMeterSupport
            }
            MeterContinuityAction::Clear => MeterContinuityCause::EvidenceLoss,
        });

        MeterContinuityCauseStack {
            primary,
            secondary: [causes[1], causes[2]],
            count: count.max(1),
        }
    }

    fn has_cause(stack: MeterContinuityCauseStack, cause: MeterContinuityCause) -> bool {
        stack.primary == cause
            || stack
                .secondary
                .into_iter()
                .flatten()
                .any(|entry| entry == cause)
    }

    fn continuity_history(
        action: MeterContinuityAction,
        source: MeterContinuitySource,
        reason: MeterContinuityReason,
        confidence: Confidence,
        trigger: MeterContinuityTrigger,
        unresolved: MeterContinuityUnresolvedSpan,
        causes: MeterContinuityCauseStack,
        stage_index: usize,
    ) -> MeterContinuityHistory {
        let has_evidence_loss = has_cause(causes, MeterContinuityCause::EvidenceLoss);
        let has_irregularity = has_cause(causes, MeterContinuityCause::IrregularBarStructure);
        let has_phase_displacement = has_cause(causes, MeterContinuityCause::PhaseDisplacement);

        match action {
            MeterContinuityAction::Clear => MeterContinuityHistory::Degrading,
            MeterContinuityAction::Lock
                if matches!(source, MeterContinuitySource::CurrentMeter)
                    && matches!(reason, MeterContinuityReason::StableEvidence)
                    && matches!(trigger, MeterContinuityTrigger::StableRevalidation)
                    && confidence.0 >= 0.28
                    && unresolved.failed_revalidations == 0
                    && !has_evidence_loss =>
            {
                MeterContinuityHistory::Reinforcing
            }
            MeterContinuityAction::Lock => MeterContinuityHistory::Preserving,
            MeterContinuityAction::Retain
                if stage_index > 0
                    || has_evidence_loss
                    || (matches!(source, MeterContinuitySource::PriorMeter)
                        && unresolved.failed_revalidations >= 2)
                    || (matches!(source, MeterContinuitySource::RecoveryWindow)
                        && has_irregularity
                        && confidence.0 < 0.30) =>
            {
                MeterContinuityHistory::Degrading
            }
            MeterContinuityAction::Retain => MeterContinuityHistory::Preserving,
            MeterContinuityAction::Reacquire
                if matches!(reason, MeterContinuityReason::PhaseDisplacement)
                    || stage_index > 0
                    || unresolved.failed_revalidations > 0
                    || has_evidence_loss
                    || has_phase_displacement =>
            {
                MeterContinuityHistory::Degrading
            }
            MeterContinuityAction::Reacquire => MeterContinuityHistory::Preserving,
        }
    }

    fn continuity_arc_support(
        unresolved: MeterContinuityUnresolvedSpan,
        causes: MeterContinuityCauseStack,
        current: MeterContinuityHistory,
        refresh: MeterContinuityTransition,
        first_decay: MeterContinuityTransition,
        final_decay: MeterContinuityTransition,
    ) -> MeterContinuityArcSupport {
        let refresh_bonus = match refresh.history {
            MeterContinuityHistory::Reinforcing => 0.28,
            MeterContinuityHistory::Preserving => 0.12,
            MeterContinuityHistory::Degrading => 0.0,
        };
        let current_bonus = match current {
            MeterContinuityHistory::Reinforcing => 0.22,
            MeterContinuityHistory::Preserving => 0.08,
            MeterContinuityHistory::Degrading => 0.0,
        };
        let decay_penalty = match first_decay.history {
            MeterContinuityHistory::Degrading => 0.08,
            _ => 0.0,
        } + match final_decay.history {
            MeterContinuityHistory::Degrading => 0.12,
            _ => 0.0,
        };
        let refresh_strength = Confidence::new(
            (refresh.confidence.0 + refresh_bonus + current_bonus - decay_penalty).clamp(0.0, 1.0),
        );

        let drift_pressure = Confidence::new(
            ((unresolved.failed_revalidations as f32 * 0.18)
                + (unresolved.bars as f32 * 0.08)
                + match current {
                    MeterContinuityHistory::Degrading => 0.18,
                    MeterContinuityHistory::Preserving => 0.08,
                    MeterContinuityHistory::Reinforcing => 0.0,
                }
                + match first_decay.history {
                    MeterContinuityHistory::Degrading => 0.16,
                    _ => 0.0,
                }
                + match final_decay.history {
                    MeterContinuityHistory::Degrading => 0.20,
                    _ => 0.0,
                })
            .clamp(0.0, 1.0),
        );

        let evidence_loss = has_cause(causes, MeterContinuityCause::EvidenceLoss);
        let irregularity = has_cause(causes, MeterContinuityCause::IrregularBarStructure);
        let phase_displacement = has_cause(causes, MeterContinuityCause::PhaseDisplacement);
        let tempo_ambiguity = has_cause(causes, MeterContinuityCause::TempoAmbiguity);
        let structural_pressure = Confidence::new(
            ((if evidence_loss { 0.42f32 } else { 0.0f32 })
                + (if irregularity { 0.28f32 } else { 0.0f32 })
                + (if phase_displacement { 0.18f32 } else { 0.0f32 })
                + (if tempo_ambiguity { 0.12f32 } else { 0.0f32 }))
            .clamp(0.0, 1.0),
        );

        MeterContinuityArcSupport {
            refresh_strength,
            drift_pressure,
            structural_pressure,
        }
    }

    fn continuity_arc_assessment(
        source: MeterContinuitySource,
        reason: MeterContinuityReason,
        confidence: Confidence,
        unresolved: MeterContinuityUnresolvedSpan,
        causes: MeterContinuityCauseStack,
        current: MeterContinuityHistory,
        refresh: MeterContinuityTransition,
        first_decay: MeterContinuityTransition,
        final_decay: MeterContinuityTransition,
    ) -> (
        MeterContinuityArc,
        MeterContinuityArcRationale,
        MeterContinuityArcSupport,
    ) {
        let has_evidence_loss = has_cause(causes, MeterContinuityCause::EvidenceLoss);
        let has_irregularity = has_cause(causes, MeterContinuityCause::IrregularBarStructure);
        let persistent_decay = matches!(first_decay.history, MeterContinuityHistory::Degrading)
            && matches!(final_decay.history, MeterContinuityHistory::Degrading);
        let support = continuity_arc_support(
            unresolved,
            causes,
            current,
            refresh,
            first_decay,
            final_decay,
        );

        if matches!(current, MeterContinuityHistory::Degrading)
            && (persistent_decay || has_evidence_loss)
        {
            return (
                MeterContinuityArc::Collapsing,
                if has_evidence_loss {
                    MeterContinuityArcRationale::EvidenceLoss
                } else {
                    MeterContinuityArcRationale::UnresolvedDrift
                },
                support,
            );
        }

        if matches!(refresh.history, MeterContinuityHistory::Reinforcing) && !has_evidence_loss {
            if matches!(current, MeterContinuityHistory::Reinforcing) {
                return (
                    MeterContinuityArc::Recovering,
                    MeterContinuityArcRationale::RefreshStrength,
                    support,
                );
            }

            if matches!(current, MeterContinuityHistory::Preserving)
                && matches!(source, MeterContinuitySource::RecoveryWindow)
                && matches!(reason, MeterContinuityReason::RecoveryWindowSupport)
                && confidence.0 >= 0.80
                && unresolved.failed_revalidations <= 2
                && !has_irregularity
            {
                return (
                    MeterContinuityArc::Recovering,
                    MeterContinuityArcRationale::RefreshStrength,
                    support,
                );
            }
        }

        if has_evidence_loss
            || (persistent_decay && confidence.0 < 0.24)
            || (matches!(current, MeterContinuityHistory::Degrading)
                && !matches!(refresh.history, MeterContinuityHistory::Reinforcing))
        {
            return (
                MeterContinuityArc::Collapsing,
                if has_evidence_loss {
                    MeterContinuityArcRationale::EvidenceLoss
                } else if has_irregularity {
                    MeterContinuityArcRationale::StructuralInstability
                } else {
                    MeterContinuityArcRationale::UnresolvedDrift
                },
                support,
            );
        }

        (
            MeterContinuityArc::Stalling,
            if has_irregularity {
                MeterContinuityArcRationale::StructuralInstability
            } else if unresolved.failed_revalidations >= 2 {
                MeterContinuityArcRationale::UnresolvedDrift
            } else {
                MeterContinuityArcRationale::StableCarry
            },
            support,
        )
    }

    fn continuity_trigger(
        action: MeterContinuityAction,
        source: MeterContinuitySource,
        reason: MeterContinuityReason,
    ) -> MeterContinuityTrigger {
        match reason {
            MeterContinuityReason::StableEvidence => MeterContinuityTrigger::StableRevalidation,
            MeterContinuityReason::TentativeEvidence => MeterContinuityTrigger::TentativeCarry,
            MeterContinuityReason::PriorStateCarry => MeterContinuityTrigger::PriorStateDrift,
            MeterContinuityReason::RecoveryWindowSupport => {
                MeterContinuityTrigger::RecoveryWindowDrift
            }
            MeterContinuityReason::PhaseDisplacement => MeterContinuityTrigger::PhaseRecovery,
            MeterContinuityReason::RevalidationDecay => match source {
                MeterContinuitySource::PriorMeter => MeterContinuityTrigger::PriorStateDrift,
                MeterContinuitySource::RecoveryWindow => {
                    MeterContinuityTrigger::RecoveryWindowDrift
                }
                MeterContinuitySource::CurrentMeter => match action {
                    MeterContinuityAction::Retain | MeterContinuityAction::Reacquire => {
                        MeterContinuityTrigger::TentativeCarry
                    }
                    MeterContinuityAction::Lock => MeterContinuityTrigger::StableRevalidation,
                    MeterContinuityAction::Clear => MeterContinuityTrigger::EvidenceLoss,
                },
                MeterContinuitySource::Cleared => MeterContinuityTrigger::EvidenceLoss,
            },
            MeterContinuityReason::InsufficientEvidence => MeterContinuityTrigger::EvidenceLoss,
        }
    }

    fn unresolved_span(
        trigger: MeterContinuityTrigger,
        beat_span: usize,
        revalidate_after_beats: usize,
        beats_per_bar: usize,
        phase_displacement_beats: usize,
        stage_index: usize,
    ) -> MeterContinuityUnresolvedSpan {
        let beats = match trigger {
            MeterContinuityTrigger::StableRevalidation => 0,
            MeterContinuityTrigger::PhaseRecovery => beat_span
                .max(revalidate_after_beats)
                .max(phase_displacement_beats.max(1)),
            MeterContinuityTrigger::TentativeCarry
            | MeterContinuityTrigger::PriorStateDrift
            | MeterContinuityTrigger::RecoveryWindowDrift => {
                beat_span.max(revalidate_after_beats.max(1))
            }
            MeterContinuityTrigger::EvidenceLoss => beat_span,
        };
        let bars = if beats == 0 {
            0
        } else {
            (beats + beats_per_bar.saturating_sub(1)) / beats_per_bar.max(1)
        };
        let failed_revalidations = if beats == 0 || revalidate_after_beats == 0 {
            0
        } else {
            ((beats + revalidate_after_beats - 1) / revalidate_after_beats).max(stage_index)
        };
        MeterContinuityUnresolvedSpan {
            beats,
            bars,
            failed_revalidations,
        }
    }

    fn continuity_reason(
        action: MeterContinuityAction,
        source: MeterContinuitySource,
        phase_displaced: bool,
        is_decay: bool,
    ) -> MeterContinuityReason {
        if matches!(action, MeterContinuityAction::Clear)
            || matches!(source, MeterContinuitySource::Cleared)
        {
            return MeterContinuityReason::InsufficientEvidence;
        }

        if phase_displaced && matches!(action, MeterContinuityAction::Reacquire) {
            return MeterContinuityReason::PhaseDisplacement;
        }

        if is_decay {
            return MeterContinuityReason::RevalidationDecay;
        }

        match source {
            MeterContinuitySource::CurrentMeter => match action {
                MeterContinuityAction::Lock => MeterContinuityReason::StableEvidence,
                MeterContinuityAction::Retain | MeterContinuityAction::Reacquire => {
                    MeterContinuityReason::TentativeEvidence
                }
                MeterContinuityAction::Clear => MeterContinuityReason::InsufficientEvidence,
            },
            MeterContinuitySource::PriorMeter => MeterContinuityReason::PriorStateCarry,
            MeterContinuitySource::RecoveryWindow => MeterContinuityReason::RecoveryWindowSupport,
            MeterContinuitySource::Cleared => MeterContinuityReason::InsufficientEvidence,
        }
    }

    fn continuity_severity(
        action: MeterContinuityAction,
        source: MeterContinuitySource,
    ) -> MeterContinuitySeverity {
        match action {
            MeterContinuityAction::Lock => MeterContinuitySeverity::Confirmed,
            MeterContinuityAction::Retain => match source {
                MeterContinuitySource::CurrentMeter | MeterContinuitySource::RecoveryWindow => {
                    MeterContinuitySeverity::Guarded
                }
                MeterContinuitySource::PriorMeter => MeterContinuitySeverity::Fragile,
                MeterContinuitySource::Cleared => MeterContinuitySeverity::Cleared,
            },
            MeterContinuityAction::Reacquire => MeterContinuitySeverity::Fragile,
            MeterContinuityAction::Clear => MeterContinuitySeverity::Cleared,
        }
    }

    fn continuity_confidence(
        action: MeterContinuityAction,
        source: MeterContinuitySource,
        state_confidence: Confidence,
        beat_span: usize,
        stage_index: usize,
    ) -> Confidence {
        let action_scale = match action {
            MeterContinuityAction::Lock => 1.0,
            MeterContinuityAction::Retain => 0.72,
            MeterContinuityAction::Reacquire => 0.45,
            MeterContinuityAction::Clear => 0.0,
        };
        let source_bias = match source {
            MeterContinuitySource::CurrentMeter => 0.12,
            MeterContinuitySource::RecoveryWindow => 0.06,
            MeterContinuitySource::PriorMeter => -0.02,
            MeterContinuitySource::Cleared => -0.30,
        };
        let span_bias = (beat_span as f32 / 24.0).clamp(0.0, 0.25);
        let decay_penalty = stage_index as f32 * 0.12;
        Confidence::new(
            (state_confidence.0 * action_scale + source_bias + span_bias - decay_penalty)
                .clamp(0.0, 1.0),
        )
    }

    fn transition(
        after_beats: usize,
        action: MeterContinuityAction,
        source: MeterContinuitySource,
        reason: MeterContinuityReason,
        confidence: Confidence,
        trigger: MeterContinuityTrigger,
        unresolved: MeterContinuityUnresolvedSpan,
        causes: MeterContinuityCauseStack,
        stage_index: usize,
    ) -> MeterContinuityTransition {
        MeterContinuityTransition {
            after_beats,
            action,
            source,
            severity: continuity_severity(action, source),
            history: continuity_history(
                action,
                source,
                reason,
                confidence,
                trigger,
                unresolved,
                causes,
                stage_index,
            ),
            reason,
            confidence,
            trigger,
            unresolved,
            causes,
        }
    }

    fn continuity_plan(
        action: MeterContinuityAction,
        source: MeterContinuitySource,
        reason: MeterContinuityReason,
        confidence: Confidence,
        trigger: MeterContinuityTrigger,
        unresolved: MeterContinuityUnresolvedSpan,
        causes: MeterContinuityCauseStack,
        trusted_beats: usize,
        revalidate_after_beats: usize,
        refresh: MeterContinuityTransition,
        first_decay: MeterContinuityTransition,
        final_decay: MeterContinuityTransition,
    ) -> MeterContinuityPlan {
        let history = continuity_history(
            action, source, reason, confidence, trigger, unresolved, causes, 0,
        );
        let (arc, arc_rationale, arc_support) = continuity_arc_assessment(
            source,
            reason,
            confidence,
            unresolved,
            causes,
            history,
            refresh,
            first_decay,
            final_decay,
        );
        MeterContinuityPlan {
            action,
            source,
            severity: continuity_severity(action, source),
            history,
            arc,
            arc_rationale,
            arc_support,
            reason,
            confidence,
            trigger,
            unresolved,
            causes,
            trusted_beats,
            revalidate_after_beats,
            lifecycle: MeterContinuityLifecycle {
                refresh,
                decay: [first_decay, final_decay],
            },
        }
    }

    fn continuity_for(
        action: MeterStateAction,
        reason: MeterStateReason,
        estimate: Option<&MeterEstimate>,
        suppression_profile: MeterSuppressionProfile,
        confidence: Confidence,
        tempo_ambiguity: Confidence,
        bpm: f32,
        beat_positions_seconds: &[f32],
    ) -> MeterContinuityRecommendation {
        let beat_duration = if bpm > 0.0 { 60.0 / bpm } else { 0.0 };
        let pickup_like_phase = estimate
            .and_then(|estimate| estimate.downbeat_positions_seconds.first().copied())
            .map(|first_downbeat| beat_duration > 0.0 && first_downbeat >= beat_duration * 1.5)
            .unwrap_or(false);
        let phase_displacement_beats = estimate
            .and_then(|estimate| estimate.downbeat_positions_seconds.first().copied())
            .map(|first_downbeat| {
                if beat_duration > 0.0 {
                    let downbeat_guard = first_downbeat - beat_duration * 0.25;
                    beat_positions_seconds
                        .iter()
                        .copied()
                        .take_while(|&beat| beat < downbeat_guard)
                        .count()
                } else {
                    0
                }
            })
            .unwrap_or(0);
        let recovery_beats = estimate
            .and_then(|estimate| estimate.recovery.as_ref())
            .map(|recovery| recovery.recovered_beats)
            .unwrap_or(0);
        let beats_per_bar = estimate
            .map(|estimate| estimate.beats_per_bar)
            .unwrap_or(4)
            .max(1);
        let support_beats = ((confidence.0 * 12.0).round() as usize).clamp(2, 12);
        let retained_beats = if estimate.is_some() {
            recovery_beats.clamp(6, 24).max(support_beats)
        } else {
            support_beats
        };
        let stage = |after_beats: usize,
                     stage_action: MeterContinuityAction,
                     stage_source: MeterContinuitySource,
                     stage_reason: MeterContinuityReason,
                     stage_index: usize| {
            let stage_trigger = continuity_trigger(stage_action, stage_source, stage_reason);
            let stage_unresolved = unresolved_span(
                stage_trigger,
                after_beats,
                after_beats,
                beats_per_bar,
                phase_displacement_beats,
                stage_index,
            );
            let stage_causes = cause_stack(
                stage_action,
                stage_source,
                stage_reason,
                stage_trigger,
                suppression_profile,
                tempo_ambiguity,
                phase_displacement_beats > 0,
                stage_index,
            );
            transition(
                after_beats,
                stage_action,
                stage_source,
                stage_reason,
                continuity_confidence(
                    stage_action,
                    stage_source,
                    confidence,
                    after_beats,
                    stage_index,
                ),
                stage_trigger,
                stage_unresolved,
                stage_causes,
                stage_index,
            )
        };
        let plan = |plan_action: MeterContinuityAction,
                    plan_source: MeterContinuitySource,
                    plan_reason: MeterContinuityReason,
                    trusted_beats: usize,
                    revalidate_after_beats: usize,
                    refresh: MeterContinuityTransition,
                    first_decay: MeterContinuityTransition,
                    final_decay: MeterContinuityTransition| {
            let plan_trigger = continuity_trigger(plan_action, plan_source, plan_reason);
            let plan_unresolved = unresolved_span(
                plan_trigger,
                trusted_beats,
                revalidate_after_beats,
                beats_per_bar,
                phase_displacement_beats,
                0,
            );
            let plan_causes = cause_stack(
                plan_action,
                plan_source,
                plan_reason,
                plan_trigger,
                suppression_profile,
                tempo_ambiguity,
                phase_displacement_beats > 0,
                0,
            );
            continuity_plan(
                plan_action,
                plan_source,
                plan_reason,
                continuity_confidence(plan_action, plan_source, confidence, trusted_beats, 0),
                plan_trigger,
                plan_unresolved,
                plan_causes,
                trusted_beats,
                revalidate_after_beats,
                refresh,
                first_decay,
                final_decay,
            )
        };

        match (action, reason) {
            (MeterStateAction::Lock, _) if pickup_like_phase => MeterContinuityRecommendation {
                bar_length: plan(
                    MeterContinuityAction::Lock,
                    MeterContinuitySource::CurrentMeter,
                    MeterContinuityReason::StableEvidence,
                    16,
                    16,
                    stage(
                        16,
                        MeterContinuityAction::Lock,
                        MeterContinuitySource::CurrentMeter,
                        MeterContinuityReason::StableEvidence,
                        0,
                    ),
                    stage(
                        24,
                        MeterContinuityAction::Retain,
                        MeterContinuitySource::CurrentMeter,
                        MeterContinuityReason::RevalidationDecay,
                        1,
                    ),
                    stage(
                        32,
                        MeterContinuityAction::Clear,
                        MeterContinuitySource::Cleared,
                        MeterContinuityReason::InsufficientEvidence,
                        2,
                    ),
                ),
                downbeat_phase: plan(
                    MeterContinuityAction::Reacquire,
                    MeterContinuitySource::CurrentMeter,
                    MeterContinuityReason::PhaseDisplacement,
                    0,
                    2,
                    stage(
                        2,
                        MeterContinuityAction::Lock,
                        MeterContinuitySource::CurrentMeter,
                        MeterContinuityReason::StableEvidence,
                        0,
                    ),
                    stage(
                        4,
                        MeterContinuityAction::Reacquire,
                        MeterContinuitySource::CurrentMeter,
                        MeterContinuityReason::PhaseDisplacement,
                        1,
                    ),
                    stage(
                        8,
                        MeterContinuityAction::Clear,
                        MeterContinuitySource::Cleared,
                        MeterContinuityReason::InsufficientEvidence,
                        2,
                    ),
                ),
            },
            (MeterStateAction::Lock, _) => MeterContinuityRecommendation {
                bar_length: plan(
                    MeterContinuityAction::Lock,
                    MeterContinuitySource::CurrentMeter,
                    MeterContinuityReason::StableEvidence,
                    16,
                    16,
                    stage(
                        16,
                        MeterContinuityAction::Lock,
                        MeterContinuitySource::CurrentMeter,
                        MeterContinuityReason::StableEvidence,
                        0,
                    ),
                    stage(
                        24,
                        MeterContinuityAction::Retain,
                        MeterContinuitySource::CurrentMeter,
                        MeterContinuityReason::RevalidationDecay,
                        1,
                    ),
                    stage(
                        32,
                        MeterContinuityAction::Clear,
                        MeterContinuitySource::Cleared,
                        MeterContinuityReason::InsufficientEvidence,
                        2,
                    ),
                ),
                downbeat_phase: plan(
                    MeterContinuityAction::Lock,
                    MeterContinuitySource::CurrentMeter,
                    MeterContinuityReason::StableEvidence,
                    16,
                    16,
                    stage(
                        16,
                        MeterContinuityAction::Lock,
                        MeterContinuitySource::CurrentMeter,
                        MeterContinuityReason::StableEvidence,
                        0,
                    ),
                    stage(
                        24,
                        MeterContinuityAction::Retain,
                        MeterContinuitySource::CurrentMeter,
                        MeterContinuityReason::RevalidationDecay,
                        1,
                    ),
                    stage(
                        32,
                        MeterContinuityAction::Clear,
                        MeterContinuitySource::Cleared,
                        MeterContinuityReason::InsufficientEvidence,
                        2,
                    ),
                ),
            },
            (MeterStateAction::Hold, MeterStateReason::TentativeMeter) => {
                MeterContinuityRecommendation {
                    bar_length: plan(
                        MeterContinuityAction::Retain,
                        MeterContinuitySource::CurrentMeter,
                        MeterContinuityReason::TentativeEvidence,
                        retained_beats.min(8),
                        4,
                        stage(
                            4,
                            MeterContinuityAction::Lock,
                            MeterContinuitySource::CurrentMeter,
                            MeterContinuityReason::StableEvidence,
                            0,
                        ),
                        stage(
                            retained_beats.min(8).saturating_add(2),
                            MeterContinuityAction::Reacquire,
                            MeterContinuitySource::CurrentMeter,
                            MeterContinuityReason::RevalidationDecay,
                            1,
                        ),
                        stage(
                            retained_beats.min(8).saturating_add(4),
                            MeterContinuityAction::Clear,
                            MeterContinuitySource::Cleared,
                            MeterContinuityReason::InsufficientEvidence,
                            2,
                        ),
                    ),
                    downbeat_phase: plan(
                        MeterContinuityAction::Reacquire,
                        MeterContinuitySource::CurrentMeter,
                        MeterContinuityReason::TentativeEvidence,
                        0,
                        2,
                        stage(
                            2,
                            MeterContinuityAction::Lock,
                            MeterContinuitySource::CurrentMeter,
                            MeterContinuityReason::StableEvidence,
                            0,
                        ),
                        stage(
                            4,
                            MeterContinuityAction::Reacquire,
                            MeterContinuitySource::CurrentMeter,
                            continuity_reason(
                                MeterContinuityAction::Reacquire,
                                MeterContinuitySource::CurrentMeter,
                                false,
                                true,
                            ),
                            1,
                        ),
                        stage(
                            6,
                            MeterContinuityAction::Clear,
                            MeterContinuitySource::Cleared,
                            MeterContinuityReason::InsufficientEvidence,
                            2,
                        ),
                    ),
                }
            }
            (MeterStateAction::Hold, MeterStateReason::DestabilizedHold) => {
                let trailing_beats = if suppression_profile.trailing_confidence.0 > 0.0 {
                    (((suppression_profile.trailing_confidence.0
                        + suppression_profile.trailing_recent_stability)
                        * 8.0)
                        .round() as usize)
                        .clamp(4, 8)
                } else {
                    4
                };
                MeterContinuityRecommendation {
                    bar_length: plan(
                        MeterContinuityAction::Retain,
                        MeterContinuitySource::PriorMeter,
                        MeterContinuityReason::PriorStateCarry,
                        trailing_beats,
                        4,
                        stage(
                            4,
                            MeterContinuityAction::Retain,
                            MeterContinuitySource::PriorMeter,
                            MeterContinuityReason::PriorStateCarry,
                            0,
                        ),
                        stage(
                            trailing_beats.saturating_add(2),
                            MeterContinuityAction::Reacquire,
                            MeterContinuitySource::PriorMeter,
                            MeterContinuityReason::RevalidationDecay,
                            1,
                        ),
                        stage(
                            trailing_beats.saturating_add(4),
                            MeterContinuityAction::Clear,
                            MeterContinuitySource::Cleared,
                            MeterContinuityReason::InsufficientEvidence,
                            2,
                        ),
                    ),
                    downbeat_phase: plan(
                        MeterContinuityAction::Retain,
                        MeterContinuitySource::PriorMeter,
                        MeterContinuityReason::PriorStateCarry,
                        trailing_beats.saturating_sub(2).max(2),
                        2,
                        stage(
                            2,
                            MeterContinuityAction::Reacquire,
                            MeterContinuitySource::PriorMeter,
                            MeterContinuityReason::RevalidationDecay,
                            1,
                        ),
                        stage(
                            trailing_beats,
                            MeterContinuityAction::Reacquire,
                            MeterContinuitySource::RecoveryWindow,
                            MeterContinuityReason::RevalidationDecay,
                            1,
                        ),
                        stage(
                            trailing_beats.saturating_add(2),
                            MeterContinuityAction::Clear,
                            MeterContinuitySource::Cleared,
                            MeterContinuityReason::InsufficientEvidence,
                            2,
                        ),
                    ),
                }
            }
            (MeterStateAction::Watch, _) => MeterContinuityRecommendation {
                bar_length: plan(
                    MeterContinuityAction::Retain,
                    MeterContinuitySource::RecoveryWindow,
                    MeterContinuityReason::RecoveryWindowSupport,
                    retained_beats,
                    retained_beats.saturating_div(2).max(4),
                    stage(
                        retained_beats.saturating_div(2).max(4),
                        MeterContinuityAction::Lock,
                        MeterContinuitySource::CurrentMeter,
                        MeterContinuityReason::StableEvidence,
                        0,
                    ),
                    stage(
                        retained_beats.saturating_add(4),
                        MeterContinuityAction::Reacquire,
                        MeterContinuitySource::RecoveryWindow,
                        MeterContinuityReason::RevalidationDecay,
                        1,
                    ),
                    stage(
                        retained_beats.saturating_add(8),
                        MeterContinuityAction::Clear,
                        MeterContinuitySource::Cleared,
                        MeterContinuityReason::InsufficientEvidence,
                        2,
                    ),
                ),
                downbeat_phase: plan(
                    MeterContinuityAction::Reacquire,
                    MeterContinuitySource::RecoveryWindow,
                    MeterContinuityReason::RecoveryWindowSupport,
                    0,
                    2,
                    stage(
                        2,
                        MeterContinuityAction::Lock,
                        MeterContinuitySource::CurrentMeter,
                        MeterContinuityReason::StableEvidence,
                        0,
                    ),
                    stage(
                        4,
                        MeterContinuityAction::Reacquire,
                        MeterContinuitySource::RecoveryWindow,
                        MeterContinuityReason::RevalidationDecay,
                        1,
                    ),
                    stage(
                        6,
                        MeterContinuityAction::Clear,
                        MeterContinuitySource::Cleared,
                        MeterContinuityReason::InsufficientEvidence,
                        2,
                    ),
                ),
            },
            (MeterStateAction::Clear, _) => MeterContinuityRecommendation {
                bar_length: plan(
                    MeterContinuityAction::Clear,
                    MeterContinuitySource::Cleared,
                    MeterContinuityReason::InsufficientEvidence,
                    0,
                    0,
                    stage(
                        0,
                        MeterContinuityAction::Clear,
                        MeterContinuitySource::Cleared,
                        MeterContinuityReason::InsufficientEvidence,
                        2,
                    ),
                    stage(
                        0,
                        MeterContinuityAction::Clear,
                        MeterContinuitySource::Cleared,
                        MeterContinuityReason::InsufficientEvidence,
                        2,
                    ),
                    stage(
                        0,
                        MeterContinuityAction::Clear,
                        MeterContinuitySource::Cleared,
                        MeterContinuityReason::InsufficientEvidence,
                        2,
                    ),
                ),
                downbeat_phase: plan(
                    MeterContinuityAction::Clear,
                    MeterContinuitySource::Cleared,
                    MeterContinuityReason::InsufficientEvidence,
                    0,
                    0,
                    stage(
                        0,
                        MeterContinuityAction::Clear,
                        MeterContinuitySource::Cleared,
                        MeterContinuityReason::InsufficientEvidence,
                        2,
                    ),
                    stage(
                        0,
                        MeterContinuityAction::Clear,
                        MeterContinuitySource::Cleared,
                        MeterContinuityReason::InsufficientEvidence,
                        2,
                    ),
                    stage(
                        0,
                        MeterContinuityAction::Clear,
                        MeterContinuitySource::Cleared,
                        MeterContinuityReason::InsufficientEvidence,
                        2,
                    ),
                ),
            },
            (MeterStateAction::Hold, _) => MeterContinuityRecommendation {
                bar_length: plan(
                    MeterContinuityAction::Retain,
                    MeterContinuitySource::PriorMeter,
                    MeterContinuityReason::PriorStateCarry,
                    retained_beats.min(6),
                    4,
                    stage(
                        4,
                        MeterContinuityAction::Retain,
                        MeterContinuitySource::PriorMeter,
                        MeterContinuityReason::PriorStateCarry,
                        0,
                    ),
                    stage(
                        retained_beats.min(6).saturating_add(2),
                        MeterContinuityAction::Reacquire,
                        MeterContinuitySource::PriorMeter,
                        MeterContinuityReason::RevalidationDecay,
                        1,
                    ),
                    stage(
                        retained_beats.min(6).saturating_add(4),
                        MeterContinuityAction::Clear,
                        MeterContinuitySource::Cleared,
                        MeterContinuityReason::InsufficientEvidence,
                        2,
                    ),
                ),
                downbeat_phase: plan(
                    MeterContinuityAction::Retain,
                    MeterContinuitySource::PriorMeter,
                    MeterContinuityReason::PriorStateCarry,
                    retained_beats.min(4),
                    2,
                    stage(
                        2,
                        MeterContinuityAction::Reacquire,
                        MeterContinuitySource::PriorMeter,
                        MeterContinuityReason::RevalidationDecay,
                        1,
                    ),
                    stage(
                        retained_beats.min(4).saturating_add(1),
                        MeterContinuityAction::Reacquire,
                        MeterContinuitySource::RecoveryWindow,
                        MeterContinuityReason::RevalidationDecay,
                        1,
                    ),
                    stage(
                        retained_beats.min(4).saturating_add(2),
                        MeterContinuityAction::Clear,
                        MeterContinuitySource::Cleared,
                        MeterContinuityReason::InsufficientEvidence,
                        2,
                    ),
                ),
            },
        }
    }

    fn build_meter_state(
        action: MeterStateAction,
        reason: MeterStateReason,
        confidence: Confidence,
        estimate: Option<&MeterEstimate>,
        suppression_profile: MeterSuppressionProfile,
        tempo_ambiguity: Confidence,
        bpm: f32,
        beat_positions_seconds: &[f32],
    ) -> MeterStateRecommendation {
        MeterStateRecommendation {
            action,
            reason,
            confidence,
            continuity: continuity_for(
                action,
                reason,
                estimate,
                suppression_profile,
                confidence,
                tempo_ambiguity,
                bpm,
                beat_positions_seconds,
            ),
        }
    }

    if let Some(estimate) = estimate {
        return match estimate.recommendation {
            MeterRecommendation::Lock => build_meter_state(
                MeterStateAction::Lock,
                MeterStateReason::StableMeter,
                estimate.confidence,
                Some(estimate),
                suppression_profile,
                tempo_ambiguity,
                bpm,
                beat_positions_seconds,
            ),
            MeterRecommendation::Monitor if estimate.trust == MeterTrustLevel::Recovering => {
                build_meter_state(
                    MeterStateAction::Watch,
                    MeterStateReason::RecoveringMeter,
                    Confidence::new(
                        0.5 * estimate.support_profile.segment_recovery_strength.0
                            + 0.3 * estimate.support_profile.recovery_duration_strength.0
                            + 0.2 * estimate.confidence.0,
                    ),
                    Some(estimate),
                    suppression_profile,
                    tempo_ambiguity,
                    bpm,
                    beat_positions_seconds,
                )
            }
            MeterRecommendation::Monitor | MeterRecommendation::Defer => build_meter_state(
                MeterStateAction::Hold,
                MeterStateReason::TentativeMeter,
                Confidence::new(
                    0.6 * estimate.confidence.0
                        + 0.4 * estimate.support_profile.whole_track_strength.0,
                ),
                Some(estimate),
                suppression_profile,
                tempo_ambiguity,
                bpm,
                beat_positions_seconds,
            ),
        };
    }

    let pulse_stability =
        (0.65 * rhythm_confidence.0 + 0.35 * (1.0 - tempo_ambiguity.0)).clamp(0.0, 1.0);
    let trailing_recovery_strength = (0.6 * suppression_profile.trailing_confidence.0
        + 0.4 * suppression_profile.trailing_recent_stability)
        .clamp(0.0, 1.0);

    if trailing_recovery_strength >= 0.24 && pulse_stability >= 0.58 {
        if tempo_ambiguity.0 >= 0.43 {
            return build_meter_state(
                MeterStateAction::Clear,
                MeterStateReason::MeterCleared,
                Confidence::new(
                    (0.5 * tempo_ambiguity.0
                        + 0.3 * trailing_recovery_strength
                        + 0.2 * (1.0 - suppression_profile.best_support.clamp(0.0, 1.0)))
                    .clamp(0.0, 1.0),
                ),
                None,
                suppression_profile,
                tempo_ambiguity,
                bpm,
                beat_positions_seconds,
            );
        }

        if tempo_ambiguity.0 <= 0.33 {
            return build_meter_state(
                MeterStateAction::Hold,
                MeterStateReason::DestabilizedHold,
                Confidence::new(
                    (0.55 * pulse_stability
                        + 0.25 * suppression_profile.best_confidence.0
                        + 0.20 * suppression_profile.best_support)
                        .clamp(0.0, 1.0),
                ),
                None,
                suppression_profile,
                tempo_ambiguity,
                bpm,
                beat_positions_seconds,
            );
        }

        build_meter_state(
            MeterStateAction::Watch,
            MeterStateReason::RecoveryEmerging,
            Confidence::new(
                (0.55 * trailing_recovery_strength + 0.45 * pulse_stability).clamp(0.0, 1.0),
            ),
            None,
            suppression_profile,
            tempo_ambiguity,
            bpm,
            beat_positions_seconds,
        )
    } else if pulse_stability >= 0.55
        && suppression_profile.best_confidence.0 >= 0.12
        && suppression_profile.best_support >= 0.48
        && suppression_profile.best_regularity >= 0.20
    {
        build_meter_state(
            MeterStateAction::Hold,
            MeterStateReason::DestabilizedHold,
            Confidence::new(
                (0.5 * pulse_stability
                    + 0.3 * suppression_profile.best_confidence.0
                    + 0.2 * suppression_profile.best_support)
                    .clamp(0.0, 1.0),
            ),
            None,
            suppression_profile,
            tempo_ambiguity,
            bpm,
            beat_positions_seconds,
        )
    } else {
        build_meter_state(
            MeterStateAction::Clear,
            MeterStateReason::MeterCleared,
            Confidence::new(
                (0.45 * (1.0 - suppression_profile.best_confidence.0)
                    + 0.35 * (1.0 - suppression_profile.best_support.clamp(0.0, 1.0))
                    + 0.20 * tempo_ambiguity.0)
                    .clamp(0.0, 1.0),
            ),
            None,
            suppression_profile,
            tempo_ambiguity,
            bpm,
            beat_positions_seconds,
        )
    }
}

fn infer_meter(
    onset_envelope: &[f32],
    meter_cue: &[f32],
    beat_frames: &[usize],
    sample_rate: SampleRate,
    hop_size: usize,
) -> MeterDecision {
    if onset_envelope.is_empty() || beat_frames.len() < 6 {
        return MeterDecision {
            estimate: None,
            suppression_profile: MeterSuppressionProfile {
                best_confidence: Confidence::new(0.0),
                best_support: 0.0,
                best_regularity: 0.0,
                trailing_confidence: Confidence::new(0.0),
                trailing_recent_stability: 0.0,
            },
            ambiguity: RhythmStructureAmbiguitySummary {
                kind: RhythmStructureAmbiguityKind::InsufficientEvidence,
                confidence: Confidence::new(0.0),
                primary: None,
                runner_up: None,
                trailing_recovery_confidence: Confidence::new(0.0),
            },
        };
    }

    let beat_strengths: Vec<f32> = beat_frames
        .iter()
        .map(|frame| neighborhood_peak(onset_envelope, *frame, 3))
        .collect();
    let meter_strengths: Vec<f32> = beat_frames
        .iter()
        .map(|frame| neighborhood_peak(meter_cue, *frame, 3))
        .collect();
    let hypotheses = meter_hypotheses(&beat_strengths, &meter_strengths);

    let Some(best) = hypotheses.first().copied() else {
        return MeterDecision {
            estimate: None,
            suppression_profile: MeterSuppressionProfile {
                best_confidence: Confidence::new(0.0),
                best_support: 0.0,
                best_regularity: 0.0,
                trailing_confidence: Confidence::new(0.0),
                trailing_recent_stability: 0.0,
            },
            ambiguity: RhythmStructureAmbiguitySummary {
                kind: RhythmStructureAmbiguityKind::InsufficientEvidence,
                confidence: Confidence::new(0.0),
                primary: None,
                runner_up: None,
                trailing_recovery_confidence: Confidence::new(0.0),
            },
        };
    };
    let runner_up = hypotheses
        .get(1)
        .map(|candidate| candidate.score)
        .unwrap_or(0.0);
    let confidence = meter_hypothesis_confidence(best, runner_up);
    let global_estimate = if best.score >= 0.12
        && confidence.0 >= 0.18
        && best.regularity >= 0.35
        && best.support_ratio >= 0.60
    {
        let confidence_breakdown = meter_confidence_breakdown(best, runner_up);
        let downbeat_frames = downbeat_frames_for_hypothesis(beat_frames, 0, best);
        Some((
            MeterEstimate {
                beats_per_bar: best.beats_per_bar,
                confidence,
                detection_kind: MeterDetectionKind::WholeTrack,
                trust: MeterTrustLevel::Tentative,
                recommendation: MeterRecommendation::Defer,
                support_profile: MeterSupportProfile {
                    whole_track_strength: confidence,
                    segment_recovery_strength: Confidence::new(0.0),
                    recovery_duration_strength: Confidence::new(0.0),
                },
                confidence_breakdown,
                recovery: None,
                downbeat_positions_seconds: beat_frames_to_seconds(
                    &downbeat_frames,
                    sample_rate,
                    hop_size,
                ),
            },
            best.regularity,
        ))
    } else {
        None
    };

    let segment_candidate = select_segment_meter_candidate(&beat_strengths, &meter_strengths);
    let trailing_candidate = trailing_meter_window_candidate(&beat_strengths, &meter_strengths);
    let ambiguity = rhythm_structure_ambiguity_summary(&hypotheses, trailing_candidate);
    let support_profile = meter_support_profile(
        global_estimate
            .as_ref()
            .map(|(estimate, _)| estimate.confidence),
        segment_candidate,
    );

    let global_estimate = global_estimate.map(|(mut estimate, regularity)| {
        estimate.support_profile = support_profile;
        estimate.trust = meter_trust_level(
            estimate.detection_kind,
            estimate.confidence,
            estimate.support_profile,
            estimate.confidence_breakdown,
        );
        estimate.recommendation = meter_recommendation(
            estimate.trust,
            estimate.detection_kind,
            estimate.confidence,
            estimate.support_profile,
            estimate.confidence_breakdown,
        );
        (estimate, regularity)
    });

    let segment_estimate = if let Some(segment_best) = segment_candidate {
        let downbeat_frames = downbeat_frames_for_hypothesis(
            beat_frames,
            segment_best.start_beat,
            segment_best.hypothesis,
        );
        Some(MeterEstimate {
            beats_per_bar: segment_best.hypothesis.beats_per_bar,
            confidence: segment_best.confidence,
            detection_kind: MeterDetectionKind::SegmentRecovery,
            trust: meter_trust_level(
                MeterDetectionKind::SegmentRecovery,
                segment_best.confidence,
                support_profile,
                segment_best.confidence_breakdown,
            ),
            recommendation: MeterRecommendation::Monitor,
            support_profile,
            confidence_breakdown: segment_best.confidence_breakdown,
            recovery: Some(meter_recovery_context(
                beat_frames,
                sample_rate,
                hop_size,
                segment_best,
            )),
            downbeat_positions_seconds: beat_frames_to_seconds(
                &downbeat_frames,
                sample_rate,
                hop_size,
            ),
        })
        .map(|mut estimate| {
            estimate.recommendation = meter_recommendation(
                estimate.trust,
                estimate.detection_kind,
                estimate.confidence,
                estimate.support_profile,
                estimate.confidence_breakdown,
            );
            estimate
        })
    } else {
        None
    };

    let estimate = match (global_estimate, segment_estimate) {
        (Some((global, global_regularity)), Some(segment))
            if segment.confidence.0 >= global.confidence.0 + 0.04 && global_regularity < 0.58 =>
        {
            Some(segment)
        }
        (Some((global, _)), _) => Some(global),
        (None, Some(segment)) => Some(segment),
        (None, None) => None,
    };

    MeterDecision {
        estimate,
        suppression_profile: MeterSuppressionProfile {
            best_confidence: confidence,
            best_support: best.support_ratio,
            best_regularity: best.regularity,
            trailing_confidence: trailing_candidate
                .map(|candidate| candidate.confidence)
                .unwrap_or(Confidence::new(0.0)),
            trailing_recent_stability: trailing_candidate
                .map(|candidate| candidate.hypothesis.recent_strength)
                .unwrap_or(0.0),
        },
        ambiguity,
    }
}

fn select_beat_phase(onset_envelope: &[f32], lag_frames: usize) -> usize {
    if onset_envelope.is_empty() || lag_frames == 0 {
        return 0;
    }

    let search_len = lag_frames.min(onset_envelope.len());
    let mut best_phase = 0usize;
    let mut best_score = 0.0f32;

    for phase in 0..search_len {
        let score = beat_phase_score(onset_envelope, lag_frames, phase);
        if score > best_score {
            best_score = score;
            best_phase = phase;
        }
    }

    best_phase
}

fn beat_phase_score(onset_envelope: &[f32], lag_frames: usize, phase_offset_frames: usize) -> f32 {
    if onset_envelope.is_empty() || lag_frames == 0 {
        return 0.0;
    }

    let radius = ((lag_frames as f32) * 0.15).round().max(1.0) as usize;
    let half_lag = lag_frames / 2;
    let mut beat_sum = 0.0;
    let mut beat_count = 0usize;
    let mut supported_beats = 0usize;
    let mut offbeat_sum = 0.0;
    let mut offbeat_count = 0usize;

    let mut index = phase_offset_frames.min(onset_envelope.len().saturating_sub(1));
    while index < onset_envelope.len() {
        let beat_peak = neighborhood_peak(onset_envelope, index, radius);
        beat_sum += beat_peak;
        beat_count += 1;
        if beat_peak > 0.35 {
            supported_beats += 1;
        }

        if half_lag > 1 {
            let midpoint = index + half_lag;
            if midpoint < onset_envelope.len() {
                offbeat_sum += neighborhood_peak(onset_envelope, midpoint, radius);
                offbeat_count += 1;
            }
        }

        index += lag_frames;
    }

    if beat_count == 0 {
        return 0.0;
    }

    let beat_average = beat_sum / beat_count as f32;
    let support_ratio = supported_beats as f32 / beat_count as f32;
    let offbeat_average = if offbeat_count > 0 {
        offbeat_sum / offbeat_count as f32
    } else {
        0.0
    };

    (0.55 * beat_average + 0.45 * support_ratio - 0.35 * offbeat_average)
        .max(0.0)
        .clamp(0.0, 1.0)
}

fn neighborhood_peak(onset_envelope: &[f32], center: usize, radius: usize) -> f32 {
    if onset_envelope.is_empty() {
        return 0.0;
    }

    let start = center.saturating_sub(radius);
    let end = (center + radius + 1).min(onset_envelope.len());
    onset_envelope[start..end]
        .iter()
        .copied()
        .fold(0.0f32, |best, value| best.max(value))
}

fn refine_beat(onset_envelope: &[f32], center: isize, tolerance_frames: isize) -> isize {
    let start = (center - tolerance_frames).max(0) as usize;
    let end =
        (center + tolerance_frames).min(onset_envelope.len().saturating_sub(1) as isize) as usize;

    let mut best_index = center.clamp(0, onset_envelope.len().saturating_sub(1) as isize) as usize;
    let mut best_value = onset_envelope[best_index];

    for index in start..=end {
        let value = onset_envelope[index];
        if value > best_value {
            best_value = value;
            best_index = index;
        }
    }

    best_index as isize
}

pub(crate) fn normalize(values: &mut [f32]) {
    let max_value = values
        .iter()
        .copied()
        .fold(0.0f32, |best, value| best.max(value));

    if max_value > 0.0 {
        for value in values {
            *value /= max_value;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{BeatTracker, BeatTrackerConfig};
    use signal_analysis::{
        run_audio_acceptance_harness, AcceptanceSeverity, AcceptanceStatus, AnalysisCorpusCase,
        AnalysisCorpusCaseMetadata, AnalysisCorpusFamily, AnalysisMetricValue, AnalysisStage,
    };
    use signal_primitives::{AudioBuffer, ChannelLayout, SampleRate, Seconds};

    #[path = "rhythm_test_bar_transition_basic.rs"]
    mod bar_transition_basic;
    #[path = "rhythm_test_bar_transition_metrics.rs"]
    mod bar_transition_metrics;
    #[path = "rhythm_test_bar_transition_recovery_metrics.rs"]
    mod bar_transition_recovery_metrics;
    #[path = "rhythm_test_bar_transition_reentry.rs"]
    mod bar_transition_reentry;
    #[path = "rhythm_test_bar_transition_reentry_extended.rs"]
    mod bar_transition_reentry_extended;
    #[path = "rhythm_test_named_preset_metrics.rs"]
    mod named_preset_metrics;
    #[path = "rhythm_test_named_preset_monotonicity.rs"]
    mod named_preset_monotonicity;
    #[path = "rhythm_test_named_preset_surface.rs"]
    mod named_preset_surface;
    #[path = "rhythm_test_named_preset_surface_cases.rs"]
    mod named_preset_surface_cases;
    #[path = "rhythm_test_presets.rs"]
    mod presets;
    #[path = "rhythm_test_transition_fixtures.rs"]
    mod transition_fixtures;
    use presets::{
        build_structured_harmony_preset, render_preset, BarTransitionVariant,
        HarmonicRhythmVariant, RhythmPreset,
    };
    use transition_fixtures::{DropoutVariant, FillDensityVariant};

    const CLICK_LENGTH: usize = 64;
    const TONE_BURST_LENGTH: usize = 2_048;
    const KICK_TONES: &[f32] = &[60.0, 95.0];
    const SNARE_TONES: &[f32] = &[220.0, 330.0, 1800.0];
    const HAT_TONES: &[f32] = &[4000.0, 6200.0, 8400.0];
    const CHORD_A: &[f32] = &[220.0, 277.18, 329.63];
    const CHORD_B: &[f32] = &[261.63, 329.63, 392.0];
    const CHORD_C: &[f32] = &[196.0, 246.94, 293.66];
    const CHORD_D: &[f32] = &[246.94, 311.13, 369.99];
    const CHORD_CYCLE_A: &[&[f32]] = &[CHORD_A];
    const CHORD_CYCLE_AB: &[&[f32]] = &[CHORD_A, CHORD_B];
    const CHORD_CYCLE_ABCD: &[&[f32]] = &[CHORD_A, CHORD_B, CHORD_C, CHORD_D];
    const CHORD_CYCLE_CD: &[&[f32]] = &[CHORD_C, CHORD_D];
    const FILL_BAR_PATTERNS: &[[f32; 4]] = &[
        [0.46, 0.24, 0.36, 0.24],
        [0.44, 0.22, 0.34, 0.22],
        [0.48, 0.24, 0.38, 0.24],
        [0.36, 0.32, 0.44, 0.62],
        [0.48, 0.24, 0.36, 0.26],
        [0.46, 0.24, 0.36, 0.24],
        [0.5, 0.26, 0.38, 0.24],
        [0.46, 0.24, 0.34, 0.24],
    ];
    const FILL_BAR_CHORDS: &[&[f32]] = &[
        CHORD_A, CHORD_A, CHORD_B, CHORD_B, CHORD_C, CHORD_C, CHORD_D, CHORD_D,
    ];
    const DENSE_FILL_BAR_PATTERNS: &[[f32; 4]] = &[
        [0.48, 0.26, 0.38, 0.26],
        [0.42, 0.34, 0.38, 0.28],
        [0.5, 0.28, 0.4, 0.28],
        [0.34, 0.4, 0.48, 0.7],
        [0.5, 0.3, 0.4, 0.3],
        [0.38, 0.34, 0.42, 0.56],
        [0.52, 0.3, 0.4, 0.32],
        [0.36, 0.36, 0.42, 0.66],
    ];
    const DENSE_FILL_BAR_CHORDS: &[&[f32]] = &[
        CHORD_A, CHORD_B, CHORD_B, CHORD_C, CHORD_C, CHORD_D, CHORD_D, CHORD_A,
    ];
    const REENTRY_HARMONIC_SHIFT_BAR_CHORDS: &[&[f32]] = &[CHORD_A, CHORD_C, CHORD_B, CHORD_D];
    const REENTRY_ACCELERATING_STAGE_BAR_CHORDS: &[&[f32]] = &[CHORD_A, CHORD_C];
    const REENTRY_DECELERATING_STAGE_BAR_CHORDS: &[&[f32]] = &[CHORD_A, CHORD_D];
    const REENTRY_ACCELERATING_DENSE_BAR_PATTERNS: &[[f32; 4]] =
        &[[0.54, 0.28, 0.4, 0.3], [0.5, 0.32, 0.42, 0.32]];
    const REENTRY_DECELERATING_DENSE_BAR_PATTERNS: &[[f32; 4]] =
        &[[0.58, 0.3, 0.44, 0.32], [0.54, 0.34, 0.46, 0.36]];
    const REENTRY_ACCELERATING_ACCENT_SHIFT_BAR_PATTERNS: &[[f32; 4]] =
        &[[0.28, 0.66, 0.3, 0.58], [0.26, 0.62, 0.3, 0.6]];
    const REENTRY_DECELERATING_ACCENT_SHIFT_BAR_PATTERNS: &[[f32; 4]] =
        &[[0.3, 0.6, 0.28, 0.62], [0.28, 0.64, 0.3, 0.58]];
    const REENTRY_HARMONIC_RESET_BAR_PATTERNS: &[[f32; 4]] =
        &[[0.56, 0.24, 0.4, 0.24], [0.58, 0.24, 0.42, 0.24]];
    const REENTRY_SUSTAINED_RESET_BAR_PATTERNS: &[[f32; 4]] = &[
        [0.6, 0.24, 0.42, 0.24],
        [0.62, 0.24, 0.44, 0.24],
        [0.64, 0.22, 0.44, 0.22],
        [0.62, 0.24, 0.42, 0.24],
        [0.66, 0.22, 0.46, 0.22],
        [0.64, 0.24, 0.44, 0.24],
    ];
    const REENTRY_CADENTIAL_REANCHOR_BAR_PATTERNS: &[[f32; 4]] =
        &[[0.72, 0.22, 0.44, 0.24], [0.64, 0.24, 0.42, 0.24]];
    const REENTRY_CADENTIAL_REANCHOR_BAR_CHORDS: &[&[f32]] = &[CHORD_D, CHORD_A];
    const LATE_SHIFT_BAR_PATTERNS: &[[f32; 4]] = &[
        [0.5, 0.26, 0.38, 0.24],
        [0.48, 0.24, 0.36, 0.24],
        [0.28, 0.72, 0.34, 0.22],
        [0.52, 0.28, 0.38, 0.24],
        [0.48, 0.24, 0.36, 0.24],
        [0.5, 0.26, 0.38, 0.24],
    ];
    const LATE_SHIFT_BAR_CHORDS: &[&[f32]] =
        &[CHORD_A, CHORD_B, CHORD_C, CHORD_C, CHORD_D, CHORD_A];
    const LIGHT_DROPOUT_BAR_PATTERNS: &[[f32; 4]] = &[
        [0.48, 0.24, 0.36, 0.24],
        [0.48, 0.24, 0.36, 0.24],
        [0.3, 0.12, 0.24, 0.12],
        [0.5, 0.24, 0.38, 0.24],
        [0.46, 0.22, 0.34, 0.22],
        [0.48, 0.24, 0.36, 0.24],
    ];
    const MEDIUM_DROPOUT_BAR_PATTERNS: &[[f32; 4]] = &[
        [0.48, 0.24, 0.36, 0.24],
        [0.24, 0.1, 0.18, 0.08],
        [0.5, 0.24, 0.38, 0.24],
        [0.22, 0.08, 0.16, 0.08],
        [0.46, 0.22, 0.34, 0.22],
        [0.48, 0.24, 0.36, 0.24],
    ];
    const DROPOUT_BAR_PATTERNS: &[[f32; 4]] = &[
        [0.48, 0.24, 0.36, 0.24],
        [0.04, 0.0, 0.03, 0.0],
        [0.5, 0.24, 0.38, 0.24],
        [0.03, 0.0, 0.02, 0.0],
        [0.46, 0.22, 0.34, 0.22],
        [0.02, 0.0, 0.02, 0.0],
    ];

    #[derive(Clone, Copy)]
    struct GrooveSection {
        bars: usize,
        beat_pattern: [f32; 4],
        chord_cycle: &'static [&'static [f32]],
        chord_every_bars: usize,
        section_marker: Option<(usize, &'static [f32], f32)>,
        bar_patterns: Option<&'static [[f32; 4]]>,
        bar_chords: Option<&'static [&'static [f32]]>,
        dropout_bars: &'static [usize],
    }

    #[derive(Default)]
    struct FixtureBuilder {
        beats: Vec<f32>,
        tone_events: Vec<(usize, &'static [f32], f32)>,
    }

    impl FixtureBuilder {
        fn new() -> Self {
            Self::default()
        }

        fn beat_len(&self) -> usize {
            self.beats.len()
        }

        fn push_four_four_section(&mut self, section: GrooveSection) {
            let start_beat = self.beat_len();
            push_four_four_groove(&mut self.beats, &mut self.tone_events, start_beat, section);
        }

        fn build(self, sample_rate: u32, bpm: f32) -> AudioBuffer {
            beat_sequence_track(sample_rate, bpm, &self.beats, &self.tone_events)
        }
    }

    fn analyze_fixture(audio: &AudioBuffer) -> super::BeatAnalysisResult {
        let mut tracker = BeatTracker::new(BeatTrackerConfig::default());
        tracker.analyze(audio)
    }

    fn analyze_preset(preset: RhythmPreset) -> (f32, super::BeatAnalysisResult) {
        let sample_rate = 48_000;
        let (bpm, audio) = render_preset(preset, sample_rate);
        (bpm, analyze_fixture(&audio))
    }

    fn rhythm_metrics(result: &super::BeatAnalysisResult) -> Vec<AnalysisMetricValue> {
        let assessment = result.rhythm_structure_assessment();
        let meter = result.meter.as_ref();
        let structure = assessment.structure.as_ref();

        vec![
            AnalysisMetricValue::new("bpm", result.bpm),
            AnalysisMetricValue::new("confidence", result.confidence.0),
            AnalysisMetricValue::new("tempo_ambiguity", result.tempo_ambiguity.0),
            AnalysisMetricValue::new("has_meter", if meter.is_some() { 1.0 } else { 0.0 }),
            AnalysisMetricValue::new(
                "beats_per_bar",
                meter
                    .map(|estimate| estimate.beats_per_bar as f32)
                    .unwrap_or(0.0),
            ),
            AnalysisMetricValue::new(
                "meter_confidence",
                meter.map(|estimate| estimate.confidence.0).unwrap_or(0.0),
            ),
            AnalysisMetricValue::new(
                "structure_bar_count",
                structure
                    .map(|summary| summary.bar_count as f32)
                    .unwrap_or(0.0),
            ),
            AnalysisMetricValue::new(
                "recovered_bar_count",
                structure
                    .map(|summary| summary.recovered_bar_count as f32)
                    .unwrap_or(0.0),
            ),
            AnalysisMetricValue::new(
                "recovery_window_available",
                if assessment.fallback.recovery_window_available {
                    1.0
                } else {
                    0.0
                },
            ),
        ]
    }

    fn rhythm_acceptance_cases() -> Vec<AnalysisCorpusCase> {
        let sample_rate = 48_000;
        vec![
            AnalysisCorpusCase::new(
                AnalysisCorpusCaseMetadata::synthetic(
                    "rhythm:steady-click120",
                    AnalysisCorpusFamily::Pulse,
                    "Stable click-track tempo reference",
                ),
                click_track(sample_rate, 120.0, 8.0),
            )
            .with_acceptance_thresholds(vec![
                signal_analysis::AcceptanceThreshold::range(
                    "bpm",
                    Some(119.9),
                    Some(120.1),
                    AcceptanceSeverity::Fail,
                ),
                signal_analysis::AcceptanceThreshold::range(
                    "confidence",
                    Some(0.2),
                    Some(1.0),
                    AcceptanceSeverity::Fail,
                ),
                signal_analysis::AcceptanceThreshold::range(
                    "tempo_ambiguity",
                    Some(0.0),
                    Some(0.4),
                    AcceptanceSeverity::Fail,
                ),
                signal_analysis::AcceptanceThreshold::range(
                    "has_meter",
                    Some(0.0),
                    Some(0.0),
                    AcceptanceSeverity::Fail,
                ),
            ]),
            AnalysisCorpusCase::new(
                AnalysisCorpusCaseMetadata::synthetic(
                    "rhythm:structured-harmony120",
                    AnalysisCorpusFamily::Pulse,
                    "Structured meter reference with stable whole-track bar grid",
                ),
                build_structured_harmony_preset(sample_rate, 120.0, HarmonicRhythmVariant::Active),
            )
            .with_acceptance_thresholds(vec![
                signal_analysis::AcceptanceThreshold::range(
                    "bpm",
                    Some(118.0),
                    Some(122.0),
                    AcceptanceSeverity::Fail,
                ),
                signal_analysis::AcceptanceThreshold::range(
                    "has_meter",
                    Some(1.0),
                    Some(1.0),
                    AcceptanceSeverity::Fail,
                ),
                signal_analysis::AcceptanceThreshold::range(
                    "beats_per_bar",
                    Some(4.0),
                    Some(4.0),
                    AcceptanceSeverity::Fail,
                ),
                signal_analysis::AcceptanceThreshold::range(
                    "meter_confidence",
                    Some(0.2),
                    Some(1.0),
                    AcceptanceSeverity::Fail,
                ),
                signal_analysis::AcceptanceThreshold::range(
                    "structure_bar_count",
                    Some(4.0),
                    None,
                    AcceptanceSeverity::Fail,
                ),
                signal_analysis::AcceptanceThreshold::range(
                    "recovered_bar_count",
                    Some(0.0),
                    Some(0.0),
                    AcceptanceSeverity::Fail,
                ),
            ]),
            AnalysisCorpusCase::new(
                AnalysisCorpusCaseMetadata::synthetic(
                    "rhythm:ambiguous-subdivision90",
                    AnalysisCorpusFamily::Pulse,
                    "Subdivision-heavy ambiguity reference",
                ),
                grid_click_track(sample_rate, 90.0, 2, 8.0, &[1.0, 0.3], None),
            )
            .with_acceptance_thresholds(vec![
                signal_analysis::AcceptanceThreshold::range(
                    "bpm",
                    Some(88.0),
                    Some(92.0),
                    AcceptanceSeverity::Fail,
                ),
                signal_analysis::AcceptanceThreshold::range(
                    "confidence",
                    Some(0.1),
                    Some(1.0),
                    AcceptanceSeverity::Fail,
                ),
                signal_analysis::AcceptanceThreshold::range(
                    "tempo_ambiguity",
                    Some(0.2),
                    Some(1.0),
                    AcceptanceSeverity::Fail,
                ),
                signal_analysis::AcceptanceThreshold::range(
                    "has_meter",
                    Some(0.0),
                    Some(0.0),
                    AcceptanceSeverity::Fail,
                ),
            ]),
        ]
    }

    fn trailing_window_audio(audio: &AudioBuffer, seconds: f32) -> AudioBuffer {
        let sample_rate = audio.sample_rate();
        let channel_count = audio.channel_count().0.max(1);
        let requested_frames = sample_rate.seconds_to_frames(Seconds(seconds)).0.max(1);
        let frames = requested_frames.min(audio.frames().0);
        let start_frame = audio.frames().0.saturating_sub(frames);
        let start_sample = start_frame.saturating_mul(channel_count);
        AudioBuffer::from_interleaved(
            sample_rate,
            audio.channels(),
            audio.samples()[start_sample..].to_vec(),
        )
    }

    fn analyze_trailing_window(
        audio: &AudioBuffer,
        config: super::BeatTrackerConfig,
        seconds: f32,
    ) -> super::BeatAnalysisResult {
        let window = trailing_window_audio(audio, seconds);
        let mut tracker = super::BeatTracker::new(config);
        tracker.analyze(&window)
    }

    fn synthetic_tempo_diagnostics(
        core_window_bpm: f32,
        boundary_bias_bpm: f32,
        trend_total_drift_bpm: f32,
        trend_fit_mad_bpm: f32,
        mean_abs_residual_ms: f32,
        core_abs_residual_ms: f32,
        anchored_drift_ms: f32,
        edge_abs_residual_ms: f32,
    ) -> super::TempoDiagnostics {
        super::TempoDiagnostics {
            interval_tempi: Vec::new(),
            windowed_tempi: Vec::new(),
            median_bpm: core_window_bpm,
            drift_span_bpm: boundary_bias_bpm,
            mean_abs_deviation_bpm: trend_fit_mad_bpm,
            windowed_median_bpm: core_window_bpm,
            windowed_drift_span_bpm: boundary_bias_bpm,
            windowed_mean_abs_deviation_bpm: trend_fit_mad_bpm,
            core_windowed_median_bpm: core_window_bpm,
            core_windowed_drift_span_bpm: trend_total_drift_bpm.abs(),
            core_windowed_mean_abs_deviation_bpm: trend_fit_mad_bpm,
            boundary_bias_bpm,
            trend: super::TempoTrendDiagnostics {
                direction: if trend_total_drift_bpm.abs() < 0.15 {
                    super::TempoTrendDirection::Stable
                } else if trend_total_drift_bpm > 0.0 {
                    super::TempoTrendDirection::Accelerating
                } else {
                    super::TempoTrendDirection::Decelerating
                },
                start_bpm: core_window_bpm - 0.5 * trend_total_drift_bpm,
                end_bpm: core_window_bpm + 0.5 * trend_total_drift_bpm,
                total_drift_bpm: trend_total_drift_bpm,
                slope_bpm_per_beat: trend_total_drift_bpm / 8.0,
                fit_mean_abs_deviation_bpm: trend_fit_mad_bpm,
            },
            beat_grid_error: super::BeatGridErrorDiagnostics {
                residuals: Vec::new(),
                mean_abs_residual_ms,
                max_abs_residual_ms: edge_abs_residual_ms.max(core_abs_residual_ms),
                edge_mean_abs_residual_ms: edge_abs_residual_ms,
                core_mean_abs_residual_ms: core_abs_residual_ms,
                end_anchored_drift_ms: anchored_drift_ms,
                mean_abs_anchored_drift_ms: anchored_drift_ms.abs(),
            },
            beat_interval_outliers: super::BeatIntervalOutlierDiagnostics {
                total_intervals: 0,
                retained_intervals: 0,
                rejected_intervals: 0,
                leading_rejected_intervals: 0,
                trailing_rejected_intervals: 0,
                median_interval: 0.0,
                median_abs_deviation: 0.0,
                max_rejected_deviation_ratio: 0.0,
            },
            stability_scope: super::TempoStabilityScopeSummary {
                scope: super::TempoStabilityScope::MidTrackUnstable,
                support: super::TempoStabilityScopeSupport {
                    edge_trimmed_coverage: super::Confidence::new(0.0),
                    contiguous_core_coverage: super::Confidence::new(0.0),
                    interior_stability: super::Confidence::new(0.0),
                    edge_locality: super::Confidence::new(0.0),
                },
            },
            edge_trimmed_stable_span: None,
            stable_core_span: None,
        }
    }

    fn synthetic_tempo_diagnostics_with_counts(
        core_window_bpm: f32,
        boundary_bias_bpm: f32,
        trend_total_drift_bpm: f32,
        trend_fit_mad_bpm: f32,
        mean_abs_residual_ms: f32,
        core_abs_residual_ms: f32,
        anchored_drift_ms: f32,
        edge_abs_residual_ms: f32,
        interval_count: usize,
        windowed_count: usize,
        residual_count: usize,
    ) -> super::TempoDiagnostics {
        let mut diagnostics = synthetic_tempo_diagnostics(
            core_window_bpm,
            boundary_bias_bpm,
            trend_total_drift_bpm,
            trend_fit_mad_bpm,
            mean_abs_residual_ms,
            core_abs_residual_ms,
            anchored_drift_ms,
            edge_abs_residual_ms,
        );
        diagnostics.interval_tempi = (0..interval_count)
            .map(|index| super::LocalTempoPoint {
                start_beat_index: index,
                end_beat_index: index + 1,
                start_seconds: index as f32,
                end_seconds: index as f32 + 60.0 / core_window_bpm.max(1.0),
                bpm: core_window_bpm,
            })
            .collect();
        diagnostics.windowed_tempi = (0..windowed_count)
            .map(|index| super::LocalTempoPoint {
                start_beat_index: index,
                end_beat_index: index + 4,
                start_seconds: index as f32,
                end_seconds: index as f32 + 4.0 * (60.0 / core_window_bpm.max(1.0)),
                bpm: core_window_bpm,
            })
            .collect();
        diagnostics.beat_grid_error.residuals = (0..residual_count)
            .map(|beat_index| super::BeatGridResidualPoint {
                beat_index,
                seconds: beat_index as f32 * (60.0 / core_window_bpm.max(1.0)),
                fitted_residual_ms: 0.0,
                anchored_drift_ms: 0.0,
            })
            .collect();
        diagnostics.beat_interval_outliers = super::BeatIntervalOutlierDiagnostics {
            total_intervals: interval_count,
            retained_intervals: interval_count,
            rejected_intervals: 0,
            leading_rejected_intervals: 0,
            trailing_rejected_intervals: 0,
            median_interval: 60.0 / core_window_bpm.max(1.0),
            median_abs_deviation: 0.0,
            max_rejected_deviation_ratio: 0.0,
        };
        let stable_span = if windowed_count == 0 {
            None
        } else {
            Some(super::BeatGridCoreSpanDiagnostics {
                start_beat_index: 0,
                end_beat_index: (windowed_count + 3).min(interval_count),
                start_seconds: 0.0,
                end_seconds: (windowed_count + 3) as f32 * (60.0 / core_window_bpm.max(1.0)),
                coverage: super::Confidence::new(1.0),
                retained_windows: windowed_count,
                total_windows: windowed_count,
                trimmed_leading_windows: 0,
                trimmed_trailing_windows: 0,
                interior_rejected_windows: 0,
            })
        };
        diagnostics.stability_scope = super::classify_tempo_stability_scope(
            windowed_count,
            &diagnostics.beat_interval_outliers,
            stable_span,
            stable_span,
        );
        diagnostics.edge_trimmed_stable_span = stable_span;
        diagnostics.stable_core_span = stable_span;
        diagnostics
    }

    fn synthetic_tempo_interpretation(
        recommendation: super::TempoRecommendation,
        trust: super::TempoTrustLevel,
        reason: super::TempoInterpretationReason,
        recommended_bpm: f32,
        snapped_bpm: Option<f32>,
        stability_score: f32,
        snap_error_bpm: f32,
        boundary_pressure: f32,
        grid_stability: f32,
    ) -> super::TempoInterpretation {
        super::TempoInterpretation {
            trust,
            recommendation,
            reason,
            recommended_bpm,
            snapped_bpm,
            support: super::TempoInterpretationSupport {
                core_consensus: super::Confidence::new(0.9),
                drift_stability: super::Confidence::new(0.8),
                grid_stability: super::Confidence::new(grid_stability),
                integer_closeness: super::Confidence::new(
                    (1.0 - snap_error_bpm / 0.12).clamp(0.0, 1.0),
                ),
                boundary_pressure: super::Confidence::new(boundary_pressure),
            },
            profile: super::TempoInterpretationProfile {
                refined_bpm: recommended_bpm,
                core_window_bpm: recommended_bpm,
                nearest_integer_bpm: recommended_bpm.round(),
                snap_error_bpm,
                stability_score: super::Confidence::new(stability_score),
                boundary_edge_gap_ms: 4.0 * boundary_pressure,
            },
        }
    }

    fn synthetic_tempo_structure_result(
        diagnostics: super::TempoDiagnostics,
        interpretation: super::TempoInterpretation,
        confidence: super::Confidence,
        tempo_ambiguity: super::Confidence,
    ) -> super::BeatAnalysisResult {
        let mut result = analyze_fixture(&click_track(
            48_000,
            interpretation.recommended_bpm.max(60.0),
            8.0,
        ));
        let stability_scope = diagnostics.stability_scope;
        result.bpm = interpretation.recommended_bpm;
        result.confidence = confidence;
        result.tempo_diagnostics = diagnostics;
        result.tempo_interpretation = interpretation;
        result.tempo_state = super::tempo_state_recommendation_with_scope(
            interpretation,
            confidence,
            tempo_ambiguity,
            stability_scope,
        );
        result.tempo_ambiguity = tempo_ambiguity;
        result
    }

    fn scope_summary(scope: super::TempoStabilityScope) -> super::TempoStabilityScopeSummary {
        match scope {
            super::TempoStabilityScope::WholeTrackStable => super::TempoStabilityScopeSummary {
                scope,
                support: super::TempoStabilityScopeSupport {
                    edge_trimmed_coverage: super::Confidence::new(1.0),
                    contiguous_core_coverage: super::Confidence::new(0.98),
                    interior_stability: super::Confidence::new(1.0),
                    edge_locality: super::Confidence::new(0.05),
                },
            },
            super::TempoStabilityScope::StableWithLocalizedEdgeDamage => {
                super::TempoStabilityScopeSummary {
                    scope,
                    support: super::TempoStabilityScopeSupport {
                        edge_trimmed_coverage: super::Confidence::new(0.99),
                        contiguous_core_coverage: super::Confidence::new(0.66),
                        interior_stability: super::Confidence::new(0.98),
                        edge_locality: super::Confidence::new(0.95),
                    },
                }
            }
            super::TempoStabilityScope::CoreStableOnly => super::TempoStabilityScopeSummary {
                scope,
                support: super::TempoStabilityScopeSupport {
                    edge_trimmed_coverage: super::Confidence::new(0.61),
                    contiguous_core_coverage: super::Confidence::new(0.54),
                    interior_stability: super::Confidence::new(0.88),
                    edge_locality: super::Confidence::new(0.32),
                },
            },
            super::TempoStabilityScope::MidTrackUnstable => super::TempoStabilityScopeSummary {
                scope,
                support: super::TempoStabilityScopeSupport {
                    edge_trimmed_coverage: super::Confidence::new(0.28),
                    contiguous_core_coverage: super::Confidence::new(0.24),
                    interior_stability: super::Confidence::new(0.42),
                    edge_locality: super::Confidence::new(0.18),
                },
            },
        }
    }

    fn assert_detected_bpm(
        preset: RhythmPreset,
        result: &super::BeatAnalysisResult,
        expected_bpm: f32,
        tolerance: f32,
    ) {
        assert!(
            (result.bpm - expected_bpm).abs() < tolerance,
            "preset {:?} detected bpm {} expected {} +/- {}",
            preset,
            result.bpm,
            expected_bpm,
            tolerance
        );
    }

    fn assert_meter(
        preset: RhythmPreset,
        result: &super::BeatAnalysisResult,
        beats_per_bar: usize,
        min_confidence: f32,
    ) -> &super::MeterEstimate {
        let meter = result
            .meter
            .as_ref()
            .unwrap_or_else(|| panic!("preset {:?} expected meter estimate", preset));
        assert_eq!(
            meter.beats_per_bar, beats_per_bar,
            "preset {:?} beats_per_bar {}",
            preset, meter.beats_per_bar
        );
        assert!(
            meter.confidence.0 > min_confidence,
            "preset {:?} meter confidence {}",
            preset,
            meter.confidence.0
        );
        meter
    }

    fn add_click(samples: &mut [f32], index: usize, amplitude: f32) {
        for offset in 0..CLICK_LENGTH {
            if let Some(sample) = samples.get_mut(index + offset) {
                *sample += amplitude * (1.0 - offset as f32 / CLICK_LENGTH as f32);
            }
        }
    }

    fn add_tone_burst(
        samples: &mut [f32],
        sample_rate: u32,
        index: usize,
        frequencies: &[f32],
        amplitude: f32,
    ) {
        for offset in 0..TONE_BURST_LENGTH {
            let Some(sample) = samples.get_mut(index + offset) else {
                break;
            };
            let t = offset as f32 / sample_rate as f32;
            let envelope = (1.0 - offset as f32 / TONE_BURST_LENGTH as f32).max(0.0);
            let tone = frequencies
                .iter()
                .copied()
                .map(|frequency| (core::f32::consts::TAU * frequency * t).sin())
                .sum::<f32>();
            *sample += amplitude * envelope * tone / frequencies.len().max(1) as f32;
        }
    }

    fn click_track(sample_rate: u32, bpm: f32, seconds: f32) -> AudioBuffer {
        let frames = (sample_rate as f32 * seconds).round() as usize;
        let mut samples = vec![0.0; frames];
        let interval = (60.0 / bpm * sample_rate as f32).round() as usize;

        let mut index = 0usize;
        while index < frames {
            add_click(&mut samples, index, 1.0);
            index = index.saturating_add(interval.max(1));
        }

        AudioBuffer::from_interleaved(SampleRate(sample_rate), ChannelLayout::Mono, samples)
    }

    fn grid_click_track(
        sample_rate: u32,
        bpm: f32,
        steps_per_beat: usize,
        seconds: f32,
        pattern: &[f32],
        swing_ratio: Option<f32>,
    ) -> AudioBuffer {
        let frames = (sample_rate as f32 * seconds).round() as usize;
        let beat_frames = 60.0 / bpm * sample_rate as f32;
        let subdivision_frames = beat_frames / steps_per_beat.max(1) as f32;
        let mut samples = vec![0.0; frames];
        let total_steps = ((seconds * bpm / 60.0) * steps_per_beat as f32).ceil() as usize;

        for step in 0..total_steps {
            let amplitude = pattern[step % pattern.len()];
            if amplitude <= 0.0 {
                continue;
            }

            let beat_index = step / steps_per_beat.max(1);
            let step_in_beat = step % steps_per_beat.max(1);
            let offset_frames = if steps_per_beat == 2 {
                match (step_in_beat, swing_ratio) {
                    (0, _) => 0.0,
                    (1, Some(ratio)) => beat_frames * ratio.clamp(0.5, 0.85),
                    _ => subdivision_frames,
                }
            } else {
                step_in_beat as f32 * subdivision_frames
            };
            let index = (beat_index as f32 * beat_frames + offset_frames).round() as usize;
            add_click(&mut samples, index, amplitude);
        }

        AudioBuffer::from_interleaved(SampleRate(sample_rate), ChannelLayout::Mono, samples)
    }

    fn beat_sequence_track(
        sample_rate: u32,
        bpm: f32,
        beat_amplitudes: &[f32],
        tone_events: &[(usize, &'static [f32], f32)],
    ) -> AudioBuffer {
        let beat_frames = (60.0 / bpm * sample_rate as f32).round() as usize;
        let frames = beat_frames
            .saturating_mul(beat_amplitudes.len())
            .saturating_add(TONE_BURST_LENGTH);
        let mut samples = vec![0.0; frames];

        for (beat_index, amplitude) in beat_amplitudes.iter().copied().enumerate() {
            if amplitude > 0.0 {
                add_click(
                    &mut samples,
                    beat_index.saturating_mul(beat_frames),
                    amplitude,
                );
            }
        }

        for (beat_index, frequencies, amplitude) in tone_events {
            add_tone_burst(
                &mut samples,
                sample_rate,
                beat_index.saturating_mul(beat_frames),
                frequencies,
                *amplitude,
            );
        }

        AudioBuffer::from_interleaved(SampleRate(sample_rate), ChannelLayout::Mono, samples)
    }

    fn push_four_four_groove(
        beats: &mut Vec<f32>,
        tone_events: &mut Vec<(usize, &'static [f32], f32)>,
        start_beat: usize,
        section: GrooveSection,
    ) {
        for bar in 0..section.bars {
            let beat_pattern = section
                .bar_patterns
                .and_then(|patterns| patterns.get(bar).copied())
                .unwrap_or(section.beat_pattern);
            let is_dropout_bar = section.dropout_bars.contains(&bar);

            for beat_in_bar in 0..4usize {
                let beat_index = start_beat + bar * 4 + beat_in_bar;
                let beat_amplitude = if is_dropout_bar {
                    0.35 * beat_pattern[beat_in_bar]
                } else {
                    beat_pattern[beat_in_bar]
                };
                beats.push(beat_amplitude);

                if !is_dropout_bar {
                    tone_events.push((beat_index, KICK_TONES, 0.18 * beat_amplitude));
                    if beat_in_bar == 1 || beat_in_bar == 3 {
                        tone_events.push((beat_index, SNARE_TONES, 0.28));
                    } else {
                        tone_events.push((beat_index, HAT_TONES, 0.12));
                    }
                } else if beat_in_bar == 3 {
                    tone_events.push((beat_index, HAT_TONES, 0.08));
                }
            }

            let bar_chord = section
                .bar_chords
                .and_then(|plan| plan.get(bar).copied())
                .or_else(|| {
                    if bar % section.chord_every_bars == 0 {
                        Some(
                            section.chord_cycle
                                [(bar / section.chord_every_bars) % section.chord_cycle.len()],
                        )
                    } else {
                        None
                    }
                });
            if let Some(chord) = bar_chord {
                tone_events.push((
                    start_beat + bar * 4,
                    chord,
                    if is_dropout_bar { 0.65 } else { 0.55 },
                ));
            }
        }

        if let Some((offset_beats, chord, amplitude)) = section.section_marker {
            tone_events.push((start_beat + offset_beats, chord, amplitude));
        }
    }

    #[test]
    fn beat_tracker_detects_click_track_tempo() {
        let audio = click_track(48_000, 120.0, 8.0);
        let mut tracker = BeatTracker::new(BeatTrackerConfig::default());
        let result = tracker.analyze(&audio);

        assert!(
            (result.bpm - 120.0).abs() < 3.0,
            "detected bpm {}",
            result.bpm
        );
        assert!(
            result.confidence.0 > 0.2,
            "confidence {}",
            result.confidence.0
        );
        assert!(result.beat_positions_seconds.len() >= 6);
        assert!(result.meter.is_none());
    }

    #[test]
    fn beat_tracker_detects_slower_click_track_tempo() {
        let audio = click_track(48_000, 90.0, 8.0);
        let mut tracker = BeatTracker::new(BeatTrackerConfig::default());
        let result = tracker.analyze(&audio);

        assert!(
            (result.bpm - 90.0).abs() < 3.0,
            "detected bpm {}",
            result.bpm
        );
        assert!(
            result.confidence.0 > 0.15,
            "confidence {}",
            result.confidence.0
        );
    }

    #[test]
    fn beat_tracker_refines_integer_click_track_tempo_to_sub_tenth_bpm() {
        let mut tracker = BeatTracker::new(BeatTrackerConfig::default());

        let fast = tracker.analyze(&click_track(48_000, 120.0, 8.0));
        assert!(
            (fast.bpm - 120.0).abs() < 0.1,
            "refined detected bpm {}",
            fast.bpm
        );
        assert!(
            fast.tempo_candidates
                .first()
                .map(|candidate| (candidate.bpm - 120.0).abs() < 0.1)
                .unwrap_or(false),
            "top tempo candidate {:?}",
            fast.tempo_candidates.first()
        );

        let slow = tracker.analyze(&click_track(48_000, 90.0, 8.0));
        assert!(
            (slow.bpm - 90.0).abs() < 0.1,
            "refined detected bpm {}",
            slow.bpm
        );
        assert!(
            slow.tempo_candidates
                .first()
                .map(|candidate| (candidate.bpm - 90.0).abs() < 0.1)
                .unwrap_or(false),
            "top tempo candidate {:?}",
            slow.tempo_candidates.first()
        );
    }

    #[test]
    fn beat_tracker_exposes_stable_local_tempo_for_integer_click_track() {
        let mut tracker = BeatTracker::new(BeatTrackerConfig::default());
        let result = tracker.analyze(&click_track(48_000, 120.0, 8.0));

        assert!(result.tempo_diagnostics.interval_tempi.len() >= 10);
        assert!(result.tempo_diagnostics.windowed_tempi.len() >= 6);
        assert!(
            (result.tempo_diagnostics.median_bpm - 120.0).abs() < 0.15,
            "median local tempo {}",
            result.tempo_diagnostics.median_bpm
        );
        assert!(
            result.tempo_diagnostics.mean_abs_deviation_bpm < 0.15,
            "local tempo MAD {}",
            result.tempo_diagnostics.mean_abs_deviation_bpm
        );
        assert!(
            result.tempo_diagnostics.windowed_mean_abs_deviation_bpm
                < result.tempo_diagnostics.mean_abs_deviation_bpm,
            "windowed MAD {} raw MAD {}",
            result.tempo_diagnostics.windowed_mean_abs_deviation_bpm,
            result.tempo_diagnostics.mean_abs_deviation_bpm
        );
        assert!(
            (result.tempo_diagnostics.core_windowed_median_bpm - 120.0).abs() < 0.15,
            "core windowed median {}",
            result.tempo_diagnostics.core_windowed_median_bpm
        );
        assert!(
            result
                .tempo_diagnostics
                .core_windowed_mean_abs_deviation_bpm
                < 0.15,
            "core windowed MAD {}",
            result
                .tempo_diagnostics
                .core_windowed_mean_abs_deviation_bpm
        );
        assert!(
            result.tempo_diagnostics.boundary_bias_bpm > 0.05,
            "boundary bias {}",
            result.tempo_diagnostics.boundary_bias_bpm
        );
        assert!(
            result.tempo_diagnostics.boundary_bias_bpm
                < result.tempo_diagnostics.windowed_drift_span_bpm,
            "boundary bias {} full windowed span {}",
            result.tempo_diagnostics.boundary_bias_bpm,
            result.tempo_diagnostics.windowed_drift_span_bpm
        );
        assert_eq!(
            result.tempo_diagnostics.trend.direction,
            super::TempoTrendDirection::Stable
        );
        assert!(
            result.tempo_diagnostics.trend.total_drift_bpm.abs() < 0.15,
            "tempo drift {}",
            result.tempo_diagnostics.trend.total_drift_bpm
        );
        assert_eq!(
            result.tempo_diagnostics.beat_grid_error.residuals.len(),
            result.beat_positions_seconds.len()
        );
        assert!(
            result
                .tempo_diagnostics
                .beat_grid_error
                .mean_abs_residual_ms
                < 6.0,
            "mean abs residual ms {}",
            result
                .tempo_diagnostics
                .beat_grid_error
                .mean_abs_residual_ms
        );
        assert_eq!(
            result.tempo_interpretation.recommendation,
            super::TempoRecommendation::SnapInteger
        );
        assert_eq!(
            result.tempo_interpretation.reason,
            super::TempoInterpretationReason::NearIntegerPulse
        );
        assert_eq!(result.tempo_interpretation.snapped_bpm, Some(120.0));
        assert!(result.tempo_interpretation.profile.snap_error_bpm < 0.12);
        assert!(result.tempo_interpretation.profile.stability_score.0 > 0.75);
        assert_eq!(result.tempo_state.action, super::TempoStateAction::Lock);
        assert_eq!(
            result.tempo_state.reason,
            super::TempoStateReason::StableIntegerTempo
        );
        assert_eq!(
            result.tempo_state.continuity.action,
            super::TempoContinuityAction::Lock
        );
        assert_eq!(
            result.tempo_state.continuity.source,
            super::TempoContinuitySource::CurrentTempo
        );
        assert_eq!(
            result.tempo_state.continuity.reason,
            super::TempoContinuityReason::IntegerTempoSnap
        );
        assert_eq!(
            result.tempo_state.continuity.provenance,
            super::TempoContinuityProvenance::IntegerSnap
        );
        assert_eq!(
            result.tempo_state.continuity.severity,
            super::TempoContinuitySeverity::Confirmed
        );
        assert_eq!(
            result.tempo_state.continuity.history,
            super::TempoContinuityHistory::Reinforcing
        );
        assert_eq!(
            result.tempo_state.continuity.expiry.guaranteed_until_beats,
            16
        );
        assert_eq!(
            result.tempo_state.continuity.expiry.downgrade_after_beats,
            20
        );
        assert_eq!(result.tempo_state.continuity.expiry.clear_after_beats, 28);
        assert!(result.tempo_state.continuity.refresh_strength.0 > 0.9);
    }

    #[test]
    fn beat_tracker_exposes_non_empty_onset_envelope_for_click_track() {
        let audio = click_track(48_000, 120.0, 4.0);
        let mut tracker = BeatTracker::new(BeatTrackerConfig::default());
        let result = tracker.analyze(&audio);

        assert!(!result.onset_envelope.is_empty());
        assert!(result.onset_envelope.iter().any(|value| *value > 0.5));
    }

    #[test]
    fn beat_tracker_returns_zero_for_silence() {
        let audio = AudioBuffer::new(
            SampleRate(48_000),
            ChannelLayout::Mono,
            signal_primitives::FrameCount(48_000),
        );
        let mut tracker = BeatTracker::new(BeatTrackerConfig::default());
        let result = tracker.analyze(&audio);

        assert_eq!(result.bpm, 0.0);
        assert_eq!(result.confidence.0, 0.0);
        assert!(result.beat_positions_seconds.is_empty());
        assert!(result.tempo_candidates.is_empty());
        assert_eq!(result.tempo_ambiguity.0, 0.0);
        assert!(result.meter.is_none());
    }

    #[test]
    fn beat_tracker_detects_swung_click_track_tempo() {
        let audio = grid_click_track(
            48_000,
            120.0,
            2,
            8.0,
            &[1.0, 0.45, 0.85, 0.35],
            Some(2.0 / 3.0),
        );
        let mut tracker = BeatTracker::new(BeatTrackerConfig::default());
        let result = tracker.analyze(&audio);

        assert!(
            (result.bpm - 120.0).abs() < 3.0,
            "detected bpm {}",
            result.bpm
        );
        assert!(
            result.confidence.0 > 0.15,
            "confidence {}",
            result.confidence.0
        );
    }

    #[test]
    fn beat_tracker_handles_syncopated_pattern_without_halving_tempo() {
        let audio = grid_click_track(
            48_000,
            120.0,
            2,
            8.0,
            &[1.0, 0.0, 0.35, 0.8, 0.95, 0.0, 0.3, 0.75],
            None,
        );
        let mut tracker = BeatTracker::new(BeatTrackerConfig::default());
        let result = tracker.analyze(&audio);

        assert!(
            (result.bpm - 120.0).abs() < 3.5,
            "detected bpm {}",
            result.bpm
        );
    }

    #[test]
    fn beat_tracker_prefers_base_tempo_over_double_time_subdivisions() {
        let audio = grid_click_track(48_000, 90.0, 2, 8.0, &[1.0, 0.3], None);
        let mut tracker = BeatTracker::new(BeatTrackerConfig::default());
        let result = tracker.analyze(&audio);

        assert!(
            (result.bpm - 90.0).abs() < 3.0,
            "detected bpm {}",
            result.bpm
        );
        assert!(
            result.confidence.0 > 0.1,
            "confidence {}",
            result.confidence.0
        );
        assert!(result.tempo_candidates.len() >= 2);
        assert!(result
            .tempo_candidates
            .iter()
            .skip(1)
            .any(|candidate| (candidate.bpm - 180.0).abs() < 4.0));
        assert!(result.tempo_ambiguity.0 > 0.2);
    }

    #[test]
    fn beat_tracker_selects_consistent_phase_over_single_loud_offbeat() {
        let sample_rate = 48_000;
        let bpm = 120.0;
        let mut audio = click_track(sample_rate, bpm, 8.0);
        let offbeat_index = (60.0 / bpm * sample_rate as f32 / 2.0).round() as usize;
        add_click(audio.samples_mut(), offbeat_index, 1.25);

        let mut tracker = BeatTracker::new(BeatTrackerConfig::default());
        let result = tracker.analyze(&audio);

        assert!(
            (result.bpm - 120.0).abs() < 3.0,
            "detected bpm {}",
            result.bpm
        );
        let quarter_note_seconds = 60.0 / bpm;
        assert!(result.beat_positions_seconds.iter().take(6).all(|beat| {
            let nearest_grid = (*beat / quarter_note_seconds).round() * quarter_note_seconds;
            (nearest_grid - *beat).abs() < 0.08
        }));
    }

    #[test]
    fn beat_tracker_infers_four_four_bar_phase_from_accent_pattern() {
        let bpm = 120.0;
        let audio = grid_click_track(48_000, bpm, 1, 12.0, &[1.0, 0.35, 0.55, 0.4], None);
        let mut tracker = BeatTracker::new(BeatTrackerConfig::default());
        let result = tracker.analyze(&audio);
        let meter = result.meter.as_ref().expect("meter estimate");

        assert_eq!(meter.beats_per_bar, 4);
        assert!(
            meter.confidence.0 > 0.2,
            "confidence {}",
            meter.confidence.0
        );
        let bar_seconds = 60.0 / bpm * 4.0;
        assert!(meter
            .downbeat_positions_seconds
            .iter()
            .take(4)
            .all(|downbeat| {
                let nearest_bar = (*downbeat / bar_seconds).round() * bar_seconds;
                (nearest_bar - *downbeat).abs() < 0.08
            }));
    }

    #[test]
    fn beat_tracker_infers_three_four_bar_phase_from_waltz_pattern() {
        let bpm = 120.0;
        let audio = grid_click_track(48_000, bpm, 1, 12.0, &[1.0, 0.4, 0.45], None);
        let mut tracker = BeatTracker::new(BeatTrackerConfig::default());
        let result = tracker.analyze(&audio);
        let meter = result.meter.as_ref().expect("meter estimate");

        assert_eq!(meter.beats_per_bar, 3);
        assert!(
            meter.confidence.0 > 0.2,
            "confidence {}",
            meter.confidence.0
        );
        let bar_seconds = 60.0 / bpm * 3.0;
        assert!(meter
            .downbeat_positions_seconds
            .iter()
            .take(4)
            .all(|downbeat| {
                let nearest_bar = (*downbeat / bar_seconds).round() * bar_seconds;
                (nearest_bar - *downbeat).abs() < 0.08
            }));
    }

    #[test]
    fn beat_tracker_infers_four_four_after_two_beat_pickup() {
        let bpm = 120.0;
        let mut beats = vec![0.45, 0.7];
        beats.extend_from_slice(&[1.0, 0.35, 0.55, 0.4]);
        beats.extend_from_slice(&[1.0, 0.35, 0.55, 0.4]);
        beats.extend_from_slice(&[1.0, 0.35, 0.55, 0.4]);

        let audio = beat_sequence_track(48_000, bpm, &beats, &[]);
        let mut tracker = BeatTracker::new(BeatTrackerConfig::default());
        let result = tracker.analyze(&audio);
        let meter = result.meter.as_ref().expect("meter estimate");

        assert_eq!(meter.beats_per_bar, 4);
        assert!(
            meter.confidence.0 > 0.2,
            "confidence {}",
            meter.confidence.0
        );
        let beat_seconds = 60.0 / bpm;
        assert!((meter.downbeat_positions_seconds[0] - 2.0 * beat_seconds).abs() < 0.08);
    }

    #[test]
    fn beat_tracker_uses_spectral_change_to_support_weak_four_four_meter() {
        let bpm = 120.0;
        let beats = [
            0.45, 0.35, 0.4, 0.35, 0.45, 0.35, 0.4, 0.35, 0.45, 0.35, 0.4, 0.35, 0.45, 0.35, 0.4,
            0.35,
        ];
        let tone_events: &[(usize, &'static [f32], f32)] = &[
            (0, &[220.0, 277.18, 329.63], 0.85),
            (4, &[261.63, 329.63, 392.0], 0.85),
            (8, &[196.0, 246.94, 293.66], 0.85),
            (12, &[246.94, 311.13, 369.99], 0.85),
        ];
        let audio = beat_sequence_track(48_000, bpm, &beats, tone_events);
        let mut tracker = BeatTracker::new(BeatTrackerConfig::default());
        let result = tracker.analyze(&audio);
        let meter = result.meter.as_ref().expect("meter estimate");

        assert_eq!(meter.beats_per_bar, 4);
        assert!(
            meter.confidence.0 > 0.18,
            "confidence {}",
            meter.confidence.0
        );
    }

    #[test]
    fn beat_tracker_suppresses_meter_on_mixed_bar_lengths() {
        let bpm = 120.0;
        let beats = [
            1.0, 0.35, 0.55, 0.4, 1.0, 0.4, 0.45, 1.0, 0.35, 0.55, 0.4, 1.0, 0.4, 0.45,
        ];
        let audio = beat_sequence_track(48_000, bpm, &beats, &[]);
        let mut tracker = BeatTracker::new(BeatTrackerConfig::default());
        let result = tracker.analyze(&audio);

        assert!(result.meter.is_none());
    }

    #[test]
    fn beat_tracker_handles_realistic_weak_backbeat_fixture() {
        let bpm = 118.0;
        let mut fixture = FixtureBuilder::new();
        fixture.push_four_four_section(GrooveSection {
            bars: 8,
            beat_pattern: [0.42, 0.24, 0.34, 0.22],
            chord_cycle: &[CHORD_A, CHORD_B, CHORD_C, CHORD_D],
            chord_every_bars: 2,
            section_marker: None,
            bar_patterns: None,
            bar_chords: None,
            dropout_bars: &[],
        });

        let audio = fixture.build(48_000, bpm);
        let mut tracker = BeatTracker::new(BeatTrackerConfig::default());
        let result = tracker.analyze(&audio);
        let meter = result.meter.as_ref().expect("meter estimate");

        assert!(
            (result.bpm - bpm).abs() < 3.0,
            "detected bpm {}",
            result.bpm
        );
        assert_eq!(meter.beats_per_bar, 4);
        assert!(
            meter.confidence.0 > 0.2,
            "confidence {}",
            meter.confidence.0
        );
    }

    #[test]
    fn beat_tracker_preserves_four_four_across_section_transition_fixture() {
        let bpm = 122.0;
        let mut fixture = FixtureBuilder::new();
        fixture.push_four_four_section(GrooveSection {
            bars: 4,
            beat_pattern: [0.48, 0.22, 0.36, 0.26],
            chord_cycle: &[CHORD_A, CHORD_B],
            chord_every_bars: 2,
            section_marker: Some((16, CHORD_C, 0.9)),
            bar_patterns: None,
            bar_chords: None,
            dropout_bars: &[],
        });
        fixture.push_four_four_section(GrooveSection {
            bars: 4,
            beat_pattern: [0.55, 0.26, 0.38, 0.28],
            chord_cycle: &[CHORD_C, CHORD_D],
            chord_every_bars: 2,
            section_marker: None,
            bar_patterns: None,
            bar_chords: None,
            dropout_bars: &[],
        });

        let audio = fixture.build(48_000, bpm);
        let mut tracker = BeatTracker::new(BeatTrackerConfig::default());
        let result = tracker.analyze(&audio);
        let meter = result.meter.as_ref().expect("meter estimate");

        assert!(
            (result.bpm - bpm).abs() < 3.0,
            "detected bpm {}",
            result.bpm
        );
        assert_eq!(meter.beats_per_bar, 4);
        assert!(
            meter.confidence.0 > 0.2,
            "confidence {}",
            meter.confidence.0
        );
        let bar_seconds = 60.0 / bpm * 4.0;
        assert!(meter
            .downbeat_positions_seconds
            .iter()
            .take(6)
            .all(|downbeat| {
                let nearest_bar = (*downbeat / bar_seconds).round() * bar_seconds;
                (nearest_bar - *downbeat).abs() < 0.09
            }));
    }

    #[test]
    fn beat_tracker_calibrates_meter_confidence_between_neutral_and_structured_fixtures() {
        let (_, neutral) = analyze_preset(RhythmPreset::NeutralClick120);
        let (_, structured) = analyze_preset(RhythmPreset::StructuredHarmony120(
            HarmonicRhythmVariant::Active,
        ));
        let structured_meter = structured.meter.as_ref().expect("structured meter");

        assert!(neutral.meter.is_none());
        assert_eq!(structured_meter.beats_per_bar, 4);
        assert!(structured_meter.confidence.0 > 0.2);
        assert_eq!(
            structured_meter.detection_kind,
            super::MeterDetectionKind::WholeTrack
        );
        assert_eq!(structured_meter.trust, super::MeterTrustLevel::Stable);
        assert!(structured_meter.recovery.is_none());
        assert!(structured_meter.confidence_breakdown.support > 0.6);
        assert!(
            structured_meter.support_profile.whole_track_strength.0
                > structured_meter.support_profile.segment_recovery_strength.0
        );
        assert_eq!(
            structured_meter
                .support_profile
                .recovery_duration_strength
                .0,
            0.0
        );
    }

    #[test]
    fn beat_tracker_calibrates_tempo_ambiguity_between_stable_and_subdivided_fixtures() {
        let (_, stable) = analyze_preset(RhythmPreset::NeutralClick120);
        let (_, ambiguous) = analyze_preset(RhythmPreset::AmbiguousSubdivision90);

        assert!(ambiguous.tempo_ambiguity.0 > stable.tempo_ambiguity.0);
        assert!(ambiguous.tempo_candidates.len() >= 2);
        assert!(stable.confidence.0 >= ambiguous.confidence.0);
    }

    #[test]
    fn beat_tracker_calibrates_local_tempo_drift_between_stable_and_irregular_fixtures() {
        let (_, stable) = analyze_preset(RhythmPreset::NeutralClick120);
        let slow = analyze_fixture(&click_track(48_000, 90.0, 8.0));
        let (_, weak_backbeat) = analyze_preset(RhythmPreset::WeakBackbeat118);
        let (_, section) = analyze_preset(RhythmPreset::SectionTransition122);
        let (_, ambiguous) = analyze_preset(RhythmPreset::AmbiguousSubdivision90);

        assert!(
            weak_backbeat.tempo_diagnostics.mean_abs_deviation_bpm
                > stable.tempo_diagnostics.mean_abs_deviation_bpm
        );
        assert!(
            section.tempo_diagnostics.drift_span_bpm >= stable.tempo_diagnostics.drift_span_bpm
        );
        assert!(!weak_backbeat.tempo_diagnostics.windowed_tempi.is_empty());
        assert!(!section.tempo_diagnostics.windowed_tempi.is_empty());
        assert!(stable.tempo_diagnostics.boundary_bias_bpm > 0.0);
        assert!(
            section.tempo_diagnostics.trend.fit_mean_abs_deviation_bpm
                >= stable.tempo_diagnostics.trend.fit_mean_abs_deviation_bpm
        );
        assert!(
            slow.tempo_diagnostics
                .beat_grid_error
                .edge_mean_abs_residual_ms
                > slow
                    .tempo_diagnostics
                    .beat_grid_error
                    .core_mean_abs_residual_ms
        );
        assert!(
            slow.tempo_diagnostics
                .beat_grid_error
                .mean_abs_anchored_drift_ms
                > stable
                    .tempo_diagnostics
                    .beat_grid_error
                    .mean_abs_anchored_drift_ms
        );
        assert_eq!(
            slow.tempo_interpretation.recommendation,
            super::TempoRecommendation::SnapInteger
        );
        assert_eq!(
            slow.tempo_interpretation.reason,
            super::TempoInterpretationReason::NearIntegerPulse
        );
        assert!(
            (slow.tempo_interpretation.recommended_bpm - 90.0).abs() < 0.1,
            "slow recommended bpm {}",
            slow.tempo_interpretation.recommended_bpm
        );
        assert!(
            slow.tempo_interpretation.profile.boundary_edge_gap_ms > 0.0,
            "slow boundary edge gap {}",
            slow.tempo_interpretation.profile.boundary_edge_gap_ms
        );
        assert_eq!(
            slow.tempo_diagnostics.stability_scope.scope,
            super::TempoStabilityScope::CoreStableOnly
        );
        assert_eq!(slow.tempo_state.action, super::TempoStateAction::Monitor);
        assert_eq!(
            slow.tempo_state.reason,
            super::TempoStateReason::CoreStableTempo
        );
        assert_eq!(
            slow.tempo_state.continuity.action,
            super::TempoContinuityAction::Reacquire
        );
        assert_eq!(
            slow.tempo_state.continuity.source,
            super::TempoContinuitySource::CurrentTempo
        );
        assert_eq!(
            slow.tempo_state.continuity.reason,
            super::TempoContinuityReason::RevalidationDecay
        );
        assert_eq!(
            slow.tempo_state.continuity.provenance,
            super::TempoContinuityProvenance::GuardedRefinedEstimate
        );
        assert_eq!(
            slow.tempo_state.continuity.severity,
            super::TempoContinuitySeverity::Fragile
        );
        assert_eq!(
            slow.tempo_state.continuity.history,
            super::TempoContinuityHistory::Preserving
        );
        assert!(matches!(
            slow.tempo_state.continuity.trigger,
            super::TempoContinuityTrigger::StableRevalidation
                | super::TempoContinuityTrigger::AmbiguityCarry
        ));
        assert!(slow.tempo_state.continuity.unresolved.beats >= 4);
        assert!(matches!(
            slow.tempo_state.continuity.causes.primary,
            super::TempoContinuityCause::StableTempoEvidence
                | super::TempoContinuityCause::TempoAmbiguity
        ));
        assert_eq!(slow.tempo_state.continuity.expiry.guaranteed_until_beats, 4);
        assert_eq!(slow.tempo_state.continuity.expiry.downgrade_after_beats, 8);
        assert_eq!(slow.tempo_state.continuity.expiry.clear_after_beats, 12);
        assert_eq!(
            weak_backbeat.tempo_interpretation.recommendation,
            super::TempoRecommendation::UseRefined
        );
        assert_eq!(
            weak_backbeat.tempo_interpretation.reason,
            super::TempoInterpretationReason::StableRefinedPulse
        );
        assert_eq!(
            weak_backbeat.tempo_state.action,
            super::TempoStateAction::Lock
        );
        assert_eq!(
            weak_backbeat.tempo_state.reason,
            super::TempoStateReason::StableRefinedTempo
        );
        assert_eq!(
            weak_backbeat.tempo_state.continuity.action,
            super::TempoContinuityAction::Lock
        );
        assert_eq!(
            weak_backbeat.tempo_state.continuity.source,
            super::TempoContinuitySource::CurrentTempo
        );
        assert_eq!(
            weak_backbeat.tempo_state.continuity.reason,
            super::TempoContinuityReason::StableTempo
        );
        assert_eq!(
            weak_backbeat.tempo_state.continuity.provenance,
            super::TempoContinuityProvenance::StableRefinedEstimate
        );
        assert_eq!(
            weak_backbeat.tempo_state.continuity.severity,
            super::TempoContinuitySeverity::Confirmed
        );
        assert_eq!(
            weak_backbeat.tempo_state.continuity.history,
            super::TempoContinuityHistory::Reinforcing
        );
        assert_eq!(
            weak_backbeat.tempo_state.continuity.trigger,
            super::TempoContinuityTrigger::StableRevalidation
        );
        assert_eq!(
            weak_backbeat.tempo_state.continuity.causes.primary,
            super::TempoContinuityCause::StableTempoEvidence
        );
        assert_eq!(
            weak_backbeat
                .tempo_state
                .continuity
                .expiry
                .max_failed_revalidations,
            3
        );
        assert!(matches!(
            ambiguous.tempo_interpretation.recommendation,
            super::TempoRecommendation::UseCoreWindow | super::TempoRecommendation::UseRefined
        ));
        assert!(matches!(
            ambiguous.tempo_interpretation.trust,
            super::TempoTrustLevel::Guarded | super::TempoTrustLevel::Stable
        ));
        assert!(ambiguous.tempo_interpretation.profile.stability_score.0 < 0.85);
        assert!(matches!(
            ambiguous.tempo_state.action,
            super::TempoStateAction::Monitor
                | super::TempoStateAction::Lock
                | super::TempoStateAction::Defer
        ));
        assert!(matches!(
            ambiguous.tempo_state.reason,
            super::TempoStateReason::CoreWindowFallback
                | super::TempoStateReason::StableRefinedTempo
                | super::TempoStateReason::CoreStableTempo
                | super::TempoStateReason::StableTempoWithEdgeDamage
                | super::TempoStateReason::TempoDeferred
        ));
        assert!(matches!(
            ambiguous.tempo_state.continuity.action,
            super::TempoContinuityAction::Retain
                | super::TempoContinuityAction::Lock
                | super::TempoContinuityAction::Clear
        ));
        assert!(matches!(
            ambiguous.tempo_state.continuity.source,
            super::TempoContinuitySource::CoreWindow
                | super::TempoContinuitySource::CurrentTempo
                | super::TempoContinuitySource::Cleared
        ));
        assert!(matches!(
            ambiguous.tempo_state.continuity.reason,
            super::TempoContinuityReason::CoreWindowCarry
                | super::TempoContinuityReason::StableTempo
                | super::TempoContinuityReason::InsufficientEvidence
        ));
        assert!(matches!(
            ambiguous.tempo_state.continuity.provenance,
            super::TempoContinuityProvenance::CoreWindowEstimate
                | super::TempoContinuityProvenance::StableRefinedEstimate
                | super::TempoContinuityProvenance::NoTempo
        ));
        assert!(matches!(
            ambiguous.tempo_state.continuity.severity,
            super::TempoContinuitySeverity::Guarded
                | super::TempoContinuitySeverity::Confirmed
                | super::TempoContinuitySeverity::Cleared
        ));
        assert!(matches!(
            ambiguous.tempo_state.continuity.history,
            super::TempoContinuityHistory::Preserving
                | super::TempoContinuityHistory::Reinforcing
                | super::TempoContinuityHistory::Degrading
        ));
        assert!(matches!(
            ambiguous.tempo_state.continuity.trigger,
            super::TempoContinuityTrigger::BoundaryDrift
                | super::TempoContinuityTrigger::StableRevalidation
                | super::TempoContinuityTrigger::EvidenceLoss
        ));
        assert!(matches!(
            ambiguous.tempo_state.continuity.causes.primary,
            super::TempoContinuityCause::BoundaryDrift
                | super::TempoContinuityCause::StableTempoEvidence
                | super::TempoContinuityCause::EvidenceLoss
                | super::TempoContinuityCause::TempoAmbiguity
        ));
        assert!(matches!(
            ambiguous
                .tempo_state
                .continuity
                .expiry
                .max_failed_revalidations,
            0 | 3
        ));
    }

    #[test]
    fn beat_tracker_resolves_tempo_consumption_across_real_analysis_paths() {
        let (_, neutral) = analyze_preset(RhythmPreset::NeutralClick120);
        let neutral_consumption = neutral.tempo_consumption(Some(119.5));

        assert_eq!(neutral_consumption.action, super::TempoStateAction::Lock);
        assert_eq!(
            neutral_consumption.continuity_action,
            super::TempoContinuityAction::Lock
        );
        assert_eq!(
            neutral_consumption.current.source,
            super::TempoConsumptionSource::SnappedCurrentTempo
        );
        assert_eq!(neutral_consumption.current.bpm, Some(120.0));
        assert_eq!(
            neutral_consumption.fallback.source,
            super::TempoConsumptionSource::SnappedCurrentTempo
        );
        assert_eq!(neutral_consumption.fallback.bpm, Some(120.0));
        assert_eq!(neutral_consumption.fallback_after_beats, 20);

        let slow = analyze_fixture(&click_track(48_000, 90.0, 8.0));
        let slow_with_prior = slow.tempo_consumption(Some(89.75));
        let slow_without_prior = slow.tempo_consumption(None);

        assert_eq!(slow_with_prior.action, super::TempoStateAction::Monitor);
        assert_eq!(
            slow_with_prior.continuity_action,
            super::TempoContinuityAction::Reacquire
        );
        assert_eq!(
            slow_with_prior.current.source,
            super::TempoConsumptionSource::SnappedCurrentTempo
        );
        assert!(slow_with_prior
            .current
            .bpm
            .map(|bpm| (bpm - 90.0).abs() < 0.1)
            .unwrap_or(false));
        assert_eq!(
            slow_with_prior.fallback.source,
            super::TempoConsumptionSource::PriorTempo
        );
        assert_eq!(slow_with_prior.fallback.bpm, Some(89.75));
        assert_eq!(slow_with_prior.fallback_after_beats, 8);
        assert_eq!(
            slow_without_prior.fallback.source,
            super::TempoConsumptionSource::NoTempo
        );
        assert_eq!(slow_without_prior.fallback.bpm, None);
        assert_eq!(slow_without_prior.fallback_after_beats, 8);
        assert_eq!(
            slow_with_prior.stability_scope.scope,
            super::TempoStabilityScope::CoreStableOnly
        );

        let (_, weak_backbeat) = analyze_preset(RhythmPreset::WeakBackbeat118);
        let weak_backbeat_consumption = weak_backbeat.tempo_consumption(Some(118.2));

        assert_eq!(
            weak_backbeat_consumption.action,
            super::TempoStateAction::Lock
        );
        assert_eq!(
            weak_backbeat_consumption.continuity_action,
            super::TempoContinuityAction::Lock
        );
        assert_eq!(
            weak_backbeat_consumption.current.source,
            super::TempoConsumptionSource::RefinedCurrentTempo
        );
        assert!(weak_backbeat_consumption
            .current
            .bpm
            .map(|bpm| (bpm - weak_backbeat.tempo_interpretation.recommended_bpm).abs() < 0.001)
            .unwrap_or(false));
        assert_eq!(
            weak_backbeat_consumption.fallback.source,
            super::TempoConsumptionSource::RefinedCurrentTempo
        );
        assert!(weak_backbeat_consumption
            .fallback
            .bpm
            .map(|bpm| (bpm - weak_backbeat.tempo_interpretation.recommended_bpm).abs() < 0.001)
            .unwrap_or(false));

        let silence = AudioBuffer::new(
            SampleRate(48_000),
            ChannelLayout::Mono,
            signal_primitives::FrameCount(48_000),
        );
        let cleared = analyze_fixture(&silence).tempo_consumption(Some(120.0));

        assert_eq!(cleared.action, super::TempoStateAction::Defer);
        assert_eq!(
            cleared.continuity_action,
            super::TempoContinuityAction::Clear
        );
        assert_eq!(
            cleared.current.source,
            super::TempoConsumptionSource::NoTempo
        );
        assert_eq!(cleared.current.bpm, None);
        assert_eq!(
            cleared.fallback.source,
            super::TempoConsumptionSource::NoTempo
        );
        assert_eq!(cleared.fallback.bpm, None);
        assert_eq!(cleared.fallback_after_beats, 0);
    }

    #[test]
    fn beat_tracker_exposes_tempo_structure_summary_for_whole_track_stable_click_track() {
        let result = analyze_fixture(&click_track(48_000, 120.0, 8.0));
        let summary = result.tempo_structure_summary();

        assert_eq!(summary.trust, super::TempoTrustLevel::Stable);
        assert_eq!(
            summary.recommendation,
            super::TempoRecommendation::SnapInteger
        );
        assert_eq!(
            summary.stability_scope.scope,
            super::TempoStabilityScope::WholeTrackStable
        );
        assert_eq!(summary.selected_bpm, Some(120.0));
        assert_eq!(summary.continuity.action, super::TempoStateAction::Lock);
        assert_eq!(
            summary.continuity.continuity_action,
            super::TempoContinuityAction::Lock
        );
        assert_eq!(
            summary.continuity.current.source,
            super::TempoConsumptionSource::SnappedCurrentTempo
        );
        assert_eq!(summary.continuity.fallback_after_beats, 20);
        assert_eq!(summary.segments.len(), 1);
        assert_eq!(
            summary.segments[0].kind,
            super::TempoSegmentKind::WholeTrack
        );
        assert!((summary.segments[0].representative_bpm - summary.core_window_bpm).abs() < 1.0);
        assert!(summary.segments[0].coverage.0 >= 0.99);
    }

    #[test]
    fn tempo_structure_summary_surfaces_localized_edge_damage_segments() {
        let mut diagnostics = synthetic_tempo_diagnostics_with_counts(
            127.94273, 0.064, -0.1097, 0.48279, 45.998, 45.774, 83.272, 86.989, 738, 735, 739,
        );
        diagnostics.beat_interval_outliers = super::BeatIntervalOutlierDiagnostics {
            total_intervals: 738,
            retained_intervals: 670,
            rejected_intervals: 68,
            leading_rejected_intervals: 0,
            trailing_rejected_intervals: 3,
            median_interval: 60.0 / 127.94273,
            median_abs_deviation: 0.000_607,
            max_rejected_deviation_ratio: 0.384,
        };
        diagnostics.edge_trimmed_stable_span = Some(super::BeatGridCoreSpanDiagnostics {
            start_beat_index: 0,
            end_beat_index: 735,
            start_seconds: 0.447,
            end_seconds: 345.333,
            coverage: super::Confidence::new(0.996),
            retained_windows: 732,
            total_windows: 735,
            trimmed_leading_windows: 0,
            trimmed_trailing_windows: 3,
            interior_rejected_windows: 14,
        });
        diagnostics.stable_core_span = Some(super::BeatGridCoreSpanDiagnostics {
            start_beat_index: 216,
            end_beat_index: 706,
            start_seconds: 101.698,
            end_seconds: 331.641,
            coverage: super::Confidence::new(0.664),
            retained_windows: 487,
            total_windows: 735,
            trimmed_leading_windows: 216,
            trimmed_trailing_windows: 32,
            interior_rejected_windows: 0,
        });
        diagnostics.stability_scope = super::classify_tempo_stability_scope(
            diagnostics.windowed_tempi.len(),
            &diagnostics.beat_interval_outliers,
            diagnostics.edge_trimmed_stable_span,
            diagnostics.stable_core_span,
        );

        let interpretation = super::interpret_tempo(
            127.96191,
            super::Confidence::new(0.666),
            super::Confidence::new(1.0),
            &diagnostics,
        );
        let result = synthetic_tempo_structure_result(
            diagnostics,
            interpretation,
            super::Confidence::new(0.666),
            super::Confidence::new(1.0),
        );
        let summary = result.tempo_structure_summary();

        assert_eq!(
            summary.stability_scope.scope,
            super::TempoStabilityScope::StableWithLocalizedEdgeDamage
        );
        assert_eq!(summary.continuity.action, super::TempoStateAction::Lock);
        assert_eq!(summary.selected_bpm, Some(128.0));
        assert!(summary
            .segments
            .iter()
            .any(|segment| segment.kind == super::TempoSegmentKind::WholeTrack));
        assert!(summary
            .segments
            .iter()
            .any(|segment| segment.kind == super::TempoSegmentKind::EdgeTrimmedStable));
        assert!(summary
            .segments
            .iter()
            .any(|segment| segment.kind == super::TempoSegmentKind::StableCore));
        let edge_trimmed = summary
            .segments
            .iter()
            .find(|segment| segment.kind == super::TempoSegmentKind::EdgeTrimmedStable)
            .unwrap();
        let stable_core = summary
            .segments
            .iter()
            .find(|segment| segment.kind == super::TempoSegmentKind::StableCore)
            .unwrap();
        assert!(edge_trimmed.coverage.0 > stable_core.coverage.0);
        assert!(edge_trimmed.end_beat_index > stable_core.end_beat_index);
    }

    #[test]
    fn beat_tracker_exposes_tempo_structure_summary_for_core_stable_monitoring() {
        let result = analyze_fixture(&click_track(48_000, 90.0, 8.0));
        let summary = result.tempo_structure_summary();

        assert_eq!(
            summary.stability_scope.scope,
            super::TempoStabilityScope::CoreStableOnly
        );
        assert_eq!(summary.continuity.action, super::TempoStateAction::Monitor);
        assert_eq!(
            summary.continuity.continuity_action,
            super::TempoContinuityAction::Reacquire
        );
        assert_eq!(
            summary.continuity.current.source,
            super::TempoConsumptionSource::SnappedCurrentTempo
        );
        assert_eq!(
            summary.continuity.fallback.source,
            super::TempoConsumptionSource::NoTempo
        );
        assert_eq!(summary.continuity.fallback_after_beats, 8);
        assert!(!summary.segments.is_empty());
        assert!(summary
            .segments
            .iter()
            .any(|segment| segment.coverage.0 >= 0.5));
    }

    #[test]
    fn tempo_structure_summary_surfaces_mid_track_unstable_clear_policy() {
        let diagnostics =
            synthetic_tempo_diagnostics(89.9, 0.42, 0.61, 0.38, 58.0, 44.0, 360.0, 92.0);
        let interpretation = synthetic_tempo_interpretation(
            super::TempoRecommendation::Defer,
            super::TempoTrustLevel::Tentative,
            super::TempoInterpretationReason::UnstableTempo,
            89.9,
            None,
            0.38,
            0.03,
            0.8,
            0.3,
        );
        let result = synthetic_tempo_structure_result(
            diagnostics,
            interpretation,
            super::Confidence::new(0.42),
            super::Confidence::new(0.55),
        );
        let summary = result.tempo_structure_summary();

        assert_eq!(
            summary.stability_scope.scope,
            super::TempoStabilityScope::MidTrackUnstable
        );
        assert_eq!(summary.continuity.action, super::TempoStateAction::Defer);
        assert_eq!(
            summary.continuity.continuity_action,
            super::TempoContinuityAction::Clear
        );
        assert_eq!(
            summary.continuity.current.source,
            super::TempoConsumptionSource::NoTempo
        );
        assert_eq!(
            summary.continuity.fallback.source,
            super::TempoConsumptionSource::NoTempo
        );
        assert_eq!(summary.segments.len(), 1);
        assert_eq!(
            summary.segments[0].kind,
            super::TempoSegmentKind::WholeTrack
        );
        assert!((summary.segments[0].representative_bpm - 89.9).abs() < 0.2);
    }

    #[test]
    fn bounded_trailing_windows_preserve_stable_structure_and_tempo_summaries() {
        let (_, audio) = render_preset(
            RhythmPreset::StructuredHarmony120(HarmonicRhythmVariant::Active),
            48_000,
        );
        let full = analyze_fixture(&audio);
        let full_structure = full
            .rhythm_structure_assessment()
            .structure
            .expect("full structure summary");
        let full_tempo = full.tempo_structure_summary();

        for seconds in [6.0, 8.0, 10.0] {
            let bounded =
                analyze_trailing_window(&audio, super::BeatTrackerConfig::default(), seconds);
            let structure = bounded
                .rhythm_structure_assessment()
                .structure
                .expect("bounded structure summary");
            let tempo = bounded.tempo_structure_summary();

            assert_eq!(structure.beats_per_bar, full_structure.beats_per_bar);
            assert!(matches!(
                structure.continuity.action,
                super::MeterStateAction::Lock | super::MeterStateAction::Hold
            ));
            assert!(matches!(
                tempo.stability_scope.scope,
                super::TempoStabilityScope::WholeTrackStable
                    | super::TempoStabilityScope::StableWithLocalizedEdgeDamage
                    | super::TempoStabilityScope::CoreStableOnly
            ));
            assert!(matches!(
                tempo.continuity.action,
                super::TempoStateAction::Lock | super::TempoStateAction::Monitor
            ));
            assert!(tempo.selected_bpm.is_some());
            assert!(
                (tempo.selected_bpm.unwrap_or(0.0) - full_tempo.selected_bpm.unwrap_or(0.0)).abs()
                    < 1.0
            );
            assert!(
                (tempo.core_window_bpm - full_tempo.core_window_bpm).abs() < 1.0,
                "seconds={seconds} core_window={} full={}",
                tempo.core_window_bpm,
                full_tempo.core_window_bpm,
            );
        }
    }

    #[test]
    fn bounded_trailing_windows_preserve_weak_accent_and_actionable_tempo_summary() {
        let (_, audio) = render_preset(RhythmPreset::WeakBackbeat118, 48_000);
        let full = analyze_fixture(&audio);
        let full_tempo = full.tempo_structure_summary();

        for seconds in [10.0, 12.0, 14.0] {
            let bounded =
                analyze_trailing_window(&audio, super::BeatTrackerConfig::default(), seconds);
            let assessment = bounded.rhythm_structure_assessment();
            let tempo = bounded.tempo_structure_summary();

            assert_ne!(
                assessment.ambiguity.kind,
                super::RhythmStructureAmbiguityKind::InsufficientEvidence
            );
            assert!(assessment.ambiguity.confidence.0 > 0.1);
            assert!(
                assessment.structure.is_some() || assessment.fallback.recovery_window_available
            );
            assert!(matches!(
                tempo.continuity.action,
                super::TempoStateAction::Lock | super::TempoStateAction::Monitor
            ));
            assert!(tempo.selected_bpm.is_some());
            assert!(
                (tempo.selected_bpm.unwrap_or(0.0) - full_tempo.selected_bpm.unwrap_or(0.0)).abs()
                    < 1.0
            );
            assert_ne!(
                tempo.continuity.current.source,
                super::TempoConsumptionSource::NoTempo
            );
        }
    }

    #[test]
    fn tempo_interpretation_prefers_refined_when_snap_benefit_is_too_small() {
        let diagnostics = synthetic_tempo_diagnostics(120.0, 0.02, 0.04, 0.03, 1.0, 0.8, 1.4, 1.1);
        let interpretation = super::interpret_tempo(
            120.01,
            super::Confidence::new(0.92),
            super::Confidence::new(0.08),
            &diagnostics,
        );

        assert_eq!(
            interpretation.recommendation,
            super::TempoRecommendation::UseRefined
        );
        assert_eq!(
            interpretation.reason,
            super::TempoInterpretationReason::StableRefinedPulse
        );
        assert!(interpretation.profile.snap_error_bpm < 0.04);
        assert!(interpretation.profile.stability_score.0 > 0.8);
    }

    #[test]
    fn tempo_interpretation_defers_when_edge_pressure_overwhelms_stability() {
        let diagnostics =
            synthetic_tempo_diagnostics(90.0, 2.4, 0.35, 0.28, 60.0, 25.0, 120.0, 140.0);
        let interpretation = super::interpret_tempo(
            89.6,
            super::Confidence::new(0.55),
            super::Confidence::new(0.42),
            &diagnostics,
        );

        assert_eq!(
            interpretation.recommendation,
            super::TempoRecommendation::Defer
        );
        assert_eq!(
            interpretation.reason,
            super::TempoInterpretationReason::UnstableTempo
        );
        assert_eq!(interpretation.trust, super::TempoTrustLevel::Tentative);
        assert!(interpretation.profile.boundary_edge_gap_ms > 2.5);
        assert!(interpretation.profile.stability_score.0 < 0.7);
    }

    #[test]
    fn tempo_interpretation_snaps_stable_near_integer_master_like_case() {
        let diagnostics = synthetic_tempo_diagnostics_with_counts(
            127.94273, 0.064, -0.1097, 0.48279, 45.998, 45.774, 83.272, 86.989, 738, 735, 739,
        );
        let interpretation = super::interpret_tempo(
            127.97321,
            super::Confidence::new(0.666),
            super::Confidence::new(1.0),
            &diagnostics,
        );

        assert_eq!(
            interpretation.recommendation,
            super::TempoRecommendation::SnapInteger
        );
        assert_eq!(
            interpretation.reason,
            super::TempoInterpretationReason::NearIntegerPulse
        );
        assert_eq!(interpretation.snapped_bpm, Some(128.0));
        assert!(interpretation.support.integer_closeness.0 > 0.9);
        assert!(interpretation.support.core_consensus.0 > 0.85);
        assert!(interpretation.support.drift_stability.0 > 0.55);
        assert!(interpretation.support.grid_stability.0 > 0.35);
        assert!(interpretation.support.boundary_pressure.0 < 0.3);
        assert!(interpretation.profile.stability_score.0 > 0.64);

        let state = super::tempo_state_recommendation(
            interpretation,
            super::Confidence::new(0.666),
            super::Confidence::new(1.0),
        );
        assert_eq!(state.action, super::TempoStateAction::Lock);
        assert_eq!(state.reason, super::TempoStateReason::StableIntegerTempo);
    }

    #[test]
    fn tempo_interpretation_localizes_boundary_pressure_for_long_form_stable_tracks() {
        let short_form = synthetic_tempo_diagnostics(
            127.94273, 0.064, -0.1097, 0.48279, 45.998, 45.774, 83.272, 86.989,
        );
        let long_form = synthetic_tempo_diagnostics_with_counts(
            127.94273, 0.064, -0.1097, 0.48279, 45.998, 45.774, 83.272, 86.989, 738, 735, 739,
        );

        let short_interpretation = super::interpret_tempo(
            127.97321,
            super::Confidence::new(0.666),
            super::Confidence::new(1.0),
            &short_form,
        );
        let long_interpretation = super::interpret_tempo(
            127.97321,
            super::Confidence::new(0.666),
            super::Confidence::new(1.0),
            &long_form,
        );

        assert!(
            short_interpretation.support.boundary_pressure.0
                > long_interpretation.support.boundary_pressure.0,
            "short={} long={}",
            short_interpretation.support.boundary_pressure.0,
            long_interpretation.support.boundary_pressure.0
        );
        assert_eq!(
            long_interpretation.recommendation,
            super::TempoRecommendation::SnapInteger
        );
        assert!(
            long_interpretation.support.boundary_pressure.0 < 0.3,
            "long boundary pressure should be localized: {}",
            long_interpretation.support.boundary_pressure.0
        );
    }

    #[test]
    fn tempo_interpretation_snaps_stable_near_integer_with_localized_tail_outliers() {
        let mut diagnostics = synthetic_tempo_diagnostics_with_counts(
            127.94273, 0.064, -0.1097, 0.48279, 45.998, 45.774, 83.272, 86.989, 738, 735, 739,
        );
        diagnostics.beat_interval_outliers = super::BeatIntervalOutlierDiagnostics {
            total_intervals: 738,
            retained_intervals: 670,
            rejected_intervals: 68,
            leading_rejected_intervals: 0,
            trailing_rejected_intervals: 3,
            median_interval: 60.0 / 127.94273,
            median_abs_deviation: 0.000607,
            max_rejected_deviation_ratio: 0.384,
        };

        let interpretation = super::interpret_tempo(
            127.96191,
            super::Confidence::new(0.666),
            super::Confidence::new(1.0),
            &diagnostics,
        );

        assert_eq!(
            interpretation.recommendation,
            super::TempoRecommendation::SnapInteger
        );
        assert_eq!(interpretation.snapped_bpm, Some(128.0));
    }

    #[test]
    fn beat_interval_outlier_filter_localizes_terminal_outliers() {
        let stable = 60.0 / 128.0;
        let intervals = vec![
            stable,
            stable,
            stable,
            stable,
            stable,
            stable,
            stable,
            stable,
            stable,
            stable,
            stable,
            stable,
            stable * 1.23,
            stable * 0.84,
            stable * 1.32,
            stable,
        ];
        let (retained, diagnostics) = super::filter_interval_outliers(&intervals);

        assert_eq!(diagnostics.total_intervals, intervals.len());
        assert_eq!(diagnostics.trailing_rejected_intervals, 3);
        assert_eq!(diagnostics.rejected_intervals, 3);
        assert_eq!(diagnostics.leading_rejected_intervals, 0);
        assert_eq!(diagnostics.retained_intervals, retained.len());
        assert!(diagnostics.max_rejected_deviation_ratio > 0.2);
        assert!((diagnostics.median_interval - stable).abs() < 1.0e-6);
    }

    #[test]
    fn stable_core_span_detects_terminal_window_damage() {
        let stable = 127.94;
        let points: Vec<super::LocalTempoPoint> = (0..12)
            .map(|index| super::LocalTempoPoint {
                start_beat_index: index,
                end_beat_index: index + 4,
                start_seconds: index as f32,
                end_seconds: index as f32 + 4.0,
                bpm: match index {
                    9 => 129.10,
                    10 => 124.40,
                    11 => 116.60,
                    _ => stable,
                },
            })
            .collect();

        let span = super::detect_stable_core_span(&points, stable, 0.12).unwrap();

        assert_eq!(span.start_beat_index, 0);
        assert_eq!(span.end_beat_index, 12);
        assert!(span.coverage.0 >= 0.8, "coverage {}", span.coverage.0);
        assert_eq!(span.trimmed_leading_windows, 0);
        assert_eq!(span.trimmed_trailing_windows, 3);
        assert_eq!(span.interior_rejected_windows, 0);
    }

    #[test]
    fn edge_trimmed_stable_span_preserves_sparse_interior_instability() {
        let stable = 127.94;
        let points: Vec<super::LocalTempoPoint> = (0..16)
            .map(|index| super::LocalTempoPoint {
                start_beat_index: index,
                end_beat_index: index + 4,
                start_seconds: index as f32,
                end_seconds: index as f32 + 4.0,
                bpm: match index {
                    3 => 130.25,
                    8 => 125.70,
                    13 => 129.40,
                    14 => 124.40,
                    15 => 116.60,
                    _ => stable,
                },
            })
            .collect();

        let edge_trimmed = super::detect_edge_trimmed_stable_span(&points, stable, 0.12).unwrap();
        let contiguous = super::detect_stable_core_span(&points, stable, 0.12).unwrap();

        assert_eq!(edge_trimmed.start_beat_index, 0);
        assert!(edge_trimmed.end_beat_index >= 16);
        assert_eq!(edge_trimmed.trimmed_leading_windows, 0);
        assert!(edge_trimmed.retained_windows >= contiguous.retained_windows);
        assert!(contiguous.trimmed_leading_windows > 0 || contiguous.trimmed_trailing_windows > 0);
    }

    #[test]
    fn beat_tracker_exposes_stable_core_span_for_integer_click_track() {
        let tracker = &mut super::BeatTracker::new(super::BeatTrackerConfig::default());
        let result = tracker.analyze(&click_track(48_000, 120.0, 8.0));
        let edge_trimmed = result
            .tempo_diagnostics
            .edge_trimmed_stable_span
            .expect("edge-trimmed stable span");
        let span = result
            .tempo_diagnostics
            .stable_core_span
            .expect("stable core span");

        assert_eq!(edge_trimmed.start_beat_index, 0);
        assert!(
            edge_trimmed.coverage.0 > 0.95,
            "coverage {}",
            edge_trimmed.coverage.0
        );
        assert_eq!(edge_trimmed.interior_rejected_windows, 0);
        assert_eq!(span.start_beat_index, 0);
        assert!(span.end_beat_index >= result.beat_positions_seconds.len().saturating_sub(2));
        assert!(span.coverage.0 > 0.9, "coverage {}", span.coverage.0);
        assert_eq!(span.interior_rejected_windows, 0);
    }

    #[test]
    fn beat_tracker_classifies_whole_track_stable_scope_for_click_track() {
        let tracker = &mut super::BeatTracker::new(super::BeatTrackerConfig::default());
        let result = tracker.analyze(&click_track(48_000, 120.0, 8.0));

        assert_eq!(
            result.tempo_diagnostics.stability_scope.scope,
            super::TempoStabilityScope::WholeTrackStable
        );
        assert!(
            result
                .tempo_consumption(None)
                .stability_scope
                .support
                .edge_trimmed_coverage
                .0
                > 0.95
        );
    }

    #[test]
    fn classify_tempo_stability_scope_detects_localized_edge_damage() {
        let mut diagnostics = synthetic_tempo_diagnostics_with_counts(
            128.0, 0.70, -0.11, 0.48, 46.0, 45.8, 278.0, 87.0, 738, 735, 739,
        );
        diagnostics.beat_interval_outliers = super::BeatIntervalOutlierDiagnostics {
            total_intervals: 738,
            retained_intervals: 670,
            rejected_intervals: 68,
            leading_rejected_intervals: 0,
            trailing_rejected_intervals: 3,
            median_interval: 0.468_956,
            median_abs_deviation: 0.000_607,
            max_rejected_deviation_ratio: 0.384,
        };
        diagnostics.edge_trimmed_stable_span = Some(super::BeatGridCoreSpanDiagnostics {
            start_beat_index: 0,
            end_beat_index: 735,
            start_seconds: 0.447,
            end_seconds: 345.333,
            coverage: super::Confidence::new(0.996),
            retained_windows: 732,
            total_windows: 735,
            trimmed_leading_windows: 0,
            trimmed_trailing_windows: 3,
            interior_rejected_windows: 14,
        });
        diagnostics.stable_core_span = Some(super::BeatGridCoreSpanDiagnostics {
            start_beat_index: 216,
            end_beat_index: 706,
            start_seconds: 101.698,
            end_seconds: 331.641,
            coverage: super::Confidence::new(0.664),
            retained_windows: 487,
            total_windows: 735,
            trimmed_leading_windows: 216,
            trimmed_trailing_windows: 32,
            interior_rejected_windows: 0,
        });
        diagnostics.stability_scope = super::classify_tempo_stability_scope(
            diagnostics.windowed_tempi.len(),
            &diagnostics.beat_interval_outliers,
            diagnostics.edge_trimmed_stable_span,
            diagnostics.stable_core_span,
        );

        assert_eq!(
            diagnostics.stability_scope.scope,
            super::TempoStabilityScope::StableWithLocalizedEdgeDamage
        );
        assert!(diagnostics.stability_scope.support.edge_locality.0 >= 0.55);
    }

    #[test]
    fn classify_tempo_stability_scope_detects_core_stable_only_case() {
        let mut diagnostics = synthetic_tempo_diagnostics_with_counts(
            120.0, 0.85, 0.42, 0.61, 58.0, 44.0, 360.0, 92.0, 128, 96, 128,
        );
        diagnostics.beat_interval_outliers = super::BeatIntervalOutlierDiagnostics {
            total_intervals: 128,
            retained_intervals: 120,
            rejected_intervals: 8,
            leading_rejected_intervals: 0,
            trailing_rejected_intervals: 0,
            median_interval: 0.5,
            median_abs_deviation: 0.004,
            max_rejected_deviation_ratio: 0.18,
        };
        diagnostics.edge_trimmed_stable_span = Some(super::BeatGridCoreSpanDiagnostics {
            start_beat_index: 24,
            end_beat_index: 92,
            start_seconds: 12.0,
            end_seconds: 46.0,
            coverage: super::Confidence::new(0.57),
            retained_windows: 69,
            total_windows: 96,
            trimmed_leading_windows: 24,
            trimmed_trailing_windows: 3,
            interior_rejected_windows: 6,
        });
        diagnostics.stable_core_span = Some(super::BeatGridCoreSpanDiagnostics {
            start_beat_index: 28,
            end_beat_index: 88,
            start_seconds: 14.0,
            end_seconds: 44.0,
            coverage: super::Confidence::new(0.50),
            retained_windows: 61,
            total_windows: 96,
            trimmed_leading_windows: 28,
            trimmed_trailing_windows: 7,
            interior_rejected_windows: 0,
        });
        diagnostics.stability_scope = super::classify_tempo_stability_scope(
            diagnostics.windowed_tempi.len(),
            &diagnostics.beat_interval_outliers,
            diagnostics.edge_trimmed_stable_span,
            diagnostics.stable_core_span,
        );

        assert_eq!(
            diagnostics.stability_scope.scope,
            super::TempoStabilityScope::CoreStableOnly
        );
        assert!(
            diagnostics
                .stability_scope
                .support
                .contiguous_core_coverage
                .0
                >= 0.5
        );
    }

    #[test]
    fn refine_bpm_from_beats_ignores_terminal_outlier_intervals() {
        let stable_interval_frames = 46.875;
        let intervals = [
            stable_interval_frames,
            stable_interval_frames,
            stable_interval_frames,
            stable_interval_frames,
            stable_interval_frames,
            stable_interval_frames,
            stable_interval_frames,
            stable_interval_frames,
            stable_interval_frames,
            stable_interval_frames,
            stable_interval_frames,
            stable_interval_frames,
            stable_interval_frames * 1.23,
            stable_interval_frames * 0.84,
            stable_interval_frames * 1.32,
            stable_interval_frames,
        ];
        let mut beat_frames = Vec::with_capacity(intervals.len() + 1);
        let mut current = 0.0;
        beat_frames.push(current);
        for interval in intervals {
            current += interval;
            beat_frames.push(current);
        }

        let refined =
            super::refine_bpm_from_beats(127.97321, &beat_frames, SampleRate(48_000), 512);

        assert!((refined - 128.0).abs() < 0.05, "refined bpm {}", refined);
    }

    #[test]
    fn tempo_state_locks_stable_integer_interpretation() {
        let interpretation = synthetic_tempo_interpretation(
            super::TempoRecommendation::SnapInteger,
            super::TempoTrustLevel::Stable,
            super::TempoInterpretationReason::NearIntegerPulse,
            120.0,
            Some(120.0),
            0.86,
            0.08,
            0.22,
            0.82,
        );
        let state = super::tempo_state_recommendation(
            interpretation,
            super::Confidence::new(0.9),
            super::Confidence::new(0.12),
        );

        assert_eq!(state.action, super::TempoStateAction::Lock);
        assert_eq!(state.reason, super::TempoStateReason::StableIntegerTempo);
        assert!(state.confidence.0 >= 0.82);
        assert_eq!(state.continuity.action, super::TempoContinuityAction::Lock);
        assert_eq!(
            state.continuity.reason,
            super::TempoContinuityReason::IntegerTempoSnap
        );
        assert_eq!(
            state.continuity.provenance,
            super::TempoContinuityProvenance::IntegerSnap
        );
        assert_eq!(
            state.continuity.severity,
            super::TempoContinuitySeverity::Confirmed
        );
        assert_eq!(
            state.continuity.history,
            super::TempoContinuityHistory::Reinforcing
        );
        assert_eq!(state.continuity.arc, super::TempoContinuityArc::Recovering);
        assert_eq!(
            state.continuity.arc_rationale,
            super::TempoContinuityArcRationale::RefreshStrength
        );
        assert_eq!(
            state.continuity.arc_decision.recommendation,
            super::TempoContinuityArcRecommendation::KeepLock
        );
        assert_eq!(
            state.continuity.arc_decision.action,
            super::TempoContinuityArcAction::LockCurrentTempo
        );
        assert_eq!(
            state.continuity.arc_decision.severity,
            super::TempoContinuitySeverity::Confirmed
        );
        assert_eq!(
            state.continuity.arc_decision.fallback_action,
            super::TempoContinuityArcAction::ReacquireCurrentTempo
        );
        assert_eq!(
            state.continuity.arc_decision.downgrade_rationale,
            super::TempoContinuityArcDowngradeRationale::StabilityWindowEnd
        );
        assert_eq!(
            state.continuity.arc_decision.downgrade_trend,
            super::TempoContinuityArcDowngradeTrend::Easing
        );
        assert_eq!(
            state.continuity.arc_decision.downgrade_trend_rationale,
            super::TempoContinuityArcDowngradeTrendRationale::StabilityWindowCarry
        );
        assert_eq!(
            state.continuity.arc_decision.downgrade_inflection.stage,
            super::TempoContinuityArcDowngradeInflectionStage::NextStage
        );
        assert_eq!(
            state
                .continuity
                .arc_decision
                .downgrade_inflection
                .after_beats,
            12
        );
        assert_eq!(
            state
                .continuity
                .arc_decision
                .downgrade_inflection
                .competing_stage,
            Some(super::TempoContinuityArcDowngradeInflectionStage::TerminalClear)
        );
        assert!(
            state
                .continuity
                .arc_decision
                .downgrade_inflection
                .competing_after_beats
                > state
                    .continuity
                    .arc_decision
                    .downgrade_inflection
                    .after_beats
        );
        assert!(
            state
                .continuity
                .arc_decision
                .downgrade_inflection
                .competing_support
                .0
                >= 0.55
        );
        assert!(
            state
                .continuity
                .arc_decision
                .downgrade_inflection
                .balance
                .competing_weight
                .0
                >= 0.0
        );
        assert!(
            state
                .continuity
                .arc_decision
                .downgrade_inflection
                .balance
                .dominance
                .0
                >= 0.0
        );
        assert_eq!(
            state
                .continuity
                .arc_decision
                .downgrade_inflection
                .rationale_balance
                .primary
                .dominant,
            super::TempoContinuityArcDowngradeStageRationale::StabilityWindow
        );
        assert!(matches!(
            state
                .continuity
                .arc_decision
                .downgrade_inflection
                .rationale_balance
                .competing
                .map(|weights| weights.dominant),
            Some(super::TempoContinuityArcDowngradeStageRationale::EvidenceLoss)
                | Some(super::TempoContinuityArcDowngradeStageRationale::StabilityWindow)
                | None
        ));
        assert!(
            state
                .continuity
                .arc_decision
                .downgrade_trend_support
                .terminal_pressure
                .0
                > state
                    .continuity
                    .arc_decision
                    .downgrade_trend_support
                    .current_pressure
                    .0
        );
        assert!(
            state
                .continuity
                .arc_decision
                .downgrade_support
                .stability_window_pressure
                .0
                > state
                    .continuity
                    .arc_decision
                    .downgrade_support
                    .boundary_drift_pressure
                    .0
        );
        assert_eq!(
            state.continuity.arc_decision.provenance,
            super::TempoContinuityProvenance::IntegerSnap
        );
        assert_eq!(
            state.continuity.arc_decision.expiry,
            super::TempoContinuityArcActionExpiry {
                guaranteed_until_beats: 16,
                fallback_after_beats: 20,
                clear_after_beats: 28,
                max_failed_revalidations: 3,
            }
        );
        assert_eq!(
            state.continuity.trigger,
            super::TempoContinuityTrigger::StableRevalidation
        );
        assert_eq!(
            state.continuity.unresolved,
            super::TempoContinuityUnresolvedSpan {
                beats: 0,
                failed_revalidations: 0,
            }
        );
        assert_eq!(
            state.continuity.causes.primary,
            super::TempoContinuityCause::StableTempoEvidence
        );
        assert_eq!(state.continuity.expiry.guaranteed_until_beats, 16);
        assert_eq!(state.continuity.expiry.max_failed_revalidations, 3);
        assert!(state.continuity.refresh_strength.0 > 0.9);
        assert_eq!(
            state.continuity.lifecycle.decay[1].action,
            super::TempoContinuityAction::Clear
        );
        assert_eq!(
            state.continuity.lifecycle.decay[1].provenance,
            super::TempoContinuityProvenance::NoTempo
        );
        assert_eq!(
            state.continuity.lifecycle.decay[1].severity,
            super::TempoContinuitySeverity::Cleared
        );
        assert_eq!(
            state.continuity.lifecycle.decay[1].history,
            super::TempoContinuityHistory::Degrading
        );
    }

    #[test]
    fn tempo_state_locks_edge_damaged_integer_scope() {
        let interpretation = synthetic_tempo_interpretation(
            super::TempoRecommendation::SnapInteger,
            super::TempoTrustLevel::Stable,
            super::TempoInterpretationReason::NearIntegerPulse,
            128.0,
            Some(128.0),
            0.80,
            0.08,
            0.22,
            0.45,
        );
        let state = super::tempo_state_recommendation_with_scope(
            interpretation,
            super::Confidence::new(0.666),
            super::Confidence::new(0.18),
            scope_summary(super::TempoStabilityScope::StableWithLocalizedEdgeDamage),
        );

        assert_eq!(state.action, super::TempoStateAction::Lock);
        assert_eq!(
            state.reason,
            super::TempoStateReason::StableTempoWithEdgeDamage
        );
        assert!(state.confidence.0 >= 0.76);
        assert_eq!(state.continuity.action, super::TempoContinuityAction::Lock);
        assert_eq!(
            state.continuity.source,
            super::TempoContinuitySource::CurrentTempo
        );
        assert_eq!(state.continuity.expiry.guaranteed_until_beats, 10);
        assert_eq!(state.continuity.expiry.downgrade_after_beats, 12);
        assert_eq!(state.continuity.expiry.clear_after_beats, 18);
        assert_eq!(
            state.continuity.arc_decision.expiry.fallback_after_beats,
            12
        );
    }

    #[test]
    fn tempo_state_monitors_core_stable_integer_scope() {
        let interpretation = synthetic_tempo_interpretation(
            super::TempoRecommendation::SnapInteger,
            super::TempoTrustLevel::Stable,
            super::TempoInterpretationReason::NearIntegerPulse,
            128.0,
            Some(128.0),
            0.79,
            0.042,
            0.20,
            0.44,
        );
        let state = super::tempo_state_recommendation_with_scope(
            interpretation,
            super::Confidence::new(0.72),
            super::Confidence::new(0.16),
            scope_summary(super::TempoStabilityScope::CoreStableOnly),
        );

        assert_eq!(state.action, super::TempoStateAction::Monitor);
        assert_eq!(state.reason, super::TempoStateReason::CoreStableTempo);
        assert_eq!(
            state.continuity.action,
            super::TempoContinuityAction::Reacquire
        );
        assert_eq!(
            state.continuity.source,
            super::TempoContinuitySource::CurrentTempo
        );
        assert_eq!(
            state.continuity.lifecycle.refresh.action,
            super::TempoContinuityAction::Lock
        );
        assert_eq!(state.continuity.expiry.guaranteed_until_beats, 4);
        assert_eq!(state.continuity.expiry.clear_after_beats, 12);
    }

    #[test]
    fn tempo_state_monitors_core_window_fallback() {
        let interpretation = synthetic_tempo_interpretation(
            super::TempoRecommendation::UseCoreWindow,
            super::TempoTrustLevel::Guarded,
            super::TempoInterpretationReason::StableCoreWindow,
            90.0,
            None,
            0.64,
            0.07,
            0.72,
            0.64,
        );
        let state = super::tempo_state_recommendation(
            interpretation,
            super::Confidence::new(0.72),
            super::Confidence::new(0.18),
        );

        assert_eq!(state.action, super::TempoStateAction::Monitor);
        assert_eq!(state.reason, super::TempoStateReason::CoreWindowFallback);
        assert!(state.confidence.0 >= 0.58);
        assert_eq!(
            state.continuity.action,
            super::TempoContinuityAction::Retain
        );
        assert_eq!(
            state.continuity.source,
            super::TempoContinuitySource::CoreWindow
        );
        assert_eq!(
            state.continuity.provenance,
            super::TempoContinuityProvenance::CoreWindowEstimate
        );
        assert_eq!(
            state.continuity.severity,
            super::TempoContinuitySeverity::Guarded
        );
        assert_eq!(
            state.continuity.history,
            super::TempoContinuityHistory::Preserving
        );
        assert_eq!(state.continuity.arc, super::TempoContinuityArc::Stalling);
        assert_eq!(
            state.continuity.arc_rationale,
            super::TempoContinuityArcRationale::BoundaryDrift
        );
        assert_eq!(
            state.continuity.arc_decision.recommendation,
            super::TempoContinuityArcRecommendation::MonitorRecovery
        );
        assert_eq!(
            state.continuity.arc_decision.action,
            super::TempoContinuityArcAction::PreferCoreWindowTempo
        );
        assert_eq!(
            state.continuity.arc_decision.severity,
            super::TempoContinuitySeverity::Guarded
        );
        assert_eq!(
            state.continuity.arc_decision.fallback_action,
            super::TempoContinuityArcAction::PreservePriorTempo
        );
        assert_eq!(
            state.continuity.arc_decision.downgrade_rationale,
            super::TempoContinuityArcDowngradeRationale::BoundaryDrift
        );
        assert_eq!(
            state.continuity.arc_decision.downgrade_trend,
            super::TempoContinuityArcDowngradeTrend::Rising
        );
        assert_eq!(
            state.continuity.arc_decision.downgrade_trend_rationale,
            super::TempoContinuityArcDowngradeTrendRationale::BoundaryEscalation
        );
        assert_eq!(
            state.continuity.arc_decision.downgrade_inflection.stage,
            super::TempoContinuityArcDowngradeInflectionStage::NextStage
        );
        assert_eq!(
            state
                .continuity
                .arc_decision
                .downgrade_inflection
                .after_beats,
            8
        );
        assert_eq!(
            state
                .continuity
                .arc_decision
                .downgrade_inflection
                .competing_stage,
            Some(super::TempoContinuityArcDowngradeInflectionStage::TerminalClear)
        );
        assert!(
            state
                .continuity
                .arc_decision
                .downgrade_inflection
                .competing_after_beats
                > state
                    .continuity
                    .arc_decision
                    .downgrade_inflection
                    .after_beats
        );
        assert!(
            state
                .continuity
                .arc_decision
                .downgrade_inflection
                .competing_support
                .0
                >= 0.55
        );
        assert!(
            state
                .continuity
                .arc_decision
                .downgrade_inflection
                .balance
                .competing_weight
                .0
                >= 0.0
        );
        assert!(
            state
                .continuity
                .arc_decision
                .downgrade_inflection
                .balance
                .dominance
                .0
                >= 0.0
        );
        assert_eq!(
            state
                .continuity
                .arc_decision
                .downgrade_inflection
                .rationale_balance
                .primary
                .dominant,
            super::TempoContinuityArcDowngradeStageRationale::BoundaryDrift
        );
        assert!(matches!(
            state
                .continuity
                .arc_decision
                .downgrade_inflection
                .rationale_balance
                .competing
                .map(|weights| weights.dominant),
            Some(super::TempoContinuityArcDowngradeStageRationale::EvidenceLoss)
                | Some(super::TempoContinuityArcDowngradeStageRationale::BoundaryDrift)
                | Some(super::TempoContinuityArcDowngradeStageRationale::StabilityWindow)
                | None
        ));
        assert!(
            state
                .continuity
                .arc_decision
                .downgrade_trend_support
                .next_stage_pressure
                .0
                > state
                    .continuity
                    .arc_decision
                    .downgrade_trend_support
                    .current_pressure
                    .0
        );
        assert!(
            state
                .continuity
                .arc_decision
                .downgrade_support
                .boundary_drift_pressure
                .0
                > state
                    .continuity
                    .arc_decision
                    .downgrade_support
                    .ambiguity_pressure
                    .0
        );
        assert_eq!(
            state.continuity.arc_decision.provenance,
            super::TempoContinuityProvenance::CoreWindowEstimate
        );
        assert_eq!(
            state.continuity.arc_decision.expiry,
            super::TempoContinuityArcActionExpiry {
                guaranteed_until_beats: 8,
                fallback_after_beats: 8,
                clear_after_beats: 12,
                max_failed_revalidations: 2,
            }
        );
        assert_eq!(
            state.continuity.trigger,
            super::TempoContinuityTrigger::BoundaryDrift
        );
        assert_eq!(
            state.continuity.unresolved,
            super::TempoContinuityUnresolvedSpan {
                beats: 8,
                failed_revalidations: 2,
            }
        );
        assert_eq!(
            state.continuity.causes.primary,
            super::TempoContinuityCause::BoundaryDrift
        );
        assert_eq!(state.continuity.expiry.guaranteed_until_beats, 8);
        assert_eq!(state.continuity.expiry.max_failed_revalidations, 3);
        assert_eq!(
            state.continuity.lifecycle.decay[0].action,
            super::TempoContinuityAction::Reacquire
        );
        assert_eq!(
            state.continuity.lifecycle.decay[0].provenance,
            super::TempoContinuityProvenance::PriorTempoCarry
        );
        assert_eq!(
            state.continuity.lifecycle.decay[0].severity,
            super::TempoContinuitySeverity::Fragile
        );
        assert_eq!(
            state.continuity.lifecycle.decay[0].history,
            super::TempoContinuityHistory::Degrading
        );
        assert_eq!(
            state.continuity.lifecycle.decay[0].trigger,
            super::TempoContinuityTrigger::PriorTempoDrift
        );
    }

    #[test]
    fn tempo_state_reacquires_guarded_refined_estimate_before_clearing() {
        let interpretation = synthetic_tempo_interpretation(
            super::TempoRecommendation::UseRefined,
            super::TempoTrustLevel::Guarded,
            super::TempoInterpretationReason::StableRefinedPulse,
            117.8,
            None,
            0.61,
            0.09,
            0.32,
            0.62,
        );
        let state = super::tempo_state_recommendation(
            interpretation,
            super::Confidence::new(0.71),
            super::Confidence::new(0.21),
        );

        assert_eq!(state.action, super::TempoStateAction::Monitor);
        assert_eq!(state.reason, super::TempoStateReason::StableRefinedTempo);
        assert_eq!(
            state.continuity.action,
            super::TempoContinuityAction::Reacquire
        );
        assert_eq!(
            state.continuity.source,
            super::TempoContinuitySource::CurrentTempo
        );
        assert_eq!(
            state.continuity.provenance,
            super::TempoContinuityProvenance::GuardedRefinedEstimate
        );
        assert_eq!(
            state.continuity.severity,
            super::TempoContinuitySeverity::Fragile
        );
        assert_eq!(
            state.continuity.history,
            super::TempoContinuityHistory::Preserving
        );
        assert_eq!(state.continuity.arc, super::TempoContinuityArc::Recovering);
        assert_eq!(
            state.continuity.arc_rationale,
            super::TempoContinuityArcRationale::RefreshStrength
        );
        assert_eq!(
            state.continuity.arc_decision.recommendation,
            super::TempoContinuityArcRecommendation::MonitorRecovery
        );
        assert_eq!(
            state.continuity.arc_decision.action,
            super::TempoContinuityArcAction::ReacquireCurrentTempo
        );
        assert_eq!(
            state.continuity.arc_decision.severity,
            super::TempoContinuitySeverity::Fragile
        );
        assert_eq!(
            state.continuity.arc_decision.fallback_action,
            super::TempoContinuityArcAction::ClearTempo
        );
        assert_eq!(
            state.continuity.arc_decision.downgrade_rationale,
            super::TempoContinuityArcDowngradeRationale::AmbiguityCarry
        );
        assert_eq!(
            state.continuity.arc_decision.downgrade_trend,
            super::TempoContinuityArcDowngradeTrend::Easing
        );
        assert_eq!(
            state.continuity.arc_decision.downgrade_trend_rationale,
            super::TempoContinuityArcDowngradeTrendRationale::AmbiguityCarry
        );
        assert_eq!(
            state.continuity.arc_decision.downgrade_inflection.stage,
            super::TempoContinuityArcDowngradeInflectionStage::NextStage
        );
        assert_eq!(
            state
                .continuity
                .arc_decision
                .downgrade_inflection
                .after_beats,
            4
        );
        assert_eq!(
            state
                .continuity
                .arc_decision
                .downgrade_inflection
                .competing_stage,
            Some(super::TempoContinuityArcDowngradeInflectionStage::TerminalClear)
        );
        assert!(
            state
                .continuity
                .arc_decision
                .downgrade_inflection
                .competing_after_beats
                > state
                    .continuity
                    .arc_decision
                    .downgrade_inflection
                    .after_beats
        );
        assert!(
            state
                .continuity
                .arc_decision
                .downgrade_inflection
                .competing_support
                .0
                >= 0.55
        );
        assert!(
            state
                .continuity
                .arc_decision
                .downgrade_inflection
                .balance
                .competing_weight
                .0
                >= 0.0
        );
        assert!(
            state
                .continuity
                .arc_decision
                .downgrade_inflection
                .balance
                .dominance
                .0
                >= 0.0
        );
        assert_eq!(
            state
                .continuity
                .arc_decision
                .downgrade_inflection
                .rationale_balance
                .primary
                .dominant,
            super::TempoContinuityArcDowngradeStageRationale::AmbiguityCarry
        );
        assert!(matches!(
            state
                .continuity
                .arc_decision
                .downgrade_inflection
                .rationale_balance
                .competing
                .map(|weights| weights.dominant),
            Some(super::TempoContinuityArcDowngradeStageRationale::EvidenceLoss)
                | Some(super::TempoContinuityArcDowngradeStageRationale::AmbiguityCarry)
                | Some(super::TempoContinuityArcDowngradeStageRationale::StabilityWindow)
                | None
        ));
        assert!(
            state
                .continuity
                .arc_decision
                .downgrade_trend_support
                .terminal_pressure
                .0
                > state
                    .continuity
                    .arc_decision
                    .downgrade_trend_support
                    .current_pressure
                    .0
        );
        assert!(
            state
                .continuity
                .arc_decision
                .downgrade_support
                .ambiguity_pressure
                .0
                > state
                    .continuity
                    .arc_decision
                    .downgrade_support
                    .boundary_drift_pressure
                    .0
        );
        assert_eq!(
            state.continuity.arc_decision.provenance,
            super::TempoContinuityProvenance::GuardedRefinedEstimate
        );
        assert_eq!(
            state.continuity.arc_decision.expiry,
            super::TempoContinuityArcActionExpiry {
                guaranteed_until_beats: 4,
                fallback_after_beats: 12,
                clear_after_beats: 12,
                max_failed_revalidations: 3,
            }
        );
        assert_eq!(
            state.continuity.trigger,
            super::TempoContinuityTrigger::AmbiguityCarry
        );
        assert_eq!(
            state.continuity.unresolved,
            super::TempoContinuityUnresolvedSpan {
                beats: 4,
                failed_revalidations: 1,
            }
        );
        assert_eq!(
            state.continuity.causes.primary,
            super::TempoContinuityCause::TempoAmbiguity
        );
        assert_eq!(state.continuity.expiry.guaranteed_until_beats, 4);
        assert_eq!(state.continuity.expiry.downgrade_after_beats, 8);
        assert_eq!(state.continuity.expiry.clear_after_beats, 12);
        assert_eq!(state.continuity.expiry.max_failed_revalidations, 3);
        assert_eq!(
            state.continuity.lifecycle.refresh.provenance,
            super::TempoContinuityProvenance::StableRefinedEstimate
        );
        assert_eq!(
            state.continuity.lifecycle.refresh.severity,
            super::TempoContinuitySeverity::Confirmed
        );
        assert_eq!(
            state.continuity.lifecycle.refresh.history,
            super::TempoContinuityHistory::Reinforcing
        );
        assert_eq!(
            state.continuity.lifecycle.refresh.trigger,
            super::TempoContinuityTrigger::StableRevalidation
        );
        assert_eq!(
            state.continuity.lifecycle.decay[0].provenance,
            super::TempoContinuityProvenance::GuardedRefinedEstimate
        );
        assert!(
            state.continuity.lifecycle.refresh.refresh_strength.0
                > state.continuity.refresh_strength.0
        );
    }

    #[test]
    fn tempo_state_defers_unstable_interpretation() {
        let interpretation = synthetic_tempo_interpretation(
            super::TempoRecommendation::Defer,
            super::TempoTrustLevel::Tentative,
            super::TempoInterpretationReason::UnstableTempo,
            89.9,
            None,
            0.38,
            0.03,
            0.8,
            0.3,
        );
        let state = super::tempo_state_recommendation(
            interpretation,
            super::Confidence::new(0.42),
            super::Confidence::new(0.55),
        );

        assert_eq!(state.action, super::TempoStateAction::Defer);
        assert_eq!(state.reason, super::TempoStateReason::TempoDeferred);
        assert!(state.confidence.0 > 0.4);
        assert_eq!(state.continuity.action, super::TempoContinuityAction::Clear);
        assert_eq!(
            state.continuity.provenance,
            super::TempoContinuityProvenance::NoTempo
        );
        assert_eq!(
            state.continuity.severity,
            super::TempoContinuitySeverity::Cleared
        );
        assert_eq!(
            state.continuity.history,
            super::TempoContinuityHistory::Degrading
        );
        assert_eq!(state.continuity.arc, super::TempoContinuityArc::Collapsing);
        assert_eq!(
            state.continuity.arc_rationale,
            super::TempoContinuityArcRationale::EvidenceLoss
        );
        assert_eq!(
            state.continuity.arc_decision.recommendation,
            super::TempoContinuityArcRecommendation::Clear
        );
        assert_eq!(
            state.continuity.arc_decision.action,
            super::TempoContinuityArcAction::ClearTempo
        );
        assert_eq!(
            state.continuity.arc_decision.severity,
            super::TempoContinuitySeverity::Cleared
        );
        assert_eq!(
            state.continuity.arc_decision.fallback_action,
            super::TempoContinuityArcAction::ClearTempo
        );
        assert_eq!(
            state.continuity.arc_decision.downgrade_rationale,
            super::TempoContinuityArcDowngradeRationale::EvidenceLoss
        );
        assert_eq!(
            state.continuity.arc_decision.downgrade_trend,
            super::TempoContinuityArcDowngradeTrend::Stable
        );
        assert_eq!(
            state.continuity.arc_decision.downgrade_trend_rationale,
            super::TempoContinuityArcDowngradeTrendRationale::FlatCollapse
        );
        assert_eq!(
            state.continuity.arc_decision.downgrade_inflection.stage,
            super::TempoContinuityArcDowngradeInflectionStage::FlatWindow
        );
        assert_eq!(
            state
                .continuity
                .arc_decision
                .downgrade_inflection
                .after_beats,
            0
        );
        assert_eq!(
            state
                .continuity
                .arc_decision
                .downgrade_inflection
                .competing_stage,
            None
        );
        assert_eq!(
            state
                .continuity
                .arc_decision
                .downgrade_inflection
                .competing_support,
            super::Confidence::new(0.0)
        );
        assert_eq!(
            state
                .continuity
                .arc_decision
                .downgrade_inflection
                .balance
                .primary_weight,
            super::Confidence::new(0.0)
        );
        assert_eq!(
            state
                .continuity
                .arc_decision
                .downgrade_inflection
                .balance
                .competing_weight,
            super::Confidence::new(0.0)
        );
        assert_eq!(
            state
                .continuity
                .arc_decision
                .downgrade_inflection
                .balance
                .unattributed_weight,
            super::Confidence::new(1.0)
        );
        assert_eq!(
            state
                .continuity
                .arc_decision
                .downgrade_inflection
                .balance
                .dominance,
            super::Confidence::new(0.0)
        );
        assert_eq!(
            state
                .continuity
                .arc_decision
                .downgrade_inflection
                .rationale_balance
                .primary
                .dominant,
            super::TempoContinuityArcDowngradeStageRationale::EvidenceLoss
        );
        assert_eq!(
            state
                .continuity
                .arc_decision
                .downgrade_inflection
                .rationale_balance
                .competing,
            None
        );
        assert_eq!(
            state
                .continuity
                .arc_decision
                .downgrade_trend_support
                .current_pressure
                .0,
            1.0 - state.confidence.0
        );
        assert!(
            state
                .continuity
                .arc_decision
                .downgrade_support
                .evidence_loss_pressure
                .0
                >= 0.95
        );
        assert_eq!(
            state.continuity.arc_decision.provenance,
            super::TempoContinuityProvenance::NoTempo
        );
        assert_eq!(
            state.continuity.arc_decision.expiry,
            super::TempoContinuityArcActionExpiry {
                guaranteed_until_beats: 0,
                fallback_after_beats: 0,
                clear_after_beats: 0,
                max_failed_revalidations: 0,
            }
        );
        assert_eq!(
            state.continuity.trigger,
            super::TempoContinuityTrigger::EvidenceLoss
        );
        assert_eq!(
            state.continuity.causes.primary,
            super::TempoContinuityCause::TempoAmbiguity
        );
        assert_eq!(state.continuity.expiry.clear_after_beats, 0);
        assert_eq!(
            state.continuity.lifecycle.refresh.action,
            super::TempoContinuityAction::Clear
        );
        assert_eq!(
            state.continuity.lifecycle.refresh.provenance,
            super::TempoContinuityProvenance::NoTempo
        );
        assert_eq!(
            state.continuity.lifecycle.refresh.severity,
            super::TempoContinuitySeverity::Cleared
        );
        assert_eq!(
            state.continuity.lifecycle.refresh.history,
            super::TempoContinuityHistory::Degrading
        );
        assert_eq!(state.continuity.refresh_strength.0, 0.0);
    }

    #[test]
    fn tempo_continuity_calibrates_severity_history_and_refresh_strength() {
        let integer = super::tempo_state_recommendation(
            synthetic_tempo_interpretation(
                super::TempoRecommendation::SnapInteger,
                super::TempoTrustLevel::Stable,
                super::TempoInterpretationReason::NearIntegerPulse,
                120.0,
                Some(120.0),
                0.86,
                0.08,
                0.22,
                0.82,
            ),
            super::Confidence::new(0.9),
            super::Confidence::new(0.12),
        );
        let core_window = super::tempo_state_recommendation(
            synthetic_tempo_interpretation(
                super::TempoRecommendation::UseCoreWindow,
                super::TempoTrustLevel::Guarded,
                super::TempoInterpretationReason::StableCoreWindow,
                90.0,
                None,
                0.64,
                0.07,
                0.72,
                0.64,
            ),
            super::Confidence::new(0.72),
            super::Confidence::new(0.18),
        );
        let guarded_refined = super::tempo_state_recommendation(
            synthetic_tempo_interpretation(
                super::TempoRecommendation::UseRefined,
                super::TempoTrustLevel::Guarded,
                super::TempoInterpretationReason::StableRefinedPulse,
                117.8,
                None,
                0.61,
                0.09,
                0.32,
                0.62,
            ),
            super::Confidence::new(0.71),
            super::Confidence::new(0.21),
        );
        let deferred = super::tempo_state_recommendation(
            synthetic_tempo_interpretation(
                super::TempoRecommendation::Defer,
                super::TempoTrustLevel::Tentative,
                super::TempoInterpretationReason::UnstableTempo,
                89.9,
                None,
                0.38,
                0.03,
                0.8,
                0.3,
            ),
            super::Confidence::new(0.42),
            super::Confidence::new(0.55),
        );

        assert_eq!(
            integer.continuity.severity,
            super::TempoContinuitySeverity::Confirmed
        );
        assert_eq!(
            integer.continuity.history,
            super::TempoContinuityHistory::Reinforcing
        );
        assert_eq!(
            core_window.continuity.severity,
            super::TempoContinuitySeverity::Guarded
        );
        assert_eq!(
            core_window.continuity.history,
            super::TempoContinuityHistory::Preserving
        );
        assert_eq!(
            guarded_refined.continuity.severity,
            super::TempoContinuitySeverity::Fragile
        );
        assert_eq!(
            guarded_refined.continuity.history,
            super::TempoContinuityHistory::Preserving
        );
        assert_eq!(
            deferred.continuity.severity,
            super::TempoContinuitySeverity::Cleared
        );
        assert_eq!(
            deferred.continuity.history,
            super::TempoContinuityHistory::Degrading
        );
        assert!(integer.continuity.refresh_strength.0 > core_window.continuity.refresh_strength.0);
        assert!(core_window.continuity.refresh_strength.0 > deferred.continuity.refresh_strength.0);
        assert!(
            guarded_refined
                .continuity
                .lifecycle
                .refresh
                .refresh_strength
                .0
                > guarded_refined.continuity.refresh_strength.0
        );
        assert_eq!(
            deferred.continuity.lifecycle.decay[1].refresh_strength.0,
            0.0
        );
    }

    #[test]
    fn tempo_continuity_calibrates_causes_and_unresolved_spans() {
        let integer = super::tempo_state_recommendation(
            synthetic_tempo_interpretation(
                super::TempoRecommendation::SnapInteger,
                super::TempoTrustLevel::Stable,
                super::TempoInterpretationReason::NearIntegerPulse,
                120.0,
                Some(120.0),
                0.86,
                0.08,
                0.22,
                0.82,
            ),
            super::Confidence::new(0.9),
            super::Confidence::new(0.12),
        );
        let core_window = super::tempo_state_recommendation(
            synthetic_tempo_interpretation(
                super::TempoRecommendation::UseCoreWindow,
                super::TempoTrustLevel::Guarded,
                super::TempoInterpretationReason::StableCoreWindow,
                90.0,
                None,
                0.64,
                0.07,
                0.72,
                0.64,
            ),
            super::Confidence::new(0.72),
            super::Confidence::new(0.18),
        );
        let guarded_refined = super::tempo_state_recommendation(
            synthetic_tempo_interpretation(
                super::TempoRecommendation::UseRefined,
                super::TempoTrustLevel::Guarded,
                super::TempoInterpretationReason::StableRefinedPulse,
                117.8,
                None,
                0.61,
                0.09,
                0.32,
                0.62,
            ),
            super::Confidence::new(0.71),
            super::Confidence::new(0.21),
        );
        let deferred = super::tempo_state_recommendation(
            synthetic_tempo_interpretation(
                super::TempoRecommendation::Defer,
                super::TempoTrustLevel::Tentative,
                super::TempoInterpretationReason::UnstableTempo,
                89.9,
                None,
                0.38,
                0.03,
                0.8,
                0.3,
            ),
            super::Confidence::new(0.42),
            super::Confidence::new(0.55),
        );

        assert_eq!(
            integer.continuity.trigger,
            super::TempoContinuityTrigger::StableRevalidation
        );
        assert_eq!(
            integer.continuity.unresolved,
            super::TempoContinuityUnresolvedSpan {
                beats: 0,
                failed_revalidations: 0,
            }
        );
        assert_eq!(
            integer.continuity.causes.primary,
            super::TempoContinuityCause::StableTempoEvidence
        );
        assert_eq!(
            core_window.continuity.trigger,
            super::TempoContinuityTrigger::BoundaryDrift
        );
        assert_eq!(
            core_window.continuity.unresolved,
            super::TempoContinuityUnresolvedSpan {
                beats: 8,
                failed_revalidations: 2,
            }
        );
        assert_eq!(
            core_window.continuity.causes.primary,
            super::TempoContinuityCause::BoundaryDrift
        );
        assert!(core_window
            .continuity
            .causes
            .secondary
            .into_iter()
            .flatten()
            .any(|cause| cause == super::TempoContinuityCause::CoreWindowCarry));
        assert_eq!(
            guarded_refined.continuity.trigger,
            super::TempoContinuityTrigger::AmbiguityCarry
        );
        assert_eq!(
            guarded_refined.continuity.unresolved,
            super::TempoContinuityUnresolvedSpan {
                beats: 4,
                failed_revalidations: 1,
            }
        );
        assert_eq!(
            guarded_refined.continuity.causes.primary,
            super::TempoContinuityCause::TempoAmbiguity
        );
        assert_eq!(
            deferred.continuity.trigger,
            super::TempoContinuityTrigger::EvidenceLoss
        );
        assert_eq!(deferred.continuity.unresolved.beats, 0);
        assert_eq!(
            deferred.continuity.causes.primary,
            super::TempoContinuityCause::TempoAmbiguity
        );
        assert!(deferred
            .continuity
            .causes
            .secondary
            .into_iter()
            .flatten()
            .any(|cause| cause == super::TempoContinuityCause::EvidenceLoss));
    }

    #[test]
    fn tempo_continuity_calibrates_arcs_and_arc_support() {
        let integer = super::tempo_state_recommendation(
            synthetic_tempo_interpretation(
                super::TempoRecommendation::SnapInteger,
                super::TempoTrustLevel::Stable,
                super::TempoInterpretationReason::NearIntegerPulse,
                120.0,
                Some(120.0),
                0.86,
                0.08,
                0.22,
                0.82,
            ),
            super::Confidence::new(0.9),
            super::Confidence::new(0.12),
        );
        let core_window = super::tempo_state_recommendation(
            synthetic_tempo_interpretation(
                super::TempoRecommendation::UseCoreWindow,
                super::TempoTrustLevel::Guarded,
                super::TempoInterpretationReason::StableCoreWindow,
                90.0,
                None,
                0.64,
                0.07,
                0.72,
                0.64,
            ),
            super::Confidence::new(0.72),
            super::Confidence::new(0.18),
        );
        let guarded_refined = super::tempo_state_recommendation(
            synthetic_tempo_interpretation(
                super::TempoRecommendation::UseRefined,
                super::TempoTrustLevel::Guarded,
                super::TempoInterpretationReason::StableRefinedPulse,
                117.8,
                None,
                0.61,
                0.09,
                0.32,
                0.62,
            ),
            super::Confidence::new(0.71),
            super::Confidence::new(0.21),
        );
        let deferred = super::tempo_state_recommendation(
            synthetic_tempo_interpretation(
                super::TempoRecommendation::Defer,
                super::TempoTrustLevel::Tentative,
                super::TempoInterpretationReason::UnstableTempo,
                89.9,
                None,
                0.38,
                0.03,
                0.8,
                0.3,
            ),
            super::Confidence::new(0.42),
            super::Confidence::new(0.55),
        );

        assert_eq!(
            integer.continuity.arc,
            super::TempoContinuityArc::Recovering
        );
        assert_eq!(
            core_window.continuity.arc,
            super::TempoContinuityArc::Stalling
        );
        assert_eq!(
            guarded_refined.continuity.arc,
            super::TempoContinuityArc::Recovering
        );
        assert_eq!(
            deferred.continuity.arc,
            super::TempoContinuityArc::Collapsing
        );
        assert_eq!(
            integer.continuity.arc_rationale,
            super::TempoContinuityArcRationale::RefreshStrength
        );
        assert_eq!(
            integer.continuity.arc_decision.recommendation,
            super::TempoContinuityArcRecommendation::KeepLock
        );
        assert_eq!(
            integer.continuity.arc_decision.action,
            super::TempoContinuityArcAction::LockCurrentTempo
        );
        assert_eq!(
            integer.continuity.arc_decision.severity,
            super::TempoContinuitySeverity::Confirmed
        );
        assert_eq!(
            integer.continuity.arc_decision.fallback_action,
            super::TempoContinuityArcAction::ReacquireCurrentTempo
        );
        assert_eq!(
            integer.continuity.arc_decision.downgrade_rationale,
            super::TempoContinuityArcDowngradeRationale::StabilityWindowEnd
        );
        assert_eq!(
            integer.continuity.arc_decision.downgrade_trend,
            super::TempoContinuityArcDowngradeTrend::Easing
        );
        assert_eq!(
            integer.continuity.arc_decision.downgrade_trend_rationale,
            super::TempoContinuityArcDowngradeTrendRationale::StabilityWindowCarry
        );
        assert_eq!(
            integer.continuity.arc_decision.downgrade_inflection.stage,
            super::TempoContinuityArcDowngradeInflectionStage::NextStage
        );
        assert_eq!(
            integer
                .continuity
                .arc_decision
                .downgrade_inflection
                .competing_stage,
            Some(super::TempoContinuityArcDowngradeInflectionStage::TerminalClear)
        );
        assert_eq!(
            integer
                .continuity
                .arc_decision
                .downgrade_inflection
                .rationale_balance
                .primary
                .dominant,
            super::TempoContinuityArcDowngradeStageRationale::StabilityWindow
        );
        assert!(matches!(
            integer
                .continuity
                .arc_decision
                .downgrade_inflection
                .rationale_balance
                .competing
                .map(|weights| weights.dominant),
            Some(super::TempoContinuityArcDowngradeStageRationale::EvidenceLoss)
                | Some(super::TempoContinuityArcDowngradeStageRationale::StabilityWindow)
                | None
        ));
        assert!(
            integer
                .continuity
                .arc_decision
                .downgrade_inflection
                .balance
                .competing_weight
                .0
                >= 0.0
        );
        assert!(
            integer
                .continuity
                .arc_decision
                .downgrade_support
                .stability_window_pressure
                .0
                > integer
                    .continuity
                    .arc_decision
                    .downgrade_support
                    .failed_revalidation_pressure
                    .0
        );
        assert_eq!(
            integer.continuity.arc_decision.provenance,
            super::TempoContinuityProvenance::IntegerSnap
        );
        assert_eq!(
            core_window.continuity.arc_rationale,
            super::TempoContinuityArcRationale::BoundaryDrift
        );
        assert_eq!(
            core_window.continuity.arc_decision.recommendation,
            super::TempoContinuityArcRecommendation::MonitorRecovery
        );
        assert_eq!(
            core_window.continuity.arc_decision.action,
            super::TempoContinuityArcAction::PreferCoreWindowTempo
        );
        assert_eq!(
            core_window.continuity.arc_decision.severity,
            super::TempoContinuitySeverity::Guarded
        );
        assert_eq!(
            core_window.continuity.arc_decision.fallback_action,
            super::TempoContinuityArcAction::PreservePriorTempo
        );
        assert_eq!(
            core_window.continuity.arc_decision.downgrade_rationale,
            super::TempoContinuityArcDowngradeRationale::BoundaryDrift
        );
        assert_eq!(
            core_window.continuity.arc_decision.downgrade_trend,
            super::TempoContinuityArcDowngradeTrend::Rising
        );
        assert_eq!(
            core_window
                .continuity
                .arc_decision
                .downgrade_trend_rationale,
            super::TempoContinuityArcDowngradeTrendRationale::BoundaryEscalation
        );
        assert_eq!(
            core_window
                .continuity
                .arc_decision
                .downgrade_inflection
                .stage,
            super::TempoContinuityArcDowngradeInflectionStage::NextStage
        );
        assert_eq!(
            core_window
                .continuity
                .arc_decision
                .downgrade_inflection
                .competing_stage,
            Some(super::TempoContinuityArcDowngradeInflectionStage::TerminalClear)
        );
        assert_eq!(
            core_window
                .continuity
                .arc_decision
                .downgrade_inflection
                .rationale_balance
                .primary
                .dominant,
            super::TempoContinuityArcDowngradeStageRationale::BoundaryDrift
        );
        assert!(matches!(
            core_window
                .continuity
                .arc_decision
                .downgrade_inflection
                .rationale_balance
                .competing
                .map(|weights| weights.dominant),
            Some(super::TempoContinuityArcDowngradeStageRationale::EvidenceLoss)
                | Some(super::TempoContinuityArcDowngradeStageRationale::BoundaryDrift)
                | Some(super::TempoContinuityArcDowngradeStageRationale::StabilityWindow)
                | None
        ));
        assert!(
            core_window
                .continuity
                .arc_decision
                .downgrade_inflection
                .balance
                .primary_weight
                .0
                >= core_window
                    .continuity
                    .arc_decision
                    .downgrade_inflection
                    .balance
                    .competing_weight
                    .0
        );
        assert!(
            core_window
                .continuity
                .arc_decision
                .downgrade_support
                .boundary_drift_pressure
                .0
                > core_window
                    .continuity
                    .arc_decision
                    .downgrade_support
                    .stability_window_pressure
                    .0
        );
        assert_eq!(
            core_window.continuity.arc_decision.provenance,
            super::TempoContinuityProvenance::CoreWindowEstimate
        );
        assert_eq!(
            guarded_refined.continuity.arc_rationale,
            super::TempoContinuityArcRationale::RefreshStrength
        );
        assert_eq!(
            guarded_refined.continuity.arc_decision.recommendation,
            super::TempoContinuityArcRecommendation::MonitorRecovery
        );
        assert_eq!(
            guarded_refined.continuity.arc_decision.action,
            super::TempoContinuityArcAction::ReacquireCurrentTempo
        );
        assert_eq!(
            guarded_refined.continuity.arc_decision.severity,
            super::TempoContinuitySeverity::Fragile
        );
        assert_eq!(
            guarded_refined.continuity.arc_decision.fallback_action,
            super::TempoContinuityArcAction::ClearTempo
        );
        assert_eq!(
            guarded_refined.continuity.arc_decision.downgrade_rationale,
            super::TempoContinuityArcDowngradeRationale::AmbiguityCarry
        );
        assert_eq!(
            guarded_refined.continuity.arc_decision.downgrade_trend,
            super::TempoContinuityArcDowngradeTrend::Easing
        );
        assert_eq!(
            guarded_refined
                .continuity
                .arc_decision
                .downgrade_trend_rationale,
            super::TempoContinuityArcDowngradeTrendRationale::AmbiguityCarry
        );
        assert_eq!(
            guarded_refined
                .continuity
                .arc_decision
                .downgrade_inflection
                .stage,
            super::TempoContinuityArcDowngradeInflectionStage::NextStage
        );
        assert_eq!(
            guarded_refined
                .continuity
                .arc_decision
                .downgrade_inflection
                .competing_stage,
            Some(super::TempoContinuityArcDowngradeInflectionStage::TerminalClear)
        );
        assert_eq!(
            guarded_refined
                .continuity
                .arc_decision
                .downgrade_inflection
                .rationale_balance
                .primary
                .dominant,
            super::TempoContinuityArcDowngradeStageRationale::AmbiguityCarry
        );
        assert!(matches!(
            guarded_refined
                .continuity
                .arc_decision
                .downgrade_inflection
                .rationale_balance
                .competing
                .map(|weights| weights.dominant),
            Some(super::TempoContinuityArcDowngradeStageRationale::EvidenceLoss)
                | Some(super::TempoContinuityArcDowngradeStageRationale::AmbiguityCarry)
                | Some(super::TempoContinuityArcDowngradeStageRationale::StabilityWindow)
                | None
        ));
        assert!(
            guarded_refined
                .continuity
                .arc_decision
                .downgrade_inflection
                .balance
                .competing_weight
                .0
                >= 0.0
        );
        assert!(
            guarded_refined
                .continuity
                .arc_decision
                .downgrade_support
                .ambiguity_pressure
                .0
                > guarded_refined
                    .continuity
                    .arc_decision
                    .downgrade_support
                    .failed_revalidation_pressure
                    .0
        );
        assert_eq!(
            guarded_refined.continuity.arc_decision.provenance,
            super::TempoContinuityProvenance::GuardedRefinedEstimate
        );
        assert_eq!(
            deferred.continuity.arc_rationale,
            super::TempoContinuityArcRationale::EvidenceLoss
        );
        assert_eq!(
            deferred.continuity.arc_decision.recommendation,
            super::TempoContinuityArcRecommendation::Clear
        );
        assert_eq!(
            deferred.continuity.arc_decision.action,
            super::TempoContinuityArcAction::ClearTempo
        );
        assert_eq!(
            deferred.continuity.arc_decision.severity,
            super::TempoContinuitySeverity::Cleared
        );
        assert_eq!(
            deferred.continuity.arc_decision.fallback_action,
            super::TempoContinuityArcAction::ClearTempo
        );
        assert_eq!(
            deferred.continuity.arc_decision.downgrade_rationale,
            super::TempoContinuityArcDowngradeRationale::EvidenceLoss
        );
        assert_eq!(
            deferred.continuity.arc_decision.downgrade_trend,
            super::TempoContinuityArcDowngradeTrend::Stable
        );
        assert_eq!(
            deferred.continuity.arc_decision.downgrade_trend_rationale,
            super::TempoContinuityArcDowngradeTrendRationale::FlatCollapse
        );
        assert_eq!(
            deferred.continuity.arc_decision.downgrade_inflection.stage,
            super::TempoContinuityArcDowngradeInflectionStage::FlatWindow
        );
        assert_eq!(
            deferred
                .continuity
                .arc_decision
                .downgrade_inflection
                .competing_stage,
            None
        );
        assert_eq!(
            deferred
                .continuity
                .arc_decision
                .downgrade_inflection
                .rationale_balance
                .primary
                .dominant,
            super::TempoContinuityArcDowngradeStageRationale::EvidenceLoss
        );
        assert_eq!(
            deferred
                .continuity
                .arc_decision
                .downgrade_inflection
                .rationale_balance
                .competing,
            None
        );
        assert_eq!(
            deferred
                .continuity
                .arc_decision
                .downgrade_inflection
                .balance
                .unattributed_weight,
            super::Confidence::new(1.0)
        );
        assert!(
            deferred
                .continuity
                .arc_decision
                .downgrade_support
                .evidence_loss_pressure
                .0
                > deferred
                    .continuity
                    .arc_decision
                    .downgrade_support
                    .ambiguity_pressure
                    .0
        );
        assert_eq!(
            deferred.continuity.arc_decision.provenance,
            super::TempoContinuityProvenance::NoTempo
        );
        assert_eq!(
            core_window
                .continuity
                .arc_decision
                .expiry
                .max_failed_revalidations,
            2
        );
        assert_eq!(
            guarded_refined
                .continuity
                .arc_decision
                .expiry
                .fallback_after_beats,
            12
        );
        assert!(
            integer.continuity.arc_support.refresh_strength.0
                > core_window.continuity.arc_support.refresh_strength.0
        );
        assert!(
            core_window.continuity.arc_support.drift_pressure.0
                > integer.continuity.arc_support.drift_pressure.0
        );
        assert!(
            deferred.continuity.arc_support.instability_pressure.0
                > guarded_refined
                    .continuity
                    .arc_support
                    .instability_pressure
                    .0
        );
        assert!(
            integer.continuity.arc_decision.confidence.0
                > guarded_refined.continuity.arc_decision.confidence.0
        );
        assert!(
            deferred.continuity.arc_decision.confidence.0
                > core_window.continuity.arc_decision.confidence.0
        );
        assert!(
            core_window
                .continuity
                .arc_decision
                .downgrade_trend_support
                .next_stage_pressure
                .0
                > core_window
                    .continuity
                    .arc_decision
                    .downgrade_trend_support
                    .current_pressure
                    .0
        );
        assert!(
            integer
                .continuity
                .arc_decision
                .downgrade_trend_support
                .terminal_pressure
                .0
                > integer
                    .continuity
                    .arc_decision
                    .downgrade_trend_support
                    .current_pressure
                    .0
        );
        assert!(
            core_window
                .continuity
                .arc_decision
                .downgrade_support
                .failed_revalidation_pressure
                .0
                > integer
                    .continuity
                    .arc_decision
                    .downgrade_support
                    .failed_revalidation_pressure
                    .0
        );
    }

    #[test]
    fn beat_tracker_calibrates_meter_confidence_between_steady_and_dropout_sections() {
        let sample_rate = 48_000;
        let bpm = 120.0;

        let mut steady_fixture = FixtureBuilder::new();
        steady_fixture.push_four_four_section(GrooveSection {
            bars: 6,
            beat_pattern: [0.5, 0.26, 0.38, 0.24],
            chord_cycle: &[CHORD_A, CHORD_B, CHORD_C],
            chord_every_bars: 1,
            section_marker: None,
            bar_patterns: None,
            bar_chords: None,
            dropout_bars: &[],
        });
        let steady = analyze_fixture(&steady_fixture.build(sample_rate, bpm));

        let mut dropout_fixture = FixtureBuilder::new();
        dropout_fixture.push_four_four_section(GrooveSection {
            bars: 6,
            beat_pattern: [0.5, 0.26, 0.38, 0.24],
            chord_cycle: &[CHORD_A, CHORD_B, CHORD_C],
            chord_every_bars: 1,
            section_marker: Some((8, CHORD_D, 0.75)),
            bar_patterns: None,
            bar_chords: None,
            dropout_bars: &[2],
        });
        let dropout = analyze_fixture(&dropout_fixture.build(sample_rate, bpm));

        let steady_meter = steady.meter.as_ref().expect("steady meter");
        let dropout_meter = dropout.meter.as_ref().expect("dropout meter");
        assert_eq!(steady_meter.beats_per_bar, 4);
        assert_eq!(dropout_meter.beats_per_bar, 4);
        assert!(steady_meter.confidence.0 > dropout_meter.confidence.0);
    }

    #[test]
    fn beat_tracker_handles_fill_bar_with_harmonic_rhythm_changes() {
        let preset = RhythmPreset::FillTransition124(FillDensityVariant::Medium);
        let (bpm, result) = analyze_preset(preset);
        let meter = assert_meter(preset, &result, 4, 0.18);

        assert_detected_bpm(preset, &result, bpm, 3.0);
        assert!(result.confidence.0 > result.tempo_ambiguity.0);
        assert!(meter.downbeat_positions_seconds.len() >= 2);
    }

    #[test]
    fn beat_tracker_prefers_unknown_meter_for_dropout_heavy_transition_fixture() {
        let preset = RhythmPreset::Dropout120(DropoutVariant::Heavy);
        let (bpm, result) = analyze_preset(preset);
        assert_detected_bpm(preset, &result, bpm, 3.0);
        assert!(result.meter.is_none());
    }

    #[test]
    fn beat_tracker_calibrates_meter_trust_levels_across_public_categories() {
        let (_, structured) = analyze_preset(RhythmPreset::StructuredHarmony120(
            HarmonicRhythmVariant::Active,
        ));
        let (_, weak_backbeat) = analyze_preset(RhythmPreset::WeakBackbeat118);
        let (_, sustained_reset) = analyze_preset(RhythmPreset::BarTransition120(
            BarTransitionVariant::ReentryAcceleratingHarmonySustainedReset,
        ));

        let structured_meter = structured.meter.as_ref().expect("structured meter");
        let weak_backbeat_meter = weak_backbeat.meter.as_ref().expect("weak backbeat meter");
        let sustained_reset_meter = sustained_reset
            .meter
            .as_ref()
            .expect("sustained reset meter");

        assert_eq!(structured_meter.trust, super::MeterTrustLevel::Stable);
        assert_eq!(
            structured_meter.recommendation,
            super::MeterRecommendation::Lock
        );
        assert_eq!(structured.meter_state.action, super::MeterStateAction::Lock);
        assert_eq!(
            structured.meter_state.continuity.bar_length.action,
            super::MeterContinuityAction::Lock
        );
        assert_eq!(
            structured.meter_state.continuity.downbeat_phase.action,
            super::MeterContinuityAction::Lock
        );
        assert_eq!(
            structured.meter_state.continuity.bar_length.source,
            super::MeterContinuitySource::CurrentMeter
        );
        assert_eq!(
            structured
                .meter_state
                .continuity
                .bar_length
                .lifecycle
                .refresh
                .action,
            super::MeterContinuityAction::Lock
        );
        assert_eq!(
            structured.meter_state.continuity.bar_length.lifecycle.decay[0].action,
            super::MeterContinuityAction::Retain
        );
        assert_eq!(
            structured.meter_state.continuity.bar_length.lifecycle.decay[1].action,
            super::MeterContinuityAction::Clear
        );
        assert_eq!(weak_backbeat_meter.trust, super::MeterTrustLevel::Tentative);
        assert_eq!(
            weak_backbeat_meter.recommendation,
            super::MeterRecommendation::Defer
        );
        assert_eq!(
            weak_backbeat.meter_state.action,
            super::MeterStateAction::Hold
        );
        assert_eq!(
            weak_backbeat.meter_state.continuity.bar_length.action,
            super::MeterContinuityAction::Retain
        );
        assert_eq!(
            weak_backbeat.meter_state.continuity.downbeat_phase.action,
            super::MeterContinuityAction::Reacquire
        );
        assert_eq!(
            weak_backbeat.meter_state.continuity.bar_length.source,
            super::MeterContinuitySource::CurrentMeter
        );
        assert_eq!(
            weak_backbeat
                .meter_state
                .continuity
                .bar_length
                .lifecycle
                .refresh
                .action,
            super::MeterContinuityAction::Lock
        );
        assert_eq!(
            weak_backbeat
                .meter_state
                .continuity
                .bar_length
                .lifecycle
                .decay[0]
                .action,
            super::MeterContinuityAction::Reacquire
        );
        assert_eq!(
            weak_backbeat
                .meter_state
                .continuity
                .bar_length
                .lifecycle
                .decay[1]
                .action,
            super::MeterContinuityAction::Clear
        );
        assert_eq!(
            sustained_reset_meter.trust,
            super::MeterTrustLevel::Recovering
        );
        assert_eq!(
            sustained_reset_meter.recommendation,
            super::MeterRecommendation::Monitor
        );
        assert_eq!(
            sustained_reset.meter_state.action,
            super::MeterStateAction::Watch
        );
        assert_eq!(
            sustained_reset.meter_state.continuity.bar_length.action,
            super::MeterContinuityAction::Retain
        );
        assert_eq!(
            sustained_reset.meter_state.continuity.downbeat_phase.action,
            super::MeterContinuityAction::Reacquire
        );
        assert_eq!(
            sustained_reset.meter_state.continuity.bar_length.source,
            super::MeterContinuitySource::RecoveryWindow
        );
        assert_eq!(
            sustained_reset
                .meter_state
                .continuity
                .bar_length
                .lifecycle
                .refresh
                .action,
            super::MeterContinuityAction::Lock
        );
        assert_eq!(
            sustained_reset
                .meter_state
                .continuity
                .bar_length
                .lifecycle
                .decay[0]
                .action,
            super::MeterContinuityAction::Reacquire
        );
        assert_eq!(
            sustained_reset
                .meter_state
                .continuity
                .bar_length
                .lifecycle
                .decay[1]
                .action,
            super::MeterContinuityAction::Clear
        );
        assert!(
            structured_meter.support_profile.whole_track_strength.0
                >= weak_backbeat_meter.support_profile.whole_track_strength.0
        );
        assert!(
            sustained_reset_meter
                .support_profile
                .segment_recovery_strength
                .0
                > sustained_reset_meter.support_profile.whole_track_strength.0
        );
    }

    #[test]
    fn beat_tracker_exposes_whole_track_structure_summary_for_stable_meter() {
        let (_, structured) = analyze_preset(RhythmPreset::StructuredHarmony120(
            HarmonicRhythmVariant::Active,
        ));

        let summary = structured
            .rhythm_structure_summary()
            .expect("structured rhythm structure summary");

        assert_eq!(summary.beats_per_bar, 4);
        assert_eq!(
            summary.detection_kind,
            super::MeterDetectionKind::WholeTrack
        );
        assert_eq!(summary.trust, super::MeterTrustLevel::Stable);
        assert_eq!(summary.recommendation, super::MeterRecommendation::Lock);
        assert_eq!(summary.continuity.action, structured.meter_state.action);
        assert_eq!(
            summary.continuity.bar_length_action,
            structured.meter_state.continuity.bar_length.action
        );
        assert_eq!(
            summary.continuity.downbeat_phase_action,
            structured.meter_state.continuity.downbeat_phase.action
        );
        assert_eq!(summary.bar_count, summary.downbeat_positions_seconds.len());
        assert!(summary.bar_count >= 2);
        assert_eq!(summary.recovered_bar_count, 0);
        assert!(summary.recovery.is_none());
        assert!(summary
            .bars
            .iter()
            .all(|bar| matches!(bar.support, super::BarSupportKind::WholeTrack)));
        assert_eq!(
            summary.bars.first().map(|bar| bar.start_seconds),
            summary.downbeat_positions_seconds.first().copied()
        );
    }

    #[test]
    fn beat_tracker_exposes_recovery_backed_structure_summary_for_segment_meter() {
        let (_, sustained_reset) = analyze_preset(RhythmPreset::BarTransition120(
            BarTransitionVariant::ReentryAcceleratingHarmonySustainedReset,
        ));

        let summary = sustained_reset
            .rhythm_structure_summary()
            .expect("recovery-backed rhythm structure summary");

        assert_eq!(
            summary.detection_kind,
            super::MeterDetectionKind::SegmentRecovery
        );
        assert!(summary.recovery.is_some());
        assert!(summary.recovered_bar_count > 0);
        assert!(summary
            .bars
            .iter()
            .any(|bar| matches!(bar.support, super::BarSupportKind::RecoveryWindow)));
        assert_eq!(
            summary.continuity.action,
            sustained_reset.meter_state.action
        );
        assert_eq!(
            summary.continuity.reason,
            sustained_reset.meter_state.reason
        );
        let recovery = summary.recovery.as_ref().expect("recovery context");
        assert!(summary.bars.iter().any(|bar| {
            matches!(bar.support, super::BarSupportKind::RecoveryWindow)
                && bar.start_seconds <= recovery.end_seconds
        }));
    }

    #[test]
    fn beat_tracker_structure_assessment_surfaces_weak_accent_ambiguity() {
        let (_, weak_backbeat) = analyze_preset(RhythmPreset::WeakBackbeat118);

        let assessment = weak_backbeat.rhythm_structure_assessment();

        assert!(assessment.structure.is_some());
        assert_eq!(
            assessment.ambiguity.kind,
            super::RhythmStructureAmbiguityKind::WeakAccent
        );
        assert!(assessment.ambiguity.runner_up.is_some());
        assert!(assessment.ambiguity.confidence.0 > 0.2);
        assert_eq!(assessment.fallback.action, super::MeterStateAction::Hold);
        assert_eq!(
            assessment.fallback.downbeat_phase_action,
            super::MeterContinuityAction::Reacquire
        );
    }

    #[test]
    fn beat_tracker_structure_assessment_surfaces_competing_meter_ambiguity() {
        let (_, ambiguous) = analyze_preset(RhythmPreset::AmbiguousSubdivision90);

        let assessment = ambiguous.rhythm_structure_assessment();

        let primary = assessment
            .ambiguity
            .primary
            .expect("primary ambiguity candidate");
        let runner_up = assessment
            .ambiguity
            .runner_up
            .expect("runner-up ambiguity candidate");

        assert_ne!(primary.beats_per_bar, runner_up.beats_per_bar);
        assert!(assessment.ambiguity.confidence.0 > 0.2);
    }

    #[test]
    fn beat_tracker_structure_assessment_surfaces_phase_fallback_for_pickup_extension() {
        let (_, pickup_extended) = analyze_preset(RhythmPreset::BarTransition120(
            BarTransitionVariant::PickupExtended,
        ));

        let assessment = pickup_extended.rhythm_structure_assessment();

        assert!(assessment.structure.is_some());
        assert_ne!(
            assessment.ambiguity.kind,
            super::RhythmStructureAmbiguityKind::InsufficientEvidence
        );
        assert_eq!(
            assessment.fallback.downbeat_phase_action,
            super::MeterContinuityAction::Reacquire
        );
    }

    #[test]
    fn beat_tracker_structure_assessment_surfaces_recovery_window_fallback_without_meter() {
        let (_, accelerating_reset) = analyze_preset(RhythmPreset::BarTransition120(
            BarTransitionVariant::ReentryAcceleratingHarmonyReset,
        ));

        let assessment = accelerating_reset.rhythm_structure_assessment();

        assert!(assessment.structure.is_none());
        assert!(assessment.fallback.recovery_window_available);
        assert_eq!(assessment.fallback.action, super::MeterStateAction::Watch);
        assert_eq!(
            assessment.ambiguity.kind,
            super::RhythmStructureAmbiguityKind::RecoveryWindowFallback
        );
        assert!(assessment.fallback.trailing_recovery_confidence.0 > 0.0);
    }

    #[test]
    fn beat_tracker_calibrates_meter_recommendations_across_action_categories() {
        let (_, structured) = analyze_preset(RhythmPreset::StructuredHarmony120(
            HarmonicRhythmVariant::Active,
        ));
        let (_, weak_backbeat) = analyze_preset(RhythmPreset::WeakBackbeat118);
        let (_, sustained_reset) = analyze_preset(RhythmPreset::BarTransition120(
            BarTransitionVariant::ReentryAcceleratingHarmonySustainedReset,
        ));

        let structured_meter = structured.meter.as_ref().expect("structured meter");
        let weak_backbeat_meter = weak_backbeat.meter.as_ref().expect("weak backbeat meter");
        let sustained_reset_meter = sustained_reset
            .meter
            .as_ref()
            .expect("sustained reset meter");

        assert_eq!(
            structured_meter.recommendation,
            super::MeterRecommendation::Lock
        );
        assert_eq!(
            structured.meter_state.reason,
            super::MeterStateReason::StableMeter
        );
        assert_eq!(
            structured.meter_state.continuity.bar_length.action,
            super::MeterContinuityAction::Lock
        );
        assert_eq!(
            sustained_reset_meter.recommendation,
            super::MeterRecommendation::Monitor
        );
        assert_eq!(
            sustained_reset.meter_state.reason,
            super::MeterStateReason::RecoveringMeter
        );
        assert_eq!(
            sustained_reset.meter_state.continuity.downbeat_phase.action,
            super::MeterContinuityAction::Reacquire
        );
        assert_eq!(
            weak_backbeat_meter.recommendation,
            super::MeterRecommendation::Defer
        );
        assert_eq!(
            weak_backbeat.meter_state.reason,
            super::MeterStateReason::TentativeMeter
        );
        assert_eq!(
            weak_backbeat.meter_state.continuity.bar_length.action,
            super::MeterContinuityAction::Retain
        );
        assert!(structured_meter.confidence.0 > weak_backbeat_meter.confidence.0);
        assert!(
            sustained_reset_meter
                .support_profile
                .recovery_duration_strength
                .0
                > 0.5
        );
        assert!(weak_backbeat_meter.recovery.is_none());
    }

    #[test]
    fn beat_tracker_calibrates_transition_meter_state_actions_for_meterless_cases() {
        let (_, dropout_heavy) = analyze_preset(RhythmPreset::Dropout120(DropoutVariant::Heavy));
        let (_, accelerating_reset) = analyze_preset(RhythmPreset::BarTransition120(
            BarTransitionVariant::ReentryAcceleratingHarmonyReset,
        ));
        let (_, modulation_extended) = analyze_preset(RhythmPreset::BarTransition120(
            BarTransitionVariant::ModulationDenseFillExtended,
        ));

        assert!(dropout_heavy.meter.is_none());
        assert!(accelerating_reset.meter.is_none());
        assert!(modulation_extended.meter.is_none());

        assert_eq!(
            dropout_heavy.meter_state.action,
            super::MeterStateAction::Hold
        );
        assert_eq!(
            dropout_heavy.meter_state.reason,
            super::MeterStateReason::DestabilizedHold
        );
        assert_eq!(
            dropout_heavy.meter_state.continuity.bar_length.action,
            super::MeterContinuityAction::Retain
        );
        assert_eq!(
            dropout_heavy.meter_state.continuity.downbeat_phase.action,
            super::MeterContinuityAction::Retain
        );
        assert_eq!(
            dropout_heavy.meter_state.continuity.bar_length.source,
            super::MeterContinuitySource::PriorMeter
        );
        assert_eq!(
            dropout_heavy
                .meter_state
                .continuity
                .bar_length
                .lifecycle
                .refresh
                .action,
            super::MeterContinuityAction::Retain
        );
        assert_eq!(
            dropout_heavy
                .meter_state
                .continuity
                .bar_length
                .lifecycle
                .decay[0]
                .action,
            super::MeterContinuityAction::Reacquire
        );
        assert_eq!(
            dropout_heavy
                .meter_state
                .continuity
                .bar_length
                .lifecycle
                .decay[1]
                .action,
            super::MeterContinuityAction::Clear
        );
        assert_eq!(
            accelerating_reset.meter_state.action,
            super::MeterStateAction::Watch
        );
        assert_eq!(
            accelerating_reset.meter_state.reason,
            super::MeterStateReason::RecoveryEmerging
        );
        assert_eq!(
            accelerating_reset.meter_state.continuity.bar_length.action,
            super::MeterContinuityAction::Retain
        );
        assert_eq!(
            accelerating_reset
                .meter_state
                .continuity
                .downbeat_phase
                .action,
            super::MeterContinuityAction::Reacquire
        );
        assert_eq!(
            accelerating_reset.meter_state.continuity.bar_length.source,
            super::MeterContinuitySource::RecoveryWindow
        );
        assert_eq!(
            accelerating_reset
                .meter_state
                .continuity
                .bar_length
                .lifecycle
                .refresh
                .action,
            super::MeterContinuityAction::Lock
        );
        assert_eq!(
            accelerating_reset
                .meter_state
                .continuity
                .bar_length
                .lifecycle
                .decay[0]
                .action,
            super::MeterContinuityAction::Reacquire
        );
        assert_eq!(
            accelerating_reset
                .meter_state
                .continuity
                .bar_length
                .lifecycle
                .decay[1]
                .action,
            super::MeterContinuityAction::Clear
        );
        assert_eq!(
            modulation_extended.meter_state.action,
            super::MeterStateAction::Clear
        );
        assert_eq!(
            modulation_extended.meter_state.reason,
            super::MeterStateReason::MeterCleared
        );
        assert_eq!(
            modulation_extended.meter_state.continuity.bar_length.action,
            super::MeterContinuityAction::Clear
        );
        assert_eq!(
            modulation_extended
                .meter_state
                .continuity
                .downbeat_phase
                .action,
            super::MeterContinuityAction::Clear
        );
        assert_eq!(
            modulation_extended.meter_state.continuity.bar_length.source,
            super::MeterContinuitySource::Cleared
        );
        assert_eq!(
            modulation_extended
                .meter_state
                .continuity
                .bar_length
                .lifecycle
                .refresh
                .action,
            super::MeterContinuityAction::Clear
        );
        assert_eq!(
            modulation_extended
                .meter_state
                .continuity
                .bar_length
                .lifecycle
                .decay[0]
                .action,
            super::MeterContinuityAction::Clear
        );
        assert_eq!(
            modulation_extended
                .meter_state
                .continuity
                .bar_length
                .lifecycle
                .decay[1]
                .action,
            super::MeterContinuityAction::Clear
        );
        assert!(
            dropout_heavy.meter_state.confidence.0
                >= modulation_extended.meter_state.confidence.0 - 0.05
        );
        assert!(
            accelerating_reset.meter_state.confidence.0
                > modulation_extended.meter_state.confidence.0
        );
    }

    #[test]
    fn beat_tracker_calibrates_meter_continuity_across_transition_families() {
        let (_, pickup) =
            analyze_preset(RhythmPreset::BarTransition120(BarTransitionVariant::Pickup));
        let (_, mixed_length) = analyze_preset(RhythmPreset::BarTransition120(
            BarTransitionVariant::MixedLength,
        ));
        let (_, sustained_reset) = analyze_preset(RhythmPreset::BarTransition120(
            BarTransitionVariant::ReentryAcceleratingHarmonySustainedReset,
        ));
        let (_, cadential_reanchor) = analyze_preset(RhythmPreset::BarTransition120(
            BarTransitionVariant::ReentryAcceleratingHarmonyCadentialReanchor,
        ));

        assert!(pickup.meter.is_some());
        assert!(mixed_length.meter.is_none());
        assert!(sustained_reset.meter.is_some());
        assert!(cadential_reanchor.meter.is_none());

        assert_eq!(
            pickup.meter_state.continuity.bar_length.action,
            super::MeterContinuityAction::Lock
        );
        assert_eq!(
            pickup
                .meter_state
                .continuity
                .bar_length
                .lifecycle
                .refresh
                .action,
            super::MeterContinuityAction::Lock
        );
        assert_eq!(
            pickup.meter_state.continuity.downbeat_phase.action,
            super::MeterContinuityAction::Reacquire
        );
        assert_eq!(
            pickup
                .meter_state
                .continuity
                .downbeat_phase
                .lifecycle
                .refresh
                .action,
            super::MeterContinuityAction::Lock
        );
        assert_eq!(
            sustained_reset.meter_state.continuity.bar_length.action,
            super::MeterContinuityAction::Retain
        );
        assert_eq!(
            sustained_reset
                .meter_state
                .continuity
                .bar_length
                .lifecycle
                .decay[0]
                .action,
            super::MeterContinuityAction::Reacquire
        );
        assert_eq!(
            sustained_reset.meter_state.continuity.downbeat_phase.action,
            super::MeterContinuityAction::Reacquire
        );
        assert_eq!(
            sustained_reset
                .meter_state
                .continuity
                .downbeat_phase
                .lifecycle
                .decay[1]
                .action,
            super::MeterContinuityAction::Clear
        );
        assert_eq!(
            mixed_length.meter_state.continuity.bar_length.action,
            super::MeterContinuityAction::Clear
        );
        assert_eq!(
            mixed_length.meter_state.continuity.downbeat_phase.action,
            super::MeterContinuityAction::Clear
        );
        assert_eq!(
            cadential_reanchor.meter_state.continuity.bar_length.action,
            super::MeterContinuityAction::Retain
        );
        assert_eq!(
            cadential_reanchor
                .meter_state
                .continuity
                .downbeat_phase
                .action,
            super::MeterContinuityAction::Reacquire
        );
    }

    #[test]
    fn beat_tracker_calibrates_meter_continuity_provenance_and_expiry_windows() {
        let (_, dropout_heavy) = analyze_preset(RhythmPreset::Dropout120(DropoutVariant::Heavy));
        let (_, dropout_extended) =
            analyze_preset(RhythmPreset::Dropout120(DropoutVariant::ExtendedHeavy));
        let (_, pickup) =
            analyze_preset(RhythmPreset::BarTransition120(BarTransitionVariant::Pickup));
        let (_, pickup_extended) = analyze_preset(RhythmPreset::BarTransition120(
            BarTransitionVariant::PickupExtended,
        ));
        let (_, sustained_reset) = analyze_preset(RhythmPreset::BarTransition120(
            BarTransitionVariant::ReentryAcceleratingHarmonySustainedReset,
        ));
        let (_, long_sustained_reset) = analyze_preset(RhythmPreset::BarTransition120(
            BarTransitionVariant::ReentryAcceleratingHarmonyLongSustainedReset,
        ));

        assert!(dropout_heavy.meter.is_none());
        assert!(dropout_extended.meter.is_none());
        assert!(pickup.meter.is_some());
        assert!(pickup_extended.meter.is_some());
        assert!(sustained_reset.meter.is_some());
        assert!(long_sustained_reset.meter.is_some());

        assert_eq!(
            dropout_heavy.meter_state.continuity.bar_length.source,
            super::MeterContinuitySource::PriorMeter
        );
        assert_eq!(
            dropout_heavy
                .meter_state
                .continuity
                .bar_length
                .lifecycle
                .refresh
                .action,
            super::MeterContinuityAction::Retain
        );
        assert_eq!(
            dropout_heavy
                .meter_state
                .continuity
                .bar_length
                .lifecycle
                .decay[0]
                .action,
            super::MeterContinuityAction::Reacquire
        );
        assert_eq!(
            dropout_extended.meter_state.continuity.bar_length.source,
            super::MeterContinuitySource::RecoveryWindow
        );
        assert_eq!(
            dropout_extended
                .meter_state
                .continuity
                .bar_length
                .lifecycle
                .refresh
                .action,
            super::MeterContinuityAction::Lock
        );
        assert_eq!(
            dropout_extended
                .meter_state
                .continuity
                .bar_length
                .lifecycle
                .decay[0]
                .action,
            super::MeterContinuityAction::Reacquire
        );
        assert!(
            dropout_extended
                .meter_state
                .continuity
                .bar_length
                .trusted_beats
                <= dropout_heavy
                    .meter_state
                    .continuity
                    .bar_length
                    .trusted_beats
        );
        assert!(
            dropout_extended
                .meter_state
                .continuity
                .bar_length
                .lifecycle
                .decay[1]
                .after_beats
                >= dropout_extended
                    .meter_state
                    .continuity
                    .bar_length
                    .trusted_beats
        );
        assert!(
            dropout_extended
                .meter_state
                .continuity
                .downbeat_phase
                .trusted_beats
                <= dropout_heavy
                    .meter_state
                    .continuity
                    .downbeat_phase
                    .trusted_beats
        );
        assert_eq!(
            dropout_extended
                .meter_state
                .continuity
                .downbeat_phase
                .source,
            super::MeterContinuitySource::RecoveryWindow
        );

        assert_eq!(
            pickup.meter_state.continuity.bar_length.source,
            super::MeterContinuitySource::CurrentMeter
        );
        assert_eq!(
            pickup_extended.meter_state.continuity.downbeat_phase.source,
            super::MeterContinuitySource::CurrentMeter
        );
        assert_eq!(
            pickup_extended
                .meter_state
                .continuity
                .downbeat_phase
                .lifecycle
                .refresh
                .action,
            super::MeterContinuityAction::Lock
        );
        assert_eq!(
            pickup_extended
                .meter_state
                .continuity
                .downbeat_phase
                .lifecycle
                .decay[0]
                .action,
            super::MeterContinuityAction::Reacquire
        );
        assert_eq!(
            pickup.meter_state.continuity.downbeat_phase.trusted_beats,
            0
        );
        assert_eq!(
            pickup_extended
                .meter_state
                .continuity
                .downbeat_phase
                .trusted_beats,
            0
        );
        assert!(
            pickup_extended
                .meter_state
                .continuity
                .downbeat_phase
                .revalidate_after_beats
                <= pickup
                    .meter_state
                    .continuity
                    .downbeat_phase
                    .revalidate_after_beats
        );

        assert_eq!(
            sustained_reset.meter_state.continuity.bar_length.source,
            super::MeterContinuitySource::RecoveryWindow
        );
        assert_eq!(
            sustained_reset
                .meter_state
                .continuity
                .bar_length
                .lifecycle
                .refresh
                .action,
            super::MeterContinuityAction::Lock
        );
        assert_eq!(
            sustained_reset
                .meter_state
                .continuity
                .bar_length
                .lifecycle
                .decay[0]
                .action,
            super::MeterContinuityAction::Reacquire
        );
        assert_eq!(
            long_sustained_reset
                .meter_state
                .continuity
                .bar_length
                .source,
            super::MeterContinuitySource::RecoveryWindow
        );
        assert!(
            long_sustained_reset
                .meter_state
                .continuity
                .bar_length
                .trusted_beats
                > sustained_reset
                    .meter_state
                    .continuity
                    .bar_length
                    .trusted_beats
        );
        assert!(
            long_sustained_reset
                .meter_state
                .continuity
                .bar_length
                .revalidate_after_beats
                >= sustained_reset
                    .meter_state
                    .continuity
                    .bar_length
                    .revalidate_after_beats
        );
        assert!(
            long_sustained_reset
                .meter_state
                .continuity
                .bar_length
                .lifecycle
                .decay[1]
                .after_beats
                > sustained_reset
                    .meter_state
                    .continuity
                    .bar_length
                    .lifecycle
                    .decay[1]
                    .after_beats
        );
    }

    #[test]
    fn beat_tracker_calibrates_meter_continuity_severity_across_lifecycle_stages() {
        let (_, structured) = analyze_preset(RhythmPreset::StructuredHarmony120(
            HarmonicRhythmVariant::Active,
        ));
        let (_, weak_backbeat) = analyze_preset(RhythmPreset::WeakBackbeat118);
        let (_, dropout_heavy) = analyze_preset(RhythmPreset::Dropout120(DropoutVariant::Heavy));
        let (_, dropout_extended) =
            analyze_preset(RhythmPreset::Dropout120(DropoutVariant::ExtendedHeavy));
        let (_, pickup_extended) = analyze_preset(RhythmPreset::BarTransition120(
            BarTransitionVariant::PickupExtended,
        ));
        let (_, sustained_reset) = analyze_preset(RhythmPreset::BarTransition120(
            BarTransitionVariant::ReentryAcceleratingHarmonySustainedReset,
        ));
        let (_, long_sustained_reset) = analyze_preset(RhythmPreset::BarTransition120(
            BarTransitionVariant::ReentryAcceleratingHarmonyLongSustainedReset,
        ));
        let (_, mixed_length) = analyze_preset(RhythmPreset::BarTransition120(
            BarTransitionVariant::MixedLength,
        ));

        assert_eq!(
            structured.meter_state.continuity.bar_length.severity,
            super::MeterContinuitySeverity::Confirmed
        );
        assert_eq!(
            structured.meter_state.continuity.bar_length.lifecycle.decay[0].severity,
            super::MeterContinuitySeverity::Guarded
        );
        assert_eq!(
            structured.meter_state.continuity.bar_length.lifecycle.decay[1].severity,
            super::MeterContinuitySeverity::Cleared
        );

        assert_eq!(
            weak_backbeat.meter_state.continuity.bar_length.severity,
            super::MeterContinuitySeverity::Guarded
        );
        assert_eq!(
            weak_backbeat
                .meter_state
                .continuity
                .bar_length
                .lifecycle
                .decay[0]
                .severity,
            super::MeterContinuitySeverity::Fragile
        );
        assert_eq!(
            weak_backbeat
                .meter_state
                .continuity
                .bar_length
                .lifecycle
                .decay[1]
                .severity,
            super::MeterContinuitySeverity::Cleared
        );

        assert_eq!(
            dropout_heavy.meter_state.continuity.bar_length.severity,
            super::MeterContinuitySeverity::Fragile
        );
        assert_eq!(
            dropout_heavy.meter_state.continuity.downbeat_phase.severity,
            super::MeterContinuitySeverity::Fragile
        );
        assert_eq!(
            dropout_extended.meter_state.continuity.bar_length.severity,
            super::MeterContinuitySeverity::Guarded
        );
        assert_eq!(
            dropout_extended
                .meter_state
                .continuity
                .downbeat_phase
                .severity,
            super::MeterContinuitySeverity::Fragile
        );

        assert_eq!(
            pickup_extended
                .meter_state
                .continuity
                .downbeat_phase
                .severity,
            super::MeterContinuitySeverity::Fragile
        );
        assert_eq!(
            pickup_extended
                .meter_state
                .continuity
                .downbeat_phase
                .lifecycle
                .refresh
                .severity,
            super::MeterContinuitySeverity::Confirmed
        );

        assert_eq!(
            sustained_reset.meter_state.continuity.bar_length.severity,
            super::MeterContinuitySeverity::Guarded
        );
        assert_eq!(
            long_sustained_reset
                .meter_state
                .continuity
                .bar_length
                .severity,
            super::MeterContinuitySeverity::Guarded
        );
        assert_eq!(
            long_sustained_reset
                .meter_state
                .continuity
                .bar_length
                .lifecycle
                .decay[0]
                .severity,
            super::MeterContinuitySeverity::Fragile
        );
        assert_eq!(
            mixed_length.meter_state.continuity.bar_length.severity,
            super::MeterContinuitySeverity::Cleared
        );
        assert_eq!(
            mixed_length
                .meter_state
                .continuity
                .downbeat_phase
                .lifecycle
                .refresh
                .severity,
            super::MeterContinuitySeverity::Cleared
        );
    }

    #[test]
    fn beat_tracker_calibrates_meter_continuity_reason_and_confidence_surface() {
        let (_, structured) = analyze_preset(RhythmPreset::StructuredHarmony120(
            HarmonicRhythmVariant::Active,
        ));
        let (_, weak_backbeat) = analyze_preset(RhythmPreset::WeakBackbeat118);
        let (_, dropout_heavy) = analyze_preset(RhythmPreset::Dropout120(DropoutVariant::Heavy));
        let (_, dropout_extended) =
            analyze_preset(RhythmPreset::Dropout120(DropoutVariant::ExtendedHeavy));
        let (_, pickup_extended) = analyze_preset(RhythmPreset::BarTransition120(
            BarTransitionVariant::PickupExtended,
        ));
        let (_, sustained_reset) = analyze_preset(RhythmPreset::BarTransition120(
            BarTransitionVariant::ReentryAcceleratingHarmonySustainedReset,
        ));
        let (_, long_sustained_reset) = analyze_preset(RhythmPreset::BarTransition120(
            BarTransitionVariant::ReentryAcceleratingHarmonyLongSustainedReset,
        ));
        let (_, mixed_length) = analyze_preset(RhythmPreset::BarTransition120(
            BarTransitionVariant::MixedLength,
        ));

        assert_eq!(
            structured.meter_state.continuity.bar_length.reason,
            super::MeterContinuityReason::StableEvidence
        );
        assert_eq!(
            structured
                .meter_state
                .continuity
                .bar_length
                .lifecycle
                .refresh
                .reason,
            super::MeterContinuityReason::StableEvidence
        );
        assert_eq!(
            structured.meter_state.continuity.bar_length.lifecycle.decay[0].reason,
            super::MeterContinuityReason::RevalidationDecay
        );

        assert_eq!(
            weak_backbeat.meter_state.continuity.bar_length.reason,
            super::MeterContinuityReason::TentativeEvidence
        );
        assert_eq!(
            weak_backbeat
                .meter_state
                .continuity
                .bar_length
                .lifecycle
                .decay[0]
                .reason,
            super::MeterContinuityReason::RevalidationDecay
        );

        assert_eq!(
            dropout_heavy.meter_state.continuity.bar_length.reason,
            super::MeterContinuityReason::PriorStateCarry
        );
        assert_eq!(
            dropout_heavy
                .meter_state
                .continuity
                .bar_length
                .lifecycle
                .decay[0]
                .reason,
            super::MeterContinuityReason::RevalidationDecay
        );

        assert_eq!(
            dropout_extended.meter_state.continuity.bar_length.reason,
            super::MeterContinuityReason::RecoveryWindowSupport
        );
        assert_eq!(
            sustained_reset.meter_state.continuity.bar_length.reason,
            super::MeterContinuityReason::RecoveryWindowSupport
        );
        assert_eq!(
            sustained_reset
                .meter_state
                .continuity
                .bar_length
                .lifecycle
                .refresh
                .reason,
            super::MeterContinuityReason::StableEvidence
        );

        assert_eq!(
            pickup_extended.meter_state.continuity.downbeat_phase.reason,
            super::MeterContinuityReason::PhaseDisplacement
        );
        assert_eq!(
            pickup_extended
                .meter_state
                .continuity
                .downbeat_phase
                .lifecycle
                .refresh
                .reason,
            super::MeterContinuityReason::StableEvidence
        );

        assert_eq!(
            mixed_length.meter_state.continuity.bar_length.reason,
            super::MeterContinuityReason::InsufficientEvidence
        );
        assert_eq!(
            mixed_length.meter_state.continuity.bar_length.confidence.0,
            0.0
        );

        assert!(
            structured.meter_state.continuity.bar_length.confidence.0
                > weak_backbeat.meter_state.continuity.bar_length.confidence.0
        );
        assert!(
            weak_backbeat.meter_state.continuity.bar_length.confidence.0
                > weak_backbeat
                    .meter_state
                    .continuity
                    .downbeat_phase
                    .confidence
                    .0
        );
        assert!(
            dropout_extended
                .meter_state
                .continuity
                .bar_length
                .confidence
                .0
                > dropout_heavy.meter_state.continuity.bar_length.confidence.0
        );
        assert!(
            pickup_extended
                .meter_state
                .continuity
                .downbeat_phase
                .lifecycle
                .refresh
                .confidence
                .0
                > pickup_extended
                    .meter_state
                    .continuity
                    .downbeat_phase
                    .confidence
                    .0
        );
        assert!(
            sustained_reset
                .meter_state
                .continuity
                .bar_length
                .lifecycle
                .refresh
                .confidence
                .0
                > sustained_reset
                    .meter_state
                    .continuity
                    .bar_length
                    .lifecycle
                    .decay[0]
                    .confidence
                    .0
        );
        assert!(
            long_sustained_reset
                .meter_state
                .continuity
                .bar_length
                .confidence
                .0
                >= sustained_reset
                    .meter_state
                    .continuity
                    .bar_length
                    .confidence
                    .0
        );
        assert!(
            dropout_heavy
                .meter_state
                .continuity
                .bar_length
                .lifecycle
                .decay[0]
                .confidence
                .0
                > dropout_heavy
                    .meter_state
                    .continuity
                    .bar_length
                    .lifecycle
                    .decay[1]
                    .confidence
                    .0
        );
    }

    #[test]
    fn beat_tracker_calibrates_meter_continuity_triggers_and_unresolved_spans() {
        let (_, structured) = analyze_preset(RhythmPreset::StructuredHarmony120(
            HarmonicRhythmVariant::Active,
        ));
        let (_, weak_backbeat) = analyze_preset(RhythmPreset::WeakBackbeat118);
        let (_, pickup) =
            analyze_preset(RhythmPreset::BarTransition120(BarTransitionVariant::Pickup));
        let (_, pickup_extended) = analyze_preset(RhythmPreset::BarTransition120(
            BarTransitionVariant::PickupExtended,
        ));
        let (_, dropout_heavy) = analyze_preset(RhythmPreset::Dropout120(DropoutVariant::Heavy));
        let (_, dropout_extended) =
            analyze_preset(RhythmPreset::Dropout120(DropoutVariant::ExtendedHeavy));
        let (_, sustained_reset) = analyze_preset(RhythmPreset::BarTransition120(
            BarTransitionVariant::ReentryAcceleratingHarmonySustainedReset,
        ));
        let (_, long_sustained_reset) = analyze_preset(RhythmPreset::BarTransition120(
            BarTransitionVariant::ReentryAcceleratingHarmonyLongSustainedReset,
        ));
        let (_, mixed_length) = analyze_preset(RhythmPreset::BarTransition120(
            BarTransitionVariant::MixedLength,
        ));

        assert_eq!(
            structured.meter_state.continuity.bar_length.trigger,
            super::MeterContinuityTrigger::StableRevalidation
        );
        assert_eq!(
            structured
                .meter_state
                .continuity
                .bar_length
                .unresolved
                .failed_revalidations,
            0
        );
        assert_eq!(
            structured
                .meter_state
                .continuity
                .bar_length
                .unresolved
                .beats,
            0
        );

        assert_eq!(
            weak_backbeat.meter_state.continuity.bar_length.trigger,
            super::MeterContinuityTrigger::TentativeCarry
        );
        assert!(
            weak_backbeat
                .meter_state
                .continuity
                .bar_length
                .unresolved
                .failed_revalidations
                >= 1
        );

        assert_eq!(
            pickup.meter_state.continuity.downbeat_phase.trigger,
            super::MeterContinuityTrigger::PhaseRecovery
        );
        assert_eq!(
            pickup_extended
                .meter_state
                .continuity
                .downbeat_phase
                .trigger,
            super::MeterContinuityTrigger::PhaseRecovery
        );
        assert!(
            pickup_extended
                .meter_state
                .continuity
                .downbeat_phase
                .lifecycle
                .decay[1]
                .unresolved
                .beats
                > pickup
                    .meter_state
                    .continuity
                    .downbeat_phase
                    .unresolved
                    .beats
        );
        assert!(
            pickup_extended
                .meter_state
                .continuity
                .downbeat_phase
                .lifecycle
                .decay[1]
                .unresolved
                .failed_revalidations
                > pickup_extended
                    .meter_state
                    .continuity
                    .downbeat_phase
                    .lifecycle
                    .decay[0]
                    .unresolved
                    .failed_revalidations
        );

        assert_eq!(
            dropout_heavy.meter_state.continuity.bar_length.trigger,
            super::MeterContinuityTrigger::PriorStateDrift
        );
        assert_eq!(
            dropout_extended.meter_state.continuity.bar_length.trigger,
            super::MeterContinuityTrigger::RecoveryWindowDrift
        );
        assert!(
            dropout_heavy
                .meter_state
                .continuity
                .bar_length
                .unresolved
                .failed_revalidations
                >= 1
        );
        assert!(
            dropout_extended
                .meter_state
                .continuity
                .bar_length
                .lifecycle
                .decay[1]
                .unresolved
                .failed_revalidations
                > dropout_extended
                    .meter_state
                    .continuity
                    .bar_length
                    .lifecycle
                    .decay[0]
                    .unresolved
                    .failed_revalidations
        );

        assert_eq!(
            sustained_reset.meter_state.continuity.bar_length.trigger,
            super::MeterContinuityTrigger::RecoveryWindowDrift
        );
        assert_eq!(
            long_sustained_reset
                .meter_state
                .continuity
                .bar_length
                .trigger,
            super::MeterContinuityTrigger::RecoveryWindowDrift
        );
        assert!(
            long_sustained_reset
                .meter_state
                .continuity
                .bar_length
                .unresolved
                .beats
                >= sustained_reset
                    .meter_state
                    .continuity
                    .bar_length
                    .unresolved
                    .beats
        );
        assert!(
            long_sustained_reset
                .meter_state
                .continuity
                .bar_length
                .unresolved
                .failed_revalidations
                >= sustained_reset
                    .meter_state
                    .continuity
                    .bar_length
                    .unresolved
                    .failed_revalidations
        );

        assert_eq!(
            mixed_length.meter_state.continuity.bar_length.trigger,
            super::MeterContinuityTrigger::EvidenceLoss
        );
        assert_eq!(
            mixed_length
                .meter_state
                .continuity
                .bar_length
                .unresolved
                .beats,
            0
        );
    }

    #[test]
    fn beat_tracker_calibrates_meter_continuity_cause_stacks_for_stacked_instability() {
        let contains_cause = |stack: super::MeterContinuityCauseStack,
                              cause: super::MeterContinuityCause| {
            stack.primary == cause
                || stack
                    .secondary
                    .into_iter()
                    .flatten()
                    .any(|entry| entry == cause)
        };

        let (_, ambiguous) = analyze_preset(RhythmPreset::AmbiguousSubdivision90);
        let (_, pickup_extended) = analyze_preset(RhythmPreset::BarTransition120(
            BarTransitionVariant::PickupExtended,
        ));
        let (_, dropout_extended) =
            analyze_preset(RhythmPreset::Dropout120(DropoutVariant::ExtendedHeavy));
        let (_, modulation_extended) = analyze_preset(RhythmPreset::BarTransition120(
            BarTransitionVariant::ModulationDenseFillExtended,
        ));

        assert!(contains_cause(
            ambiguous.meter_state.continuity.bar_length.causes,
            super::MeterContinuityCause::EvidenceLoss,
        ));
        assert!(contains_cause(
            ambiguous.meter_state.continuity.bar_length.causes,
            super::MeterContinuityCause::TempoAmbiguity,
        ));
        assert!(contains_cause(
            ambiguous.meter_state.continuity.bar_length.causes,
            super::MeterContinuityCause::SparseMeterSupport,
        ));
        assert!(ambiguous.meter_state.continuity.bar_length.causes.count >= 2);

        assert!(contains_cause(
            pickup_extended.meter_state.continuity.downbeat_phase.causes,
            super::MeterContinuityCause::PhaseDisplacement,
        ));
        assert!(contains_cause(
            pickup_extended
                .meter_state
                .continuity
                .downbeat_phase
                .lifecycle
                .decay[1]
                .causes,
            super::MeterContinuityCause::EvidenceLoss,
        ));
        assert!(
            pickup_extended
                .meter_state
                .continuity
                .downbeat_phase
                .lifecycle
                .decay[1]
                .causes
                .count
                >= 2
        );

        assert!(contains_cause(
            dropout_extended.meter_state.continuity.bar_length.causes,
            super::MeterContinuityCause::RecoveryWindowInstability,
        ));
        assert!(contains_cause(
            dropout_extended.meter_state.continuity.bar_length.causes,
            super::MeterContinuityCause::TempoAmbiguity,
        ));
        assert!(contains_cause(
            dropout_extended.meter_state.continuity.bar_length.causes,
            super::MeterContinuityCause::IrregularBarStructure,
        ));
        assert!(
            dropout_extended
                .meter_state
                .continuity
                .bar_length
                .causes
                .count
                >= 2
        );

        assert!(contains_cause(
            modulation_extended.meter_state.continuity.bar_length.causes,
            super::MeterContinuityCause::EvidenceLoss,
        ));
        assert!(contains_cause(
            modulation_extended.meter_state.continuity.bar_length.causes,
            super::MeterContinuityCause::TempoAmbiguity,
        ));
        assert!(
            modulation_extended
                .meter_state
                .continuity
                .bar_length
                .causes
                .count
                >= 2
        );
    }

    #[test]
    fn beat_tracker_calibrates_meter_continuity_history_across_transition_families() {
        let (_, structured) = analyze_preset(RhythmPreset::StructuredHarmony120(
            HarmonicRhythmVariant::Active,
        ));
        let (_, weak_backbeat) = analyze_preset(RhythmPreset::WeakBackbeat118);
        let (_, pickup_extended) = analyze_preset(RhythmPreset::BarTransition120(
            BarTransitionVariant::PickupExtended,
        ));
        let (_, dropout_extended) =
            analyze_preset(RhythmPreset::Dropout120(DropoutVariant::ExtendedHeavy));
        let (_, sustained_reset) = analyze_preset(RhythmPreset::BarTransition120(
            BarTransitionVariant::ReentryAcceleratingHarmonySustainedReset,
        ));
        let (_, long_sustained_reset) = analyze_preset(RhythmPreset::BarTransition120(
            BarTransitionVariant::ReentryAcceleratingHarmonyLongSustainedReset,
        ));
        let (_, modulation_extended) = analyze_preset(RhythmPreset::BarTransition120(
            BarTransitionVariant::ModulationDenseFillExtended,
        ));

        assert_eq!(
            structured.meter_state.continuity.bar_length.history,
            super::MeterContinuityHistory::Reinforcing
        );
        assert_eq!(
            structured
                .meter_state
                .continuity
                .bar_length
                .lifecycle
                .refresh
                .history,
            super::MeterContinuityHistory::Reinforcing
        );

        assert_eq!(
            weak_backbeat.meter_state.continuity.bar_length.history,
            super::MeterContinuityHistory::Preserving
        );
        assert_eq!(
            weak_backbeat.meter_state.continuity.downbeat_phase.history,
            super::MeterContinuityHistory::Degrading
        );

        assert_eq!(
            pickup_extended
                .meter_state
                .continuity
                .downbeat_phase
                .history,
            super::MeterContinuityHistory::Degrading
        );
        assert_eq!(
            pickup_extended
                .meter_state
                .continuity
                .downbeat_phase
                .lifecycle
                .decay[1]
                .history,
            super::MeterContinuityHistory::Degrading
        );

        assert_eq!(
            dropout_extended.meter_state.continuity.bar_length.history,
            super::MeterContinuityHistory::Preserving
        );
        assert_eq!(
            dropout_extended
                .meter_state
                .continuity
                .bar_length
                .lifecycle
                .decay[0]
                .history,
            super::MeterContinuityHistory::Degrading
        );

        assert_eq!(
            sustained_reset.meter_state.continuity.bar_length.history,
            super::MeterContinuityHistory::Preserving
        );
        assert_eq!(
            long_sustained_reset
                .meter_state
                .continuity
                .bar_length
                .history,
            super::MeterContinuityHistory::Preserving
        );
        assert_eq!(
            sustained_reset
                .meter_state
                .continuity
                .bar_length
                .lifecycle
                .refresh
                .history,
            super::MeterContinuityHistory::Reinforcing
        );
        assert_eq!(
            long_sustained_reset
                .meter_state
                .continuity
                .bar_length
                .lifecycle
                .refresh
                .history,
            super::MeterContinuityHistory::Reinforcing
        );

        assert_eq!(
            modulation_extended
                .meter_state
                .continuity
                .bar_length
                .history,
            super::MeterContinuityHistory::Degrading
        );
    }

    #[test]
    fn beat_tracker_calibrates_meter_continuity_arcs_across_transition_families() {
        let (_, structured) = analyze_preset(RhythmPreset::StructuredHarmony120(
            HarmonicRhythmVariant::Active,
        ));
        let (_, weak_backbeat) = analyze_preset(RhythmPreset::WeakBackbeat118);
        let (_, ambiguous) = analyze_preset(RhythmPreset::AmbiguousSubdivision90);
        let (_, pickup_extended) = analyze_preset(RhythmPreset::BarTransition120(
            BarTransitionVariant::PickupExtended,
        ));
        let (_, dropout_extended) =
            analyze_preset(RhythmPreset::Dropout120(DropoutVariant::ExtendedHeavy));
        let (_, sustained_reset) = analyze_preset(RhythmPreset::BarTransition120(
            BarTransitionVariant::ReentryAcceleratingHarmonySustainedReset,
        ));
        let (_, long_sustained_reset) = analyze_preset(RhythmPreset::BarTransition120(
            BarTransitionVariant::ReentryAcceleratingHarmonyLongSustainedReset,
        ));
        let (_, modulation_extended) = analyze_preset(RhythmPreset::BarTransition120(
            BarTransitionVariant::ModulationDenseFillExtended,
        ));

        assert_eq!(
            structured.meter_state.continuity.bar_length.arc,
            super::MeterContinuityArc::Recovering
        );
        assert_eq!(
            weak_backbeat.meter_state.continuity.bar_length.arc,
            super::MeterContinuityArc::Stalling
        );
        assert_eq!(
            ambiguous.meter_state.continuity.bar_length.arc,
            super::MeterContinuityArc::Collapsing
        );
        assert_eq!(
            pickup_extended.meter_state.continuity.downbeat_phase.arc,
            super::MeterContinuityArc::Collapsing
        );
        assert_eq!(
            dropout_extended.meter_state.continuity.bar_length.arc,
            super::MeterContinuityArc::Stalling
        );
        assert_eq!(
            sustained_reset.meter_state.continuity.bar_length.arc,
            super::MeterContinuityArc::Recovering
        );
        assert_eq!(
            long_sustained_reset.meter_state.continuity.bar_length.arc,
            super::MeterContinuityArc::Recovering
        );
        assert_eq!(
            modulation_extended.meter_state.continuity.bar_length.arc,
            super::MeterContinuityArc::Collapsing
        );
    }

    #[test]
    fn beat_tracker_calibrates_meter_continuity_arc_rationales_and_support() {
        let (_, structured) = analyze_preset(RhythmPreset::StructuredHarmony120(
            HarmonicRhythmVariant::Active,
        ));
        let (_, weak_backbeat) = analyze_preset(RhythmPreset::WeakBackbeat118);
        let (_, ambiguous) = analyze_preset(RhythmPreset::AmbiguousSubdivision90);
        let (_, pickup_extended) = analyze_preset(RhythmPreset::BarTransition120(
            BarTransitionVariant::PickupExtended,
        ));
        let (_, dropout_extended) =
            analyze_preset(RhythmPreset::Dropout120(DropoutVariant::ExtendedHeavy));
        let (_, sustained_reset) = analyze_preset(RhythmPreset::BarTransition120(
            BarTransitionVariant::ReentryAcceleratingHarmonySustainedReset,
        ));
        let (_, long_sustained_reset) = analyze_preset(RhythmPreset::BarTransition120(
            BarTransitionVariant::ReentryAcceleratingHarmonyLongSustainedReset,
        ));
        let (_, modulation_extended) = analyze_preset(RhythmPreset::BarTransition120(
            BarTransitionVariant::ModulationDenseFillExtended,
        ));

        assert_eq!(
            structured.meter_state.continuity.bar_length.arc_rationale,
            super::MeterContinuityArcRationale::RefreshStrength
        );
        assert!(
            structured
                .meter_state
                .continuity
                .bar_length
                .arc_support
                .refresh_strength
                .0
                > structured
                    .meter_state
                    .continuity
                    .bar_length
                    .arc_support
                    .drift_pressure
                    .0
        );

        assert_eq!(
            weak_backbeat
                .meter_state
                .continuity
                .bar_length
                .arc_rationale,
            super::MeterContinuityArcRationale::UnresolvedDrift
        );
        assert!(
            weak_backbeat
                .meter_state
                .continuity
                .bar_length
                .arc_support
                .drift_pressure
                .0
                > weak_backbeat
                    .meter_state
                    .continuity
                    .bar_length
                    .arc_support
                    .refresh_strength
                    .0
        );

        assert_eq!(
            ambiguous.meter_state.continuity.bar_length.arc_rationale,
            super::MeterContinuityArcRationale::EvidenceLoss
        );
        assert!(
            ambiguous
                .meter_state
                .continuity
                .bar_length
                .arc_support
                .structural_pressure
                .0
                >= 0.5
        );

        assert_eq!(
            pickup_extended
                .meter_state
                .continuity
                .downbeat_phase
                .arc_rationale,
            super::MeterContinuityArcRationale::UnresolvedDrift
        );
        assert!(
            pickup_extended
                .meter_state
                .continuity
                .downbeat_phase
                .arc_support
                .refresh_strength
                .0
                > 0.8
        );
        assert!(
            pickup_extended
                .meter_state
                .continuity
                .downbeat_phase
                .arc_support
                .drift_pressure
                .0
                > pickup_extended
                    .meter_state
                    .continuity
                    .downbeat_phase
                    .arc_support
                    .structural_pressure
                    .0
        );

        assert_eq!(
            dropout_extended
                .meter_state
                .continuity
                .bar_length
                .arc_rationale,
            super::MeterContinuityArcRationale::StructuralInstability
        );
        assert!(
            dropout_extended
                .meter_state
                .continuity
                .bar_length
                .arc_support
                .structural_pressure
                .0
                > sustained_reset
                    .meter_state
                    .continuity
                    .bar_length
                    .arc_support
                    .structural_pressure
                    .0
        );

        assert_eq!(
            sustained_reset
                .meter_state
                .continuity
                .bar_length
                .arc_rationale,
            super::MeterContinuityArcRationale::RefreshStrength
        );
        assert_eq!(
            long_sustained_reset
                .meter_state
                .continuity
                .bar_length
                .arc_rationale,
            super::MeterContinuityArcRationale::RefreshStrength
        );
        assert!(
            long_sustained_reset
                .meter_state
                .continuity
                .bar_length
                .arc_support
                .refresh_strength
                .0
                >= sustained_reset
                    .meter_state
                    .continuity
                    .bar_length
                    .arc_support
                    .refresh_strength
                    .0
        );

        assert_eq!(
            modulation_extended
                .meter_state
                .continuity
                .bar_length
                .arc_rationale,
            super::MeterContinuityArcRationale::EvidenceLoss
        );
        assert!(
            modulation_extended
                .meter_state
                .continuity
                .bar_length
                .arc_support
                .structural_pressure
                .0
                >= ambiguous
                    .meter_state
                    .continuity
                    .bar_length
                    .arc_support
                    .structural_pressure
                    .0
        );
    }

    #[test]
    fn non_native_input_rate_preserves_click_track_tempo_under_frozen_analysis_rate() {
        let native = click_track(48_000, 120.0, 8.0);
        let non_native = click_track(44_100, 120.0, 8.0);
        let mut tracker = BeatTracker::new(BeatTrackerConfig::default());

        let native_result = tracker.analyze(&native);
        let non_native_result = tracker.analyze(&non_native);

        assert!((native_result.bpm - 120.0).abs() < 1.0);
        assert!((non_native_result.bpm - 120.0).abs() < 1.0);
        assert!((native_result.bpm - non_native_result.bpm).abs() < 0.5);
        assert!(
            (native_result.confidence.0 - non_native_result.confidence.0).abs() < 0.1,
            "confidence drifted from {} to {}",
            native_result.confidence.0,
            non_native_result.confidence.0,
        );
    }

    #[test]
    fn harness_rhythm_cases_meet_frozen_acceptance_thresholds() {
        let cases = rhythm_acceptance_cases();
        let mut tracker = BeatTracker::new(BeatTrackerConfig::default());

        let report =
            run_audio_acceptance_harness(&cases, |audio| tracker.analyze(audio), rhythm_metrics);

        assert_eq!(report.status, AcceptanceStatus::Pass);
        assert!(report
            .cases
            .iter()
            .all(|case| case.status == AcceptanceStatus::Pass));
    }

    #[test]
    fn frozen_rhythm_acceptance_report_remains_interpretable_for_closeout() {
        let cases = rhythm_acceptance_cases();
        let mut tracker = BeatTracker::new(BeatTrackerConfig::default());

        let report =
            run_audio_acceptance_harness(&cases, |audio| tracker.analyze(audio), rhythm_metrics);

        println!("rhythm_acceptance_report={:#?}", report);

        assert_eq!(report.status, AcceptanceStatus::Pass);
        assert_eq!(report.cases.len(), 3);
    }
}
