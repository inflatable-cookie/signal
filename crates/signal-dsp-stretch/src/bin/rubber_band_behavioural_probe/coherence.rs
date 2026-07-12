use rustfft::{num_complex::Complex32, FftPlanner};

pub(super) fn vertical_phase_coherence(samples: &[f32]) -> f64 {
    const FFT: usize = 1_024;
    const HOP: usize = 256;
    if samples.len() < FFT {
        return f64::NAN;
    }
    let mut planner = FftPlanner::new();
    let fft = planner.plan_fft_forward(FFT);
    let mut sum = Complex32::new(0.0, 0.0);
    let mut count = 0;
    for start in (0..=samples.len() - FFT).step_by(HOP) {
        let mut buffer = (0..FFT)
            .map(|index| {
                let window = 0.5 - 0.5 * (std::f32::consts::TAU * index as f32 / FFT as f32).cos();
                Complex32::new(samples[start + index] * window, 0.0)
            })
            .collect::<Vec<_>>();
        fft.process(&mut buffer);
        for bin in 2..FFT / 2 {
            if buffer[bin].norm() > 1.0e-6 && buffer[bin - 1].norm() > 1.0e-6 {
                let relation = buffer[bin] * buffer[bin - 1].conj();
                sum += relation / relation.norm().max(1.0e-12);
                count += 1;
            }
        }
    }
    f64::from(sum.norm()) / count.max(1) as f64
}
