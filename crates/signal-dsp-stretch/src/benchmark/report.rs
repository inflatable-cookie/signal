use super::types::{
    StretchAcceptanceReport, StretchAcceptanceSeverity, StretchAcceptanceStatus,
    StretchBenchmarkComparisonOutcome, StretchMetric, StretchMetricAssessment, StretchMetricLimit,
    StretchMetricValue, StretchQualityPriority, StretchQualityWorkArea,
    StretchSyntheticBenchmarkComparison, StretchSyntheticBenchmarkComparisonReport,
};

/// Rank quality-tuning work from comparison evidence.
///
/// Only regressions and inconclusive rows become priorities. Lower-is-better
/// metric values are normalized by metric family so the result is useful for
/// ordering, not for acceptance.
pub fn prioritize_stretch_quality_work(
    report: &StretchSyntheticBenchmarkComparisonReport,
    limit: usize,
) -> Vec<StretchQualityPriority> {
    let mut priorities = report
        .comparisons
        .iter()
        .filter_map(priority_from_comparison)
        .collect::<Vec<_>>();
    priorities.sort_by(|left, right| {
        right
            .priority_score
            .total_cmp(&left.priority_score)
            .then_with(|| left.case_id.cmp(right.case_id))
            .then_with(|| format!("{:?}", left.metric).cmp(&format!("{:?}", right.metric)))
            .then_with(|| left.ratio.total_cmp(&right.ratio))
    });
    priorities.truncate(limit);
    priorities
}

/// Evaluate measured stretch metrics against upper-bound limits.
pub fn assess_stretch_metrics(
    measurements: &[StretchMetricValue],
    limits: &[StretchMetricLimit],
) -> StretchAcceptanceReport {
    let mut status = StretchAcceptanceStatus::Pass;
    let mut metrics = Vec::with_capacity(limits.len());
    for limit in limits {
        let value = measurements
            .iter()
            .find(|measurement| measurement.metric == limit.metric)
            .map(|measurement| measurement.value)
            .unwrap_or(f64::NAN);
        let metric_status = if value.is_finite() && value <= limit.max {
            StretchAcceptanceStatus::Pass
        } else {
            severity_to_stretch_status(limit.severity)
        };
        status = combine_stretch_status(status, metric_status);
        metrics.push(StretchMetricAssessment {
            metric: limit.metric,
            value,
            max: limit.max,
            status: metric_status,
        });
    }

    StretchAcceptanceReport { status, metrics }
}

/// Deterministic line-oriented report for one stretch acceptance result.
pub fn format_stretch_acceptance_report(case_id: &str, report: &StretchAcceptanceReport) -> String {
    let mut lines = Vec::with_capacity(report.metrics.len() + 1);
    lines.push(format!("case={case_id} status={:?}", report.status));
    for metric in &report.metrics {
        lines.push(format!(
            "metric={:?} value={:.6} max={:.6} status={:?}",
            metric.metric, metric.value, metric.max, metric.status
        ));
    }
    lines.join("\n")
}

/// Deterministic line-oriented report for synthetic baseline-vs-prototype
/// benchmark comparisons.
pub fn format_synthetic_stretch_comparison_report(
    report: &StretchSyntheticBenchmarkComparisonReport,
) -> String {
    let mut lines = Vec::with_capacity(report.comparisons.len() + 1);
    lines.push(format!(
        "synthetic_stretch_comparison improved={} regressed={} unchanged={} inconclusive={}",
        report.improved_count,
        report.regressed_count,
        report.unchanged_count,
        report.inconclusive_count
    ));
    for comparison in &report.comparisons {
        let pitch_shift = comparison
            .pitch_shift_semitones
            .map(|semitones| format!("{semitones:.6}"))
            .unwrap_or_else(|| "none".to_string());
        lines.push(format!(
            "case={} ratio={:.6} path={:?} pitch_shift={} metric={:?} baseline_backend={:?} candidate_backend={:?} baseline={:.6} candidate={:.6} delta={:.6} outcome={:?}",
            comparison.case_id,
            comparison.ratio,
            comparison.path,
            pitch_shift,
            comparison.metric,
            comparison.baseline_backend,
            comparison.candidate_backend,
            comparison.baseline_value,
            comparison.candidate_value,
            comparison.delta,
            comparison.outcome
        ));
    }
    lines.join("\n")
}

