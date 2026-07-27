//! Shared windowing and STFT support for the evidence metrics.
//!
//! `tonal_texture` and `formant_boundary` each carried a private copy of
//! `window_fits`, `hann_window`, and the windowed-magnitude extraction, and
//! several modules built their own planner. One copy lives here so a change to
//! the analysis window or the magnitude convention lands in one place.
//!
//! Extraction is bit-exact: callers keep their own post-processing, including
//! which bins they read.

use std::sync::Arc;

use rustfft::{num_complex::Complex32, Fft, FftPlanner};

use crate::Sample;

/// Whether a centered analysis window of `size` lies fully inside `sample_len`.
pub(crate) fn window_fits(sample_len: usize, center: usize, size: usize) -> bool {
    let radius = size / 2;
    center >= radius && center.saturating_add(radius) <= sample_len
}

/// Symmetric Hann window used by every evidence metric.
pub(crate) fn hann_window(size: usize) -> Vec<f32> {
    (0..size)
        .map(|index| {
            let phase = std::f32::consts::TAU * index as f32 / (size - 1) as f32;
            0.5 - 0.5 * phase.cos()
        })
        .collect()
}

/// Forward plan plus its matching window, so a caller sets both up together.
pub(crate) fn plan_forward_analysis(size: usize) -> (Arc<dyn Fft<f32>>, Vec<f32>) {
    let mut planner = FftPlanner::<f32>::new();
    (planner.plan_fft_forward(size), hann_window(size))
}

/// Magnitudes of one centered, windowed analysis frame, bins `0..=size / 2`.
///
/// Callers select the bins they need; this returns the full non-negative half
/// so no caller has to repeat the transform.
pub(crate) fn windowed_magnitudes(
    samples: &[Sample],
    center: usize,
    size: usize,
    window: &[f32],
    fft: Arc<dyn Fft<f32>>,
) -> Vec<f64> {
    let start = center - size / 2;
    let mut buffer = vec![Complex32::new(0.0, 0.0); size];
    for index in 0..size {
        buffer[index].re = samples[start + index] * window[index];
    }
    fft.process(&mut buffer);
    buffer
        .iter()
        .take(size / 2 + 1)
        .map(|value| value.norm() as f64)
        .collect()
}
