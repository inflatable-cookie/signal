#[allow(clippy::too_many_arguments)]
fn synthetic_tempo_diagnostics(
    core_window_bpm: f32,
    boundary_bias_bpm: f32,
    trend_total_drift_bpm: f32,
    trend_fit_mad_bpm: f32,
    mean_abs_residual_ms: f32,
    core_abs_residual_ms: f32,
    anchored_drift_ms: f32,
    edge_abs_residual_ms: f32,
) -> super::TempoDiagnostics {
    super::TempoDiagnostics {
        interval_tempi: Vec::new(),
        windowed_tempi: Vec::new(),
        median_bpm: core_window_bpm,
        drift_span_bpm: boundary_bias_bpm,
        mean_abs_deviation_bpm: trend_fit_mad_bpm,
        windowed_median_bpm: core_window_bpm,
        windowed_drift_span_bpm: boundary_bias_bpm,
        windowed_mean_abs_deviation_bpm: trend_fit_mad_bpm,
        core_windowed_median_bpm: core_window_bpm,
        core_windowed_drift_span_bpm: trend_total_drift_bpm.abs(),
        core_windowed_mean_abs_deviation_bpm: trend_fit_mad_bpm,
        boundary_bias_bpm,
        trend: super::TempoTrendDiagnostics {
            direction: if trend_total_drift_bpm.abs() < 0.15 {
                super::TempoTrendDirection::Stable
            } else if trend_total_drift_bpm > 0.0 {
                super::TempoTrendDirection::Accelerating
            } else {
                super::TempoTrendDirection::Decelerating
            },
            start_bpm: core_window_bpm - 0.5 * trend_total_drift_bpm,
            end_bpm: core_window_bpm + 0.5 * trend_total_drift_bpm,
            total_drift_bpm: trend_total_drift_bpm,
            slope_bpm_per_beat: trend_total_drift_bpm / 8.0,
            fit_mean_abs_deviation_bpm: trend_fit_mad_bpm,
        },
        beat_grid_error: super::BeatGridErrorDiagnostics {
            residuals: Vec::new(),
            mean_abs_residual_ms,
            max_abs_residual_ms: edge_abs_residual_ms.max(core_abs_residual_ms),
            edge_mean_abs_residual_ms: edge_abs_residual_ms,
            core_mean_abs_residual_ms: core_abs_residual_ms,
            end_anchored_drift_ms: anchored_drift_ms,
            mean_abs_anchored_drift_ms: anchored_drift_ms.abs(),
        },
        beat_interval_outliers: super::BeatIntervalOutlierDiagnostics {
            total_intervals: 0,
            retained_intervals: 0,
            rejected_intervals: 0,
            leading_rejected_intervals: 0,
            trailing_rejected_intervals: 0,
            median_interval: 0.0,
            median_abs_deviation: 0.0,
            max_rejected_deviation_ratio: 0.0,
        },
        stability_scope: super::TempoStabilityScopeSummary {
            scope: super::TempoStabilityScope::MidTrackUnstable,
            support: super::TempoStabilityScopeSupport {
                edge_trimmed_coverage: super::Confidence::new(0.0),
                contiguous_core_coverage: super::Confidence::new(0.0),
                interior_stability: super::Confidence::new(0.0),
                edge_locality: super::Confidence::new(0.0),
            },
        },
        edge_trimmed_stable_span: None,
        stable_core_span: None,
    }
}

