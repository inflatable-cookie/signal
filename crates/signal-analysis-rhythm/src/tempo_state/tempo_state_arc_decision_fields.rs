use super::tempo_state_continuity_helpers::{
    compute_action_expiry, compute_stage_rationale_weights, TempoContinuityArcDecisionInputs,
};
use crate::tempo_policy::*;
use crate::tempo_state_continuity_basics::has_tempo_cause;
use signal_analysis::Confidence;

#[allow(clippy::type_complexity)]
pub fn arc_decision_fields(
    inputs: TempoContinuityArcDecisionInputs,
    action: TempoContinuityArcAction,
) -> (
    TempoContinuitySeverity,
    TempoContinuityArcAction,
    TempoContinuityArcDowngradeRationale,
    TempoContinuityArcDowngradeSupport,
    TempoContinuityArcDowngradeTrend,
    TempoContinuityArcDowngradeTrendRationale,
    TempoContinuityArcDowngradeTrendSupport,
    TempoContinuityArcDowngradeInflection,
    TempoContinuityProvenance,
    TempoContinuityArcActionExpiry,
) {
    let trigger = inputs.trigger;
    let history = inputs.history;
    let arc = inputs.arc;
    let support = inputs.support;
    let confidence = inputs.confidence;
    let unresolved = inputs.unresolved;
    let cause_stack = inputs.causes;
    let provenance = inputs.provenance;
    let refresh = inputs.refresh;
    let first_decay = inputs.first_decay;
    let final_decay = inputs.final_decay;

    let action_severity = match action {
        TempoContinuityArcAction::LockCurrentTempo => TempoContinuitySeverity::Confirmed,
        TempoContinuityArcAction::PreferCoreWindowTempo => TempoContinuitySeverity::Guarded,
        TempoContinuityArcAction::PreservePriorTempo => TempoContinuitySeverity::Fragile,
        TempoContinuityArcAction::ReacquireCurrentTempo => {
            if matches!(history, TempoContinuityHistory::Reinforcing)
                && support.refresh_strength.0 >= 0.72
            {
                TempoContinuitySeverity::Guarded
            } else {
                TempoContinuitySeverity::Fragile
            }
        }
        TempoContinuityArcAction::ClearTempo => TempoContinuitySeverity::Cleared,
    };
    let fallback_action = match action {
        TempoContinuityArcAction::LockCurrentTempo => {
            TempoContinuityArcAction::ReacquireCurrentTempo
        }
        TempoContinuityArcAction::PreferCoreWindowTempo => {
            TempoContinuityArcAction::PreservePriorTempo
        }
        TempoContinuityArcAction::PreservePriorTempo
        | TempoContinuityArcAction::ReacquireCurrentTempo => {
            TempoContinuityArcAction::ClearTempo
        }
        TempoContinuityArcAction::ClearTempo => TempoContinuityArcAction::ClearTempo,
    };
    let action_provenance = match action {
        TempoContinuityArcAction::LockCurrentTempo
        | TempoContinuityArcAction::ReacquireCurrentTempo => provenance,
        TempoContinuityArcAction::PreferCoreWindowTempo => {
            TempoContinuityProvenance::CoreWindowEstimate
        }
        TempoContinuityArcAction::PreservePriorTempo => {
            TempoContinuityProvenance::PriorTempoCarry
        }
        TempoContinuityArcAction::ClearTempo => TempoContinuityProvenance::NoTempo,
    };
    let downgrade_support = TempoContinuityArcDowngradeSupport {
        stability_window_pressure: Confidence::new(
            if matches!(trigger, TempoContinuityTrigger::StableRevalidation) {
                (0.55
                    + 0.25 * support.refresh_strength.0
                    + 0.20 * (1.0 - support.drift_pressure.0))
                    .clamp(0.0, 1.0)
            } else {
                0.0
            },
        ),
        boundary_drift_pressure: Confidence::new(
            ((if matches!(trigger, TempoContinuityTrigger::BoundaryDrift) {
                0.45_f32
            } else {
                0.0
            }) + if has_tempo_cause(cause_stack, TempoContinuityCause::BoundaryDrift) {
                0.35
            } else {
                0.0
            } + if has_tempo_cause(cause_stack, TempoContinuityCause::CoreWindowCarry) {
                0.15
            } else {
                0.0
            } + 0.10 * support.drift_pressure.0)
                .clamp(0.0, 1.0),
        ),
        ambiguity_pressure: Confidence::new(
            ((if matches!(trigger, TempoContinuityTrigger::AmbiguityCarry) {
                0.55_f32
            } else {
                0.0
            }) + if has_tempo_cause(cause_stack, TempoContinuityCause::TempoAmbiguity) {
                0.35
            } else {
                0.0
            } + 0.10 * support.instability_pressure.0)
                .clamp(0.0, 1.0),
        ),
        failed_revalidation_pressure: Confidence::new(
            ((unresolved.failed_revalidations as f32 / 3.0) * 0.75
                + if unresolved.failed_revalidations >= 2 {
                    0.20
                } else {
                    0.0
                })
            .clamp(0.0, 1.0),
        ),
        evidence_loss_pressure: Confidence::new(
            ((if matches!(trigger, TempoContinuityTrigger::EvidenceLoss) {
                0.55_f32
            } else {
                0.0
            }) + if has_tempo_cause(cause_stack, TempoContinuityCause::EvidenceLoss) {
                0.35
            } else {
                0.0
            } + if matches!(action, TempoContinuityArcAction::ClearTempo) {
                0.10
            } else {
                0.0
            })
            .clamp(0.0, 1.0),
        ),
    };
    let downgrade_rationale = if matches!(action, TempoContinuityArcAction::ClearTempo)
        || matches!(trigger, TempoContinuityTrigger::EvidenceLoss)
    {
        TempoContinuityArcDowngradeRationale::EvidenceLoss
    } else if unresolved.failed_revalidations >= 3
        || (unresolved.failed_revalidations >= 2
            && matches!(action, TempoContinuityArcAction::PreservePriorTempo))
    {
        TempoContinuityArcDowngradeRationale::RepeatedFailedRevalidation
    } else {
        match trigger {
            TempoContinuityTrigger::StableRevalidation => {
                TempoContinuityArcDowngradeRationale::StabilityWindowEnd
            }
            TempoContinuityTrigger::BoundaryDrift => {
                TempoContinuityArcDowngradeRationale::BoundaryDrift
            }
            TempoContinuityTrigger::AmbiguityCarry => {
                TempoContinuityArcDowngradeRationale::AmbiguityCarry
            }
            TempoContinuityTrigger::PriorTempoDrift => {
                TempoContinuityArcDowngradeRationale::PriorTempoDrift
            }
            TempoContinuityTrigger::EvidenceLoss => {
                TempoContinuityArcDowngradeRationale::EvidenceLoss
            }
        }
    };
    let downgrade_trend_support = {
        let current_pressure = Confidence::new((1.0 - confidence.0).clamp(0.0, 1.0));
        let next_stage_pressure = match arc {
            TempoContinuityArc::Recovering => {
                Confidence::new((1.0 - refresh.refresh_strength.0).clamp(0.0, 1.0))
            }
            TempoContinuityArc::Stalling => {
                Confidence::new((1.0 - first_decay.refresh_strength.0).clamp(0.0, 1.0))
            }
            TempoContinuityArc::Collapsing => {
                Confidence::new((1.0 - final_decay.refresh_strength.0).clamp(0.0, 1.0))
            }
        };
        let terminal_pressure =
            Confidence::new((1.0 - final_decay.refresh_strength.0).clamp(0.0, 1.0));

        TempoContinuityArcDowngradeTrendSupport {
            current_pressure,
            next_stage_pressure,
            terminal_pressure,
        }
    };
    let downgrade_trend = if matches!(action, TempoContinuityArcAction::ClearTempo) {
        TempoContinuityArcDowngradeTrend::Stable
    } else if downgrade_trend_support.next_stage_pressure.0
        > downgrade_trend_support.current_pressure.0 + 0.08
    {
        TempoContinuityArcDowngradeTrend::Rising
    } else if downgrade_trend_support.next_stage_pressure.0 + 0.12
        < downgrade_trend_support.current_pressure.0
    {
        TempoContinuityArcDowngradeTrend::Easing
    } else {
        TempoContinuityArcDowngradeTrend::Stable
    };
    let downgrade_trend_rationale = match downgrade_trend {
        TempoContinuityArcDowngradeTrend::Rising
            if matches!(trigger, TempoContinuityTrigger::BoundaryDrift) =>
        {
            TempoContinuityArcDowngradeTrendRationale::BoundaryEscalation
        }
        TempoContinuityArcDowngradeTrend::Rising => {
            TempoContinuityArcDowngradeTrendRationale::RevalidationDecay
        }
        TempoContinuityArcDowngradeTrend::Easing
            if matches!(trigger, TempoContinuityTrigger::AmbiguityCarry) =>
        {
            TempoContinuityArcDowngradeTrendRationale::AmbiguityCarry
        }
        TempoContinuityArcDowngradeTrend::Easing => {
            TempoContinuityArcDowngradeTrendRationale::StabilityWindowCarry
        }
        TempoContinuityArcDowngradeTrend::Stable
            if matches!(action, TempoContinuityArcAction::ClearTempo) =>
        {
            TempoContinuityArcDowngradeTrendRationale::FlatCollapse
        }
        TempoContinuityArcDowngradeTrend::Stable
            if downgrade_trend_support.terminal_pressure.0
                > downgrade_trend_support.current_pressure.0 + 0.12 =>
        {
            TempoContinuityArcDowngradeTrendRationale::TerminalClearPressure
        }
        TempoContinuityArcDowngradeTrend::Stable
            if matches!(trigger, TempoContinuityTrigger::AmbiguityCarry) =>
        {
            TempoContinuityArcDowngradeTrendRationale::AmbiguityCarry
        }
        TempoContinuityArcDowngradeTrend::Stable
            if matches!(trigger, TempoContinuityTrigger::BoundaryDrift) =>
        {
            TempoContinuityArcDowngradeTrendRationale::BoundaryEscalation
        }
        TempoContinuityArcDowngradeTrend::Stable => {
            TempoContinuityArcDowngradeTrendRationale::StabilityWindowCarry
        }
    };
    let downgrade_inflection = {
        let next_stage_after_beats = match arc {
            TempoContinuityArc::Recovering => refresh.after_beats,
            TempoContinuityArc::Stalling => first_decay.after_beats,
            TempoContinuityArc::Collapsing => final_decay.after_beats,
        };
        let next_stage_delta = Confidence::new(
            (downgrade_trend_support.next_stage_pressure.0
                - downgrade_trend_support.current_pressure.0)
                .abs()
                .clamp(0.0, 1.0),
        );
        let terminal_delta = Confidence::new(
            (downgrade_trend_support.terminal_pressure.0
                - downgrade_trend_support.current_pressure.0)
                .abs()
                .clamp(0.0, 1.0),
        );
        let stage = if matches!(action, TempoContinuityArcAction::ClearTempo)
            || (matches!(downgrade_trend, TempoContinuityArcDowngradeTrend::Stable)
                && next_stage_delta.0 < 0.06
                && terminal_delta.0 < 0.06)
        {
            TempoContinuityArcDowngradeInflectionStage::FlatWindow
        } else if matches!(
            downgrade_trend,
            TempoContinuityArcDowngradeTrend::Rising | TempoContinuityArcDowngradeTrend::Easing
        ) {
            TempoContinuityArcDowngradeInflectionStage::NextStage
        } else if terminal_delta.0 > next_stage_delta.0 + 0.06 {
            TempoContinuityArcDowngradeInflectionStage::TerminalClear
        } else if next_stage_delta.0 >= 0.06 {
            TempoContinuityArcDowngradeInflectionStage::NextStage
        } else {
            TempoContinuityArcDowngradeInflectionStage::FlatWindow
        };
        let after_beats = match stage {
            TempoContinuityArcDowngradeInflectionStage::FlatWindow => 0,
            TempoContinuityArcDowngradeInflectionStage::NextStage => next_stage_after_beats,
            TempoContinuityArcDowngradeInflectionStage::TerminalClear => {
                final_decay.after_beats
            }
        };
        let primary_delta = match stage {
            TempoContinuityArcDowngradeInflectionStage::FlatWindow => Confidence::new(0.0),
            TempoContinuityArcDowngradeInflectionStage::NextStage => next_stage_delta,
            TempoContinuityArcDowngradeInflectionStage::TerminalClear => terminal_delta,
        };
        let (competing_stage, competing_after_beats, competing_delta) = match stage {
            TempoContinuityArcDowngradeInflectionStage::NextStage
                if terminal_delta.0 >= 0.06 && terminal_delta.0 >= (primary_delta.0 * 0.55) =>
            {
                (
                    Some(TempoContinuityArcDowngradeInflectionStage::TerminalClear),
                    final_decay.after_beats,
                    terminal_delta,
                )
            }
            TempoContinuityArcDowngradeInflectionStage::TerminalClear
                if next_stage_delta.0 >= 0.06
                    && next_stage_delta.0 >= (primary_delta.0 * 0.55) =>
            {
                (
                    Some(TempoContinuityArcDowngradeInflectionStage::NextStage),
                    next_stage_after_beats,
                    next_stage_delta,
                )
            }
            _ => (None, 0, Confidence::new(0.0)),
        };
        let competing_support = if primary_delta.0 > 0.0 {
            Confidence::new((competing_delta.0 / primary_delta.0).clamp(0.0, 1.0))
        } else {
            Confidence::new(0.0)
        };
        let balance = {
            let modeled_total = (primary_delta.0 + competing_delta.0).clamp(0.0, 1.0);
            let primary_weight = if modeled_total > 0.0 {
                Confidence::new(primary_delta.0 / modeled_total)
            } else {
                Confidence::new(0.0)
            };
            let competing_weight = if modeled_total > 0.0 {
                Confidence::new(competing_delta.0 / modeled_total)
            } else {
                Confidence::new(0.0)
            };
            let unattributed_weight = Confidence::new(1.0 - modeled_total);
            let dominance = Confidence::new((primary_weight.0 - competing_weight.0).max(0.0));

            TempoContinuityArcDowngradeInflectionBalance {
                primary_weight,
                competing_weight,
                unattributed_weight,
                dominance,
            }
        };
        let rationale_balance = TempoContinuityArcDowngradeInflectionRationaleBalance {
            primary: compute_stage_rationale_weights(
                stage,
                primary_delta,
                downgrade_support,
                trigger,
                action,
                cause_stack,
                unresolved,
            ),
            competing: competing_stage.map(|s| {
                compute_stage_rationale_weights(
                    s,
                    competing_delta,
                    downgrade_support,
                    trigger,
                    action,
                    cause_stack,
                    unresolved,
                )
            }),
        };

        TempoContinuityArcDowngradeInflection {
            stage,
            after_beats,
            next_stage_delta,
            terminal_delta,
            competing_stage,
            competing_after_beats,
            competing_delta,
            competing_support,
            balance,
            rationale_balance,
        }
    };
    let expiry =
        compute_action_expiry(action, inputs.expiry, inputs.trusted_beats, inputs.revalidate_after_beats);

    (
        action_severity,
        fallback_action,
        downgrade_rationale,
        downgrade_support,
        downgrade_trend,
        downgrade_trend_rationale,
        downgrade_trend_support,
        downgrade_inflection,
        action_provenance,
        expiry,
    )
}
