use crate::tempo_policy::{
    classify_tempo_stability_scope, core_tempo_points, detect_edge_trimmed_stable_span,
    detect_stable_core_span, filter_interval_outliers, linear_fit, mean_abs, median, tempo_summary,
    BeatGridErrorDiagnostics, BeatGridResidualPoint, LocalTempoPoint, TempoDiagnostics,
    TempoTrendDiagnostics, TempoTrendDirection,
};

pub(crate) fn tempo_points(
    beat_positions_seconds: &[f32],
    beat_span: usize,
) -> Vec<LocalTempoPoint> {
    if beat_span == 0 || beat_positions_seconds.len() <= beat_span {
        return Vec::new();
    }

    let mut points = Vec::with_capacity(beat_positions_seconds.len() - beat_span);
    for start_beat in 0..(beat_positions_seconds.len() - beat_span) {
        let end_beat = start_beat + beat_span;
        let start_seconds = beat_positions_seconds[start_beat];
        let end_seconds = beat_positions_seconds[end_beat];
        let duration = end_seconds - start_seconds;
        if duration <= 0.0 {
            continue;
        }
        points.push(LocalTempoPoint {
            start_beat_index: start_beat,
            end_beat_index: end_beat,
            start_seconds,
            end_seconds,
            bpm: 60.0 * beat_span as f32 / duration,
        });
    }
    points
}

pub(crate) fn analyze_tempo_trend(points: &[LocalTempoPoint]) -> TempoTrendDiagnostics {
    let fit_points: Vec<(f32, f32)> = points
        .iter()
        .map(|point| (point.start_beat_index as f32, point.bpm))
        .collect();
    let Some((intercept, slope_bpm_per_beat)) = linear_fit(&fit_points) else {
        return TempoTrendDiagnostics {
            direction: TempoTrendDirection::Stable,
            start_bpm: points.first().map(|point| point.bpm).unwrap_or(0.0),
            end_bpm: points.last().map(|point| point.bpm).unwrap_or(0.0),
            total_drift_bpm: 0.0,
            slope_bpm_per_beat: 0.0,
            fit_mean_abs_deviation_bpm: 0.0,
        };
    };
    let start_x = fit_points.first().map(|(x, _)| *x).unwrap_or(0.0);
    let end_x = fit_points.last().map(|(x, _)| *x).unwrap_or(start_x);
    let start_bpm = intercept + slope_bpm_per_beat * start_x;
    let end_bpm = intercept + slope_bpm_per_beat * end_x;
    let total_drift_bpm = end_bpm - start_bpm;
    let fit_mean_abs_deviation_bpm = if fit_points.is_empty() {
        0.0
    } else {
        fit_points
            .iter()
            .map(|(x, bpm)| (bpm - (intercept + slope_bpm_per_beat * *x)).abs())
            .sum::<f32>()
            / fit_points.len() as f32
    };
    let direction = if total_drift_bpm.abs() < 0.15 {
        TempoTrendDirection::Stable
    } else if total_drift_bpm > 0.0 {
        TempoTrendDirection::Accelerating
    } else {
        TempoTrendDirection::Decelerating
    };

    TempoTrendDiagnostics {
        direction,
        start_bpm,
        end_bpm,
        total_drift_bpm,
        slope_bpm_per_beat,
        fit_mean_abs_deviation_bpm,
    }
}

