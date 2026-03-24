use signal_analysis::Confidence;

#[path = "tempo_state_continuity_basics/outcomes.rs"]
mod outcomes;

pub(crate) use outcomes::{continuity_history, continuity_provenance, continuity_severity};

use crate::{
    TempoContinuityAction, TempoContinuityCause, TempoContinuityCauseStack, TempoContinuityReason,
    TempoContinuitySource, TempoContinuityTrigger, TempoContinuityUnresolvedSpan,
};

fn push_tempo_cause(
    slots: &mut [Option<TempoContinuityCause>; 3],
    count: &mut usize,
    cause: TempoContinuityCause,
) {
    if slots.iter().flatten().any(|existing| *existing == cause) {
        return;
    }

    if *count < slots.len() {
        slots[*count] = Some(cause);
        *count += 1;
    }
}

pub(crate) fn continuity_trigger(
    action: TempoContinuityAction,
    source: TempoContinuitySource,
    reason: TempoContinuityReason,
    boundary_pressure: Confidence,
    tempo_ambiguity: Confidence,
) -> TempoContinuityTrigger {
    if matches!(action, TempoContinuityAction::Clear)
        || matches!(source, TempoContinuitySource::Cleared)
        || matches!(reason, TempoContinuityReason::InsufficientEvidence)
    {
        return TempoContinuityTrigger::EvidenceLoss;
    }

    if matches!(source, TempoContinuitySource::PriorTempo) {
        return TempoContinuityTrigger::PriorTempoDrift;
    }

    if matches!(source, TempoContinuitySource::CoreWindow)
        || matches!(reason, TempoContinuityReason::CoreWindowCarry)
        || boundary_pressure.0 >= 0.45
    {
        return TempoContinuityTrigger::BoundaryDrift;
    }

    if tempo_ambiguity.0 >= 0.18 && !matches!(action, TempoContinuityAction::Lock) {
        return TempoContinuityTrigger::AmbiguityCarry;
    }

    TempoContinuityTrigger::StableRevalidation
}

pub(crate) fn unresolved_span(
    trigger: TempoContinuityTrigger,
    beat_span: usize,
    revalidate_after_beats: usize,
    stage_index: usize,
) -> TempoContinuityUnresolvedSpan {
    let beats = match trigger {
        TempoContinuityTrigger::StableRevalidation => 0,
        TempoContinuityTrigger::BoundaryDrift
        | TempoContinuityTrigger::AmbiguityCarry
        | TempoContinuityTrigger::PriorTempoDrift => beat_span.max(revalidate_after_beats.max(1)),
        TempoContinuityTrigger::EvidenceLoss => beat_span,
    };
    let failed_revalidations = if beats == 0 || revalidate_after_beats == 0 {
        0
    } else {
        beats.div_ceil(revalidate_after_beats).max(stage_index)
    };

    TempoContinuityUnresolvedSpan {
        beats,
        failed_revalidations,
    }
}

pub(crate) fn continuity_cause_stack(
    action: TempoContinuityAction,
    source: TempoContinuitySource,
    reason: TempoContinuityReason,
    boundary_pressure: Confidence,
    tempo_ambiguity: Confidence,
) -> TempoContinuityCauseStack {
    let mut causes = [None; 3];
    let mut count = 0;

    if matches!(action, TempoContinuityAction::Lock)
        && matches!(source, TempoContinuitySource::CurrentTempo)
        && matches!(
            reason,
            TempoContinuityReason::StableTempo | TempoContinuityReason::IntegerTempoSnap
        )
    {
        push_tempo_cause(
            &mut causes,
            &mut count,
            TempoContinuityCause::StableTempoEvidence,
        );
    }

    if boundary_pressure.0 >= 0.45 || matches!(source, TempoContinuitySource::CoreWindow) {
        push_tempo_cause(&mut causes, &mut count, TempoContinuityCause::BoundaryDrift);
    }

    if tempo_ambiguity.0 >= 0.18 {
        push_tempo_cause(
            &mut causes,
            &mut count,
            TempoContinuityCause::TempoAmbiguity,
        );
    }

    if matches!(source, TempoContinuitySource::PriorTempo) {
        push_tempo_cause(
            &mut causes,
            &mut count,
            TempoContinuityCause::PriorTempoCarry,
        );
    }

    if matches!(source, TempoContinuitySource::CoreWindow)
        || matches!(reason, TempoContinuityReason::CoreWindowCarry)
    {
        push_tempo_cause(
            &mut causes,
            &mut count,
            TempoContinuityCause::CoreWindowCarry,
        );
    }

    if matches!(action, TempoContinuityAction::Clear)
        || matches!(source, TempoContinuitySource::Cleared)
    {
        push_tempo_cause(&mut causes, &mut count, TempoContinuityCause::EvidenceLoss);
    }

    let primary = if matches!(action, TempoContinuityAction::Clear) && tempo_ambiguity.0 >= 0.18 {
        TempoContinuityCause::TempoAmbiguity
    } else {
        causes[0].unwrap_or(match action {
            TempoContinuityAction::Lock => TempoContinuityCause::StableTempoEvidence,
            TempoContinuityAction::Retain | TempoContinuityAction::Reacquire => {
                TempoContinuityCause::TempoAmbiguity
            }
            TempoContinuityAction::Clear => TempoContinuityCause::EvidenceLoss,
        })
    };

    TempoContinuityCauseStack {
        primary,
        secondary: [causes[1], causes[2]],
        count: count.max(1),
    }
}

pub(crate) fn has_tempo_cause(
    stack: TempoContinuityCauseStack,
    cause: TempoContinuityCause,
) -> bool {
    stack.primary == cause
        || stack
            .secondary
            .into_iter()
            .flatten()
            .any(|entry| entry == cause)
}
