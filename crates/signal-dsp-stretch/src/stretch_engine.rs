//! Core stretch render helpers (ratio sanitization, dynamic segments, pitch).

use signal_dsp_resample::{resample_mono, ResampleConfig, ResampleQuality};
use signal_primitives::{Sample, SampleRate};

use crate::cache_identity::StretchRatioPoint;
use crate::phase_vocoder::transient_reset_phase_vocoder_linked_stereo;
use crate::stretch_backend::{
    OfflineHighQualityPath, PhaseVocoderStretcher, TimeStretcher,
    COMPRESSION_SHORT_WINDOW_SELECTOR_ANALYSIS_HOP,
    COMPRESSION_SHORT_WINDOW_SELECTOR_MIN_CURRENT_MISSES,
    COMPRESSION_SHORT_WINDOW_SELECTOR_MIN_CURRENT_SMEAR_FRAMES,
    COMPRESSION_SHORT_WINDOW_SELECTOR_WINDOW_SIZE, DEFAULT_ANALYSIS_HOP, DEFAULT_WINDOW_SIZE,
    EXPANSION_SHORT_WINDOW_SELECTOR_ANALYSIS_HOP,
    EXPANSION_SHORT_WINDOW_SELECTOR_MIN_CURRENT_MISSES,
    EXPANSION_SHORT_WINDOW_SELECTOR_WINDOW_SIZE,
};
use crate::transient_smear;

pub(crate) const DYNAMIC_RATIO_SEAM_SMOOTH_FRAMES: usize = 256;

/// Analysis hops of source, beyond one window, that every dynamic-ratio
/// segment must carry so the phase vocoder has overlapping frames to track.
///
/// Contract `046` freezes one window as the floor. This is stricter for two
/// measured reasons.
///
/// Pitch: a single-window segment gives the phase vocoder one analysis frame
/// and tracks the source poorly. On a `440 Hz` tone through a curve sampled
/// every `1024` frames, three extra hops leave `19.6` cents of error, eight
/// leave `2.8`.
///
/// Seam-rate modulation: segments render independently, so every join leaves an
/// envelope dip and the render modulates at the segment rate. Concealed
/// listening heard it as a secondary rhythmic pulse. Measured envelope
/// modulation at the segment period against a `0.04 dB` whole-render floor:
/// `0.545 dB` at eight extra hops, `0.268` at sixteen, `0.115` at
/// thirty-two, `0.039` at sixty-four.
///
/// Thirty-two is the balance point. Sixty-four reaches the floor but its
/// `725 ms` minimum swallows realistic tempo-ramp spans. At the retained
/// `2048/512` geometry this is `18432` source frames, `384 ms` at 48 kHz.
///
/// The modulation is inherent to independently rendered segments. `g10.039`
/// removes it by carrying renderer state across the join instead of lengthening
/// segments.
pub(crate) const MIN_DYNAMIC_RATIO_SEGMENT_EXTRA_HOPS: usize = 32;

/// Largest whole-buffer render, in output samples across all channels.
///
/// One gibibyte of [`Sample`]: roughly 93 minutes mono or 46 minutes stereo at
/// 48 kHz in a single call. Longer material is the offline chunk plan's
/// responsibility (see [`plan_offline_stretch_chunks`]). Frozen by Contract
/// `046`, 2026-07-27 addendum.
pub const MAX_OFFLINE_STRETCH_OUTPUT_SAMPLES: usize = 268_435_456;

/// Whole-buffer stretch render failure.
///
/// A backend that cannot serve a request says so instead of attempting the
/// allocation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StretchRenderError {
    /// The resumable renderer was configured outside its supported geometry.
    UnsupportedResumableConfiguration,
    /// The requested output exceeds [`MAX_OFFLINE_STRETCH_OUTPUT_SAMPLES`].
    OutputTooLarge {
        /// Output samples the request would have produced, saturated.
        requested_samples: u128,
        /// Frozen ceiling in output samples.
        maximum_samples: usize,
    },
}

/// Validate the output size one whole-buffer render would produce.
///
/// Returns the target frame count when the render fits inside
/// [`MAX_OFFLINE_STRETCH_OUTPUT_SAMPLES`].
pub(crate) fn checked_target_frames(
    source_frames: usize,
    ratio: f64,
    channels: usize,
) -> Result<usize, StretchRenderError> {
    let target_frames = (source_frames as f64 * ratio).round();
    checked_output_frames(target_frames, channels)
}

