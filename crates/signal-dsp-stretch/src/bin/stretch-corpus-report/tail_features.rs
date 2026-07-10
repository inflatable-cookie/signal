use rustfft::{num_complex::Complex32, FftPlanner};

use super::quoted_report_field;

const TAIL_FEATURE_WINDOW_FRAMES: usize = 2_048;
const MOVEMENT_WINDOW_FRAMES: usize = TAIL_FEATURE_WINDOW_FRAMES / 2;
const CORRECTION_WINDOW_FRAMES: usize = 256;
const LOW_BAND_LIMIT_HZ: f64 = 250.0;

pub(crate) fn measure_tail_spectral_centroid_hz(sample_rate_hz: u32, current: &[f32]) -> f64 {
    let tail = tail_window(current, TAIL_FEATURE_WINDOW_FRAMES);
    let spectrum = magnitude_spectrum(&tail);
    spectral_summary(&spectrum, sample_rate_hz).1
}

pub(super) fn format_tail_local_feature_line(
    case_id: &str,
    source_path: &str,
    ratio: f64,
    sample_rate_hz: u32,
    current: &[f32],
    additive: &[f32],
    multiplicative: &[f32],
) -> String {
    let features = measure_tail_local_features(sample_rate_hz, current, additive, multiplicative);
    format!(
        "external_benchmark_tail_local_features case={} source={} ratio={:.6} sample_rate={} window_frames={} dc_offset_ratio={:.9} low_band_energy_share={:.9} spectral_centroid_hz={:.6} short_spectral_movement={:.9} zero_crossing_distance_frames={} additive_correction_energy_ratio={:.9} multiplicative_correction_energy_ratio={:.9}",
        case_id,
        quoted_report_field(source_path),
        ratio,
        sample_rate_hz,
        TAIL_FEATURE_WINDOW_FRAMES,
        features.dc_offset_ratio,
        features.low_band_energy_share,
        features.spectral_centroid_hz,
        features.short_spectral_movement,
        features.zero_crossing_distance_frames,
        features.additive_correction_energy_ratio,
        features.multiplicative_correction_energy_ratio,
    )
}

#[derive(Clone, Copy, Debug)]
struct TailLocalFeatures {
    dc_offset_ratio: f64,
    low_band_energy_share: f64,
    spectral_centroid_hz: f64,
    short_spectral_movement: f64,
    zero_crossing_distance_frames: usize,
    additive_correction_energy_ratio: f64,
    multiplicative_correction_energy_ratio: f64,
}

fn measure_tail_local_features(
    sample_rate_hz: u32,
    current: &[f32],
    additive: &[f32],
    multiplicative: &[f32],
) -> TailLocalFeatures {
    let tail = tail_window(current, TAIL_FEATURE_WINDOW_FRAMES);
    let mean = tail.iter().map(|sample| *sample as f64).sum::<f64>() / tail.len() as f64;
    let rms = (tail
        .iter()
        .map(|sample| (*sample as f64) * (*sample as f64))
        .sum::<f64>()
        / tail.len() as f64)
        .sqrt();
    let spectrum = magnitude_spectrum(&tail);
    let (low_band_energy_share, spectral_centroid_hz) = spectral_summary(&spectrum, sample_rate_hz);
    let movement_start = tail.len() - 2 * MOVEMENT_WINDOW_FRAMES;
    let previous =
        magnitude_spectrum(&tail[movement_start..movement_start + MOVEMENT_WINDOW_FRAMES]);
    let final_window = magnitude_spectrum(&tail[tail.len() - MOVEMENT_WINDOW_FRAMES..]);

    TailLocalFeatures {
        dc_offset_ratio: if rms > 0.0 { mean.abs() / rms } else { 0.0 },
        low_band_energy_share,
        spectral_centroid_hz,
        short_spectral_movement: normalized_spectral_movement(&previous, &final_window),
        zero_crossing_distance_frames: zero_crossing_distance(&tail),
        additive_correction_energy_ratio: correction_energy_ratio(current, additive),
        multiplicative_correction_energy_ratio: correction_energy_ratio(current, multiplicative),
    }
}

