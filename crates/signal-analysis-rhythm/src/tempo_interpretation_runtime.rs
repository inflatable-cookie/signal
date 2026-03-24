use signal_analysis::Confidence;

use crate::{
    TempoDiagnostics, TempoInterpretation, TempoInterpretationProfile, TempoInterpretationReason,
    TempoInterpretationSupport, TempoRecommendation, TempoTrendDirection, TempoTrustLevel,
};

pub(crate) fn interpret_tempo(
    refined_bpm: f32,
    confidence: Confidence,
    tempo_ambiguity: Confidence,
    diagnostics: &TempoDiagnostics,
) -> TempoInterpretation {
    let beat_period_ms = if diagnostics.core_windowed_median_bpm > 0.0 {
        60_000.0 / diagnostics.core_windowed_median_bpm
    } else if refined_bpm > 0.0 {
        60_000.0 / refined_bpm
    } else {
        0.0
    };
    let drift_stability = Confidence::new(
        (1.0 - (0.45 * (diagnostics.trend.total_drift_bpm.abs() / 0.6)
            + 0.55 * (diagnostics.trend.fit_mean_abs_deviation_bpm / 0.75)))
            .clamp(0.0, 1.0),
    );
    let mean_residual_fraction =
        (diagnostics.beat_grid_error.mean_abs_residual_ms / beat_period_ms.max(1.0)).max(0.0);
    let core_residual_fraction =
        (diagnostics.beat_grid_error.core_mean_abs_residual_ms / beat_period_ms.max(1.0)).max(0.0);
    let anchored_drift_fraction =
        (diagnostics.beat_grid_error.mean_abs_anchored_drift_ms / beat_period_ms.max(1.0)).max(0.0);
    let grid_stability = Confidence::new(
        (1.0 - (0.45 * (mean_residual_fraction / 0.18)
            + 0.35 * (core_residual_fraction / 0.15)
            + 0.20 * (anchored_drift_fraction / 0.45)))
            .clamp(0.0, 1.0),
    );
    let core_consensus = Confidence::new(
        (1.0 - ((refined_bpm - diagnostics.core_windowed_median_bpm).abs() / 0.35)).clamp(0.0, 1.0),
    );
    let integer_closeness =
        Confidence::new((1.0 - ((refined_bpm - refined_bpm.round()).abs() / 0.5)).clamp(0.0, 1.0));
    let edge_core_gap_ms = (diagnostics.beat_grid_error.edge_mean_abs_residual_ms
        - diagnostics.beat_grid_error.core_mean_abs_residual_ms)
        .max(0.0);
    let edge_core_gap_fraction = (edge_core_gap_ms / beat_period_ms.max(1.0)).max(0.0);
    let boundary_scope_discount = if diagnostics.windowed_tempi.len() >= 256
        && matches!(diagnostics.trend.direction, TempoTrendDirection::Stable)
        && diagnostics.trend.fit_mean_abs_deviation_bpm <= 0.6
    {
        0.55
    } else if diagnostics.windowed_tempi.len() >= 64
        && matches!(diagnostics.trend.direction, TempoTrendDirection::Stable)
    {
        0.70
    } else if diagnostics.windowed_tempi.len() >= 24 {
        0.85
    } else {
        1.0
    };
    let boundary_bias_scale_bpm = (0.35
        + 1.5 * diagnostics.trend.fit_mean_abs_deviation_bpm
        + 0.15 * diagnostics.trend.total_drift_bpm.abs())
    .clamp(0.35, 1.5);
    let edge_gap_scale_fraction = if diagnostics.windowed_tempi.len() >= 128 {
        0.30
    } else if diagnostics.windowed_tempi.len() >= 64 {
        0.24
    } else {
        0.18
    };
    let boundary_pressure = Confidence::new(
        ((0.35 * (diagnostics.boundary_bias_bpm / boundary_bias_scale_bpm)
            + 0.65 * (edge_core_gap_fraction / edge_gap_scale_fraction))
            * boundary_scope_discount)
            .clamp(0.0, 1.0),
    );
    let strong_integer_anchor = integer_closeness.0 > 0.92
        && core_consensus.0 > 0.88
        && matches!(diagnostics.trend.direction, TempoTrendDirection::Stable)
        && diagnostics.trend.fit_mean_abs_deviation_bpm <= 0.6
        && diagnostics.trend.total_drift_bpm.abs() <= 0.2
        && boundary_pressure.0 < 0.6;
    let localized_terminal_outliers = diagnostics.beat_interval_outliers.total_intervals >= 32
        && diagnostics
            .beat_interval_outliers
            .trailing_rejected_intervals
            >= 2
        && diagnostics
            .beat_interval_outliers
            .leading_rejected_intervals
            == 0
        && diagnostics.beat_interval_outliers.rejected_intervals
            <= diagnostics.beat_interval_outliers.total_intervals / 4;
    let effective_ambiguity = Confidence::new(
        (tempo_ambiguity.0
            * if strong_integer_anchor {
                0.35
            } else if core_consensus.0 > 0.88
                && drift_stability.0 > 0.5
                && boundary_pressure.0 < 0.65
            {
                0.55
            } else {
                1.0
            })
        .clamp(0.0, 1.0),
    );
    let support = TempoInterpretationSupport {
        core_consensus,
        drift_stability,
        grid_stability,
        integer_closeness,
        boundary_pressure,
    };
    let stability_score = (0.35 * confidence.0
        + 0.20 * core_consensus.0
        + 0.20 * drift_stability.0
        + 0.15 * grid_stability.0
        + 0.10 * (1.0 - effective_ambiguity.0))
        .clamp(0.0, 1.0);
    let nearest_integer_bpm = refined_bpm.round();
    let snap_error_bpm = (refined_bpm - nearest_integer_bpm).abs();
    let profile = TempoInterpretationProfile {
        refined_bpm,
        core_window_bpm: diagnostics.core_windowed_median_bpm,
        nearest_integer_bpm,
        snap_error_bpm,
        stability_score: Confidence::new(stability_score),
        boundary_edge_gap_ms: edge_core_gap_ms,
    };
    let destabilized_edge_pressure = boundary_pressure.0 > 0.72
        && edge_core_gap_fraction > 0.12
        && (stability_score < 0.62
            || core_consensus.0 < 0.75
            || (drift_stability.0 < 0.48 && grid_stability.0 < 0.48));

    if confidence.0 < 0.4
        || stability_score < 0.45
        || destabilized_edge_pressure
        || (effective_ambiguity.0 > 0.6 && integer_closeness.0 < 0.8)
    {
        return TempoInterpretation {
            trust: TempoTrustLevel::Tentative,
            recommendation: TempoRecommendation::Defer,
            reason: TempoInterpretationReason::UnstableTempo,
            recommended_bpm: refined_bpm,
            snapped_bpm: None,
            support,
            profile,
        };
    }

    if boundary_pressure.0 > 0.55
        && core_consensus.0 > 0.8
        && drift_stability.0 > 0.55
        && !strong_integer_anchor
        && diagnostics.core_windowed_mean_abs_deviation_bpm
            <= diagnostics.windowed_mean_abs_deviation_bpm + 0.02
    {
        return TempoInterpretation {
            trust: if stability_score >= 0.8 {
                TempoTrustLevel::Stable
            } else {
                TempoTrustLevel::Guarded
            },
            recommendation: TempoRecommendation::UseCoreWindow,
            reason: TempoInterpretationReason::StableCoreWindow,
            recommended_bpm: diagnostics.core_windowed_median_bpm,
            snapped_bpm: None,
            support,
            profile,
        };
    }

    if integer_closeness.0 > 0.8
        && (snap_error_bpm >= 0.04
            || ((0.015..=0.03).contains(&snap_error_bpm)
                && strong_integer_anchor
                && drift_stability.0 > 0.55
                && grid_stability.0 > 0.35)
            || (snap_error_bpm <= 0.04
                && localized_terminal_outliers
                && strong_integer_anchor
                && drift_stability.0 > 0.5
                && grid_stability.0 > 0.35))
        && boundary_pressure.0 < 0.6
        && drift_stability.0 > 0.4
        && grid_stability.0 > 0.35
        && effective_ambiguity.0 < 0.7
    {
        let snapped_bpm = nearest_integer_bpm;
        return TempoInterpretation {
            trust: if stability_score >= 0.8 {
                TempoTrustLevel::Stable
            } else {
                TempoTrustLevel::Guarded
            },
            recommendation: TempoRecommendation::SnapInteger,
            reason: TempoInterpretationReason::NearIntegerPulse,
            recommended_bpm: snapped_bpm,
            snapped_bpm: Some(snapped_bpm),
            support,
            profile,
        };
    }

    TempoInterpretation {
        trust: if stability_score >= 0.7 {
            TempoTrustLevel::Stable
        } else {
            TempoTrustLevel::Guarded
        },
        recommendation: TempoRecommendation::UseRefined,
        reason: TempoInterpretationReason::StableRefinedPulse,
        recommended_bpm: refined_bpm,
        snapped_bpm: None,
        support,
        profile,
    }
}
