use rustfft::{num_complex::Complex32, FftPlanner};
use signal_primitives::Sample;

use super::types::StretchTransientEvent;

#[derive(Clone, Copy, Debug)]
pub(in crate::transient_smear) struct TransientFrameFeature {
    pub(in crate::transient_smear) frame_index: usize,
    pub(in crate::transient_smear) energy: f64,
    pub(in crate::transient_smear) spectral_flux: f64,
}

pub(super) fn transient_frame_features(
    samples: &[Sample],
    window_size: usize,
    hop_size: usize,
) -> Vec<TransientFrameFeature> {
    let bins = window_size / 2 + 1;
    let window: Vec<f32> = (0..window_size)
        .map(|index| 0.5 - 0.5 * (std::f32::consts::TAU * index as f32 / window_size as f32).cos())
        .collect();
    let mut planner = FftPlanner::<f32>::new();
    let forward = planner.plan_fft_forward(window_size);
    let mut buffer = vec![Complex32::new(0.0, 0.0); window_size];
    let mut previous_magnitudes = vec![0.0f32; bins];
    let mut magnitudes = vec![0.0f32; bins];
    let mut features = Vec::new();

    for start in (0..=samples.len() - window_size).step_by(hop_size) {
        let mut energy = 0.0f64;
        for (slot, (sample, weight)) in buffer.iter_mut().zip(
            samples[start..start + window_size]
                .iter()
                .zip(window.iter()),
        ) {
            let windowed = sample * weight;
            energy += (windowed * windowed) as f64;
            *slot = Complex32::new(windowed, 0.0);
        }
        forward.process(&mut buffer);

        let mut flux = 0.0f64;
        for bin in 0..bins {
            let magnitude = buffer[bin].norm();
            magnitudes[bin] = magnitude;
            flux += (magnitude - previous_magnitudes[bin]).max(0.0) as f64;
        }
        previous_magnitudes.copy_from_slice(&magnitudes);

        features.push(TransientFrameFeature {
            frame_index: start,
            energy: energy / window_size as f64,
            spectral_flux: flux / bins as f64,
        });
    }

    features
}

pub(super) fn mean_plus_stddev(values: &[f64]) -> f64 {
    if values.is_empty() {
        return f64::NAN;
    }
    let mean = values.iter().sum::<f64>() / values.len() as f64;
    let variance = values
        .iter()
        .map(|value| {
            let delta = value - mean;
            delta * delta
        })
        .sum::<f64>()
        / values.len() as f64;
    mean + variance.sqrt()
}

pub(super) fn merge_nearby_transients(
    events: Vec<StretchTransientEvent>,
    merge_distance_frames: usize,
) -> Vec<StretchTransientEvent> {
    let mut merged = Vec::<StretchTransientEvent>::new();
    for event in events {
        if let Some(last) = merged.last_mut() {
            if event.frame_index.saturating_sub(last.frame_index) <= merge_distance_frames {
                if event.combined_score > last.combined_score {
                    *last = event;
                }
                continue;
            }
        }
        merged.push(event);
    }
    merged
}

pub(super) fn nearest_transient(
    events: &[StretchTransientEvent],
    expected_frame: f64,
    tolerance_frames: f64,
) -> Option<StretchTransientEvent> {
    events
        .iter()
        .copied()
        .filter_map(|event| {
            let distance = (event.frame_index as f64 - expected_frame).abs();
            if distance <= tolerance_frames {
                Some((distance, event))
            } else {
                None
            }
        })
        .min_by(|left, right| left.0.total_cmp(&right.0))
        .map(|(_, event)| event)
}

pub(super) fn transient_attack_width(
    samples: &[Sample],
    event_frame: usize,
    search_radius: usize,
) -> f64 {
    if samples.is_empty() {
        return f64::NAN;
    }

    let start = event_frame.saturating_sub(search_radius);
    let end = (event_frame + search_radius).min(samples.len().saturating_sub(1));
    if start >= end {
        return f64::NAN;
    }

    let mut peak_index = start;
    let mut peak = 0.0f32;
    for (offset, sample) in samples[start..=end].iter().enumerate() {
        let magnitude = sample.abs();
        if magnitude > peak {
            peak = magnitude;
            peak_index = start + offset;
        }
    }
    if peak <= 1.0e-6 {
        return f64::NAN;
    }

    let threshold = peak * 0.5;
    let mut left = peak_index;
    while left > start && samples[left - 1].abs() >= threshold {
        left -= 1;
    }
    let mut right = peak_index;
    while right < end && samples[right + 1].abs() >= threshold {
        right += 1;
    }

    (right - left + 1) as f64
}
