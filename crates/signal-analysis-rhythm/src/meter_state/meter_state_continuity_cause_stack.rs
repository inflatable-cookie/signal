use super::meter_state_continuity_rule_surface::{
    primary_cause_for_reason, primary_cause_for_trigger,
};
use super::meter_state_continuity_types::*;
use crate::rhythm_policy::*;

pub fn cause_stack(inputs: MeterContinuityCauseInputs) -> MeterContinuityCauseStack {
    let MeterContinuityCauseInputs {
        action,
        source,
        reason,
        trigger,
        suppression_profile,
        tempo_ambiguity,
        phase_displaced,
        stage_index,
    } = inputs;
    let mut causes = [None; 3];
    let mut count = 0usize;

    if let Some(cause) = primary_cause_for_reason(reason) {
        push_cause(&mut causes, &mut count, cause);
    }

    push_cause(&mut causes, &mut count, primary_cause_for_trigger(trigger));

    if phase_displaced {
        push_cause(
            &mut causes,
            &mut count,
            MeterContinuityCause::PhaseDisplacement,
        );
    }

    if tempo_ambiguity.0 >= 0.28 {
        push_cause(
            &mut causes,
            &mut count,
            MeterContinuityCause::TempoAmbiguity,
        );
    }

    if suppression_profile.best_support < 0.58 || suppression_profile.best_confidence.0 < 0.24 {
        push_cause(
            &mut causes,
            &mut count,
            MeterContinuityCause::SparseMeterSupport,
        );
    }

    if suppression_profile.best_regularity < 0.32
        || (stage_index > 0 && suppression_profile.trailing_recent_stability < 0.30)
    {
        push_cause(
            &mut causes,
            &mut count,
            MeterContinuityCause::IrregularBarStructure,
        );
    }

    if matches!(source, MeterContinuitySource::Cleared)
        || matches!(action, MeterContinuityAction::Clear)
    {
        push_cause(&mut causes, &mut count, MeterContinuityCause::EvidenceLoss);
    }

    let primary = causes[0].unwrap_or(match action {
        MeterContinuityAction::Lock => MeterContinuityCause::StableMeterEvidence,
        MeterContinuityAction::Retain | MeterContinuityAction::Reacquire => {
            MeterContinuityCause::SparseMeterSupport
        }
        MeterContinuityAction::Clear => MeterContinuityCause::EvidenceLoss,
    });

    MeterContinuityCauseStack {
        primary,
        secondary: [causes[1], causes[2]],
        count: count.max(1),
    }
}

pub fn has_cause(stack: MeterContinuityCauseStack, cause: MeterContinuityCause) -> bool {
    stack.primary == cause
        || stack
            .secondary
            .into_iter()
            .flatten()
            .any(|entry| entry == cause)
}
