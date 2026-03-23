use crate::tempo_policy::{BeatIntervalOutlierDiagnostics, LocalTempoPoint};

pub(crate) fn filter_interval_outliers(
    intervals: &[f32],
) -> (Vec<f32>, BeatIntervalOutlierDiagnostics) {
    let valid: Vec<f32> = intervals
        .iter()
        .copied()
        .filter(|interval| *interval > 0.0)
        .collect();
    if valid.is_empty() {
        return (
            Vec::new(),
            BeatIntervalOutlierDiagnostics {
                total_intervals: 0,
                retained_intervals: 0,
                rejected_intervals: 0,
                leading_rejected_intervals: 0,
                trailing_rejected_intervals: 0,
                median_interval: 0.0,
                median_abs_deviation: 0.0,
                max_rejected_deviation_ratio: 0.0,
            },
        );
    }

    let mut medians = valid.clone();
    let median_interval = median(&mut medians);
    let mut deviations: Vec<f32> = valid
        .iter()
        .map(|interval| (interval - median_interval).abs())
        .collect();
    let median_abs_deviation = median(&mut deviations);
    let deviation_limit = (median_interval * 0.08)
        .max(4.0 * median_abs_deviation)
        .max(median_interval * 0.02);

    let keep_mask: Vec<bool> = valid
        .iter()
        .map(|interval| (interval - median_interval).abs() <= deviation_limit)
        .collect();
    let retained: Vec<f32> = valid
        .iter()
        .zip(keep_mask.iter())
        .filter_map(|(interval, keep)| keep.then_some(*interval))
        .collect();
    let edge_window = keep_mask.len().min(4);
    let leading_rejected_intervals = keep_mask
        .iter()
        .take(edge_window)
        .filter(|keep| !**keep)
        .count();
    let trailing_rejected_intervals = keep_mask
        .iter()
        .rev()
        .take(edge_window)
        .filter(|keep| !**keep)
        .count();
    let max_rejected_deviation_ratio = valid
        .iter()
        .zip(keep_mask.iter())
        .filter_map(|(interval, keep)| {
            (!*keep).then_some((interval - median_interval).abs() / median_interval.max(1.0e-6))
        })
        .fold(0.0, f32::max);

    (
        retained,
        BeatIntervalOutlierDiagnostics {
            total_intervals: valid.len(),
            retained_intervals: keep_mask.iter().filter(|keep| **keep).count(),
            rejected_intervals: keep_mask.iter().filter(|keep| !**keep).count(),
            leading_rejected_intervals,
            trailing_rejected_intervals,
            median_interval,
            median_abs_deviation,
            max_rejected_deviation_ratio,
        },
    )
}

pub(crate) fn median(values: &mut [f32]) -> f32 {
    if values.is_empty() {
        return 0.0;
    }
    values.sort_by(|lhs, rhs| lhs.partial_cmp(rhs).unwrap_or(core::cmp::Ordering::Equal));
    let mid = values.len() / 2;
    if values.len() % 2 == 0 {
        0.5 * (values[mid - 1] + values[mid])
    } else {
        values[mid]
    }
}

pub(crate) fn tempo_summary(points: &[LocalTempoPoint]) -> (f32, f32, f32) {
    let mut bpms: Vec<f32> = points.iter().map(|point| point.bpm).collect();
    let median_bpm = median(&mut bpms);
    let (min_bpm, max_bpm) = bpms.iter().copied().fold(
        (f32::INFINITY, f32::NEG_INFINITY),
        |(min_bpm, max_bpm), bpm| (min_bpm.min(bpm), max_bpm.max(bpm)),
    );
    let drift_span_bpm = if bpms.is_empty() {
        0.0
    } else {
        max_bpm - min_bpm
    };
    let mean_abs_deviation_bpm = if bpms.is_empty() {
        0.0
    } else {
        bpms.iter().map(|bpm| (bpm - median_bpm).abs()).sum::<f32>() / bpms.len() as f32
    };

    (median_bpm, drift_span_bpm, mean_abs_deviation_bpm)
}

pub(crate) fn linear_fit(points: &[(f32, f32)]) -> Option<(f32, f32)> {
    if points.len() < 2 {
        return None;
    }

    let count = points.len() as f32;
    let mean_x = points.iter().map(|(x, _)| *x).sum::<f32>() / count;
    let mean_y = points.iter().map(|(_, y)| *y).sum::<f32>() / count;
    let mut numerator = 0.0;
    let mut denominator = 0.0;
    for (x, y) in points {
        let dx = *x - mean_x;
        numerator += dx * (*y - mean_y);
        denominator += dx * dx;
    }

    if denominator <= f32::EPSILON {
        return None;
    }

    let slope = numerator / denominator;
    let intercept = mean_y - slope * mean_x;
    Some((intercept, slope))
}

pub(crate) fn mean_abs(values: &[f32]) -> f32 {
    if values.is_empty() {
        0.0
    } else {
        values.iter().map(|value| value.abs()).sum::<f32>() / values.len() as f32
    }
}
