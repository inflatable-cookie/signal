use super::meter_state_continuity_context::MeterStagePlanContext;
use super::meter_state_continuity_helpers::continuity_reason;
use crate::rhythm_policy::*;

pub fn hold_arms(
    ctx: MeterStagePlanContext,
    reason: MeterStateReason,
    retained_beats: usize,
) -> MeterContinuityRecommendation {
    match reason {
        MeterStateReason::TentativeMeter => MeterContinuityRecommendation {
            bar_length: ctx.plan(
                MeterContinuityAction::Retain,
                MeterContinuitySource::CurrentMeter,
                MeterContinuityReason::TentativeEvidence,
                retained_beats.min(8),
                4,
                ctx.stage(
                    4,
                    MeterContinuityAction::Lock,
                    MeterContinuitySource::CurrentMeter,
                    MeterContinuityReason::StableEvidence,
                    0,
                ),
                ctx.stage(
                    retained_beats.min(8).saturating_add(2),
                    MeterContinuityAction::Reacquire,
                    MeterContinuitySource::CurrentMeter,
                    MeterContinuityReason::RevalidationDecay,
                    1,
                ),
                ctx.stage(
                    retained_beats.min(8).saturating_add(4),
                    MeterContinuityAction::Clear,
                    MeterContinuitySource::Cleared,
                    MeterContinuityReason::InsufficientEvidence,
                    2,
                ),
            ),
            downbeat_phase: ctx.plan(
                MeterContinuityAction::Reacquire,
                MeterContinuitySource::CurrentMeter,
                MeterContinuityReason::TentativeEvidence,
                0,
                2,
                ctx.stage(
                    2,
                    MeterContinuityAction::Lock,
                    MeterContinuitySource::CurrentMeter,
                    MeterContinuityReason::StableEvidence,
                    0,
                ),
                ctx.stage(
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
                ctx.stage(
                    6,
                    MeterContinuityAction::Clear,
                    MeterContinuitySource::Cleared,
                    MeterContinuityReason::InsufficientEvidence,
                    2,
                ),
            ),
        },
        MeterStateReason::DestabilizedHold => {
            let trailing_beats = if ctx.suppression_profile.trailing_confidence.0 > 0.0 {
                (((ctx.suppression_profile.trailing_confidence.0
                    + ctx.suppression_profile.trailing_recent_stability)
                    * 8.0)
                    .round() as usize)
                    .clamp(4, 8)
            } else {
                4
            };
            MeterContinuityRecommendation {
                bar_length: ctx.plan(
                    MeterContinuityAction::Retain,
                    MeterContinuitySource::PriorMeter,
                    MeterContinuityReason::PriorStateCarry,
                    trailing_beats,
                    4,
                    ctx.stage(
                        4,
                        MeterContinuityAction::Retain,
                        MeterContinuitySource::PriorMeter,
                        MeterContinuityReason::PriorStateCarry,
                        0,
                    ),
                    ctx.stage(
                        trailing_beats.saturating_add(2),
                        MeterContinuityAction::Reacquire,
                        MeterContinuitySource::PriorMeter,
                        MeterContinuityReason::RevalidationDecay,
                        1,
                    ),
                    ctx.stage(
                        trailing_beats.saturating_add(4),
                        MeterContinuityAction::Clear,
                        MeterContinuitySource::Cleared,
                        MeterContinuityReason::InsufficientEvidence,
                        2,
                    ),
                ),
                downbeat_phase: ctx.plan(
                    MeterContinuityAction::Retain,
                    MeterContinuitySource::PriorMeter,
                    MeterContinuityReason::PriorStateCarry,
                    trailing_beats.saturating_sub(2).max(2),
                    2,
                    ctx.stage(
                        2,
                        MeterContinuityAction::Reacquire,
                        MeterContinuitySource::PriorMeter,
                        MeterContinuityReason::RevalidationDecay,
                        1,
                    ),
                    ctx.stage(
                        trailing_beats,
                        MeterContinuityAction::Reacquire,
                        MeterContinuitySource::RecoveryWindow,
                        MeterContinuityReason::RevalidationDecay,
                        1,
                    ),
                    ctx.stage(
                        trailing_beats.saturating_add(2),
                        MeterContinuityAction::Clear,
                        MeterContinuitySource::Cleared,
                        MeterContinuityReason::InsufficientEvidence,
                        2,
                    ),
                ),
            }
        }
        _ => MeterContinuityRecommendation {
            bar_length: ctx.plan(
                MeterContinuityAction::Retain,
                MeterContinuitySource::PriorMeter,
                MeterContinuityReason::PriorStateCarry,
                retained_beats.min(6),
                4,
                ctx.stage(
                    4,
                    MeterContinuityAction::Retain,
                    MeterContinuitySource::PriorMeter,
                    MeterContinuityReason::PriorStateCarry,
                    0,
                ),
                ctx.stage(
                    retained_beats.min(6).saturating_add(2),
                    MeterContinuityAction::Reacquire,
                    MeterContinuitySource::PriorMeter,
                    MeterContinuityReason::RevalidationDecay,
                    1,
                ),
                ctx.stage(
                    retained_beats.min(6).saturating_add(4),
                    MeterContinuityAction::Clear,
                    MeterContinuitySource::Cleared,
                    MeterContinuityReason::InsufficientEvidence,
                    2,
                ),
            ),
            downbeat_phase: ctx.plan(
                MeterContinuityAction::Retain,
                MeterContinuitySource::PriorMeter,
                MeterContinuityReason::PriorStateCarry,
                retained_beats.min(4),
                2,
                ctx.stage(
                    2,
                    MeterContinuityAction::Reacquire,
                    MeterContinuitySource::PriorMeter,
                    MeterContinuityReason::RevalidationDecay,
                    1,
                ),
                ctx.stage(
                    retained_beats.min(4).saturating_add(1),
                    MeterContinuityAction::Reacquire,
                    MeterContinuitySource::RecoveryWindow,
                    MeterContinuityReason::RevalidationDecay,
                    1,
                ),
                ctx.stage(
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
