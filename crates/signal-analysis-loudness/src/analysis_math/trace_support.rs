use crate::types::{LoudnessDynamicsSummary, LoudnessTrace, LoudnessTracePoint};

use super::lufs_from_mean_square;

pub(crate) fn loudness_trace_from_energies(
    energies: &[f32],
    window_seconds: f32,
    hop_seconds: f32,
) -> LoudnessTrace {
    let points = energies
        .iter()
        .copied()
        .enumerate()
        .map(|(index, energy)| {
            let start_seconds = index as f32 * hop_seconds;
            LoudnessTracePoint {
                index,
                start_seconds,
                end_seconds: start_seconds + window_seconds,
                loudness_lufs: lufs_from_mean_square(energy),
            }
        })
        .collect();

    LoudnessTrace {
        window_seconds,
        hop_seconds,
        points,
    }
}

pub(crate) fn trace_tail(trace: &LoudnessTrace, max_points: usize) -> LoudnessTrace {
    let keep_from = trace.points.len().saturating_sub(max_points);
    LoudnessTrace {
        window_seconds: trace.window_seconds,
        hop_seconds: trace.hop_seconds,
        points: trace.points[keep_from..].to_vec(),
    }
}

pub(crate) fn trace_latest_loudness(trace: &LoudnessTrace) -> f32 {
    trace
        .points
        .last()
        .map(|point| point.loudness_lufs)
        .unwrap_or(f32::NEG_INFINITY)
}

pub(crate) fn dynamics_summary(
    target_lufs: f32,
    integrated_lufs: f32,
    true_peak_dbtp: f32,
    momentary_trace: &LoudnessTrace,
    short_term_trace: &LoudnessTrace,
) -> LoudnessDynamicsSummary {
    let momentary_values: Vec<f32> = momentary_trace
        .points
        .iter()
        .map(|point| point.loudness_lufs)
        .filter(|value| value.is_finite())
        .collect();
    let short_term_values: Vec<f32> = short_term_trace
        .points
        .iter()
        .map(|point| point.loudness_lufs)
        .filter(|value| value.is_finite())
        .collect();
    let momentary_max_lufs = finite_max_or_neg_infinity(&momentary_values);
    let short_term_max_lufs = finite_max_or_neg_infinity(&short_term_values);
    let peak_to_loudness_lu = if true_peak_dbtp.is_finite() && integrated_lufs.is_finite() {
        (true_peak_dbtp - integrated_lufs).max(0.0)
    } else {
        0.0
    };

    LoudnessDynamicsSummary {
        target_offset_lu: if integrated_lufs.is_finite() {
            integrated_lufs - target_lufs
        } else {
            0.0
        },
        peak_to_loudness_lu,
        momentary_max_lufs,
        short_term_max_lufs,
        momentary_range_lu: loudness_range_from_values(&momentary_values),
        short_term_range_lu: loudness_range_from_values(&short_term_values),
    }
}

fn finite_max_or_neg_infinity(values: &[f32]) -> f32 {
    values.iter().copied().fold(f32::NEG_INFINITY, f32::max)
}

fn loudness_range_from_values(values: &[f32]) -> f32 {
    if values.len() < 2 {
        return 0.0;
    }

    let mut sorted = values.to_vec();
    sorted.sort_by(|lhs, rhs| lhs.partial_cmp(rhs).unwrap_or(core::cmp::Ordering::Equal));
    let lower = percentile(&sorted, 0.10);
    let upper = percentile(&sorted, 0.95);
    (upper - lower).max(0.0)
}

fn percentile(sorted: &[f32], fraction: f32) -> f32 {
    let index = ((sorted.len() - 1) as f32 * fraction).round() as usize;
    sorted[index.min(sorted.len() - 1)]
}
