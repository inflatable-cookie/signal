use super::*;

#[test]
fn loop_boundary_metric_reports_direct_discontinuity() {
    let frames = [0.1, -0.2, 0.3, 0.1];
    let measurement = measure_loop_boundary_click(&frames, 2, 1.0);

    assert_eq!(measurement.ratio, 1.0);
    assert_eq!(measurement.channels, 2);
    assert!((measurement.peak_boundary_delta - 0.3).abs() < 1.0e-6);
    assert!((measurement.click_dbfs - (20.0f64 * 0.3f64.log10())).abs() < 1.0e-6);
    assert_eq!(
        measurement.metric.metric,
        StretchMetric::LoopBoundaryClickDbfs
    );
    assert_eq!(measurement.metric.value, measurement.click_dbfs);
}

#[test]
fn loop_boundary_smoothing_equalizes_endpoints() {
    let mut frames = [1.0, -0.5, 0.25, 0.25, -1.0, 0.75];

    benchmark::smooth_loop_boundary_interleaved(&mut frames, 2, 1);

    assert!((frames[0] - frames[4]).abs() < 1.0e-6);
    assert!((frames[1] - frames[5]).abs() < 1.0e-6);
    assert!((frames[2] - 0.25).abs() < 1.0e-6);
    assert!((frames[3] - 0.25).abs() < 1.0e-6);
}
#[test]
fn loop_boundary_metric_reports_synthetic_draft_case() {
    let measurement = measure_draft_loop_boundary_click(1.25);

    assert_eq!(measurement.ratio, 1.25);
    assert_eq!(measurement.channels, 2);
    assert!(measurement.peak_boundary_delta.is_finite());
    assert!(measurement.click_dbfs.is_finite());
    assert_eq!(
        measurement.metric.metric,
        StretchMetric::LoopBoundaryClickDbfs
    );
}

#[test]
fn transient_reset_loop_boundary_metric_reports_synthetic_case() {
    let measurement = measure_transient_reset_loop_boundary_click(1.25);

    assert_eq!(measurement.ratio, 1.25);
    assert_eq!(measurement.channels, 2);
    assert!(measurement.peak_boundary_delta.is_finite());
    assert!(measurement.click_dbfs.is_finite());
    assert_eq!(
        measurement.metric.metric,
        StretchMetric::LoopBoundaryClickDbfs
    );
}

#[test]
fn loop_boundary_metric_formats_as_acceptance_metric() {
    let measurement = measure_draft_loop_boundary_click(1.5);
    let report = assess_stretch_metrics(
        &[measurement.metric],
        &[StretchMetricLimit::max(
            StretchMetric::LoopBoundaryClickDbfs,
            f64::INFINITY,
            StretchAcceptanceSeverity::Warn,
        )],
    );
    let formatted = format_stretch_acceptance_report("stretch:loop_seam", &report);

    assert_eq!(report.status, StretchAcceptanceStatus::Pass);
    assert!(formatted.contains("metric=LoopBoundaryClickDbfs"));
    assert!(formatted.contains("status=Pass"));
}
