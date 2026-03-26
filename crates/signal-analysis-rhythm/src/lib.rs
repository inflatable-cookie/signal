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
mod tempo_state_recommendation;
pub use tempo_state_recommendation::tempo_state_recommendation_with_scope;

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
mod rhythm_tests;
