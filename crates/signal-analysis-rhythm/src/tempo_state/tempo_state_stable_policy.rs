use super::tempo_state_continuity_arc::continuity_plan;
use super::tempo_state_continuity_helpers::TempoContinuityPlanInputs;
use super::tempo_state_scope_context::TempoStateScopeContext;
use crate::tempo_policy::*;
use crate::tempo_state_continuity_transition::{
    continuity_transition, TempoContinuityTransitionInputs,
};
use signal_analysis::Confidence;

pub struct TempoStateSharedScopePolicy {
    pub core_stable_min_confidence: f32,
    pub mid_track_unstable_min_confidence: f32,
    pub lock_confidence_scale: f32,
    pub reacquire_confidence_scale: f32,
}

pub struct TempoStateLockedPolicy {
    pub localized_edge_min_confidence: f32,
    pub anchored_min_confidence: Option<f32>,
    pub default_min_confidence: f32,
    pub default_decay_confidence_scale: f32,
    pub locked_reason: TempoStateReason,
    pub continuity_reason: TempoContinuityReason,
}

pub fn scoped_monitor_or_defer_recommendation(
    ctx: TempoStateScopeContext,
    policy: TempoStateSharedScopePolicy,
) -> Option<TempoStateRecommendation> {
    if !(ctx.core_stable_scope || ctx.mid_track_unstable_scope) {
        return None;
    }

    let state_confidence = Confidence::new(ctx.base_confidence.max(if ctx.core_stable_scope {
        policy.core_stable_min_confidence
    } else {
        policy.mid_track_unstable_min_confidence
    }));

    Some(TempoStateRecommendation {
        action: if ctx.core_stable_scope {
            TempoStateAction::Monitor
        } else {
            TempoStateAction::Defer
        },
        reason: if ctx.core_stable_scope {
            TempoStateReason::CoreStableTempo
        } else {
            TempoStateReason::TempoDeferred
        },
        confidence: state_confidence,
        continuity: continuity_plan(
            TempoContinuityPlanInputs {
                action: if ctx.core_stable_scope {
                    TempoContinuityAction::Reacquire
                } else {
                    TempoContinuityAction::Clear
                },
                source: if ctx.core_stable_scope {
                    TempoContinuitySource::CurrentTempo
                } else {
                    TempoContinuitySource::Cleared
                },
                reason: if ctx.core_stable_scope {
                    TempoContinuityReason::RevalidationDecay
                } else {
                    TempoContinuityReason::InsufficientEvidence
                },
                boundary_pressure: ctx.boundary_pressure,
                tempo_ambiguity: ctx.tempo_ambiguity,
                confidence: state_confidence,
                trusted_beats: if ctx.core_stable_scope { 4 } else { 0 },
                revalidate_after_beats: if ctx.core_stable_scope { 4 } else { 0 },
            },
            continuity_transition(TempoContinuityTransitionInputs {
                after_beats: if ctx.core_stable_scope { 4 } else { 0 },
                action: if ctx.core_stable_scope {
                    TempoContinuityAction::Lock
                } else {
                    TempoContinuityAction::Clear
                },
                source: if ctx.core_stable_scope {
                    TempoContinuitySource::CurrentTempo
                } else {
                    TempoContinuitySource::Cleared
                },
                reason: if ctx.core_stable_scope {
                    TempoContinuityReason::StableTempo
                } else {
                    TempoContinuityReason::InsufficientEvidence
                },
                boundary_pressure: ctx.boundary_pressure,
                tempo_ambiguity: ctx.tempo_ambiguity,
                revalidate_after_beats: if ctx.core_stable_scope { 4 } else { 0 },
                stage_index: 0,
                confidence: if ctx.core_stable_scope {
                    Confidence::new(
                        (state_confidence.0 * policy.lock_confidence_scale).clamp(0.0, 1.0),
                    )
                } else {
                    Confidence::new(0.0)
                },
            }),
            continuity_transition(TempoContinuityTransitionInputs {
                after_beats: if ctx.core_stable_scope { 8 } else { 0 },
                action: if ctx.core_stable_scope {
                    TempoContinuityAction::Reacquire
                } else {
                    TempoContinuityAction::Clear
                },
                source: if ctx.core_stable_scope {
                    TempoContinuitySource::CurrentTempo
                } else {
                    TempoContinuitySource::Cleared
                },
                reason: if ctx.core_stable_scope {
                    TempoContinuityReason::RevalidationDecay
                } else {
                    TempoContinuityReason::InsufficientEvidence
                },
                boundary_pressure: ctx.boundary_pressure,
                tempo_ambiguity: ctx.tempo_ambiguity,
                revalidate_after_beats: if ctx.core_stable_scope { 4 } else { 0 },
                stage_index: 1,
                confidence: if ctx.core_stable_scope {
                    Confidence::new(
                        (state_confidence.0 * policy.reacquire_confidence_scale).clamp(0.0, 1.0),
                    )
                } else {
                    Confidence::new(0.0)
                },
            }),
            continuity_transition(TempoContinuityTransitionInputs {
                after_beats: if ctx.core_stable_scope { 12 } else { 0 },
                action: TempoContinuityAction::Clear,
                source: TempoContinuitySource::Cleared,
                reason: TempoContinuityReason::InsufficientEvidence,
                boundary_pressure: ctx.boundary_pressure,
                tempo_ambiguity: ctx.tempo_ambiguity,
                revalidate_after_beats: if ctx.core_stable_scope { 4 } else { 0 },
                stage_index: 2,
                confidence: Confidence::new(0.0),
            }),
        ),
    })
}

