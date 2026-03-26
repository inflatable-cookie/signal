//! Acceptance and regression harnesses for audio analysis validation.
//!
//! This module provides test harnesses for evaluating analyzer outputs
//! against acceptance thresholds and regression drift limits.

use crate::{
    AcceptanceCaseReport, AcceptanceHarnessReport, AcceptanceSeverity, AcceptanceStatus,
    AnalysisCorpusCase, AnalysisMetricValue, RegressionCaseReport, RegressionHarnessReport,
    RegressionMetricDelta,
};
use signal_primitives::AudioBuffer;
use std::time::Instant;

/// Evaluate analyzer outputs against per-case metric thresholds.
pub fn run_audio_acceptance_harness<Output, Analyze, Measure>(
    cases: &[AnalysisCorpusCase],
    mut analyze: Analyze,
    mut measure: Measure,
) -> AcceptanceHarnessReport
where
    Analyze: FnMut(&AudioBuffer) -> Output,
    Measure: FnMut(&Output) -> Vec<AnalysisMetricValue>,
{
    let mut overall_status = AcceptanceStatus::Pass;
    let mut reports = Vec::with_capacity(cases.len());

    for case in cases {
        let started = Instant::now();
        let output = analyze(&case.audio);
        let elapsed_ms = started.elapsed().as_secs_f32() * 1_000.0;
        let measured = measure(&output);
        let mut case_status = AcceptanceStatus::Pass;
        let mut metric_reports = Vec::with_capacity(case.acceptance_thresholds.len());

        for threshold in &case.acceptance_thresholds {
            let value = measured
                .iter()
                .find(|metric| metric.name == threshold.metric)
                .map(|metric| metric.value)
                .unwrap_or(f32::NAN);
            let passes_min = threshold.min.map(|min| value >= min).unwrap_or(true);
            let passes_max = threshold.max.map(|max| value <= max).unwrap_or(true);
            let status = if value.is_nan() || !(passes_min && passes_max) {
                severity_to_status(threshold.severity)
            } else {
                AcceptanceStatus::Pass
            };

            case_status = combine_status(case_status, status);
            metric_reports.push(AcceptanceMetricAssessment {
                metric: threshold.metric.clone(),
                value,
                min: threshold.min,
                max: threshold.max,
                status,
            });
        }

        overall_status = combine_status(overall_status, case_status);
        reports.push(AcceptanceCaseReport {
            case_id: case.metadata.case_id.clone(),
            status: case_status,
            elapsed_ms,
            metrics: metric_reports,
        });
    }

    AcceptanceHarnessReport {
        status: overall_status,
        cases: reports,
    }
}

/// Compare baseline and candidate analyzer outputs against per-case drift limits.
pub fn compare_audio_analyzers<
    BaselineOutput,
    CandidateOutput,
    AnalyzeBaseline,
    AnalyzeCandidate,
    MeasureBaseline,
    MeasureCandidate,
>(
    cases: &[AnalysisCorpusCase],
    mut baseline_analyze: AnalyzeBaseline,
    mut candidate_analyze: AnalyzeCandidate,
    mut baseline_measure: MeasureBaseline,
    mut candidate_measure: MeasureCandidate,
) -> RegressionHarnessReport
where
    AnalyzeBaseline: FnMut(&AudioBuffer) -> BaselineOutput,
    AnalyzeCandidate: FnMut(&AudioBuffer) -> CandidateOutput,
    MeasureBaseline: FnMut(&BaselineOutput) -> Vec<AnalysisMetricValue>,
    MeasureCandidate: FnMut(&CandidateOutput) -> Vec<AnalysisMetricValue>,
{
    let mut overall_status = AcceptanceStatus::Pass;
    let mut reports = Vec::with_capacity(cases.len());

    for case in cases {
        let baseline_started = Instant::now();
        let baseline_output = baseline_analyze(&case.audio);
        let baseline_elapsed_ms = baseline_started.elapsed().as_secs_f32() * 1_000.0;
        let candidate_started = Instant::now();
        let candidate_output = candidate_analyze(&case.audio);
        let candidate_elapsed_ms = candidate_started.elapsed().as_secs_f32() * 1_000.0;

        let baseline_metrics = baseline_measure(&baseline_output);
        let candidate_metrics = candidate_measure(&candidate_output);
        let mut case_status = AcceptanceStatus::Pass;
        let mut metric_reports = Vec::with_capacity(case.regression_limits.len());

        for limit in &case.regression_limits {
            let baseline = baseline_metrics
                .iter()
                .find(|metric| metric.name == limit.metric)
                .map(|metric| metric.value)
                .unwrap_or(f32::NAN);
            let candidate = candidate_metrics
                .iter()
                .find(|metric| metric.name == limit.metric)
                .map(|metric| metric.value)
                .unwrap_or(f32::NAN);
            let abs_delta = if baseline.is_nan() || candidate.is_nan() {
                f32::NAN
            } else {
                (candidate - baseline).abs()
            };
            let status = if abs_delta.is_nan() || abs_delta > limit.max_abs_delta {
                severity_to_status(limit.severity)
            } else {
                AcceptanceStatus::Pass
            };

            case_status = combine_status(case_status, status);
            metric_reports.push(RegressionMetricDelta {
                metric: limit.metric.clone(),
                baseline,
                candidate,
                abs_delta,
                max_abs_delta: limit.max_abs_delta,
                status,
            });
        }

        overall_status = combine_status(overall_status, case_status);
        reports.push(RegressionCaseReport {
            case_id: case.metadata.case_id.clone(),
            status: case_status,
            baseline_elapsed_ms,
            candidate_elapsed_ms,
            metrics: metric_reports,
        });
    }

    RegressionHarnessReport {
        status: overall_status,
        cases: reports,
    }
}

fn severity_to_status(severity: AcceptanceSeverity) -> AcceptanceStatus {
    match severity {
        AcceptanceSeverity::Warn => AcceptanceStatus::Warn,
        AcceptanceSeverity::Fail => AcceptanceStatus::Fail,
    }
}

fn combine_status(left: AcceptanceStatus, right: AcceptanceStatus) -> AcceptanceStatus {
    match (left, right) {
        (AcceptanceStatus::Fail, _) | (_, AcceptanceStatus::Fail) => AcceptanceStatus::Fail,
        (AcceptanceStatus::Warn, _) | (_, AcceptanceStatus::Warn) => AcceptanceStatus::Warn,
        _ => AcceptanceStatus::Pass,
    }
}

/// Evaluation of one metric against one acceptance threshold.
#[derive(Clone, Debug, PartialEq)]
pub struct AcceptanceMetricAssessment {
    pub metric: String,
    pub value: f32,
    pub min: Option<f32>,
    pub max: Option<f32>,
    pub status: AcceptanceStatus,
}
