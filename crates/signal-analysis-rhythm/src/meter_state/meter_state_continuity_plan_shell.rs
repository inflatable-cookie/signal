use super::meter_state_continuity_context::MeterStagePlanContext;
use crate::rhythm_policy::*;

#[derive(Clone, Copy)]
pub struct MeterPlanStageSpec {
    pub after_beats: usize,
    pub action: MeterContinuityAction,
    pub source: MeterContinuitySource,
    pub reason: MeterContinuityReason,
    pub stage_index: usize,
}

#[derive(Clone, Copy)]
pub struct MeterPlanSpec {
    pub action: MeterContinuityAction,
    pub source: MeterContinuitySource,
    pub reason: MeterContinuityReason,
    pub trusted_beats: usize,
    pub revalidate_after_beats: usize,
    pub refresh: MeterPlanStageSpec,
    pub first_decay: MeterPlanStageSpec,
    pub final_decay: MeterPlanStageSpec,
}

pub fn stage(
    after_beats: usize,
    action: MeterContinuityAction,
    source: MeterContinuitySource,
    reason: MeterContinuityReason,
    stage_index: usize,
) -> MeterPlanStageSpec {
    MeterPlanStageSpec {
        after_beats,
        action,
        source,
        reason,
        stage_index,
    }
}

pub fn plan(
    action: MeterContinuityAction,
    source: MeterContinuitySource,
    reason: MeterContinuityReason,
    trusted_beats: usize,
    revalidate_after_beats: usize,
    refresh: MeterPlanStageSpec,
    first_decay: MeterPlanStageSpec,
    final_decay: MeterPlanStageSpec,
) -> MeterPlanSpec {
    MeterPlanSpec {
        action,
        source,
        reason,
        trusted_beats,
        revalidate_after_beats,
        refresh,
        first_decay,
        final_decay,
    }
}

pub fn build_plan(ctx: MeterStagePlanContext, spec: MeterPlanSpec) -> MeterContinuityPlan {
    ctx.plan(
        spec.action,
        spec.source,
        spec.reason,
        spec.trusted_beats,
        spec.revalidate_after_beats,
        ctx.stage(
            spec.refresh.after_beats,
            spec.refresh.action,
            spec.refresh.source,
            spec.refresh.reason,
            spec.refresh.stage_index,
        ),
        ctx.stage(
            spec.first_decay.after_beats,
            spec.first_decay.action,
            spec.first_decay.source,
            spec.first_decay.reason,
            spec.first_decay.stage_index,
        ),
        ctx.stage(
            spec.final_decay.after_beats,
            spec.final_decay.action,
            spec.final_decay.source,
            spec.final_decay.reason,
            spec.final_decay.stage_index,
        ),
    )
}

pub fn build_recommendation(
    ctx: MeterStagePlanContext,
    bar_length: MeterPlanSpec,
    downbeat_phase: MeterPlanSpec,
) -> MeterContinuityRecommendation {
    MeterContinuityRecommendation {
        bar_length: build_plan(ctx, bar_length),
        downbeat_phase: build_plan(ctx, downbeat_phase),
    }
}

#[cfg(test)]
mod tests {
    use super::{build_recommendation, plan, stage};
    use crate::meter_state::MeterSuppressionProfile;
    use crate::rhythm_policy::*;
    use signal_analysis::Confidence;

    fn context() -> super::MeterStagePlanContext {
        super::MeterStagePlanContext {
            confidence: Confidence::new(0.7),
            tempo_ambiguity: Confidence::new(0.2),
            beats_per_bar: 4,
            phase_displacement_beats: 2,
            suppression_profile: MeterSuppressionProfile {
                best_confidence: Confidence::new(0.6),
                best_support: 0.6,
                best_regularity: 0.5,
                trailing_confidence: Confidence::new(0.4),
                trailing_recent_stability: 0.4,
            },
        }
    }

    #[test]
    fn meter_plan_shell_preserves_per_plan_policy_differences() {
        let recommendation = build_recommendation(
            context(),
            plan(
                MeterContinuityAction::Retain,
                MeterContinuitySource::PriorMeter,
                MeterContinuityReason::PriorStateCarry,
                6,
                4,
                stage(
                    4,
                    MeterContinuityAction::Retain,
                    MeterContinuitySource::PriorMeter,
                    MeterContinuityReason::PriorStateCarry,
                    0,
                ),
                stage(
                    8,
                    MeterContinuityAction::Reacquire,
                    MeterContinuitySource::PriorMeter,
                    MeterContinuityReason::RevalidationDecay,
                    1,
                ),
                stage(
                    10,
                    MeterContinuityAction::Clear,
                    MeterContinuitySource::Cleared,
                    MeterContinuityReason::InsufficientEvidence,
                    2,
                ),
            ),
            plan(
                MeterContinuityAction::Reacquire,
                MeterContinuitySource::CurrentMeter,
                MeterContinuityReason::PhaseDisplacement,
                0,
                2,
                stage(
                    2,
                    MeterContinuityAction::Lock,
                    MeterContinuitySource::CurrentMeter,
                    MeterContinuityReason::StableEvidence,
                    0,
                ),
                stage(
                    4,
                    MeterContinuityAction::Reacquire,
                    MeterContinuitySource::CurrentMeter,
                    MeterContinuityReason::PhaseDisplacement,
                    1,
                ),
                stage(
                    8,
                    MeterContinuityAction::Clear,
                    MeterContinuitySource::Cleared,
                    MeterContinuityReason::InsufficientEvidence,
                    2,
                ),
            ),
        );

        assert_eq!(
            recommendation.bar_length.reason,
            MeterContinuityReason::PriorStateCarry
        );
        assert_eq!(
            recommendation.downbeat_phase.reason,
            MeterContinuityReason::PhaseDisplacement
        );
        assert_eq!(recommendation.bar_length.trusted_beats, 6);
        assert_eq!(recommendation.downbeat_phase.trusted_beats, 0);
        assert_eq!(
            recommendation.downbeat_phase.lifecycle.decay[0].reason,
            MeterContinuityReason::PhaseDisplacement
        );
    }
}
