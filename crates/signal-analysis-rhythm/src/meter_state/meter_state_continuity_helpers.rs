use super::MeterSuppressionProfile;
use crate::rhythm_policy::*;
use signal_analysis::Confidence;

pub fn push_cause(
    slots: &mut [Option<MeterContinuityCause>; 3],
    count: &mut usize,
    cause: MeterContinuityCause,
) {
    if slots.iter().flatten().any(|existing| *existing == cause) {
        return;
    }
    if *count < slots.len() {
        slots[*count] = Some(cause);
        *count += 1;
    }
}

#[derive(Clone, Copy)]
pub struct MeterContinuityCauseInputs {
    pub action: MeterContinuityAction,
    pub source: MeterContinuitySource,
    pub reason: MeterContinuityReason,
    pub trigger: MeterContinuityTrigger,
    pub suppression_profile: MeterSuppressionProfile,
    pub tempo_ambiguity: Confidence,
    pub phase_displaced: bool,
    pub stage_index: usize,
}

#[derive(Clone, Copy)]
pub struct MeterContinuityStageContext {
    pub action: MeterContinuityAction,
    pub source: MeterContinuitySource,
    pub reason: MeterContinuityReason,
    pub confidence: Confidence,
    pub trigger: MeterContinuityTrigger,
    pub unresolved: MeterContinuityUnresolvedSpan,
    pub causes: MeterContinuityCauseStack,
    pub stage_index: usize,
}

#[derive(Clone, Copy)]
pub struct MeterContinuityArcInputs {
    pub source: MeterContinuitySource,
    pub reason: MeterContinuityReason,
    pub confidence: Confidence,
    pub unresolved: MeterContinuityUnresolvedSpan,
    pub causes: MeterContinuityCauseStack,
    pub current: MeterContinuityHistory,
    pub refresh: MeterContinuityTransition,
    pub first_decay: MeterContinuityTransition,
    pub final_decay: MeterContinuityTransition,
}

#[derive(Clone, Copy)]
pub struct MeterContinuityPlanInputs {
    pub action: MeterContinuityAction,
    pub source: MeterContinuitySource,
    pub reason: MeterContinuityReason,
    pub confidence: Confidence,
    pub trigger: MeterContinuityTrigger,
    pub unresolved: MeterContinuityUnresolvedSpan,
    pub causes: MeterContinuityCauseStack,
    pub trusted_beats: usize,
    pub revalidate_after_beats: usize,
}

#[derive(Clone, Copy)]
pub struct MeterContinuityInputs<'a> {
    pub estimate: Option<&'a MeterEstimate>,
    pub suppression_profile: MeterSuppressionProfile,
    pub confidence: Confidence,
    pub tempo_ambiguity: Confidence,
    pub bpm: f32,
    pub beat_positions_seconds: &'a [f32],
}

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

    match reason {
        MeterContinuityReason::StableEvidence => {
            push_cause(
                &mut causes,
                &mut count,
                MeterContinuityCause::StableMeterEvidence,
            );
        }
        MeterContinuityReason::PriorStateCarry => {
            push_cause(
                &mut causes,
                &mut count,
                MeterContinuityCause::PriorContinuityCarry,
            );
        }
        MeterContinuityReason::RecoveryWindowSupport => {
            push_cause(
                &mut causes,
                &mut count,
                MeterContinuityCause::RecoveryWindowInstability,
            );
        }
        MeterContinuityReason::PhaseDisplacement => {
            push_cause(
                &mut causes,
                &mut count,
                MeterContinuityCause::PhaseDisplacement,
            );
        }
        MeterContinuityReason::InsufficientEvidence => {
            push_cause(&mut causes, &mut count, MeterContinuityCause::EvidenceLoss);
        }
        MeterContinuityReason::TentativeEvidence | MeterContinuityReason::RevalidationDecay => {}
    }

    match trigger {
        MeterContinuityTrigger::StableRevalidation => {
            push_cause(
                &mut causes,
                &mut count,
                MeterContinuityCause::StableMeterEvidence,
            );
        }
        MeterContinuityTrigger::TentativeCarry => {
            push_cause(
                &mut causes,
                &mut count,
                MeterContinuityCause::SparseMeterSupport,
            );
        }
        MeterContinuityTrigger::PhaseRecovery => {
            push_cause(
                &mut causes,
                &mut count,
                MeterContinuityCause::PhaseDisplacement,
            );
        }
        MeterContinuityTrigger::PriorStateDrift => {
            push_cause(
                &mut causes,
                &mut count,
                MeterContinuityCause::PriorContinuityCarry,
            );
        }
        MeterContinuityTrigger::RecoveryWindowDrift => {
            push_cause(
                &mut causes,
                &mut count,
                MeterContinuityCause::RecoveryWindowInstability,
            );
        }
        MeterContinuityTrigger::EvidenceLoss => {
            push_cause(&mut causes, &mut count, MeterContinuityCause::EvidenceLoss);
        }
    }

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

