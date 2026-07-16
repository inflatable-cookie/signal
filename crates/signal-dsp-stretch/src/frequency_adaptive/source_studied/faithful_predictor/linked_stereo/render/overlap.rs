use rustfft::num_complex::Complex64;

use super::super::super::TransformGrid;

#[derive(Clone, Copy)]
pub(super) enum SynthesisMode {
    Real,
    Analytic,
}

pub(super) struct Overlap {
    real: [Vec<f64>; 2],
    analytic: Option<[Vec<Complex64>; 2]>,
    normalization: [Vec<f64>; 2],
}

impl Overlap {
    pub(super) fn new(target_length: usize, mode: SynthesisMode) -> Self {
        Self {
            real: std::array::from_fn(|_| vec![0.0; target_length]),
            analytic: (matches!(mode, SynthesisMode::Analytic))
                .then(|| std::array::from_fn(|_| vec![Complex64::new(0.0, 0.0); target_length])),
            normalization: std::array::from_fn(|_| vec![0.0; target_length]),
        }
    }

    pub(super) fn add_real(
        &mut self,
        channel: usize,
        output_center: isize,
        frame: &[f64],
        window: &[f64],
        transform_length: usize,
    ) {
        let target_length = self.real[channel].len() as isize;
        for offset in 0..frame.len() {
            let output_index = output_center - frame.len() as isize / 2 + offset as isize;
            if (0..target_length).contains(&output_index) {
                let output_index = output_index as usize;
                self.real[channel][output_index] +=
                    frame[offset] * window[offset] / transform_length as f64;
                self.normalization[channel][output_index] += window[offset] * window[offset];
            }
        }
    }

    pub(super) fn add_analytic(
        &mut self,
        channel: usize,
        output_center: isize,
        frame: &[Complex64],
        window: &[f64],
        transform_length: usize,
    ) {
        let output = self.analytic.as_mut().expect("analytic overlap");
        let target_length = output[channel].len() as isize;
        for offset in 0..frame.len() {
            let output_index = output_center - frame.len() as isize / 2 + offset as isize;
            if (0..target_length).contains(&output_index) {
                let output_index = output_index as usize;
                output[channel][output_index] +=
                    frame[offset] * window[offset] / transform_length as f64;
                self.normalization[channel][output_index] += window[offset] * window[offset];
            }
        }
    }

    pub(super) fn real_output(&self) -> &[Vec<f64>; 2] {
        &self.real
    }

    pub(super) fn uncovered(&self) -> usize {
        self.normalization
            .iter()
            .flat_map(|channel| channel.iter())
            .filter(|weight| **weight <= 0.0)
            .count()
    }

    pub(super) fn finish(mut self) -> [Vec<f64>; 2] {
        if let Some(analytic) = self.analytic {
            for channel in 0..2 {
                for (sample, value) in self.real[channel].iter_mut().zip(&analytic[channel]) {
                    *sample = value.re;
                }
            }
        }
        for channel in 0..2 {
            for (sample, weight) in self.real[channel]
                .iter_mut()
                .zip(&self.normalization[channel])
            {
                if *weight > 0.0 {
                    *sample /= *weight;
                }
            }
        }
        self.real
    }
}

pub(super) fn synthesise_analytic(
    bins: &[Complex64],
    support_length: usize,
    transform_length: usize,
    grid: TransformGrid,
    inverse: &std::sync::Arc<dyn rustfft::Fft<f64>>,
) -> Vec<Complex64> {
    let mut spectrum = vec![Complex64::new(0.0, 0.0); transform_length];
    spectrum[..bins.len()].copy_from_slice(bins);
    for (bin, value) in spectrum[..bins.len()].iter_mut().enumerate() {
        let edge = grid == TransformGrid::Standard && (bin == 0 || bin == transform_length / 2);
        if !edge {
            *value *= 2.0;
        }
    }
    inverse.process(&mut spectrum);
    (0..support_length)
        .map(|offset| match grid {
            TransformGrid::Standard => spectrum[offset],
            TransformGrid::ModifiedHalfBin => {
                let relative = offset as isize - support_length as isize / 2;
                let index = relative.rem_euclid(transform_length as isize) as usize;
                let phase = std::f64::consts::PI * relative as f64 / transform_length as f64;
                spectrum[index] * Complex64::from_polar(1.0, phase)
            }
        })
        .collect()
}
