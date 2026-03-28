use signal_dsp_spectral::StftConfig;
use signal_primitives::SampleRate;

use crate::stats::{mean_or_zero, reduce_median, reduce_median_or_zero};
use crate::types::TemporalShapeDescriptorPack;

const TEMPORAL_SHAPE_LOOKBACK_FRAMES: usize = 32;
const TEMPORAL_SHAPE_LOOKAHEAD_FRAMES: usize = 32;
const TEMPORAL_SHAPE_PEAK_SEARCH_BACK_FRAMES: usize = 1;
const TEMPORAL_SHAPE_PEAK_SEARCH_FORWARD_FRAMES: usize = 16;
const SUSTAIN_PLATEAU_THRESHOLD_RATIO: f32 = 0.7;

pub(crate) fn compute_temporal_shape_pack(
    sample_rate: SampleRate,
    stft_config: StftConfig,
    spectral_flux: &[f32],
    frame_envelope: &[f32],
    transient_peak_indices: &[usize],
) -> TemporalShapeDescriptorPack {
    if sample_rate.0 == 0 || spectral_flux.is_empty() || frame_envelope.is_empty() {
        return TemporalShapeDescriptorPack::zero();
    }

    let flux_peak = spectral_flux.iter().copied().fold(0.0f32, f32::max);
    if flux_peak <= 0.0 || transient_peak_indices.is_empty() {
        return TemporalShapeDescriptorPack::zero();
    }

    let smoothed_envelope = smooth_series(frame_envelope, 1);
    let hop_seconds = stft_config.hop_size.0.max(1) as f32 / sample_rate.0 as f32;
    let mut transient_strengths = Vec::new();
    let mut attack_times_ms = Vec::new();
    let mut decay_times_ms = Vec::new();
    let mut sustain_plateau_ratios = Vec::new();

    for &peak_index in transient_peak_indices {
        if peak_index >= spectral_flux.len() || peak_index >= smoothed_envelope.len() {
            continue;
        }

        let normalized_strength = (spectral_flux[peak_index] / flux_peak).clamp(0.0, 1.0);
        transient_strengths.push(normalized_strength);

        let event_peak_index = local_argmax_index(
            &smoothed_envelope,
            peak_index.saturating_sub(TEMPORAL_SHAPE_PEAK_SEARCH_BACK_FRAMES),
            peak_index
                .saturating_add(TEMPORAL_SHAPE_PEAK_SEARCH_FORWARD_FRAMES)
                .min(smoothed_envelope.len().saturating_sub(1)),
        );
        let left_floor_index = local_argmin_index(
            &smoothed_envelope,
            event_peak_index.saturating_sub(TEMPORAL_SHAPE_LOOKBACK_FRAMES),
            event_peak_index,
        );
        let right_floor_index = local_argmin_index(
            &smoothed_envelope,
            event_peak_index,
            event_peak_index
                .saturating_add(TEMPORAL_SHAPE_LOOKAHEAD_FRAMES)
                .min(smoothed_envelope.len().saturating_sub(1)),
        );

        let peak_level = smoothed_envelope[event_peak_index];
        let left_baseline = smoothed_envelope[left_floor_index];
        let right_baseline = smoothed_envelope[right_floor_index];

        if peak_level > left_baseline {
            let attack_low = left_baseline + (peak_level - left_baseline) * 0.1;
            let attack_high = left_baseline + (peak_level - left_baseline) * 0.9;
            let attack_start_index = find_last_index_at_or_below(
                &smoothed_envelope,
                left_floor_index,
                event_peak_index,
                attack_low,
            );
            let attack_end_index = find_first_index_at_or_above(
                &smoothed_envelope,
                attack_start_index,
                event_peak_index,
                attack_high,
            );

            if attack_end_index > attack_start_index {
                attack_times_ms
                    .push((attack_end_index - attack_start_index) as f32 * hop_seconds * 1_000.0);
            }
        }

        if right_floor_index > event_peak_index && peak_level > right_baseline {
            let decay_high = right_baseline + (peak_level - right_baseline) * 0.9;
            let decay_low = right_baseline + (peak_level - right_baseline) * 0.1;
            let decay_start_index = find_first_index_at_or_below(
                &smoothed_envelope,
                event_peak_index,
                right_floor_index,
                decay_high,
            );
            let decay_end_index = find_first_index_at_or_below(
                &smoothed_envelope,
                decay_start_index,
                right_floor_index,
                decay_low,
            );

            if decay_end_index > decay_start_index {
                decay_times_ms
                    .push((decay_end_index - decay_start_index) as f32 * hop_seconds * 1_000.0);
            }
        }

        let baseline = left_baseline.min(right_baseline);
        let threshold =
            baseline + (peak_level - baseline).max(0.0) * SUSTAIN_PLATEAU_THRESHOLD_RATIO;

        if right_floor_index > event_peak_index && peak_level > baseline {
            let mut sustain_frames = 0usize;
            for &value in &smoothed_envelope[event_peak_index..=right_floor_index] {
                if value >= threshold {
                    sustain_frames += 1;
                } else {
                    break;
                }
            }

            let decay_frames = right_floor_index - event_peak_index;
            sustain_plateau_ratios
                .push((sustain_frames as f32 / decay_frames.max(1) as f32).clamp(0.0, 1.0));
        }
    }

    if transient_strengths.is_empty() {
        return TemporalShapeDescriptorPack::zero();
    }

    TemporalShapeDescriptorPack {
        peak_transient_strength: transient_strengths.iter().copied().fold(0.0, f32::max),
        median_transient_strength: reduce_median(&mut transient_strengths),
        attack_time_ms: reduce_median_or_zero(&mut attack_times_ms),
        decay_time_ms: reduce_median_or_zero(&mut decay_times_ms),
        sustain_plateau_ratio: mean_or_zero(&sustain_plateau_ratios).clamp(0.0, 1.0),
    }
}