/// Deterministic line-oriented report for prioritized stretch quality work.
pub fn format_stretch_quality_priority_report(priorities: &[StretchQualityPriority]) -> String {
    let mut lines = Vec::with_capacity(priorities.len() + 1);
    lines.push(format!(
        "stretch_quality_priorities count={}",
        priorities.len()
    ));
    for priority in priorities {
        let pitch_shift = priority
            .pitch_shift_semitones
            .map(|semitones| format!("{semitones:.6}"))
            .unwrap_or_else(|| "none".to_string());
        lines.push(format!(
            "area={:?} case={} ratio={:.6} path={:?} pitch_shift={} metric={:?} baseline={:.6} candidate={:.6} delta={:.6} outcome={:?} score={:.6}",
            priority.area,
            priority.case_id,
            priority.ratio,
            priority.path,
            pitch_shift,
            priority.metric,
            priority.baseline_value,
            priority.candidate_value,
            priority.delta,
            priority.outcome,
            priority.priority_score
        ));
    }
    lines.join("\n")
}
fn priority_from_comparison(
    comparison: &StretchSyntheticBenchmarkComparison,
) -> Option<StretchQualityPriority> {
    let priority_score = match comparison.outcome {
        StretchBenchmarkComparisonOutcome::Regressed => {
            priority_score(comparison.metric, comparison.delta)
        }
        StretchBenchmarkComparisonOutcome::Inconclusive => 1.0e9,
        StretchBenchmarkComparisonOutcome::Improved
        | StretchBenchmarkComparisonOutcome::Unchanged => {
            return None;
        }
    };
    if !priority_score.is_finite() || priority_score <= 0.0 {
        return None;
    }

    Some(StretchQualityPriority {
        area: quality_work_area(comparison.metric),
        case_id: comparison.case_id,
        path: comparison.path,
        metric: comparison.metric,
        ratio: comparison.ratio,
        pitch_shift_semitones: comparison.pitch_shift_semitones,
        baseline_value: comparison.baseline_value,
        candidate_value: comparison.candidate_value,
        delta: comparison.delta,
        outcome: comparison.outcome,
        priority_score,
    })
}

fn quality_work_area(metric: StretchMetric) -> StretchQualityWorkArea {
    match metric {
        StretchMetric::TimingDriftSamples => StretchQualityWorkArea::TimingAlignment,
        StretchMetric::TransientSmearFrames => StretchQualityWorkArea::TransientPreservation,
        StretchMetric::VerticalCoherenceDelta => StretchQualityWorkArea::VerticalCoherence,
        StretchMetric::StereoImageDelta => StretchQualityWorkArea::StereoImageStability,
        StretchMetric::LoopBoundaryClickDbfs => StretchQualityWorkArea::LoopBoundaryClicks,
        StretchMetric::DynamicSegmentSeamClickDbfs => StretchQualityWorkArea::DynamicRatioSeams,
        StretchMetric::PitchErrorCents => StretchQualityWorkArea::PitchShiftAccuracy,
        StretchMetric::CpuRealtimeFactor
        | StretchMetric::LatencyFrames
        | StretchMetric::PeakMemoryBytes => StretchQualityWorkArea::ResourceBudget,
    }
}

fn priority_score(metric: StretchMetric, delta: f64) -> f64 {
    if !delta.is_finite() || delta <= 0.0 {
        return f64::NAN;
    }

    match metric {
        StretchMetric::LoopBoundaryClickDbfs | StretchMetric::DynamicSegmentSeamClickDbfs => {
            delta / 6.0
        }
        StretchMetric::StereoImageDelta | StretchMetric::VerticalCoherenceDelta => delta * 10.0,
        StretchMetric::PitchErrorCents => delta / 10.0,
        _ => delta,
    }
}
fn severity_to_stretch_status(severity: StretchAcceptanceSeverity) -> StretchAcceptanceStatus {
    match severity {
        StretchAcceptanceSeverity::Warn => StretchAcceptanceStatus::Warn,
        StretchAcceptanceSeverity::Fail => StretchAcceptanceStatus::Fail,
    }
}

fn combine_stretch_status(
    left: StretchAcceptanceStatus,
    right: StretchAcceptanceStatus,
) -> StretchAcceptanceStatus {
    match (left, right) {
        (StretchAcceptanceStatus::Fail, _) | (_, StretchAcceptanceStatus::Fail) => {
            StretchAcceptanceStatus::Fail
        }
        (StretchAcceptanceStatus::Warn, _) | (_, StretchAcceptanceStatus::Warn) => {
            StretchAcceptanceStatus::Warn
        }
        _ => StretchAcceptanceStatus::Pass,
    }
}
