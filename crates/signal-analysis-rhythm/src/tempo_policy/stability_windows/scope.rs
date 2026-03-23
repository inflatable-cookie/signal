use signal_analysis::Confidence;

use crate::tempo_policy::{
    BeatGridCoreSpanDiagnostics, BeatIntervalOutlierDiagnostics, TempoStabilityScope,
    TempoStabilityScopeSummary, TempoStabilityScopeSupport,
};

pub(crate) fn classify_tempo_stability_scope(
    window_count: usize,
    beat_interval_outliers: &BeatIntervalOutlierDiagnostics,
    edge_trimmed_stable_span: Option<BeatGridCoreSpanDiagnostics>,
    stable_core_span: Option<BeatGridCoreSpanDiagnostics>,
) -> TempoStabilityScopeSummary {
    let edge_trimmed_coverage = edge_trimmed_stable_span
        .map(|span| span.coverage)
        .unwrap_or_else(|| Confidence::new(0.0));
    let contiguous_core_coverage = stable_core_span
        .map(|span| span.coverage)
        .unwrap_or_else(|| Confidence::new(0.0));
    let interior_stability = edge_trimmed_stable_span
        .map(|span| {
            if span.retained_windows == 0 {
                0.0
            } else {
                1.0 - (span.interior_rejected_windows as f32 / span.retained_windows as f32)
            }
        })
        .unwrap_or(0.0);
    let edge_locality = edge_trimmed_stable_span
        .map(|span| {
            let trimmed_fraction = if span.total_windows == 0 {
                0.0
            } else {
                (span.trimmed_leading_windows + span.trimmed_trailing_windows) as f32
                    / span.total_windows as f32
            };
            let edge_outlier_signal = if beat_interval_outliers.trailing_rejected_intervals
                + beat_interval_outliers.leading_rejected_intervals
                >= 2
            {
                if beat_interval_outliers.trailing_rejected_intervals == 0
                    || beat_interval_outliers.leading_rejected_intervals == 0
                {
                    1.0
                } else {
                    0.8
                }
            } else if span.trimmed_leading_windows + span.trimmed_trailing_windows > 0 {
                0.7
            } else {
                0.0
            };
            let span_gain = (span.coverage.0 - contiguous_core_coverage.0).clamp(0.0, 1.0);
            let trimmed_support = if span_gain > 0.0 {
                (1.0 - (trimmed_fraction / 0.08)).clamp(0.0, 1.0)
            } else {
                0.0
            };
            (0.50 * trimmed_support
                + 0.30 * edge_outlier_signal
                + 0.20 * interior_stability.clamp(0.0, 1.0))
            .clamp(0.0, 1.0)
        })
        .unwrap_or(0.0);
    let support = TempoStabilityScopeSupport {
        edge_trimmed_coverage,
        contiguous_core_coverage,
        interior_stability: Confidence::new(interior_stability),
        edge_locality: Confidence::new(edge_locality),
    };
    let scope = if edge_trimmed_coverage.0 >= 0.97
        && support.interior_stability.0 >= 0.98
        && support.edge_locality.0 < 0.35
    {
        TempoStabilityScope::WholeTrackStable
    } else if edge_trimmed_coverage.0 >= 0.90
        && support.interior_stability.0 >= 0.95
        && support.edge_locality.0 >= 0.55
    {
        TempoStabilityScope::StableWithLocalizedEdgeDamage
    } else if contiguous_core_coverage.0 >= 0.50
        && (edge_trimmed_coverage.0 >= contiguous_core_coverage.0
            || window_count
                >= stable_core_span
                    .map(|span| span.retained_windows)
                    .unwrap_or(0))
    {
        TempoStabilityScope::CoreStableOnly
    } else {
        TempoStabilityScope::MidTrackUnstable
    };

    TempoStabilityScopeSummary { scope, support }
}