pub fn continuity_history(context: MeterContinuityStageContext) -> MeterContinuityHistory {
    let MeterContinuityStageContext {
        action,
        source,
        reason,
        confidence,
        trigger,
        unresolved,
        causes,
        stage_index,
    } = context;
    let has_evidence_loss = has_cause(causes, MeterContinuityCause::EvidenceLoss);
    let has_irregularity = has_cause(causes, MeterContinuityCause::IrregularBarStructure);
    let has_phase_displacement = has_cause(causes, MeterContinuityCause::PhaseDisplacement);

    match action {
        MeterContinuityAction::Clear => MeterContinuityHistory::Degrading,
        MeterContinuityAction::Lock
            if matches!(source, MeterContinuitySource::CurrentMeter)
                && matches!(reason, MeterContinuityReason::StableEvidence)
                && matches!(trigger, MeterContinuityTrigger::StableRevalidation)
                && confidence.0 >= 0.28
                && unresolved.failed_revalidations == 0
                && !has_evidence_loss =>
        {
            MeterContinuityHistory::Reinforcing
        }
        MeterContinuityAction::Lock => MeterContinuityHistory::Preserving,
        MeterContinuityAction::Retain
            if stage_index > 0
                || has_evidence_loss
                || (matches!(source, MeterContinuitySource::PriorMeter)
                    && unresolved.failed_revalidations >= 2)
                || (matches!(source, MeterContinuitySource::RecoveryWindow)
                    && has_irregularity
                    && confidence.0 < 0.30) =>
        {
            MeterContinuityHistory::Degrading
        }
        MeterContinuityAction::Retain => MeterContinuityHistory::Preserving,
        MeterContinuityAction::Reacquire
            if matches!(reason, MeterContinuityReason::PhaseDisplacement)
                || stage_index > 0
                || unresolved.failed_revalidations > 0
                || has_evidence_loss
                || has_phase_displacement =>
        {
            MeterContinuityHistory::Degrading
        }
        MeterContinuityAction::Reacquire => MeterContinuityHistory::Preserving,
    }
}

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

pub fn continuity_trigger(
    action: MeterContinuityAction,
    source: MeterContinuitySource,
    reason: MeterContinuityReason,
) -> MeterContinuityTrigger {
    match reason {
        MeterContinuityReason::StableEvidence => MeterContinuityTrigger::StableRevalidation,
        MeterContinuityReason::TentativeEvidence => MeterContinuityTrigger::TentativeCarry,
        MeterContinuityReason::PriorStateCarry => MeterContinuityTrigger::PriorStateDrift,
        MeterContinuityReason::RecoveryWindowSupport => MeterContinuityTrigger::RecoveryWindowDrift,
        MeterContinuityReason::PhaseDisplacement => MeterContinuityTrigger::PhaseRecovery,
        MeterContinuityReason::RevalidationDecay => match source {
            MeterContinuitySource::PriorMeter => MeterContinuityTrigger::PriorStateDrift,
            MeterContinuitySource::RecoveryWindow => MeterContinuityTrigger::RecoveryWindowDrift,
            MeterContinuitySource::CurrentMeter => match action {
                MeterContinuityAction::Retain | MeterContinuityAction::Reacquire => {
                    MeterContinuityTrigger::TentativeCarry
                }
                MeterContinuityAction::Lock => MeterContinuityTrigger::StableRevalidation,
                MeterContinuityAction::Clear => MeterContinuityTrigger::EvidenceLoss,
            },
            MeterContinuitySource::Cleared => MeterContinuityTrigger::EvidenceLoss,
        },
        MeterContinuityReason::InsufficientEvidence => MeterContinuityTrigger::EvidenceLoss,
    }
}

