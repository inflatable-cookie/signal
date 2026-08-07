use super::super::*;

#[test]
fn synthetic_corpus_cases_run_without_file_io() {
    let cases = synthetic_stretch_corpus_cases();
    assert_eq!(cases.len(), 3);
    for (case, audio) in cases {
        assert_eq!(case.source, StretchCorpusSource::Synthetic);
        assert!(audio.sample_rate_hz > 0);
        assert!(audio.channels > 0);
        assert_eq!(audio.samples.len() % audio.channels as usize, 0);
        assert!(audio.samples.iter().any(|sample| sample.abs() > 0.01));
    }
}
#[test]
fn synthetic_backend_comparison_covers_all_synthetic_cases() {
    let report = compare_synthetic_stretch_backends();

    assert_eq!(report.comparisons.len(), 27);
    assert_eq!(
        report.improved_count
            + report.regressed_count
            + report.unchanged_count
            + report.inconclusive_count,
        report.comparisons.len()
    );
    for comparison in &report.comparisons {
        assert_eq!(comparison.baseline_backend, StretchBenchmarkBackend::Draft);
        assert_eq!(
            comparison.candidate_backend,
            StretchBenchmarkBackend::OfflineHighQualityPrototype
        );
        assert!(comparison.ratio.is_finite());
        assert!(comparison.ratio > 0.0);
        assert!(matches!(
            comparison.case_id,
            "stretch:tempo_ramp"
                | "stretch:loop_seam"
                | "stretch:extreme_ratio"
                | "stretch:pitch_shift"
                | "stretch:sustained_coherence"
        ));
    }
    assert!(report.comparisons.iter().any(|comparison| {
        comparison.case_id == "stretch:tempo_ramp"
            && comparison.metric == StretchMetric::TimingDriftSamples
            && comparison.ratio > 1.0
            && comparison.path == StretchBenchmarkPath::DynamicRatio
    }));
    assert!(report.comparisons.iter().any(|comparison| {
        comparison.case_id == "stretch:tempo_ramp"
            && comparison.metric == StretchMetric::DynamicSegmentSeamClickDbfs
            && comparison.ratio > 1.0
            && comparison.path == StretchBenchmarkPath::DynamicRatio
    }));
    assert!(report.comparisons.iter().any(|comparison| {
        comparison.case_id == "stretch:loop_seam"
            && comparison.metric == StretchMetric::LoopBoundaryClickDbfs
            && comparison.path == StretchBenchmarkPath::FixedRatio
    }));
    assert!(report.comparisons.iter().any(|comparison| {
        comparison.case_id == "stretch:loop_seam"
            && comparison.metric == StretchMetric::StereoImageDelta
            && comparison.path == StretchBenchmarkPath::LinkedStereo
    }));
    assert!(report.comparisons.iter().any(|comparison| {
        comparison.case_id == "stretch:extreme_ratio"
            && comparison.metric == StretchMetric::TransientSmearFrames
            && comparison.path == StretchBenchmarkPath::FixedRatio
    }));
    let expanded_transient = report
        .comparisons
        .iter()
        .find(|comparison| {
            comparison.case_id == "stretch:extreme_ratio"
                && comparison.metric == StretchMetric::TransientSmearFrames
                && comparison.path == StretchBenchmarkPath::FixedRatio
                && comparison.ratio == 2.0
        })
        .expect("2x transient-smear comparison should remain covered");
    assert!(expanded_transient.baseline_value.is_finite());
    assert!(expanded_transient.candidate_value.is_finite());
    assert!(expanded_transient.delta.is_finite());
    assert_ne!(
        expanded_transient.outcome,
        StretchBenchmarkComparisonOutcome::Inconclusive
    );
    assert!(report.comparisons.iter().any(|comparison| {
        comparison.case_id == "stretch:pitch_shift"
            && comparison.metric == StretchMetric::PitchErrorCents
            && comparison.path == StretchBenchmarkPath::PitchShift
            && comparison.pitch_shift_semitones == Some(12.0)
    }));
    assert!(report.comparisons.iter().any(|comparison| {
        comparison.case_id == "stretch:sustained_coherence"
            && comparison.metric == StretchMetric::VerticalCoherenceDelta
            && comparison.path == StretchBenchmarkPath::PhaseLocked
    }));
}
