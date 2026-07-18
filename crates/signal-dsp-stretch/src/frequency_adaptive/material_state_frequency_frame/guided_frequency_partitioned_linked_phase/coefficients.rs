use super::*;

pub(super) fn mirror_coefficients(
    representation: &super::super::Representation,
    positive: &[usize],
    coefficients: &mut [Vec<Vec<Complex64>>; CHANNEL_CAPACITY],
) {
    for &band in positive {
        let center = representation.bands[band].center;
        if center == 0 || center == representation.fft_frames / 2 {
            for channel in coefficients.iter_mut() {
                for value in &mut channel[band] {
                    value.im = 0.0;
                }
            }
            continue;
        }
        let mirror_center = representation.fft_frames - center;
        let mirror = representation
            .bands
            .binary_search_by_key(&mirror_center, |candidate| candidate.center)
            .expect("conjugate atom");
        for channel in coefficients.iter_mut() {
            channel[mirror] = channel[band].iter().map(Complex64::conj).collect();
        }
    }
}
