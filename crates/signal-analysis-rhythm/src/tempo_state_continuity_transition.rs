use signal_analysis::Confidence;

use crate::tempo_state_continuity_basics::{
    continuity_cause_stack, continuity_history, continuity_provenance, continuity_severity,
    continuity_trigger, unresolved_span,
};
use crate::tempo_state_continuity_refresh::continuity_refresh_strength;
use crate::{
    TempoContinuityAction, TempoContinuityExpiry, TempoContinuityReason, TempoContinuitySource,
    TempoContinuityTransition,
};

#[derive(Clone, Copy)]
pub(crate) struct TempoContinuityTransitionInputs {
    pub(crate) after_beats: usize,
    pub(crate) action: TempoContinuityAction,
    pub(crate) source: TempoContinuitySource,
    pub(crate) reason: TempoContinuityReason,
    pub(crate) boundary_pressure: Confidence,
    pub(crate) tempo_ambiguity: Confidence,
    pub(crate) revalidate_after_beats: usize,
    pub(crate) stage_index: usize,
    pub(crate) confidence: Confidence,
}

pub(crate) fn continuity_transition(
    inputs: TempoContinuityTransitionInputs,
) -> TempoContinuityTransition {
    let TempoContinuityTransitionInputs {
        after_beats,
        action,
        source,
        reason,
        boundary_pressure,
        tempo_ambiguity,
        revalidate_after_beats,
        stage_index,
        confidence,
    } = inputs;
    let trigger = continuity_trigger(action, source, reason, boundary_pressure, tempo_ambiguity);
    let unresolved = unresolved_span(trigger, after_beats, revalidate_after_beats, stage_index);
    let causes = continuity_cause_stack(action, source, reason, boundary_pressure, tempo_ambiguity);
    let severity = continuity_severity(action, source);
    let history = continuity_history(
        action,
        source,
        reason,
        trigger,
        unresolved,
        causes,
        stage_index,
    );
    TempoContinuityTransition {
        after_beats,
        action,
        source,
        severity,
        history,
        reason,
        trigger,
        unresolved,
        causes,
        provenance: continuity_provenance(action, source, reason),
        confidence,
        refresh_strength: continuity_refresh_strength(
            action,
            source,
            confidence,
            history,
            unresolved,
            causes,
            after_beats,
        ),
    }
}

pub(crate) fn continuity_expiry(
    trusted_beats: usize,
    revalidate_after_beats: usize,
    first_decay: TempoContinuityTransition,
    final_decay: TempoContinuityTransition,
) -> TempoContinuityExpiry {
    let max_failed_revalidations = if revalidate_after_beats == 0 || final_decay.after_beats == 0 {
        0
    } else {
        final_decay.after_beats.div_ceil(revalidate_after_beats)
    };
    TempoContinuityExpiry {
        guaranteed_until_beats: trusted_beats,
        downgrade_after_beats: first_decay.after_beats,
        clear_after_beats: final_decay.after_beats,
        max_failed_revalidations,
    }
}
