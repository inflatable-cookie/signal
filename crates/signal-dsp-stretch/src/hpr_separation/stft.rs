use rustfft::{num_complex::Complex32, Fft, FftPlanner};
use signal_primitives::Sample;
use std::sync::Arc;

const HORIZONTAL_MEDIAN_SECONDS: f64 = 0.200;
const VERTICAL_MEDIAN_HZ: f64 = 500.0;

pub(super) struct StageConfig {
    pub(super) window_size: usize,
    pub(super) hop_size: usize,
    pub(super) horizontal_span: usize,
    pub(super) vertical_span: usize,
}

pub(super) struct BinaryStageResult {
    pub(super) selected: Vec<Sample>,
    pub(super) complement: Vec<Sample>,
    pub(super) selected_bins: usize,
    pub(super) complement_bins: usize,
    pub(super) partition_exact: bool,
    pub(super) uncovered_samples: usize,
}

pub(super) fn stage_config(sample_rate_hz: f64, window_seconds: f64) -> StageConfig {
    let requested_window = (sample_rate_hz * window_seconds).round().max(64.0) as usize;
    let window_size = nearest_power_of_two(requested_window).max(64);
    let hop_size = window_size / 4;
    let frame_seconds = hop_size as f64 / sample_rate_hz;
    let bin_hz = sample_rate_hz / window_size as f64;
    StageConfig {
        window_size,
        hop_size,
        horizontal_span: nearest_odd(HORIZONTAL_MEDIAN_SECONDS / frame_seconds),
        vertical_span: nearest_odd(VERTICAL_MEDIAN_HZ / bin_hz),
    }
}

pub(super) fn binary_median_stage(
    input: &[Sample],
    config: &StageConfig,
    selected: impl Fn(f32, f32) -> bool,
) -> BinaryStageResult {
    if input.is_empty() {
        return BinaryStageResult {
            selected: Vec::new(),
            complement: Vec::new(),
            selected_bins: 0,
            complement_bins: 0,
            partition_exact: true,
            uncovered_samples: 0,
        };
    }
    let window = hann_window(config.window_size);
    let (spectra, crop_start) = analyze(input, config, &window);
    let bins = config.window_size / 2 + 1;
    let magnitudes = spectra
        .iter()
        .flat_map(|spectrum| spectrum[..bins].iter().map(|bin| bin.norm()))
        .collect::<Vec<_>>();
    let mut mask = vec![false; spectra.len() * bins];
    let mut horizontal_scratch = Vec::with_capacity(config.horizontal_span);
    let mut vertical_scratch = Vec::with_capacity(config.vertical_span);
    for frame in 0..spectra.len() {
        for bin in 0..bins {
            let horizontal = median_horizontal(
                &magnitudes,
                spectra.len(),
                bins,
                frame,
                bin,
                config.horizontal_span,
                &mut horizontal_scratch,
            );
            let vertical = median_vertical(
                &magnitudes,
                bins,
                frame,
                bin,
                config.vertical_span,
                &mut vertical_scratch,
            );
            mask[frame * bins + bin] = selected(horizontal, vertical);
        }
    }
    let selected_bins = mask.iter().filter(|owned| **owned).count();
    let complement_bins = mask.len() - selected_bins;
    let (selected_output, selected_uncovered) = synthesize_masked(
        &spectra,
        &mask,
        true,
        input.len(),
        crop_start,
        config,
        &window,
    );
    let (complement_output, complement_uncovered) = synthesize_masked(
        &spectra,
        &mask,
        false,
        input.len(),
        crop_start,
        config,
        &window,
    );
    BinaryStageResult {
        selected: selected_output,
        complement: complement_output,
        selected_bins,
        complement_bins,
        partition_exact: selected_bins + complement_bins == mask.len(),
        uncovered_samples: selected_uncovered.max(complement_uncovered),
    }
}

fn nearest_power_of_two(value: usize) -> usize {
    let upper = value.next_power_of_two();
    let lower = upper / 2;
    if lower >= 1 && value - lower < upper - value {
        lower
    } else {
        upper
    }
}

fn nearest_odd(value: f64) -> usize {
    let rounded = value.round().max(1.0) as usize;
    if rounded % 2 == 1 {
        rounded
    } else if value < rounded as f64 {
        rounded.saturating_sub(1).max(1)
    } else {
        rounded + 1
    }
}

