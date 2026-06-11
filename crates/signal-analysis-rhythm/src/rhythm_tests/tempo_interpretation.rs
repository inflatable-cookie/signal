use super::*;

#[test]
fn tempo_interpretation_prefers_refined_when_snap_benefit_is_too_small() {
    let diagnostics = synthetic_tempo_diagnostics(120.0, 0.02, 0.04, 0.03, 1.0, 0.8, 1.4, 1.1);
    let interpretation = super::interpret_tempo(
        120.01,
        super::Confidence::new(0.92),
        super::Confidence::new(0.08),
        &diagnostics,
    );

    assert_eq!(
        interpretation.recommendation,
        super::TempoRecommendation::UseRefined
    );
    assert_eq!(
        interpretation.reason,
        super::TempoInterpretationReason::StableRefinedPulse
    );
    assert!(interpretation.profile.snap_error_bpm < 0.04);
    assert!(interpretation.profile.stability_score.0 > 0.8);
}

#[test]
fn tempo_interpretation_defers_when_edge_pressure_overwhelms_stability() {
    let diagnostics = synthetic_tempo_diagnostics(90.0, 2.4, 0.35, 0.28, 60.0, 25.0, 120.0, 140.0);
    let interpretation = super::interpret_tempo(
        89.6,
        super::Confidence::new(0.55),
        super::Confidence::new(0.42),
        &diagnostics,
    );

    assert_eq!(
        interpretation.recommendation,
        super::TempoRecommendation::Defer
    );
    assert_eq!(
        interpretation.reason,
        super::TempoInterpretationReason::UnstableTempo
    );
    assert_eq!(interpretation.trust, super::TempoTrustLevel::Tentative);
    assert!(interpretation.profile.boundary_edge_gap_ms > 2.5);
    assert!(interpretation.profile.stability_score.0 < 0.7);
}

#[test]
fn tempo_interpretation_snaps_stable_near_integer_master_like_case() {
    let diagnostics = synthetic_tempo_diagnostics_with_counts(
        127.94273, 0.064, -0.1097, 0.48279, 45.998, 45.774, 83.272, 86.989, 738, 735, 739,
    );
    let interpretation = super::interpret_tempo(
        127.97321,
        super::Confidence::new(0.666),
        super::Confidence::new(1.0),
        &diagnostics,
    );

    assert_eq!(
        interpretation.recommendation,
        super::TempoRecommendation::SnapInteger
    );
    assert_eq!(
        interpretation.reason,
        super::TempoInterpretationReason::NearIntegerPulse
    );
    assert_eq!(interpretation.snapped_bpm, Some(128.0));
    assert!(interpretation.support.integer_closeness.0 > 0.9);
    assert!(interpretation.support.core_consensus.0 > 0.85);
    assert!(interpretation.support.drift_stability.0 > 0.55);
    assert!(interpretation.support.grid_stability.0 > 0.35);
    assert!(interpretation.support.boundary_pressure.0 < 0.3);
    assert!(interpretation.profile.stability_score.0 > 0.64);
}

#[test]
fn tempo_interpretation_localizes_boundary_pressure_for_long_form_stable_tracks() {
    let short_form = synthetic_tempo_diagnostics(
        127.94273, 0.064, -0.1097, 0.48279, 45.998, 45.774, 83.272, 86.989,
    );
    let long_form = synthetic_tempo_diagnostics_with_counts(
        127.94273, 0.064, -0.1097, 0.48279, 45.998, 45.774, 83.272, 86.989, 738, 735, 739,
    );

    let short_interpretation = super::interpret_tempo(
        127.97321,
        super::Confidence::new(0.666),
        super::Confidence::new(1.0),
        &short_form,
    );
    let long_interpretation = super::interpret_tempo(
        127.97321,
        super::Confidence::new(0.666),
        super::Confidence::new(1.0),
        &long_form,
    );

    assert!(
        short_interpretation.support.boundary_pressure.0
            > long_interpretation.support.boundary_pressure.0,
        "short={} long={}",
        short_interpretation.support.boundary_pressure.0,
        long_interpretation.support.boundary_pressure.0
    );
    assert_eq!(
        long_interpretation.recommendation,
        super::TempoRecommendation::SnapInteger
    );
    assert!(
        long_interpretation.support.boundary_pressure.0 < 0.3,
        "long boundary pressure should be localized: {}",
        long_interpretation.support.boundary_pressure.0
    );
}

#[test]
fn tempo_interpretation_snaps_stable_near_integer_with_localized_tail_outliers() {
    let mut diagnostics = synthetic_tempo_diagnostics_with_counts(
        127.94273, 0.064, -0.1097, 0.48279, 45.998, 45.774, 83.272, 86.989, 738, 735, 739,
    );
    diagnostics.beat_interval_outliers = super::BeatIntervalOutlierDiagnostics {
        total_intervals: 738,
        retained_intervals: 670,
        rejected_intervals: 68,
        leading_rejected_intervals: 0,
        trailing_rejected_intervals: 3,
        median_interval: 60.0 / 127.94273,
        median_abs_deviation: 0.000607,
        max_rejected_deviation_ratio: 0.384,
    };

    let interpretation = super::interpret_tempo(
        127.96191,
        super::Confidence::new(0.666),
        super::Confidence::new(1.0),
        &diagnostics,
    );

    assert_eq!(
        interpretation.recommendation,
        super::TempoRecommendation::SnapInteger
    );
    assert_eq!(interpretation.snapped_bpm, Some(128.0));
}
