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

impl MeterStagePlanContext {
    pub fn stage(
        self,
        after_beats: usize,
        stage_action: MeterContinuityAction,
        stage_source: MeterContinuitySource,
        stage_reason: MeterContinuityReason,
        stage_index: usize,
    ) -> MeterContinuityTransition {
        let stage_trigger = continuity_trigger(stage_action, stage_source, stage_reason);
        let stage_unresolved = unresolved_span(
            stage_trigger,
            after_beats,
            after_beats,
            self.beats_per_bar,
            self.phase_displacement_beats,
            stage_index,
        );
        let stage_causes = cause_stack(MeterContinuityCauseInputs {
            action: stage_action,
            source: stage_source,
            reason: stage_reason,
            trigger: stage_trigger,
            suppression_profile: self.suppression_profile,
            tempo_ambiguity: self.tempo_ambiguity,
            phase_displaced: self.phase_displacement_beats > 0,
            stage_index,
        });
        transition(
            after_beats,
            MeterContinuityStageContext {
                action: stage_action,
                source: stage_source,
                reason: stage_reason,
                confidence: continuity_confidence(
                    stage_action,
                    stage_source,
                    self.confidence,
                    after_beats,
                    stage_index,
                ),
                trigger: stage_trigger,
                unresolved: stage_unresolved,
                causes: stage_causes,
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
        let plan_trigger = continuity_trigger(plan_action, plan_source, plan_reason);
        let plan_unresolved = unresolved_span(
            plan_trigger,
            trusted_beats,
            revalidate_after_beats,
            self.beats_per_bar,
            self.phase_displacement_beats,
            0,
        );
        let plan_causes = cause_stack(MeterContinuityCauseInputs {
            action: plan_action,
            source: plan_source,
            reason: plan_reason,
            trigger: plan_trigger,
            suppression_profile: self.suppression_profile,
            tempo_ambiguity: self.tempo_ambiguity,
            phase_displaced: self.phase_displacement_beats > 0,
            stage_index: 0,
        });
        continuity_plan(
            MeterContinuityPlanInputs {
                action: plan_action,
                source: plan_source,
                reason: plan_reason,
                confidence: continuity_confidence(
                    plan_action,
                    plan_source,
                    self.confidence,
                    trusted_beats,
                    0,
                ),
                trigger: plan_trigger,
                unresolved: plan_unresolved,
                causes: plan_causes,
                trusted_beats,
                revalidate_after_beats,
            },
            refresh,
            first_decay,
            final_decay,
        )
    }
}
