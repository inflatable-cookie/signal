use rustfft::num_complex::Complex32;

pub(crate) const FRAME_TAG: u64 = 0x454d_4152_4657_4e52;
pub(crate) const BIN_TAG: u64 = 0x3030_4e49_4257_4e52;
pub(crate) const BASE_TAG: u64 = 0x3045_5341_4257_4e52;
pub(crate) const SPACE_TAG: u64 = 0x4543_4150_5357_4e52;
#[cfg(test)]
pub(crate) const TEST_TAG: u64 = 0x3054_5345_5457_4e52;
pub(crate) const ADMISSION_SEED: u64 = 0x0123_4567_89ab_cdef;

pub(crate) fn mix64(mut value: u64) -> u64 {
    value = (value ^ (value >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    value = (value ^ (value >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    value ^ (value >> 31)
}

pub(crate) fn address(seed: u64, frame: usize, bin: usize, stream: u64) -> u64 {
    mix64(
        seed ^ mix64(frame as u64 ^ FRAME_TAG)
            ^ mix64(bin as u64 ^ BIN_TAG).rotate_left(21)
            ^ stream,
    )
}

pub(crate) fn high_53(value: u64) -> u64 {
    value >> 11
}

pub(crate) fn phase(value: u64) -> f64 {
    let unit = high_53(value) as f64 / (1_u64 << 53) as f64;
    std::f64::consts::TAU * unit - std::f64::consts::PI
}

pub(crate) fn frequency_weight(frequency_hz: f64) -> f64 {
    if frequency_hz <= 250.0 {
        0.0
    } else if frequency_hz >= 1_500.0 {
        1.0
    } else {
        let t = (frequency_hz - 250.0) / 1_250.0;
        t * t * (3.0 - 2.0 * t)
    }
}

fn rotation(phase: f64) -> Complex32 {
    let (sine, cosine) = phase.sin_cos();
    Complex32::new(cosine as f32, sine as f32)
}

pub(crate) fn renew_mono(spectrum: &mut [Complex32], frame: usize, seed: u64) {
    let fft_size = spectrum.len();
    let nyquist = fft_size / 2;
    spectrum[0] = Complex32::new(spectrum[0].re, 0.0);
    spectrum[nyquist] = Complex32::new(spectrum[nyquist].re, 0.0);
    for bin in 1..nyquist {
        let source = spectrum[bin];
        let renewed = if source.re == 0.0 && source.im == 0.0 {
            Complex32::new(0.0, 0.0)
        } else {
            source * rotation(phase(address(seed, frame, bin, BASE_TAG)))
        };
        spectrum[bin] = renewed;
        spectrum[fft_size - bin] = renewed.conj();
    }
}

pub(crate) fn renew_stereo(
    left: &mut [Complex32],
    right: &mut [Complex32],
    frame: usize,
    seed: u64,
    space: f32,
    sample_rate: u32,
) {
    let fft_size = left.len();
    let nyquist = fft_size / 2;
    left[0] = Complex32::new(left[0].re, 0.0);
    right[0] = Complex32::new(right[0].re, 0.0);
    left[nyquist] = Complex32::new(left[nyquist].re, 0.0);
    right[nyquist] = Complex32::new(right[nyquist].re, 0.0);

    for bin in 1..nyquist {
        let theta = phase(address(seed, frame, bin, BASE_TAG));
        let zeta = phase(address(seed, frame, bin, SPACE_TAG));
        let frequency = bin as f64 * sample_rate as f64 / fft_size as f64;
        let delta = 0.5 * space as f64 * frequency_weight(frequency) * zeta;
        let source_left = left[bin];
        let source_right = right[bin];
        let renewed_left = if source_left.re == 0.0 && source_left.im == 0.0 {
            Complex32::new(0.0, 0.0)
        } else {
            source_left * rotation(theta - delta)
        };
        let renewed_right = if source_right.re == 0.0 && source_right.im == 0.0 {
            Complex32::new(0.0, 0.0)
        } else {
            source_right * rotation(theta + delta)
        };
        left[bin] = renewed_left;
        right[bin] = renewed_right;
        left[fft_size - bin] = renewed_left.conj();
        right[fft_size - bin] = renewed_right.conj();
    }
}
