use crate::tempo_policy::*;
use crate::tempo_state_continuity_basics::has_tempo_cause;
use signal_analysis::Confidence;

#[derive(Clone, Copy)]
pub struct TempoContinuityArcInputs {
    pub source: TempoContinuitySource,
    pub confidence: Confidence,
    pub unresolved: TempoContinuityUnresolvedSpan,
    pub causes: TempoContinuityCauseStack,
    pub current: TempoContinuityHistory,
    pub refresh: TempoContinuityTransition,
    pub first_decay: TempoContinuityTransition,
    pub final_decay: TempoContinuityTransition,
}

#[derive(Clone, Copy)]
pub struct TempoContinuityArcDecisionInputs {
    pub arc: TempoContinuityArc,
    pub rationale: TempoContinuityArcRationale,
    pub support: TempoContinuityArcSupport,
    pub severity: TempoContinuitySeverity,
    pub history: TempoContinuityHistory,
    pub trigger: TempoContinuityTrigger,
    pub causes: TempoContinuityCauseStack,
    pub provenance: TempoContinuityProvenance,
    pub expiry: TempoContinuityExpiry,
    pub trusted_beats: usize,
    pub revalidate_after_beats: usize,
    pub confidence: Confidence,
    pub unresolved: TempoContinuityUnresolvedSpan,
    pub refresh: TempoContinuityTransition,
    pub first_decay: TempoContinuityTransition,
    pub final_decay: TempoContinuityTransition,
}

#[derive(Clone, Copy)]
pub struct TempoContinuityPlanInputs {
    pub action: TempoContinuityAction,
    pub source: TempoContinuitySource,
    pub reason: TempoContinuityReason,
    pub boundary_pressure: Confidence,
    pub tempo_ambiguity: Confidence,
    pub confidence: Confidence,
    pub trusted_beats: usize,
    pub revalidate_after_beats: usize,
}

pub fn compute_action_expiry(
    action: TempoContinuityArcAction,
    expiry: TempoContinuityExpiry,
    trusted_beats: usize,
    revalidate_after_beats: usize,
) -> TempoContinuityArcActionExpiry {
    let guaranteed_until_beats = match action {
        TempoContinuityArcAction::LockCurrentTempo => trusted_beats,
        TempoContinuityArcAction::PreferCoreWindowTempo => trusted_beats
            .min(revalidate_after_beats.saturating_mul(2))
            .max(1),
        TempoContinuityArcAction::PreservePriorTempo => {
            trusted_beats.min(revalidate_after_beats).max(1)
        }
        TempoContinuityArcAction::ReacquireCurrentTempo => trusted_beats.max(1),
        TempoContinuityArcAction::ClearTempo => 0,
    };
    let fallback_after_beats = match action {
        TempoContinuityArcAction::LockCurrentTempo => expiry.downgrade_after_beats,
        TempoContinuityArcAction::PreferCoreWindowTempo => {
            expiry.downgrade_after_beats.min(expiry.clear_after_beats)
        }
        TempoContinuityArcAction::PreservePriorTempo
        | TempoContinuityArcAction::ReacquireCurrentTempo => expiry.clear_after_beats,
        TempoContinuityArcAction::ClearTempo => 0,
    };
    let max_failed_revalidations = match action {
        TempoContinuityArcAction::LockCurrentTempo => expiry.max_failed_revalidations,
        TempoContinuityArcAction::PreferCoreWindowTempo
        | TempoContinuityArcAction::PreservePriorTempo => {
            expiry.max_failed_revalidations.clamp(1, 2)
        }
        TempoContinuityArcAction::ReacquireCurrentTempo => {
            expiry.max_failed_revalidations.clamp(1, 3)
        }
        TempoContinuityArcAction::ClearTempo => 0,
    };

    TempoContinuityArcActionExpiry {
        guaranteed_until_beats,
        fallback_after_beats,
        clear_after_beats: expiry.clear_after_beats,
        max_failed_revalidations,
    }
}

