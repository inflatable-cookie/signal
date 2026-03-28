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
mod beat_utils;
mod meter_state;
mod onset_features;
mod rhythm_policy;
mod tempo_interpretation_runtime;
mod tempo_policy;
mod tempo_state;
mod tempo_state_continuity_basics;
mod tempo_state_continuity_refresh;
mod tempo_state_continuity_transition;
mod tracker_config;
mod tracker_support;
pub use tempo_state::tempo_state_recommendation_with_scope;
pub use tracker_config::{AnalysisProfile, BeatTrackerConfig};

use beat_tempo_core::{
    beat_frames_to_seconds_refined, combined_confidence, estimate_tempo, refine_beat_frames,
    refine_bpm_from_beats, track_beats,
};
pub use beat_utils::normalize;
#[allow(unused_imports)]
pub(crate) use beat_utils::{beat_phase_score, neighborhood_peak, refine_beat, select_beat_phase};
use meter_state::{
    infer_meter, meter_state_recommendation, MeterDecision, MeterSuppressionProfile,
};
use onset_features::{band_profile_change, low_band_flux, multifeature_onset_envelope};
pub use rhythm_policy::*;
use signal_analysis::{
    prepare_audio_analysis, prepare_mono_analysis, AnalysisInputConfig, AnalysisMode,
    AnalysisStage, Confidence,
};
use signal_dsp_spectral::Stft;
use signal_primitives::{AudioBuffer, Sample, SampleRate, Seconds};
use tempo_interpretation_runtime::interpret_tempo;
pub use tempo_policy::*;
use tempo_policy::{analyze_local_tempo, tempo_summary};
use tracker_support::combine_meter_cues;
pub(crate) use tracker_support::{
    beat_index_to_seconds, downbeat_frames_for_hypothesis, TempoEstimate, TempoHypothesis,
};

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

mod rhythm_tests;