pub fn locked_recommendation(
    ctx: TempoStateScopeContext,
    policy: TempoStateLockedPolicy,
) -> TempoStateRecommendation {
    let state_confidence = Confidence::new(ctx.base_confidence.max(if ctx.localized_edge_scope {
        policy.localized_edge_min_confidence
    } else if ctx.strong_integer_anchor {
        policy
            .anchored_min_confidence
            .unwrap_or(policy.default_min_confidence)
    } else {
        policy.default_min_confidence
    }));
    let (
        localized_trusted_beats,
        localized_revalidate_after_beats,
        localized_downgrade_after_beats,
        localized_clear_after_beats,
        localized_decay_confidence_scale,
    ) = ctx.localized_edge_horizons();

    TempoStateRecommendation {
        action: TempoStateAction::Lock,
        reason: if ctx.localized_edge_scope {
            TempoStateReason::StableTempoWithEdgeDamage
        } else {
            policy.locked_reason
        },
        confidence: state_confidence,
        continuity: continuity_plan(
            TempoContinuityPlanInputs {
                action: TempoContinuityAction::Lock,
                source: TempoContinuitySource::CurrentTempo,
                reason: policy.continuity_reason,
                boundary_pressure: ctx.boundary_pressure,
                tempo_ambiguity: ctx.tempo_ambiguity,
                confidence: state_confidence,
                trusted_beats: if ctx.localized_edge_scope {
                    localized_trusted_beats
                } else {
                    16
                },
                revalidate_after_beats: if ctx.localized_edge_scope {
                    localized_revalidate_after_beats
                } else {
                    12
                },
            },
            continuity_transition(TempoContinuityTransitionInputs {
                after_beats: if ctx.localized_edge_scope {
                    localized_revalidate_after_beats
                } else {
                    12
                },
                action: TempoContinuityAction::Lock,
                source: TempoContinuitySource::CurrentTempo,
                reason: policy.continuity_reason,
                boundary_pressure: ctx.boundary_pressure,
                tempo_ambiguity: ctx.tempo_ambiguity,
                revalidate_after_beats: if ctx.localized_edge_scope {
                    localized_revalidate_after_beats
                } else {
                    12
                },
                stage_index: 0,
                confidence: state_confidence,
            }),
            continuity_transition(TempoContinuityTransitionInputs {
                after_beats: if ctx.localized_edge_scope {
                    localized_downgrade_after_beats
                } else {
                    20
                },
                action: TempoContinuityAction::Retain,
                source: TempoContinuitySource::CurrentTempo,
                reason: TempoContinuityReason::RevalidationDecay,
                boundary_pressure: ctx.boundary_pressure,
                tempo_ambiguity: ctx.tempo_ambiguity,
                revalidate_after_beats: if ctx.localized_edge_scope {
                    localized_revalidate_after_beats
                } else {
                    12
                },
                stage_index: 1,
                confidence: Confidence::new(
                    (state_confidence.0
                        * if ctx.localized_edge_scope {
                            localized_decay_confidence_scale
                        } else {
                            policy.default_decay_confidence_scale
                        })
                    .clamp(0.0, 1.0),
                ),
            }),
            continuity_transition(TempoContinuityTransitionInputs {
                after_beats: if ctx.localized_edge_scope {
                    localized_clear_after_beats
                } else {
                    28
                },
                action: TempoContinuityAction::Clear,
                source: TempoContinuitySource::Cleared,
                reason: TempoContinuityReason::InsufficientEvidence,
                boundary_pressure: ctx.boundary_pressure,
                tempo_ambiguity: ctx.tempo_ambiguity,
                revalidate_after_beats: if ctx.localized_edge_scope {
                    localized_revalidate_after_beats
                } else {
                    12
                },
                stage_index: 2,
                confidence: Confidence::new(0.0),
            }),
        ),
    }
}
