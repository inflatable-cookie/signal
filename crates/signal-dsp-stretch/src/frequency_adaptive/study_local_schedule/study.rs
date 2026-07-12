use rustfft::{num_complex::Complex64, FftPlanner};

use super::{hash, BASE_HOP, HASH_OFFSET};

const LAYERS: [usize; 3] = [512, 2_048, 8_192];

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Study {
    pub evidence: Vec<f64>,
    pub agreement: Vec<usize>,
    pub layer_peaks: [Vec<usize>; 3],
    pub hash: u64,
}

pub(crate) fn analyze(channels: &[Vec<f64>], source_frames: usize) -> Study {
    let frame_count = source_frames / BASE_HOP + 1;
    let mut layer_scores: [Vec<f64>; 3] = std::array::from_fn(|_| vec![0.0; frame_count]);
    let mut planner = FftPlanner::<f64>::new();
    for (layer, length) in LAYERS.into_iter().enumerate() {
        let fft = planner.plan_fft_forward(length);
        let window = window(length);
        let mut energies = vec![0.0; frame_count];
        let mut spectra = Vec::with_capacity(frame_count);
        for frame in 0..frame_count {
            let center = frame * BASE_HOP;
            let mut linked_magnitudes = vec![0.0; length / 2 + 1];
            for channel in channels {
                let mut buffer = (0..length)
                    .map(|offset| {
                        let logical = center as isize - length as isize / 2 + offset as isize;
                        Complex64::new(reflected(channel, logical) * window[offset], 0.0)
                    })
                    .collect::<Vec<_>>();
                fft.process(&mut buffer);
                for (linked, coefficient) in linked_magnitudes.iter_mut().zip(buffer) {
                    *linked += coefficient.norm_sqr();
                }
            }
            energies[frame] = linked_magnitudes
                .iter()
                .sum::<f64>()
                .max(f64::MIN_POSITIVE)
                .ln();
            spectra.push(
                linked_magnitudes
                    .into_iter()
                    .map(f64::sqrt)
                    .collect::<Vec<_>>(),
            );
        }
        let rise = (0..frame_count)
            .map(|frame| {
                if frame == 0 {
                    0.0
                } else {
                    (energies[frame] - energies[frame - 1]).max(0.0)
                }
            })
            .collect::<Vec<_>>();
        let flux = (0..frame_count)
            .map(|frame| {
                if frame == 0 {
                    0.0
                } else {
                    spectra[frame]
                        .iter()
                        .zip(&spectra[frame - 1])
                        .map(|(now, before)| (now - before).max(0.0))
                        .sum()
                }
            })
            .collect::<Vec<f64>>();
        let rise = robust_normalize(&rise);
        let flux = robust_normalize(&flux);
        for frame in 0..frame_count {
            layer_scores[layer][frame] = 0.5 * (rise[frame] + flux[frame]);
        }
    }
    let layer_peaks = std::array::from_fn(|layer| local_peaks(&layer_scores[layer]));
    let mut agreement = vec![0; frame_count];
    for frame in 0..frame_count {
        let center = frame * BASE_HOP;
        agreement[frame] = layer_peaks
            .iter()
            .filter(|peaks| peaks.iter().any(|peak| peak.abs_diff(center) <= 256))
            .count();
    }
    let evidence = (0..frame_count)
        .map(|frame| layer_scores.iter().map(|scores| scores[frame]).sum::<f64>() / 3.0)
        .collect::<Vec<_>>();
    let mut result = Study {
        evidence,
        agreement,
        layer_peaks,
        hash: 0,
    };
    result.hash = study_hash(&result);
    result
}

pub(crate) fn select(study: &Study, threshold: f64, required_layers: usize) -> Vec<usize> {
    let normalized = robust_normalize(&study.evidence);
    let mut points = vec![0];
    for frame in 1..normalized.len() - 1 {
        let center = frame * BASE_HOP;
        let layer_peak = study
            .layer_peaks
            .iter()
            .any(|peaks| peaks.binary_search(&center).is_ok());
        if normalized[frame] >= threshold && layer_peak && study.agreement[frame] >= required_layers
        {
            points.push(center);
        }
    }
    points.push((normalized.len() - 1) * BASE_HOP);
    points
}

fn local_peaks(values: &[f64]) -> Vec<usize> {
    (2..values.len() - 2)
        .filter(|index| {
            let candidate = values[*index];
            let neighborhood = &values[index - 2..=index + 2];
            neighborhood.iter().all(|value| candidate >= *value)
                && neighborhood.iter().any(|value| candidate > *value)
        })
        .map(|index| index * BASE_HOP)
        .collect()
}

fn robust_normalize(values: &[f64]) -> Vec<f64> {
    let center = median(values);
    let deviations = values
        .iter()
        .map(|value| (value - center).abs())
        .collect::<Vec<_>>();
    let mad = median(&deviations).max(1.0e-12);
    values.iter().map(|value| (value - center) / mad).collect()
}

fn median(values: &[f64]) -> f64 {
    let mut sorted = values.to_vec();
    sorted.sort_by(f64::total_cmp);
    let middle = sorted.len() / 2;
    if sorted.len() % 2 == 0 {
        0.5 * (sorted[middle - 1] + sorted[middle])
    } else {
        sorted[middle]
    }
}

fn window(length: usize) -> Vec<f64> {
    (0..length)
        .map(|index| {
            (0.5 - 0.5 * (std::f64::consts::TAU * index as f64 / length as f64).cos()).sqrt()
        })
        .collect()
}
fn reflected(input: &[f64], mut index: isize) -> f64 {
    let end = input.len() as isize - 1;
    while index < 0 || index > end {
        index = if index < 0 {
            -index - 1
        } else {
            2 * end - index + 1
        };
    }
    input[index as usize]
}
fn study_hash(study: &Study) -> u64 {
    let mut state = HASH_OFFSET;
    for value in &study.evidence {
        hash(&mut state, value.to_bits());
    }
    for value in &study.agreement {
        hash(&mut state, *value as u64);
    }
    for peaks in &study.layer_peaks {
        for peak in peaks {
            hash(&mut state, *peak as u64);
        }
    }
    state
}
