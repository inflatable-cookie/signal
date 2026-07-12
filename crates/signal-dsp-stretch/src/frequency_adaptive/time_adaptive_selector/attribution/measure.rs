use rustfft::{num_complex::Complex64, FftPlanner};

use super::super::{reflected, window, ALPHA, FFT};
use super::{FRAMES, REGIONS};

pub(super) struct Frame {
    pub center: isize,
    pub energy: f64,
    pub alpha_sum: f64,
    pub frequency_counts: [usize; REGIONS],
    pub frequency_energies: [f64; REGIONS],
    pub frequency_alpha_sums: [f64; REGIONS],
}

pub(super) fn measure(input: &[f64], length: usize, hop: usize) -> Vec<Frame> {
    let mut planner = FftPlanner::<f64>::new();
    let fft = planner.plan_fft_forward(FFT);
    let window = window(length);
    let mut result = Vec::new();
    for center in ((-(FFT as isize) / 2)..(FRAMES as isize + FFT as isize / 2)).step_by(hop) {
        let mut buffer = vec![Complex64::new(0.0, 0.0); FFT];
        let offset = (FFT - length) / 2;
        for (index, weight) in window.iter().copied().enumerate() {
            let logical = center - length as isize / 2 + index as isize;
            buffer[offset + index].re = reflected(input, logical) * weight;
        }
        fft.process(&mut buffer);
        let mut energy = 0.0;
        let mut alpha_sum = 0.0;
        let mut frequency_counts = [0; REGIONS];
        let mut frequency_energies = [0.0; REGIONS];
        let mut frequency_alpha_sums = [0.0; REGIONS];
        for (bin, coefficient) in buffer.iter().enumerate() {
            let value = coefficient.norm_sqr();
            let alpha = value.powf(ALPHA);
            let folded = bin.min(FFT - bin);
            let region = (REGIONS * folded / (FFT / 2 + 1)).min(REGIONS - 1);
            energy += value;
            alpha_sum += alpha;
            frequency_counts[region] += 1;
            frequency_energies[region] += value;
            frequency_alpha_sums[region] += alpha;
        }
        result.push(Frame {
            center,
            energy,
            alpha_sum,
            frequency_counts,
            frequency_energies,
            frequency_alpha_sums,
        });
    }
    result
}
