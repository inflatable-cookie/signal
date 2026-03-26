use super::*;

#[test]
fn beat_interval_outlier_filter_localizes_terminal_outliers() {
    let stable = 60.0 / 128.0;
    let intervals = vec![
        stable,
        stable,
        stable,
        stable,
        stable,
        stable,
        stable,
        stable,
        stable,
        stable,
        stable,
        stable,
        stable * 1.23,
        stable * 0.84,
        stable * 1.32,
        stable,
    ];
    let (retained, diagnostics) = super::filter_interval_outliers(&intervals);

    assert_eq!(diagnostics.total_intervals, intervals.len());
    assert_eq!(diagnostics.trailing_rejected_intervals, 3);
    assert_eq!(diagnostics.rejected_intervals, 3);
    assert_eq!(diagnostics.leading_rejected_intervals, 0);
    assert_eq!(diagnostics.retained_intervals, retained.len());
    assert!(diagnostics.max_rejected_deviation_ratio > 0.2);
    assert!((diagnostics.median_interval - stable).abs() < 1.0e-6);
}

#[test]
fn stable_core_span_detects_terminal_window_damage() {
    let stable = 127.94;
    let points: Vec<super::LocalTempoPoint> = (0..12)
        .map(|index| super::LocalTempoPoint {
            start_beat_index: index,
            end_beat_index: index + 4,
            start_seconds: index as f32,
            end_seconds: index as f32 + 4.0,
            bpm: match index {
                9 => 129.10,
                10 => 124.40,
                11 => 116.60,
                _ => stable,
            },
        })
        .collect();

    let span = super::detect_stable_core_span(&points, stable, 0.12).unwrap();

    assert_eq!(span.start_beat_index, 0);
    assert_eq!(span.end_beat_index, 12);
    assert!(span.coverage.0 >= 0.8, "coverage {}", span.coverage.0);
    assert_eq!(span.trimmed_leading_windows, 0);
    assert_eq!(span.trimmed_trailing_windows, 3);
    assert_eq!(span.interior_rejected_windows, 0);
}

#[test]
fn edge_trimmed_stable_span_preserves_sparse_interior_instability() {
    let stable = 127.94;
    let points: Vec<super::LocalTempoPoint> = (0..16)
        .map(|index| super::LocalTempoPoint {
            start_beat_index: index,
            end_beat_index: index + 4,
            start_seconds: index as f32,
            end_seconds: index as f32 + 4.0,
            bpm: match index {
                3 => 130.25,
                8 => 125.70,
                13 => 129.40,
                14 => 124.40,
                15 => 116.60,
                _ => stable,
            },
        })
        .collect();

    let edge_trimmed = super::detect_edge_trimmed_stable_span(&points, stable, 0.12).unwrap();
    let contiguous = super::detect_stable_core_span(&points, stable, 0.12).unwrap();

    assert_eq!(edge_trimmed.start_beat_index, 0);
    assert!(edge_trimmed.end_beat_index >= 16);
    assert_eq!(edge_trimmed.trimmed_leading_windows, 0);
    assert!(edge_trimmed.retained_windows >= contiguous.retained_windows);
    assert!(contiguous.trimmed_leading_windows > 0 || contiguous.trimmed_trailing_windows > 0);
}

#[test]
fn beat_tracker_exposes_stable_core_span_for_integer_click_track() {
    let tracker = &mut super::BeatTracker::new(super::BeatTrackerConfig::default());
    let result = tracker.analyze(&click_track(48_000, 120.0, 8.0));
    let edge_trimmed = result
        .tempo_diagnostics
        .edge_trimmed_stable_span
        .expect("edge-trimmed stable span");
    let span = result
        .tempo_diagnostics
        .stable_core_span
        .expect("stable core span");

    assert_eq!(edge_trimmed.start_beat_index, 0);
    assert!(
        edge_trimmed.coverage.0 > 0.95,
        "coverage {}",
        edge_trimmed.coverage.0
    );
    assert_eq!(edge_trimmed.interior_rejected_windows, 0);
    assert_eq!(span.start_beat_index, 0);
    assert!(span.end_beat_index >= result.beat_positions_seconds.len().saturating_sub(2));
    assert!(span.coverage.0 > 0.9, "coverage {}", span.coverage.0);
    assert_eq!(span.interior_rejected_windows, 0);
}

