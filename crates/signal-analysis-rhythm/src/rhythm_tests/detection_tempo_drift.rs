use super::*;

#[test]
fn beat_tracker_calibrates_local_tempo_drift_between_stable_and_irregular_fixtures() {
    let (_, stable) = analyze_preset(RhythmPreset::NeutralClick120);
    let slow = analyze_fixture(&click_track(48_000, 90.0, 8.0));
    let (_, weak_backbeat) = analyze_preset(RhythmPreset::WeakBackbeat118);
    let (_, section) = analyze_preset(RhythmPreset::SectionTransition122);
    let (_, ambiguous) = analyze_preset(RhythmPreset::AmbiguousSubdivision90);

    assert!(
        weak_backbeat.tempo_diagnostics.mean_abs_deviation_bpm
            > stable.tempo_diagnostics.mean_abs_deviation_bpm
    );
    assert!(section.tempo_diagnostics.drift_span_bpm >= stable.tempo_diagnostics.drift_span_bpm);
    assert!(!weak_backbeat.tempo_diagnostics.windowed_tempi.is_empty());
    assert!(!section.tempo_diagnostics.windowed_tempi.is_empty());
    assert!(stable.tempo_diagnostics.boundary_bias_bpm > 0.0);
    assert!(
        section.tempo_diagnostics.trend.fit_mean_abs_deviation_bpm
            >= stable.tempo_diagnostics.trend.fit_mean_abs_deviation_bpm
    );
    assert!(
        slow.tempo_diagnostics
            .beat_grid_error
            .edge_mean_abs_residual_ms
            > slow
                .tempo_diagnostics
                .beat_grid_error
                .core_mean_abs_residual_ms
    );
    assert!(
        slow.tempo_diagnostics
            .beat_grid_error
            .mean_abs_anchored_drift_ms
            > stable
                .tempo_diagnostics
                .beat_grid_error
                .mean_abs_anchored_drift_ms
    );
    assert_eq!(
        slow.tempo_interpretation.recommendation,
        super::TempoRecommendation::SnapInteger
    );
    assert_eq!(
        slow.tempo_interpretation.reason,
        super::TempoInterpretationReason::NearIntegerPulse
    );
    assert!(
        (slow.tempo_interpretation.recommended_bpm - 90.0).abs() < 0.1,
        "slow recommended bpm {}",
        slow.tempo_interpretation.recommended_bpm
    );
    assert!(
        slow.tempo_interpretation.profile.boundary_edge_gap_ms > 0.0,
        "slow boundary edge gap {}",
        slow.tempo_interpretation.profile.boundary_edge_gap_ms
    );
    assert_eq!(
        slow.tempo_diagnostics.stability_scope.scope,
        super::TempoStabilityScope::CoreStableOnly
    );
    assert_eq!(
        weak_backbeat.tempo_interpretation.recommendation,
        super::TempoRecommendation::UseRefined
    );
    assert_eq!(
        weak_backbeat.tempo_interpretation.reason,
        super::TempoInterpretationReason::StableRefinedPulse
    );
    assert!(matches!(
        ambiguous.tempo_interpretation.recommendation,
        super::TempoRecommendation::UseCoreWindow | super::TempoRecommendation::UseRefined
    ));
    assert!(matches!(
        ambiguous.tempo_interpretation.trust,
        super::TempoTrustLevel::Guarded | super::TempoTrustLevel::Stable
    ));
    assert!(ambiguous.tempo_interpretation.profile.stability_score.0 < 0.85);
}