fn tail_window(samples: &[f32], frame_count: usize) -> Vec<f32> {
    let mut window = vec![0.0; frame_count];
    let copied = frame_count.min(samples.len());
    window[frame_count - copied..].copy_from_slice(&samples[samples.len() - copied..]);
    window
}

fn magnitude_spectrum(samples: &[f32]) -> Vec<f64> {
    let mut bins = samples
        .iter()
        .enumerate()
        .map(|(index, sample)| {
            let window = 0.5
                - 0.5 * (std::f32::consts::TAU * index as f32 / (samples.len() - 1) as f32).cos();
            Complex32::new(*sample * window, 0.0)
        })
        .collect::<Vec<_>>();
    let mut planner = FftPlanner::<f32>::new();
    planner.plan_fft_forward(samples.len()).process(&mut bins);
    bins[..=samples.len() / 2]
        .iter()
        .map(|bin| bin.norm() as f64)
        .collect()
}

fn spectral_summary(spectrum: &[f64], sample_rate_hz: u32) -> (f64, f64) {
    let bin_hz = sample_rate_hz as f64 / ((spectrum.len() - 1) * 2) as f64;
    let total_energy = spectrum[1..]
        .iter()
        .map(|magnitude| magnitude * magnitude)
        .sum::<f64>();
    let low_energy = spectrum[1..]
        .iter()
        .enumerate()
        .filter(|(index, _)| (*index + 1) as f64 * bin_hz <= LOW_BAND_LIMIT_HZ)
        .map(|(_, magnitude)| magnitude * magnitude)
        .sum::<f64>();
    let magnitude_sum = spectrum[1..].iter().sum::<f64>();
    let weighted_sum = spectrum[1..]
        .iter()
        .enumerate()
        .map(|(index, magnitude)| (index + 1) as f64 * bin_hz * magnitude)
        .sum::<f64>();
    (
        if total_energy > 0.0 {
            low_energy / total_energy
        } else {
            0.0
        },
        if magnitude_sum > 0.0 {
            weighted_sum / magnitude_sum
        } else {
            0.0
        },
    )
}

fn normalized_spectral_movement(previous: &[f64], current: &[f64]) -> f64 {
    let previous_sum = previous[1..].iter().sum::<f64>();
    let current_sum = current[1..].iter().sum::<f64>();
    if previous_sum <= 0.0 || current_sum <= 0.0 {
        return 0.0;
    }
    0.5 * previous[1..]
        .iter()
        .zip(&current[1..])
        .map(|(left, right)| (left / previous_sum - right / current_sum).abs())
        .sum::<f64>()
}

fn zero_crossing_distance(samples: &[f32]) -> usize {
    samples
        .windows(2)
        .rposition(|pair| pair[0].is_sign_positive() != pair[1].is_sign_positive())
        .map(|index| samples.len() - 1 - index)
        .unwrap_or(samples.len())
}

fn correction_energy_ratio(current: &[f32], candidate: &[f32]) -> f64 {
    let compared = CORRECTION_WINDOW_FRAMES
        .min(current.len())
        .min(candidate.len());
    if compared == 0 {
        return 0.0;
    }
    let current = &current[current.len() - compared..];
    let candidate = &candidate[candidate.len() - compared..];
    let reference_energy = current
        .iter()
        .map(|sample| (*sample as f64) * (*sample as f64))
        .sum::<f64>();
    let correction_energy = current
        .iter()
        .zip(candidate)
        .map(|(left, right)| {
            let delta = (*right - *left) as f64;
            delta * delta
        })
        .sum::<f64>();
    if reference_energy > 0.0 {
        (correction_energy / reference_energy).sqrt()
    } else {
        0.0
    }
}

#[cfg(test)]
#[path = "tail_features/tests.rs"]
mod tests;
