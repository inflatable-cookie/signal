use std::sync::Arc;

use rustfft::{num_complex::Complex32, Fft};

use crate::Sample;

use super::{SIDEBAND_FLOOR_RATIO, SPECTRAL_WINDOW_SIZE};

pub(super) fn window_fits(sample_len: usize, center: usize, size: usize) -> bool {
    let radius = size / 2;
    center >= radius && center.saturating_add(radius) <= sample_len
}

pub(super) fn normalized_spectrum(
    samples: &[Sample],
    center: usize,
    window: &[f32],
    fft: Arc<dyn Fft<f32>>,
) -> Vec<f64> {
    let start = center - SPECTRAL_WINDOW_SIZE / 2;
    let mut buffer = vec![Complex32::new(0.0, 0.0); SPECTRAL_WINDOW_SIZE];
    for index in 0..SPECTRAL_WINDOW_SIZE {
        buffer[index].re = samples[start + index] * window[index];
    }
    fft.process(&mut buffer);
    let mut magnitudes = buffer
        .iter()
        .take(SPECTRAL_WINDOW_SIZE / 2 + 1)
        .skip(1)
        .map(|value| value.norm() as f64)
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

pub(super) fn hann_window(size: usize) -> Vec<f32> {
    (0..size)
        .map(|index| {
            let phase = std::f32::consts::TAU * index as f32 / (size - 1) as f32;
            0.5 - 0.5 * phase.cos()
        })
        .collect()
}
