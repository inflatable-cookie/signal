use super::meter_state_continuity_context::MeterStagePlanContext;
use super::meter_state_continuity_helpers::MeterContinuityInputs;
use super::meter_state_continuity_hold_arms::hold_arms;
use super::meter_state_continuity_lock_arms::lock_arms;
use super::meter_state_continuity_watch_clear_arms::{clear_arm, watch_arm};
use crate::rhythm_policy::*;

pub fn continuity_for(
    action: MeterStateAction,
    reason: MeterStateReason,
    inputs: MeterContinuityInputs<'_>,
) -> MeterContinuityRecommendation {
    let MeterContinuityInputs {
        estimate,
        suppression_profile,
        confidence,
        tempo_ambiguity,
        bpm,
        beat_positions_seconds,
    } = inputs;
    let beat_duration = if bpm > 0.0 { 60.0 / bpm } else { 0.0 };
    let pickup_like_phase = estimate
        .and_then(|estimate| estimate.downbeat_positions_seconds.first().copied())
        .map(|first_downbeat| beat_duration > 0.0 && first_downbeat >= beat_duration * 1.5)
        .unwrap_or(false);
    let phase_displacement_beats = estimate
        .and_then(|estimate| estimate.downbeat_positions_seconds.first().copied())
        .map(|first_downbeat| {
            if beat_duration > 0.0 {
                let downbeat_guard = first_downbeat - beat_duration * 0.25;
                beat_positions_seconds
                    .iter()
                    .copied()
                    .take_while(|&beat| beat < downbeat_guard)
                    .count()
            } else {
                0
            }
        })
        .unwrap_or(0);
    let recovery_beats = estimate
        .and_then(|estimate| estimate.recovery.as_ref())
        .map(|recovery| recovery.recovered_beats)
        .unwrap_or(0);
    let beats_per_bar = estimate
        .map(|estimate| estimate.beats_per_bar)
        .unwrap_or(4)
        .max(1);
    let support_beats = ((confidence.0 * 12.0).round() as usize).clamp(2, 12);
    let retained_beats = if estimate.is_some() {
        recovery_beats.clamp(6, 24).max(support_beats)
    } else {
        support_beats
    };

    let ctx = MeterStagePlanContext {
        confidence,
        tempo_ambiguity,
        beats_per_bar,
        phase_displacement_beats,
        suppression_profile,
    };

    match action {
        MeterStateAction::Lock => lock_arms(ctx, pickup_like_phase, retained_beats),
        MeterStateAction::Hold => hold_arms(ctx, reason, retained_beats),
        MeterStateAction::Watch => watch_arm(ctx, retained_beats),
        MeterStateAction::Clear => clear_arm(ctx),
    }
}

pub fn build_meter_state(
    action: MeterStateAction,
    reason: MeterStateReason,
    inputs: MeterContinuityInputs<'_>,
) -> MeterStateRecommendation {
    MeterStateRecommendation {
        action,
        reason,
        confidence: inputs.confidence,
        continuity: continuity_for(action, reason, inputs),
    }
}
