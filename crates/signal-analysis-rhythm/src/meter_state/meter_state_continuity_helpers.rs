use crate::rhythm_policy::*;
use signal_analysis::Confidence;

pub use super::meter_state_continuity_cause_stack::cause_stack;
pub use super::meter_state_continuity_cause_stack::has_cause;
use super::meter_state_continuity_rule_surface::{reason_for_stage, trigger_for_reason};
pub use super::meter_state_continuity_types::*;

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

pub fn continuity_trigger(
    action: MeterContinuityAction,
    source: MeterContinuitySource,
    reason: MeterContinuityReason,
) -> MeterContinuityTrigger {
    trigger_for_reason(action, source, reason)
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
    reason_for_stage(action, source, phase_displaced, is_decay)
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
