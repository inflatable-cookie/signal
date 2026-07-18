use std::sync::Arc;

use rustfft::Fft;

use super::*;

pub(super) struct SliceResult {
    pub samples: Vec<f64>,
    pub slice_count: usize,
    pub maximum_live_slices: usize,
    pub peak_live_coefficients: usize,
    pub counted_operations: usize,
    pub imaginary_residue: f64,
    pub conjugate_error: f64,
    pub partition_error: f64,
    pub coverage_failures: usize,
    pub non_finite_values: usize,
    pub output_hash: u64,
}

pub(super) struct SlicedRenderer<'a> {
    representation: &'a Representation,
    window: Vec<f64>,
    forward_full: Arc<dyn Fft<f64>>,
    inverse_full: Arc<dyn Fft<f64>>,
    forward_band: Arc<dyn Fft<f64>>,
    inverse_band: Arc<dyn Fft<f64>>,
}

impl<'a> SlicedRenderer<'a> {
    pub fn new(representation: &'a Representation) -> Self {
        let mut planner = FftPlanner::<f64>::new();
        Self {
            representation,
            window: (0..FFT_FRAMES)
                .map(|index| {
                    (std::f64::consts::PI * (index as f64 + 0.5) / FFT_FRAMES as f64).sin()
                })
                .collect(),
            forward_full: planner.plan_fft_forward(FFT_FRAMES),
            inverse_full: planner.plan_fft_inverse(FFT_FRAMES),
            forward_band: planner.plan_fft_forward(representation.common_coefficients),
            inverse_band: planner.plan_fft_inverse(representation.common_coefficients),
        }
    }

    pub fn render(&self, input: &[f64]) -> SliceResult {
        let mut samples = vec![0.0; input.len()];
        let mut coverage = vec![0_usize; input.len()];
        let mut imaginary_residue = 0.0_f64;
        let mut conjugate_error = 0.0_f64;
        let mut non_finite_values = 0;
        let slice_count = required_slice_count(input.len());
        for slice_index in 0..slice_count {
            let start = (slice_index as isize - 1) * OUTER_ADVANCE as isize;
            let windowed = (0..FFT_FRAMES)
                .map(|local| {
                    Complex64::new(
                        reflected_sample(input, start + local as isize) * self.window[local],
                        0.0,
                    )
                })
                .collect::<Vec<_>>();
            let inner = self.reconstruct_slice(windowed);
            imaginary_residue = imaginary_residue.max(inner.imaginary_residue);
            conjugate_error = conjugate_error.max(inner.conjugate_error);
            non_finite_values += inner.non_finite_values;
            for (local, value) in inner.samples.into_iter().enumerate() {
                let logical = start + local as isize;
                if (0..input.len() as isize).contains(&logical) {
                    let output = value * self.window[local];
                    samples[logical as usize] += output.re;
                    coverage[logical as usize] += 1;
                    imaginary_residue = imaginary_residue.max(output.im.abs());
                }
            }
        }
        let mut output_hash = HASH_OFFSET;
        for sample in &samples {
            non_finite_values += usize::from(!sample.is_finite());
            hash_u64(&mut output_hash, sample.to_bits());
        }
        SliceResult {
            samples,
            slice_count,
            maximum_live_slices: coverage.iter().copied().max().unwrap_or(0),
            peak_live_coefficients: 2
                * self.representation.bands.len()
                * self.representation.common_coefficients,
            counted_operations: slice_count * per_slice_operations(self.representation),
            imaginary_residue,
            conjugate_error,
            partition_error: partition_error(&self.window),
            coverage_failures: coverage.iter().filter(|count| **count != 2).count(),
            non_finite_values,
            output_hash,
        }
    }

    fn reconstruct_slice(&self, mut spectrum: Vec<Complex64>) -> InnerResult {
        self.forward_full.process(&mut spectrum);
        let mut output = vec![Complex64::new(0.0, 0.0); FFT_FRAMES];
        let mut coefficients =
            vec![Complex64::new(0.0, 0.0); self.representation.common_coefficients];
        let coefficient_count = coefficients.len();
        for band in &self.representation.bands {
            coefficients.fill(Complex64::new(0.0, 0.0));
            for &(bin, weight) in &band.taps {
                coefficients[local_coefficient(bin, band.center, coefficient_count, FFT_FRAMES)] =
                    spectrum[bin] * weight;
            }
            self.inverse_band.process(&mut coefficients);
            let scale = 1.0 / coefficients.len() as f64;
            coefficients.iter_mut().for_each(|value| *value *= scale);
            self.forward_band.process(&mut coefficients);
            for &(bin, weight) in &band.taps {
                output[bin] += coefficients
                    [local_coefficient(bin, band.center, coefficient_count, FFT_FRAMES)]
                    * (weight / self.representation.frame_operator[bin]);
            }
        }
        close_conjugate_spectrum(&mut output);
        let conjugate_error = (0..FFT_FRAMES)
            .map(|bin| {
                let mirror = if bin == 0 { 0 } else { FFT_FRAMES - bin };
                (output[bin] - output[mirror].conj()).norm()
            })
            .fold(0.0_f64, f64::max);
        self.inverse_full.process(&mut output);
        let scale = 1.0 / FFT_FRAMES as f64;
        output.iter_mut().for_each(|value| *value *= scale);
        let imaginary_residue = output
            .iter()
            .map(|value| value.im.abs())
            .fold(0.0_f64, f64::max);
        let non_finite_values = output
            .iter()
            .map(|value| usize::from(!value.re.is_finite()) + usize::from(!value.im.is_finite()))
            .sum();
        InnerResult {
            samples: output,
            imaginary_residue,
            conjugate_error,
            non_finite_values,
        }
    }
}

fn close_conjugate_spectrum(spectrum: &mut [Complex64]) {
    spectrum[0].im = 0.0;
    spectrum[FFT_FRAMES / 2].im = 0.0;
    for bin in 1..FFT_FRAMES / 2 {
        let mirror = FFT_FRAMES - bin;
        let closed = (spectrum[bin] + spectrum[mirror].conj()) * 0.5;
        spectrum[bin] = closed;
        spectrum[mirror] = closed.conj();
    }
}

struct InnerResult {
    samples: Vec<Complex64>,
    imaginary_residue: f64,
    conjugate_error: f64,
    non_finite_values: usize,
}

pub(super) fn required_slice_count(length: usize) -> usize {
    if length == 0 {
        0
    } else {
        (length - 1) / OUTER_ADVANCE + 2
    }
}

pub(super) fn per_slice_operations(representation: &Representation) -> usize {
    // One unit is one FFT butterfly proxy or one explicit coefficient/sample visit.
    let full = FFT_FRAMES * FFT_FRAMES.ilog2() as usize;
    let band =
        representation.common_coefficients * representation.common_coefficients.ilog2() as usize;
    let taps = representation
        .bands
        .iter()
        .map(|band| band.taps.len())
        .sum::<usize>();
    let band_count = representation.bands.len();
    let coefficient_visits = 2 * band_count * representation.common_coefficients;
    let sample_visits = 4 * FFT_FRAMES;
    let conjugate_closure = FFT_FRAMES / 2 + 1;
    2 * (full + band_count * band + taps) + coefficient_visits + sample_visits + conjugate_closure
}

fn partition_error(window: &[f64]) -> f64 {
    (0..OUTER_ADVANCE)
        .map(|index| (window[index].powi(2) + window[index + OUTER_ADVANCE].powi(2) - 1.0).abs())
        .fold(0.0_f64, f64::max)
}
