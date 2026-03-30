use super::tempo_state_arc_decision::continuity_arc_decision;
use super::tempo_state_continuity_helpers::{
    TempoContinuityArcDecisionInputs, TempoContinuityArcInputs, TempoContinuityPlanInputs,
};
use crate::tempo_policy::*;
use crate::tempo_state_continuity_basics::{
    continuity_cause_stack, continuity_history, continuity_provenance, continuity_severity,
    continuity_trigger, has_tempo_cause, unresolved_span,
};
use crate::tempo_state_continuity_refresh::continuity_refresh_strength;
use crate::tempo_state_continuity_transition::continuity_expiry;
use signal_analysis::Confidence;

pub fn continuity_arc_support(
    unresolved: TempoContinuityUnresolvedSpan,
    causes: TempoContinuityCauseStack,
    current: TempoContinuityHistory,
    refresh: TempoContinuityTransition,
    first_decay: TempoContinuityTransition,
    final_decay: TempoContinuityTransition,
) -> TempoContinuityArcSupport {
    let refresh_bonus = match refresh.history {
        TempoContinuityHistory::Reinforcing => 0.26,
        TempoContinuityHistory::Preserving => 0.12,
        TempoContinuityHistory::Degrading => 0.0,
    };
    let current_bonus = match current {
        TempoContinuityHistory::Reinforcing => 0.18,
        TempoContinuityHistory::Preserving => 0.08,
        TempoContinuityHistory::Degrading => 0.0,
    };
    let decay_penalty = match first_decay.history {
        TempoContinuityHistory::Degrading => 0.08,
        _ => 0.0,
    } + match final_decay.history {
        TempoContinuityHistory::Degrading => 0.12,
        _ => 0.0,
    };
    let refresh_strength = Confidence::new(
        (refresh.refresh_strength.0 + refresh_bonus + current_bonus - decay_penalty)
            .clamp(0.0, 1.0),
    );

    let drift_pressure = Confidence::new(
        ((unresolved.failed_revalidations as f32 * 0.20)
            + match current {
                TempoContinuityHistory::Degrading => 0.18,
                TempoContinuityHistory::Preserving => 0.08,
                TempoContinuityHistory::Reinforcing => 0.0,
            }
            + match first_decay.history {
                TempoContinuityHistory::Degrading => 0.14,
                _ => 0.0,
            }
            + match final_decay.history {
                TempoContinuityHistory::Degrading => 0.18,
                _ => 0.0,
            })
        .clamp(0.0, 1.0),
    );

    let instability_pressure = Confidence::new(
        ((if has_tempo_cause(causes, TempoContinuityCause::BoundaryDrift) {
            0.28_f32
        } else {
            0.0
        }) + (if has_tempo_cause(causes, TempoContinuityCause::TempoAmbiguity) {
            0.18
        } else {
            0.0
        }) + (if has_tempo_cause(causes, TempoContinuityCause::PriorTempoCarry) {
            0.16
        } else {
            0.0
        }) + (if has_tempo_cause(causes, TempoContinuityCause::CoreWindowCarry) {
            0.10
        } else {
            0.0
        }) + (if has_tempo_cause(causes, TempoContinuityCause::EvidenceLoss) {
            0.40
        } else {
            0.0
        }))
        .clamp(0.0, 1.0),
    );

    TempoContinuityArcSupport {
        refresh_strength,
        drift_pressure,
        instability_pressure,
    }
}

