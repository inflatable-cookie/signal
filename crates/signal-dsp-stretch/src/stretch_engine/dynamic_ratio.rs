//! Dynamic-ratio segment planning and seam smoothing.

use signal_primitives::Sample;

use crate::cache_identity::StretchRatioPoint;
#[cfg(any(test, feature = "evidence"))]
use crate::stretch_backend::{DEFAULT_ANALYSIS_HOP, DEFAULT_WINDOW_SIZE};

use super::limits::MIN_DYNAMIC_RATIO_SEGMENT_EXTRA_HOPS;
use super::math::sanitize_ratio;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct DynamicRatioSegment {
    pub(crate) start_frame: usize,
    pub(crate) end_frame: usize,
    pub(crate) target_frames: usize,
    pub(crate) ratio: f64,
}

pub(crate) fn dynamic_ratio_output_frames(
    input_frames: usize,
    ratio_curve: &[StretchRatioPoint],
    fallback_ratio: f64,
) -> usize {
    dynamic_ratio_segments(input_frames, ratio_curve, sanitize_ratio(fallback_ratio))
        .iter()
        .map(|segment| segment.target_frames)
        .sum()
}

/// Output-frame positions of the seams a dynamic-ratio render actually
/// produces, after short segments are coalesced.
#[cfg(any(test, feature = "evidence"))]
pub(crate) fn dynamic_ratio_output_boundaries(
    input_frames: usize,
    ratio_curve: &[StretchRatioPoint],
    fallback_ratio: f64,
) -> Vec<usize> {
    let segments = coalesce_short_dynamic_ratio_segments(
        dynamic_ratio_segments(input_frames, ratio_curve, sanitize_ratio(fallback_ratio)),
        min_dynamic_ratio_segment_frames(DEFAULT_WINDOW_SIZE, DEFAULT_ANALYSIS_HOP),
    );
    dynamic_ratio_segment_boundaries(&segments)
}

pub(crate) fn dynamic_ratio_segments(
    input_frames: usize,
    ratio_curve: &[StretchRatioPoint],
    fallback_ratio: f64,
) -> Vec<DynamicRatioSegment> {
    if input_frames == 0 {
        return Vec::new();
    }

    let mut points = std::collections::BTreeMap::<usize, f64>::new();
    for point in ratio_curve {
        if point.timeline_frame < 0 || !point.ratio.is_finite() || point.ratio <= 0.0 {
            continue;
        }
        points.insert(point.timeline_frame as usize, point.ratio);
    }

    let mut segments = Vec::new();
    let mut start_frame = 0usize;
    let mut ratio = sanitize_ratio(fallback_ratio);
    for (point_frame, point_ratio) in points {
        let point_frame = point_frame.min(input_frames);
        if point_frame > start_frame {
            segments.push(dynamic_ratio_segment(start_frame, point_frame, ratio));
        }
        ratio = point_ratio;
        start_frame = point_frame;
    }

    if start_frame < input_frames {
        segments.push(dynamic_ratio_segment(start_frame, input_frames, ratio));
    }
    segments
}

pub(crate) fn dynamic_ratio_segment(
    start_frame: usize,
    end_frame: usize,
    ratio: f64,
) -> DynamicRatioSegment {
    DynamicRatioSegment {
        start_frame,
        end_frame,
        target_frames: ((end_frame - start_frame) as f64 * ratio).round() as usize,
        ratio,
    }
}

