use rustfft::{num_complex::Complex32, FftPlanner};
use signal_primitives::Sample;

use super::{BINS, FFT_FRAMES, WINDOW_FRAMES};

pub(super) struct Analysis {
    pub(super) spectra: Vec<Vec<Complex32>>,
    pub(super) phases: Vec<Vec<f32>>,
    pub(super) magnitudes: Vec<Vec<f32>>,
}

pub(super) fn hann_window() -> Vec<f32> {
    (0..WINDOW_FRAMES)
        .map(|index| {
            0.5 - 0.5 * (std::f32::consts::TAU * index as f32 / WINDOW_FRAMES as f32).cos()
        })
        .collect()
}

pub(super) fn analyze(input: &[Sample], hop: usize, frames: usize, window: &[f32]) -> Analysis {
    let padded_len = (frames - 1) * hop + FFT_FRAMES;
    let mut padded = vec![0.0_f32; padded_len];
    let source_start = hop + WINDOW_FRAMES / 2;
    padded[source_start..source_start + input.len()].copy_from_slice(input);
    let mut planner = FftPlanner::<f32>::new();
    let forward = planner.plan_fft_forward(FFT_FRAMES);
    let mut spectra = Vec::with_capacity(frames);
    let mut phases = Vec::with_capacity(frames);
    let mut magnitudes = Vec::with_capacity(frames);
    for frame in 0..frames {
        let start = frame * hop;
        let mut spectrum = vec![Complex32::new(0.0, 0.0); FFT_FRAMES];
        for index in 0..WINDOW_FRAMES {
            spectrum[index].re = padded[start + index] * window[index];
        }
        forward.process(&mut spectrum);
        phases.push(spectrum[..BINS].iter().map(|bin| bin.arg()).collect());
        magnitudes.push(spectrum[..BINS].iter().map(|bin| bin.norm()).collect());
        spectra.push(spectrum);
    }
    Analysis {
        spectra,
        phases,
        magnitudes,
    }
}

pub(super) fn time_phase_derivatives(phases: &[Vec<f32>], hop: usize) -> Vec<Vec<f32>> {
    let mut derivatives = vec![vec![0.0; BINS]; phases.len()];
    for frame in 1..phases.len() - 1 {
        for bin in 0..BINS {
            let expected = std::f32::consts::TAU * bin as f32 * hop as f32 / FFT_FRAMES as f32;
            let backward = wrap_phase(phases[frame][bin] - phases[frame - 1][bin] - expected)
                / hop as f32
                + std::f32::consts::TAU * bin as f32 / FFT_FRAMES as f32;
            let forward = wrap_phase(phases[frame + 1][bin] - phases[frame][bin] - expected)
                / hop as f32
                + std::f32::consts::TAU * bin as f32 / FFT_FRAMES as f32;
            derivatives[frame][bin] = 0.5 * (backward + forward);
        }
    }
    derivatives
}

pub(super) fn frequency_phase_derivative(phase: &[f32]) -> Vec<f32> {
    let mut derivative = vec![0.0; BINS];
    derivative[0] = wrap_phase(phase[1] - phase[0]);
    for bin in 1..BINS - 1 {
        let backward = wrap_phase(phase[bin] - phase[bin - 1]);
        let forward = wrap_phase(phase[bin + 1] - phase[bin]);
        derivative[bin] = 0.5 * (backward + forward);
    }
    derivative[BINS - 1] = wrap_phase(phase[BINS - 1] - phase[BINS - 2]);
    derivative
}

fn wrap_phase(value: f32) -> f32 {
    (value + std::f32::consts::PI).rem_euclid(std::f32::consts::TAU) - std::f32::consts::PI
}
