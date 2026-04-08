use super::tempo_state_continuity_arc::continuity_plan;
use super::tempo_state_continuity_helpers::TempoContinuityPlanInputs;
use super::tempo_state_scope_context::TempoStateScopeContext;
use crate::tempo_policy::*;
use crate::tempo_state_continuity_transition::{
    continuity_transition, TempoContinuityTransitionInputs,
};
use signal_analysis::Confidence;

pub fn use_core_window_arm(ctx: TempoStateScopeContext) -> TempoStateRecommendation {
    let state_confidence = Confidence::new(ctx.base_confidence.max(0.58));
    TempoStateRecommendation {
        action: TempoStateAction::Monitor,
        reason: TempoStateReason::CoreWindowFallback,
        confidence: state_confidence,
        continuity: continuity_plan(
            TempoContinuityPlanInputs {
                action: TempoContinuityAction::Retain,
                source: TempoContinuitySource::CoreWindow,
                reason: TempoContinuityReason::CoreWindowCarry,
                boundary_pressure: ctx.boundary_pressure,
                tempo_ambiguity: ctx.tempo_ambiguity,
                confidence: state_confidence,
                trusted_beats: 8,
                revalidate_after_beats: 4,
            },
            continuity_transition(TempoContinuityTransitionInputs {
                after_beats: 4,
                action: TempoContinuityAction::Retain,
                source: TempoContinuitySource::CoreWindow,
                reason: TempoContinuityReason::CoreWindowCarry,
                boundary_pressure: ctx.boundary_pressure,
                tempo_ambiguity: ctx.tempo_ambiguity,
                revalidate_after_beats: 4,
                stage_index: 0,
                confidence: state_confidence,
            }),
            continuity_transition(TempoContinuityTransitionInputs {
                after_beats: 8,
                action: TempoContinuityAction::Reacquire,
                source: TempoContinuitySource::PriorTempo,
                reason: TempoContinuityReason::RevalidationDecay,
                boundary_pressure: ctx.boundary_pressure,
                tempo_ambiguity: ctx.tempo_ambiguity,
                revalidate_after_beats: 4,
                stage_index: 1,
                confidence: Confidence::new((state_confidence.0 * 0.68).clamp(0.0, 1.0)),
            }),
            continuity_transition(TempoContinuityTransitionInputs {
                after_beats: 12,
                action: TempoContinuityAction::Clear,
                source: TempoContinuitySource::Cleared,
                reason: TempoContinuityReason::InsufficientEvidence,
                boundary_pressure: ctx.boundary_pressure,
                tempo_ambiguity: ctx.tempo_ambiguity,
                revalidate_after_beats: 4,
                stage_index: 2,
                confidence: Confidence::new(0.0),
            }),
        ),
    }
}

pub fn use_refined_guarded_arm(ctx: TempoStateScopeContext) -> TempoStateRecommendation {
    let state_confidence = Confidence::new(ctx.base_confidence.max(0.56));
    TempoStateRecommendation {
        action: TempoStateAction::Monitor,
        reason: TempoStateReason::StableRefinedTempo,
        confidence: state_confidence,
        continuity: continuity_plan(
            TempoContinuityPlanInputs {
                action: TempoContinuityAction::Reacquire,
                source: TempoContinuitySource::CurrentTempo,
                reason: TempoContinuityReason::RevalidationDecay,
                boundary_pressure: ctx.boundary_pressure,
                tempo_ambiguity: ctx.tempo_ambiguity,
                confidence: state_confidence,
                trusted_beats: 4,
                revalidate_after_beats: 4,
            },
            continuity_transition(TempoContinuityTransitionInputs {
                after_beats: 4,
                action: TempoContinuityAction::Lock,
                source: TempoContinuitySource::CurrentTempo,
                reason: TempoContinuityReason::StableTempo,
                boundary_pressure: ctx.boundary_pressure,
                tempo_ambiguity: ctx.tempo_ambiguity,
                revalidate_after_beats: 4,
                stage_index: 0,
                confidence: Confidence::new((state_confidence.0 * 0.96).clamp(0.0, 1.0)),
            }),
            continuity_transition(TempoContinuityTransitionInputs {
                after_beats: 8,
                action: TempoContinuityAction::Reacquire,
                source: TempoContinuitySource::CurrentTempo,
                reason: TempoContinuityReason::RevalidationDecay,
                boundary_pressure: ctx.boundary_pressure,
                tempo_ambiguity: ctx.tempo_ambiguity,
                revalidate_after_beats: 4,
                stage_index: 1,
                confidence: Confidence::new((state_confidence.0 * 0.66).clamp(0.0, 1.0)),
            }),
            continuity_transition(TempoContinuityTransitionInputs {
                after_beats: 12,
                action: TempoContinuityAction::Clear,
                source: TempoContinuitySource::Cleared,
                reason: TempoContinuityReason::InsufficientEvidence,
                boundary_pressure: ctx.boundary_pressure,
                tempo_ambiguity: ctx.tempo_ambiguity,
                revalidate_after_beats: 4,
                stage_index: 2,
                confidence: Confidence::new(0.0),
            }),
        ),
    }
}

pub fn defer_arm(ctx: TempoStateScopeContext, base_instability: f32) -> TempoStateRecommendation {
    let state_confidence =
        Confidence::new((0.55 * base_instability + 0.45 * ctx.tempo_ambiguity.0).clamp(0.0, 1.0));
    TempoStateRecommendation {
        action: TempoStateAction::Defer,
        reason: TempoStateReason::TempoDeferred,
        confidence: state_confidence,
        continuity: continuity_plan(
            TempoContinuityPlanInputs {
                action: TempoContinuityAction::Clear,
                source: TempoContinuitySource::Cleared,
                reason: TempoContinuityReason::InsufficientEvidence,
                boundary_pressure: ctx.boundary_pressure,
                tempo_ambiguity: ctx.tempo_ambiguity,
                confidence: state_confidence,
                trusted_beats: 0,
                revalidate_after_beats: 0,
            },
            continuity_transition(TempoContinuityTransitionInputs {
                after_beats: 0,
                action: TempoContinuityAction::Clear,
                source: TempoContinuitySource::Cleared,
                reason: TempoContinuityReason::InsufficientEvidence,
                boundary_pressure: ctx.boundary_pressure,
                tempo_ambiguity: ctx.tempo_ambiguity,
                revalidate_after_beats: 0,
                stage_index: 0,
                confidence: Confidence::new(0.0),
            }),
            continuity_transition(TempoContinuityTransitionInputs {
                after_beats: 0,
                action: TempoContinuityAction::Clear,
                source: TempoContinuitySource::Cleared,
                reason: TempoContinuityReason::InsufficientEvidence,
                boundary_pressure: ctx.boundary_pressure,
                tempo_ambiguity: ctx.tempo_ambiguity,
                revalidate_after_beats: 0,
                stage_index: 1,
                confidence: Confidence::new(0.0),
            }),
            continuity_transition(TempoContinuityTransitionInputs {
                after_beats: 0,
                action: TempoContinuityAction::Clear,
                source: TempoContinuitySource::Cleared,
                reason: TempoContinuityReason::InsufficientEvidence,
                boundary_pressure: ctx.boundary_pressure,
                tempo_ambiguity: ctx.tempo_ambiguity,
                revalidate_after_beats: 0,
                stage_index: 2,
                confidence: Confidence::new(0.0),
            }),
        ),
    }
}
