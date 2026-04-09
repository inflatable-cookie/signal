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

use crate::rhythm_policy::*;
use signal_analysis::Confidence;

mod meter_state_continuity_helpers;
mod meter_state_continuity_types;
use meter_state_continuity_helpers::*;
mod meter_state_continuity_arc;
mod meter_state_continuity_cause_stack;
mod meter_state_continuity_context;
mod meter_state_continuity_core;
mod meter_state_continuity_hold_arms;
mod meter_state_continuity_lock_arms;
mod meter_state_continuity_plan_shell;
mod meter_state_continuity_rule_surface;
mod meter_state_continuity_watch_clear_arms;
use meter_state_continuity_core::*;
mod meter_state_infer;
pub(crate) use meter_state_infer::infer_meter;

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
