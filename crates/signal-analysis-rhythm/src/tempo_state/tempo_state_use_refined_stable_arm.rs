use super::tempo_state_continuity_arc::continuity_plan;
use super::tempo_state_continuity_helpers::TempoContinuityPlanInputs;
use super::tempo_state_scope_context::TempoStateScopeContext;
use crate::tempo_policy::*;
use crate::tempo_state_continuity_transition::{
    continuity_transition, TempoContinuityTransitionInputs,
};
use signal_analysis::Confidence;

pub fn use_refined_stable_arm(ctx: TempoStateScopeContext) -> TempoStateRecommendation {
    if ctx.core_stable_scope || ctx.mid_track_unstable_scope {
        let state_confidence = Confidence::new(ctx.base_confidence.max(if ctx.core_stable_scope {
            0.56
        } else {
            0.46
        }));
        return TempoStateRecommendation {
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
                        Confidence::new((state_confidence.0 * 0.94).clamp(0.0, 1.0))
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
                        Confidence::new((state_confidence.0 * 0.66).clamp(0.0, 1.0))
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
        };
    }

    let state_confidence = Confidence::new(ctx.base_confidence.max(if ctx.localized_edge_scope {
        0.72
    } else {
        0.76
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
            TempoStateReason::StableRefinedTempo
        },
        confidence: state_confidence,
        continuity: continuity_plan(
            TempoContinuityPlanInputs {
                action: TempoContinuityAction::Lock,
                source: TempoContinuitySource::CurrentTempo,
                reason: TempoContinuityReason::StableTempo,
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
                reason: TempoContinuityReason::StableTempo,
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
                            0.72
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
