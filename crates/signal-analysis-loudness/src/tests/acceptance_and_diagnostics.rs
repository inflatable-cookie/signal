use super::*;
use signal_analysis::{AcceptanceStatus, AnalysisStage};

#[test]
fn harness_loudness_cases_meet_frozen_acceptance_thresholds() {
    let cases = loudness_acceptance_cases();
    let mut meter = LoudnessMeter::new(LoudnessMeterConfig::default());

    let report =
        run_audio_acceptance_harness(&cases, |audio| meter.analyze(audio), loudness_metrics);

    assert_eq!(report.status, AcceptanceStatus::Pass);
    assert!(report
        .cases
        .iter()
        .all(|case| case.status == AcceptanceStatus::Pass));
}

#[test]
fn frozen_loudness_acceptance_report_remains_interpretable_for_closeout() {
    let cases = loudness_acceptance_cases();
    let mut meter = LoudnessMeter::new(LoudnessMeterConfig::default());

    let report =
        run_audio_acceptance_harness(&cases, |audio| meter.analyze(audio), loudness_metrics);

    println!("loudness_acceptance_report={:#?}", report);

    assert_eq!(report.status, AcceptanceStatus::Pass);
    assert_eq!(report.cases.len(), 3);
}

#[test]
fn loudness_traces_capture_level_step_and_dynamics_summary() {
    let audio = sine_sequence(48_000, &[(440.0, 0.08, 4.0), (440.0, 0.35, 4.0)]);
    let mut meter = LoudnessMeter::new(LoudnessMeterConfig::default());
    let result = meter.analyze(&audio);

    assert!(result.momentary_trace.points.len() > result.short_term_trace.points.len());
    assert!(result.momentary_trace.points.len() > 10);
    assert!(result.short_term_trace.points.len() >= 2);
    assert!(result.dynamics.momentary_max_lufs >= result.integrated_lufs);
    assert!(result.dynamics.short_term_max_lufs >= result.integrated_lufs);
    assert!(result.dynamics.momentary_range_lu > 0.0);
    assert!(result.dynamics.short_term_range_lu > 0.0);
    assert!(result.dynamics.target_offset_lu.is_finite());

    let loudest_momentary = result
        .momentary_trace
        .points
        .iter()
        .max_by(|lhs, rhs| {
            lhs.loudness_lufs
                .partial_cmp(&rhs.loudness_lufs)
                .unwrap_or(core::cmp::Ordering::Equal)
        })
        .expect("loudest momentary point");
    assert!(loudest_momentary.start_seconds >= 3.0);
}

#[test]
fn runtime_diagnostics_summary_uses_bounded_recent_trace_tails() {
    let audio = sine_sequence(
        48_000,
        &[(440.0, 0.05, 3.0), (440.0, 0.2, 3.0), (440.0, 0.35, 3.0)],
    );
    let mut meter = LoudnessMeter::new(LoudnessMeterConfig::default());
    let result = meter.analyze(&audio);
    let diagnostics = result.runtime_diagnostics_summary();

    assert!(diagnostics.recent_momentary.points.len() <= RUNTIME_MOMENTARY_TAIL_POINTS);
    assert!(diagnostics.recent_short_term.points.len() <= RUNTIME_SHORT_TERM_TAIL_POINTS);
    assert_eq!(
        diagnostics.current_momentary_lufs,
        diagnostics
            .recent_momentary
            .points
            .last()
            .expect("recent momentary point")
            .loudness_lufs
    );
    assert_eq!(
        diagnostics.current_short_term_lufs,
        diagnostics
            .recent_short_term
            .points
            .last()
            .expect("recent short-term point")
            .loudness_lufs
    );
    assert_eq!(diagnostics.integrated_lufs, result.integrated_lufs);
    assert_eq!(diagnostics.true_peak_dbtp, result.true_peak_dbtp);
    assert_eq!(
        diagnostics.target_offset_lu,
        result.dynamics.target_offset_lu
    );
    assert_eq!(
        diagnostics.momentary_max_lufs,
        result.dynamics.momentary_max_lufs
    );
    assert_eq!(
        diagnostics.short_term_max_lufs,
        result.dynamics.short_term_max_lufs
    );
}
