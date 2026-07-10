use std::sync::Arc;

use rustfft::{num_complex::Complex32, Fft};

use crate::Sample;

use super::{FORMANT_HIGH_HZ, FORMANT_LOW_HZ, FORMANT_SMOOTHING_HZ, SPECTRAL_WINDOW_SIZE};

pub(super) fn window_fits(sample_len: usize, center: usize, size: usize) -> bool {
    let radius = size / 2;
    center >= radius && center.saturating_add(radius) <= sample_len
}

pub(super) fn smoothed_spectral_envelope(
    samples: &[Sample],
    center: usize,
    window: &[f32],
    fft: Arc<dyn Fft<f32>>,
    sample_rate_hz: u32,
) -> Vec<f64> {
    let start = center - SPECTRAL_WINDOW_SIZE / 2;
    let mut buffer = vec![Complex32::new(0.0, 0.0); SPECTRAL_WINDOW_SIZE];
    for index in 0..SPECTRAL_WINDOW_SIZE {
        buffer[index].re = samples[start + index] * window[index];
    }
    fft.process(&mut buffer);
    let magnitudes = buffer
        .iter()
        .take(SPECTRAL_WINDOW_SIZE / 2 + 1)
        .map(|value| value.norm() as f64)
        .collect::<Vec<_>>();
    let bin_hz = sample_rate_hz as f64 / SPECTRAL_WINDOW_SIZE as f64;
    let first_bin = (FORMANT_LOW_HZ / bin_hz).ceil() as usize;
    let last_bin = (FORMANT_HIGH_HZ.min(sample_rate_hz as f64 * 0.5) / bin_hz).floor() as usize;
    let smoothing_radius = ((FORMANT_SMOOTHING_HZ * 0.5 / bin_hz).round() as usize).max(1);
    let mut envelope = (first_bin..=last_bin)
        .map(|bin| {
            let start = bin.saturating_sub(smoothing_radius).max(first_bin);
            let end = (bin + smoothing_radius + 1).min(last_bin + 1);
            magnitudes[start..end].iter().sum::<f64>() / (end - start) as f64
        })
        .collect::<Vec<_>>();
    let sum = envelope.iter().sum::<f64>();
    for value in &mut envelope {
        *value /= sum + 1.0e-20;
    }
    envelope
}

pub(super) fn envelope_residual(source: &[f64], output: &[f64]) -> f64 {
    source
        .iter()
        .zip(output)
        .map(|(left, right)| (left - right).abs())
        .sum::<f64>()
        * 0.5
}

pub(super) fn envelope_centroid_hz(envelope: &[f64], sample_rate_hz: u32) -> f64 {
    let bin_hz = sample_rate_hz as f64 / SPECTRAL_WINDOW_SIZE as f64;
    let first_bin = (FORMANT_LOW_HZ / bin_hz).ceil() as usize;
    envelope
        .iter()
        .enumerate()
        .map(|(index, weight)| (first_bin + index) as f64 * bin_hz * weight)
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
