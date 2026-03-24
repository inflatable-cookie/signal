use crate::tempo_state_continuity_basics::has_tempo_cause;
use crate::{
    TempoContinuityAction, TempoContinuityCause, TempoContinuityCauseStack, TempoContinuityHistory,
    TempoContinuityProvenance, TempoContinuityReason, TempoContinuitySeverity,
    TempoContinuitySource, TempoContinuityTrigger, TempoContinuityUnresolvedSpan,
};

pub(crate) fn continuity_severity(
    action: TempoContinuityAction,
    source: TempoContinuitySource,
) -> TempoContinuitySeverity {
    match action {
        TempoContinuityAction::Lock => TempoContinuitySeverity::Confirmed,
        TempoContinuityAction::Retain => match source {
            TempoContinuitySource::CurrentTempo | TempoContinuitySource::CoreWindow => {
                TempoContinuitySeverity::Guarded
            }
            TempoContinuitySource::PriorTempo => TempoContinuitySeverity::Fragile,
            TempoContinuitySource::Cleared => TempoContinuitySeverity::Cleared,
        },
        TempoContinuityAction::Reacquire => TempoContinuitySeverity::Fragile,
        TempoContinuityAction::Clear => TempoContinuitySeverity::Cleared,
    }
}

pub(crate) fn continuity_history(
    action: TempoContinuityAction,
    source: TempoContinuitySource,
    reason: TempoContinuityReason,
    trigger: TempoContinuityTrigger,
    unresolved: TempoContinuityUnresolvedSpan,
    causes: TempoContinuityCauseStack,
    stage_index: usize,
) -> TempoContinuityHistory {
    let has_evidence_loss = has_tempo_cause(causes, TempoContinuityCause::EvidenceLoss);
    let has_prior_carry = has_tempo_cause(causes, TempoContinuityCause::PriorTempoCarry);
    let has_boundary = has_tempo_cause(causes, TempoContinuityCause::BoundaryDrift);

    match action {
        TempoContinuityAction::Clear => TempoContinuityHistory::Degrading,
        TempoContinuityAction::Lock
            if matches!(trigger, TempoContinuityTrigger::StableRevalidation)
                && unresolved.failed_revalidations == 0
                && !has_evidence_loss =>
        {
            TempoContinuityHistory::Reinforcing
        }
        TempoContinuityAction::Lock => TempoContinuityHistory::Preserving,
        TempoContinuityAction::Retain
            if stage_index > 0
                || has_evidence_loss
                || has_prior_carry
                || (has_boundary && unresolved.failed_revalidations >= 3) =>
        {
            TempoContinuityHistory::Degrading
        }
        TempoContinuityAction::Retain => match source {
            TempoContinuitySource::CurrentTempo | TempoContinuitySource::CoreWindow => {
                TempoContinuityHistory::Preserving
            }
            TempoContinuitySource::PriorTempo | TempoContinuitySource::Cleared => {
                TempoContinuityHistory::Degrading
            }
        },
        TempoContinuityAction::Reacquire
            if stage_index > 0
                || has_evidence_loss
                || has_prior_carry
                || unresolved.failed_revalidations > 1 =>
        {
            TempoContinuityHistory::Degrading
        }
        TempoContinuityAction::Reacquire
            if matches!(source, TempoContinuitySource::CurrentTempo)
                && matches!(reason, TempoContinuityReason::StableTempo)
                && !has_boundary =>
        {
            TempoContinuityHistory::Reinforcing
        }
        TempoContinuityAction::Reacquire => TempoContinuityHistory::Preserving,
    }
}

pub(crate) fn continuity_provenance(
    action: TempoContinuityAction,
    source: TempoContinuitySource,
    reason: TempoContinuityReason,
) -> TempoContinuityProvenance {
    match reason {
        TempoContinuityReason::IntegerTempoSnap => TempoContinuityProvenance::IntegerSnap,
        TempoContinuityReason::StableTempo => match action {
            TempoContinuityAction::Lock => TempoContinuityProvenance::StableRefinedEstimate,
            TempoContinuityAction::Reacquire => TempoContinuityProvenance::GuardedRefinedEstimate,
            TempoContinuityAction::Retain => match source {
                TempoContinuitySource::CurrentTempo => {
                    TempoContinuityProvenance::StableRefinedEstimate
                }
                TempoContinuitySource::PriorTempo => TempoContinuityProvenance::PriorTempoCarry,
                TempoContinuitySource::CoreWindow => TempoContinuityProvenance::CoreWindowEstimate,
                TempoContinuitySource::Cleared => TempoContinuityProvenance::NoTempo,
            },
            TempoContinuityAction::Clear => TempoContinuityProvenance::NoTempo,
        },
        TempoContinuityReason::CoreWindowCarry => TempoContinuityProvenance::CoreWindowEstimate,
        TempoContinuityReason::RevalidationDecay => match source {
            TempoContinuitySource::CurrentTempo => {
                TempoContinuityProvenance::GuardedRefinedEstimate
            }
            TempoContinuitySource::PriorTempo => TempoContinuityProvenance::PriorTempoCarry,
            TempoContinuitySource::CoreWindow => TempoContinuityProvenance::CoreWindowEstimate,
            TempoContinuitySource::Cleared => TempoContinuityProvenance::NoTempo,
        },
        TempoContinuityReason::InsufficientEvidence => TempoContinuityProvenance::NoTempo,
    }
}
