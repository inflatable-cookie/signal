use super::tempo_state_arc_decision::continuity_arc_decision;
use crate::tempo_policy::*;
use crate::tempo_state_continuity_basics::{
    continuity_cause_stack, continuity_history, continuity_provenance, continuity_severity,
    continuity_trigger, has_tempo_cause, unresolved_span,
};
use crate::tempo_state_continuity_refresh::continuity_refresh_strength;
use crate::tempo_state_continuity_transition::continuity_expiry;
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

pub fn continuity_arc_support(
    unresolved: TempoContinuityUnresolvedSpan,
    causes: TempoContinuityCauseStack,
    current: TempoContinuityHistory,
    refresh: TempoContinuityTransition,
    first_decay: TempoContinuityTransition,
    final_decay: TempoContinuityTransition,
) -> TempoContinuityArcSupport {
    let refresh_bonus = match refresh.history {
        TempoContinuityHistory::Reinforcing => 0.26,
        TempoContinuityHistory::Preserving => 0.12,
        TempoContinuityHistory::Degrading => 0.0,
    };
    let current_bonus = match current {
        TempoContinuityHistory::Reinforcing => 0.18,
        TempoContinuityHistory::Preserving => 0.08,
        TempoContinuityHistory::Degrading => 0.0,
    };
    let decay_penalty = match first_decay.history {
        TempoContinuityHistory::Degrading => 0.08,
        _ => 0.0,
    } + match final_decay.history {
        TempoContinuityHistory::Degrading => 0.12,
        _ => 0.0,
    };
    let refresh_strength = Confidence::new(
        (refresh.refresh_strength.0 + refresh_bonus + current_bonus - decay_penalty)
            .clamp(0.0, 1.0),
    );

    let drift_pressure = Confidence::new(
        ((unresolved.failed_revalidations as f32 * 0.20)
            + match current {
                TempoContinuityHistory::Degrading => 0.18,
                TempoContinuityHistory::Preserving => 0.08,
                TempoContinuityHistory::Reinforcing => 0.0,
            }
            + match first_decay.history {
                TempoContinuityHistory::Degrading => 0.14,
                _ => 0.0,
            }
            + match final_decay.history {
                TempoContinuityHistory::Degrading => 0.18,
                _ => 0.0,
            })
        .clamp(0.0, 1.0),
    );

    let instability_pressure = Confidence::new(
        ((if has_tempo_cause(causes, TempoContinuityCause::BoundaryDrift) {
            0.28_f32
        } else {
            0.0
        }) + (if has_tempo_cause(causes, TempoContinuityCause::TempoAmbiguity) {
            0.18
        } else {
            0.0
        }) + (if has_tempo_cause(causes, TempoContinuityCause::PriorTempoCarry) {
            0.16
        } else {
            0.0
        }) + (if has_tempo_cause(causes, TempoContinuityCause::CoreWindowCarry) {
            0.10
        } else {
            0.0
        }) + (if has_tempo_cause(causes, TempoContinuityCause::EvidenceLoss) {
            0.40
        } else {
            0.0
        }))
        .clamp(0.0, 1.0),
    );

    TempoContinuityArcSupport {
        refresh_strength,
        drift_pressure,
        instability_pressure,
    }
}

