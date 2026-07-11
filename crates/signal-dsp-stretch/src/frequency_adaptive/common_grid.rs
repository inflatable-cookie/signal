use rustfft::{num_complex::Complex64, FftPlanner};
use signal_primitives::{Sample, SampleRate};

use super::types::{
    StretchCommonGridTonePhaseEvidence, StretchCommonGridWaveletEvidence,
    StretchCommonGridWaveletReview,
};

pub(super) const CHANNELS: usize = 1_536;
const LOWPASS_CHANNELS: usize = 16;
pub(super) const HOP: usize = 384;
const ALPHA: f64 = 900.0;
pub(super) const HASH_OFFSET: u64 = 0xcbf2_9ce4_8422_2325;

pub(crate) fn common_grid_tone_phase_review_mono(
    input: &[Sample],
    sample_rate: SampleRate,
    expected_frequency_hz: f64,
) -> StretchCommonGridTonePhaseEvidence {
    let fft_frames = input.len().max(HOP).div_ceil(HOP) * HOP;
    let coefficient_frames = fft_frames / HOP;
    let coefficients = analyze_coefficients(input, fft_frames).0;
    let omega = std::f64::consts::TAU * expected_frequency_hz / sample_rate.0.max(1) as f64;
    let maximum = coefficients
        .iter()
        .map(|value| value.norm())
        .fold(0.0, f64::max);
    let threshold = maximum * 0.5;
    let mut frequencies = vec![f64::NAN; coefficients.len()];
    let mut max_frequency_error = 0.0_f64;
    let mut horizontal_measurements = 0;
    let mut trace_hash = HASH_OFFSET;
    for channel in 0..CHANNELS {
        let center = std::f64::consts::PI * channel as f64 / (CHANNELS - 1) as f64;
        for frame in 1..coefficient_frames.saturating_sub(1) {
            let previous = coefficients[channel * coefficient_frames + frame - 1];
            let current = coefficients[channel * coefficient_frames + frame];
            if previous.norm() < threshold || current.norm() < threshold {
                continue;
            }
            let residual = wrap_phase(current.arg() - previous.arg() - center * HOP as f64);
            let estimate = center + residual / HOP as f64;
            frequencies[channel * coefficient_frames + frame] = estimate;
            max_frequency_error = max_frequency_error.max((estimate - omega).abs());
            horizontal_measurements += 1;
            hash_u64(&mut trace_hash, estimate.to_bits());
        }
    }
    let mut shared_frequencies = vec![f64::NAN; coefficient_frames];
    for frame in 1..coefficient_frames.saturating_sub(1) {
        let mut weighted = 0.0;
        let mut weight = 0.0;
        for channel in 0..CHANNELS {
            let index = channel * coefficient_frames + frame;
            if frequencies[index].is_finite() {
                let magnitude = coefficients[index].norm();
                weighted += frequencies[index] * magnitude;
                weight += magnitude;
            }
        }
        if weight > 0.0 {
            shared_frequencies[frame] = weighted / weight;
        }
    }
    let mut max_phase_residual = 0.0_f64;
    let mut vertical_measurements = 0;
    for channel in 0..CHANNELS - 1 {
        for frame in 1..coefficient_frames.saturating_sub(1) {
            let left_index = channel * coefficient_frames + frame;
            let right_index = (channel + 1) * coefficient_frames + frame;
            let left_frequency = frequencies[left_index];
            let right_frequency = frequencies[right_index];
            if !left_frequency.is_finite() || !right_frequency.is_finite() {
                continue;
            }
            let shared_frequency = shared_frequencies[frame];
            let delay_difference =
                HOP as f64 * (digital_delay(channel + 1) - digital_delay(channel));
            let residual = wrap_phase(
                coefficients[right_index].arg()
                    - coefficients[left_index].arg()
                    - shared_frequency * delay_difference,
            )
            .abs();
            max_phase_residual = max_phase_residual.max(residual);
            vertical_measurements += 1;
            hash_u64(&mut trace_hash, residual.to_bits());
        }
    }
    StretchCommonGridTonePhaseEvidence {
        expected_angular_frequency: omega,
        max_angular_frequency_error: max_frequency_error,
        max_compensated_phase_residual: max_phase_residual,
        horizontal_measurements,
        vertical_measurements,
        all_values_finite: max_frequency_error.is_finite() && max_phase_residual.is_finite(),
        zero_energy_skips: coefficients.len() - horizontal_measurements,
        auxiliary_hash: 0,
        trace_hash,
    }
}

