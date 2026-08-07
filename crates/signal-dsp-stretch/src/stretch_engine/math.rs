//! Ratio sanitization, frame math, and cheap time-scale fallbacks.

use signal_primitives::Sample;

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
