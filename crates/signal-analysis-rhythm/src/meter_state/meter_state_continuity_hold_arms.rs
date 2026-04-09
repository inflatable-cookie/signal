use super::meter_state_continuity_context::MeterStagePlanContext;
use super::meter_state_continuity_helpers::continuity_reason;
use super::meter_state_continuity_plan_shell::{build_recommendation, plan, stage};
use crate::rhythm_policy::*;

pub fn hold_arms(
    ctx: MeterStagePlanContext,
    reason: MeterStateReason,
    retained_beats: usize,
) -> MeterContinuityRecommendation {
    match reason {
        MeterStateReason::TentativeMeter => build_recommendation(
            ctx,
            plan(
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
            plan(
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
        ),
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
            build_recommendation(
                ctx,
                plan(
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
                plan(
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
            )
        }
        _ => build_recommendation(
            ctx,
            plan(
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
            plan(
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
        ),
    }
}
