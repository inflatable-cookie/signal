use super::*;

pub(super) fn synthesise(
    representation: &Representation,
    coefficients: [Vec<Vec<Complex64>>; 2],
    target_length: usize,
) -> ([Vec<f64>; 2], usize) {
    let mut planner = FftPlanner::<f64>::new();
    let forward_band = planner.plan_fft_forward(representation.common_coefficients);
    let inverse = planner.plan_fft_inverse(representation.fft_frames);
    let mut non_finite = 0;
    let channels = std::array::from_fn(|channel| {
        let mut spectrum = vec![Complex64::default(); representation.fft_frames];
        for (band, mut values) in representation
            .bands
            .iter()
            .zip(coefficients[channel].clone())
        {
            forward_band.process(&mut values);
            for &(bin, weight) in &band.taps {
                let local = local_coefficient(
                    bin,
                    band.center,
                    representation.common_coefficients,
                    representation.fft_frames,
                );
                spectrum[bin] += values[local] * weight / representation.frame_operator[bin];
            }
        }
        inverse.process(&mut spectrum);
        let scale = 1.0 / representation.fft_frames as f64;
        spectrum[PAD_FRAMES..PAD_FRAMES + target_length]
            .iter()
            .map(|sample| {
                let sample = *sample * scale;
                non_finite +=
                    usize::from(!sample.re.is_finite()) + usize::from(!sample.im.is_finite());
                sample.re
            })
            .collect::<Vec<_>>()
    });
    (channels, non_finite)
}