fn hann_window(size: usize) -> Vec<f32> {
    (0..size)
        .map(|index| 0.5 - 0.5 * (std::f32::consts::TAU * index as f32 / size as f32).cos())
        .collect()
}

fn analyze(input: &[Sample], config: &StageConfig, window: &[f32]) -> (Vec<Vec<Complex32>>, usize) {
    let crop_start = config.window_size / 2;
    let minimum_len = crop_start + input.len() + config.window_size / 2;
    let frame_count = minimum_len
        .saturating_sub(config.window_size)
        .div_ceil(config.hop_size)
        + 1;
    let padded_len = (frame_count - 1) * config.hop_size + config.window_size;
    let mut padded = vec![0.0; padded_len];
    padded[crop_start..crop_start + input.len()].copy_from_slice(input);
    let mut planner = FftPlanner::<f32>::new();
    let forward = planner.plan_fft_forward(config.window_size);
    let mut spectra = Vec::with_capacity(frame_count);
    for frame in 0..frame_count {
        let start = frame * config.hop_size;
        let mut spectrum = padded[start..start + config.window_size]
            .iter()
            .zip(window)
            .map(|(sample, weight)| Complex32::new(sample * weight, 0.0))
            .collect::<Vec<_>>();
        forward.process(&mut spectrum);
        spectra.push(spectrum);
    }
    (spectra, crop_start)
}

fn median_horizontal(
    magnitudes: &[f32],
    frames: usize,
    bins: usize,
    frame: usize,
    bin: usize,
    span: usize,
    scratch: &mut Vec<f32>,
) -> f32 {
    scratch.clear();
    let half = span / 2;
    for offset in 0..span {
        let source_frame = frame
            .saturating_add(offset)
            .saturating_sub(half)
            .min(frames - 1);
        scratch.push(magnitudes[source_frame * bins + bin]);
    }
    median(scratch)
}

fn median_vertical(
    magnitudes: &[f32],
    bins: usize,
    frame: usize,
    bin: usize,
    span: usize,
    scratch: &mut Vec<f32>,
) -> f32 {
    scratch.clear();
    let half = span / 2;
    for offset in 0..span {
        let source_bin = bin
            .saturating_add(offset)
            .saturating_sub(half)
            .min(bins - 1);
        scratch.push(magnitudes[frame * bins + source_bin]);
    }
    median(scratch)
}

fn median(values: &mut [f32]) -> f32 {
    let middle = values.len() / 2;
    values
        .select_nth_unstable_by(middle, f32::total_cmp)
        .1
        .to_owned()
}

fn synthesize_masked(
    spectra: &[Vec<Complex32>],
    positive_mask: &[bool],
    selected: bool,
    output_len: usize,
    crop_start: usize,
    config: &StageConfig,
    window: &[f32],
) -> (Vec<Sample>, usize) {
    let bins = config.window_size / 2 + 1;
    let ola_len = (spectra.len() - 1) * config.hop_size + config.window_size;
    let mut output = vec![0.0_f32; ola_len];
    let mut normalization = vec![0.0_f32; ola_len];
    let mut planner = FftPlanner::<f32>::new();
    let inverse: Arc<dyn Fft<f32>> = planner.plan_fft_inverse(config.window_size);
    for (frame, source_spectrum) in spectra.iter().enumerate() {
        let mut spectrum = source_spectrum.clone();
        for bin in 0..bins {
            if positive_mask[frame * bins + bin] != selected {
                spectrum[bin] = Complex32::new(0.0, 0.0);
                if bin > 0 && bin < config.window_size / 2 {
                    spectrum[config.window_size - bin] = Complex32::new(0.0, 0.0);
                }
            }
        }
        inverse.process(&mut spectrum);
        let start = frame * config.hop_size;
        for index in 0..config.window_size {
            let weight = window[index];
            output[start + index] += spectrum[index].re * weight / config.window_size as f32;
            normalization[start + index] += weight * weight;
        }
    }
    let crop_end = crop_start + output_len;
    let uncovered = normalization[crop_start..crop_end]
        .iter()
        .filter(|weight| **weight <= f32::EPSILON)
        .count();
    for (sample, weight) in output.iter_mut().zip(&normalization) {
        if *weight > f32::EPSILON {
            *sample /= *weight;
        }
    }
    (output[crop_start..crop_end].to_vec(), uncovered)
}
