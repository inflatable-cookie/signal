//! Spectral analysis helpers for chroma and spectral centroid extraction.
//!
//! This module provides analysis functions that operate on spectrograms to
//! extract musical and timbral features.

use signal_primitives::SampleRate;

/// Fractional width of one semitone: 2^(1/24) − 2^(−1/24).
///
/// Used to compute the minimum frequency at which one FFT bin fits within
/// a single semitone: `min_freq = bin_spacing / SEMITONE_WIDTH_RATIO`.
pub const SEMITONE_WIDTH_RATIO: f32 = 0.057_77;

/// Map a frequency (already range-checked by the caller) to its nearest
/// pitch class (0 = C … 11 = B).
pub fn frequency_to_pitch_class_with_reference(frequency: f32, reference_hz: f32) -> usize {
    let midi = 69.0 + 12.0 * (frequency / reference_hz).log2();
    let rounded = midi.round() as i32;
    rounded.rem_euclid(12) as usize
}

/// Normalize an array of 12 values to sum to 1.0.
pub fn normalize_array(values: &mut [f32; 12]) {
    let sum = values.iter().copied().sum::<f32>();
    if sum > 0.0 {
        for value in values {
            *value /= sum;
        }
    }
}

/// Compute the chroma (12-bin pitch-class profile) from spectrogram magnitudes.
///
/// Returns a normalized 12-element array where each bin represents the
/// relative strength of one pitch class (C, C#, D, ..., B).
pub fn compute_chroma(
    frames: &[crate::SpectrumFrame],
    sample_rate: SampleRate,
    window_size: usize,
    reference_hz: f32,
) -> [f32; 12] {
    let mut chroma = [0.0; 12];
    if frames.is_empty() || window_size == 0 || sample_rate.0 == 0 {
        return chroma;
    }

    // Compute the minimum frequency at which FFT bins are narrow enough
    // for reliable pitch-class assignment.  Below this threshold the bin
    // spacing exceeds one semitone, so a single note's energy can land
    // in the wrong pitch class entirely (e.g. B1 at 61.7 Hz splits
    // between A# and C bins with no bin mapping to B).  Combined with
    // 1/f weighting these mis-mapped sub-bass bins would dominate the
    // chroma and shift the detected key by a semitone.
    let bin_spacing = sample_rate.0 as f32 / window_size as f32;
    let min_frequency = bin_spacing / SEMITONE_WIDTH_RATIO;

    for frame in frames {
        for (bin_index, magnitude) in frame.magnitudes.iter().enumerate().skip(1) {
            let frequency = crate::bin_frequency(bin_index, sample_rate, window_size);
            if frequency < min_frequency || frequency > 5_000.0 {
                continue;
            }
            let pitch_class = frequency_to_pitch_class_with_reference(frequency, reference_hz);
            chroma[pitch_class] += *magnitude / frequency;
        }
    }

    normalize_array(&mut chroma);
    chroma
}

/// Compute the median spectral centroid from spectrogram magnitudes.
///
/// The spectral centroid is the magnitude-weighted mean of FFT bin
/// frequencies — a single number capturing the "centre of mass" of the
/// spectrum.  It is a standard timbral brightness indicator.
///
/// Returns `0.0` for empty input or when all magnitudes are zero.
pub fn compute_spectral_centroid(
    frames: &[crate::SpectrumFrame],
    sample_rate: SampleRate,
    window_size: usize,
) -> f32 {
    if frames.is_empty() || window_size == 0 || sample_rate.0 == 0 {
        return 0.0;
    }

    let mut frame_centroids = Vec::with_capacity(frames.len());

    for frame in frames {
        let mut weighted_sum = 0.0f32;
        let mut magnitude_sum = 0.0f32;

        for (bin_index, &magnitude) in frame.magnitudes.iter().enumerate() {
            let frequency = crate::bin_frequency(bin_index, sample_rate, window_size);
            weighted_sum += frequency * magnitude;
            magnitude_sum += magnitude;
        }

        if magnitude_sum > 0.0 {
            frame_centroids.push(weighted_sum / magnitude_sum);
        }
    }

    if frame_centroids.is_empty() {
        return 0.0;
    }

    // Median for robustness against transient outliers.
    frame_centroids.sort_by(|a, b| a.partial_cmp(b).unwrap_or(core::cmp::Ordering::Equal));
    frame_centroids[frame_centroids.len() / 2]
}
