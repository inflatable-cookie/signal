use signal_analysis::Confidence;
use signal_dsp_spectral::StftConfig;
use signal_primitives::{Sample, SampleRate};

use crate::types::DynamicsDescriptorPack;

mod shape;

pub(crate) use shape::compute_temporal_shape_pack;

const TRANSIENT_PEAK_MIN_SPACING_FRAMES: usize = 8;

pub(crate) fn spectral_flux_envelope(spectrogram: &signal_dsp_spectral::Spectrogram) -> Vec<f32> {
    if spectrogram.is_empty() {
        return Vec::new();
    }

    let mut flux = Vec::with_capacity(spectrogram.frames.len());
    let mut previous: Option<&[f32]> = None;

    for frame in &spectrogram.frames {
        let current = frame.magnitudes.as_slice();
        let value = if let Some(last) = previous {
            current
                .iter()
                .zip(last.iter())
                .map(|(now, then)| (now - then).max(0.0))
                .sum()
        } else {
            0.0
        };
        flux.push(value);
        previous = Some(current);
    }

    flux
}

pub(crate) fn detect_peak_indices(values: &[f32], threshold_multiplier: f32) -> Vec<usize> {
    if values.len() < 3 {
        return Vec::new();
    }

    let mean = values.iter().copied().sum::<f32>() / values.len() as f32;
    let threshold = mean * threshold_multiplier;
    let mut peaks = Vec::new();

    for index in 1..values.len().saturating_sub(1) {
        if values[index] > threshold
            && values[index] > values[index - 1]
            && values[index] > values[index + 1]
        {
            if let Some(last_peak) = peaks.last_mut() {
                if index - *last_peak <= TRANSIENT_PEAK_MIN_SPACING_FRAMES {
                    if values[index] > values[*last_peak] {
                        *last_peak = index;
                    }
                    continue;
                }
            }
            peaks.push(index);
        }
    }

    peaks
}

pub(crate) fn compute_event_density(event_count: usize, duration_seconds: f32) -> f32 {
    if duration_seconds <= 0.0 {
        0.0
    } else {
        event_count as f32 / duration_seconds
    }
}

pub(crate) fn frame_rms_envelope(samples: &[Sample], config: StftConfig) -> Vec<f32> {
    let window_size = config.window_size.0;
    let hop_size = config.hop_size.0.max(1);
    if samples.is_empty() || window_size == 0 {
        return Vec::new();
    }

    let mut envelope = Vec::new();
    let mut start = 0usize;

    while start + window_size <= samples.len() {
        envelope.push(compute_rms(&samples[start..start + window_size]));
        start += hop_size;
    }

    let overlap_tail = window_size.saturating_sub(hop_size);
    if start == 0 || (start < samples.len() && samples.len() - start > overlap_tail) {
        envelope.push(compute_rms(&samples[start..]));
    }

    envelope
}

pub(crate) fn compute_zcr(samples: &[Sample], sample_rate: SampleRate) -> f32 {
    if samples.len() < 2 || sample_rate.0 == 0 {
        return 0.0;
    }

    let mut crossings = 0u64;
    for pair in samples.windows(2) {
        if (pair[0] >= 0.0) != (pair[1] >= 0.0) {
            crossings += 1;
        }
    }

    let duration_seconds = samples.len() as f64 / sample_rate.0 as f64;
    (crossings as f64 / duration_seconds) as f32
}

pub(crate) fn compute_rms(samples: &[Sample]) -> f32 {
    if samples.is_empty() {
        return 0.0;
    }

    let sum_squares: f64 = samples
        .iter()
        .map(|&sample| (sample as f64) * (sample as f64))
        .sum();
    let rms = (sum_squares / samples.len() as f64).sqrt() as f32;
    rms.clamp(0.0, 1.0)
}

pub(crate) fn compute_peak_amplitude(samples: &[Sample]) -> f32 {
    samples
        .iter()
        .fold(0.0f32, |peak, &sample| peak.max(sample.abs()))
}

pub(crate) fn compute_transient_density(
    samples: &[Sample],
    sample_rate: SampleRate,
    duration_seconds: f32,
) -> f32 {
    const SLOPE_THRESHOLD: f32 = 0.12;
    const MIN_SPACING_DIVISOR: u32 = 20;

    if samples.len() < 2 || sample_rate.0 == 0 || duration_seconds <= 0.0 {
        return 0.0;
    }

    let min_spacing = (sample_rate.0 / MIN_SPACING_DIVISOR).max(1) as usize;
    let mut count = 0u32;
    let mut samples_since_last = min_spacing;

    for index in 1..samples.len() {
        let slope = (samples[index] - samples[index - 1]).abs();
        samples_since_last += 1;
        if slope >= SLOPE_THRESHOLD && samples_since_last >= min_spacing {
            count += 1;
            samples_since_last = 0;
        }
    }

    count as f32 / duration_seconds
}

pub(crate) fn compute_sustain_ratio(samples: &[Sample]) -> f32 {
    const SILENCE_THRESHOLD: f32 = 0.02;

    if samples.is_empty() {
        return 0.0;
    }

    let sustained = samples
        .iter()
        .filter(|&&sample| sample.abs() >= SILENCE_THRESHOLD)
        .count();

    sustained as f32 / samples.len() as f32
}

pub(crate) fn compute_dynamics_pack(samples: &[Sample]) -> DynamicsDescriptorPack {
    let rms_energy = compute_rms(samples);
    let peak_amplitude = compute_peak_amplitude(samples);
    DynamicsDescriptorPack {
        rms_energy,
        peak_amplitude,
        dynamic_range: (peak_amplitude - rms_energy).max(0.0),
    }
}

pub(crate) fn character_confidence(sample_rate: SampleRate, sample_count: usize) -> Confidence {
    if sample_rate.0 == 0 {
        return Confidence::new(0.0);
    }

    let duration_seconds = sample_count as f32 / sample_rate.0 as f32;
    Confidence::new(duration_seconds / 10.0)
}
