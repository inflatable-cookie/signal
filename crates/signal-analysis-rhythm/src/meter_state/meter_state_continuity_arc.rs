use super::meter_state_continuity_helpers::{
    continuity_history, continuity_severity, has_cause, MeterContinuityArcInputs,
    MeterContinuityPlanInputs, MeterContinuityStageContext,
};
use crate::rhythm_policy::*;
use signal_analysis::Confidence;

pub fn continuity_arc_support(
    unresolved: MeterContinuityUnresolvedSpan,
    causes: MeterContinuityCauseStack,
    current: MeterContinuityHistory,
    refresh: MeterContinuityTransition,
    first_decay: MeterContinuityTransition,
    final_decay: MeterContinuityTransition,
) -> MeterContinuityArcSupport {
    let refresh_bonus = match refresh.history {
        MeterContinuityHistory::Reinforcing => 0.28,
        MeterContinuityHistory::Preserving => 0.12,
        MeterContinuityHistory::Degrading => 0.0,
    };
    let current_bonus = match current {
        MeterContinuityHistory::Reinforcing => 0.22,
        MeterContinuityHistory::Preserving => 0.08,
        MeterContinuityHistory::Degrading => 0.0,
    };
    let decay_penalty = match first_decay.history {
        MeterContinuityHistory::Degrading => 0.08,
        _ => 0.0,
    } + match final_decay.history {
        MeterContinuityHistory::Degrading => 0.12,
        _ => 0.0,
    };
    let refresh_strength = Confidence::new(
        (refresh.confidence.0 + refresh_bonus + current_bonus - decay_penalty).clamp(0.0, 1.0),
    );

    let drift_pressure = Confidence::new(
        ((unresolved.failed_revalidations as f32 * 0.18)
            + (unresolved.bars as f32 * 0.08)
            + match current {
                MeterContinuityHistory::Degrading => 0.18,
                MeterContinuityHistory::Preserving => 0.08,
                MeterContinuityHistory::Reinforcing => 0.0,
            }
            + match first_decay.history {
                MeterContinuityHistory::Degrading => 0.16,
                _ => 0.0,
            }
            + match final_decay.history {
                MeterContinuityHistory::Degrading => 0.20,
                _ => 0.0,
            })
        .clamp(0.0, 1.0),
    );

    let evidence_loss = has_cause(causes, MeterContinuityCause::EvidenceLoss);
    let irregularity = has_cause(causes, MeterContinuityCause::IrregularBarStructure);
    let phase_displacement = has_cause(causes, MeterContinuityCause::PhaseDisplacement);
    let tempo_ambiguity = has_cause(causes, MeterContinuityCause::TempoAmbiguity);
    let structural_pressure = Confidence::new(
        ((if evidence_loss { 0.42f32 } else { 0.0f32 })
            + (if irregularity { 0.28f32 } else { 0.0f32 })
            + (if phase_displacement { 0.18f32 } else { 0.0f32 })
            + (if tempo_ambiguity { 0.12f32 } else { 0.0f32 }))
        .clamp(0.0, 1.0),
    );

    MeterContinuityArcSupport {
        refresh_strength,
        drift_pressure,
        structural_pressure,
    }
}

pub fn continuity_arc_assessment(
    inputs: MeterContinuityArcInputs,
) -> (
    MeterContinuityArc,
    MeterContinuityArcRationale,
    MeterContinuityArcSupport,
) {
    let MeterContinuityArcInputs {
        source,
        reason,
        confidence,
        unresolved,
        causes,
        current,
        refresh,
        first_decay,
        final_decay,
    } = inputs;
    let has_evidence_loss = has_cause(causes, MeterContinuityCause::EvidenceLoss);
    let has_irregularity = has_cause(causes, MeterContinuityCause::IrregularBarStructure);
    let persistent_decay = matches!(first_decay.history, MeterContinuityHistory::Degrading)
        && matches!(final_decay.history, MeterContinuityHistory::Degrading);
    let support = continuity_arc_support(
        unresolved,
        causes,
        current,
        refresh,
        first_decay,
        final_decay,
    );

    if matches!(current, MeterContinuityHistory::Degrading)
        && (persistent_decay || has_evidence_loss)
    {
        return (
            MeterContinuityArc::Collapsing,
            if has_evidence_loss {
                MeterContinuityArcRationale::EvidenceLoss
            } else {
                MeterContinuityArcRationale::UnresolvedDrift
            },
            support,
        );
    }

    if matches!(refresh.history, MeterContinuityHistory::Reinforcing) && !has_evidence_loss {
        if matches!(current, MeterContinuityHistory::Reinforcing) {
            return (
                MeterContinuityArc::Recovering,
                MeterContinuityArcRationale::RefreshStrength,
                support,
            );
        }

        if matches!(current, MeterContinuityHistory::Preserving)
            && matches!(source, MeterContinuitySource::RecoveryWindow)
            && matches!(reason, MeterContinuityReason::RecoveryWindowSupport)
            && confidence.0 >= 0.80
            && unresolved.failed_revalidations <= 2
            && !has_irregularity
        {
            return (
                MeterContinuityArc::Recovering,
                MeterContinuityArcRationale::RefreshStrength,
                support,
            );
        }
    }

    if has_evidence_loss
        || (persistent_decay && confidence.0 < 0.24)
        || (matches!(current, MeterContinuityHistory::Degrading)
            && !matches!(refresh.history, MeterContinuityHistory::Reinforcing))
    {
        return (
            MeterContinuityArc::Collapsing,
            if has_evidence_loss {
                MeterContinuityArcRationale::EvidenceLoss
            } else if has_irregularity {
                MeterContinuityArcRationale::StructuralInstability
            } else {
                MeterContinuityArcRationale::UnresolvedDrift
            },
            support,
        );
    }

    (
        MeterContinuityArc::Stalling,
        if has_irregularity {
            MeterContinuityArcRationale::StructuralInstability
        } else if unresolved.failed_revalidations >= 2 {
            MeterContinuityArcRationale::UnresolvedDrift
        } else {
            MeterContinuityArcRationale::StableCarry
        },
        support,
    )
}

pub fn continuity_plan(
    plan: MeterContinuityPlanInputs,
    refresh: MeterContinuityTransition,
    first_decay: MeterContinuityTransition,
    final_decay: MeterContinuityTransition,
) -> MeterContinuityPlan {
    let MeterContinuityPlanInputs {
        action,
        source,
        reason,
        confidence,
        trigger,
        unresolved,
        causes,
        trusted_beats,
        revalidate_after_beats,
    } = plan;
    let history = continuity_history(MeterContinuityStageContext {
        action,
        source,
        reason,
        confidence,
        trigger,
        unresolved,
        causes,
        stage_index: 0,
    });
    let (arc, arc_rationale, arc_support) = continuity_arc_assessment(MeterContinuityArcInputs {
        source,
        reason,
        confidence,
        unresolved,
        causes,
        current: history,
        refresh,
        first_decay,
        final_decay,
    });
    MeterContinuityPlan {
        action,
        source,
        severity: continuity_severity(action, source),
        history,
        arc,
        arc_rationale,
        arc_support,
        reason,
        confidence,
        trigger,
        unresolved,
        causes,
        trusted_beats,
        revalidate_after_beats,
        lifecycle: MeterContinuityLifecycle {
            refresh,
            decay: [first_decay, final_decay],
        },
    }
}
