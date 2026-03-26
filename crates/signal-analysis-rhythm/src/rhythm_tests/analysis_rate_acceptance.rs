use super::*;

#[test]
fn non_native_input_rate_preserves_click_track_tempo_under_frozen_analysis_rate() {
    let native = click_track(48_000, 120.0, 8.0);
    let non_native = click_track(44_100, 120.0, 8.0);
    let mut tracker = BeatTracker::new(BeatTrackerConfig::default());

    let native_result = tracker.analyze(&native);
    let non_native_result = tracker.analyze(&non_native);

    assert!((native_result.bpm - 120.0).abs() < 1.0);
    assert!((non_native_result.bpm - 120.0).abs() < 1.0);
    assert!((native_result.bpm - non_native_result.bpm).abs() < 0.5);
    assert!(
        (native_result.confidence.0 - non_native_result.confidence.0).abs() < 0.1,
        "confidence drifted from {} to {}",
        native_result.confidence.0,
        non_native_result.confidence.0,
    );
}

#[test]
fn harness_rhythm_cases_meet_frozen_acceptance_thresholds() {
    let cases = rhythm_acceptance_cases();
    let mut tracker = BeatTracker::new(BeatTrackerConfig::default());

    let report =
        run_audio_acceptance_harness(&cases, |audio| tracker.analyze(audio), rhythm_metrics);

    assert_eq!(report.status, AcceptanceStatus::Pass);
    assert!(report
        .cases
        .iter()
        .all(|case| case.status == AcceptanceStatus::Pass));
}

#[test]
fn frozen_rhythm_acceptance_report_remains_interpretable_for_closeout() {
    let cases = rhythm_acceptance_cases();
    let mut tracker = BeatTracker::new(BeatTrackerConfig::default());

    let report =
        run_audio_acceptance_harness(&cases, |audio| tracker.analyze(audio), rhythm_metrics);

    println!("rhythm_acceptance_report={:#?}", report);

    assert_eq!(report.status, AcceptanceStatus::Pass);
    assert_eq!(report.cases.len(), 3);
}
