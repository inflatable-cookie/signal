use core::cmp::Ordering;

pub(crate) fn reduce_median(values: &mut [f32]) -> f32 {
    if values.is_empty() {
        return 0.0;
    }

    sort_f32(values);
    values[values.len() / 2]
}

pub(crate) fn reduce_median_or_zero(values: &mut [f32]) -> f32 {
    if values.is_empty() {
        0.0
    } else {
        reduce_median(values)
    }
}

pub(crate) fn sort_f32(values: &mut [f32]) {
    values.sort_by(|left, right| left.partial_cmp(right).unwrap_or(Ordering::Equal));
}

pub(crate) fn percentile_value(sorted_values: &[f32], fraction: f32) -> f32 {
    if sorted_values.is_empty() {
        return 0.0;
    }

    let clamped = fraction.clamp(0.0, 1.0);
    let index = ((sorted_values.len().saturating_sub(1) as f32) * clamped).round() as usize;
    sorted_values[index.min(sorted_values.len() - 1)]
}

pub(crate) fn normalize_slice(values: &mut [f32]) {
    let sum = values.iter().copied().sum::<f32>();
    if sum > 0.0 {
        for value in values {
            *value /= sum;
        }
    }
}

pub(crate) fn mean_or_zero(values: &[f32]) -> f32 {
    if values.is_empty() {
        0.0
    } else {
        values.iter().copied().sum::<f32>() / values.len() as f32
    }
}