pub fn continuity_arc_assessment(
    inputs: TempoContinuityArcInputs,
) -> (
    TempoContinuityArc,
    TempoContinuityArcRationale,
    TempoContinuityArcSupport,
) {
    let TempoContinuityArcInputs {
        source,
        confidence,
        unresolved,
        causes,
        current,
        refresh,
        first_decay,
        final_decay,
    } = inputs;
    let has_evidence_loss = has_tempo_cause(causes, TempoContinuityCause::EvidenceLoss);
    let has_boundary = has_tempo_cause(causes, TempoContinuityCause::BoundaryDrift);
    let has_prior_carry = has_tempo_cause(causes, TempoContinuityCause::PriorTempoCarry);
    let persistent_decay = matches!(first_decay.history, TempoContinuityHistory::Degrading)
        && matches!(final_decay.history, TempoContinuityHistory::Degrading);
    let support = continuity_arc_support(
        unresolved,
        causes,
        current,
        refresh,
        first_decay,
        final_decay,
    );

    if matches!(current, TempoContinuityHistory::Degrading)
        && (persistent_decay || has_evidence_loss)
    {
        return (
            TempoContinuityArc::Collapsing,
            if has_evidence_loss {
                TempoContinuityArcRationale::EvidenceLoss
            } else {
                TempoContinuityArcRationale::UnresolvedDrift
            },
            support,
        );
    }

    if matches!(refresh.history, TempoContinuityHistory::Reinforcing) && !has_evidence_loss {
        if matches!(current, TempoContinuityHistory::Reinforcing) {
            return (
                TempoContinuityArc::Recovering,
                TempoContinuityArcRationale::RefreshStrength,
                support,
            );
        }

        if matches!(current, TempoContinuityHistory::Preserving)
            && confidence.0 >= 0.56
            && unresolved.failed_revalidations <= 1
            && !has_prior_carry
        {
            return (
                TempoContinuityArc::Recovering,
                TempoContinuityArcRationale::RefreshStrength,
                support,
            );
        }
    }

    if has_evidence_loss
        || (persistent_decay && confidence.0 < 0.24)
        || (matches!(current, TempoContinuityHistory::Degrading)
            && !matches!(refresh.history, TempoContinuityHistory::Reinforcing))
    {
        return (
            TempoContinuityArc::Collapsing,
            if has_evidence_loss {
                TempoContinuityArcRationale::EvidenceLoss
            } else if has_boundary {
                TempoContinuityArcRationale::BoundaryDrift
            } else {
                TempoContinuityArcRationale::UnresolvedDrift
            },
            support,
        );
    }

    (
        TempoContinuityArc::Stalling,
        if has_boundary || matches!(source, TempoContinuitySource::CoreWindow) {
            TempoContinuityArcRationale::BoundaryDrift
        } else if unresolved.failed_revalidations >= 2 || has_prior_carry {
            TempoContinuityArcRationale::UnresolvedDrift
        } else {
            TempoContinuityArcRationale::StableCarry
        },
        support,
    )
}

pub fn continuity_plan(
    plan: TempoContinuityPlanInputs,
    refresh: TempoContinuityTransition,
    first_decay: TempoContinuityTransition,
    final_decay: TempoContinuityTransition,
) -> TempoContinuityPlan {
    let TempoContinuityPlanInputs {
        action,
        source,
        reason,
        boundary_pressure,
        tempo_ambiguity,
        confidence,
        trusted_beats,
        revalidate_after_beats,
    } = plan;
    let trigger = continuity_trigger(action, source, reason, boundary_pressure, tempo_ambiguity);
    let unresolved = unresolved_span(trigger, trusted_beats, revalidate_after_beats, 0);
    let causes = continuity_cause_stack(action, source, reason, boundary_pressure, tempo_ambiguity);
    let severity = continuity_severity(action, source);
    let history = continuity_history(action, source, reason, trigger, unresolved, causes, 0);
    let provenance = continuity_provenance(action, source, reason);
    let expiry = continuity_expiry(
        trusted_beats,
        revalidate_after_beats,
        first_decay,
        final_decay,
    );
    let (arc, arc_rationale, arc_support) = continuity_arc_assessment(TempoContinuityArcInputs {
        source,
        confidence,
        unresolved,
        causes,
        current: history,
        refresh,
        first_decay,
        final_decay,
    });
    let arc_decision = continuity_arc_decision(TempoContinuityArcDecisionInputs {
        arc,
        rationale: arc_rationale,
        support: arc_support,
        severity,
        history,
        trigger,
        causes,
        provenance,
        expiry,
        trusted_beats,
        revalidate_after_beats,
        confidence,
        unresolved,
        refresh,
        first_decay,
        final_decay,
    });
    TempoContinuityPlan {
        action,
        source,
        severity,
        history,
        arc,
        arc_rationale,
        arc_support,
        arc_decision,
        reason,
        trigger,
        unresolved,
        causes,
        provenance,
        confidence,
        refresh_strength: continuity_refresh_strength(
            action,
            source,
            confidence,
            history,
            unresolved,
            causes,
            trusted_beats.max(revalidate_after_beats),
        ),
        trusted_beats,
        revalidate_after_beats,
        expiry,
        lifecycle: TempoContinuityLifecycle {
            refresh,
            decay: [first_decay, final_decay],
        },
    }
}
