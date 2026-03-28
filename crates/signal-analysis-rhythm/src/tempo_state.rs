use crate::tempo_policy::*;
use crate::tempo_state_continuity_basics::{
    continuity_cause_stack, continuity_history, continuity_provenance, continuity_severity,
    continuity_trigger, has_tempo_cause, unresolved_span,
};
use crate::tempo_state_continuity_refresh::continuity_refresh_strength;
use crate::tempo_state_continuity_transition::{
    continuity_expiry, continuity_transition, TempoContinuityTransitionInputs,
};
use signal_analysis::Confidence;

pub fn tempo_state_recommendation_with_scope(
    interpretation: TempoInterpretation,
    confidence: Confidence,
    tempo_ambiguity: Confidence,
    stability_scope: TempoStabilityScopeSummary,
) -> TempoStateRecommendation {
    #[derive(Clone, Copy)]
    struct TempoContinuityArcInputs {
        source: TempoContinuitySource,
        confidence: Confidence,
        unresolved: TempoContinuityUnresolvedSpan,
        causes: TempoContinuityCauseStack,
        current: TempoContinuityHistory,
        refresh: TempoContinuityTransition,
        first_decay: TempoContinuityTransition,
        final_decay: TempoContinuityTransition,
    }

    #[derive(Clone, Copy)]
    struct TempoContinuityArcDecisionInputs {
        arc: TempoContinuityArc,
        rationale: TempoContinuityArcRationale,
        support: TempoContinuityArcSupport,
        severity: TempoContinuitySeverity,
        history: TempoContinuityHistory,
        trigger: TempoContinuityTrigger,
        causes: TempoContinuityCauseStack,
        provenance: TempoContinuityProvenance,
        expiry: TempoContinuityExpiry,
        trusted_beats: usize,
        revalidate_after_beats: usize,
        confidence: Confidence,
        unresolved: TempoContinuityUnresolvedSpan,
        refresh: TempoContinuityTransition,
        first_decay: TempoContinuityTransition,
        final_decay: TempoContinuityTransition,
    }

    #[derive(Clone, Copy)]
    struct TempoContinuityPlanInputs {
        action: TempoContinuityAction,
        source: TempoContinuitySource,
        reason: TempoContinuityReason,
        boundary_pressure: Confidence,
        tempo_ambiguity: Confidence,
        confidence: Confidence,
        trusted_beats: usize,
        revalidate_after_beats: usize,
    }

    fn continuity_arc_support(
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

    pub(crate) fn continuity_arc_assessment(
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

    pub(crate) fn continuity_arc_decision(
        inputs: TempoContinuityArcDecisionInputs,
    ) -> TempoContinuityArcDecision {
        let TempoContinuityArcDecisionInputs {
            arc,
            rationale,
            support,
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
        } = inputs;
        let cause_stack = causes;
        let action_expiry = |action: TempoContinuityArcAction| -> TempoContinuityArcActionExpiry {
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
        };

        let decision_fields = |action: TempoContinuityArcAction| {
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
                    TempoContinuityArcDowngradeTrend::Rising
                        | TempoContinuityArcDowngradeTrend::Easing
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
                        if terminal_delta.0 >= 0.06
                            && terminal_delta.0 >= (primary_delta.0 * 0.55) =>
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
                    let dominance =
                        Confidence::new((primary_weight.0 - competing_weight.0).max(0.0));

                    TempoContinuityArcDowngradeInflectionBalance {
                        primary_weight,
                        competing_weight,
                        unattributed_weight,
                        dominance,
                    }
                };
                let stage_rationale_weights =
                    |stage: TempoContinuityArcDowngradeInflectionStage,
                     stage_delta: Confidence|
                     -> TempoContinuityArcDowngradeStageRationaleWeights {
                        let has_prior_carry =
                            has_tempo_cause(cause_stack, TempoContinuityCause::PriorTempoCarry);
                        let trigger_is_stable =
                            matches!(trigger, TempoContinuityTrigger::StableRevalidation);
                        let trigger_is_boundary =
                            matches!(trigger, TempoContinuityTrigger::BoundaryDrift);
                        let trigger_is_ambiguity =
                            matches!(trigger, TempoContinuityTrigger::AmbiguityCarry);
                        let trigger_is_evidence =
                            matches!(trigger, TempoContinuityTrigger::EvidenceLoss);

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
                        let revalidation_decay = (0.70
                            * downgrade_support.failed_revalidation_pressure.0)
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
                        let evidence_loss = ((if trigger_is_evidence
                            || matches!(action, TempoContinuityArcAction::ClearTempo)
                        {
                            0.18
                        } else {
                            0.0
                        }) + 0.82
                            * downgrade_support.evidence_loss_pressure.0
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

                        let raw_stability_window = (stability_window
                            + stage_bias * if trigger_is_stable { 1.0 } else { 0.0 })
                        .clamp(0.0, 1.0);
                        let raw_boundary_drift = (boundary_drift
                            + stage_bias * if trigger_is_boundary { 1.0 } else { 0.0 })
                        .clamp(0.0, 1.0);
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
                    };
                let rationale_balance = TempoContinuityArcDowngradeInflectionRationaleBalance {
                    primary: stage_rationale_weights(stage, primary_delta),
                    competing: competing_stage
                        .map(|stage| stage_rationale_weights(stage, competing_delta)),
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
            let expiry = action_expiry(action);

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
        };

        match arc {
            TempoContinuityArc::Recovering
                if matches!(severity, TempoContinuitySeverity::Confirmed)
                    && matches!(history, TempoContinuityHistory::Reinforcing)
                    && unresolved.failed_revalidations == 0
                    && matches!(rationale, TempoContinuityArcRationale::RefreshStrength) =>
            {
                let action = TempoContinuityArcAction::LockCurrentTempo;
                let (
                    severity,
                    fallback_action,
                    downgrade_rationale,
                    downgrade_support,
                    downgrade_trend,
                    downgrade_trend_rationale,
                    downgrade_trend_support,
                    downgrade_inflection,
                    provenance,
                    expiry,
                ) = decision_fields(action);
                TempoContinuityArcDecision {
                    recommendation: TempoContinuityArcRecommendation::KeepLock,
                    action,
                    severity,
                    fallback_action,
                    downgrade_rationale,
                    downgrade_support,
                    downgrade_trend,
                    downgrade_trend_rationale,
                    downgrade_trend_support,
                    downgrade_inflection,
                    provenance,
                    expiry,
                    confidence: Confidence::new(
                        (0.55 * support.refresh_strength.0
                            + 0.25 * confidence.0
                            + 0.20 * (1.0 - support.instability_pressure.0))
                            .clamp(0.0, 1.0),
                    ),
                }
            }
            TempoContinuityArc::Recovering | TempoContinuityArc::Stalling => {
                let action = match arc {
                    TempoContinuityArc::Recovering => {
                        TempoContinuityArcAction::ReacquireCurrentTempo
                    }
                    TempoContinuityArc::Stalling
                        if matches!(rationale, TempoContinuityArcRationale::BoundaryDrift) =>
                    {
                        TempoContinuityArcAction::PreferCoreWindowTempo
                    }
                    TempoContinuityArc::Stalling => TempoContinuityArcAction::PreservePriorTempo,
                    TempoContinuityArc::Collapsing => TempoContinuityArcAction::ClearTempo,
                };
                let (
                    severity,
                    fallback_action,
                    downgrade_rationale,
                    downgrade_support,
                    downgrade_trend,
                    downgrade_trend_rationale,
                    downgrade_trend_support,
                    downgrade_inflection,
                    provenance,
                    expiry,
                ) = decision_fields(action);
                TempoContinuityArcDecision {
                    recommendation: TempoContinuityArcRecommendation::MonitorRecovery,
                    action,
                    severity,
                    fallback_action,
                    downgrade_rationale,
                    downgrade_support,
                    downgrade_trend,
                    downgrade_trend_rationale,
                    downgrade_trend_support,
                    downgrade_inflection,
                    provenance,
                    expiry,
                    confidence: Confidence::new(
                        (0.45 * support.refresh_strength.0
                            + 0.20 * confidence.0
                            + 0.20 * (1.0 - support.drift_pressure.0)
                            + 0.15 * (1.0 - support.instability_pressure.0))
                            .clamp(0.0, 1.0),
                    ),
                }
            }
            TempoContinuityArc::Collapsing => {
                let action = TempoContinuityArcAction::ClearTempo;
                let (
                    severity,
                    fallback_action,
                    downgrade_rationale,
                    downgrade_support,
                    downgrade_trend,
                    downgrade_trend_rationale,
                    downgrade_trend_support,
                    downgrade_inflection,
                    provenance,
                    expiry,
                ) = decision_fields(action);
                TempoContinuityArcDecision {
                    recommendation: TempoContinuityArcRecommendation::Clear,
                    action,
                    severity,
                    fallback_action,
                    downgrade_rationale,
                    downgrade_support,
                    downgrade_trend,
                    downgrade_trend_rationale,
                    downgrade_trend_support,
                    downgrade_inflection,
                    provenance,
                    expiry,
                    confidence: Confidence::new(
                        (0.50 * support.instability_pressure.0
                            + 0.30 * support.drift_pressure.0
                            + 0.20
                                * if matches!(rationale, TempoContinuityArcRationale::EvidenceLoss)
                                {
                                    1.0
                                } else {
                                    0.65
                                })
                        .clamp(0.0, 1.0),
                    ),
                }
            }
        }
    }

    fn continuity_plan(
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
        let trigger =
            continuity_trigger(action, source, reason, boundary_pressure, tempo_ambiguity);
        let unresolved = unresolved_span(trigger, trusted_beats, revalidate_after_beats, 0);
        let causes =
            continuity_cause_stack(action, source, reason, boundary_pressure, tempo_ambiguity);
        let severity = continuity_severity(action, source);
        let history = continuity_history(action, source, reason, trigger, unresolved, causes, 0);
        let provenance = continuity_provenance(action, source, reason);
        let expiry = continuity_expiry(
            trusted_beats,
            revalidate_after_beats,
            first_decay,
            final_decay,
        );
        let (arc, arc_rationale, arc_support) =
            continuity_arc_assessment(TempoContinuityArcInputs {
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

    let base_confidence = (0.45 * interpretation.profile.stability_score.0
        + 0.25 * confidence.0
        + 0.15 * (1.0 - tempo_ambiguity.0)
        + 0.15 * interpretation.support.grid_stability.0)
        .clamp(0.0, 1.0);
    let localized_edge_horizons = || {
        if interpretation.support.boundary_pressure.0 >= 0.20 {
            (10, 6, 12, 18, 0.60)
        } else {
            (12, 8, 14, 20, 0.64)
        }
    };
    let localized_edge_scope = matches!(
        stability_scope.scope,
        TempoStabilityScope::StableWithLocalizedEdgeDamage
    );
    let core_stable_scope = matches!(stability_scope.scope, TempoStabilityScope::CoreStableOnly);
    let mid_track_unstable_scope =
        matches!(stability_scope.scope, TempoStabilityScope::MidTrackUnstable);
    let strong_integer_anchor = matches!(
        interpretation.recommendation,
        TempoRecommendation::SnapInteger
    ) && interpretation.support.integer_closeness.0 > 0.85
        && interpretation.support.core_consensus.0 > 0.8
        && interpretation.support.drift_stability.0 > 0.5
        && interpretation.support.grid_stability.0 > 0.35
        && interpretation.support.boundary_pressure.0 < 0.6;
    let ambiguity_guard = tempo_ambiguity.0 < 0.4 || strong_integer_anchor;

    match interpretation.recommendation {
        TempoRecommendation::SnapInteger
            if interpretation.trust != TempoTrustLevel::Tentative
                && (interpretation.profile.stability_score.0 >= 0.78 || strong_integer_anchor)
                && (interpretation.profile.snap_error_bpm >= 0.04
                    || interpretation.support.integer_closeness.0 > 0.9)
                && ambiguity_guard =>
        {
            if core_stable_scope || mid_track_unstable_scope {
                let state_confidence = Confidence::new(base_confidence.max(if core_stable_scope {
                    0.58
                } else {
                    0.48
                }));
                return TempoStateRecommendation {
                    action: if core_stable_scope {
                        TempoStateAction::Monitor
                    } else {
                        TempoStateAction::Defer
                    },
                    reason: if core_stable_scope {
                        TempoStateReason::CoreStableTempo
                    } else {
                        TempoStateReason::TempoDeferred
                    },
                    confidence: state_confidence,
                    continuity: continuity_plan(
                        TempoContinuityPlanInputs {
                            action: if core_stable_scope {
                                TempoContinuityAction::Reacquire
                            } else {
                                TempoContinuityAction::Clear
                            },
                            source: if core_stable_scope {
                                TempoContinuitySource::CurrentTempo
                            } else {
                                TempoContinuitySource::Cleared
                            },
                            reason: if core_stable_scope {
                                TempoContinuityReason::RevalidationDecay
                            } else {
                                TempoContinuityReason::InsufficientEvidence
                            },
                            boundary_pressure: interpretation.support.boundary_pressure,
                            tempo_ambiguity,
                            confidence: state_confidence,
                            trusted_beats: if core_stable_scope { 4 } else { 0 },
                            revalidate_after_beats: if core_stable_scope { 4 } else { 0 },
                        },
                        continuity_transition(TempoContinuityTransitionInputs {
                            after_beats: if core_stable_scope { 4 } else { 0 },
                            action: if core_stable_scope {
                                TempoContinuityAction::Lock
                            } else {
                                TempoContinuityAction::Clear
                            },
                            source: if core_stable_scope {
                                TempoContinuitySource::CurrentTempo
                            } else {
                                TempoContinuitySource::Cleared
                            },
                            reason: if core_stable_scope {
                                TempoContinuityReason::StableTempo
                            } else {
                                TempoContinuityReason::InsufficientEvidence
                            },
                            boundary_pressure: interpretation.support.boundary_pressure,
                            tempo_ambiguity,
                            revalidate_after_beats: if core_stable_scope { 4 } else { 0 },
                            stage_index: 0,
                            confidence: if core_stable_scope {
                                Confidence::new((state_confidence.0 * 0.92).clamp(0.0, 1.0))
                            } else {
                                Confidence::new(0.0)
                            },
                        }),
                        continuity_transition(TempoContinuityTransitionInputs {
                            after_beats: if core_stable_scope { 8 } else { 0 },
                            action: if core_stable_scope {
                                TempoContinuityAction::Reacquire
                            } else {
                                TempoContinuityAction::Clear
                            },
                            source: if core_stable_scope {
                                TempoContinuitySource::CurrentTempo
                            } else {
                                TempoContinuitySource::Cleared
                            },
                            reason: if core_stable_scope {
                                TempoContinuityReason::RevalidationDecay
                            } else {
                                TempoContinuityReason::InsufficientEvidence
                            },
                            boundary_pressure: interpretation.support.boundary_pressure,
                            tempo_ambiguity,
                            revalidate_after_beats: if core_stable_scope { 4 } else { 0 },
                            stage_index: 1,
                            confidence: if core_stable_scope {
                                Confidence::new((state_confidence.0 * 0.64).clamp(0.0, 1.0))
                            } else {
                                Confidence::new(0.0)
                            },
                        }),
                        continuity_transition(TempoContinuityTransitionInputs {
                            after_beats: if core_stable_scope { 12 } else { 0 },
                            action: TempoContinuityAction::Clear,
                            source: TempoContinuitySource::Cleared,
                            reason: TempoContinuityReason::InsufficientEvidence,
                            boundary_pressure: interpretation.support.boundary_pressure,
                            tempo_ambiguity,
                            revalidate_after_beats: if core_stable_scope { 4 } else { 0 },
                            stage_index: 2,
                            confidence: Confidence::new(0.0),
                        }),
                    ),
                };
            }
            let state_confidence = Confidence::new(base_confidence.max(if localized_edge_scope {
                0.76
            } else if strong_integer_anchor {
                0.80
            } else {
                0.82
            }));
            let (
                localized_trusted_beats,
                localized_revalidate_after_beats,
                localized_downgrade_after_beats,
                localized_clear_after_beats,
                localized_decay_confidence_scale,
            ) = localized_edge_horizons();
            TempoStateRecommendation {
                action: TempoStateAction::Lock,
                reason: if localized_edge_scope {
                    TempoStateReason::StableTempoWithEdgeDamage
                } else {
                    TempoStateReason::StableIntegerTempo
                },
                confidence: state_confidence,
                continuity: continuity_plan(
                    TempoContinuityPlanInputs {
                        action: TempoContinuityAction::Lock,
                        source: TempoContinuitySource::CurrentTempo,
                        reason: TempoContinuityReason::IntegerTempoSnap,
                        boundary_pressure: interpretation.support.boundary_pressure,
                        tempo_ambiguity,
                        confidence: state_confidence,
                        trusted_beats: if localized_edge_scope {
                            localized_trusted_beats
                        } else {
                            16
                        },
                        revalidate_after_beats: if localized_edge_scope {
                            localized_revalidate_after_beats
                        } else {
                            12
                        },
                    },
                    continuity_transition(TempoContinuityTransitionInputs {
                        after_beats: if localized_edge_scope {
                            localized_revalidate_after_beats
                        } else {
                            12
                        },
                        action: TempoContinuityAction::Lock,
                        source: TempoContinuitySource::CurrentTempo,
                        reason: TempoContinuityReason::IntegerTempoSnap,
                        boundary_pressure: interpretation.support.boundary_pressure,
                        tempo_ambiguity,
                        revalidate_after_beats: if localized_edge_scope {
                            localized_revalidate_after_beats
                        } else {
                            12
                        },
                        stage_index: 0,
                        confidence: state_confidence,
                    }),
                    continuity_transition(TempoContinuityTransitionInputs {
                        after_beats: if localized_edge_scope {
                            localized_downgrade_after_beats
                        } else {
                            20
                        },
                        action: TempoContinuityAction::Retain,
                        source: TempoContinuitySource::CurrentTempo,
                        reason: TempoContinuityReason::RevalidationDecay,
                        boundary_pressure: interpretation.support.boundary_pressure,
                        tempo_ambiguity,
                        revalidate_after_beats: if localized_edge_scope {
                            localized_revalidate_after_beats
                        } else {
                            12
                        },
                        stage_index: 1,
                        confidence: Confidence::new(
                            (state_confidence.0
                                * if localized_edge_scope {
                                    localized_decay_confidence_scale
                                } else {
                                    0.72
                                })
                            .clamp(0.0, 1.0),
                        ),
                    }),
                    continuity_transition(TempoContinuityTransitionInputs {
                        after_beats: if localized_edge_scope {
                            localized_clear_after_beats
                        } else {
                            28
                        },
                        action: TempoContinuityAction::Clear,
                        source: TempoContinuitySource::Cleared,
                        reason: TempoContinuityReason::InsufficientEvidence,
                        boundary_pressure: interpretation.support.boundary_pressure,
                        tempo_ambiguity,
                        revalidate_after_beats: if localized_edge_scope {
                            localized_revalidate_after_beats
                        } else {
                            12
                        },
                        stage_index: 2,
                        confidence: Confidence::new(0.0),
                    }),
                ),
            }
        }
        TempoRecommendation::UseRefined
            if interpretation.trust == TempoTrustLevel::Stable
                && interpretation.profile.stability_score.0 >= 0.72
                && interpretation.support.boundary_pressure.0 < 0.55
                && ambiguity_guard =>
        {
            if core_stable_scope || mid_track_unstable_scope {
                let state_confidence = Confidence::new(base_confidence.max(if core_stable_scope {
                    0.56
                } else {
                    0.46
                }));
                return TempoStateRecommendation {
                    action: if core_stable_scope {
                        TempoStateAction::Monitor
                    } else {
                        TempoStateAction::Defer
                    },
                    reason: if core_stable_scope {
                        TempoStateReason::CoreStableTempo
                    } else {
                        TempoStateReason::TempoDeferred
                    },
                    confidence: state_confidence,
                    continuity: continuity_plan(
                        TempoContinuityPlanInputs {
                            action: if core_stable_scope {
                                TempoContinuityAction::Reacquire
                            } else {
                                TempoContinuityAction::Clear
                            },
                            source: if core_stable_scope {
                                TempoContinuitySource::CurrentTempo
                            } else {
                                TempoContinuitySource::Cleared
                            },
                            reason: if core_stable_scope {
                                TempoContinuityReason::RevalidationDecay
                            } else {
                                TempoContinuityReason::InsufficientEvidence
                            },
                            boundary_pressure: interpretation.support.boundary_pressure,
                            tempo_ambiguity,
                            confidence: state_confidence,
                            trusted_beats: if core_stable_scope { 4 } else { 0 },
                            revalidate_after_beats: if core_stable_scope { 4 } else { 0 },
                        },
                        continuity_transition(TempoContinuityTransitionInputs {
                            after_beats: if core_stable_scope { 4 } else { 0 },
                            action: if core_stable_scope {
                                TempoContinuityAction::Lock
                            } else {
                                TempoContinuityAction::Clear
                            },
                            source: if core_stable_scope {
                                TempoContinuitySource::CurrentTempo
                            } else {
                                TempoContinuitySource::Cleared
                            },
                            reason: if core_stable_scope {
                                TempoContinuityReason::StableTempo
                            } else {
                                TempoContinuityReason::InsufficientEvidence
                            },
                            boundary_pressure: interpretation.support.boundary_pressure,
                            tempo_ambiguity,
                            revalidate_after_beats: if core_stable_scope { 4 } else { 0 },
                            stage_index: 0,
                            confidence: if core_stable_scope {
                                Confidence::new((state_confidence.0 * 0.94).clamp(0.0, 1.0))
                            } else {
                                Confidence::new(0.0)
                            },
                        }),
                        continuity_transition(TempoContinuityTransitionInputs {
                            after_beats: if core_stable_scope { 8 } else { 0 },
                            action: if core_stable_scope {
                                TempoContinuityAction::Reacquire
                            } else {
                                TempoContinuityAction::Clear
                            },
                            source: if core_stable_scope {
                                TempoContinuitySource::CurrentTempo
                            } else {
                                TempoContinuitySource::Cleared
                            },
                            reason: if core_stable_scope {
                                TempoContinuityReason::RevalidationDecay
                            } else {
                                TempoContinuityReason::InsufficientEvidence
                            },
                            boundary_pressure: interpretation.support.boundary_pressure,
                            tempo_ambiguity,
                            revalidate_after_beats: if core_stable_scope { 4 } else { 0 },
                            stage_index: 1,
                            confidence: if core_stable_scope {
                                Confidence::new((state_confidence.0 * 0.66).clamp(0.0, 1.0))
                            } else {
                                Confidence::new(0.0)
                            },
                        }),
                        continuity_transition(TempoContinuityTransitionInputs {
                            after_beats: if core_stable_scope { 12 } else { 0 },
                            action: TempoContinuityAction::Clear,
                            source: TempoContinuitySource::Cleared,
                            reason: TempoContinuityReason::InsufficientEvidence,
                            boundary_pressure: interpretation.support.boundary_pressure,
                            tempo_ambiguity,
                            revalidate_after_beats: if core_stable_scope { 4 } else { 0 },
                            stage_index: 2,
                            confidence: Confidence::new(0.0),
                        }),
                    ),
                };
            }
            let state_confidence = Confidence::new(base_confidence.max(if localized_edge_scope {
                0.72
            } else {
                0.76
            }));
            let (
                localized_trusted_beats,
                localized_revalidate_after_beats,
                localized_downgrade_after_beats,
                localized_clear_after_beats,
                localized_decay_confidence_scale,
            ) = localized_edge_horizons();
            TempoStateRecommendation {
                action: TempoStateAction::Lock,
                reason: if localized_edge_scope {
                    TempoStateReason::StableTempoWithEdgeDamage
                } else {
                    TempoStateReason::StableRefinedTempo
                },
                confidence: state_confidence,
                continuity: continuity_plan(
                    TempoContinuityPlanInputs {
                        action: TempoContinuityAction::Lock,
                        source: TempoContinuitySource::CurrentTempo,
                        reason: TempoContinuityReason::StableTempo,
                        boundary_pressure: interpretation.support.boundary_pressure,
                        tempo_ambiguity,
                        confidence: state_confidence,
                        trusted_beats: if localized_edge_scope {
                            localized_trusted_beats
                        } else {
                            16
                        },
                        revalidate_after_beats: if localized_edge_scope {
                            localized_revalidate_after_beats
                        } else {
                            12
                        },
                    },
                    continuity_transition(TempoContinuityTransitionInputs {
                        after_beats: if localized_edge_scope {
                            localized_revalidate_after_beats
                        } else {
                            12
                        },
                        action: TempoContinuityAction::Lock,
                        source: TempoContinuitySource::CurrentTempo,
                        reason: TempoContinuityReason::StableTempo,
                        boundary_pressure: interpretation.support.boundary_pressure,
                        tempo_ambiguity,
                        revalidate_after_beats: if localized_edge_scope {
                            localized_revalidate_after_beats
                        } else {
                            12
                        },
                        stage_index: 0,
                        confidence: state_confidence,
                    }),
                    continuity_transition(TempoContinuityTransitionInputs {
                        after_beats: if localized_edge_scope {
                            localized_downgrade_after_beats
                        } else {
                            20
                        },
                        action: TempoContinuityAction::Retain,
                        source: TempoContinuitySource::CurrentTempo,
                        reason: TempoContinuityReason::RevalidationDecay,
                        boundary_pressure: interpretation.support.boundary_pressure,
                        tempo_ambiguity,
                        revalidate_after_beats: if localized_edge_scope {
                            localized_revalidate_after_beats
                        } else {
                            12
                        },
                        stage_index: 1,
                        confidence: Confidence::new(
                            (state_confidence.0
                                * if localized_edge_scope {
                                    localized_decay_confidence_scale
                                } else {
                                    0.72
                                })
                            .clamp(0.0, 1.0),
                        ),
                    }),
                    continuity_transition(TempoContinuityTransitionInputs {
                        after_beats: if localized_edge_scope {
                            localized_clear_after_beats
                        } else {
                            28
                        },
                        action: TempoContinuityAction::Clear,
                        source: TempoContinuitySource::Cleared,
                        reason: TempoContinuityReason::InsufficientEvidence,
                        boundary_pressure: interpretation.support.boundary_pressure,
                        tempo_ambiguity,
                        revalidate_after_beats: if localized_edge_scope {
                            localized_revalidate_after_beats
                        } else {
                            12
                        },
                        stage_index: 2,
                        confidence: Confidence::new(0.0),
                    }),
                ),
            }
        }
        TempoRecommendation::UseCoreWindow
            if interpretation.profile.stability_score.0 >= 0.55
                && interpretation.support.boundary_pressure.0 >= 0.45 =>
        {
            let state_confidence = Confidence::new(base_confidence.max(0.58));
            TempoStateRecommendation {
                action: TempoStateAction::Monitor,
                reason: TempoStateReason::CoreWindowFallback,
                confidence: state_confidence,
                continuity: continuity_plan(
                    TempoContinuityPlanInputs {
                        action: TempoContinuityAction::Retain,
                        source: TempoContinuitySource::CoreWindow,
                        reason: TempoContinuityReason::CoreWindowCarry,
                        boundary_pressure: interpretation.support.boundary_pressure,
                        tempo_ambiguity,
                        confidence: state_confidence,
                        trusted_beats: 8,
                        revalidate_after_beats: 4,
                    },
                    continuity_transition(TempoContinuityTransitionInputs {
                        after_beats: 4,
                        action: TempoContinuityAction::Retain,
                        source: TempoContinuitySource::CoreWindow,
                        reason: TempoContinuityReason::CoreWindowCarry,
                        boundary_pressure: interpretation.support.boundary_pressure,
                        tempo_ambiguity,
                        revalidate_after_beats: 4,
                        stage_index: 0,
                        confidence: state_confidence,
                    }),
                    continuity_transition(TempoContinuityTransitionInputs {
                        after_beats: 8,
                        action: TempoContinuityAction::Reacquire,
                        source: TempoContinuitySource::PriorTempo,
                        reason: TempoContinuityReason::RevalidationDecay,
                        boundary_pressure: interpretation.support.boundary_pressure,
                        tempo_ambiguity,
                        revalidate_after_beats: 4,
                        stage_index: 1,
                        confidence: Confidence::new((state_confidence.0 * 0.68).clamp(0.0, 1.0)),
                    }),
                    continuity_transition(TempoContinuityTransitionInputs {
                        after_beats: 12,
                        action: TempoContinuityAction::Clear,
                        source: TempoContinuitySource::Cleared,
                        reason: TempoContinuityReason::InsufficientEvidence,
                        boundary_pressure: interpretation.support.boundary_pressure,
                        tempo_ambiguity,
                        revalidate_after_beats: 4,
                        stage_index: 2,
                        confidence: Confidence::new(0.0),
                    }),
                ),
            }
        }
        TempoRecommendation::UseRefined
            if interpretation.trust == TempoTrustLevel::Guarded
                && interpretation.profile.stability_score.0 >= 0.58 =>
        {
            let state_confidence = Confidence::new(base_confidence.max(0.56));
            TempoStateRecommendation {
                action: TempoStateAction::Monitor,
                reason: TempoStateReason::StableRefinedTempo,
                confidence: state_confidence,
                continuity: continuity_plan(
                    TempoContinuityPlanInputs {
                        action: TempoContinuityAction::Reacquire,
                        source: TempoContinuitySource::CurrentTempo,
                        reason: TempoContinuityReason::RevalidationDecay,
                        boundary_pressure: interpretation.support.boundary_pressure,
                        tempo_ambiguity,
                        confidence: state_confidence,
                        trusted_beats: 4,
                        revalidate_after_beats: 4,
                    },
                    continuity_transition(TempoContinuityTransitionInputs {
                        after_beats: 4,
                        action: TempoContinuityAction::Lock,
                        source: TempoContinuitySource::CurrentTempo,
                        reason: TempoContinuityReason::StableTempo,
                        boundary_pressure: interpretation.support.boundary_pressure,
                        tempo_ambiguity,
                        revalidate_after_beats: 4,
                        stage_index: 0,
                        confidence: Confidence::new((state_confidence.0 * 0.96).clamp(0.0, 1.0)),
                    }),
                    continuity_transition(TempoContinuityTransitionInputs {
                        after_beats: 8,
                        action: TempoContinuityAction::Reacquire,
                        source: TempoContinuitySource::CurrentTempo,
                        reason: TempoContinuityReason::RevalidationDecay,
                        boundary_pressure: interpretation.support.boundary_pressure,
                        tempo_ambiguity,
                        revalidate_after_beats: 4,
                        stage_index: 1,
                        confidence: Confidence::new((state_confidence.0 * 0.66).clamp(0.0, 1.0)),
                    }),
                    continuity_transition(TempoContinuityTransitionInputs {
                        after_beats: 12,
                        action: TempoContinuityAction::Clear,
                        source: TempoContinuitySource::Cleared,
                        reason: TempoContinuityReason::InsufficientEvidence,
                        boundary_pressure: interpretation.support.boundary_pressure,
                        tempo_ambiguity,
                        revalidate_after_beats: 4,
                        stage_index: 2,
                        confidence: Confidence::new(0.0),
                    }),
                ),
            }
        }
        _ => {
            let state_confidence = Confidence::new(
                (0.55 * (1.0 - interpretation.profile.stability_score.0)
                    + 0.45 * tempo_ambiguity.0)
                    .clamp(0.0, 1.0),
            );
            TempoStateRecommendation {
                action: TempoStateAction::Defer,
                reason: TempoStateReason::TempoDeferred,
                confidence: state_confidence,
                continuity: continuity_plan(
                    TempoContinuityPlanInputs {
                        action: TempoContinuityAction::Clear,
                        source: TempoContinuitySource::Cleared,
                        reason: TempoContinuityReason::InsufficientEvidence,
                        boundary_pressure: interpretation.support.boundary_pressure,
                        tempo_ambiguity,
                        confidence: state_confidence,
                        trusted_beats: 0,
                        revalidate_after_beats: 0,
                    },
                    continuity_transition(TempoContinuityTransitionInputs {
                        after_beats: 0,
                        action: TempoContinuityAction::Clear,
                        source: TempoContinuitySource::Cleared,
                        reason: TempoContinuityReason::InsufficientEvidence,
                        boundary_pressure: interpretation.support.boundary_pressure,
                        tempo_ambiguity,
                        revalidate_after_beats: 0,
                        stage_index: 0,
                        confidence: Confidence::new(0.0),
                    }),
                    continuity_transition(TempoContinuityTransitionInputs {
                        after_beats: 0,
                        action: TempoContinuityAction::Clear,
                        source: TempoContinuitySource::Cleared,
                        reason: TempoContinuityReason::InsufficientEvidence,
                        boundary_pressure: interpretation.support.boundary_pressure,
                        tempo_ambiguity,
                        revalidate_after_beats: 0,
                        stage_index: 1,
                        confidence: Confidence::new(0.0),
                    }),
                    continuity_transition(TempoContinuityTransitionInputs {
                        after_beats: 0,
                        action: TempoContinuityAction::Clear,
                        source: TempoContinuitySource::Cleared,
                        reason: TempoContinuityReason::InsufficientEvidence,
                        boundary_pressure: interpretation.support.boundary_pressure,
                        tempo_ambiguity,
                        revalidate_after_beats: 0,
                        stage_index: 2,
                        confidence: Confidence::new(0.0),
                    }),
                ),
            }
        }
    }
}
