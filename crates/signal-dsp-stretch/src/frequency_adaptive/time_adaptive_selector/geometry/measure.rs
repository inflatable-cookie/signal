use rustfft::{num_complex::Complex64, FftPlanner};

use super::super::{reflected, window, ALPHA, ANCHOR_HOP, FFT};

pub(super) struct Frame {
    pub center: isize,
    pub energy: f64,
    pub alpha_sum: f64,
}

pub(super) fn measure(
    channels: &[&[f64]],
    length: usize,
    first_center: isize,
    last_center: isize,
) -> (Vec<Frame>, usize, f64, usize) {
    let mut planner = FftPlanner::<f64>::new();
    let fft = planner.plan_fft_forward(FFT);
    let window = window(length);
    let mut result = Vec::new();
    let mut reflected_reads = 0;
    let mut closure = 0.0_f64;
    let mut invalid = 0;
    for center in (first_center..=last_center).step_by(ANCHOR_HOP) {
        let mut spectra = Vec::with_capacity(channels.len());
        for channel in channels {
            let mut buffer = vec![Complex64::new(0.0, 0.0); FFT];
            let offset = (FFT - length) / 2;
            for (index, weight) in window.iter().copied().enumerate() {
                let logical = center - length as isize / 2 + index as isize;
                reflected_reads += usize::from(logical < 0 || logical >= channel.len() as isize);
                buffer[offset + index].re = reflected(channel, logical) * weight;
            }
            fft.process(&mut buffer);
            spectra.push(buffer);
        }
        let mut energy = 0.0;
        let mut alpha_sum = 0.0;
        for bin in 0..FFT {
            let combined = spectra
                .iter()
                .map(|spectrum| spectrum[bin].norm_sqr())
                .sum::<f64>();
            let separate = spectra
                .iter()
                .map(|spectrum| spectrum[bin].norm_sqr())
                .sum::<f64>();
            closure = closure.max((combined - separate).abs() / combined.max(f64::MIN_POSITIVE));
            energy += combined;
            alpha_sum += combined.powf(ALPHA);
            invalid += usize::from(!combined.is_finite());
        }
        result.push(Frame {
            center,
            energy,
            alpha_sum,
        });
    }
    (result, reflected_reads, closure, invalid)
}