pub fn unresolved_span(
    trigger: MeterContinuityTrigger,
    beat_span: usize,
    revalidate_after_beats: usize,
    beats_per_bar: usize,
    phase_displacement_beats: usize,
    stage_index: usize,
) -> MeterContinuityUnresolvedSpan {
    let beats = match trigger {
        MeterContinuityTrigger::StableRevalidation => 0,
        MeterContinuityTrigger::PhaseRecovery => beat_span
            .max(revalidate_after_beats)
            .max(phase_displacement_beats.max(1)),
        MeterContinuityTrigger::TentativeCarry
        | MeterContinuityTrigger::PriorStateDrift
        | MeterContinuityTrigger::RecoveryWindowDrift => {
            beat_span.max(revalidate_after_beats.max(1))
        }
        MeterContinuityTrigger::EvidenceLoss => beat_span,
    };
    let bars = if beats == 0 {
        0
    } else {
        beats.div_ceil(beats_per_bar.max(1))
    };
    let failed_revalidations = if beats == 0 || revalidate_after_beats == 0 {
        0
    } else {
        beats.div_ceil(revalidate_after_beats).max(stage_index)
    };
    MeterContinuityUnresolvedSpan {
        beats,
        bars,
        failed_revalidations,
    }
}

pub fn continuity_reason(
    action: MeterContinuityAction,
    source: MeterContinuitySource,
    phase_displaced: bool,
    is_decay: bool,
) -> MeterContinuityReason {
    if matches!(action, MeterContinuityAction::Clear)
        || matches!(source, MeterContinuitySource::Cleared)
    {
        return MeterContinuityReason::InsufficientEvidence;
    }

    if phase_displaced && matches!(action, MeterContinuityAction::Reacquire) {
        return MeterContinuityReason::PhaseDisplacement;
    }

    if is_decay {
        return MeterContinuityReason::RevalidationDecay;
    }

    match source {
        MeterContinuitySource::CurrentMeter => match action {
            MeterContinuityAction::Lock => MeterContinuityReason::StableEvidence,
            MeterContinuityAction::Retain | MeterContinuityAction::Reacquire => {
                MeterContinuityReason::TentativeEvidence
            }
            MeterContinuityAction::Clear => MeterContinuityReason::InsufficientEvidence,
        },
        MeterContinuitySource::PriorMeter => MeterContinuityReason::PriorStateCarry,
        MeterContinuitySource::RecoveryWindow => MeterContinuityReason::RecoveryWindowSupport,
        MeterContinuitySource::Cleared => MeterContinuityReason::InsufficientEvidence,
    }
}

pub fn continuity_severity(
    action: MeterContinuityAction,
    source: MeterContinuitySource,
) -> MeterContinuitySeverity {
    match action {
        MeterContinuityAction::Lock => MeterContinuitySeverity::Confirmed,
        MeterContinuityAction::Retain => match source {
            MeterContinuitySource::CurrentMeter | MeterContinuitySource::RecoveryWindow => {
                MeterContinuitySeverity::Guarded
            }
            MeterContinuitySource::PriorMeter => MeterContinuitySeverity::Fragile,
            MeterContinuitySource::Cleared => MeterContinuitySeverity::Cleared,
        },
        MeterContinuityAction::Reacquire => MeterContinuitySeverity::Fragile,
        MeterContinuityAction::Clear => MeterContinuitySeverity::Cleared,
    }
}

pub fn continuity_confidence(
    action: MeterContinuityAction,
    source: MeterContinuitySource,
    state_confidence: Confidence,
    beat_span: usize,
    stage_index: usize,
) -> Confidence {
    let action_scale = match action {
        MeterContinuityAction::Lock => 1.0,
        MeterContinuityAction::Retain => 0.72,
        MeterContinuityAction::Reacquire => 0.45,
        MeterContinuityAction::Clear => 0.0,
    };
    let source_bias = match source {
        MeterContinuitySource::CurrentMeter => 0.12,
        MeterContinuitySource::RecoveryWindow => 0.06,
        MeterContinuitySource::PriorMeter => -0.02,
        MeterContinuitySource::Cleared => -0.30,
    };
    let span_bias = (beat_span as f32 / 24.0).clamp(0.0, 0.25);
    let decay_penalty = stage_index as f32 * 0.12;
    Confidence::new(
        (state_confidence.0 * action_scale + source_bias + span_bias - decay_penalty)
            .clamp(0.0, 1.0),
    )
}

pub fn transition(
    after_beats: usize,
    context: MeterContinuityStageContext,
) -> MeterContinuityTransition {
    let MeterContinuityStageContext {
        action,
        source,
        reason,
        confidence,
        trigger,
        unresolved,
        causes,
        ..
    } = context;
    MeterContinuityTransition {
        after_beats,
        action,
        source,
        severity: continuity_severity(action, source),
        history: continuity_history(context),
        reason,
        confidence,
        trigger,
        unresolved,
        causes,
    }
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
