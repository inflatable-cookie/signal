use crate::rhythm_policy::*;

pub fn trigger_for_reason(
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

pub fn reason_for_stage(
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

pub fn primary_cause_for_reason(reason: MeterContinuityReason) -> Option<MeterContinuityCause> {
    match reason {
        MeterContinuityReason::StableEvidence => Some(MeterContinuityCause::StableMeterEvidence),
        MeterContinuityReason::PriorStateCarry => Some(MeterContinuityCause::PriorContinuityCarry),
        MeterContinuityReason::RecoveryWindowSupport => {
            Some(MeterContinuityCause::RecoveryWindowInstability)
        }
        MeterContinuityReason::PhaseDisplacement => Some(MeterContinuityCause::PhaseDisplacement),
        MeterContinuityReason::InsufficientEvidence => Some(MeterContinuityCause::EvidenceLoss),
        MeterContinuityReason::TentativeEvidence | MeterContinuityReason::RevalidationDecay => None,
    }
}

pub fn primary_cause_for_trigger(trigger: MeterContinuityTrigger) -> MeterContinuityCause {
    match trigger {
        MeterContinuityTrigger::StableRevalidation => MeterContinuityCause::StableMeterEvidence,
        MeterContinuityTrigger::TentativeCarry => MeterContinuityCause::SparseMeterSupport,
        MeterContinuityTrigger::PhaseRecovery => MeterContinuityCause::PhaseDisplacement,
        MeterContinuityTrigger::PriorStateDrift => MeterContinuityCause::PriorContinuityCarry,
        MeterContinuityTrigger::RecoveryWindowDrift => {
            MeterContinuityCause::RecoveryWindowInstability
        }
        MeterContinuityTrigger::EvidenceLoss => MeterContinuityCause::EvidenceLoss,
    }
}

#[cfg(test)]
mod tests {
    use super::{
        primary_cause_for_reason, primary_cause_for_trigger, reason_for_stage, trigger_for_reason,
    };
    use crate::rhythm_policy::*;

    #[test]
    fn meter_continuity_rule_surface_preserves_trigger_reason_and_cause_mapping() {
        assert_eq!(
            reason_for_stage(
                MeterContinuityAction::Reacquire,
                MeterContinuitySource::CurrentMeter,
                true,
                false,
            ),
            MeterContinuityReason::PhaseDisplacement
        );
        assert_eq!(
            reason_for_stage(
                MeterContinuityAction::Retain,
                MeterContinuitySource::RecoveryWindow,
                false,
                false,
            ),
            MeterContinuityReason::RecoveryWindowSupport
        );
        assert_eq!(
            trigger_for_reason(
                MeterContinuityAction::Retain,
                MeterContinuitySource::RecoveryWindow,
                MeterContinuityReason::RevalidationDecay,
            ),
            MeterContinuityTrigger::RecoveryWindowDrift
        );
        assert_eq!(
            primary_cause_for_reason(MeterContinuityReason::PriorStateCarry),
            Some(MeterContinuityCause::PriorContinuityCarry)
        );
        assert_eq!(
            primary_cause_for_trigger(MeterContinuityTrigger::PhaseRecovery),
            MeterContinuityCause::PhaseDisplacement
        );
    }
}
