use std::f64::consts::TAU;

use crate::measure_tonal_texture;

use super::super::super::study_local_schedule::{schedule::Schedule, BASE_HOP, SOURCE_FRAMES};
use super::control::{sinusoid, RATIOS, SAMPLE_RATE};

const DENSE_EXCLUSION: usize = 32;
const TIMING_SEARCH: usize = 512;

pub(in crate::frequency_adaptive) fn error(input: &[f64], output: &[f64]) -> [f64; 4] {
    let differences = input
        .iter()
        .zip(output)
        .map(|(input, output)| (input - output).abs())
        .collect::<Vec<_>>();
    let peak = differences.iter().copied().fold(0.0_f64, f64::max);
    let rms = (differences.iter().map(|value| value * value).sum::<f64>()
        / differences.len() as f64)
        .sqrt();
    [
        peak,
        rms,
        differences[0],
        differences[differences.len() - 1],
    ]
}

pub(in crate::frequency_adaptive) fn angular_frequency_error(samples: &[f64], hz: f64) -> f64 {
    let window = 4_096;
    let first_start = (samples.len() / 3).saturating_sub(window / 2);
    let second_start = (samples.len() * 2 / 3).saturating_sub(window / 2);
    let omega = TAU * hz / SAMPLE_RATE;
    let coefficient = |start: usize| {
        samples[start..start + window].iter().enumerate().fold(
            (0.0_f64, 0.0_f64),
            |sum, (index, sample)| {
                let weight = 0.5 - 0.5 * (TAU * index as f64 / window as f64).cos();
                let phase = omega * index as f64;
                (
                    sum.0 + weight * sample * phase.cos(),
                    sum.1 - weight * sample * phase.sin(),
                )
            },
        )
    };
    let left = coefficient(first_start);
    let right = coefficient(second_start);
    let phase_delta = (right.1.atan2(right.0)
        - left.1.atan2(left.0)
        - omega * (second_start - first_start) as f64)
        .rem_euclid(TAU);
    let wrapped = if phase_delta > std::f64::consts::PI {
        phase_delta - TAU
    } else {
        phase_delta
    };
    (wrapped / (second_start - first_start) as f64).abs()
}

pub(in crate::frequency_adaptive) fn projected(schedule: &Schedule, source: usize) -> usize {
    let base = source / BASE_HOP;
    let remainder = source % BASE_HOP;
    if remainder == 0 {
        return schedule.positions[base];
    }
    let left = schedule.positions[base] as f64;
    let right = schedule.positions[base + 1] as f64;
    (left + (right - left) * remainder as f64 / BASE_HOP as f64).round() as usize
}

pub(in crate::frequency_adaptive) fn peak_index(
    samples: &[f64],
    center: usize,
    radius: usize,
) -> usize {
    let start = center.saturating_sub(radius);
    let end = (center + radius + 1).min(samples.len());
    (start..end)
        .max_by(|left, right| samples[*left].abs().total_cmp(&samples[*right].abs()))
        .unwrap_or(start)
}

pub(in crate::frequency_adaptive) fn dense_event_errors(
    samples: &[f64],
    expected: [usize; 2],
) -> ([usize; 2], usize) {
    let start = expected[0].saturating_sub(TIMING_SEARCH);
    let end = (expected[1] + TIMING_SEARCH + 1).min(samples.len());
    let first = peak_index(samples, (start + end) / 2, (end - start) / 2);
    let second = samples[start..end]
        .iter()
        .enumerate()
        .filter(|(index, _)| (start + *index).abs_diff(first) > DENSE_EXCLUSION)
        .max_by(|left, right| left.1.abs().total_cmp(&right.1.abs()))
        .map(|(index, _)| start + index);
    let Some(second) = second else {
        return ([usize::MAX; 2], 2);
    };
    let mut actual = [first, second];
    actual.sort_unstable();
    let unmatched = actual
        .iter()
        .zip(expected)
        .filter(|(actual, expected)| actual.abs_diff(*expected) > TIMING_SEARCH)
        .count();
    (
        [
            actual[0].abs_diff(expected[0]),
            actual[1].abs_diff(expected[1]),
        ],
        unmatched,
    )
}

pub(super) fn crest_db(samples: &[f64], center: usize, radius: usize) -> f64 {
    let start = center.saturating_sub(radius);
    let end = (center + radius + 1).min(samples.len());
    let slice = &samples[start..end];
    let rms = (slice.iter().map(|sample| sample * sample).sum::<f64>() / slice.len() as f64).sqrt();
    20.0 * (peak(slice) / (rms + 1.0e-15)).log10()
}

pub(super) fn replica_ratio(samples: &[f64], center: usize) -> f64 {
    let primary_start = center.saturating_sub(64);
    let primary_end = (center + 65).min(samples.len());
    let secondary_end = (center + 513).min(samples.len());
    let primary = peak(&samples[primary_start..primary_end]);
    let secondary = if primary_end < secondary_end {
        peak(&samples[primary_end..secondary_end])
    } else {
        0.0
    };
    secondary / (primary + 1.0e-15)
}

pub(super) fn texture(source: &[f64], output: &[f64], ratio: f64) -> [f64; 6] {
    let source = source
        .iter()
        .map(|sample| *sample as f32)
        .collect::<Vec<_>>();
    let output = output
        .iter()
        .map(|sample| *sample as f32)
        .collect::<Vec<_>>();
    let evidence = measure_tonal_texture(&source, &output, ratio);
    [
        evidence.mean_spectral_residual_ratio,
        evidence.mean_added_sideband_ratio,
        evidence.spectral_modulation_delta,
        evidence.envelope_modulation_delta_db,
        evidence.max_spectral_residual_ratio,
        evidence.max_added_sideband_ratio,
    ]
}

pub(in crate::frequency_adaptive) fn peak(samples: &[f64]) -> f64 {
    samples
        .iter()
        .map(|sample| sample.abs())
        .fold(0.0_f64, f64::max)
}

pub(super) fn rms_prefix(samples: &[f64], count: usize) -> f64 {
    rms(&samples[..count.min(samples.len())])
}

pub(super) fn rms_suffix(samples: &[f64], count: usize) -> f64 {
    rms(&samples[samples.len().saturating_sub(count)..])
}

fn rms(samples: &[f64]) -> f64 {
    if samples.is_empty() {
        return 0.0;
    }
    (samples.iter().map(|sample| sample * sample).sum::<f64>() / samples.len() as f64).sqrt()
}

#[test]
fn synthetic_tone_frequency_measurement_resolves_rule_30n_limit() {
    for hz in [55.0, 440.0, 8_000.0] {
        for ratio in RATIOS {
            let length = (SOURCE_FRAMES as f64 * ratio).round() as usize;
            let samples = (0..length)
                .map(|index| sinusoid(hz, index))
                .collect::<Vec<_>>();
            assert!(
                angular_frequency_error(&samples, hz) <= 2.5e-7,
                "hz={hz} ratio={ratio} error={}",
                angular_frequency_error(&samples, hz),
            );
        }
    }
}