/// Merge adjacent segments until every one carries at least
/// `min_segment_frames` source frames.
///
/// A segment shorter than one analysis window cannot be rendered by the STFT
/// engine and would fall through to time-domain interpolation, which
/// pitch-shifts. Merging keeps the render pitch-preserving.
///
/// The merged target frame count is the sum of the counts its constituent
/// spans would have produced, so total output length and the average tempo
/// over the merged span are preserved exactly and the segment renders at the
/// mean ratio of the spans it covers. Frozen by Contract `046`, 2026-07-27
/// addendum.
pub(crate) fn coalesce_short_dynamic_ratio_segments(
    segments: Vec<DynamicRatioSegment>,
    min_segment_frames: usize,
) -> Vec<DynamicRatioSegment> {
    if min_segment_frames <= 1 || segments.len() < 2 {
        return segments;
    }

    let mut coalesced: Vec<DynamicRatioSegment> = Vec::with_capacity(segments.len());
    for segment in segments {
        match coalesced.last_mut() {
            Some(previous) if previous.end_frame - previous.start_frame < min_segment_frames => {
                previous.end_frame = segment.end_frame;
                previous.target_frames += segment.target_frames;
                previous.ratio = mean_segment_ratio(previous);
            }
            _ => coalesced.push(segment),
        }
    }

    // The final segment can still be short when the source ends mid-span. Fold
    // it backwards rather than leaving one sub-window render at the tail.
    while coalesced.len() >= 2 {
        let last = coalesced[coalesced.len() - 1];
        if last.end_frame - last.start_frame >= min_segment_frames {
            break;
        }
        coalesced.pop();
        let previous = coalesced
            .last_mut()
            .expect("length checked before the pop above");
        previous.end_frame = last.end_frame;
        previous.target_frames += last.target_frames;
        previous.ratio = mean_segment_ratio(previous);
    }

    coalesced
}

/// Shortest source span a dynamic-ratio segment may render.
///
/// One window yields a single analysis frame, which is enough to avoid the
/// interpolation fallback but not enough for the phase vocoder to track the
/// source. The extra hops give every segment several overlapping frames.
pub(crate) fn min_dynamic_ratio_segment_frames(window_size: usize, analysis_hop: usize) -> usize {
    window_size + analysis_hop.saturating_mul(MIN_DYNAMIC_RATIO_SEGMENT_EXTRA_HOPS)
}

pub(crate) fn mean_segment_ratio(segment: &DynamicRatioSegment) -> f64 {
    let source_frames = segment.end_frame - segment.start_frame;
    if source_frames == 0 {
        return segment.ratio;
    }
    segment.target_frames as f64 / source_frames as f64
}

pub(crate) fn dynamic_ratio_segment_boundaries(segments: &[DynamicRatioSegment]) -> Vec<usize> {
    let mut boundaries = Vec::with_capacity(segments.len().saturating_sub(1));
    let total_frames: usize = segments.iter().map(|segment| segment.target_frames).sum();
    let mut output_frame = 0usize;
    for segment in segments.iter().take(segments.len().saturating_sub(1)) {
        output_frame += segment.target_frames;
        if output_frame > 0 && output_frame < total_frames {
            boundaries.push(output_frame);
        }
    }
    boundaries
}

/// Smooth deterministic dynamic-ratio segment joins in place.
///
/// This is an offline render helper for independently rendered segment joins.
/// It does not change output length or boundary positions.
pub(crate) fn smooth_dynamic_segment_boundaries_interleaved(
    interleaved_samples: &mut [Sample],
    channels: u16,
    boundary_frames: &[usize],
    fade_frames: usize,
) {
    let channel_count = channels as usize;
    if channel_count == 0 || fade_frames == 0 {
        return;
    }
    let frames = interleaved_samples.len() / channel_count;
    if frames < 2 {
        return;
    }

    for boundary in boundary_frames {
        if *boundary == 0 || *boundary >= frames {
            continue;
        }
        let fade_frames = fade_frames.min(*boundary).min(frames - *boundary).max(1);
        for channel in 0..channel_count {
            let before_edge_index = (*boundary - 1) * channel_count + channel;
            let after_edge_index = *boundary * channel_count + channel;
            let before_edge = interleaved_samples[before_edge_index];
            let after_edge = interleaved_samples[after_edge_index];
            let midpoint = (before_edge + after_edge) * 0.5;
            for offset in 0..fade_frames {
                let weight = (fade_frames - offset) as f32 / fade_frames as f32;
                let before_frame = *boundary - 1 - offset;
                let after_frame = *boundary + offset;
                let before_index = before_frame * channel_count + channel;
                let after_index = after_frame * channel_count + channel;
                interleaved_samples[before_index] += (midpoint - before_edge) * weight;
                interleaved_samples[after_index] += (midpoint - after_edge) * weight;
            }
        }
    }
}