pub fn compute_stage_rationale_weights(
    stage: TempoContinuityArcDowngradeInflectionStage,
    stage_delta: Confidence,
    downgrade_support: TempoContinuityArcDowngradeSupport,
    trigger: TempoContinuityTrigger,
    action: TempoContinuityArcAction,
    cause_stack: TempoContinuityCauseStack,
    unresolved: TempoContinuityUnresolvedSpan,
) -> TempoContinuityArcDowngradeStageRationaleWeights {
    let has_prior_carry = has_tempo_cause(cause_stack, TempoContinuityCause::PriorTempoCarry);
    let trigger_is_stable = matches!(trigger, TempoContinuityTrigger::StableRevalidation);
    let trigger_is_boundary = matches!(trigger, TempoContinuityTrigger::BoundaryDrift);
    let trigger_is_ambiguity = matches!(trigger, TempoContinuityTrigger::AmbiguityCarry);
    let trigger_is_evidence = matches!(trigger, TempoContinuityTrigger::EvidenceLoss);

    let base = stage_delta.0.clamp(0.0, 1.0);
    let stage_bias = match stage {
        TempoContinuityArcDowngradeInflectionStage::FlatWindow => 0.0,
        TempoContinuityArcDowngradeInflectionStage::NextStage => 0.18,
        TempoContinuityArcDowngradeInflectionStage::TerminalClear => 0.12,
    };
    let stability_window = (if trigger_is_stable {
        0.18 + 0.82 * downgrade_support.stability_window_pressure.0
    } else {
        0.35 * downgrade_support.stability_window_pressure.0
    }) * match stage {
        TempoContinuityArcDowngradeInflectionStage::FlatWindow => {
            if trigger_is_stable {
                0.15
            } else {
                0.0
            }
        }
        TempoContinuityArcDowngradeInflectionStage::NextStage => 1.0,
        TempoContinuityArcDowngradeInflectionStage::TerminalClear => 0.40,
    };
    let boundary_drift = (if trigger_is_boundary {
        0.18 + 0.82 * downgrade_support.boundary_drift_pressure.0
    } else {
        0.55 * downgrade_support.boundary_drift_pressure.0
    }) * match stage {
        TempoContinuityArcDowngradeInflectionStage::FlatWindow => {
            if trigger_is_boundary {
                0.20
            } else {
                0.0
            }
        }
        TempoContinuityArcDowngradeInflectionStage::NextStage => 1.0,
        TempoContinuityArcDowngradeInflectionStage::TerminalClear => 0.70,
    };
    let ambiguity_carry = (if trigger_is_ambiguity {
        0.18 + 0.82 * downgrade_support.ambiguity_pressure.0
    } else {
        0.55 * downgrade_support.ambiguity_pressure.0
    }) * match stage {
        TempoContinuityArcDowngradeInflectionStage::FlatWindow => {
            if trigger_is_ambiguity {
                0.20
            } else {
                0.0
            }
        }
        TempoContinuityArcDowngradeInflectionStage::NextStage => 1.0,
        TempoContinuityArcDowngradeInflectionStage::TerminalClear => 0.68,
    };
    let prior_tempo_drift = ((if has_prior_carry { 0.22 } else { 0.0 })
        + 0.55 * downgrade_support.failed_revalidation_pressure.0)
        * match stage {
            TempoContinuityArcDowngradeInflectionStage::FlatWindow => {
                if has_prior_carry {
                    0.25
                } else {
                    0.0
                }
            }
            TempoContinuityArcDowngradeInflectionStage::NextStage => {
                if has_prior_carry {
                    0.70
                } else {
                    0.20
                }
            }
            TempoContinuityArcDowngradeInflectionStage::TerminalClear => {
                if has_prior_carry {
                    0.82
                } else {
                    0.30
                }
            }
        };
    let revalidation_decay = (0.70 * downgrade_support.failed_revalidation_pressure.0)
        * match stage {
            TempoContinuityArcDowngradeInflectionStage::FlatWindow => {
                if unresolved.failed_revalidations > 0 {
                    0.25
                } else {
                    0.0
                }
            }
            TempoContinuityArcDowngradeInflectionStage::NextStage => 0.78,
            TempoContinuityArcDowngradeInflectionStage::TerminalClear => 0.88,
        };
    let evidence_loss =
        ((if trigger_is_evidence || matches!(action, TempoContinuityArcAction::ClearTempo) {
            0.18
        } else {
            0.0
        }) + 0.82 * downgrade_support.evidence_loss_pressure.0
            + if matches!(
                stage,
                TempoContinuityArcDowngradeInflectionStage::TerminalClear
            ) {
                0.22
            } else {
                0.0
            })
            * match stage {
                TempoContinuityArcDowngradeInflectionStage::FlatWindow => {
                    if matches!(action, TempoContinuityArcAction::ClearTempo) {
                        1.0
                    } else {
                        0.0
                    }
                }
                TempoContinuityArcDowngradeInflectionStage::NextStage => 0.62,
                TempoContinuityArcDowngradeInflectionStage::TerminalClear => 1.0,
            };

    let raw_stability_window =
        (stability_window + stage_bias * if trigger_is_stable { 1.0 } else { 0.0 }).clamp(0.0, 1.0);
    let raw_boundary_drift =
        (boundary_drift + stage_bias * if trigger_is_boundary { 1.0 } else { 0.0 }).clamp(0.0, 1.0);
    let raw_ambiguity_carry = (ambiguity_carry
        + stage_bias * if trigger_is_ambiguity { 1.0 } else { 0.0 })
    .clamp(0.0, 1.0);
    let raw_prior_tempo_drift = prior_tempo_drift.clamp(0.0, 1.0);
    let raw_revalidation_decay = revalidation_decay.clamp(0.0, 1.0);
    let raw_evidence_loss = evidence_loss.clamp(0.0, 1.0);

    let total = raw_stability_window
        + raw_boundary_drift
        + raw_ambiguity_carry
        + raw_prior_tempo_drift
        + raw_revalidation_decay
        + raw_evidence_loss;
    if total < 0.001
        || (matches!(
            stage,
            TempoContinuityArcDowngradeInflectionStage::FlatWindow
        ) && base <= 0.0
            && !matches!(action, TempoContinuityArcAction::ClearTempo))
    {
        return TempoContinuityArcDowngradeStageRationaleWeights {
            dominant: TempoContinuityArcDowngradeStageRationale::NoPressure,
            stability_window: Confidence::new(0.0),
            boundary_drift: Confidence::new(0.0),
            ambiguity_carry: Confidence::new(0.0),
            prior_tempo_drift: Confidence::new(0.0),
            revalidation_decay: Confidence::new(0.0),
            evidence_loss: Confidence::new(0.0),
        };
    }

    let stability_window = Confidence::new(raw_stability_window / total);
    let boundary_drift = Confidence::new(raw_boundary_drift / total);
    let ambiguity_carry = Confidence::new(raw_ambiguity_carry / total);
    let prior_tempo_drift = Confidence::new(raw_prior_tempo_drift / total);
    let revalidation_decay = Confidence::new(raw_revalidation_decay / total);
    let evidence_loss = Confidence::new(raw_evidence_loss / total);
    let dominant = [
        (
            TempoContinuityArcDowngradeStageRationale::StabilityWindow,
            stability_window.0,
        ),
        (
            TempoContinuityArcDowngradeStageRationale::BoundaryDrift,
            boundary_drift.0,
        ),
        (
            TempoContinuityArcDowngradeStageRationale::AmbiguityCarry,
            ambiguity_carry.0,
        ),
        (
            TempoContinuityArcDowngradeStageRationale::PriorTempoDrift,
            prior_tempo_drift.0,
        ),
        (
            TempoContinuityArcDowngradeStageRationale::RevalidationDecay,
            revalidation_decay.0,
        ),
        (
            TempoContinuityArcDowngradeStageRationale::EvidenceLoss,
            evidence_loss.0,
        ),
    ]
    .into_iter()
    .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
    .map(|entry| entry.0)
    .unwrap_or(TempoContinuityArcDowngradeStageRationale::NoPressure);

    TempoContinuityArcDowngradeStageRationaleWeights {
        dominant,
        stability_window,
        boundary_drift,
        ambiguity_carry,
        prior_tempo_drift,
        revalidation_decay,
        evidence_loss,
    }
}