#[test]
fn beat_tracker_classifies_whole_track_stable_scope_for_click_track() {
    let tracker = &mut super::BeatTracker::new(super::BeatTrackerConfig::default());
    let result = tracker.analyze(&click_track(48_000, 120.0, 8.0));

    assert_eq!(
        result.tempo_diagnostics.stability_scope.scope,
        super::TempoStabilityScope::WholeTrackStable
    );
    assert!(
        result
            .tempo_consumption(None)
            .stability_scope
            .support
            .edge_trimmed_coverage
            .0
            > 0.95
    );
}

#[test]
fn classify_tempo_stability_scope_detects_localized_edge_damage() {
    let mut diagnostics = synthetic_tempo_diagnostics_with_counts(
        128.0, 0.70, -0.11, 0.48, 46.0, 45.8, 278.0, 87.0, 738, 735, 739,
    );
    diagnostics.beat_interval_outliers = super::BeatIntervalOutlierDiagnostics {
        total_intervals: 738,
        retained_intervals: 670,
        rejected_intervals: 68,
        leading_rejected_intervals: 0,
        trailing_rejected_intervals: 3,
        median_interval: 0.468_956,
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

    assert_eq!(
        diagnostics.stability_scope.scope,
        super::TempoStabilityScope::StableWithLocalizedEdgeDamage
    );
    assert!(diagnostics.stability_scope.support.edge_locality.0 >= 0.55);
}

#[test]
fn classify_tempo_stability_scope_detects_core_stable_only_case() {
    let mut diagnostics = synthetic_tempo_diagnostics_with_counts(
        120.0, 0.85, 0.42, 0.61, 58.0, 44.0, 360.0, 92.0, 128, 96, 128,
    );
    diagnostics.beat_interval_outliers = super::BeatIntervalOutlierDiagnostics {
        total_intervals: 128,
        retained_intervals: 120,
        rejected_intervals: 8,
        leading_rejected_intervals: 0,
        trailing_rejected_intervals: 0,
        median_interval: 0.5,
        median_abs_deviation: 0.004,
        max_rejected_deviation_ratio: 0.18,
    };
    diagnostics.edge_trimmed_stable_span = Some(super::BeatGridCoreSpanDiagnostics {
        start_beat_index: 24,
        end_beat_index: 92,
        start_seconds: 12.0,
        end_seconds: 46.0,
        coverage: super::Confidence::new(0.57),
        retained_windows: 69,
        total_windows: 96,
        trimmed_leading_windows: 24,
        trimmed_trailing_windows: 3,
        interior_rejected_windows: 6,
    });
    diagnostics.stable_core_span = Some(super::BeatGridCoreSpanDiagnostics {
        start_beat_index: 28,
        end_beat_index: 88,
        start_seconds: 14.0,
        end_seconds: 44.0,
        coverage: super::Confidence::new(0.50),
        retained_windows: 61,
        total_windows: 96,
        trimmed_leading_windows: 28,
        trimmed_trailing_windows: 7,
        interior_rejected_windows: 0,
    });
    diagnostics.stability_scope = super::classify_tempo_stability_scope(
        diagnostics.windowed_tempi.len(),
        &diagnostics.beat_interval_outliers,
        diagnostics.edge_trimmed_stable_span,
        diagnostics.stable_core_span,
    );

    assert_eq!(
        diagnostics.stability_scope.scope,
        super::TempoStabilityScope::CoreStableOnly
    );
    assert!(
        diagnostics
            .stability_scope
            .support
            .contiguous_core_coverage
            .0
            >= 0.5
    );
}