pub(crate) fn analyze_beat_grid_error(beat_positions_seconds: &[f32]) -> BeatGridErrorDiagnostics {
    if beat_positions_seconds.is_empty() {
        return BeatGridErrorDiagnostics {
            residuals: Vec::new(),
            mean_abs_residual_ms: 0.0,
            max_abs_residual_ms: 0.0,
            edge_mean_abs_residual_ms: 0.0,
            core_mean_abs_residual_ms: 0.0,
            end_anchored_drift_ms: 0.0,
            mean_abs_anchored_drift_ms: 0.0,
        };
    }

    let fit_points: Vec<(f32, f32)> = beat_positions_seconds
        .iter()
        .enumerate()
        .map(|(beat_index, seconds)| (beat_index as f32, *seconds))
        .collect();
    let median_interval_seconds = {
        let mut intervals: Vec<f32> = beat_positions_seconds
            .windows(2)
            .map(|window| window[1] - window[0])
            .collect();
        median(&mut intervals)
    };
    let (intercept, slope_seconds_per_beat) =
        linear_fit(&fit_points).unwrap_or((beat_positions_seconds[0], median_interval_seconds));
    let anchor = beat_positions_seconds[0];
    let residuals: Vec<BeatGridResidualPoint> = beat_positions_seconds
        .iter()
        .enumerate()
        .map(|(beat_index, seconds)| {
            let beat = beat_index as f32;
            BeatGridResidualPoint {
                beat_index,
                seconds: *seconds,
                fitted_residual_ms: 1_000.0
                    * (*seconds - (intercept + slope_seconds_per_beat * beat)),
                anchored_drift_ms: 1_000.0 * (*seconds - (anchor + median_interval_seconds * beat)),
            }
        })
        .collect();
    let fitted_residuals: Vec<f32> = residuals
        .iter()
        .map(|point| point.fitted_residual_ms)
        .collect();
    let anchored_drifts: Vec<f32> = residuals
        .iter()
        .map(|point| point.anchored_drift_ms)
        .collect();
    let edge_count = residuals.len().min(2);
    let edge_values: Vec<f32> = residuals
        .iter()
        .enumerate()
        .filter(|(index, _)| *index < edge_count || *index + edge_count >= residuals.len())
        .map(|(_, point)| point.fitted_residual_ms)
        .collect();
    let core_values: Vec<f32> = if residuals.len() > edge_count * 2 {
        residuals[edge_count..residuals.len() - edge_count]
            .iter()
            .map(|point| point.fitted_residual_ms)
            .collect()
    } else {
        fitted_residuals.clone()
    };

    BeatGridErrorDiagnostics {
        residuals,
        mean_abs_residual_ms: mean_abs(&fitted_residuals),
        max_abs_residual_ms: fitted_residuals
            .iter()
            .map(|value| value.abs())
            .fold(0.0, f32::max),
        edge_mean_abs_residual_ms: mean_abs(&edge_values),
        core_mean_abs_residual_ms: mean_abs(&core_values),
        end_anchored_drift_ms: anchored_drifts.last().copied().unwrap_or(0.0),
        mean_abs_anchored_drift_ms: mean_abs(&anchored_drifts),
    }
}

pub(crate) fn analyze_local_tempo(beat_positions_seconds: &[f32]) -> TempoDiagnostics {
    let interval_durations: Vec<f32> = beat_positions_seconds
        .windows(2)
        .filter_map(|window| {
            let duration = window[1] - window[0];
            (duration > 0.0).then_some(duration)
        })
        .collect();
    let (_, beat_interval_outliers) = filter_interval_outliers(&interval_durations);
    let interval_tempi = tempo_points(beat_positions_seconds, 1);
    let windowed_tempi = tempo_points(beat_positions_seconds, 4);
    let (median_bpm, drift_span_bpm, mean_abs_deviation_bpm) = tempo_summary(&interval_tempi);
    let (windowed_median_bpm, windowed_drift_span_bpm, windowed_mean_abs_deviation_bpm) =
        tempo_summary(&windowed_tempi);
    let core_windowed_tempi = core_tempo_points(&windowed_tempi);
    let (
        core_windowed_median_bpm,
        core_windowed_drift_span_bpm,
        core_windowed_mean_abs_deviation_bpm,
    ) = tempo_summary(core_windowed_tempi);
    let boundary_bias_bpm = if windowed_tempi.len() <= core_windowed_tempi.len() {
        0.0
    } else {
        let edge_window_count = windowed_tempi.len().min(4);
        let mut edge_deviations: Vec<f32> = windowed_tempi
            .iter()
            .enumerate()
            .filter(|(index, _)| {
                *index < edge_window_count || *index + edge_window_count >= windowed_tempi.len()
            })
            .map(|(_, point)| (point.bpm - core_windowed_median_bpm).abs())
            .collect();
        median(&mut edge_deviations)
    };
    let trend_points = if core_windowed_tempi.is_empty() {
        &windowed_tempi
    } else {
        core_windowed_tempi
    };
    let trend = analyze_tempo_trend(trend_points);
    let beat_grid_error = analyze_beat_grid_error(beat_positions_seconds);
    let edge_trimmed_stable_span = detect_edge_trimmed_stable_span(
        &windowed_tempi,
        core_windowed_median_bpm,
        core_windowed_mean_abs_deviation_bpm,
    );
    let stable_core_span = detect_stable_core_span(
        &windowed_tempi,
        core_windowed_median_bpm,
        core_windowed_mean_abs_deviation_bpm,
    );
    let stability_scope = classify_tempo_stability_scope(
        windowed_tempi.len(),
        &beat_interval_outliers,
        edge_trimmed_stable_span,
        stable_core_span,
    );

    TempoDiagnostics {
        interval_tempi,
        windowed_tempi,
        median_bpm,
        drift_span_bpm,
        mean_abs_deviation_bpm,
        windowed_median_bpm,
        windowed_drift_span_bpm,
        windowed_mean_abs_deviation_bpm,
        core_windowed_median_bpm,
        core_windowed_drift_span_bpm,
        core_windowed_mean_abs_deviation_bpm,
        boundary_bias_bpm,
        trend,
        beat_grid_error,
        beat_interval_outliers,
        stability_scope,
        edge_trimmed_stable_span,
        stable_core_span,
    }
}