pub(crate) fn checked_output_frames(
    target_frames: f64,
    channels: usize,
) -> Result<usize, StretchRenderError> {
    let samples = target_frames * channels as f64;
    if !samples.is_finite() || samples < 0.0 || samples > MAX_OFFLINE_STRETCH_OUTPUT_SAMPLES as f64
    {
        return Err(StretchRenderError::OutputTooLarge {
            requested_samples: saturating_u128(samples),
            maximum_samples: MAX_OFFLINE_STRETCH_OUTPUT_SAMPLES,
        });
    }
    Ok(target_frames as usize)
}

pub(crate) fn saturating_u128(value: f64) -> u128 {
    if !value.is_finite() || value <= 0.0 {
        0
    } else if value >= u128::MAX as f64 {
        u128::MAX
    } else {
        value as u128
    }
}

/// Cheap fallback for sub-window inputs: linear interpolation over time
/// (this pitch-shifts, but a sub-window buffer is too short for the phase
/// vocoder to do better; documented, deterministic).
pub(crate) fn linear_time_scale(input: &[Sample], target_len: usize) -> Vec<Sample> {
    if input.len() == 1 {
        return vec![input[0]; target_len];
    }
    let step = (input.len() - 1) as f64 / (target_len.max(2) - 1) as f64;
    (0..target_len)
        .map(|index| {
            let position = index as f64 * step;
            let left = position.floor() as usize;
            let right = (left + 1).min(input.len() - 1);
            let fraction = (position - left as f64) as f32;
            input[left] + (input[right] - input[left]) * fraction
        })
        .collect()
}

pub(crate) fn linear_time_scale_interleaved_stereo(
    input: &[Sample],
    target_frames: usize,
) -> Vec<Sample> {
    let frame_count = input.len() / 2;
    let mut left = Vec::with_capacity(frame_count);
    let mut right = Vec::with_capacity(frame_count);
    for frame in input.chunks_exact(2) {
        left.push(frame[0]);
        right.push(frame[1]);
    }
    let left = linear_time_scale(&left, target_frames);
    let right = linear_time_scale(&right, target_frames);
    let out_frames = left.len().min(right.len()).min(target_frames);
    let mut output = Vec::with_capacity(target_frames * 2);
    for index in 0..out_frames {
        output.push(left[index]);
        output.push(right[index]);
    }
    output.resize(target_frames * 2, 0.0);
    output
}

pub(crate) fn sanitize_ratio(ratio: f64) -> f64 {
    if ratio.is_finite() && ratio > 0.0 {
        ratio
    } else {
        1.0
    }
}

pub(crate) fn align_to_next_grid(frame: u64, grid: u64) -> u64 {
    if grid == 0 {
        return frame;
    }
    let remainder = frame % grid;
    if remainder == 0 {
        frame
    } else {
        frame.saturating_add(grid - remainder)
    }
}

pub(crate) fn usize_to_u64(value: usize) -> u64 {
    value.try_into().unwrap_or(u64::MAX)
}

pub(crate) fn abs_diff_frames(left: u64, right: u64) -> usize {
    left.abs_diff(right).try_into().unwrap_or(usize::MAX)
}

pub(crate) fn floor_frame_to_u64(frame: f64) -> u64 {
    if !frame.is_finite() || frame <= 0.0 {
        0
    } else if frame >= u64::MAX as f64 {
        u64::MAX
    } else {
        frame.floor() as u64
    }
}

pub(crate) fn ceil_frame_to_u64(frame: f64) -> u64 {
    if !frame.is_finite() || frame <= 0.0 {
        0
    } else if frame >= u64::MAX as f64 {
        u64::MAX
    } else {
        frame.ceil() as u64
    }
}

pub(crate) fn ceil_frame_to_usize(frame: f64) -> usize {
    if !frame.is_finite() || frame <= 0.0 {
        0
    } else if frame >= usize::MAX as f64 {
        usize::MAX
    } else {
        frame.ceil() as usize
    }
}

