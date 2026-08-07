use super::*;

#[test]
fn metric_assessment_aggregates_warnings_and_failures() {
    let measurements = [
        StretchMetricValue::new(StretchMetric::TimingDriftSamples, 0.0),
        StretchMetricValue::new(StretchMetric::StereoImageDelta, 0.2),
        StretchMetricValue::new(StretchMetric::LoopBoundaryClickDbfs, -24.0),
    ];
    let limits = [
        StretchMetricLimit::max(
            StretchMetric::TimingDriftSamples,
            1.0,
            StretchAcceptanceSeverity::Fail,
        ),
        StretchMetricLimit::max(
            StretchMetric::StereoImageDelta,
            0.1,
            StretchAcceptanceSeverity::Warn,
        ),
        StretchMetricLimit::max(
            StretchMetric::LoopBoundaryClickDbfs,
            -60.0,
            StretchAcceptanceSeverity::Fail,
        ),
    ];

    let report = assess_stretch_metrics(&measurements, &limits);

    assert_eq!(report.status, StretchAcceptanceStatus::Fail);
    assert_eq!(report.metrics[0].status, StretchAcceptanceStatus::Pass);
    assert_eq!(report.metrics[1].status, StretchAcceptanceStatus::Warn);
    assert_eq!(report.metrics[2].status, StretchAcceptanceStatus::Fail);
}
#[test]
fn acceptance_report_format_is_deterministic() {
    let report = assess_stretch_metrics(
        &[StretchMetricValue::new(
            StretchMetric::TimingDriftSamples,
            0.0,
        )],
        &[StretchMetricLimit::max(
            StretchMetric::TimingDriftSamples,
            1.0,
            StretchAcceptanceSeverity::Fail,
        )],
    );

    assert_eq!(
        format_stretch_acceptance_report("stretch:tempo_ramp", &report),
        "case=stretch:tempo_ramp status=Pass\nmetric=TimingDriftSamples value=0.000000 max=1.000000 status=Pass"
    );
}
