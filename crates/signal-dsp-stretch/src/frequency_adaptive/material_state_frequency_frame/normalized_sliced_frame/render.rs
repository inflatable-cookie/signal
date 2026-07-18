use std::sync::Arc;

use rustfft::{num_complex::Complex64, Fft};

use super::*;
use crate::frequency_adaptive::material_state_frequency_frame::{
    local_coefficient, reflected_sample,
};

mod schedule;
use schedule::coverage_failures;

pub(super) fn required_slice_count(length: usize, advance: usize) -> usize {
    schedule::required_slice_count(length, advance)
}

pub(super) fn boundary_token_review(length: usize, geometry: &Geometry) -> TokenReview {
    schedule::boundary_token_review(length, geometry)
}

pub(super) struct RenderResult {
    pub samples: Vec<f64>,
    pub slice_count: usize,
    pub work: WorkCounts,
    pub peak_error: f64,
    pub rms_error: f64,
    pub head_error: f64,
    pub tail_error: f64,
    pub imaginary_residue: f64,
    pub conjugate_error: f64,
    pub partition_error: f64,
    pub coverage_failures: usize,
    pub non_finite_values: usize,
    pub hash: u64,
}

pub(super) struct Renderer<'a> {
    geometry: &'a Geometry,
    window: Vec<f64>,
    forward_full: Arc<dyn Fft<f64>>,
    inverse_full: Arc<dyn Fft<f64>>,
    forward_band: Arc<dyn Fft<f64>>,
    inverse_band: Arc<dyn Fft<f64>>,
    scratch: Vec<Complex64>,
}

impl<'a> Renderer<'a> {
    pub fn new(geometry: &'a Geometry) -> Self {
        let mut planner = rustfft::FftPlanner::<f64>::new();
        Self {
            geometry,
            window: (0..geometry.fft_frames)
                .map(|index| {
                    (std::f64::consts::PI * (index as f64 + 0.5) / geometry.fft_frames as f64).sin()
                })
                .collect(),
            forward_full: planner.plan_fft_forward(geometry.fft_frames),
            inverse_full: planner.plan_fft_inverse(geometry.fft_frames),
            forward_band: planner.plan_fft_forward(COEFFICIENT_CAPACITY),
            inverse_band: planner.plan_fft_inverse(COEFFICIENT_CAPACITY),
            scratch: vec![Complex64::default(); geometry.scratch_capacity],
        }
    }

    pub fn render(&mut self, input: &[f64]) -> RenderResult {
        let mut samples = vec![0.0; input.len()];
        let mut imaginary_residue = 0.0_f64;
        let mut conjugate_error = 0.0_f64;
        let mut non_finite_values = 0;
        let slice_count = required_slice_count(input.len(), self.geometry.outer_advance);
        for slice_index in 0..slice_count {
            let start = (slice_index as isize - 1) * self.geometry.outer_advance as isize;
            let mut spectrum = (0..self.geometry.fft_frames)
                .map(|local| {
                    Complex64::new(
                        reflected_sample(input, start + local as isize) * self.window[local],
                        0.0,
                    )
                })
                .collect::<Vec<_>>();
            self.forward_full
                .process_with_scratch(&mut spectrum, &mut self.scratch);
            let (inner, inner_imaginary, inner_conjugate, inner_non_finite) =
                self.reconstruct_slice(&spectrum);
            imaginary_residue = imaginary_residue.max(inner_imaginary);
            conjugate_error = conjugate_error.max(inner_conjugate);
            non_finite_values += inner_non_finite;
            for (local, value) in inner.into_iter().enumerate() {
                let logical = start + local as isize;
                if (0..input.len() as isize).contains(&logical) {
                    let output = value * self.window[local];
                    samples[logical as usize] += output.re;
                    imaginary_residue = imaginary_residue.max(output.im.abs());
                }
            }
        }

        let mut peak_error = 0.0_f64;
        let mut square_sum = 0.0;
        let mut hash = HASH_OFFSET;
        for (source, output) in input.iter().zip(&samples) {
            let error = (source - output).abs();
            peak_error = peak_error.max(error);
            square_sum += error * error;
            non_finite_values += usize::from(!output.is_finite());
            hash_u64(&mut hash, output.to_bits());
        }
        let head_error = input
            .first()
            .zip(samples.first())
            .map_or(0.0, |(source, output)| (source - output).abs());
        let tail_error = input
            .last()
            .zip(samples.last())
            .map_or(0.0, |(source, output)| (source - output).abs());
        RenderResult {
            samples,
            slice_count,
            work: self.geometry.per_slice_work.scaled(slice_count),
            peak_error,
            rms_error: (square_sum / input.len().max(1) as f64).sqrt(),
            head_error,
            tail_error,
            imaginary_residue,
            conjugate_error,
            partition_error: partition_error(&self.window, self.geometry.outer_advance),
            coverage_failures: coverage_failures(
                input.len(),
                slice_count,
                self.geometry.outer_advance,
            ),
            non_finite_values,
            hash,
        }
    }

