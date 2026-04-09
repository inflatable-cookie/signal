use super::tempo_state_scope_context::TempoStateScopeContext;
use super::tempo_state_stable_policy::{
    locked_recommendation, scoped_monitor_or_defer_recommendation, TempoStateLockedPolicy,
    TempoStateSharedScopePolicy,
};
use crate::tempo_policy::*;

pub fn use_refined_stable_arm(ctx: TempoStateScopeContext) -> TempoStateRecommendation {
    if let Some(recommendation) = scoped_monitor_or_defer_recommendation(
        ctx,
        TempoStateSharedScopePolicy {
            core_stable_min_confidence: 0.56,
            mid_track_unstable_min_confidence: 0.46,
            lock_confidence_scale: 0.94,
            reacquire_confidence_scale: 0.66,
        },
    ) {
        return recommendation;
    }

    locked_recommendation(
        ctx,
        TempoStateLockedPolicy {
            localized_edge_min_confidence: 0.72,
            anchored_min_confidence: None,
            default_min_confidence: 0.76,
            default_decay_confidence_scale: 0.72,
            locked_reason: TempoStateReason::StableRefinedTempo,
            continuity_reason: TempoContinuityReason::StableTempo,
        },
    )
}
