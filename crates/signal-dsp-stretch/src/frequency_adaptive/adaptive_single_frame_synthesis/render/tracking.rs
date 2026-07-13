use std::sync::Arc;

use rustfft::{num_complex::Complex64, Fft, FftPlanner};

use super::FFT_FRAMES;

pub(super) fn analytic_channels(
    channels: &[Vec<f64>],
    planner: &mut FftPlanner<f64>,
) -> Vec<Vec<Complex64>> {
    channels
        .iter()
        .map(|channel| analytic(channel, planner))
        .collect()
}

pub(super) fn spectrum(
    input: &[Complex64],
    source: isize,
    weights: &[f64],
    forward: &Arc<dyn Fft<f64>>,
) -> Vec<Complex64> {
    let mut result = (0..FFT_FRAMES)
        .map(|offset| {
            let logical = source - FFT_FRAMES as isize / 2 + offset as isize;
            reflected(input, logical) * weights[offset]
        })
        .collect::<Vec<_>>();
    forward.process(&mut result);
    result
}

pub(super) fn linked(spectra: &[Vec<Complex64>]) -> Vec<f64> {
    let mut result = vec![0.0_f64; FFT_FRAMES / 2 + 1];
    for spectrum in spectra {
        for (linked, value) in result.iter_mut().zip(spectrum) {
            *linked += value.norm_sqr();
        }
    }
    result
}

pub(super) fn legacy_peaks(linked: &[f64]) -> Vec<usize> {
    (1..linked.len() - 1)
        .filter(|bin| {
            linked[*bin] > 1.0e-18
                && linked[*bin] > linked[*bin - 1]
                && linked[*bin] >= linked[*bin + 1]
        })
        .collect()
}

pub(super) fn active_peaks(linked: &[f64]) -> Vec<usize> {
    // A peak owns phase only when it carries at least one percent of the
    // strongest linked-bin energy. Lower local maxima are window sidelobes,
    // not independent physical trajectories.
    let active_floor = linked.iter().copied().fold(0.0_f64, f64::max) * 1.0e-2;
    (1..linked.len() - 1)
        .filter(|bin| {
            linked[*bin] > active_floor.max(1.0e-18)
                && linked[*bin] > linked[*bin - 1]
                && linked[*bin] >= linked[*bin + 1]
        })
        .collect()
}

fn analytic(input: &[f64], planner: &mut FftPlanner<f64>) -> Vec<Complex64> {
    let mut spectrum = input
        .iter()
        .map(|sample| Complex64::new(*sample, 0.0))
        .collect::<Vec<_>>();
    planner.plan_fft_forward(input.len()).process(&mut spectrum);
    for bin in 1..input.len() / 2 {
        spectrum[bin] *= 2.0;
    }
    for value in &mut spectrum[input.len() / 2 + 1..] {
        *value = Complex64::new(0.0, 0.0);
    }
    planner.plan_fft_inverse(input.len()).process(&mut spectrum);
    for value in &mut spectrum {
        *value /= input.len() as f64;
    }
    spectrum
}

fn reflected(input: &[Complex64], mut index: isize) -> Complex64 {
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
