//! Whole-buffer stretch render entry points.

use signal_primitives::Sample;

use crate::cache_identity::StretchRatioPoint;
use crate::phase_vocoder::transient_reset_phase_vocoder_linked_stereo;

use super::dynamic_ratio::{
    coalesce_short_dynamic_ratio_segments, dynamic_ratio_segment_boundaries,
    dynamic_ratio_segments, min_dynamic_ratio_segment_frames,
    smooth_dynamic_segment_boundaries_interleaved,
};
use super::limits::{
    checked_output_frames, checked_target_frames, StretchRenderError,
    DYNAMIC_RATIO_SEAM_SMOOTH_FRAMES,
};
use super::math::{linear_time_scale, linear_time_scale_interleaved_stereo, sanitize_ratio};

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
