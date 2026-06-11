use super::*;

#[test]
fn beat_tracker_detects_click_track_tempo() {
    let audio = click_track(48_000, 120.0, 8.0);
    let mut tracker = BeatTracker::new(BeatTrackerConfig::default());
    let result = tracker.analyze(&audio);

    assert!(
        (result.bpm - 120.0).abs() < 3.0,
        "detected bpm {}",
        result.bpm
    );
    assert!(
        result.confidence.0 > 0.2,
        "confidence {}",
        result.confidence.0
    );
    assert!(result.beat_positions_seconds.len() >= 6);
    assert!(result.meter.is_none());
}

#[test]
fn beat_tracker_detects_slower_click_track_tempo() {
    let audio = click_track(48_000, 90.0, 8.0);
    let mut tracker = BeatTracker::new(BeatTrackerConfig::default());
    let result = tracker.analyze(&audio);

    assert!(
        (result.bpm - 90.0).abs() < 3.0,
        "detected bpm {}",
        result.bpm
    );
    assert!(
        result.confidence.0 > 0.15,
        "confidence {}",
        result.confidence.0
    );
}

#[test]
fn beat_tracker_refines_integer_click_track_tempo_to_sub_tenth_bpm() {
    let mut tracker = BeatTracker::new(BeatTrackerConfig::default());

    let fast = tracker.analyze(&click_track(48_000, 120.0, 8.0));
    assert!(
        (fast.bpm - 120.0).abs() < 0.1,
        "refined detected bpm {}",
        fast.bpm
    );
    assert!(
        fast.tempo_candidates
            .first()
            .map(|candidate| (candidate.bpm - 120.0).abs() < 0.1)
            .unwrap_or(false),
        "top tempo candidate {:?}",
        fast.tempo_candidates.first()
    );

    let slow = tracker.analyze(&click_track(48_000, 90.0, 8.0));
    assert!(
        (slow.bpm - 90.0).abs() < 0.1,
        "refined detected bpm {}",
        slow.bpm
    );
    assert!(
        slow.tempo_candidates
            .first()
            .map(|candidate| (candidate.bpm - 90.0).abs() < 0.1)
            .unwrap_or(false),
        "top tempo candidate {:?}",
        slow.tempo_candidates.first()
    );
}

#[test]
fn beat_tracker_exposes_stable_local_tempo_for_integer_click_track() {
    let mut tracker = BeatTracker::new(BeatTrackerConfig::default());
    let result = tracker.analyze(&click_track(48_000, 120.0, 8.0));

    assert!(result.tempo_diagnostics.interval_tempi.len() >= 10);
    assert!(result.tempo_diagnostics.windowed_tempi.len() >= 6);
    assert!(
        (result.tempo_diagnostics.median_bpm - 120.0).abs() < 0.15,
        "median local tempo {}",
        result.tempo_diagnostics.median_bpm
    );
    assert!(
        result.tempo_diagnostics.mean_abs_deviation_bpm < 0.15,
        "local tempo MAD {}",
        result.tempo_diagnostics.mean_abs_deviation_bpm
    );
    assert!(
        result.tempo_diagnostics.windowed_mean_abs_deviation_bpm
            < result.tempo_diagnostics.mean_abs_deviation_bpm,
        "windowed MAD {} raw MAD {}",
        result.tempo_diagnostics.windowed_mean_abs_deviation_bpm,
        result.tempo_diagnostics.mean_abs_deviation_bpm
    );
    assert!(
        (result.tempo_diagnostics.core_windowed_median_bpm - 120.0).abs() < 0.15,
        "core windowed median {}",
        result.tempo_diagnostics.core_windowed_median_bpm
    );
    assert!(
        result
            .tempo_diagnostics
            .core_windowed_mean_abs_deviation_bpm
            < 0.15,
        "core windowed MAD {}",
        result
            .tempo_diagnostics
            .core_windowed_mean_abs_deviation_bpm
    );
    assert!(
        result.tempo_diagnostics.boundary_bias_bpm > 0.05,
        "boundary bias {}",
        result.tempo_diagnostics.boundary_bias_bpm
    );
    assert!(
        result.tempo_diagnostics.boundary_bias_bpm
            < result.tempo_diagnostics.windowed_drift_span_bpm,
        "boundary bias {} full windowed span {}",
        result.tempo_diagnostics.boundary_bias_bpm,
        result.tempo_diagnostics.windowed_drift_span_bpm
    );
    assert_eq!(
        result.tempo_diagnostics.trend.direction,
        super::TempoTrendDirection::Stable
    );
    assert!(
        result.tempo_diagnostics.trend.total_drift_bpm.abs() < 0.15,
        "tempo drift {}",
        result.tempo_diagnostics.trend.total_drift_bpm
    );
    assert_eq!(
        result.tempo_diagnostics.beat_grid_error.residuals.len(),
        result.beat_positions_seconds.len()
    );
    assert!(
        result
            .tempo_diagnostics
            .beat_grid_error
            .mean_abs_residual_ms
            < 6.0,
        "mean abs residual ms {}",
        result
            .tempo_diagnostics
            .beat_grid_error
            .mean_abs_residual_ms
    );
    assert_eq!(
        result.tempo_interpretation.recommendation,
        super::TempoRecommendation::SnapInteger
    );
    assert_eq!(
        result.tempo_interpretation.reason,
        super::TempoInterpretationReason::NearIntegerPulse
    );
    assert_eq!(result.tempo_interpretation.snapped_bpm, Some(120.0));
    assert!(result.tempo_interpretation.profile.snap_error_bpm < 0.12);
    assert!(result.tempo_interpretation.profile.stability_score.0 > 0.75);
}
