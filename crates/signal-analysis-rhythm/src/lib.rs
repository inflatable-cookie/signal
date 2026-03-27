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
mod tempo_state_continuity_basics;
mod tempo_state_continuity_refresh;
mod tempo_state_continuity_transition;
mod tempo_state;
pub use tempo_state::tempo_state_recommendation_with_scope;

use beat_tempo_core::{
    beat_frames_to_seconds, beat_frames_to_seconds_refined, combined_confidence, estimate_tempo,
    refine_beat_frames, refine_bpm_from_beats, track_beats,
};
pub use beat_utils::normalize;
#[allow(unused_imports)]
pub(crate) use beat_utils::{beat_phase_score, neighborhood_peak, refine_beat, select_beat_phase};
use meter_state::{
    infer_meter, meter_state_recommendation, MeterDecision, MeterSuppressionProfile,
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

mod rhythm_tests;
