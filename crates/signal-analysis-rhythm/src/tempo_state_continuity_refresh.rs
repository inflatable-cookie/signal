use signal_analysis::Confidence;

use crate::tempo_state_continuity_basics::has_tempo_cause;
use crate::{
    TempoContinuityAction, TempoContinuityCause, TempoContinuityCauseStack, TempoContinuityHistory,
    TempoContinuitySource, TempoContinuityUnresolvedSpan,
};

pub(crate) fn continuity_refresh_strength(
    action: TempoContinuityAction,
    source: TempoContinuitySource,
    confidence: Confidence,
    history: TempoContinuityHistory,
    unresolved: TempoContinuityUnresolvedSpan,
    causes: TempoContinuityCauseStack,
    beat_span: usize,
) -> Confidence {
    if matches!(action, TempoContinuityAction::Clear)
        || matches!(source, TempoContinuitySource::Cleared)
    {
        return Confidence::new(0.0);
    }

    let action_scale = match action {
        TempoContinuityAction::Lock => 0.96,
        TempoContinuityAction::Retain => 0.76,
        TempoContinuityAction::Reacquire => 0.64,
        TempoContinuityAction::Clear => 0.0,
    };
    let source_bias = match source {
        TempoContinuitySource::CurrentTempo => 0.10,
        TempoContinuitySource::CoreWindow => 0.04,
        TempoContinuitySource::PriorTempo => -0.06,
        TempoContinuitySource::Cleared => -0.30,
    };
    let history_bias = match history {
        TempoContinuityHistory::Reinforcing => 0.16,
        TempoContinuityHistory::Preserving => 0.06,
        TempoContinuityHistory::Degrading => -0.12,
    };
    let span_bias = (beat_span as f32 / 16.0).min(1.0) * 0.10;
    let unresolved_penalty = unresolved.failed_revalidations as f32 * 0.08;
    let cause_penalty = (if has_tempo_cause(causes, TempoContinuityCause::BoundaryDrift) {
        0.10
    } else {
        0.0
    }) + (if has_tempo_cause(causes, TempoContinuityCause::TempoAmbiguity) {
        0.08
    } else {
        0.0
    }) + (if has_tempo_cause(causes, TempoContinuityCause::PriorTempoCarry) {
        0.12
    } else {
        0.0
    }) + (if has_tempo_cause(causes, TempoContinuityCause::EvidenceLoss) {
        0.20
    } else {
        0.0
    });
    let evidence_bonus = if has_tempo_cause(causes, TempoContinuityCause::StableTempoEvidence) {
        0.06
    } else {
        0.0
    };

    Confidence::new(
        (confidence.0 * action_scale + source_bias + history_bias + span_bias + evidence_bonus
            - unresolved_penalty
            - cause_penalty)
            .clamp(0.0, 1.0),
    )
}
