use super::meter_state_continuity_helpers::*;
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
    let stage = |after_beats: usize,
                 stage_action: MeterContinuityAction,
                 stage_source: MeterContinuitySource,
                 stage_reason: MeterContinuityReason,
                 stage_index: usize| {
        let stage_trigger = continuity_trigger(stage_action, stage_source, stage_reason);
        let stage_unresolved = unresolved_span(
            stage_trigger,
            after_beats,
            after_beats,
            beats_per_bar,
            phase_displacement_beats,
            stage_index,
        );
        let stage_causes = cause_stack(MeterContinuityCauseInputs {
            action: stage_action,
            source: stage_source,
            reason: stage_reason,
            trigger: stage_trigger,
            suppression_profile,
            tempo_ambiguity,
            phase_displaced: phase_displacement_beats > 0,
            stage_index,
        });
        transition(
            after_beats,
            MeterContinuityStageContext {
                action: stage_action,
                source: stage_source,
                reason: stage_reason,
                confidence: continuity_confidence(
                    stage_action,
                    stage_source,
                    confidence,
                    after_beats,
                    stage_index,
                ),
                trigger: stage_trigger,
                unresolved: stage_unresolved,
                causes: stage_causes,
                stage_index,
            },
        )
    };
    let plan = |plan_action: MeterContinuityAction,
                plan_source: MeterContinuitySource,
                plan_reason: MeterContinuityReason,
                trusted_beats: usize,
                revalidate_after_beats: usize,
                refresh: MeterContinuityTransition,
                first_decay: MeterContinuityTransition,
                final_decay: MeterContinuityTransition| {
        let plan_trigger = continuity_trigger(plan_action, plan_source, plan_reason);
        let plan_unresolved = unresolved_span(
            plan_trigger,
            trusted_beats,
            revalidate_after_beats,
            beats_per_bar,
            phase_displacement_beats,
            0,
        );
        let plan_causes = cause_stack(MeterContinuityCauseInputs {
            action: plan_action,
            source: plan_source,
            reason: plan_reason,
            trigger: plan_trigger,
            suppression_profile,
            tempo_ambiguity,
            phase_displaced: phase_displacement_beats > 0,
            stage_index: 0,
        });
        continuity_plan(
            MeterContinuityPlanInputs {
                action: plan_action,
                source: plan_source,
                reason: plan_reason,
                confidence: continuity_confidence(
                    plan_action,
                    plan_source,
                    confidence,
                    trusted_beats,
                    0,
                ),
                trigger: plan_trigger,
                unresolved: plan_unresolved,
                causes: plan_causes,
                trusted_beats,
                revalidate_after_beats,
            },
            refresh,
            first_decay,
            final_decay,
        )
    };

    match (action, reason) {
        (MeterStateAction::Lock, _) if pickup_like_phase => MeterContinuityRecommendation {
            bar_length: plan(
                MeterContinuityAction::Lock,
                MeterContinuitySource::CurrentMeter,
                MeterContinuityReason::StableEvidence,
                16,
                16,
                stage(
                    16,
                    MeterContinuityAction::Lock,
                    MeterContinuitySource::CurrentMeter,
                    MeterContinuityReason::StableEvidence,
                    0,
                ),
                stage(
                    24,
                    MeterContinuityAction::Retain,
                    MeterContinuitySource::CurrentMeter,
                    MeterContinuityReason::RevalidationDecay,
                    1,
                ),
                stage(
                    32,
                    MeterContinuityAction::Clear,
                    MeterContinuitySource::Cleared,
                    MeterContinuityReason::InsufficientEvidence,
                    2,
                ),
            ),
            downbeat_phase: plan(
                MeterContinuityAction::Reacquire,
                MeterContinuitySource::CurrentMeter,
                MeterContinuityReason::PhaseDisplacement,
                0,
                2,
                stage(
                    2,
                    MeterContinuityAction::Lock,
                    MeterContinuitySource::CurrentMeter,
                    MeterContinuityReason::StableEvidence,
                    0,
                ),
                stage(
                    4,
                    MeterContinuityAction::Reacquire,
                    MeterContinuitySource::CurrentMeter,
                    MeterContinuityReason::PhaseDisplacement,
                    1,
                ),
                stage(
                    8,
                    MeterContinuityAction::Clear,
                    MeterContinuitySource::Cleared,
                    MeterContinuityReason::InsufficientEvidence,
                    2,
                ),
            ),
        },
        (MeterStateAction::Lock, _) => MeterContinuityRecommendation {
            bar_length: plan(
                MeterContinuityAction::Lock,
                MeterContinuitySource::CurrentMeter,
                MeterContinuityReason::StableEvidence,
                16,
                16,
                stage(
                    16,
                    MeterContinuityAction::Lock,
                    MeterContinuitySource::CurrentMeter,
                    MeterContinuityReason::StableEvidence,
                    0,
                ),
                stage(
                    24,
                    MeterContinuityAction::Retain,
                    MeterContinuitySource::CurrentMeter,
                    MeterContinuityReason::RevalidationDecay,
                    1,
                ),
                stage(
                    32,
                    MeterContinuityAction::Clear,
                    MeterContinuitySource::Cleared,
                    MeterContinuityReason::InsufficientEvidence,
                    2,
                ),
            ),
            downbeat_phase: plan(
                MeterContinuityAction::Lock,
                MeterContinuitySource::CurrentMeter,
                MeterContinuityReason::StableEvidence,
                16,
                16,
                stage(
                    16,
                    MeterContinuityAction::Lock,
                    MeterContinuitySource::CurrentMeter,
                    MeterContinuityReason::StableEvidence,
                    0,
                ),
                stage(
                    24,
                    MeterContinuityAction::Retain,
                    MeterContinuitySource::CurrentMeter,
                    MeterContinuityReason::RevalidationDecay,
                    1,
                ),
                stage(
                    32,
                    MeterContinuityAction::Clear,
                    MeterContinuitySource::Cleared,
                    MeterContinuityReason::InsufficientEvidence,
                    2,
                ),
            ),
        },
        (MeterStateAction::Hold, MeterStateReason::TentativeMeter) => {
            MeterContinuityRecommendation {
                bar_length: plan(
                    MeterContinuityAction::Retain,
                    MeterContinuitySource::CurrentMeter,
                    MeterContinuityReason::TentativeEvidence,
                    retained_beats.min(8),
                    4,
                    stage(
                        4,
                        MeterContinuityAction::Lock,
                        MeterContinuitySource::CurrentMeter,
                        MeterContinuityReason::StableEvidence,
                        0,
                    ),
                    stage(
                        retained_beats.min(8).saturating_add(2),
                        MeterContinuityAction::Reacquire,
                        MeterContinuitySource::CurrentMeter,
                        MeterContinuityReason::RevalidationDecay,
                        1,
                    ),
                    stage(
                        retained_beats.min(8).saturating_add(4),
                        MeterContinuityAction::Clear,
                        MeterContinuitySource::Cleared,
                        MeterContinuityReason::InsufficientEvidence,
                        2,
                    ),
                ),
                downbeat_phase: plan(
                    MeterContinuityAction::Reacquire,
                    MeterContinuitySource::CurrentMeter,
                    MeterContinuityReason::TentativeEvidence,
                    0,
                    2,
                    stage(
                        2,
                        MeterContinuityAction::Lock,
                        MeterContinuitySource::CurrentMeter,
                        MeterContinuityReason::StableEvidence,
                        0,
                    ),
                    stage(
                        4,
                        MeterContinuityAction::Reacquire,
                        MeterContinuitySource::CurrentMeter,
                        continuity_reason(
                            MeterContinuityAction::Reacquire,
                            MeterContinuitySource::CurrentMeter,
                            false,
                            true,
                        ),
                        1,
                    ),
                    stage(
                        6,
                        MeterContinuityAction::Clear,
                        MeterContinuitySource::Cleared,
                        MeterContinuityReason::InsufficientEvidence,
                        2,
                    ),
                ),
            }
        }
        (MeterStateAction::Hold, MeterStateReason::DestabilizedHold) => {
            let trailing_beats = if suppression_profile.trailing_confidence.0 > 0.0 {
                (((suppression_profile.trailing_confidence.0
                    + suppression_profile.trailing_recent_stability)
                    * 8.0)
                    .round() as usize)
                    .clamp(4, 8)
            } else {
                4
            };
            MeterContinuityRecommendation {
                bar_length: plan(
                    MeterContinuityAction::Retain,
                    MeterContinuitySource::PriorMeter,
                    MeterContinuityReason::PriorStateCarry,
                    trailing_beats,
                    4,
                    stage(
                        4,
                        MeterContinuityAction::Retain,
                        MeterContinuitySource::PriorMeter,
                        MeterContinuityReason::PriorStateCarry,
                        0,
                    ),
                    stage(
                        trailing_beats.saturating_add(2),
                        MeterContinuityAction::Reacquire,
                        MeterContinuitySource::PriorMeter,
                        MeterContinuityReason::RevalidationDecay,
                        1,
                    ),
                    stage(
                        trailing_beats.saturating_add(4),
                        MeterContinuityAction::Clear,
                        MeterContinuitySource::Cleared,
                        MeterContinuityReason::InsufficientEvidence,
                        2,
                    ),
                ),
                downbeat_phase: plan(
                    MeterContinuityAction::Retain,
                    MeterContinuitySource::PriorMeter,
                    MeterContinuityReason::PriorStateCarry,
                    trailing_beats.saturating_sub(2).max(2),
                    2,
                    stage(
                        2,
                        MeterContinuityAction::Reacquire,
                        MeterContinuitySource::PriorMeter,
                        MeterContinuityReason::RevalidationDecay,
                        1,
                    ),
                    stage(
                        trailing_beats,
                        MeterContinuityAction::Reacquire,
                        MeterContinuitySource::RecoveryWindow,
                        MeterContinuityReason::RevalidationDecay,
                        1,
                    ),
                    stage(
                        trailing_beats.saturating_add(2),
                        MeterContinuityAction::Clear,
                        MeterContinuitySource::Cleared,
                        MeterContinuityReason::InsufficientEvidence,
                        2,
                    ),
                ),
            }
        }
        (MeterStateAction::Watch, _) => MeterContinuityRecommendation {
            bar_length: plan(
                MeterContinuityAction::Retain,
                MeterContinuitySource::RecoveryWindow,
                MeterContinuityReason::RecoveryWindowSupport,
                retained_beats,
                retained_beats.saturating_div(2).max(4),
                stage(
                    retained_beats.saturating_div(2).max(4),
                    MeterContinuityAction::Lock,
                    MeterContinuitySource::CurrentMeter,
                    MeterContinuityReason::StableEvidence,
                    0,
                ),
                stage(
                    retained_beats.saturating_add(4),
                    MeterContinuityAction::Reacquire,
                    MeterContinuitySource::RecoveryWindow,
                    MeterContinuityReason::RevalidationDecay,
                    1,
                ),
                stage(
                    retained_beats.saturating_add(8),
                    MeterContinuityAction::Clear,
                    MeterContinuitySource::Cleared,
                    MeterContinuityReason::InsufficientEvidence,
                    2,
                ),
            ),
            downbeat_phase: plan(
                MeterContinuityAction::Reacquire,
                MeterContinuitySource::RecoveryWindow,
                MeterContinuityReason::RecoveryWindowSupport,
                0,
                2,
                stage(
                    2,
                    MeterContinuityAction::Lock,
                    MeterContinuitySource::CurrentMeter,
                    MeterContinuityReason::StableEvidence,
                    0,
                ),
                stage(
                    4,
                    MeterContinuityAction::Reacquire,
                    MeterContinuitySource::RecoveryWindow,
                    MeterContinuityReason::RevalidationDecay,
                    1,
                ),
                stage(
                    6,
                    MeterContinuityAction::Clear,
                    MeterContinuitySource::Cleared,
                    MeterContinuityReason::InsufficientEvidence,
                    2,
                ),
            ),
        },
        (MeterStateAction::Clear, _) => MeterContinuityRecommendation {
            bar_length: plan(
                MeterContinuityAction::Clear,
                MeterContinuitySource::Cleared,
                MeterContinuityReason::InsufficientEvidence,
                0,
                0,
                stage(
                    0,
                    MeterContinuityAction::Clear,
                    MeterContinuitySource::Cleared,
                    MeterContinuityReason::InsufficientEvidence,
                    2,
                ),
                stage(
                    0,
                    MeterContinuityAction::Clear,
                    MeterContinuitySource::Cleared,
                    MeterContinuityReason::InsufficientEvidence,
                    2,
                ),
                stage(
                    0,
                    MeterContinuityAction::Clear,
                    MeterContinuitySource::Cleared,
                    MeterContinuityReason::InsufficientEvidence,
                    2,
                ),
            ),
            downbeat_phase: plan(
                MeterContinuityAction::Clear,
                MeterContinuitySource::Cleared,
                MeterContinuityReason::InsufficientEvidence,
                0,
                0,
                stage(
                    0,
                    MeterContinuityAction::Clear,
                    MeterContinuitySource::Cleared,
                    MeterContinuityReason::InsufficientEvidence,
                    2,
                ),
                stage(
                    0,
                    MeterContinuityAction::Clear,
                    MeterContinuitySource::Cleared,
                    MeterContinuityReason::InsufficientEvidence,
                    2,
                ),
                stage(
                    0,
                    MeterContinuityAction::Clear,
                    MeterContinuitySource::Cleared,
                    MeterContinuityReason::InsufficientEvidence,
                    2,
                ),
            ),
        },
        (MeterStateAction::Hold, _) => MeterContinuityRecommendation {
            bar_length: plan(
                MeterContinuityAction::Retain,
                MeterContinuitySource::PriorMeter,
                MeterContinuityReason::PriorStateCarry,
                retained_beats.min(6),
                4,
                stage(
                    4,
                    MeterContinuityAction::Retain,
                    MeterContinuitySource::PriorMeter,
                    MeterContinuityReason::PriorStateCarry,
                    0,
                ),
                stage(
                    retained_beats.min(6).saturating_add(2),
                    MeterContinuityAction::Reacquire,
                    MeterContinuitySource::PriorMeter,
                    MeterContinuityReason::RevalidationDecay,
                    1,
                ),
                stage(
                    retained_beats.min(6).saturating_add(4),
                    MeterContinuityAction::Clear,
                    MeterContinuitySource::Cleared,
                    MeterContinuityReason::InsufficientEvidence,
                    2,
                ),
            ),
            downbeat_phase: plan(
                MeterContinuityAction::Retain,
                MeterContinuitySource::PriorMeter,
                MeterContinuityReason::PriorStateCarry,
                retained_beats.min(4),
                2,
                stage(
                    2,
                    MeterContinuityAction::Reacquire,
                    MeterContinuitySource::PriorMeter,
                    MeterContinuityReason::RevalidationDecay,
                    1,
                ),
                stage(
                    retained_beats.min(4).saturating_add(1),
                    MeterContinuityAction::Reacquire,
                    MeterContinuitySource::RecoveryWindow,
                    MeterContinuityReason::RevalidationDecay,
                    1,
                ),
                stage(
                    retained_beats.min(4).saturating_add(2),
                    MeterContinuityAction::Clear,
                    MeterContinuitySource::Cleared,
                    MeterContinuityReason::InsufficientEvidence,
                    2,
                ),
            ),
        },
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
