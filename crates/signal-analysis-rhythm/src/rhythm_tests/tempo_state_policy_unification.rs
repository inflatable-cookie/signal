use super::*;

#[test]
fn tempo_state_stable_policy_preserves_integer_and_refined_divergence() {
    let integer_interpretation = synthetic_tempo_interpretation(
        super::TempoRecommendation::SnapInteger,
        super::TempoTrustLevel::Stable,
        super::TempoInterpretationReason::NearIntegerPulse,
        128.0,
        Some(128.0),
        0.80,
        0.05,
        0.16,
        0.44,
    );
    let refined_interpretation = synthetic_tempo_interpretation(
        super::TempoRecommendation::UseRefined,
        super::TempoTrustLevel::Stable,
        super::TempoInterpretationReason::StableRefinedPulse,
        127.8,
        None,
        0.80,
        0.10,
        0.16,
        0.44,
    );

    let integer_state = super::tempo_state_recommendation_with_scope(
        integer_interpretation,
        super::Confidence::new(0.71),
        super::Confidence::new(0.12),
        scope_summary(super::TempoStabilityScope::StableWithLocalizedEdgeDamage),
    );
    let refined_state = super::tempo_state_recommendation_with_scope(
        refined_interpretation,
        super::Confidence::new(0.71),
        super::Confidence::new(0.12),
        scope_summary(super::TempoStabilityScope::StableWithLocalizedEdgeDamage),
    );

    assert_eq!(integer_state.action, super::TempoStateAction::Lock);
    assert_eq!(refined_state.action, super::TempoStateAction::Lock);
    assert_eq!(
        integer_state.reason,
        super::TempoStateReason::StableTempoWithEdgeDamage
    );
    assert_eq!(
        refined_state.reason,
        super::TempoStateReason::StableTempoWithEdgeDamage
    );
    assert_eq!(
        integer_state.continuity.reason,
        super::TempoContinuityReason::IntegerTempoSnap
    );
    assert_eq!(
        refined_state.continuity.reason,
        super::TempoContinuityReason::StableTempo
    );
    assert!(
        integer_state.confidence.0 > refined_state.confidence.0,
        "integer policy should retain the stronger localized-edge floor"
    );
    assert_eq!(integer_state.continuity.expiry.guaranteed_until_beats, 12);
    assert_eq!(refined_state.continuity.expiry.guaranteed_until_beats, 12);
    assert_eq!(integer_state.continuity.expiry.clear_after_beats, 20);
    assert_eq!(refined_state.continuity.expiry.clear_after_beats, 20);
}