pub fn continuity_arc_assessment(
    inputs: TempoContinuityArcInputs,
) -> (
    TempoContinuityArc,
    TempoContinuityArcRationale,
    TempoContinuityArcSupport,
) {
    let TempoContinuityArcInputs {
        source,
        confidence,
        unresolved,
        causes,
        current,
        refresh,
        first_decay,
        final_decay,
    } = inputs;
    let has_evidence_loss = has_tempo_cause(causes, TempoContinuityCause::EvidenceLoss);
    let has_boundary = has_tempo_cause(causes, TempoContinuityCause::BoundaryDrift);
    let has_prior_carry = has_tempo_cause(causes, TempoContinuityCause::PriorTempoCarry);
    let persistent_decay = matches!(first_decay.history, TempoContinuityHistory::Degrading)
        && matches!(final_decay.history, TempoContinuityHistory::Degrading);
    let support = continuity_arc_support(
        unresolved,
        causes,
        current,
        refresh,
        first_decay,
        final_decay,
    );

    if matches!(current, TempoContinuityHistory::Degrading)
        && (persistent_decay || has_evidence_loss)
    {
        return (
            TempoContinuityArc::Collapsing,
            if has_evidence_loss {
                TempoContinuityArcRationale::EvidenceLoss
            } else {
                TempoContinuityArcRationale::UnresolvedDrift
            },
            support,
        );
    }

    if matches!(refresh.history, TempoContinuityHistory::Reinforcing) && !has_evidence_loss {
        if matches!(current, TempoContinuityHistory::Reinforcing) {
            return (
                TempoContinuityArc::Recovering,
                TempoContinuityArcRationale::RefreshStrength,
                support,
            );
        }

        if matches!(current, TempoContinuityHistory::Preserving)
            && confidence.0 >= 0.56
            && unresolved.failed_revalidations <= 1
            && !has_prior_carry
        {
            return (
                TempoContinuityArc::Recovering,
                TempoContinuityArcRationale::RefreshStrength,
                support,
            );
        }
    }

    if has_evidence_loss
        || (persistent_decay && confidence.0 < 0.24)
        || (matches!(current, TempoContinuityHistory::Degrading)
            && !matches!(refresh.history, TempoContinuityHistory::Reinforcing))
    {
        return (
            TempoContinuityArc::Collapsing,
            if has_evidence_loss {
                TempoContinuityArcRationale::EvidenceLoss
            } else if has_boundary {
                TempoContinuityArcRationale::BoundaryDrift
            } else {
                TempoContinuityArcRationale::UnresolvedDrift
            },
            support,
        );
    }

    (
        TempoContinuityArc::Stalling,
        if has_boundary || matches!(source, TempoContinuitySource::CoreWindow) {
            TempoContinuityArcRationale::BoundaryDrift
        } else if unresolved.failed_revalidations >= 2 || has_prior_carry {
            TempoContinuityArcRationale::UnresolvedDrift
        } else {
            TempoContinuityArcRationale::StableCarry
        },
        support,
    )
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

pub fn continuity_plan(
    plan: TempoContinuityPlanInputs,
    refresh: TempoContinuityTransition,
    first_decay: TempoContinuityTransition,
    final_decay: TempoContinuityTransition,
) -> TempoContinuityPlan {
    let TempoContinuityPlanInputs {
        action,
        source,
        reason,
        boundary_pressure,
        tempo_ambiguity,
        confidence,
        trusted_beats,
        revalidate_after_beats,
    } = plan;
    let trigger = continuity_trigger(action, source, reason, boundary_pressure, tempo_ambiguity);
    let unresolved = unresolved_span(trigger, trusted_beats, revalidate_after_beats, 0);
    let causes = continuity_cause_stack(action, source, reason, boundary_pressure, tempo_ambiguity);
    let severity = continuity_severity(action, source);
    let history = continuity_history(action, source, reason, trigger, unresolved, causes, 0);
    let provenance = continuity_provenance(action, source, reason);
    let expiry = continuity_expiry(
        trusted_beats,
        revalidate_after_beats,
        first_decay,
        final_decay,
    );
    let (arc, arc_rationale, arc_support) = continuity_arc_assessment(TempoContinuityArcInputs {
        source,
        confidence,
        unresolved,
        causes,
        current: history,
        refresh,
        first_decay,
        final_decay,
    });
    let arc_decision = continuity_arc_decision(TempoContinuityArcDecisionInputs {
        arc,
        rationale: arc_rationale,
        support: arc_support,
        severity,
        history,
        trigger,
        causes,
        provenance,
        expiry,
        trusted_beats,
        revalidate_after_beats,
        confidence,
        unresolved,
        refresh,
        first_decay,
        final_decay,
    });
    TempoContinuityPlan {
        action,
        source,
        severity,
        history,
        arc,
        arc_rationale,
        arc_support,
        arc_decision,
        reason,
        trigger,
        unresolved,
        causes,
        provenance,
        confidence,
        refresh_strength: continuity_refresh_strength(
            action,
            source,
            confidence,
            history,
            unresolved,
            causes,
            trusted_beats.max(revalidate_after_beats),
        ),
        trusted_beats,
        revalidate_after_beats,
        expiry,
        lifecycle: TempoContinuityLifecycle {
            refresh,
            decay: [first_decay, final_decay],
        },
    }
}