#[allow(clippy::too_many_arguments)]
fn synthetic_tempo_diagnostics_with_counts(
    core_window_bpm: f32,
    boundary_bias_bpm: f32,
    trend_total_drift_bpm: f32,
    trend_fit_mad_bpm: f32,
    mean_abs_residual_ms: f32,
    core_abs_residual_ms: f32,
    anchored_drift_ms: f32,
    edge_abs_residual_ms: f32,
    interval_count: usize,
    windowed_count: usize,
    residual_count: usize,
) -> super::TempoDiagnostics {
    let mut diagnostics = synthetic_tempo_diagnostics(
        core_window_bpm,
        boundary_bias_bpm,
        trend_total_drift_bpm,
        trend_fit_mad_bpm,
        mean_abs_residual_ms,
        core_abs_residual_ms,
        anchored_drift_ms,
        edge_abs_residual_ms,
    );
    diagnostics.interval_tempi = (0..interval_count)
        .map(|index| super::LocalTempoPoint {
            start_beat_index: index,
            end_beat_index: index + 1,
            start_seconds: index as f32,
            end_seconds: index as f32 + 60.0 / core_window_bpm.max(1.0),
            bpm: core_window_bpm,
        })
        .collect();
    diagnostics.windowed_tempi = (0..windowed_count)
        .map(|index| super::LocalTempoPoint {
            start_beat_index: index,
            end_beat_index: index + 4,
            start_seconds: index as f32,
            end_seconds: index as f32 + 4.0 * (60.0 / core_window_bpm.max(1.0)),
            bpm: core_window_bpm,
        })
        .collect();
    diagnostics.beat_grid_error.residuals = (0..residual_count)
        .map(|beat_index| super::BeatGridResidualPoint {
            beat_index,
            seconds: beat_index as f32 * (60.0 / core_window_bpm.max(1.0)),
            fitted_residual_ms: 0.0,
            anchored_drift_ms: 0.0,
        })
        .collect();
    diagnostics.beat_interval_outliers = super::BeatIntervalOutlierDiagnostics {
        total_intervals: interval_count,
        retained_intervals: interval_count,
        rejected_intervals: 0,
        leading_rejected_intervals: 0,
        trailing_rejected_intervals: 0,
        median_interval: 60.0 / core_window_bpm.max(1.0),
        median_abs_deviation: 0.0,
        max_rejected_deviation_ratio: 0.0,
    };
    let stable_span = if windowed_count == 0 {
        None
    } else {
        Some(super::BeatGridCoreSpanDiagnostics {
            start_beat_index: 0,
            end_beat_index: (windowed_count + 3).min(interval_count),
            start_seconds: 0.0,
            end_seconds: (windowed_count + 3) as f32 * (60.0 / core_window_bpm.max(1.0)),
            coverage: super::Confidence::new(1.0),
            retained_windows: windowed_count,
            total_windows: windowed_count,
            trimmed_leading_windows: 0,
            trimmed_trailing_windows: 0,
            interior_rejected_windows: 0,
        })
    };
    diagnostics.stability_scope = super::classify_tempo_stability_scope(
        windowed_count,
        &diagnostics.beat_interval_outliers,
        stable_span,
        stable_span,
    );
    diagnostics.edge_trimmed_stable_span = stable_span;
    diagnostics.stable_core_span = stable_span;
    diagnostics
}

#[allow(clippy::too_many_arguments)]
fn synthetic_tempo_interpretation(
    recommendation: super::TempoRecommendation,
    trust: super::TempoTrustLevel,
    reason: super::TempoInterpretationReason,
    recommended_bpm: f32,
    snapped_bpm: Option<f32>,
    stability_score: f32,
    snap_error_bpm: f32,
    boundary_pressure: f32,
    grid_stability: f32,
) -> super::TempoInterpretation {
    super::TempoInterpretation {
        trust,
        recommendation,
        reason,
        recommended_bpm,
        snapped_bpm,
        support: super::TempoInterpretationSupport {
            core_consensus: super::Confidence::new(0.9),
            drift_stability: super::Confidence::new(0.8),
            grid_stability: super::Confidence::new(grid_stability),
            integer_closeness: super::Confidence::new(
                (1.0 - snap_error_bpm / 0.12).clamp(0.0, 1.0),
            ),
            boundary_pressure: super::Confidence::new(boundary_pressure),
        },
        profile: super::TempoInterpretationProfile {
            refined_bpm: recommended_bpm,
            core_window_bpm: recommended_bpm,
            nearest_integer_bpm: recommended_bpm.round(),
            snap_error_bpm,
            stability_score: super::Confidence::new(stability_score),
            boundary_edge_gap_ms: 4.0 * boundary_pressure,
        },
    }
}

fn synthetic_tempo_structure_result(
    diagnostics: super::TempoDiagnostics,
    interpretation: super::TempoInterpretation,
    confidence: super::Confidence,
    tempo_ambiguity: super::Confidence,
) -> super::BeatAnalysisResult {
    let mut result = analyze_fixture(&click_track(
        48_000,
        interpretation.recommended_bpm.max(60.0),
        8.0,
    ));
    let stability_scope = diagnostics.stability_scope;
    result.bpm = interpretation.recommended_bpm;
    result.confidence = confidence;
    result.tempo_diagnostics = diagnostics;
    result.tempo_interpretation = interpretation;
    result.tempo_state = super::tempo_state_recommendation_with_scope(
        interpretation,
        confidence,
        tempo_ambiguity,
        stability_scope,
    );
    result.tempo_ambiguity = tempo_ambiguity;
    result
}
