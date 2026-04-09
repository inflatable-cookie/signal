use super::tempo_state_scope_context::TempoStateScopeContext;
use super::tempo_state_stable_policy::{
    locked_recommendation, scoped_monitor_or_defer_recommendation, TempoStateLockedPolicy,
    TempoStateSharedScopePolicy,
};
use crate::tempo_policy::*;

pub fn snap_integer_arm(ctx: TempoStateScopeContext) -> TempoStateRecommendation {
    if let Some(recommendation) = scoped_monitor_or_defer_recommendation(
        ctx,
        TempoStateSharedScopePolicy {
            core_stable_min_confidence: 0.58,
            mid_track_unstable_min_confidence: 0.48,
            lock_confidence_scale: 0.92,
            reacquire_confidence_scale: 0.64,
        },
    ) {
        return recommendation;
    }

    locked_recommendation(
        ctx,
        TempoStateLockedPolicy {
            localized_edge_min_confidence: 0.76,
            anchored_min_confidence: Some(0.80),
            default_min_confidence: 0.82,
            default_decay_confidence_scale: 0.72,
            locked_reason: TempoStateReason::StableIntegerTempo,
            continuity_reason: TempoContinuityReason::IntegerTempoSnap,
        },
    )
}
