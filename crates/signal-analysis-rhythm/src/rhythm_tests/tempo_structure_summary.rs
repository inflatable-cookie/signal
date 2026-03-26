use super::*;

#[test]
fn beat_tracker_exposes_tempo_structure_summary_for_whole_track_stable_click_track() {
    let result = analyze_fixture(&click_track(48_000, 120.0, 8.0));
    let summary = result.tempo_structure_summary();

    assert_eq!(summary.trust, super::TempoTrustLevel::Stable);
    assert_eq!(
        summary.recommendation,
        super::TempoRecommendation::SnapInteger
    );
    assert_eq!(
        summary.stability_scope.scope,
        super::TempoStabilityScope::WholeTrackStable
    );
    assert_eq!(summary.selected_bpm, Some(120.0));
    assert_eq!(summary.continuity.action, super::TempoStateAction::Lock);
    assert_eq!(
        summary.continuity.continuity_action,
        super::TempoContinuityAction::Lock
    );
    assert_eq!(
        summary.continuity.current.source,
        super::TempoConsumptionSource::SnappedCurrentTempo
    );
    assert_eq!(summary.continuity.fallback_after_beats, 20);
    assert_eq!(summary.segments.len(), 1);
    assert_eq!(
        summary.segments[0].kind,
        super::TempoSegmentKind::WholeTrack
    );
    assert!((summary.segments[0].representative_bpm - summary.core_window_bpm).abs() < 1.0);
    assert!(summary.segments[0].coverage.0 >= 0.99);
}

#[test]
fn tempo_structure_summary_surfaces_localized_edge_damage_segments() {
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
        median_abs_deviation: 0.000_607,
        max_rejected_deviation_ratio: 0.384,
    };
    diagnostics.edge_trimmed_stable_span = Some(super::BeatGridCoreSpanDiagnostics {
        start_beat_index: 0,
        end_beat_index: 735,
        start_seconds: 0.447,
        end_seconds: 345.333,
        coverage: super::Confidence::new(0.996),
        retained_windows: 732,
        total_windows: 735,
        trimmed_leading_windows: 0,
        trimmed_trailing_windows: 3,
        interior_rejected_windows: 14,
    });
    diagnostics.stable_core_span = Some(super::BeatGridCoreSpanDiagnostics {
        start_beat_index: 216,
        end_beat_index: 706,
        start_seconds: 101.698,
        end_seconds: 331.641,
        coverage: super::Confidence::new(0.664),
        retained_windows: 487,
        total_windows: 735,
        trimmed_leading_windows: 216,
        trimmed_trailing_windows: 32,
        interior_rejected_windows: 0,
    });
    diagnostics.stability_scope = super::classify_tempo_stability_scope(
        diagnostics.windowed_tempi.len(),
        &diagnostics.beat_interval_outliers,
        diagnostics.edge_trimmed_stable_span,
        diagnostics.stable_core_span,
    );

    let interpretation = super::interpret_tempo(
        127.96191,
        super::Confidence::new(0.666),
        super::Confidence::new(1.0),
        &diagnostics,
    );
    let result = synthetic_tempo_structure_result(
        diagnostics,
        interpretation,
        super::Confidence::new(0.666),
        super::Confidence::new(1.0),
    );
    let summary = result.tempo_structure_summary();

    assert_eq!(
        summary.stability_scope.scope,
        super::TempoStabilityScope::StableWithLocalizedEdgeDamage
    );
    assert_eq!(summary.continuity.action, super::TempoStateAction::Lock);
    assert_eq!(summary.selected_bpm, Some(128.0));
    assert!(summary
        .segments
        .iter()
        .any(|segment| segment.kind == super::TempoSegmentKind::WholeTrack));
    assert!(summary
        .segments
        .iter()
        .any(|segment| segment.kind == super::TempoSegmentKind::EdgeTrimmedStable));
    assert!(summary
        .segments
        .iter()
        .any(|segment| segment.kind == super::TempoSegmentKind::StableCore));
    let edge_trimmed = summary
        .segments
        .iter()
        .find(|segment| segment.kind == super::TempoSegmentKind::EdgeTrimmedStable)
        .unwrap();
    let stable_core = summary
        .segments
        .iter()
        .find(|segment| segment.kind == super::TempoSegmentKind::StableCore)
        .unwrap();
    assert!(edge_trimmed.coverage.0 > stable_core.coverage.0);
    assert!(edge_trimmed.end_beat_index > stable_core.end_beat_index);
}

#[test]
fn beat_tracker_exposes_tempo_structure_summary_for_core_stable_monitoring() {
    let result = analyze_fixture(&click_track(48_000, 90.0, 8.0));
    let summary = result.tempo_structure_summary();

    assert_eq!(
        summary.stability_scope.scope,
        super::TempoStabilityScope::CoreStableOnly
    );
    assert_eq!(summary.continuity.action, super::TempoStateAction::Monitor);
    assert_eq!(
        summary.continuity.continuity_action,
        super::TempoContinuityAction::Reacquire
    );
    assert_eq!(
        summary.continuity.current.source,
        super::TempoConsumptionSource::SnappedCurrentTempo
    );
    assert_eq!(
        summary.continuity.fallback.source,
        super::TempoConsumptionSource::NoTempo
    );
    assert_eq!(summary.continuity.fallback_after_beats, 8);
    assert!(!summary.segments.is_empty());
    assert!(summary
        .segments
        .iter()
        .any(|segment| segment.coverage.0 >= 0.5));
}

#[test]
fn tempo_structure_summary_surfaces_mid_track_unstable_clear_policy() {
    let diagnostics = synthetic_tempo_diagnostics(89.9, 0.42, 0.61, 0.38, 58.0, 44.0, 360.0, 92.0);
    let interpretation = synthetic_tempo_interpretation(
        super::TempoRecommendation::Defer,
        super::TempoTrustLevel::Tentative,
        super::TempoInterpretationReason::UnstableTempo,
        89.9,
        None,
        0.38,
        0.03,
        0.8,
        0.3,
    );
    let result = synthetic_tempo_structure_result(
        diagnostics,
        interpretation,
        super::Confidence::new(0.42),
        super::Confidence::new(0.55),
    );
    let summary = result.tempo_structure_summary();

    assert_eq!(
        summary.stability_scope.scope,
        super::TempoStabilityScope::MidTrackUnstable
    );
    assert_eq!(summary.continuity.action, super::TempoStateAction::Defer);
    assert_eq!(
        summary.continuity.continuity_action,
        super::TempoContinuityAction::Clear
    );
    assert_eq!(
        summary.continuity.current.source,
        super::TempoConsumptionSource::NoTempo
    );
    assert_eq!(
        summary.continuity.fallback.source,
        super::TempoConsumptionSource::NoTempo
    );
    assert_eq!(summary.segments.len(), 1);
    assert_eq!(
        summary.segments[0].kind,
        super::TempoSegmentKind::WholeTrack
    );
    assert!((summary.segments[0].representative_bpm - 89.9).abs() < 0.2);
}
