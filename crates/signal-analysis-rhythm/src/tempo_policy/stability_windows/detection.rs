use signal_analysis::Confidence;

use crate::tempo_policy::{BeatGridCoreSpanDiagnostics, LocalTempoPoint};

pub(crate) fn core_tempo_points(points: &[LocalTempoPoint]) -> &[LocalTempoPoint] {
    if points.len() <= 4 {
        points
    } else {
        &points[1..points.len() - 1]
    }
}

fn stable_window_mask(
    points: &[LocalTempoPoint],
    core_median_bpm: f32,
    core_mean_abs_deviation_bpm: f32,
) -> Vec<bool> {
    if points.is_empty() || core_median_bpm <= 0.0 {
        return Vec::new();
    }

    let tolerance_bpm = (0.45 + 3.0 * core_mean_abs_deviation_bpm).clamp(0.45, 2.0);
    let mut keep_mask: Vec<bool> = points
        .iter()
        .map(|point| (point.bpm - core_median_bpm).abs() <= tolerance_bpm)
        .collect();
    let gap_limit = 2usize;
    let mut index = 0usize;
    while index < keep_mask.len() {
        if keep_mask[index] {
            index += 1;
            continue;
        }
        let gap_start = index;
        while index < keep_mask.len() && !keep_mask[index] {
            index += 1;
        }
        let gap_end = index;
        let gap_len = gap_end - gap_start;
        let bounded_by_true = gap_start > 0
            && gap_end < keep_mask.len()
            && keep_mask[gap_start - 1]
            && keep_mask[gap_end];
        if bounded_by_true && gap_len <= gap_limit {
            for keep in &mut keep_mask[gap_start..gap_end] {
                *keep = true;
            }
        }
    }
    keep_mask
}

fn stable_span_diagnostics(
    points: &[LocalTempoPoint],
    keep_mask: &[bool],
    start: usize,
    end: usize,
) -> Option<BeatGridCoreSpanDiagnostics> {
    if points.is_empty() || keep_mask.len() != points.len() || start > end || end >= points.len() {
        return None;
    }

    let span_start = &points[start];
    let span_end = &points[end];
    let trimmed_leading_windows = start;
    let trimmed_trailing_windows = points.len().saturating_sub(end + 1);
    let retained_windows = end + 1 - start;
    let interior_rejected_windows = keep_mask[start..=end].iter().filter(|keep| !**keep).count();
    let total_beat_span = points
        .last()
        .map(|point| {
            point
                .end_beat_index
                .saturating_sub(points[0].start_beat_index)
        })
        .unwrap_or(0);
    let retained_beat_span = span_end
        .end_beat_index
        .saturating_sub(span_start.start_beat_index);
    let coverage = if total_beat_span == 0 {
        1.0
    } else {
        retained_beat_span as f32 / total_beat_span as f32
    };

    Some(BeatGridCoreSpanDiagnostics {
        start_beat_index: span_start.start_beat_index,
        end_beat_index: span_end.end_beat_index,
        start_seconds: span_start.start_seconds,
        end_seconds: span_end.end_seconds,
        coverage: Confidence::new(coverage),
        retained_windows,
        total_windows: points.len(),
        trimmed_leading_windows,
        trimmed_trailing_windows,
        interior_rejected_windows,
    })
}

pub(crate) fn detect_stable_core_span(
    points: &[LocalTempoPoint],
    core_median_bpm: f32,
    core_mean_abs_deviation_bpm: f32,
) -> Option<BeatGridCoreSpanDiagnostics> {
    let keep_mask = stable_window_mask(points, core_median_bpm, core_mean_abs_deviation_bpm);
    if keep_mask.is_empty() {
        return None;
    }

    let mut best_start = None;
    let mut best_len = 0usize;
    let mut current_start = None;
    for (index, keep) in keep_mask.iter().copied().enumerate() {
        if keep {
            current_start.get_or_insert(index);
        } else if let Some(start) = current_start.take() {
            let len = index - start;
            if len > best_len {
                best_len = len;
                best_start = Some(start);
            }
        }
    }
    if let Some(start) = current_start {
        let len = keep_mask.len() - start;
        if len > best_len {
            best_len = len;
            best_start = Some(start);
        }
    }

    let start = best_start?;
    let end = start + best_len - 1;
    stable_span_diagnostics(points, &keep_mask, start, end)
}

pub(crate) fn detect_edge_trimmed_stable_span(
    points: &[LocalTempoPoint],
    core_median_bpm: f32,
    core_mean_abs_deviation_bpm: f32,
) -> Option<BeatGridCoreSpanDiagnostics> {
    let keep_mask = stable_window_mask(points, core_median_bpm, core_mean_abs_deviation_bpm);
    if keep_mask.is_empty() {
        return None;
    }

    let mut best: Option<(usize, usize, usize)> = None;
    for start in 0..keep_mask.len() {
        if !keep_mask[start] {
            continue;
        }
        let mut rejected = 0usize;
        for end in start..keep_mask.len() {
            if !keep_mask[end] {
                rejected += 1;
                continue;
            }
            let len = end + 1 - start;
            let allowed_rejections = (len / 10).max(2);
            if rejected > allowed_rejections {
                continue;
            }
            let candidate = (start, end, rejected);
            let replace = match best {
                None => true,
                Some((best_start, best_end, best_rejected)) => {
                    let best_len = best_end + 1 - best_start;
                    len > best_len
                        || (len == best_len && rejected < best_rejected)
                        || (len == best_len
                            && rejected == best_rejected
                            && (start + (keep_mask.len() - end - 1))
                                < (best_start + (keep_mask.len() - best_end - 1)))
                }
            };
            if replace {
                best = Some(candidate);
            }
        }
    }

    let (start, end, _) = best?;
    stable_span_diagnostics(points, &keep_mask, start, end)
}
