use super::*;

#[test]
fn realtime_preview_backend_comparison_covers_preview_subset() {
    let report = compare_synthetic_realtime_preview_backends();

    assert_eq!(report.comparisons.len(), 24);
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
            StretchBenchmarkBackend::RealtimePreviewPrototype
        );
        assert!(comparison.ratio.is_finite());
        assert!(comparison.ratio > 0.0);
    }
    assert!(report.comparisons.iter().any(|comparison| {
        comparison.case_id == "stretch:tempo_ramp"
            && comparison.metric == StretchMetric::DynamicSegmentSeamClickDbfs
            && comparison.path == StretchBenchmarkPath::DynamicRatio
    }));
    assert!(report.comparisons.iter().any(|comparison| {
        comparison.case_id == "stretch:loop_seam"
            && comparison.metric == StretchMetric::StereoImageDelta
            && comparison.path == StretchBenchmarkPath::LinkedStereo
    }));
    assert!(report.comparisons.iter().any(|comparison| {
        comparison.case_id == "stretch:pitch_shift"
            && comparison.metric == StretchMetric::PitchErrorCents
            && comparison.path == StretchBenchmarkPath::PitchShift
            && comparison.pitch_shift_semitones == Some(12.0)
    }));
}