fn smooth_series(values: &[f32], radius: usize) -> Vec<f32> {
    if values.is_empty() {
        return Vec::new();
    }

    let mut smoothed = Vec::with_capacity(values.len());
    for index in 0..values.len() {
        let start = index.saturating_sub(radius);
        let end = (index + radius).min(values.len() - 1);
        let slice = &values[start..=end];
        smoothed.push(slice.iter().copied().sum::<f32>() / slice.len() as f32);
    }

    smoothed
}

fn local_argmax_index(values: &[f32], start: usize, end: usize) -> usize {
    let mut best_index = start.min(values.len().saturating_sub(1));
    let mut best_value = values.get(best_index).copied().unwrap_or(0.0);
    for (index, value) in values
        .iter()
        .copied()
        .enumerate()
        .skip(start)
        .take(end.min(values.len().saturating_sub(1)) - start + 1)
    {
        if value > best_value {
            best_value = value;
            best_index = index;
        }
    }
    best_index
}

fn local_argmin_index(values: &[f32], start: usize, end: usize) -> usize {
    let mut best_index = start.min(values.len().saturating_sub(1));
    let mut best_value = values.get(best_index).copied().unwrap_or(0.0);
    for (index, value) in values
        .iter()
        .copied()
        .enumerate()
        .skip(start)
        .take(end.min(values.len().saturating_sub(1)) - start + 1)
    {
        if value < best_value {
            best_value = value;
            best_index = index;
        }
    }
    best_index
}

fn find_last_index_at_or_below(values: &[f32], start: usize, end: usize, threshold: f32) -> usize {
    let bounded_end = end.min(values.len().saturating_sub(1));
    for index in (start.min(bounded_end)..=bounded_end).rev() {
        if values[index] <= threshold {
            return index;
        }
    }
    start.min(bounded_end)
}

fn find_first_index_at_or_above(values: &[f32], start: usize, end: usize, threshold: f32) -> usize {
    let bounded_end = end.min(values.len().saturating_sub(1));
    for (index, value) in values
        .iter()
        .copied()
        .enumerate()
        .skip(start.min(bounded_end))
        .take(bounded_end - start.min(bounded_end) + 1)
    {
        if value >= threshold {
            return index;
        }
    }
    bounded_end
}

fn find_first_index_at_or_below(values: &[f32], start: usize, end: usize, threshold: f32) -> usize {
    let bounded_end = end.min(values.len().saturating_sub(1));
    for (index, value) in values
        .iter()
        .copied()
        .enumerate()
        .skip(start.min(bounded_end))
        .take(bounded_end - start.min(bounded_end) + 1)
    {
        if value <= threshold {
            return index;
        }
    }
    bounded_end
}
