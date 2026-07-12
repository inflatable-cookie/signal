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
    pub low_counts: [usize; REGIONS],
    pub low_energies: [f64; REGIONS],
    pub low_alpha_sums: [f64; REGIONS],
    pub complement_count: usize,
    pub complement_energy: f64,
    pub complement_alpha_sum: f64,
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
        let mut low_counts = [0; REGIONS];
        let mut low_energies = [0.0; REGIONS];
        let mut low_alpha_sums = [0.0; REGIONS];
        let mut complement_count = 0;
        let mut complement_energy = 0.0;
        let mut complement_alpha_sum = 0.0;
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
            if folded <= 256 {
                let low_region = (REGIONS * folded / 257).min(REGIONS - 1);
                low_counts[low_region] += 1;
                low_energies[low_region] += value;
                low_alpha_sums[low_region] += alpha;
            } else {
                complement_count += 1;
                complement_energy += value;
                complement_alpha_sum += alpha;
            }
        }
        result.push(Frame {
            center,
            energy,
            alpha_sum,
            frequency_counts,
            frequency_energies,
            frequency_alpha_sums,
            low_counts,
            low_energies,
            low_alpha_sums,
            complement_count,
            complement_energy,
            complement_alpha_sum,
        });
    }
    result
}
