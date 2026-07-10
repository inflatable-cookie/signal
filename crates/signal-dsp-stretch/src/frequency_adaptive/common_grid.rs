use rustfft::{num_complex::Complex64, FftPlanner};
use signal_primitives::{Sample, SampleRate};

use super::types::{StretchCommonGridWaveletEvidence, StretchCommonGridWaveletReview};

const CHANNELS: usize = 1_536;
const LOWPASS_CHANNELS: usize = 16;
const HOP: usize = 384;
const ALPHA: f64 = 900.0;
const HASH_OFFSET: u64 = 0xcbf2_9ce4_8422_2325;

pub(crate) fn common_grid_wavelet_reconstruction_review_mono(
    input: &[Sample],
    _sample_rate: SampleRate,
) -> StretchCommonGridWaveletReview {
    let fft_frames = input.len().max(HOP).div_ceil(HOP) * HOP;
    let coefficient_frames = fft_frames / HOP;
    let positive_bins = fft_frames / 2 + 1;
    let mut filters = build_filters(fft_frames);
    tighten_frequency_response(&mut filters, positive_bins);
    let mut delay_hash = HASH_OFFSET;
    for channel in 0..CHANNELS {
        hash_u64(&mut delay_hash, digital_delay(channel).to_bits());
    }

    let mut planner = FftPlanner::<f64>::new();
    let mut spectrum = vec![Complex64::new(0.0, 0.0); fft_frames];
    for (slot, sample) in spectrum.iter_mut().zip(input) {
        slot.re = f64::from(*sample);
    }
    planner.plan_fft_forward(fft_frames).process(&mut spectrum);
    let mut channel_frequency = vec![Complex64::new(0.0, 0.0); CHANNELS * coefficient_frames];
    for channel in 0..CHANNELS {
        for residue in 0..coefficient_frames {
            let mut value = Complex64::new(0.0, 0.0);
            for bin in (residue..positive_bins).step_by(coefficient_frames) {
                value += spectrum[bin] * filters[channel * positive_bins + bin].conj();
            }
            channel_frequency[channel * coefficient_frames + residue] = value;
        }
    }
    let mut coefficient_hash = HASH_OFFSET;
    let inverse_coeff = planner.plan_fft_inverse(coefficient_frames);
    for row in channel_frequency.chunks_mut(coefficient_frames) {
        inverse_coeff.process(row);
        for value in row.iter_mut() {
            *value /= coefficient_frames as f64;
            hash_u64(&mut coefficient_hash, value.re.to_bits());
            hash_u64(&mut coefficient_hash, value.im.to_bits());
        }
    }
    let forward_coeff = planner.plan_fft_forward(coefficient_frames);
    for row in channel_frequency.chunks_mut(coefficient_frames) {
        forward_coeff.process(row);
    }

    let mut reconstructed = vec![Complex64::new(0.0, 0.0); fft_frames];
    let mut lower = f64::INFINITY;
    let mut upper = 0.0_f64;
    let mut max_residual = 0.0_f64;
    for residue in 0..coefficient_frames {
        let bins = (residue..positive_bins)
            .step_by(coefficient_frames)
            .collect::<Vec<_>>();
        let size = bins.len();
        let mut frame = vec![Complex64::new(0.0, 0.0); size * size];
        let mut rhs = vec![Complex64::new(0.0, 0.0); size];
        for channel in 0..CHANNELS {
            let y = channel_frequency[channel * coefficient_frames + residue];
            let active = bins
                .iter()
                .enumerate()
                .filter_map(|(index, bin)| {
                    let value = filters[channel * positive_bins + *bin];
                    (value.norm_sqr() > 0.0).then_some((index, value))
                })
                .collect::<Vec<_>>();
            for &(row, value) in &active {
                rhs[row] += value * y;
                for &(column, column_value) in &active {
                    frame[row * size + column] += value * column_value.conj();
                }
            }
        }
        let max_eigen = power_eigenvalue(&frame, size, false);
        let min_eigen = power_eigenvalue(&frame, size, true);
        lower = lower.min(min_eigen);
        upper = upper.max(max_eigen);
        let (solution, residual) = conjugate_gradient(&frame, &rhs, size);
        max_residual = max_residual.max(residual);
        for (bin, value) in bins.into_iter().zip(solution) {
            reconstructed[bin] = value;
            if bin > 0 && bin < fft_frames / 2 {
                reconstructed[fft_frames - bin] = value.conj();
            }
        }
    }
    planner
        .plan_fft_inverse(fft_frames)
        .process(&mut reconstructed);
    let samples = reconstructed[..input.len()]
        .iter()
        .map(|value| (value.re / fft_frames as f64) as f32)
        .collect::<Vec<_>>();
    let errors = input
        .iter()
        .zip(&samples)
        .map(|(a, b)| (f64::from(*a) - f64::from(*b)).abs())
        .collect::<Vec<_>>();
    let peak = errors.iter().copied().fold(0.0, f64::max);
    let rms = if errors.is_empty() {
        0.0
    } else {
        (errors.iter().map(|e| e * e).sum::<f64>() / errors.len() as f64).sqrt()
    };
    let non_finite_values = channel_frequency
        .iter()
        .filter(|value| !value.re.is_finite() || !value.im.is_finite())
        .count()
        + samples.iter().filter(|value| !value.is_finite()).count();
    StretchCommonGridWaveletReview {
        evidence: StretchCommonGridWaveletEvidence {
            channel_count: CHANNELS,
            lowpass_channel_count: LOWPASS_CHANNELS,
            hop_frames: HOP,
            redundancy: 2.0 * CHANNELS as f64 / HOP as f64,
            delay_hash,
            frame_bound_min: lower,
            frame_bound_max: upper,
            frame_condition_ratio: upper / lower,
            canonical_dual_residual: max_residual,
            analysis_coefficient_count: CHANNELS * coefficient_frames,
            synthesis_coefficient_count: CHANNELS * coefficient_frames,
            reconstruction_peak_error: peak,
            reconstruction_rms_error: rms,
            reconstruction_head_error: errors.first().copied().unwrap_or(0.0),
            reconstruction_tail_error: errors.last().copied().unwrap_or(0.0),
            non_finite_values,
            source_hash: sample_hash(input),
            output_hash: sample_hash(&samples),
            coefficient_hash,
        },
        samples,
    }
}

