use super::*;

#[test]
fn tempo_state_monitors_core_window_fallback() {
    let interpretation = synthetic_tempo_interpretation(
        super::TempoRecommendation::UseCoreWindow,
        super::TempoTrustLevel::Guarded,
        super::TempoInterpretationReason::StableCoreWindow,
        90.0,
        None,
        0.64,
        0.07,
        0.72,
        0.64,
    );
    let state = super::tempo_state_recommendation(
        interpretation,
        super::Confidence::new(0.72),
        super::Confidence::new(0.18),
    );

    assert_eq!(state.action, super::TempoStateAction::Monitor);
    assert_eq!(state.reason, super::TempoStateReason::CoreWindowFallback);
    assert!(state.confidence.0 >= 0.58);
    assert_eq!(
        state.continuity.action,
        super::TempoContinuityAction::Retain
    );
    assert_eq!(
        state.continuity.source,
        super::TempoContinuitySource::CoreWindow
    );
    assert_eq!(
        state.continuity.provenance,
        super::TempoContinuityProvenance::CoreWindowEstimate
    );
    assert_eq!(
        state.continuity.severity,
        super::TempoContinuitySeverity::Guarded
    );
    assert_eq!(
        state.continuity.history,
        super::TempoContinuityHistory::Preserving
    );
    assert_eq!(state.continuity.arc, super::TempoContinuityArc::Stalling);
    assert_eq!(
        state.continuity.arc_rationale,
        super::TempoContinuityArcRationale::BoundaryDrift
    );
    assert_eq!(
        state.continuity.arc_decision.recommendation,
        super::TempoContinuityArcRecommendation::MonitorRecovery
    );
    assert_eq!(
        state.continuity.arc_decision.action,
        super::TempoContinuityArcAction::PreferCoreWindowTempo
    );
    assert_eq!(
        state.continuity.arc_decision.severity,
        super::TempoContinuitySeverity::Guarded
    );
    assert_eq!(
        state.continuity.arc_decision.fallback_action,
        super::TempoContinuityArcAction::PreservePriorTempo
    );
    assert_eq!(
        state.continuity.arc_decision.downgrade_rationale,
        super::TempoContinuityArcDowngradeRationale::BoundaryDrift
    );
    assert_eq!(
        state.continuity.arc_decision.downgrade_trend,
        super::TempoContinuityArcDowngradeTrend::Rising
    );
    assert_eq!(
        state.continuity.arc_decision.downgrade_trend_rationale,
        super::TempoContinuityArcDowngradeTrendRationale::BoundaryEscalation
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
        8
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
        super::TempoContinuityArcDowngradeStageRationale::BoundaryDrift
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
            | Some(super::TempoContinuityArcDowngradeStageRationale::BoundaryDrift)
            | Some(super::TempoContinuityArcDowngradeStageRationale::StabilityWindow)
            | None
    ));
    assert!(
        state
            .continuity
            .arc_decision
            .downgrade_trend_support
            .next_stage_pressure
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
            .boundary_drift_pressure
            .0
            > state
                .continuity
                .arc_decision
                .downgrade_support
                .ambiguity_pressure
                .0
    );
    assert_eq!(
        state.continuity.arc_decision.provenance,
        super::TempoContinuityProvenance::CoreWindowEstimate
    );
    assert_eq!(
        state.continuity.arc_decision.expiry,
        super::TempoContinuityArcActionExpiry {
            guaranteed_until_beats: 8,
            fallback_after_beats: 8,
            clear_after_beats: 12,
            max_failed_revalidations: 2,
        }
    );
    assert_eq!(
        state.continuity.trigger,
        super::TempoContinuityTrigger::BoundaryDrift
    );
    assert_eq!(
        state.continuity.unresolved,
        super::TempoContinuityUnresolvedSpan {
            beats: 8,
            failed_revalidations: 2,
        }
    );
    assert_eq!(
        state.continuity.causes.primary,
        super::TempoContinuityCause::BoundaryDrift
    );
    assert_eq!(state.continuity.expiry.guaranteed_until_beats, 8);
    assert_eq!(state.continuity.expiry.max_failed_revalidations, 3);
    assert_eq!(
        state.continuity.lifecycle.decay[0].action,
        super::TempoContinuityAction::Reacquire
    );
    assert_eq!(
        state.continuity.lifecycle.decay[0].provenance,
        super::TempoContinuityProvenance::PriorTempoCarry
    );
    assert_eq!(
        state.continuity.lifecycle.decay[0].severity,
        super::TempoContinuitySeverity::Fragile
    );
    assert_eq!(
        state.continuity.lifecycle.decay[0].history,
        super::TempoContinuityHistory::Degrading
    );
    assert_eq!(
        state.continuity.lifecycle.decay[0].trigger,
        super::TempoContinuityTrigger::PriorTempoDrift
    );
}
