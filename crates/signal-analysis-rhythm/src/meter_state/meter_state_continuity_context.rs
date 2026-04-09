use super::meter_state_continuity_arc::continuity_plan;
use super::meter_state_continuity_helpers::{
    cause_stack, continuity_confidence, continuity_trigger, transition, unresolved_span,
    MeterContinuityCauseInputs, MeterContinuityPlanInputs, MeterContinuityStageContext,
};
use super::MeterSuppressionProfile;
use crate::rhythm_policy::*;
use signal_analysis::Confidence;

#[derive(Clone, Copy)]
pub struct MeterStagePlanContext {
    pub confidence: Confidence,
    pub tempo_ambiguity: Confidence,
    pub beats_per_bar: usize,
    pub phase_displacement_beats: usize,
    pub suppression_profile: MeterSuppressionProfile,
}

#[derive(Clone, Copy)]
struct MeterContinuityAssembly {
    action: MeterContinuityAction,
    source: MeterContinuitySource,
    reason: MeterContinuityReason,
    confidence: Confidence,
    trigger: MeterContinuityTrigger,
    unresolved: MeterContinuityUnresolvedSpan,
    causes: MeterContinuityCauseStack,
}

impl MeterStagePlanContext {
    fn assemble(
        self,
        action: MeterContinuityAction,
        source: MeterContinuitySource,
        reason: MeterContinuityReason,
        beat_span: usize,
        revalidate_after_beats: usize,
        stage_index: usize,
    ) -> MeterContinuityAssembly {
        let trigger = continuity_trigger(action, source, reason);
        let unresolved = unresolved_span(
            trigger,
            beat_span,
            revalidate_after_beats,
            self.beats_per_bar,
            self.phase_displacement_beats,
            stage_index,
        );
        let causes = cause_stack(MeterContinuityCauseInputs {
            action,
            source,
            reason,
            trigger,
            suppression_profile: self.suppression_profile,
            tempo_ambiguity: self.tempo_ambiguity,
            phase_displaced: self.phase_displacement_beats > 0,
            stage_index,
        });

        MeterContinuityAssembly {
            action,
            source,
            reason,
            confidence: continuity_confidence(
                action,
                source,
                self.confidence,
                beat_span,
                stage_index,
            ),
            trigger,
            unresolved,
            causes,
        }
    }

    pub fn stage(
        self,
        after_beats: usize,
        stage_action: MeterContinuityAction,
        stage_source: MeterContinuitySource,
        stage_reason: MeterContinuityReason,
        stage_index: usize,
    ) -> MeterContinuityTransition {
        let assembled = self.assemble(
            stage_action,
            stage_source,
            stage_reason,
            after_beats,
            after_beats,
            stage_index,
        );
        transition(
            after_beats,
            MeterContinuityStageContext {
                action: assembled.action,
                source: assembled.source,
                reason: assembled.reason,
                confidence: assembled.confidence,
                trigger: assembled.trigger,
                unresolved: assembled.unresolved,
                causes: assembled.causes,
                stage_index,
            },
        )
    }

    pub fn plan(
        self,
        plan_action: MeterContinuityAction,
        plan_source: MeterContinuitySource,
        plan_reason: MeterContinuityReason,
        trusted_beats: usize,
        revalidate_after_beats: usize,
        refresh: MeterContinuityTransition,
        first_decay: MeterContinuityTransition,
        final_decay: MeterContinuityTransition,
    ) -> MeterContinuityPlan {
        let assembled = self.assemble(
            plan_action,
            plan_source,
            plan_reason,
            trusted_beats,
            revalidate_after_beats,
            0,
        );
        continuity_plan(
            MeterContinuityPlanInputs {
                action: assembled.action,
                source: assembled.source,
                reason: assembled.reason,
                confidence: assembled.confidence,
                trigger: assembled.trigger,
                unresolved: assembled.unresolved,
                causes: assembled.causes,
                trusted_beats,
                revalidate_after_beats,
            },
            refresh,
            first_decay,
            final_decay,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::MeterStagePlanContext;
    use crate::meter_state::MeterSuppressionProfile;
    use crate::rhythm_policy::*;
    use signal_analysis::Confidence;

    fn context() -> MeterStagePlanContext {
        MeterStagePlanContext {
            confidence: Confidence::new(0.72),
            tempo_ambiguity: Confidence::new(0.18),
            beats_per_bar: 4,
            phase_displacement_beats: 2,
            suppression_profile: MeterSuppressionProfile {
                best_confidence: Confidence::new(0.62),
                best_support: 0.58,
                best_regularity: 0.54,
                trailing_confidence: Confidence::new(0.41),
                trailing_recent_stability: 0.38,
            },
        }
    }

    #[test]
    fn meter_stage_plan_context_preserves_stage_and_plan_policy_differences() {
        let ctx = context();
        let stage = ctx.stage(
            4,
            MeterContinuityAction::Retain,
            MeterContinuitySource::PriorMeter,
            MeterContinuityReason::PriorStateCarry,
            1,
        );
        let plan = ctx.plan(
            MeterContinuityAction::Retain,
            MeterContinuitySource::PriorMeter,
            MeterContinuityReason::PriorStateCarry,
            6,
            3,
            ctx.stage(
                2,
                MeterContinuityAction::Retain,
                MeterContinuitySource::PriorMeter,
                MeterContinuityReason::PriorStateCarry,
                0,
            ),
            ctx.stage(
                6,
                MeterContinuityAction::Reacquire,
                MeterContinuitySource::PriorMeter,
                MeterContinuityReason::RevalidationDecay,
                1,
            ),
            ctx.stage(
                10,
                MeterContinuityAction::Clear,
                MeterContinuitySource::Cleared,
                MeterContinuityReason::InsufficientEvidence,
                2,
            ),
        );

        assert_eq!(stage.trigger, MeterContinuityTrigger::PriorStateDrift);
        assert_eq!(plan.trigger, MeterContinuityTrigger::PriorStateDrift);
        assert_eq!(stage.unresolved.beats, 4);
        assert_eq!(plan.unresolved.beats, 6);
        assert_eq!(stage.unresolved.failed_revalidations, 1);
        assert_eq!(plan.unresolved.failed_revalidations, 2);
        assert_eq!(stage.reason, MeterContinuityReason::PriorStateCarry);
        assert_eq!(plan.reason, MeterContinuityReason::PriorStateCarry);
        assert_eq!(stage.causes.primary, plan.causes.primary);
        assert_eq!(plan.trusted_beats, 6);
        assert_eq!(plan.revalidate_after_beats, 3);
    }
}
