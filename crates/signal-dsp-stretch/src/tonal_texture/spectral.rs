use std::sync::Arc;

use rustfft::Fft;

use crate::spectral_support::windowed_magnitudes;
use crate::Sample;

use super::{SIDEBAND_FLOOR_RATIO, SPECTRAL_WINDOW_SIZE};

pub(super) use crate::spectral_support::window_fits;

pub(super) fn normalized_spectrum(
    samples: &[Sample],
    center: usize,
    window: &[f32],
    fft: Arc<dyn Fft<f32>>,
) -> Vec<f64> {
    // Bin zero is dropped: this metric compares spectral shape, not DC.
    let mut magnitudes = windowed_magnitudes(samples, center, SPECTRAL_WINDOW_SIZE, window, fft)
        .into_iter()
        .skip(1)
        .collect::<Vec<_>>();
    let sum = magnitudes.iter().sum::<f64>();
    for magnitude in &mut magnitudes {
        *magnitude /= sum + 1.0e-20;
    }
    magnitudes
}

pub(super) fn normalized_spectral_distance(left: &[f64], right: &[f64]) -> f64 {
    left.iter()
        .zip(right)
        .map(|(a, b)| (a - b).abs())
        .sum::<f64>()
        * 0.5
}

pub(super) fn added_sideband_ratio(source: &[f64], output: &[f64]) -> f64 {
    let floor = source.iter().copied().fold(0.0, f64::max) * SIDEBAND_FLOOR_RATIO;
    source
        .iter()
        .zip(output)
        .filter(|(source_bin, _)| **source_bin <= floor)
        .map(|(source_bin, output_bin)| (output_bin - source_bin).max(0.0))
        .sum()
}
