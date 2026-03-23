use signal_analysis::Confidence;

use super::BeatAnalysisResult;
use crate::{
    tempo_summary, BeatGridCoreSpanDiagnostics, LocalTempoPoint, TempoSegmentKind,
    TempoSegmentSummary,
};

pub(crate) fn push_tempo_segment_summary(
    segments: &mut Vec<TempoSegmentSummary>,
    candidate: Option<TempoSegmentSummary>,
) {
    let Some(candidate) = candidate else {
        return;
    };
    if segments.iter().any(|existing| {
        existing.start_beat_index == candidate.start_beat_index
            && existing.end_beat_index == candidate.end_beat_index
            && (existing.start_seconds - candidate.start_seconds).abs() < 1.0e-6
            && (existing.end_seconds - candidate.end_seconds).abs() < 1.0e-6
    }) {
        return;
    }
    segments.push(candidate);
}

pub(crate) fn whole_track_tempo_segment_summary(
    result: &BeatAnalysisResult,
) -> Option<TempoSegmentSummary> {
    if let Some(segment) = tempo_segment_summary_from_points(
        TempoSegmentKind::WholeTrack,
        &result.tempo_diagnostics.windowed_tempi,
        Confidence::new(1.0),
        result.tempo_diagnostics.windowed_tempi.len(),
        result.tempo_diagnostics.windowed_tempi.len(),
    ) {
        return Some(segment);
    }

    if let Some(segment) = tempo_segment_summary_from_points(
        TempoSegmentKind::WholeTrack,
        &result.tempo_diagnostics.interval_tempi,
        Confidence::new(1.0),
        result.tempo_diagnostics.interval_tempi.len(),
        result.tempo_diagnostics.interval_tempi.len(),
    ) {
        return Some(segment);
    }

    let start_seconds = *result.beat_positions_seconds.first()?;
    let end_seconds = *result.beat_positions_seconds.last()?;
    if end_seconds <= start_seconds {
        return None;
    }

    Some(TempoSegmentSummary {
        kind: TempoSegmentKind::WholeTrack,
        start_beat_index: 0,
        end_beat_index: result.beat_positions_seconds.len().saturating_sub(1),
        start_seconds,
        end_seconds,
        representative_bpm: if result.tempo_diagnostics.core_windowed_median_bpm > 0.0 {
            result.tempo_diagnostics.core_windowed_median_bpm
        } else if result.tempo_diagnostics.windowed_median_bpm > 0.0 {
            result.tempo_diagnostics.windowed_median_bpm
        } else {
            result.bpm
        },
        drift_span_bpm: if result.tempo_diagnostics.windowed_drift_span_bpm > 0.0 {
            result.tempo_diagnostics.windowed_drift_span_bpm
        } else {
            result.tempo_diagnostics.drift_span_bpm
        },
        mean_abs_deviation_bpm: if result.tempo_diagnostics.windowed_mean_abs_deviation_bpm > 0.0 {
            result.tempo_diagnostics.windowed_mean_abs_deviation_bpm
        } else {
            result.tempo_diagnostics.mean_abs_deviation_bpm
        },
        coverage: Confidence::new(1.0),
        retained_windows: result.tempo_diagnostics.windowed_tempi.len(),
        total_windows: result.tempo_diagnostics.windowed_tempi.len(),
    })
}

pub(crate) fn stable_span_tempo_segment_summary(
    kind: TempoSegmentKind,
    points: &[LocalTempoPoint],
    span: Option<BeatGridCoreSpanDiagnostics>,
    fallback_bpm: f32,
    fallback_drift_span_bpm: f32,
    fallback_mean_abs_deviation_bpm: f32,
) -> Option<TempoSegmentSummary> {
    let span = span?;
    let subset: Vec<LocalTempoPoint> = points
        .iter()
        .filter(|point| {
            point.start_beat_index >= span.start_beat_index
                && point.end_beat_index <= span.end_beat_index
        })
        .cloned()
        .collect();
    let stats_points = if subset.is_empty() { points } else { &subset };
    let (representative_bpm, drift_span_bpm, mean_abs_deviation_bpm) = if stats_points.is_empty() {
        (
            fallback_bpm,
            fallback_drift_span_bpm,
            fallback_mean_abs_deviation_bpm,
        )
    } else {
        tempo_summary(stats_points)
    };

    Some(TempoSegmentSummary {
        kind,
        start_beat_index: span.start_beat_index,
        end_beat_index: span.end_beat_index,
        start_seconds: span.start_seconds,
        end_seconds: span.end_seconds,
        representative_bpm,
        drift_span_bpm,
        mean_abs_deviation_bpm,
        coverage: span.coverage,
        retained_windows: span.retained_windows,
        total_windows: span.total_windows,
    })
}

fn tempo_segment_summary_from_points(
    kind: TempoSegmentKind,
    points: &[LocalTempoPoint],
    coverage: Confidence,
    retained_windows: usize,
    total_windows: usize,
) -> Option<TempoSegmentSummary> {
    let first = points.first()?;
    let last = points.last()?;
    let (representative_bpm, drift_span_bpm, mean_abs_deviation_bpm) = tempo_summary(points);

    Some(TempoSegmentSummary {
        kind,
        start_beat_index: first.start_beat_index,
        end_beat_index: last.end_beat_index,
        start_seconds: first.start_seconds,
        end_seconds: last.end_seconds,
        representative_bpm,
        drift_span_bpm,
        mean_abs_deviation_bpm,
        coverage,
        retained_windows,
        total_windows,
    })
}
