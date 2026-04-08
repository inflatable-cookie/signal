use super::meter_state_continuity_context::MeterStagePlanContext;
use crate::rhythm_policy::*;

pub fn lock_arms(
    ctx: MeterStagePlanContext,
    pickup_like_phase: bool,
    retained_beats: usize,
) -> MeterContinuityRecommendation {
    let _ = retained_beats;
    if pickup_like_phase {
        MeterContinuityRecommendation {
            bar_length: ctx.plan(
                MeterContinuityAction::Lock,
                MeterContinuitySource::CurrentMeter,
                MeterContinuityReason::StableEvidence,
                16,
                16,
                ctx.stage(
                    16,
                    MeterContinuityAction::Lock,
                    MeterContinuitySource::CurrentMeter,
                    MeterContinuityReason::StableEvidence,
                    0,
                ),
                ctx.stage(
                    24,
                    MeterContinuityAction::Retain,
                    MeterContinuitySource::CurrentMeter,
                    MeterContinuityReason::RevalidationDecay,
                    1,
                ),
                ctx.stage(
                    32,
                    MeterContinuityAction::Clear,
                    MeterContinuitySource::Cleared,
                    MeterContinuityReason::InsufficientEvidence,
                    2,
                ),
            ),
            downbeat_phase: ctx.plan(
                MeterContinuityAction::Reacquire,
                MeterContinuitySource::CurrentMeter,
                MeterContinuityReason::PhaseDisplacement,
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
                    MeterContinuityReason::PhaseDisplacement,
                    1,
                ),
                ctx.stage(
                    8,
                    MeterContinuityAction::Clear,
                    MeterContinuitySource::Cleared,
                    MeterContinuityReason::InsufficientEvidence,
                    2,
                ),
            ),
        }
    } else {
        MeterContinuityRecommendation {
            bar_length: ctx.plan(
                MeterContinuityAction::Lock,
                MeterContinuitySource::CurrentMeter,
                MeterContinuityReason::StableEvidence,
                16,
                16,
                ctx.stage(
                    16,
                    MeterContinuityAction::Lock,
                    MeterContinuitySource::CurrentMeter,
                    MeterContinuityReason::StableEvidence,
                    0,
                ),
                ctx.stage(
                    24,
                    MeterContinuityAction::Retain,
                    MeterContinuitySource::CurrentMeter,
                    MeterContinuityReason::RevalidationDecay,
                    1,
                ),
                ctx.stage(
                    32,
                    MeterContinuityAction::Clear,
                    MeterContinuitySource::Cleared,
                    MeterContinuityReason::InsufficientEvidence,
                    2,
                ),
            ),
            downbeat_phase: ctx.plan(
                MeterContinuityAction::Lock,
                MeterContinuitySource::CurrentMeter,
                MeterContinuityReason::StableEvidence,
                16,
                16,
                ctx.stage(
                    16,
                    MeterContinuityAction::Lock,
                    MeterContinuitySource::CurrentMeter,
                    MeterContinuityReason::StableEvidence,
                    0,
                ),
                ctx.stage(
                    24,
                    MeterContinuityAction::Retain,
                    MeterContinuitySource::CurrentMeter,
                    MeterContinuityReason::RevalidationDecay,
                    1,
                ),
                ctx.stage(
                    32,
                    MeterContinuityAction::Clear,
                    MeterContinuitySource::Cleared,
                    MeterContinuityReason::InsufficientEvidence,
                    2,
                ),
            ),
        }
    }
}
