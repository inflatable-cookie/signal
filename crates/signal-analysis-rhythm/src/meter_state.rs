#[derive(Clone, Copy, Debug)]
pub(crate) struct MeterSuppressionProfile {
    pub best_confidence: Confidence,
    pub best_support: f32,
    pub best_regularity: f32,
    pub trailing_confidence: Confidence,
    pub trailing_recent_stability: f32,
}

pub(crate) struct MeterDecision {
    pub estimate: Option<MeterEstimate>,
    pub suppression_profile: MeterSuppressionProfile,
    pub ambiguity: RhythmStructureAmbiguitySummary,
}

use crate::beat_tempo_core::beat_frames_to_seconds;
use crate::rhythm_policy::*;
use crate::{downbeat_frames_for_hypothesis, neighborhood_peak};
use signal_analysis::Confidence;
use signal_primitives::SampleRate;

mod meter_state_continuity_helpers;
mod meter_state_continuity_types;
use meter_state_continuity_helpers::*;
mod meter_state_continuity_arc;
mod meter_state_continuity_core;
use meter_state_continuity_core::*;

pub(crate) fn meter_state_recommendation(
    estimate: Option<&MeterEstimate>,
    suppression_profile: MeterSuppressionProfile,
    rhythm_confidence: Confidence,
    tempo_ambiguity: Confidence,
    bpm: f32,
    beat_positions_seconds: &[f32],
) -> MeterStateRecommendation {
    if let Some(estimate) = estimate {
        return match estimate.recommendation {
            MeterRecommendation::Lock => build_meter_state(
                MeterStateAction::Lock,
                MeterStateReason::StableMeter,
                MeterContinuityInputs {
                    estimate: Some(estimate),
                    suppression_profile,
                    confidence: estimate.confidence,
                    tempo_ambiguity,
                    bpm,
                    beat_positions_seconds,
                },
            ),
            MeterRecommendation::Monitor if estimate.trust == MeterTrustLevel::Recovering => {
                build_meter_state(
                    MeterStateAction::Watch,
                    MeterStateReason::RecoveringMeter,
                    MeterContinuityInputs {
                        estimate: Some(estimate),
                        suppression_profile,
                        confidence: Confidence::new(
                            0.5 * estimate.support_profile.segment_recovery_strength.0
                                + 0.3 * estimate.support_profile.recovery_duration_strength.0
                                + 0.2 * estimate.confidence.0,
                        ),
                        tempo_ambiguity,
                        bpm,
                        beat_positions_seconds,
                    },
                )
            }
            MeterRecommendation::Monitor | MeterRecommendation::Defer => build_meter_state(
                MeterStateAction::Hold,
                MeterStateReason::TentativeMeter,
                MeterContinuityInputs {
                    estimate: Some(estimate),
                    suppression_profile,
                    confidence: Confidence::new(
                        0.6 * estimate.confidence.0
                            + 0.4 * estimate.support_profile.whole_track_strength.0,
                    ),
                    tempo_ambiguity,
                    bpm,
                    beat_positions_seconds,
                },
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
                MeterContinuityInputs {
                    estimate: None,
                    suppression_profile,
                    confidence: Confidence::new(
                        (0.5 * tempo_ambiguity.0
                            + 0.3 * trailing_recovery_strength
                            + 0.2 * (1.0 - suppression_profile.best_support.clamp(0.0, 1.0)))
                        .clamp(0.0, 1.0),
                    ),
                    tempo_ambiguity,
                    bpm,
                    beat_positions_seconds,
                },
            );
        }

        if tempo_ambiguity.0 <= 0.33 {
            return build_meter_state(
                MeterStateAction::Hold,
                MeterStateReason::DestabilizedHold,
                MeterContinuityInputs {
                    estimate: None,
                    suppression_profile,
                    confidence: Confidence::new(
                        (0.55 * pulse_stability
                            + 0.25 * suppression_profile.best_confidence.0
                            + 0.20 * suppression_profile.best_support)
                            .clamp(0.0, 1.0),
                    ),
                    tempo_ambiguity,
                    bpm,
                    beat_positions_seconds,
                },
            );
        }

        build_meter_state(
            MeterStateAction::Watch,
            MeterStateReason::RecoveryEmerging,
            MeterContinuityInputs {
                estimate: None,
                suppression_profile,
                confidence: Confidence::new(
                    (0.55 * trailing_recovery_strength + 0.45 * pulse_stability).clamp(0.0, 1.0),
                ),
                tempo_ambiguity,
                bpm,
                beat_positions_seconds,
            },
        )
    } else if pulse_stability >= 0.55
        && suppression_profile.best_confidence.0 >= 0.12
        && suppression_profile.best_support >= 0.48
        && suppression_profile.best_regularity >= 0.20
    {
        build_meter_state(
            MeterStateAction::Hold,
            MeterStateReason::DestabilizedHold,
            MeterContinuityInputs {
                estimate: None,
                suppression_profile,
                confidence: Confidence::new(
                    (0.5 * pulse_stability
                        + 0.3 * suppression_profile.best_confidence.0
                        + 0.2 * suppression_profile.best_support)
                        .clamp(0.0, 1.0),
                ),
                tempo_ambiguity,
                bpm,
                beat_positions_seconds,
            },
        )
    } else {
        build_meter_state(
            MeterStateAction::Clear,
            MeterStateReason::MeterCleared,
            MeterContinuityInputs {
                estimate: None,
                suppression_profile,
                confidence: Confidence::new(
                    (0.45 * (1.0 - suppression_profile.best_confidence.0)
                        + 0.35 * (1.0 - suppression_profile.best_support.clamp(0.0, 1.0))
                        + 0.20 * tempo_ambiguity.0)
                        .clamp(0.0, 1.0),
                ),
                tempo_ambiguity,
                bpm,
                beat_positions_seconds,
            },
        )
    }
}

pub(crate) fn infer_meter(
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