/// Wrap a phase into `-PI..PI` by remainder.
///
/// `phase_vocoder` carries a second implementation using
/// `phase - TAU * (phase / TAU).round()`. The two are **not** interchangeable:
/// over a `-50..50` sweep at `1e-4` steps, `945158` of `1005319` values differ
/// in bits, worst delta `2.6e-6`, and at exactly `-PI` they disagree in sign,
/// this one returning `-PI` and the round form `+PI`.
///
/// Unifying them is therefore an output change, not a refactor, so `g10.038`
/// left both in place. It needs a batch that can carry a re-baseline with
/// evidence. Audit finding `A10` is refined rather than closed.
pub(crate) fn wrap_phase(phase: f32) -> f32 {
    (phase + std::f32::consts::PI).rem_euclid(std::f32::consts::TAU) - std::f32::consts::PI
}

pub(crate) fn stretch_mono_with_engine(
    input: &[Sample],
    ratio: f64,
    window_size: usize,
    analysis_hop: usize,
    engine: fn(&[Sample], usize, f64, usize, usize) -> Vec<Sample>,
) -> Result<Vec<Sample>, StretchRenderError> {
    let target_len = checked_target_frames(input.len(), ratio, 1)?;
    if input.is_empty() || target_len == 0 {
        return Ok(Vec::new());
    }
    if (ratio - 1.0).abs() < 1.0e-9 {
        return Ok(input.to_vec());
    }
    if input.len() < window_size {
        return Ok(linear_time_scale(input, target_len));
    }
    Ok(engine(input, target_len, ratio, window_size, analysis_hop))
}

