use super::*;

#[test]
fn stereo_image_metric_reports_direct_movement() {
    let input = [0.5, 0.5, 0.25, 0.25, -0.25, -0.25, -0.5, -0.5];
    let output = [0.5, -0.5, 0.25, -0.25, -0.25, 0.25, -0.5, 0.5];
    let measurement = measure_stereo_image_delta(&input, &output, 1.0);

    assert_eq!(measurement.ratio, 1.0);
    assert!(measurement.input_correlation > 0.99);
    assert!(measurement.output_correlation < -0.99);
    assert!(measurement.image_delta > 1.0);
    assert_eq!(measurement.metric.metric, StretchMetric::StereoImageDelta);
    assert_eq!(measurement.metric.value, measurement.image_delta);
}

#[test]
fn stereo_image_metric_reports_synthetic_draft_case() {
    let measurement = measure_draft_stereo_image_delta(1.25);

    assert_eq!(measurement.ratio, 1.25);
    assert!(measurement.input_correlation.is_finite());
    assert!(measurement.output_correlation.is_finite());
    assert!(measurement.input_side_mid_ratio.is_finite());
    assert!(measurement.output_side_mid_ratio.is_finite());
    assert!(measurement.image_delta.is_finite());
    assert_eq!(measurement.metric.metric, StretchMetric::StereoImageDelta);
}

#[test]
fn transient_reset_stereo_image_metric_reports_synthetic_case() {
    let measurement = measure_transient_reset_stereo_image_delta(1.25);

    assert_eq!(measurement.ratio, 1.25);
    assert!(measurement.input_correlation.is_finite());
    assert!(measurement.output_correlation.is_finite());
    assert!(measurement.image_delta.is_finite());
    assert_eq!(measurement.metric.metric, StretchMetric::StereoImageDelta);
}

#[test]
fn stereo_image_metric_formats_as_acceptance_metric() {
    let measurement = measure_draft_stereo_image_delta(1.5);
    let report = assess_stretch_metrics(
        &[measurement.metric],
        &[StretchMetricLimit::max(
            StretchMetric::StereoImageDelta,
            f64::INFINITY,
            StretchAcceptanceSeverity::Warn,
        )],
    );
    let formatted = format_stretch_acceptance_report("stretch:full_mix", &report);

    assert_eq!(report.status, StretchAcceptanceStatus::Pass);
    assert!(formatted.contains("metric=StereoImageDelta"));
    assert!(formatted.contains("status=Pass"));
}