    fn reconstruct_slice(&mut self, spectrum: &[Complex64]) -> (Vec<Complex64>, f64, f64, usize) {
        let representation = &self.geometry.representation;
        let mut output = vec![Complex64::default(); self.geometry.fft_frames];
        let mut coefficients = vec![Complex64::default(); COEFFICIENT_CAPACITY];
        for band in &representation.bands {
            coefficients.fill(Complex64::default());
            for &(bin, weight) in &band.taps {
                let local = local_coefficient(
                    bin,
                    band.center,
                    COEFFICIENT_CAPACITY,
                    self.geometry.fft_frames,
                );
                coefficients[local] = spectrum[bin] * weight;
            }
            self.inverse_band
                .process_with_scratch(&mut coefficients, &mut self.scratch);
            coefficients
                .iter_mut()
                .for_each(|value| *value /= COEFFICIENT_CAPACITY as f64);
            self.forward_band
                .process_with_scratch(&mut coefficients, &mut self.scratch);
            for &(bin, weight) in &band.taps {
                let local = local_coefficient(
                    bin,
                    band.center,
                    COEFFICIENT_CAPACITY,
                    self.geometry.fft_frames,
                );
                output[bin] += coefficients[local] * weight / representation.frame_operator[bin];
            }
        }
        close_conjugate_spectrum(&mut output);
        let conjugate_error = conjugate_error(&output);
        self.inverse_full
            .process_with_scratch(&mut output, &mut self.scratch);
        output
            .iter_mut()
            .for_each(|value| *value /= self.geometry.fft_frames as f64);
        let imaginary_residue = output
            .iter()
            .map(|value| value.im.abs())
            .fold(0.0_f64, f64::max);
        let non_finite = output
            .iter()
            .map(|value| usize::from(!value.re.is_finite()) + usize::from(!value.im.is_finite()))
            .sum();
        (output, imaginary_residue, conjugate_error, non_finite)
    }
}

fn partition_error(window: &[f64], advance: usize) -> f64 {
    (0..advance)
        .map(|index| (window[index].powi(2) + window[index + advance].powi(2) - 1.0).abs())
        .fold(0.0_f64, f64::max)
}

fn close_conjugate_spectrum(spectrum: &mut [Complex64]) {
    spectrum[0].im = 0.0;
    spectrum[spectrum.len() / 2].im = 0.0;
    for bin in 1..spectrum.len() / 2 {
        let mirror = spectrum.len() - bin;
        let closed = (spectrum[bin] + spectrum[mirror].conj()) * 0.5;
        spectrum[bin] = closed;
        spectrum[mirror] = closed.conj();
    }
}

fn conjugate_error(spectrum: &[Complex64]) -> f64 {
    (0..spectrum.len())
        .map(|bin| {
            let mirror = if bin == 0 { 0 } else { spectrum.len() - bin };
            (spectrum[bin] - spectrum[mirror].conj()).norm()
        })
        .fold(0.0_f64, f64::max)
}
