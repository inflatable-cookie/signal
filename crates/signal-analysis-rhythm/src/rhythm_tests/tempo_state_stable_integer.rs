use super::*;

#[test]
fn tempo_state_locks_stable_integer_interpretation() {
    let interpretation = synthetic_tempo_interpretation(
        super::TempoRecommendation::SnapInteger,
        super::TempoTrustLevel::Stable,
        super::TempoInterpretationReason::NearIntegerPulse,
        120.0,
        Some(120.0),
        0.86,
        0.08,
        0.22,
        0.82,
    );
    let state = super::tempo_state_recommendation(
        interpretation,
        super::Confidence::new(0.9),
        super::Confidence::new(0.12),
    );

    assert_eq!(state.action, super::TempoStateAction::Lock);
    assert_eq!(state.reason, super::TempoStateReason::StableIntegerTempo);
    assert!(state.confidence.0 >= 0.82);
    assert_eq!(state.continuity.action, super::TempoContinuityAction::Lock);
    assert_eq!(
        state.continuity.reason,
        super::TempoContinuityReason::IntegerTempoSnap
    );
    assert_eq!(
        state.continuity.provenance,
        super::TempoContinuityProvenance::IntegerSnap
    );
    assert_eq!(
        state.continuity.severity,
        super::TempoContinuitySeverity::Confirmed
    );
    assert_eq!(
        state.continuity.history,
        super::TempoContinuityHistory::Reinforcing
    );
    assert_eq!(state.continuity.arc, super::TempoContinuityArc::Recovering);
    assert_eq!(
        state.continuity.arc_rationale,
        super::TempoContinuityArcRationale::RefreshStrength
    );
    assert_eq!(
        state.continuity.arc_decision.recommendation,
        super::TempoContinuityArcRecommendation::KeepLock
    );
    assert_eq!(
        state.continuity.arc_decision.action,
        super::TempoContinuityArcAction::LockCurrentTempo
    );
    assert_eq!(
        state.continuity.arc_decision.severity,
        super::TempoContinuitySeverity::Confirmed
    );
    assert_eq!(
        state.continuity.arc_decision.fallback_action,
        super::TempoContinuityArcAction::ReacquireCurrentTempo
    );
    assert_eq!(
        state.continuity.arc_decision.downgrade_rationale,
        super::TempoContinuityArcDowngradeRationale::StabilityWindowEnd
    );
    assert_eq!(
        state.continuity.arc_decision.downgrade_trend,
        super::TempoContinuityArcDowngradeTrend::Easing
    );
    assert_eq!(
        state.continuity.arc_decision.downgrade_trend_rationale,
        super::TempoContinuityArcDowngradeTrendRationale::StabilityWindowCarry
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
        12
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
        super::TempoContinuityArcDowngradeStageRationale::StabilityWindow
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
            .stability_window_pressure
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
        super::TempoContinuityProvenance::IntegerSnap
    );
    assert_eq!(
        state.continuity.arc_decision.expiry,
        super::TempoContinuityArcActionExpiry {
            guaranteed_until_beats: 16,
            fallback_after_beats: 20,
            clear_after_beats: 28,
            max_failed_revalidations: 3,
        }
    );
    assert_eq!(
        state.continuity.trigger,
        super::TempoContinuityTrigger::StableRevalidation
    );
    assert_eq!(
        state.continuity.unresolved,
        super::TempoContinuityUnresolvedSpan {
            beats: 0,
            failed_revalidations: 0,
        }
    );
    assert_eq!(
        state.continuity.causes.primary,
        super::TempoContinuityCause::StableTempoEvidence
    );
    assert_eq!(state.continuity.expiry.guaranteed_until_beats, 16);
    assert_eq!(state.continuity.expiry.max_failed_revalidations, 3);
    assert!(state.continuity.refresh_strength.0 > 0.9);
    assert_eq!(
        state.continuity.lifecycle.decay[1].action,
        super::TempoContinuityAction::Clear
    );
    assert_eq!(
        state.continuity.lifecycle.decay[1].provenance,
        super::TempoContinuityProvenance::NoTempo
    );
    assert_eq!(
        state.continuity.lifecycle.decay[1].severity,
        super::TempoContinuitySeverity::Cleared
    );
    assert_eq!(
        state.continuity.lifecycle.decay[1].history,
        super::TempoContinuityHistory::Degrading
    );
}
