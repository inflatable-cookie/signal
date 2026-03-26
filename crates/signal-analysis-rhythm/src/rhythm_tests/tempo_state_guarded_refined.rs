use super::*;

#[test]
fn tempo_state_reacquires_guarded_refined_estimate_before_clearing() {
    let interpretation = synthetic_tempo_interpretation(
        super::TempoRecommendation::UseRefined,
        super::TempoTrustLevel::Guarded,
        super::TempoInterpretationReason::StableRefinedPulse,
        117.8,
        None,
        0.61,
        0.09,
        0.32,
        0.62,
    );
    let state = super::tempo_state_recommendation(
        interpretation,
        super::Confidence::new(0.71),
        super::Confidence::new(0.21),
    );

    assert_eq!(state.action, super::TempoStateAction::Monitor);
    assert_eq!(state.reason, super::TempoStateReason::StableRefinedTempo);
    assert_eq!(
        state.continuity.action,
        super::TempoContinuityAction::Reacquire
    );
    assert_eq!(
        state.continuity.source,
        super::TempoContinuitySource::CurrentTempo
    );
    assert_eq!(
        state.continuity.provenance,
        super::TempoContinuityProvenance::GuardedRefinedEstimate
    );
    assert_eq!(
        state.continuity.severity,
        super::TempoContinuitySeverity::Fragile
    );
    assert_eq!(
        state.continuity.history,
        super::TempoContinuityHistory::Preserving
    );
    assert_eq!(state.continuity.arc, super::TempoContinuityArc::Recovering);
    assert_eq!(
        state.continuity.arc_rationale,
        super::TempoContinuityArcRationale::RefreshStrength
    );
    assert_eq!(
        state.continuity.arc_decision.recommendation,
        super::TempoContinuityArcRecommendation::MonitorRecovery
    );
    assert_eq!(
        state.continuity.arc_decision.action,
        super::TempoContinuityArcAction::ReacquireCurrentTempo
    );
    assert_eq!(
        state.continuity.arc_decision.severity,
        super::TempoContinuitySeverity::Fragile
    );
    assert_eq!(
        state.continuity.arc_decision.fallback_action,
        super::TempoContinuityArcAction::ClearTempo
    );
    assert_eq!(
        state.continuity.arc_decision.downgrade_rationale,
        super::TempoContinuityArcDowngradeRationale::AmbiguityCarry
    );
    assert_eq!(
        state.continuity.arc_decision.downgrade_trend,
        super::TempoContinuityArcDowngradeTrend::Easing
    );
    assert_eq!(
        state.continuity.arc_decision.downgrade_trend_rationale,
        super::TempoContinuityArcDowngradeTrendRationale::AmbiguityCarry
    );
    assert_eq!(
        state.continuity.arc_decision.downgrade_inflection.stage,
        super::TempoContinuityArcDowngradeInflectionStage::NextStage
    );
    assert_eq!(
        state
            .continuity
            .arc_decision
            .downgrade_inflection
            .after_beats,
        4
    );
    assert_eq!(
        state
            .continuity
            .arc_decision
            .downgrade_inflection
            .competing_stage,
        Some(super::TempoContinuityArcDowngradeInflectionStage::TerminalClear)
    );
    assert!(
        state
            .continuity
            .arc_decision
            .downgrade_inflection
            .competing_after_beats
            > state
                .continuity
                .arc_decision
                .downgrade_inflection
                .after_beats
    );
    assert!(
        state
            .continuity
            .arc_decision
            .downgrade_inflection
            .competing_support
            .0
            >= 0.55
    );
    assert!(
        state
            .continuity
            .arc_decision
            .downgrade_inflection
            .balance
            .competing_weight
            .0
            >= 0.0
    );
    assert!(
        state
            .continuity
            .arc_decision
            .downgrade_inflection
            .balance
            .dominance
            .0
            >= 0.0
    );
    assert_eq!(
        state
            .continuity
            .arc_decision
            .downgrade_inflection
            .rationale_balance
            .primary
            .dominant,
        super::TempoContinuityArcDowngradeStageRationale::AmbiguityCarry
    );
    assert!(matches!(
        state
            .continuity
            .arc_decision
            .downgrade_inflection
            .rationale_balance
            .competing
            .map(|weights| weights.dominant),
        Some(super::TempoContinuityArcDowngradeStageRationale::EvidenceLoss)
            | Some(super::TempoContinuityArcDowngradeStageRationale::AmbiguityCarry)
            | Some(super::TempoContinuityArcDowngradeStageRationale::StabilityWindow)
            | None
    ));
    assert!(
        state
            .continuity
            .arc_decision
            .downgrade_trend_support
            .terminal_pressure
            .0
            > state
                .continuity
                .arc_decision
                .downgrade_trend_support
                .current_pressure
                .0
    );
    assert!(
        state
            .continuity
            .arc_decision
            .downgrade_support
            .ambiguity_pressure
            .0
            > state
                .continuity
                .arc_decision
                .downgrade_support
                .boundary_drift_pressure
                .0
    );
    assert_eq!(
        state.continuity.arc_decision.provenance,
        super::TempoContinuityProvenance::GuardedRefinedEstimate
    );
    assert_eq!(
        state.continuity.arc_decision.expiry,
        super::TempoContinuityArcActionExpiry {
            guaranteed_until_beats: 4,
            fallback_after_beats: 12,
            clear_after_beats: 12,
            max_failed_revalidations: 3,
        }
    );
    assert_eq!(
        state.continuity.trigger,
        super::TempoContinuityTrigger::AmbiguityCarry
    );
    assert_eq!(
        state.continuity.unresolved,
        super::TempoContinuityUnresolvedSpan {
            beats: 4,
            failed_revalidations: 1,
        }
    );
    assert_eq!(
        state.continuity.causes.primary,
        super::TempoContinuityCause::TempoAmbiguity
    );
    assert_eq!(state.continuity.expiry.guaranteed_until_beats, 4);
    assert_eq!(state.continuity.expiry.downgrade_after_beats, 8);
    assert_eq!(state.continuity.expiry.clear_after_beats, 12);
    assert_eq!(state.continuity.expiry.max_failed_revalidations, 3);
    assert_eq!(
        state.continuity.lifecycle.refresh.provenance,
        super::TempoContinuityProvenance::StableRefinedEstimate
    );
    assert_eq!(
        state.continuity.lifecycle.refresh.severity,
        super::TempoContinuitySeverity::Confirmed
    );
    assert_eq!(
        state.continuity.lifecycle.refresh.history,
        super::TempoContinuityHistory::Reinforcing
    );
    assert_eq!(
        state.continuity.lifecycle.refresh.trigger,
        super::TempoContinuityTrigger::StableRevalidation
    );
    assert_eq!(
        state.continuity.lifecycle.decay[0].provenance,
        super::TempoContinuityProvenance::GuardedRefinedEstimate
    );
    assert!(
        state.continuity.lifecycle.refresh.refresh_strength.0 > state.continuity.refresh_strength.0
    );
}
