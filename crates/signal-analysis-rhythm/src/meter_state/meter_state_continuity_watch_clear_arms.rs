use super::meter_state_continuity_context::MeterStagePlanContext;
use crate::rhythm_policy::*;

pub fn watch_arm(
    ctx: MeterStagePlanContext,
    retained_beats: usize,
) -> MeterContinuityRecommendation {
    MeterContinuityRecommendation {
        bar_length: ctx.plan(
            MeterContinuityAction::Retain,
            MeterContinuitySource::RecoveryWindow,
            MeterContinuityReason::RecoveryWindowSupport,
            retained_beats,
            retained_beats.saturating_div(2).max(4),
            ctx.stage(
                retained_beats.saturating_div(2).max(4),
                MeterContinuityAction::Lock,
                MeterContinuitySource::CurrentMeter,
                MeterContinuityReason::StableEvidence,
                0,
            ),
            ctx.stage(
                retained_beats.saturating_add(4),
                MeterContinuityAction::Reacquire,
                MeterContinuitySource::RecoveryWindow,
                MeterContinuityReason::RevalidationDecay,
                1,
            ),
            ctx.stage(
                retained_beats.saturating_add(8),
                MeterContinuityAction::Clear,
                MeterContinuitySource::Cleared,
                MeterContinuityReason::InsufficientEvidence,
                2,
            ),
        ),
        downbeat_phase: ctx.plan(
            MeterContinuityAction::Reacquire,
            MeterContinuitySource::RecoveryWindow,
            MeterContinuityReason::RecoveryWindowSupport,
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
                MeterContinuitySource::RecoveryWindow,
                MeterContinuityReason::RevalidationDecay,
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
    }
}

pub fn clear_arm(ctx: MeterStagePlanContext) -> MeterContinuityRecommendation {
    MeterContinuityRecommendation {
        bar_length: ctx.plan(
            MeterContinuityAction::Clear,
            MeterContinuitySource::Cleared,
            MeterContinuityReason::InsufficientEvidence,
            0,
            0,
            ctx.stage(
                0,
                MeterContinuityAction::Clear,
                MeterContinuitySource::Cleared,
                MeterContinuityReason::InsufficientEvidence,
                2,
            ),
            ctx.stage(
                0,
                MeterContinuityAction::Clear,
                MeterContinuitySource::Cleared,
                MeterContinuityReason::InsufficientEvidence,
                2,
            ),
            ctx.stage(
                0,
                MeterContinuityAction::Clear,
                MeterContinuitySource::Cleared,
                MeterContinuityReason::InsufficientEvidence,
                2,
            ),
        ),
        downbeat_phase: ctx.plan(
            MeterContinuityAction::Clear,
            MeterContinuitySource::Cleared,
            MeterContinuityReason::InsufficientEvidence,
            0,
            0,
            ctx.stage(
                0,
                MeterContinuityAction::Clear,
                MeterContinuitySource::Cleared,
                MeterContinuityReason::InsufficientEvidence,
                2,
            ),
            ctx.stage(
                0,
                MeterContinuityAction::Clear,
                MeterContinuitySource::Cleared,
                MeterContinuityReason::InsufficientEvidence,
                2,
            ),
            ctx.stage(
                0,
                MeterContinuityAction::Clear,
                MeterContinuitySource::Cleared,
                MeterContinuityReason::InsufficientEvidence,
                2,
            ),
        ),
    }
}
