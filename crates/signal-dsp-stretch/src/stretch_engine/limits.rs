//! Output-size limits and whole-buffer stretch render errors.

use super::math::saturating_u128;

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
/// One gibibyte of `Sample`: roughly 93 minutes mono or 46 minutes stereo at
/// 48 kHz in a single call. Longer material is the offline chunk plan's
/// responsibility (see `plan_offline_stretch_chunks`). Frozen by Contract
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
