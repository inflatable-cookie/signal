use super::*;

#[test]
fn tempo_continuity_calibrates_severity_history_and_refresh_strength() {
    let integer = super::tempo_state_recommendation(
        synthetic_tempo_interpretation(
            super::TempoRecommendation::SnapInteger,
            super::TempoTrustLevel::Stable,
            super::TempoInterpretationReason::NearIntegerPulse,
            120.0,
            Some(120.0),
            0.86,
            0.08,
            0.22,
            0.82,
        ),
        super::Confidence::new(0.9),
        super::Confidence::new(0.12),
    );
    let core_window = super::tempo_state_recommendation(
        synthetic_tempo_interpretation(
            super::TempoRecommendation::UseCoreWindow,
            super::TempoTrustLevel::Guarded,
            super::TempoInterpretationReason::StableCoreWindow,
            90.0,
            None,
            0.64,
            0.07,
            0.72,
            0.64,
        ),
        super::Confidence::new(0.72),
        super::Confidence::new(0.18),
    );
    let guarded_refined = super::tempo_state_recommendation(
        synthetic_tempo_interpretation(
            super::TempoRecommendation::UseRefined,
            super::TempoTrustLevel::Guarded,
            super::TempoInterpretationReason::StableRefinedPulse,
            117.8,
            None,
            0.61,
            0.09,
            0.32,
            0.62,
        ),
        super::Confidence::new(0.71),
        super::Confidence::new(0.21),
    );
    let deferred = super::tempo_state_recommendation(
        synthetic_tempo_interpretation(
            super::TempoRecommendation::Defer,
            super::TempoTrustLevel::Tentative,
            super::TempoInterpretationReason::UnstableTempo,
            89.9,
            None,
            0.38,
            0.03,
            0.8,
            0.3,
        ),
        super::Confidence::new(0.42),
        super::Confidence::new(0.55),
    );

    assert_eq!(
        integer.continuity.severity,
        super::TempoContinuitySeverity::Confirmed
    );
    assert_eq!(
        integer.continuity.history,
        super::TempoContinuityHistory::Reinforcing
    );
    assert_eq!(
        core_window.continuity.severity,
        super::TempoContinuitySeverity::Guarded
    );
    assert_eq!(
        core_window.continuity.history,
        super::TempoContinuityHistory::Preserving
    );
    assert_eq!(
        guarded_refined.continuity.severity,
        super::TempoContinuitySeverity::Fragile
    );
    assert_eq!(
        guarded_refined.continuity.history,
        super::TempoContinuityHistory::Preserving
    );
    assert_eq!(
        deferred.continuity.severity,
        super::TempoContinuitySeverity::Cleared
    );
    assert_eq!(
        deferred.continuity.history,
        super::TempoContinuityHistory::Degrading
    );
    assert!(integer.continuity.refresh_strength.0 > core_window.continuity.refresh_strength.0);
    assert!(core_window.continuity.refresh_strength.0 > deferred.continuity.refresh_strength.0);
    assert!(
        guarded_refined
            .continuity
            .lifecycle
            .refresh
            .refresh_strength
            .0
            > guarded_refined.continuity.refresh_strength.0
    );
    assert_eq!(
        deferred.continuity.lifecycle.decay[1].refresh_strength.0,
        0.0
    );
}