fn build_filters(fft_frames: usize) -> Vec<Complex64> {
    let positive_bins = fft_frames / 2 + 1;
    let spacing = 0.5 / (CHANNELS - 1) as f64;
    let base_center = LOWPASS_CHANNELS as f64 * spacing;
    let order = (ALPHA - 1.0) * 0.5;
    let mut filters = vec![Complex64::new(0.0, 0.0); CHANNELS * positive_bins];
    for channel in 0..CHANNELS {
        let center = channel as f64 * spacing;
        let delay = HOP as f64 * digital_delay(channel);
        for bin in 0..positive_bins {
            let frequency = bin as f64 / fft_frames as f64;
            let ratio = if channel < LOWPASS_CHANNELS {
                (frequency - center + base_center) / base_center
            } else {
                frequency / center
            };
            if ratio <= 0.0 {
                continue;
            }
            let log_magnitude = order * (ratio.ln() + 1.0 - ratio);
            if log_magnitude < -40.0 {
                continue;
            }
            let dilation = if channel < LOWPASS_CHANNELS {
                1.0
            } else {
                (base_center / center).sqrt()
            };
            let magnitude = dilation * log_magnitude.exp();
            let phase = -std::f64::consts::TAU * bin as f64 * delay / fft_frames as f64;
            filters[channel * positive_bins + bin] = Complex64::from_polar(magnitude, phase);
        }
    }
    filters
}

fn tighten_frequency_response(filters: &mut [Complex64], positive_bins: usize) {
    for bin in 0..positive_bins {
        let energy = (0..CHANNELS)
            .map(|channel| filters[channel * positive_bins + bin].norm_sqr())
            .sum::<f64>();
        let scale = energy.sqrt();
        for channel in 0..CHANNELS {
            filters[channel * positive_bins + bin] /= scale;
        }
    }
}

fn digital_delay(index: usize) -> f64 {
    (0..usize::BITS as usize)
        .map(|bit| {
            let current = (index >> bit) & 1;
            let previous = if bit == 0 {
                0
            } else {
                (index >> (bit - 1)) & 1
            };
            (current ^ previous) as f64 / 2.0_f64.powi(bit as i32 + 1)
        })
        .sum()
}

fn multiply(matrix: &[Complex64], vector: &[Complex64], size: usize) -> Vec<Complex64> {
    (0..size)
        .map(|row| {
            (0..size)
                .map(|column| matrix[row * size + column] * vector[column])
                .sum()
        })
        .collect()
}

fn conjugate_gradient(
    matrix: &[Complex64],
    rhs: &[Complex64],
    size: usize,
) -> (Vec<Complex64>, f64) {
    let mut x = vec![Complex64::new(0.0, 0.0); size];
    if rhs.iter().all(|value| value.norm_sqr() == 0.0) {
        return (x, 0.0);
    }
    let mut residual = rhs.to_vec();
    let mut direction = residual.clone();
    let rhs_norm = dot(rhs, rhs).re.sqrt().max(f64::MIN_POSITIVE);
    let mut squared = dot(&residual, &residual).re;
    for _ in 0..64 {
        let product = multiply(matrix, &direction, size);
        let alpha = squared / dot(&direction, &product).re;
        for index in 0..size {
            x[index] += direction[index] * alpha;
            residual[index] -= product[index] * alpha;
        }
        let next = dot(&residual, &residual).re;
        if next.sqrt() / rhs_norm <= 1.0e-10 {
            squared = next;
            break;
        }
        let beta = next / squared;
        for index in 0..size {
            direction[index] = residual[index] + direction[index] * beta;
        }
        squared = next;
    }
    (x, squared.sqrt() / rhs_norm)
}

fn power_eigenvalue(matrix: &[Complex64], size: usize, inverse: bool) -> f64 {
    let mut vector = vec![Complex64::new(1.0 / (size as f64).sqrt(), 0.0); size];
    for _ in 0..16 {
        let next = if inverse {
            conjugate_gradient(matrix, &vector, size).0
        } else {
            multiply(matrix, &vector, size)
        };
        let norm = dot(&next, &next).re.sqrt();
        vector = next.into_iter().map(|value| value / norm).collect();
    }
    let rayleigh = dot(&vector, &multiply(matrix, &vector, size)).re;
    rayleigh
}

fn dot(left: &[Complex64], right: &[Complex64]) -> Complex64 {
    left.iter().zip(right).map(|(a, b)| a.conj() * b).sum()
}

fn sample_hash(samples: &[Sample]) -> u64 {
    let mut hash = HASH_OFFSET;
    for sample in samples {
        hash_u64(&mut hash, u64::from(sample.to_bits()));
    }
    hash
}

fn hash_u64(hash: &mut u64, value: u64) {
    for byte in value.to_le_bytes() {
        *hash ^= u64::from(byte);
        *hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
}
