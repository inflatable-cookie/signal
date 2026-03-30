use super::tempo_state_arc_decision_fields::arc_decision_fields;
use super::tempo_state_continuity_helpers::TempoContinuityArcDecisionInputs;
use crate::tempo_policy::*;
use signal_analysis::Confidence;

pub fn continuity_arc_decision(
    inputs: TempoContinuityArcDecisionInputs,
) -> TempoContinuityArcDecision {
    let arc = inputs.arc;
    let rationale = inputs.rationale;
    let support = inputs.support;
    let severity = inputs.severity;
    let history = inputs.history;
    let unresolved = inputs.unresolved;
    let confidence = inputs.confidence;

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
            ) = arc_decision_fields(inputs, action);
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
                TempoContinuityArc::Recovering => TempoContinuityArcAction::ReacquireCurrentTempo,
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
            ) = arc_decision_fields(inputs, action);
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
            ) = arc_decision_fields(inputs, action);
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
                            * if matches!(rationale, TempoContinuityArcRationale::EvidenceLoss) {
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