pub(crate) fn common_grid_derivative_tone_review_mono(
    input: &[Sample],
    sample_rate: SampleRate,
    expected_frequency_hz: f64,
) -> StretchCommonGridTonePhaseEvidence {
    let fft_frames = input.len().max(HOP).div_ceil(HOP) * HOP;
    let coefficient_frames = fft_frames / HOP;
    let (coefficients, derivatives) = analyze_coefficients(input, fft_frames);
    let expected = std::f64::consts::TAU * expected_frequency_hz / sample_rate.0.max(1) as f64;
    let maximum = coefficients
        .iter()
        .map(|value| value.norm())
        .fold(0.0, f64::max);
    let threshold = maximum * 0.5;
    let mut frequencies = vec![f64::NAN; coefficients.len()];
    let mut max_frequency_error = 0.0_f64;
    let mut horizontal_measurements = 0;
    let mut zero_energy_skips = 0;
    let mut auxiliary_hash = HASH_OFFSET;
    for index in 0..coefficients.len() {
        hash_u64(&mut auxiliary_hash, derivatives[index].re.to_bits());
        hash_u64(&mut auxiliary_hash, derivatives[index].im.to_bits());
        let energy = coefficients[index].norm_sqr();
        if energy == 0.0 || coefficients[index].norm() < threshold {
            zero_energy_skips += 1;
            continue;
        }
        let estimate = (derivatives[index] * coefficients[index].conj()).im / energy;
        frequencies[index] = estimate;
        horizontal_measurements += 1;
    }
    for frame in 0..coefficient_frames {
        let strongest = (0..CHANNELS)
            .filter_map(|channel| {
                let index = channel * coefficient_frames + frame;
                frequencies[index]
                    .is_finite()
                    .then_some((index, coefficients[index].norm_sqr()))
            })
            .max_by(|left, right| left.1.total_cmp(&right.1));
        if let Some((strongest_index, _)) = strongest {
            let shared = frequencies[strongest_index];
            max_frequency_error = max_frequency_error.max((shared - expected).abs());
            for channel in 0..CHANNELS {
                let index = channel * coefficient_frames + frame;
                if frequencies[index].is_finite() {
                    frequencies[index] = shared;
                }
            }
        }
    }
    let mut max_phase_residual = 0.0_f64;
    let mut vertical_measurements = 0;
    let mut trace_hash = HASH_OFFSET;
    for frame in 0..coefficient_frames {
        let strongest_pair = (0..CHANNELS - 1)
            .filter_map(|channel| {
                let left = channel * coefficient_frames + frame;
                let right = (channel + 1) * coefficient_frames + frame;
                if !frequencies[left].is_finite() || !frequencies[right].is_finite() {
                    return None;
                }
                Some((
                    channel,
                    coefficients[left].norm().min(coefficients[right].norm()),
                ))
            })
            .max_by(|left, right| left.1.total_cmp(&right.1));
        if let Some((channel, _)) = strongest_pair {
            let left = channel * coefficient_frames + frame;
            let right = (channel + 1) * coefficient_frames + frame;
            let shared_frequency = (frequencies[left] + frequencies[right]) * 0.5;
            let delay_difference =
                HOP as f64 * (digital_delay(channel + 1) - digital_delay(channel));
            let residual = wrap_phase(
                coefficients[right].arg()
                    - coefficients[left].arg()
                    - shared_frequency * delay_difference,
            )
            .abs();
            max_phase_residual = max_phase_residual.max(residual);
            vertical_measurements += 1;
            hash_u64(&mut trace_hash, residual.to_bits());
        }
    }
    StretchCommonGridTonePhaseEvidence {
        expected_angular_frequency: expected,
        max_angular_frequency_error: max_frequency_error,
        max_compensated_phase_residual: max_phase_residual,
        horizontal_measurements,
        vertical_measurements,
        all_values_finite: frequencies
            .iter()
            .filter(|value| !value.is_nan())
            .all(|value| value.is_finite()),
        zero_energy_skips,
        auxiliary_hash,
        trace_hash,
    }
}

pub(super) fn analyze_coefficients(
    input: &[Sample],
    fft_frames: usize,
) -> (Vec<Complex64>, Vec<Complex64>) {
    let coefficient_frames = fft_frames / HOP;
    let positive_bins = fft_frames / 2 + 1;
    let mut filters = build_filters(fft_frames);
    tighten_frequency_response(&mut filters, positive_bins);
    let mut planner = FftPlanner::<f64>::new();
    let mut spectrum = vec![Complex64::new(0.0, 0.0); fft_frames];
    for (slot, sample) in spectrum.iter_mut().zip(input) {
        slot.re = f64::from(*sample);
    }
    planner.plan_fft_forward(fft_frames).process(&mut spectrum);
    let mut coefficients = vec![Complex64::new(0.0, 0.0); CHANNELS * coefficient_frames];
    let mut derivatives = vec![Complex64::new(0.0, 0.0); CHANNELS * coefficient_frames];
    for channel in 0..CHANNELS {
        for residue in 0..coefficient_frames {
            for bin in (residue..positive_bins).step_by(coefficient_frames) {
                let contribution = spectrum[bin] * filters[channel * positive_bins + bin].conj();
                coefficients[channel * coefficient_frames + residue] += contribution;
                let angular_frequency = std::f64::consts::TAU * bin as f64 / fft_frames as f64;
                derivatives[channel * coefficient_frames + residue] +=
                    contribution * Complex64::new(0.0, angular_frequency);
            }
        }
    }
    let inverse = planner.plan_fft_inverse(coefficient_frames);
    for row in coefficients.chunks_mut(coefficient_frames) {
        inverse.process(row);
        for value in row {
            *value /= coefficient_frames as f64;
        }
    }
    for row in derivatives.chunks_mut(coefficient_frames) {
        inverse.process(row);
        for value in row {
            *value /= coefficient_frames as f64;
        }
    }
    (coefficients, derivatives)
}

pub(super) fn wrap_phase(value: f64) -> f64 {
    (value + std::f64::consts::PI).rem_euclid(std::f64::consts::TAU) - std::f64::consts::PI
}

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

pub(super) fn digital_delay(index: usize) -> f64 {
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

pub(super) fn hash_u64(hash: &mut u64, value: u64) {
    for byte in value.to_le_bytes() {
        *hash ^= u64::from(byte);
        *hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
}
