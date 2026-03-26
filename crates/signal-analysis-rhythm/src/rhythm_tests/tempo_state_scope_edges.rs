use super::*;

#[test]
fn tempo_state_locks_edge_damaged_integer_scope() {
    let interpretation = synthetic_tempo_interpretation(
        super::TempoRecommendation::SnapInteger,
        super::TempoTrustLevel::Stable,
        super::TempoInterpretationReason::NearIntegerPulse,
        128.0,
        Some(128.0),
        0.80,
        0.08,
        0.22,
        0.45,
    );
    let state = super::tempo_state_recommendation_with_scope(
        interpretation,
        super::Confidence::new(0.666),
        super::Confidence::new(0.18),
        scope_summary(super::TempoStabilityScope::StableWithLocalizedEdgeDamage),
    );

    assert_eq!(state.action, super::TempoStateAction::Lock);
    assert_eq!(
        state.reason,
        super::TempoStateReason::StableTempoWithEdgeDamage
    );
    assert!(state.confidence.0 >= 0.76);
    assert_eq!(state.continuity.action, super::TempoContinuityAction::Lock);
    assert_eq!(
        state.continuity.source,
        super::TempoContinuitySource::CurrentTempo
    );
    assert_eq!(state.continuity.expiry.guaranteed_until_beats, 10);
    assert_eq!(state.continuity.expiry.downgrade_after_beats, 12);
    assert_eq!(state.continuity.expiry.clear_after_beats, 18);
    assert_eq!(
        state.continuity.arc_decision.expiry.fallback_after_beats,
        12
    );
}

#[test]
fn tempo_state_monitors_core_stable_integer_scope() {
    let interpretation = synthetic_tempo_interpretation(
        super::TempoRecommendation::SnapInteger,
        super::TempoTrustLevel::Stable,
        super::TempoInterpretationReason::NearIntegerPulse,
        128.0,
        Some(128.0),
        0.79,
        0.042,
        0.20,
        0.44,
    );
    let state = super::tempo_state_recommendation_with_scope(
        interpretation,
        super::Confidence::new(0.72),
        super::Confidence::new(0.16),
        scope_summary(super::TempoStabilityScope::CoreStableOnly),
    );

    assert_eq!(state.action, super::TempoStateAction::Monitor);
    assert_eq!(state.reason, super::TempoStateReason::CoreStableTempo);
    assert_eq!(
        state.continuity.action,
        super::TempoContinuityAction::Reacquire
    );
    assert_eq!(
        state.continuity.source,
        super::TempoContinuitySource::CurrentTempo
    );
    assert_eq!(
        state.continuity.lifecycle.refresh.action,
        super::TempoContinuityAction::Lock
    );
    assert_eq!(state.continuity.expiry.guaranteed_until_beats, 4);
    assert_eq!(state.continuity.expiry.clear_after_beats, 12);
}
