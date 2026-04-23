use crate::tempo_policy::*;
use signal_analysis::Confidence;

mod tempo_state_arc_decision;
mod tempo_state_arc_decision_fields;
mod tempo_state_continuity_arc;
mod tempo_state_continuity_helpers;
mod tempo_state_scope_context;
use tempo_state_scope_context::TempoStateScopeContext;
mod tempo_state_snap_integer_arm;
mod tempo_state_stable_policy;
use tempo_state_snap_integer_arm::snap_integer_arm;
mod tempo_state_use_refined_stable_arm;
use tempo_state_use_refined_stable_arm::use_refined_stable_arm;
mod tempo_state_fallback_arms;
use tempo_state_fallback_arms::{defer_arm, use_core_window_arm, use_refined_guarded_arm};

/// Produce a [`TempoStateRecommendation`] from the interpretation pipeline output,
/// taking the stability scope into account when choosing the action and continuity plan.
pub fn tempo_state_recommendation_with_scope(
    interpretation: TempoInterpretation,
    confidence: Confidence,
    tempo_ambiguity: Confidence,
    stability_scope: TempoStabilityScopeSummary,
) -> TempoStateRecommendation {
    let base_confidence = (0.45 * interpretation.profile.stability_score.0
        + 0.25 * confidence.0
        + 0.15 * (1.0 - tempo_ambiguity.0)
        + 0.15 * interpretation.support.grid_stability.0)
        .clamp(0.0, 1.0);
    let strong_integer_anchor = matches!(
        interpretation.recommendation,
        TempoRecommendation::SnapInteger
    ) && interpretation.support.integer_closeness.0 > 0.85
        && interpretation.support.core_consensus.0 > 0.8
        && interpretation.support.drift_stability.0 > 0.5
        && interpretation.support.grid_stability.0 > 0.35
        && interpretation.support.boundary_pressure.0 < 0.6;
    let ambiguity_guard = tempo_ambiguity.0 < 0.4 || strong_integer_anchor;

    let ctx = TempoStateScopeContext {
        boundary_pressure: interpretation.support.boundary_pressure,
        tempo_ambiguity,
        base_confidence,
        localized_edge_scope: matches!(
            stability_scope.scope,
            TempoStabilityScope::StableWithLocalizedEdgeDamage
        ),
        core_stable_scope: matches!(stability_scope.scope, TempoStabilityScope::CoreStableOnly),
        mid_track_unstable_scope: matches!(
            stability_scope.scope,
            TempoStabilityScope::MidTrackUnstable
        ),
        strong_integer_anchor,
    };

    match interpretation.recommendation {
        TempoRecommendation::SnapInteger
            if interpretation.trust != TempoTrustLevel::Tentative
                && (interpretation.profile.stability_score.0 >= 0.78 || strong_integer_anchor)
                && (interpretation.profile.snap_error_bpm >= 0.04
                    || interpretation.support.integer_closeness.0 > 0.9)
                && ambiguity_guard =>
        {
            snap_integer_arm(ctx)
        }
        TempoRecommendation::UseRefined
            if interpretation.trust == TempoTrustLevel::Stable
                && interpretation.profile.stability_score.0 >= 0.72
                && interpretation.support.boundary_pressure.0 < 0.55
                && ambiguity_guard =>
        {
            use_refined_stable_arm(ctx)
        }
        TempoRecommendation::UseCoreWindow
            if interpretation.profile.stability_score.0 >= 0.55
                && interpretation.support.boundary_pressure.0 >= 0.45 =>
        {
            use_core_window_arm(ctx)
        }
        TempoRecommendation::UseRefined
            if interpretation.trust == TempoTrustLevel::Guarded
                && interpretation.profile.stability_score.0 >= 0.58 =>
        {
            use_refined_guarded_arm(ctx)
        }
        _ => defer_arm(ctx, 1.0 - interpretation.profile.stability_score.0),
    }
}