pub(crate) fn stretch_to_exact_mono(
    input: &[Sample],
    target_len: usize,
    window_size: usize,
    analysis_hop: usize,
    engine: fn(&[Sample], usize, f64, usize, usize) -> Vec<Sample>,
) -> Vec<Sample> {
    if input.is_empty() || target_len == 0 {
        return Vec::new();
    }
    let ratio = target_len as f64 / input.len() as f64;
    if (ratio - 1.0).abs() < 1.0e-9 {
        let mut output = input.to_vec();
        output.resize(target_len, 0.0);
        return output;
    }
    if input.len() < window_size {
        return linear_time_scale(input, target_len);
    }
    engine(input, target_len, ratio, window_size, analysis_hop)
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

pub(crate) fn stretch_dynamic_ratio_mono_with_engine(
    input: &[Sample],
    ratio_curve: &[StretchRatioPoint],
    fallback_ratio: f64,
    window_size: usize,
    analysis_hop: usize,
    engine: fn(&[Sample], usize, f64, usize, usize) -> Vec<Sample>,
) -> Result<Vec<Sample>, StretchRenderError> {
    let segments = coalesce_short_dynamic_ratio_segments(
        dynamic_ratio_segments(input.len(), ratio_curve, sanitize_ratio(fallback_ratio)),
        min_dynamic_ratio_segment_frames(window_size, analysis_hop),
    );
    let boundaries = dynamic_ratio_segment_boundaries(&segments);
    let target_len: usize = segments.iter().map(|segment| segment.target_frames).sum();
    checked_output_frames(target_len as f64, 1)?;
    let mut output = Vec::with_capacity(target_len);
    for segment in segments {
        let rendered = stretch_to_exact_mono(
            &input[segment.start_frame..segment.end_frame],
            segment.target_frames,
            window_size,
            analysis_hop,
            engine,
        );
        output.extend(rendered);
    }
    smooth_dynamic_segment_boundaries_interleaved(
        &mut output,
        1,
        &boundaries,
        DYNAMIC_RATIO_SEAM_SMOOTH_FRAMES,
    );
    Ok(output)
}

pub(crate) fn stretch_dynamic_ratio_linked_stereo_with_engine(
    input: &[Sample],
    ratio_curve: &[StretchRatioPoint],
    fallback_ratio: f64,
    window_size: usize,
    analysis_hop: usize,
) -> Result<Vec<Sample>, StretchRenderError> {
    let frame_count = input.len() / 2;
    let even_input = &input[..frame_count * 2];
    let segments = coalesce_short_dynamic_ratio_segments(
        dynamic_ratio_segments(frame_count, ratio_curve, sanitize_ratio(fallback_ratio)),
        min_dynamic_ratio_segment_frames(window_size, analysis_hop),
    );
    let boundaries = dynamic_ratio_segment_boundaries(&segments);
    let target_frames: usize = segments.iter().map(|segment| segment.target_frames).sum();
    checked_output_frames(target_frames as f64, 2)?;
    let mut output = Vec::with_capacity(target_frames * 2);
    for segment in segments {
        let start = segment.start_frame * 2;
        let end = segment.end_frame * 2;
        let rendered = stretch_to_exact_linked_stereo(
            &even_input[start..end],
            segment.target_frames,
            window_size,
            analysis_hop,
        );
        output.extend(rendered);
    }
    smooth_dynamic_segment_boundaries_interleaved(
        &mut output,
        2,
        &boundaries,
        DYNAMIC_RATIO_SEAM_SMOOTH_FRAMES,
    );
    Ok(output)
}

pub(crate) fn stretch_to_exact_linked_stereo(
    input: &[Sample],
    target_frames: usize,
    window_size: usize,
    analysis_hop: usize,
) -> Vec<Sample> {
    let frame_count = input.len() / 2;
    if frame_count == 0 || target_frames == 0 {
        return Vec::new();
    }
    let ratio = target_frames as f64 / frame_count as f64;
    if (ratio - 1.0).abs() < 1.0e-9 {
        let mut output = input[..frame_count * 2].to_vec();
        output.resize(target_frames * 2, 0.0);
        return output;
    }
    if frame_count < window_size {
        return linear_time_scale_interleaved_stereo(&input[..frame_count * 2], target_frames);
    }
    transient_reset_phase_vocoder_linked_stereo(
        &input[..frame_count * 2],
        target_frames,
        ratio,
        window_size,
        analysis_hop,
    )
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct DynamicRatioSegment {
    pub(crate) start_frame: usize,
    pub(crate) end_frame: usize,
    pub(crate) target_frames: usize,
    pub(crate) ratio: f64,
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

pub(crate) fn pitch_shift_mono_to_nominal_rate(
    input: &[Sample],
    sample_rate: SampleRate,
    semitones: f64,
) -> Vec<Sample> {
    let Some(config) = pitch_shift_resample_config(sample_rate, semitones) else {
        return input.to_vec();
    };
    resample_mono(config, input)
}

pub(crate) fn pitch_shift_interleaved_stereo_to_nominal_rate(
    input: &[Sample],
    sample_rate: SampleRate,
    semitones: f64,
) -> Vec<Sample> {
    let frame_count = input.len() / 2;
    let Some(config) = pitch_shift_resample_config(sample_rate, semitones) else {
        return input[..frame_count * 2].to_vec();
    };

    let mut mid = Vec::with_capacity(frame_count);
    let mut side = Vec::with_capacity(frame_count);
    for frame in input[..frame_count * 2].chunks_exact(2) {
        let left = frame[0];
        let right = frame[1];
        mid.push((left + right) * 0.5);
        side.push((left - right) * 0.5);
    }
    let mid = resample_mono(config, &mid);
    let side = resample_mono(config, &side);
    let out_frames = mid.len().min(side.len());
    let mut output = Vec::with_capacity(out_frames * 2);
    for index in 0..out_frames {
        output.push(mid[index] + side[index]);
        output.push(mid[index] - side[index]);
    }
    output
}

pub(crate) fn pitch_shift_resample_config(
    sample_rate: SampleRate,
    semitones: f64,
) -> Option<ResampleConfig> {
    if sample_rate.0 == 0 || !semitones.is_finite() || semitones.abs() < 1.0e-9 {
        return None;
    }
    let factor = 2.0f64.powf(semitones / 12.0);
    if !factor.is_finite() || factor <= 0.0 {
        return None;
    }
    let virtual_input_rate =
        ((sample_rate.0 as f64 * factor).round()).clamp(1.0, u32::MAX as f64) as u32;
    Some(ResampleConfig::new(
        SampleRate(virtual_input_rate),
        sample_rate,
        ResampleQuality::BandLimited,
    ))
}

pub(crate) fn should_select_compression_short_window(
    input: &[Sample],
    current_output: &[Sample],
    ratio: f64,
) -> bool {
    if ratio >= 1.0 || input.is_empty() || current_output.is_empty() {
        return false;
    }

    let current_smear = transient_smear::measure_selector_transient_smear(
        input,
        current_output,
        ratio,
        COMPRESSION_SHORT_WINDOW_SELECTOR_WINDOW_SIZE,
        COMPRESSION_SHORT_WINDOW_SELECTOR_ANALYSIS_HOP,
    );
    current_smear.missed_transients >= COMPRESSION_SHORT_WINDOW_SELECTOR_MIN_CURRENT_MISSES
        || current_smear.max_smear_frames
            >= COMPRESSION_SHORT_WINDOW_SELECTOR_MIN_CURRENT_SMEAR_FRAMES
}

pub(crate) fn should_select_compression_short_window_interleaved(
    input: &[Sample],
    current_output: &[Sample],
    ratio: f64,
) -> bool {
    let input_mono = downmix_interleaved_stereo_to_mono(input);
    let output_mono = downmix_interleaved_stereo_to_mono(current_output);
    should_select_compression_short_window(&input_mono, &output_mono, ratio)
}

pub(crate) fn should_select_expansion_short_window(
    input: &[Sample],
    current_output: &[Sample],
    ratio: f64,
) -> bool {
    if ratio <= 1.0 || input.is_empty() || current_output.is_empty() {
        return false;
    }

    // Source transients are detected once and reused for both comparisons.
    // The current-output and draft-baseline measurements previously each
    // re-detected them from the same input with the same policy and geometry.
    let input_events = transient_smear::detect_stretch_transients_with_policy(
        input,
        EXPANSION_SHORT_WINDOW_SELECTOR_WINDOW_SIZE,
        EXPANSION_SHORT_WINDOW_SELECTOR_ANALYSIS_HOP,
        transient_smear::StretchTransientDetectorPolicy::production(),
    );
    let current_smear = transient_smear::measure_selector_transient_smear_with_input_events(
        input,
        &input_events,
        current_output,
        ratio,
        EXPANSION_SHORT_WINDOW_SELECTOR_WINDOW_SIZE,
        EXPANSION_SHORT_WINDOW_SELECTOR_ANALYSIS_HOP,
    );
    if current_smear.missed_transients >= EXPANSION_SHORT_WINDOW_SELECTOR_MIN_CURRENT_MISSES {
        return true;
    }

    let mut draft = PhaseVocoderStretcher::new(ratio);
    let Ok(draft_output) = draft.stretch_mono(input) else {
        // The default render already succeeded at this size, so a draft render
        // of the same input cannot exceed the bound. Stay on the current path
        // rather than switching on missing evidence.
        return false;
    };
    let draft_smear = transient_smear::measure_selector_transient_smear_with_input_events(
        input,
        &input_events,
        &draft_output,
        ratio,
        EXPANSION_SHORT_WINDOW_SELECTOR_WINDOW_SIZE,
        EXPANSION_SHORT_WINDOW_SELECTOR_ANALYSIS_HOP,
    );
    metric_worsened(current_smear.max_smear_frames, draft_smear.max_smear_frames)
}

pub(crate) fn should_select_expansion_short_window_interleaved(
    input: &[Sample],
    current_output: &[Sample],
    ratio: f64,
) -> bool {
    let input_mono = downmix_interleaved_stereo_to_mono(input);
    let output_mono = downmix_interleaved_stereo_to_mono(current_output);
    should_select_expansion_short_window(&input_mono, &output_mono, ratio)
}

pub(crate) fn metric_worsened(candidate: f64, production: f64) -> bool {
    if candidate.is_finite() && production.is_finite() {
        candidate > production
    } else {
        !candidate.is_finite() && production.is_finite()
    }
}

pub(crate) fn short_window_size_for_path(path: OfflineHighQualityPath) -> usize {
    match path {
        OfflineHighQualityPath::Default => DEFAULT_WINDOW_SIZE,
        OfflineHighQualityPath::CompressionShortWindowSelector => {
            COMPRESSION_SHORT_WINDOW_SELECTOR_WINDOW_SIZE
        }
        OfflineHighQualityPath::ExpansionShortWindowSelector => {
            EXPANSION_SHORT_WINDOW_SELECTOR_WINDOW_SIZE
        }
    }
}

pub(crate) fn short_window_analysis_hop_for_path(path: OfflineHighQualityPath) -> usize {
    match path {
        OfflineHighQualityPath::Default => DEFAULT_ANALYSIS_HOP,
        OfflineHighQualityPath::CompressionShortWindowSelector => {
            COMPRESSION_SHORT_WINDOW_SELECTOR_ANALYSIS_HOP
        }
        OfflineHighQualityPath::ExpansionShortWindowSelector => {
            EXPANSION_SHORT_WINDOW_SELECTOR_ANALYSIS_HOP
        }
    }
}

pub(crate) fn downmix_interleaved_stereo_to_mono(samples: &[Sample]) -> Vec<Sample> {
    samples
        .chunks_exact(2)
        .map(|frame| (frame[0] + frame[1]) * 0.5)
        .collect()
}
